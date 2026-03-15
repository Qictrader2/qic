# QIC Trader — Project Rules

QIC Trader is a crypto P2P trading platform. Two submodules:
- `frontend/` — Next.js 16 + React 19 + TypeScript (bun, Tailwind, Shadcn, Redux, Zustand, React Query, Socket.IO, Wagmi/Viem)
- `qictrader-backend-rs/` — Rust + Axum + SQLx + PostgreSQL

## Design Intent Documents

**Read these before implementing any ticket.** They define what we are building and how it actually works today.

- **Intent** (what we are aiming for): `qictrader-backend-rs/docs/intended-entity-state-machines.md`
- **AS BOLT** (how it is actually implemented): `qictrader-backend-rs/docs/as-built-state-machines.md`

Any implementation that contradicts the intent document is wrong. If the AS BOLT diverges from intent, flag it — don't silently perpetuate the divergence.

For monorepo commits and deploys use `./commit-all.sh`:

```
./commit-all.sh "message"                  # commit all submodules + update root
./commit-all.sh "message" --push           # commit + push all
./commit-all.sh "message" --deploy         # commit + push + deploy both (Vercel + Heroku)
./commit-all.sh "message" --frontend-only  # frontend submodule only
./commit-all.sh "message" --backend-only   # backend submodule only
./commit-all.sh "message" --dry-run        # preview without making changes
```

Deploy targets:
- Frontend → `vercel --prod --yes --scope qictraders-projects` from `frontend/` dir
- Backend  → `git push heroku main` (Heroku app: `qictrader-backend-rs`)

---

## Rust Backend Rules (`qictrader-backend-rs/`)

### Types-First Development

Define types before implementation. Start every feature by defining enums/structs/newtypes in `src/types/` or `src/models/`, then let compiler errors drive the implementation.

- Domain concepts live in `src/types/enums.rs` as enums, not strings
- State machines (TradeStatus, EscrowStatus) must have `can_transition_to()` and `is_terminal()` methods with tests
- Newtype wrappers for all IDs: `UserId(Uuid)`, `TradeId(Uuid)` — never raw `Uuid`

### Make Impossible States Impossible

- Enums over booleans — `(is_active: bool, is_deleted: bool)` is wrong
- Separate types for separate states when fields only exist in certain states
- Enums over strings — payment methods, event types, currencies, statuses
- Exhaustive matching — no `_ =>` catch-alls on domain enums

### Pure Functional Style

- Prefer pure functions: same input → same output, no side effects
- Push side effects to the edges (Axum handlers, `main`)
- `src/services/` and `src/types/` should be testable without IO
- No mocks unless explicitly asked — extract pure logic instead
- `let mut` requires justification

### Robust Error Handling

- `Result` types over panics
- No `.unwrap()` or `.expect()` in production code — use `?` or explicit match
- Errors as enum variants via `thiserror` — callers match on error kinds, not strings
- Axum handlers return `Result<_, AppError>` — use the project's `AppError` type

### Resumable Processes

- Long-running operations resume from the point of failure, not restart
- Idempotent operations — safe to retry
- Design for "what happens if this crashes halfway?"

### Auth & Security

- Handlers accepting `AuthUser` MUST verify the user is authorized for the specific resource
- `let _ = auth;` is a security vulnerability — auth runs but result is discarded
- Always check: is `auth.user_id` a participant in this specific trade/escrow/wallet?

---

## NO SUPPRESSION — ZERO TOLERANCE

**This is financial software. Silent failures mean money moves but audit trails vanish.**

### `let _ = fallible_call()` is FORBIDDEN

Every `let _ =` on a Result is a bug. Fix it:

```rust
// WRONG — silent failure
let _ = record_platform_fee(db, trade_id, fee).await;

// RIGHT — propagate
record_platform_fee(db, trade_id, fee).await?;

// RIGHT — log if can't propagate
if let Err(e) = record_platform_fee(db, trade_id, fee).await {
    tracing::error!(error = %e, "failed to record fee");
}
```

