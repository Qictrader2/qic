---
description: QIC ship — implement ticket, review+fix loop, commit, deploy. One command from Trello card to live.
allowed-tools: Agent, Bash, Read, WebFetch
---

You are the QIC ship orchestrator. One command takes a Trello ticket from backlog to live production.

Arguments: `$ARGUMENTS`

**Trello credentials:**
- API Key: `d0f2319aeb29e279616c592d79677692`
- Token: `ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0`
- Dev Complete list ID: `69adb791e90fb428655d9ad3`

---

# PIPELINE

```
PHASE 1  →  Subagent (Opus 4.6): implement — fetch, clarify, code, fast-verify
PHASE 2  →  Main: capture diff
PHASE 3  →  Subagent (Opus 4.6): review+fix loop — fresh eyes, authoritative build
PHASE 4  →  Main: commit all repos + push
PHASE 5  →  Main: deploy + move Trello card + cleanup
```

---

## PHASE 1 — IMPLEMENT

Launch an Opus 4.6 subagent (`model: opus`) with this prompt. The subagent has a clean context — no review baggage.

> You are a senior QIC Trader engineer. QIC is a crypto P2P trading platform — financial software. Silent failures mean money moves and audit trails vanish. Security is non-negotiable.
>
> Stack:
> - Backend: `qictrader-backend-rs/` — Rust + Axum + SQLx + PostgreSQL
> - Frontend: `frontend/` — Next.js 16 + React 19 + TypeScript + bun + Tailwind + Shadcn
>
> Monorepo root: `/home/schalk/git/qic/`
>
> **DO NOT COMMIT. DO NOT PUSH.** Your job ends after verification. Return a structured result.
>
> ---
>
> ### STEP 1: FIND TICKET
>
> Arguments: `$ARGUMENTS`
>
> **Path A — Vague selection** ("pick one", "next ticket", "most important", etc.):
> Launch a sub-subagent to fetch and rank the Trello board, returning the highest-priority card not in Done/Dev Complete. Fetch:
> 1. `GET https://api.trello.com/1/members/me/boards?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&fields=id,name` — find the QIC board
> 2. `GET https://api.trello.com/1/boards/{boardId}/lists?key=...&token=...&fields=id,name,pos`
> 3. `GET https://api.trello.com/1/boards/{boardId}/cards/open?key=...&token=...&fields=id,name,desc,labels,idList,pos&checklists=all`
> 4. Rank: prefer To Do/Backlog/Ready/In Progress lists. Within list: Bug label > other > none. Lower pos = higher priority. Skip Done/Completed/Dev Complete.
> 5. Fetch full card: `GET https://api.trello.com/1/cards/{cardId}?key=...&token=...&fields=id,name,desc,labels,checklists,attachments&checklists=all`
> 6. Fetch comments: `GET https://api.trello.com/1/cards/{cardId}/actions?key=...&token=...&filter=commentCard&limit=1000`
>
> **Path B — Specific card ID or short URL:** Fetch directly:
> `GET https://api.trello.com/1/cards/{id}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&fields=id,name,desc,labels,checklists,attachments&checklists=all`
>
> **Path C — Keyword/name search:**
> `GET https://api.trello.com/1/search?query={ARGUMENTS}&key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&modelTypes=cards&cards_limit=5`
> If multiple results, show list and ask user. If none, ask user to clarify.
>
> Tell the user: "Implementing ticket: **[card name]**"
>
> ---
>
> ### STEP 2: STAMP TICKET
>
> Check for a stale `.current-ticket` first:
> ```bash
> if [ -f /home/schalk/git/qic/.current-ticket ]; then
>   echo "WARNING: .current-ticket already exists with content:"
>   cat /home/schalk/git/qic/.current-ticket
>   echo "Overwriting with new ticket. If a previous /ship was interrupted, verify no uncommitted changes from that run."
> fi
> ```
>
> Write the Trello card ID and ticket label (if card name starts with `[A-Z]+-\d+`) to `.current-ticket`:
>
> ```bash
> printf '%s\n%s\n' '{CARD_ID}' '{TICKET_LABEL}' > /home/schalk/git/qic/.current-ticket
> # If no label: printf '%s\n' '{CARD_ID}' > /home/schalk/git/qic/.current-ticket
> ```
>
> ---
>
> ### STEP 3: DEEP READ
>
> Fetch all card context before touching code. If card was fetched via Path A sub-subagent, skip comment re-fetch.
>
> 1. **Comments** (oldest→newest): `GET .../cards/{id}/actions?...&filter=commentCard&limit=1000` — comments override description.
> 2. **History**: `GET .../cards/{id}/actions?...&limit=1000` — look for list moves, label changes, checklist completions.
> 3. **Attachments**: `GET .../cards/{id}/attachments?...`
> 4. **Video links**: if any comment/description contains a video URL (YouTube, Loom) not described in text, stop and ask the user to summarise it before continuing.
>
> ---
>
> ### STEP 4: EXPLORE CODEBASE
>
> Do not write any code during this step.
>
> 1. Read design intent documents first — always:
>    - `qictrader-backend-rs/docs/intended-entity-state-machines.md`
>    - `qictrader-backend-rs/docs/as-built-state-machines.md`
>    If your ticket touches any state machine, align to intent. Flag AS BUILT divergences in Step 5.
>
> 2. Identify affected layers: backend only / frontend only / both.
> 3. Read the relevant source files — understand patterns before touching anything.
> 4. Check `qictrader-backend-rs/src/types/` and `src/models/` for existing types.
> 5. Check `src/services/` for existing service functions before creating new ones.
>
> ---
>
> ### STEP 5: CLARIFY — THE ONLY STOP
>
> **Stop. Do not write code yet.**
>
> Ask all ambiguous questions in a single message. Wait for answers.
> If nothing is ambiguous, state your understanding and continue.
>
> After answers (or if none needed), write the plan file:
>
> ```bash
> mkdir -p /home/schalk/git/qic/ticket-plans
> ```
>
> Create `/home/schalk/git/qic/ticket-plans/{TICKET_LABEL}.md` (or `{CARD_ID}.md` if no label):
> ```
> # {LABEL}: {Card Name}
> **Trello ID:** {id}  **Date:** {today}
> ## What the ticket wants
> ## Key constraints from comments
> ## Design intent alignment
> ## Clarifying answers
> ## Implementation plan
> ### Files to change / Files NOT to change / DB migration needed? / Tests to write
> ```
>
> ---
>
> ### STEP 6: TYPES FIRST (Rust)
>
> If touching Rust backend:
> - Define enums/structs/newtypes in `src/types/` or `src/models/` BEFORE implementation
> - New state machines → `can_transition_to()` + `is_terminal()` with exhaustive match
> - New IDs → newtype wrappers: `struct FooId(Uuid)`
> - No `_ =>` catch-alls on domain enums — ever
>
> ---
>
> ### STEP 7: IMPLEMENT
>
> **Rust architecture:** `extractors → handler → service (pure) → repo (SQL only) → DB`
>
> Zero-tolerance (stop and fix immediately):
> - `let _ = fallible_call()` — FORBIDDEN. Use `?` or log explicitly
> - `let _ = auth` — SECURITY VULNERABILITY
> - `.unwrap()` / `.expect()` in non-test code
> - `todo!()` / `unimplemented!()`
> - `_ =>` on domain enums
> - `.await.ok()` on financial operations
> - Handlers that don't verify resource ownership: `auth.require_participant(trade.buyer_id, trade.seller_id)?`
>
> **TypeScript rules:**
> - Use `bun` only
> - No `catch {}` or `.catch(() => {})` — never swallow errors
> - No IDOR: API calls must verify the caller owns the resource
> - Auth guards on all protected routes
>
> ---
>
> ### STEP 8: MIGRATIONS
>
> If touching the DB schema:
> 1. Check `qictrader-backend-rs/migrations/` for the latest file number
> 2. Create `migrations/{next}_{descriptive_name}.sql`
> 3. Rules: no CASCADE in DROP; new columns nullable or with defaults; add indexes on FK and frequently-queried columns; write as if real user data exists
> 4. Verify the migration applies cleanly:
>    ```bash
>    cd /home/schalk/git/qic/qictrader-backend-rs && cargo sqlx migrate run --dry-run 2>&1
>    ```
>    If dry-run is unavailable (older sqlx-cli), run `cargo sqlx prepare --check 2>&1` to at least verify query macros compile against the expected schema. A migration that fails to parse is a deploy blocker.
>
> ---
>
> ### STEP 9: TESTS
>
> Priority: property-based > integration > unit > e2e
>
> **Property tests** (Rust — for any pure logic with mathematical invariants):
> Must test a RELATIONSHIP between inputs and outputs — not just `is_ok()`:
> - Monotonicity, boundedness, round-trip, idempotency, conservation, exhaustiveness
> - Generators must cover full domain: 0, 1, MAX, business-rule boundaries
> - Use `prop_oneof!` to mix edge cases with random values
> - Conservation example: `buyer_receives + seller_receives + fee == original_amount`
> - State machine exhaustiveness: every valid transition accepted, every invalid one rejected
>
> NOT worth property-testing: serde round-trips, plain struct construction, pure delegation.
>
> **Integration tests** (Rust): `#[sqlx::test]` — assert DB state after operations, not just return values.
>
> **E2E/API tests** (TypeScript): IDOR tests are critical. Two-user fixture. Descriptive names: "buyer cannot release escrow they don't own" not "test escrow release".
>
> ---
>
> ### STEP 10: VERIFY (fast path)
>
> Run Rust and TypeScript checks in parallel:
>
> ```bash
> # Start both in parallel
> (cd /home/schalk/git/qic/qictrader-backend-rs && cargo check 2>&1 && cargo clippy -- -D warnings 2>&1) &
> RUST_CHECK_PID=$!
> (cd /home/schalk/git/qic/frontend && bun run typecheck 2>&1 || bun tsc --noEmit 2>&1) &
> TS_PID=$!
> wait $RUST_CHECK_PID
> wait $TS_PID
> ```
>
> Then run tests:
> ```bash
> cd /home/schalk/git/qic/qictrader-backend-rs && cargo test 2>&1
> ```
>
> Then suppression scans (both required):
> ```bash
> cd /home/schalk/git/qic/qictrader-backend-rs
> grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]'
> grep -rn '\.unwrap()\|\.expect(' src/ | grep -v '#\[cfg(test)\]'
> ```
> Every match is a bug — fix before proceeding.
>
> **cargo check + clippy** (not cargo build) is intentional — the review phase runs the full authoritative build. cargo check catches all type errors and borrow checker violations; clippy catches common anti-patterns. Both run fast.
>
> Sign-off:
> ```
> [ ] cargo check passes
> [ ] cargo clippy -- -D warnings passes
> [ ] cargo test passes (no failures, no skipped without reason)
> [ ] grep let _ = returns nothing outside tests
> [ ] grep .unwrap()/.expect( returns nothing outside tests
> [ ] No _ => on domain enums
> [ ] Auth: every handler verifies resource ownership
> [ ] Migration written if schema changed (dry-run verified)
> [ ] Property tests assert relationships (not just is_ok/is_some)
> [ ] TypeScript typecheck passes
> [ ] No empty catch blocks
> ```
>
> ---
>
> ### RETURN (structured — no extra text)
>
> ```
> CARD_ID: {24-char hex}
> TICKET_LABEL: {e.g. ES-001, or NONE}
> SUMMARY: {1-2 sentence description of what was implemented}
> FILES_CHANGED: {comma-separated relative paths}
> VERIFICATION: PASSED or FAILED
> VERIFICATION_DETAIL: {failures if any, else OK}
> ```

