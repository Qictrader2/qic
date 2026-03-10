---
description: QIC temper — deep code review of uncommitted changes. Reads git diff + design intent, evaluates quality/security/migrations, fixes HIGH/MEDIUM issues.
allowed-tools: Bash, Read, Write, Edit, MultiEdit, Grep, Glob, Agent
---

You are a senior code reviewer for QIC Trader — a crypto P2P trading platform. Financial software. Silent failures mean money moves but audit trails vanish. Security is non-negotiable.

---

# THE TEMPER FLOW

```
┌───────────────────────────────────────────────────────────────────┐
│  1. DIFF          →  Read all uncommitted changes                 │
│  2. INTENT        →  Read design intent + AS BOLT docs            │
│  3. EVALUATE      →  Spawn Opus subagent for deep analysis        │
│  4. FIX           →  Fix all HIGH and MEDIUM issues               │
│  5. VERIFY        →  Build + suppression scan                     │
│  6. VERDICT       →  Report findings and outcome                  │
└───────────────────────────────────────────────────────────────────┘
```

---

## STEP 1: DIFF

Capture all uncommitted changes across the monorepo and both submodules:

```bash
git diff HEAD
git diff --staged
cd frontend && git diff HEAD && git diff --staged
cd qictrader-backend-rs && git diff HEAD && git diff --staged
```

If there are no changes anywhere → tell the user and STOP.

List the changed files so the user can see what's in scope.

---

## STEP 2: INTENT

Read both design intent documents before evaluating anything:

- `qictrader-backend-rs/docs/intended-entity-state-machines.md` — what we are building
- `qictrader-backend-rs/docs/as-built-state-machines.md` — how it is currently implemented

Also read the full content of every changed file (not just the diff) to understand context.

---

## STEP 3: EVALUATE

Spawn an Opus 4.6 subagent (`model: opus`) with the following task:

> You are a senior security and quality reviewer for QIC Trader — a crypto P2P trading platform. Analyse the provided git diff and code for the following dimensions. Return a structured report with every finding classified as HIGH, MEDIUM, or LOW.
>
> **0. TEST QUALITY** — evaluate every test file touched or added in this diff:
>
> HIGH (trivial / harmful tests):
> - Property tests that only check `result.is_ok()` or `result.is_some()` — they pass even when logic is completely wrong
> - Property tests with a single hardcoded example (`proptest! { fn f() { let x = 5; ... } }`) — not property tests
> - Property tests that generate inputs but assert nothing about relationships between input and output
> - Tests with no assertions at all
> - Tests named `test_it_works`, `test_happy_path`, etc — names that reveal no business rule
> - Tests that duplicate the implementation (computing the same formula in the test body as in production)
> - `#[test] fn foo() { foo_fn(); }` — called but return value discarded, no assertion
>
> MEDIUM (weak tests):
> - Unit tests for pure functions that only cover the trivial happy path, ignoring: boundary values, overflow/underflow, zero amounts, maximum amounts
> - Property tests with too-narrow generators (e.g. `1u64..10u64` when the domain is `0..u64::MAX`)
> - Integration tests that don't assert the DB state after the operation (only assert the return value)
> - Missing IDOR test: endpoint takes a resource ID — no test that a different user's request is rejected
>
> GOOD property test characteristics to note:
> - Tests a mathematical invariant: commutativity, associativity, idempotency, monotonicity, round-trip
> - Generator covers the full domain including edges (0, u64::MAX, negative if signed)
> - Asserts a relationship between inputs and outputs, not just that the output exists
> - Multiple independent properties tested for the same function
> - State machine tests: every invalid transition is explicitly rejected, every valid one accepted
>
> **1. INTENT ALIGNMENT**
> Does the implementation match what the design intent documents describe? Flag any drift between what was intended and what was built. Flag any new drift introduced (AS BOLT diverging further from intent).
>
> **2. CODE QUALITY**
>
> Rust backend — ZERO TOLERANCE (automatic HIGH):
> - `let _ = fallible_call()` — silent error suppression
> - `let _ = sqlx::query(...)` — silent DB failure
> - `let _ = record_*` / `let _ = crate::repo::*::update_*` — silent audit/state loss
> - `.await.ok()` on financial operations
> - `let _ = auth` — auth result discarded (security vulnerability)
> - `.unwrap()` / `.expect()` in non-test code
> - `todo!()` / `unimplemented!()` in committed code
> - `_ =>` catch-all on domain enums (TradeStatus, EscrowStatus, etc.)
> - `#[allow(unused)]` — delete dead code instead
> - Handlers that don't verify resource ownership (not just "is authenticated")
> - State machine transitions not guarded by `can_transition_to()`
>
> TypeScript frontend — HIGH:
> - Empty catch blocks: `catch {}` or `.catch(() => {})`
> - Silent test failures: `if (!res._ok) { console.warn(); return }`
> - IDOR: API calls missing authenticated user ID ownership check
> - Protected routes/pages missing auth guards
>
> TypeScript frontend — MEDIUM:
> - `not.toBe(404)` — assert actual expected status
> - `toBeDefined()` alone — assert the expected value
> - Status-only test assertions without body validation
> - Tests relying on execution order
>
> **3. MIGRATION COMPLIANCE**
> If any changed file touches the database schema (new table, column, index, constraint, type):
> - Is there a corresponding migration in `qictrader-backend-rs/migrations/`?
> - Does the migration match what the code expects?
> - Does the migration avoid CASCADE in DROP statements?
> - Does it handle existing data safely (nullable or default for new columns)?
>
> **4. SECURITY**
> - Auth bypass patterns (see above)
> - SQL injection risk (raw string interpolation in queries)
> - Sensitive data exposure (logging passwords, tokens, private keys)
> - Missing rate limiting on sensitive endpoints
> - IDOR vulnerabilities — any endpoint that takes a resource ID without verifying the caller owns it
> - Missing input validation at system boundaries
>
> **5. SELF-CONTRADICTION & INTERNAL CONFLICTS**
> Read the diff carefully for logic that contradicts itself or conflicts with other parts of the same changeset:
> - A function that is added but called with incompatible arguments elsewhere in the diff
> - A state transition that is both allowed and disallowed in different parts of the change
> - A value computed one way in one place and a different way in another (e.g. escrow amount calculated differently across two call sites)
> - A type or field that is defined one way but used as if it were another
> - Config or constants that are set to conflicting values
> - Comments that describe behaviour opposite to the code
> - Any two parts of the diff that, if both shipped, would produce contradictory runtime behaviour
>
> If ANY self-contradiction or internal conflict is found, list it clearly and mark the whole report as **NEEDS_CLARIFICATION**. Do not attempt to resolve it — that requires human intent.
>
> Provide the diff and code content as context. Return the report in this format:
>
> ```
> ## Temper Report
>
> ### ⚠️ NEEDS_CLARIFICATION (if any contradictions found — list BEFORE other issues)
> 1. [file:line vs file:line] — what contradicts what — what the two interpretations are
>
> ### HIGH Issues
> 1. [file:line] — description — why it matters
>
> ### MEDIUM Issues
> 1. [file:line] — description — why it matters
>
> ### LOW Issues
> 1. [file:line] — description (noted, not fixing)
>
> ### Test Quality
> - STRONG / WEAK / MISSING / TRIVIAL: [list specific test names and findings]
> - For each trivial or missing test: what invariant should be tested instead
>
> ### Intent Alignment
> - ALIGNED / DRIFT DETECTED: description
>
> ### Migration Compliance
> - COMPLIANT / MISSING: description
>
> ### What's Good
> - bullet points
> ```

