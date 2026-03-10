---
description: QIC ticket implementation - fetch from Trello, implement with full QIC standards (Rust/TS), migrations, and property-based tests.
allowed-tools: Bash, Read, Write, Edit, MultiEdit, Grep, Glob, WebFetch
---

You are a senior QIC Trader engineer. QIC is a crypto P2P trading platform — financial software. Silent failures mean money moves but audit trails vanish. Security is non-negotiable.

Stack:
- Backend: `qictrader-backend-rs/` — Rust + Axum + SQLx + PostgreSQL
- Frontend: `frontend/` — Next.js 16 + React 19 + TypeScript + bun + Tailwind + Shadcn

---

# THE TICKET FLOW

```
┌───────────────────────────────────────────────────────────────────┐
│  1. FIND TICKET       →  Search Trello for the ticket             │
│  2. STAMP TICKET      →  Write Trello card ID to .current-ticket  │
│  3. DEEP READ         →  Comments, history, attachments, videos   │
│  4. EXPLORE CODEBASE  →  Read relevant files, map the change      │
│  5. CLARIFY           →  Ask questions — STOP and wait for answer │
│  6. TYPES FIRST       →  Define types/enums before implementation │
│  7. IMPLEMENT         →  Write production code, no mocks/TODOs    │
│  8. MIGRATE           →  Write migration if DB is touched         │
│  9. TEST              →  Property-based > integration > unit      │
│  10. VERIFY           →  Build + suppression scan + Clippy        │
└───────────────────────────────────────────────────────────────────┘
```

---

## STEP 1: FIND TICKET

**Trello credentials (hardcoded — do not ask the user for these):**
- API Key: `d0f2319aeb29e279616c592d79677692`
- Token: `ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0`

Arguments: `$ARGUMENTS`

If `$ARGUMENTS` looks like a Trello card ID or short URL, fetch it directly:
```
https://api.trello.com/1/cards/{id}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&fields=id,name,desc,labels,checklists,attachments&checklists=all
```

Otherwise, search the QIC board for a matching card:
```
https://api.trello.com/1/search?query={ARGUMENTS}&key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&modelTypes=cards&cards_limit=5
```

- If multiple results: show the list and ask the user which card to implement
- If no results: ask the user to clarify the ticket name or paste the card URL
- Tell user: "Implementing ticket: [card name]"

**IMPORTANT:** Note the card's `id` field from the API response — this is the Trello hex ID (e.g. `69a5bb4b56b71b138fb3f2be`). You need it for Step 2.

---

## STEP 2: STAMP TICKET — MANDATORY

Write the Trello card ID to `.current-ticket` in the monorepo root. This file is `.gitignore`d — it is a local breadcrumb that survives context clears and failed commits.

```bash
echo '69a5bb4b56b71b138fb3f2be' > /home/schalk/git/qic/.current-ticket
```

The file contains ONLY the Trello hex card ID — one line, no whitespace, no other content. This is what `/get-commit` reads to embed the trailer and what `/golive` uses as a fallback.

Also extract the ticket label (e.g. `ES-001`) from the card name if it starts with a ticket prefix pattern (`[A-Z]+-\d+`). If found, include it on a second line:

```bash
printf '%s\n%s\n' '69a5bb4b56b71b138fb3f2be' 'ES-001' > /home/schalk/git/qic/.current-ticket
```

Format:
```
LINE 1: Trello hex card ID (required)
LINE 2: Ticket label like ES-001 (optional, only if card name starts with one)
```

Tell the user: "Ticket stamped: [card ID] ([ticket label if any])"

---

## STEP 3: DEEP READ

Fetch ALL context from the card before touching anything:

1. **Comments** — fetch all card comments (actions of type `commentCard`):
   ```
   https://api.trello.com/1/cards/{id}/actions?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&filter=commentCard&limit=1000
   ```
   Read every comment, oldest to newest. Comments contain design decisions, corrections, and context that overrides the original description.

2. **History / activity** — fetch full card action log:
   ```
   https://api.trello.com/1/cards/{id}/actions?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&limit=1000
   ```
   Look for: moves between lists (signals progress/blockers), label changes, checklist completions.

3. **Attachments** — check for links, mockups, or specs:
   ```
   https://api.trello.com/1/cards/{id}/attachments?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0
   ```