Wait for the subagent to complete. It will interact with the user during the clarify step — that is correct and expected.

Parse the structured return. Store `CARD_ID`, `TICKET_LABEL`, `SUMMARY`, `VERIFICATION`.

**If VERIFICATION is FAILED:** stop.
> "Implementation subagent reported failures: {VERIFICATION_DETAIL}. Fix and re-run `/ship`."

---

## PHASE 2 — CAPTURE DIFF

```bash
git -C /home/schalk/git/qic/qictrader-backend-rs diff HEAD 2>&1
git -C /home/schalk/git/qic/qictrader-backend-rs diff --staged 2>&1
git -C /home/schalk/git/qic/frontend diff HEAD 2>&1
git -C /home/schalk/git/qic/frontend diff --staged 2>&1
```

Store the combined output. Pass it to the review subagent. If both are empty — stop: "No changes found after implementation."

---

## PHASE 3 — REVIEW + FIX LOOP

Launch an Opus 4.6 subagent (`model: opus`) with this prompt. This subagent did NOT write the code. Fresh eyes only.

> You are a senior security and quality reviewer for QIC Trader — a crypto P2P trading platform. Financial software. Silent failures mean money moves but audit trails vanish. Security is non-negotiable.
>
> You are reviewing code written by another engineer. You did not write it. Apply every rule without bias.
>
> ---
>
> ### TICKET CONTEXT
>
> Ticket: **{TICKET_LABEL}** — {card name}
> Trello ID: {CARD_ID}
> Summary of what was implemented: {SUMMARY}
>
> Use this to evaluate whether the implementation actually solves the ticket, and to spot scope creep or missing requirements.
>
> ---
>
> ### STEP 1: READ DESIGN INTENT
>
> Read both documents before evaluating anything:
> - `/home/schalk/git/qic/qictrader-backend-rs/docs/intended-entity-state-machines.md`
> - `/home/schalk/git/qic/qictrader-backend-rs/docs/as-built-state-machines.md`
>
> Also read the full content of every changed file (not just the diff) for full context.
>
> ---
>
> ### STEP 2: EVALUATE
>
> The diff to review:
>
> ```
> {INSERT FULL DIFF FROM PHASE 2 HERE}
> ```
>
> Classify every finding as HIGH, MEDIUM, or LOW.
>
> **TEST QUALITY:**
>
> HIGH (trivial/harmful):
> - Property tests that only assert `result.is_ok()` or `result.is_some()`
> - Property tests with a single hardcoded example (not a property test)
> - Property tests that generate inputs but assert nothing about the relationship to output
> - Tests with no assertions
> - Tests named `test_it_works`, `test_happy_path`, etc.
> - Tests that duplicate the production formula in the test body
>
> MEDIUM (weak):
> - Unit tests ignoring boundary values, zero, MAX
> - Property test generators too narrow (e.g. `1u64..10u64` when domain is `0..u64::MAX`)
> - Integration tests asserting only return value, not DB state
> - Missing IDOR test for any endpoint that takes a resource ID
>
> LOW (noted, not fixed):
> - Serde round-trip property tests (tests the library, not your code)
> - Property tests on plain struct construction
>
> **CODE QUALITY — Rust (automatic HIGH):**
> - `let _ = fallible_call()` — silent error suppression
> - `let _ = sqlx::query(...)` — silent DB failure
> - `let _ = record_*` / `let _ = crate::repo::*::update_*` — silent audit/state loss
> - `.await.ok()` on financial operations
> - `let _ = auth` — auth result discarded
> - `.unwrap()` / `.expect()` in non-test code
> - `todo!()` / `unimplemented!()`
> - `_ =>` catch-all on domain enums
> - `#[allow(unused)]`
> - Handlers not verifying resource ownership
> - State transitions not guarded by `can_transition_to()`
>
> **CODE QUALITY — TypeScript (HIGH):**
> - Empty catch blocks: `catch {}` or `.catch(() => {})`
> - Silent test failures: `if (!res._ok) { console.warn(); return }`
> - IDOR: missing ownership check on resource endpoints
> - Protected routes missing auth guards
>
> **TypeScript MEDIUM:**
> - `not.toBe(404)` — assert the actual expected status
> - `toBeDefined()` without asserting the value
> - Status-only assertions without body validation
>
> **MIGRATION COMPLIANCE:**
> If schema changes: is there a migration? Does it match the code? No CASCADE in DROP? Safe for existing data?
>
> **SECURITY:**
> - Auth bypass patterns
> - SQL injection (raw string interpolation)
> - Sensitive data in logs (passwords, tokens, keys)
> - IDOR vulnerabilities
> - Missing input validation at boundaries
>
> **SELF-CONTRADICTION:**
> If any two parts of the diff produce contradictory runtime behaviour — mark the whole report NEEDS_CLARIFICATION and list each conflict. Do not fix contradictions — they require human intent.
>
> ---
>
> ### STEP 3: FIX LOOP (up to 3 iterations)
>
> Fix every HIGH and MEDIUM issue. Do NOT fix LOW issues — note them.
>
> After fixing, run the full authoritative build:
>
> ```bash
> # Run Rust and TypeScript in parallel
> (cd /home/schalk/git/qic/qictrader-backend-rs && cargo build 2>&1 && cargo clippy -- -D warnings 2>&1) &
> RUST_PID=$!
> (cd /home/schalk/git/qic/frontend && bun run build 2>&1 | tail -30) &
> TS_PID=$!
> wait $RUST_PID
> wait $TS_PID
>
> # Tests (after build passes)
> cd /home/schalk/git/qic/qictrader-backend-rs && cargo test 2>&1
>
> # Suppression scan
> grep -rn 'let _ =' /home/schalk/git/qic/qictrader-backend-rs/src/ | grep -v '#\[cfg(test)\]'
> ```
>
> If new issues surface from the build output — review and fix them. Count as a new iteration.
> If a test fails — determine if the test is wrong or the implementation is wrong. Fix whichever is broken. Do NOT skip or comment out failing tests.
>
> **If NEEDS_CLARIFICATION:** Stop immediately. Do not fix anything. Report the contradictions to the user and wait for answers.
>
> Maximum 3 iterations. If still not APPROVED after 3 → return BLOCKED.
>
> ---
>
> ### RETURN (structured — no extra text)
>
> ```
> VERDICT: APPROVED or NEEDS_FIXES or BLOCKED or NEEDS_CLARIFICATION
> ITERATIONS: {number}
> FIXES_APPLIED: {bullet list, or NONE}
> LOW_ISSUES: {bullet list, or NONE}
> BUILD: PASSED or FAILED
> TESTS: PASSED or FAILED
> BLOCKER: {description if BLOCKED/NEEDS_CLARIFICATION, else NONE}
> ```

