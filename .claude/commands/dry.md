---
description: QIC dry - DRY refactoring across frontend and backend. Finds real duplication, consolidates it safely, and verifies tests still pass.
allowed-tools: Agent, Bash, Read, Write, Edit, Grep, Glob, LSP
---

You are performing a DRY (Don't Repeat Yourself) refactoring pass across the QIC Trader codebase. Your job is to find genuine knowledge duplication, consolidate it into clean abstractions, and verify nothing breaks.

Arguments: `$ARGUMENTS`

If $ARGUMENTS specifies a scope (e.g. "backend", "frontend", "services/escrow", "hooks"), limit the refactoring to that scope. Otherwise, refactor both frontend and backend.

---

# THE DRY FLOW

```
┌──────────────────────────────────────────────────────────────────────┐
│  1. BASELINE       →  Run all tests, record the green state          │
│  2. SCAN           →  Find duplication candidates                    │
│  3. CLASSIFY       →  Real knowledge duplication vs. incidental      │
│  4. PLAN           →  Rank candidates, propose extractions           │
│  5. REFACTOR       →  Extract one at a time, verify after each       │
│  6. FINAL VERIFY   →  Full test suite + build + clippy/lint          │
│  7. REPORT         →  Summary of what changed and why                │
└──────────────────────────────────────────────────────────────────────┘
```

---

# FOUNDATIONAL PRINCIPLES

These principles govern every decision you make. Violating any of them is grounds to revert.

## DRY is about KNOWLEDGE, not CODE

Two identical-looking code blocks that represent DIFFERENT domain concepts are NOT duplication. They are incidental similarity. Merging them creates coupling between unrelated concerns.

Two different-looking code blocks that encode the SAME business rule ARE duplication, even if the syntax differs.

The test: "If requirement X changes, do BOTH of these change for the SAME reason?" If yes, real duplication. If no, incidental - leave it alone.

## Rule of Three

- First occurrence: just write it.
- Second occurrence: notice it, tolerate it.
- Third occurrence: now you have enough evidence. Refactor.

Do NOT extract an abstraction from only two occurrences unless the duplication is exact and clearly encodes the same knowledge.

## "Duplication is far cheaper than the wrong abstraction" (Sandi Metz)

A wrong abstraction accumulates complexity as developers add parameters, conditionals, and special cases. When in doubt, leave the duplication. Three similar lines of code is better than a premature abstraction.

## Tests must pass WITHOUT modification

If your refactoring requires changing test assertions, you have changed behavior, not just structure. Revert and try a smaller step. The only acceptable test changes are:
- Updating import paths after a move
- Removing tests for deleted dead code

If a test was testing internal structure (not behavior) and breaks, that is still a signal to pause and verify you haven't changed semantics.

---

# STEP 1: BASELINE

Run the full test suite and record the results. This is your safety net.

**Backend:**
```bash
cd qictrader-backend-rs && cargo test 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -5
```

**Frontend:**
```bash
cd frontend && bun run build 2>&1 | tail -10
```

If anything fails BEFORE you start, STOP. Tell the user:
> "Tests/build are already failing before refactoring. Fix these first, then re-run /dry."

Record the test count and pass/fail status. You will compare against this at the end.

---

# STEP 2: SCAN

Search for duplication candidates. Use these strategies:

## Backend (Rust)

1. **Near-identical functions** - functions with the same structure, differing only in type names or field names:
```bash
# Find functions with similar bodies
grep -rn 'pub fn\|pub async fn' qictrader-backend-rs/src/ --include='*.rs'
```

2. **Repeated query patterns** - SQL queries in `repo/` that share the same shape:
```bash
grep -rn 'sqlx::query' qictrader-backend-rs/src/repo/ --include='*.rs'
```

3. **Repeated error handling patterns** - identical match arms or error conversions

4. **Repeated validation logic** - same checks appearing in multiple handlers

5. **Repeated struct transformations** - identical From/Into patterns

Use LSP `workspaceSymbol` to understand type hierarchies. Use Grep to find patterns.

## Frontend (TypeScript/React)

1. **Near-identical components** - components that render the same structure with minor prop differences

2. **Repeated hooks patterns** - identical useState + useEffect combinations across components

3. **Repeated API call patterns** - identical fetch/mutation patterns with different endpoints

4. **Repeated type definitions** - identical or near-identical interfaces/types in different files

5. **Repeated utility logic** - identical transformations, formatters, validators

```bash
# Find components with similar structure
grep -rn 'export function\|export const.*=' frontend/src/ --include='*.tsx' --include='*.ts'
```

---

# STEP 3: CLASSIFY

For each candidate found in Step 2, answer ALL of these questions:

```
[ ] Do the duplicated pieces change for the SAME reason?
[ ] Do they change at the SAME rate?
[ ] Do they belong to the SAME bounded context/domain?
[ ] Can you name the abstraction after what it DOES, not where it's used?
[ ] Will the abstraction reduce total complexity, not just line count?
[ ] Does this appear 3+ times (Rule of Three)?
```

If any answer is "no" or "unsure", mark it as INCIDENTAL and skip it.

### Things that are NEVER worth DRY-ing

- Two SQL queries that happen to look similar but serve different endpoints
- Two handlers with similar structure but different authorization rules
- Two forms with overlapping fields (forms evolve independently)
- Similar API call hooks for different endpoints (cache keys, error handling, transforms diverge)
- Test setup code (test clarity > DRY)
- Code that crosses deployment boundaries (frontend/backend sharing creates coupling)

### Things that ARE worth DRY-ing

- Identical validation rules applied in 3+ places (same business rule)
- Identical currency/amount formatting in 3+ places (same knowledge)
- Identical error response construction in 3+ handlers (same pattern)
- Identical state machine transition checks duplicated across services (same domain logic)
- Identical type definitions that represent the same entity

---

# STEP 4: PLAN

For each candidate classified as REAL duplication, plan the extraction:

1. **Name the abstraction** - it must describe WHAT it does, not WHERE it's used. If you can only name it "shared_utils" or "common_helpers", the abstraction is suspect.

2. **Choose the right mechanism:**

   **Rust (prefer simpler mechanisms first):**
   1. Extract a pure function
   2. Extract a module (group related functions)
   3. Use generics with trait bounds
   4. Define a trait for shared behavior
   5. Use `macro_rules!` for syntactic patterns only
   6. Proc macros - last resort, avoid if possible

   **TypeScript/React (prefer simpler mechanisms first):**
   1. Extract a utility function (pure, no React)
   2. Extract a custom hook (stateful logic reuse)
   3. Extract a component (UI reuse)
   4. Use composition (children, render props)
   5. Use TypeScript generics/utility types (Pick, Omit, Partial)

3. **Estimate blast radius** - how many files change? Smaller is better. If a single extraction touches more than 8 files, consider breaking it into smaller steps.

4. **Order by risk** - do the safest, most mechanical extractions first.

Present the plan to the user before proceeding:
> "Found N candidates for DRY refactoring. Here's what I'd extract: [list]. Proceed?"

Wait for confirmation before Step 5.

---

# STEP 5: REFACTOR

Apply each extraction one at a time. After EACH extraction:

### 5a. Make the change

Follow Kent Beck: "Make the change easy, then make the easy change."

Each extraction is a sequence of mechanical steps:
1. Create the new function/hook/component/trait (no behavior change)
2. Replace the FIRST call site to use the new abstraction (no behavior change)
3. Run tests - if they pass, continue. If they fail, REVERT this step.
4. Replace the SECOND call site (no behavior change)
5. Run tests again
6. Replace remaining call sites one at a time, testing after each
7. Delete the old duplicated code
8. Run tests one final time

### 5b. Verify after each extraction

**Backend:**
```bash
cd qictrader-backend-rs && cargo check 2>&1 | tail -5
cargo test 2>&1 | tail -5
```

**Frontend:**
```bash
cd frontend && bun run build 2>&1 | tail -10
```

### 5c. Revert if broken

If tests fail after an extraction and you cannot fix it within one small adjustment:
```bash
git checkout -- .  # revert the current extraction
```

Then move on to the next candidate. Do NOT force an extraction that breaks tests.

### 5d. Anti-patterns to watch for

After each extraction, check:

- **Parameter explosion** - did the new function/component gain more than 2 parameters to accommodate different callers? If yes, the abstraction is wrong. Revert.
- **Conditional branches per caller** - does the new code have if/else or match arms that serve specific callers? If yes, the abstraction is wrong. Revert.
- **Utility junk drawer** - are you putting unrelated functions into "utils.rs" or "helpers.ts"? Each utility belongs near its domain or in a purpose-named module.
- **Coupling across boundaries** - are you sharing code between frontend and backend? Don't. Duplication across deployment boundaries is preferable to coupling.

---

# STEP 6: FINAL VERIFY

Run the complete verification suite:

**Backend:**
```bash
cd qictrader-backend-rs
cargo build 2>&1 | tail -10
cargo clippy -- -D warnings 2>&1 | tail -10
cargo test 2>&1 | tail -10
grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]'
```

**Frontend:**
```bash
cd frontend
bun run build 2>&1 | tail -10
```

Compare test counts against the baseline from Step 1:
- Same number of tests passing = GOOD
- Fewer tests passing = you broke something, investigate
- More tests passing = suspicious, did you accidentally skip failing tests?

If anything fails, identify which extraction caused it and revert that specific change.

---

# STEP 7: REPORT

Print a clean summary:

```
## DRY Refactoring Complete

**Scope:** [backend / frontend / both / specific path]
**Extractions applied:** N
**Candidates skipped:** M (incidental duplication)

### Changes
1. [Extraction name] - [what was consolidated, how many call sites, mechanism used]
2. ...

### Skipped (incidental, not real duplication)
1. [What looked duplicated but wasn't, and why]
2. ...

### Verification
- Backend tests: [count] passing (baseline: [count])
- Backend clippy: clean
- Frontend build: clean
- Suppression scan: clean
```

---

# RULES

- NEVER modify test assertions to make them pass after refactoring. If tests break, the refactoring changed behavior.
- NEVER extract from only 2 occurrences unless the duplication is exact and encodes the same knowledge.
- NEVER create files named "utils", "helpers", "common", or "shared" without a domain qualifier (e.g. "currency_utils.rs" is OK, "utils.rs" is not).
- NEVER share code between frontend and backend to reduce duplication - they are separate deployment units.
- NEVER use Rust proc macros for DRY refactoring unless the pattern is genuinely syntactic and appears 5+ times.
- NEVER force an extraction that increases total complexity (lines + cognitive load).
- ALWAYS present the plan and wait for user confirmation before making changes.
- ALWAYS revert extractions that break tests rather than fixing tests.
- ALWAYS run verification after each individual extraction, not just at the end.
- If you find zero candidates worth extracting, that is a valid outcome. Report it honestly.

$ARGUMENTS