Specific forbidden patterns:
- `let _ = sqlx::query(...).execute(db).await;` — silent DB failure
- `let _ = crate::services::ledger::record_*(...)` — silent audit trail loss
- `let _ = crate::repo::*::update_*(...)` — silent state corruption
- `let _ = crate::services::affiliate_commission::*(...)` — silent commission loss
- `.await.ok();` on anything financial

### `let _ = auth` is a SECURITY VULNERABILITY

### Other forbidden suppressions

- No `#[allow(unused)]` — delete dead code
- No `todo!()` or `unimplemented!()` in committed code
- No `_ =>` catch-all on domain enums
- No `.ok()` to silently convert errors to None in financial/state-changing paths
- If the compiler warns, fix the cause — don't suppress the symptom

### Pre-push scan (mandatory)

```bash
grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]'
```

Every match is a reject unless explicitly justified.

### Clippy denies (configured in `lib.rs`)

```rust
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
```

Build must pass before pushing — broken build = zero static analysis enforcement.

### Project structure

```
src/
  types/       # Domain types, enums, Money
  models/      # Database row structs (sqlx::FromRow)
  repo/        # Database queries (pure SQL, no logic)
  services/    # Business logic (pure where possible)
  api/         # Axum handlers (thin, delegate to services)
  extractors/  # Axum extractors (auth, validation)
  middleware/  # Axum middleware
```

Handlers are thin: extract → validate → delegate to service → return response.

### Database Migrations (Multi-Agent Safety)

Multiple agents create migrations concurrently. **Before creating any migration:**

1. `git pull` the latest `qictrader-backend-rs` to get all remote migrations
2. `ls migrations/ | sort | tail -10` to see the highest existing timestamp
3. Pick a timestamp **strictly greater** than the highest existing one
4. After creating the file, verify no timestamp collision: `ls migrations/ | sort | awk -F'_' '{print $1}' | sort | uniq -d`
5. Filename pattern: `{TIMESTAMP}_{TICKET_ID}_{description}.up.sql` / `.down.sql`

**Never reuse a timestamp.** Duplicate timestamps cause one migration to silently skip or override.

---

## TypeScript Frontend Rules (`frontend/`)

### Test Assertions Must Be Specific

- NEVER `not.toBe(404)` — assert actual expected status: `toBe(200)`, `toBe(201)`
- NEVER `toBeDefined()` alone — assert the expected value: `toBe('USDT')`
- Assert response BODY content, not just status codes

### Tests Must Not Silently Pass on Failure

- NEVER `if (!res._ok) { console.warn(); return }` — hides real problems behind green checks
- Use `test.skip('reason')` if setup might fail — a silent pass is worse than no test
- NEVER empty catch blocks: `catch {}` or `.catch(() => {})` — swallowed errors are invisible bugs

### Test Structure

- Each test must be independent — don't rely on execution order between `it()` blocks
- Clean up created resources when practical (delete test users/offers after suite)
- Descriptive test names: "buyer cannot release escrow they don't own" not "test escrow release"
- No broken tests dumped in `broken/` — fix them or delete them; use `test.skip('reason')` for in-progress work

### Test priority order

1. Auth & authorization — IDOR tests are critical
2. State transitions — valid transitions succeed, invalid ones fail
3. Business rules — amounts, fees, limits, validation
4. Error handling — bad input returns proper codes, not 500s
5. UI flows — signup, login, form submission

### Reference test templates

- `e2e/tests/security/idor.test.ts` — two-user fixture, authorization verification
- `e2e/tests/regression/api-contracts.test.ts` — response shape validation
- `e2e/tests/phase1a/auth-001-signup.test.ts` — UI test with form interaction
- `e2e/tests/smoke/backend-health.test.ts` — clean smoke test pattern

### Build

Always use `bun` (not npm/yarn/pnpm). Run `bun run build` to verify before pushing.