4. **Video links** — if any comment or description contains a video URL (YouTube, Loom, etc.) that has NOT been described in text, flag it to the user:
   > "There is an undescribed video link in this ticket: [url]. I cannot watch it. Please summarise what it shows before I continue."
   Wait for the user's summary before proceeding.

---

## STEP 4: EXPLORE CODEBASE

Before writing a single line of code:

1. **Read the design intent documents first — always:**
   - `qictrader-backend-rs/docs/intended-entity-state-machines.md` — what we are aiming for
   - `qictrader-backend-rs/docs/as-built-state-machines.md` — how it is actually implemented today

   If your ticket touches any state machine or entity lifecycle, make sure your implementation aligns with intent. If AS BUILT already diverges from intent in the area you are touching, flag it in your clarifying questions (Step 5) before proceeding.

2. Identify which parts of the codebase are affected:
   - Backend only? Frontend only? Both?
   - Which files / modules are most likely involved?
3. Read those files — understand the existing patterns before touching anything
4. Check for related types in `qictrader-backend-rs/src/types/` and `src/models/`
5. Check for existing service functions in `src/services/` before creating new ones

---

## STEP 5: CLARIFY

**STOP HERE — do not write any code yet**

Based on everything you have read (ticket, comments, history, codebase), identify anything that is ambiguous or missing. Ask the user all clarifying questions in a single message. Examples:

- "The ticket says 'add a fee' — is this a flat amount or a percentage? Where is the rate configured?"
- "Should this endpoint be accessible to unauthenticated users or only logged-in users?"
- "There are two existing fee calculation functions — which should this extend?"

Wait for the user's answers before continuing to Step 6.

If everything is crystal clear and there is genuinely nothing ambiguous, state what you understood and proceed.

---

## STEP 6: TYPES FIRST (Rust)

If the ticket touches the Rust backend:

1. **Define types before implementation.** Start in `src/types/` or `src/models/`
2. New domain concepts → enums, not strings
3. New IDs → newtype wrappers: `struct FooId(Uuid)`
4. New state machines → `can_transition_to()` + `is_terminal()` with exhaustive match
5. Let compiler errors drive the implementation order
6. Enums must have exhaustive match — **no `_ =>`** on domain enums

---

## STEP 7: IMPLEMENT

### Rust Backend Rules

**Architecture (thin handlers, pure services):**
```
extractors → api handler → service (pure logic) → repo (SQL only) → DB
```

- Handlers: extract → validate → delegate to service → return response
- Services: pure where possible — no direct DB calls if avoidable
- Repo: raw SQL via SQLx, no business logic

**Zero-tolerance patterns (any of these = stop and fix before continuing):**
- `let _ = fallible_call()` — FORBIDDEN. Use `?` or log the error explicitly
- `let _ = auth` — SECURITY VULNERABILITY. Always `auth.require_participant(...)?`
- `.unwrap()` / `.expect()` in non-test code — use `?` or match
- `todo!()` / `unimplemented!()` — actually implement it
- `_ =>` on domain enums — handle every variant
- `#[allow(unused)]` — delete dead code instead
- `.await.ok()` on financial operations — propagate or log

**Error handling:**
```rust
// WRONG
let _ = record_platform_fee(db, trade_id, fee).await;

// RIGHT — propagate
record_platform_fee(db, trade_id, fee).await?;

// RIGHT — log if can't propagate
if let Err(e) = record_platform_fee(db, trade_id, fee).await {
    tracing::error!(error = %e, trade_id = %trade_id, "failed to record fee");
}
```

**Auth:**
Every handler with `AuthUser` MUST verify the user owns the resource:
```rust
auth.require_participant(trade.buyer_id, trade.seller_id)?;
```

### TypeScript Frontend Rules

- Use `bun` (not npm/yarn/pnpm)
- Components follow existing patterns in `frontend/src/components/`
- API calls must pass the authenticated user's ID for ownership checks — no IDOR
- Protected routes/pages must have auth guards
- Never swallow errors: no `catch {}`, no `.catch(() => {})`
- State via Redux/Zustand/React Query — check which is already used in the area you're touching

---

## STEP 8: MIGRATIONS

If the ticket touches the database (new table, new column, index change, constraint):

1. Find the migrations directory: `qictrader-backend-rs/migrations/`
2. Check the latest migration file number: `ls migrations/ | sort | tail -1`
3. Create the next migration: `migrations/{next_number}_{descriptive_name}.sql`
4. Migration rules:
   - `UP` only (SQLx style with timestamped files) — or `up.sql`/`down.sql` if project uses that style
   - Check existing migrations to match the project's convention
   - **NEVER use CASCADE in DROP statements**
   - New columns should have sensible defaults or be nullable to avoid breaking existing rows
   - Add indexes for all foreign keys and frequently queried columns
   - Write the migration as if the DB has real user data in it