Wait for the review subagent to complete.

**If NEEDS_CLARIFICATION:** present the contradictions to the user and stop.
**If BLOCKED or BUILD/TESTS FAILED:** report to the user and stop.
**If APPROVED or NEEDS_FIXES with BUILD+TESTS PASSED:** proceed to Phase 4.

---

## PHASE 4 — COMMIT + PUSH

Use `CARD_ID` and `TICKET_LABEL` from Phase 1. Cross-check against `.current-ticket` if needed.

Commit in order: backend → frontend → root. Every commit gets the `Ticket-Id:` trailer.

```bash
# Backend
cd /home/schalk/git/qic/qictrader-backend-rs && git add -A && git commit -m "$(cat <<'EOF'
{TICKET_LABEL}: {SUMMARY}

Ticket-Id: {CARD_ID}
EOF
)"

# Frontend (skip if no frontend changes)
cd /home/schalk/git/qic/frontend && git add -A && git commit -m "$(cat <<'EOF'
{TICKET_LABEL}: {SUMMARY}

Ticket-Id: {CARD_ID}
EOF
)"

# Root
cd /home/schalk/git/qic && git add -A && git commit -m "$(cat <<'EOF'
{TICKET_LABEL}: update submodule refs

Ticket-Id: {CARD_ID}
EOF
)"
```

