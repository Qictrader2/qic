---
description: QIC dry - DRY refactoring across frontend and backend. Parallel subagent scanning, then safe consolidation with test verification.
allowed-tools: Agent, Bash, Read, Write, Edit, Grep, Glob, LSP
---

You are performing a DRY (Don't Repeat Yourself) refactoring pass across the QIC Trader codebase. You orchestrate parallel Opus subagents to deeply scan for duplication, then classify and refactor the findings yourself.

Arguments: `$ARGUMENTS`

If $ARGUMENTS specifies a scope (e.g. "backend", "frontend", "services/escrow", "hooks"), limit the scanning and refactoring to that scope. Otherwise, refactor both frontend and backend.

---

# THE DRY FLOW

```
┌──────────────────────────────────────────────────────────────────────┐
│  1. BASELINE        →  Run all tests, record the green state         │
│  2. PARALLEL SCAN   →  Spawn Opus subagents to find duplication      │
│  3. CONSOLIDATE     →  Merge subagent findings, deduplicate          │
│  4. CLASSIFY        →  Real knowledge duplication vs. incidental      │
│  5. PLAN            →  Rank candidates, propose extractions           │
│  6. REFACTOR        →  Extract one at a time, verify after each       │
│  7. FINAL VERIFY    →  Full test suite + build + clippy/lint          │
│  8. REPORT          →  Summary of what changed and why                │
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

# STEP 2: PARALLEL SCAN

This is the core innovation. Instead of scanning the codebase yourself (which leads to shallow, narrow results), spawn parallel Opus subagents that each deeply scan a slice of the codebase.

## Determine scan slices

Based on the scope ($ARGUMENTS or full codebase), create scan slices. Each slice should be a coherent domain area, not an arbitrary file split.

**Full codebase scan (default) - spawn 5 subagents:**

| Subagent | Scope | What to look for |
|----------|-------|------------------|
| `dry-scan-repo` | `qictrader-backend-rs/src/repo/` | Repeated SQL query shapes, identical WHERE clauses, repeated pagination patterns, similar CRUD operations, repeated join patterns |
| `dry-scan-services` | `qictrader-backend-rs/src/services/` | Duplicated business logic, repeated validation, identical state transition checks, repeated fee/amount calculations, similar error handling patterns |
| `dry-scan-api` | `qictrader-backend-rs/src/api/` + `src/extractors/` + `src/middleware/` | Repeated handler patterns, identical auth checks, repeated response construction, similar request validation, repeated error response formatting |
| `dry-scan-fe-components` | `frontend/src/components/` + `frontend/src/app/` | Near-identical components, repeated UI patterns, duplicated form logic, repeated modal/dialog patterns, similar data display components |
| `dry-scan-fe-logic` | `frontend/src/hooks/` + `frontend/src/lib/` + `frontend/src/store/` + `frontend/src/types/` | Repeated hooks, duplicated API call patterns, identical type definitions, repeated state management patterns, similar utility functions |

**Backend-only scan - spawn 3 subagents:** `dry-scan-repo`, `dry-scan-services`, `dry-scan-api`

**Frontend-only scan - spawn 2 subagents:** `dry-scan-fe-components`, `dry-scan-fe-logic`

**Narrow scope** (e.g. "services/escrow") - spawn 1 subagent focused on that directory.

## Subagent prompt template

Spawn ALL scan subagents in a SINGLE message (parallel launch). Use `model: "opus"` for each. Give each subagent this prompt structure:

```
You are a DRY duplication scanner for a [Rust Axum backend / Next.js React frontend].
Your job is to deeply read all code in your assigned scope and find duplication candidates.

SCOPE: [directory paths]
Read EVERY file in scope. Do not skim. Do not sample. Read them all.

WHAT TO LOOK FOR:
- [scope-specific items from the table above]
- Functions/methods with near-identical structure (differing only in names/types)
- Copy-pasted blocks with minor variations
- Identical business rules encoded in multiple places
- Same validation logic repeated across files
- Same data transformation pattern in 3+ places

FOR EACH CANDIDATE, report:
1. PATTERN NAME: A short descriptive name for what is duplicated
2. OCCURRENCES: List each occurrence with file path and line range
3. SIMILARITY: How similar are they? (exact / near-identical / structural)
4. CHANGE REASON: Would these all change for the same reason? (yes/no/unsure + why)
5. CROSS-CUTTING: Does this pattern likely also appear outside your scope? If so, where?
6. EXTRACTION IDEA: Brief suggestion for how to consolidate (function, trait, hook, component, etc.)

Be thorough. Read every file. The goal is to find ALL duplication worth considering,
not just the most obvious cases. Report even borderline candidates - the orchestrator
will classify them.

Return your findings as a structured list. No preamble, no summary - just the candidates.
If you find nothing worth reporting, say "NO CANDIDATES FOUND" and explain what you checked.
```

## Wait for all subagents

All subagents must complete before proceeding. Do NOT start classifying partial results.

---

# STEP 3: CONSOLIDATE

Merge findings from all subagents into a single candidate list.

1. **Deduplicate** - if two subagents flagged the same pattern (e.g. a service function and the handler that wraps it), merge them into one candidate with all occurrences listed.
2. **Connect cross-cutting hints** - if subagent A noted "this might also appear in services/" and subagent B found it in services/, link them.
3. **Drop noise** - if a subagent reported something with only 1 occurrence and no cross-cutting hint, drop it.

---

# STEP 4: CLASSIFY

For each consolidated candidate, answer ALL of these questions:

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

# STEP 5: PLAN

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

Wait for confirmation before Step 6.

---

# STEP 6: REFACTOR

Apply each extraction one at a time. After EACH extraction:

### 6a. Make the change

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

### 6b. Verify after each extraction

**Backend:**
```bash
cd qictrader-backend-rs && cargo check 2>&1 | tail -5
cargo test 2>&1 | tail -5
```

**Frontend:**
```bash
cd frontend && bun run build 2>&1 | tail -10
```

### 6c. Revert if broken

If tests fail after an extraction and you cannot fix it within one small adjustment:
```bash
git checkout -- .  # revert the current extraction
```

Then move on to the next candidate. Do NOT force an extraction that breaks tests.

### 6d. Anti-patterns to watch for

After each extraction, check:

- **Parameter explosion** - did the new function/component gain more than 2 parameters to accommodate different callers? If yes, the abstraction is wrong. Revert.
- **Conditional branches per caller** - does the new code have if/else or match arms that serve specific callers? If yes, the abstraction is wrong. Revert.
- **Utility junk drawer** - are you putting unrelated functions into "utils.rs" or "helpers.ts"? Each utility belongs near its domain or in a purpose-named module.
- **Coupling across boundaries** - are you sharing code between frontend and backend? Don't. Duplication across deployment boundaries is preferable to coupling.

---

# STEP 7: FINAL VERIFY

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

# STEP 8: REPORT

Print a clean summary:

```
## DRY Refactoring Complete

**Scope:** [backend / frontend / both / specific path]
**Subagents dispatched:** N
**Candidates found (raw):** X
**Candidates after classification:** Y (Z skipped as incidental)
**Extractions applied:** A
**Extractions reverted:** B

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
- ALWAYS spawn scan subagents in a SINGLE message for parallel execution.
- ALWAYS use `model: "opus"` for scan subagents - thorough scanning requires deep reasoning.
- ALWAYS present the plan and wait for user confirmation before making changes.
- ALWAYS revert extractions that break tests rather than fixing tests.
- ALWAYS run verification after each individual extraction, not just at the end.
- If subagents collectively find zero candidates worth extracting, that is a valid outcome. Report it honestly.

$ARGUMENTS