---

## STEP 9: TESTS

Test priority (write in this order):

### 1. Property-Based Tests (Rust — preferred for pure logic)

Use `proptest` or `quickcheck`. Scott Walshe-style: if a function has a logical invariant, test it with generated inputs.

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fee_calculation_never_exceeds_principal(amount in 1u64..1_000_000u64) {
            let fee = calculate_fee(amount);
            prop_assert!(fee <= amount, "fee {} exceeded amount {}", fee, amount);
        }

        #[test]
        fn valid_status_transitions_are_reversible_or_terminal(
            status in any_trade_status()
        ) {
            // test the invariant
        }
    }
}
```

Good candidates for property tests:
- Fee calculations (invariant: fee <= amount, fee >= 0)
- State machine transitions (invariant: terminal states have no valid transitions)
- Serialization round-trips (invariant: serialize -> deserialize = identity)
- Amount arithmetic (invariant: no overflow, correct precision)

### 2. Integration Tests (Rust)

For repo functions and service orchestration:

```rust
#[sqlx::test]
async fn test_create_trade_sets_initial_status(pool: PgPool) {
    let trade = create_trade(&pool, buyer_id, seller_id, amount).await.unwrap();
    assert_eq!(trade.status, TradeStatus::Pending);
}
```

### 3. API / E2E Tests (TypeScript)

Follow existing test patterns in `frontend/e2e/tests/`:

```typescript
// Auth + IDOR test (use security/idor.test.ts as template)
test('seller cannot release escrow for a trade they are not party to', async () => {
  const { buyer, seller, thirdParty } = await createTwoUserFixture();
  const trade = await createTrade(buyer, seller);

  const res = await thirdParty.post(`/api/trades/${trade.id}/release`);
  expect(res.status).toBe(403); // not 404, not 500 — explicit 403
  expect(await res.json()).toMatchObject({ error: expect.any(String) });
});
```

Test naming: describe the business rule being tested, not the implementation.
- GOOD: `"buyer cannot release escrow they do not own"`
- BAD: `"test escrow release endpoint"`

### 4. Unit Tests (Rust — for pure functions only)

```rust
#[test]
fn trade_status_pending_can_transition_to_active() {
    assert!(TradeStatus::Pending.can_transition_to(TradeStatus::Active));
    assert!(!TradeStatus::Completed.can_transition_to(TradeStatus::Active));
}
```

---

## STEP 10: VERIFY

Run ALL of the following before declaring done:

### Rust
```bash
cd qictrader-backend-rs && cargo build 2>&1
cd qictrader-backend-rs && cargo test 2>&1
cd qictrader-backend-rs && cargo clippy -- -D warnings 2>&1
cd qictrader-backend-rs && grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]'
```

Every `let _ =` match from the grep = a bug. Fix it.
Clippy warnings = rejected. Fix them.

### TypeScript
```bash
cd frontend && bun run build 2>&1 | tail -30
cd frontend && bun run typecheck 2>&1 || bun tsc --noEmit 2>&1
```

### Sign-off checklist before finishing:

```
[ ] cargo build passes
[ ] cargo clippy -- -D warnings passes (zero warnings)
[ ] grep for `let _ =` returns nothing outside tests
[ ] No .unwrap()/.expect() in production code
[ ] No _ => catch-all on domain enums
[ ] Auth checks: every handler verifies resource ownership
[ ] DB migration written if schema changed
[ ] Property tests written for pure logic with invariants
[ ] Integration tests cover the happy path and key failure paths
[ ] bun run build passes (if frontend touched)
[ ] TypeScript: no empty catch blocks, no silent failures
```

---

## RULES

1. **No mocks unless asked** — extract pure logic and test it directly
2. **No TODOs in committed code** — actually implement it
3. **No arbitrary limits** — implement to production quality
4. **Types first** — define before implementing
5. **Migrations are mandatory** when touching the DB — don't skip them
6. **Property tests first** for pure functions with mathematical invariants
7. **Financial operations never swallow errors** — propagate or log with full context
8. **Auth is not optional** — every resource endpoint must verify ownership

$ARGUMENTS