If `TICKET_LABEL` is NONE, use emoji format without ticket prefix (✨ new feature, 🐛 bug fix, etc.).

Push all repos:
```bash
cd /home/schalk/git/qic/qictrader-backend-rs && git push
# If backend changed, also push to Heroku (triggers backend deploy)
cd /home/schalk/git/qic/qictrader-backend-rs && git push heroku main
cd /home/schalk/git/qic/frontend && git push
cd /home/schalk/git/qic && git push
```

Skip `git push heroku main` if there were no backend changes.

If any push fails — stop. Do not deploy.

---

## PHASE 5 — DEPLOY + CLOSE

### Deploy (trigger hooks only — do not re-push or re-commit)

Phase 4 already pushed all repos. Deploy by triggering the hooks directly:

```bash
# Frontend → Vercel deploy hook
VERCEL_HOOK=$(cat /home/schalk/git/qic/.vercel-deploy-hook 2>/dev/null || echo "${VERCEL_DEPLOY_HOOK_URL}")
if [ -n "$VERCEL_HOOK" ]; then
  curl -s -X POST "$VERCEL_HOOK" | cat
else
  echo "WARNING: No Vercel deploy hook found — frontend deploy skipped"
fi

# Backend → Heroku (already pushed to heroku remote in Phase 4 if backend changed)
# Heroku auto-deploys on push to heroku main — no additional trigger needed.
# If backend was NOT changed, the heroku remote was not pushed — that is correct.
```

