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
> Provide the diff and code content as context. Return the report in this format:
>
> ```
> ## Temper Report
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
cd qictrader-backend-rs && grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]'

# If TypeScript was touched:
cd frontend && bun run build 2>&1 | tail -30
```

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

### Intent Alignment: ALIGNED / DRIFT DETECTED
### Migration Compliance: COMPLIANT / MISSING / N/A
### Build: ✅ passes / ❌ FAILED

VERDICT: APPROVED / NEEDS_FIXES / BLOCKED
```

- **APPROVED** — zero HIGH/MEDIUM remaining, build passes
- **NEEDS_FIXES** — HIGH/MEDIUM were found and fixed; recommend review before shipping
- **BLOCKED** — unfixable issue (e.g. missing migration, fundamental design conflict) — describe what needs human decision

$ARGUMENTS