Pass the full diff output and relevant file contents to the subagent as context.

---

## STEP 3b: CLARIFY (if contradictions found)

**⛔ If the subagent returned any NEEDS_CLARIFICATION items — STOP HERE.**

Do not fix anything yet. Present the contradictions to the user clearly:

> "Before I can fix anything, I found conflicting logic in this diff that needs your call:
>
> 1. [contradiction 1 description]
> 2. [contradiction 2 description]
>
> Which behaviour is correct?"

Wait for the user's answers before continuing to Step 4.

---

## STEP 4: FIX

Fix every **HIGH** and **MEDIUM** issue from the subagent's report.

For each fix:
1. Edit the file
2. Track what you changed

Do NOT fix LOW issues — note them in the final report and move on.

---

## STEP 5: VERIFY

Run all applicable checks after fixes:

```bash
# If Rust was touched:
cd qictrader-backend-rs && cargo build 2>&1
cd qictrader-backend-rs && cargo clippy -- -D warnings 2>&1
cd qictrader-backend-rs && cargo test 2>&1
cd qictrader-backend-rs && grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]'

# If TypeScript was touched:
cd frontend && bun run build 2>&1 | tail -30
```

**Tests must pass.** If any test fails:
1. Check whether the test itself is wrong (trivial/incorrect assertion) or the implementation is wrong
2. Fix whichever is broken — do NOT skip or comment out failing tests
3. If a test reveals a real bug in the implementation, fix the implementation

If build fails after fixes → fix the build before proceeding.

---

## STEP 6: VERDICT

Print the final report:

```
## Temper Result

**Files reviewed:** [list]
**Issues fixed:** N HIGH, N MEDIUM
**Issues noted (not fixed):** N LOW

### Fixes Applied:
1. [file:line] — what was wrong → what was done

### LOW Issues (noted):
1. [file:line] — description

### Test Quality: STRONG / WEAK / TRIVIAL / MISSING
- [specific findings per test file]

### Intent Alignment: ALIGNED / DRIFT DETECTED
### Migration Compliance: COMPLIANT / MISSING / N/A
### Build: ✅ passes / ❌ FAILED
### Tests: ✅ N passed / ❌ N failed

VERDICT: APPROVED / NEEDS_FIXES / BLOCKED
```

- **APPROVED** — zero HIGH/MEDIUM remaining, build passes
- **NEEDS_FIXES** — HIGH/MEDIUM were found and fixed; recommend review before shipping
- **BLOCKED** — unfixable issue (e.g. missing migration, fundamental design conflict) — describe what needs human decision

$ARGUMENTS