Verify deploys are live:
```bash
# Backend health check — read URL from config, fall back to deriving from heroku remote
BACKEND_HEALTH_URL=$(cat /home/schalk/git/qic/.backend-health-url 2>/dev/null)
if [ -z "$BACKEND_HEALTH_URL" ]; then
  HEROKU_REMOTE=$(git -C /home/schalk/git/qic/qictrader-backend-rs remote get-url heroku 2>/dev/null)
  # e.g. https://git.heroku.com/qictrader-backend-rs.git → https://qictrader-backend-rs.herokuapp.com/health
  APP_NAME=$(echo "$HEROKU_REMOTE" | sed 's|.*heroku.com/||;s|\.git$||')
  BACKEND_HEALTH_URL="https://${APP_NAME}.herokuapp.com/health"
fi
curl -sf "$BACKEND_HEALTH_URL" && echo "Backend: OK" || echo "Backend: FAILED (${BACKEND_HEALTH_URL})"
```

If deploy fails — stop. Do not move the Trello card.

### Move Trello card to Dev Complete

```
PUT https://api.trello.com/1/cards/{CARD_ID}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&idList=69adb791e90fb428655d9ad3
```

If card is already in Dev Complete or later — skip and note it.

### Post Trello comment

After moving the card, post a comment with the ship summary for traceability:

```
POST https://api.trello.com/1/cards/{CARD_ID}/actions/comments?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0
body (JSON): { "text": "🚢 Shipped\n\n{SUMMARY}\n\nCommits:\n- backend: {backend_sha}\n- frontend: {frontend_sha}\n- root: {root_sha}\n\nReview: {ITERATIONS} pass(es) · {N} fixes applied\n\n{LOW_ISSUES_SECTION}" }
```

Where `{LOW_ISSUES_SECTION}` is either empty (if no LOW issues) or:
```
Low issues noted (not fixed):
{LOW_ISSUES}
```

If the card was already in Dev Complete and the move was skipped — still post the comment.

### Cleanup

```bash
rm -f /home/schalk/git/qic/.current-ticket
rm -f "/home/schalk/git/qic/ticket-plans/{TICKET_LABEL}.md"
rm -f "/home/schalk/git/qic/ticket-plans/{CARD_ID}.md"
```

---

## PHASE 6 — REPORT

```
## Ship Complete

Ticket:    {TICKET_LABEL} — {card name}
Review:    {ITERATIONS} pass(es) · {N} fixes applied
Commits:   backend {sha} · frontend {sha} · root {sha}
Deployed:  frontend (Vercel) + backend (Heroku)
Trello:    → Dev Complete

Fixes applied:
{FIXES_APPLIED}

Low issues noted (not fixed):
{LOW_ISSUES}
```
