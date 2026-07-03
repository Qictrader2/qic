# QIC Trader — Project Rules

QIC Trader is a crypto P2P trading platform. Two submodules:
- `frontend/` — Next.js 16 + React 19 + TypeScript (bun, Tailwind, Shadcn, Redux, Zustand, React Query, Socket.IO, Wagmi/Viem)
- `qictrader-backend-rs/` — Rust + Axum + SQLx + PostgreSQL

## Design Intent Documents

**Read these before implementing any ticket.** They define what we are building and how it actually works today.

- **Intent** (what we are aiming for): `qictrader-backend-rs/docs/intended-entity-state-machines.md`
- **AS BUILT** (how it is actually implemented): `qictrader-backend-rs/docs/as-built/as-built-state-machines.md`
- **Per-entity design system** (intended / as-built / drift): `qictrader-backend-rs/docs/design/README.md`

Any implementation that contradicts the intent document is wrong. If the AS BUILT diverges from intent, flag it — don't silently perpetuate the divergence.

## Design Doc Interview Questionnaire

When seeding or updating the intended design doc for an entity (`qictrader-backend-rs/docs/design/{entity}.md`), run **one interview session per entity** and ask the assigned dev exactly these questions, then write the answers into the entity doc using the structured schema in `docs/design/README.md`:

1. **Overview** — In one paragraph, what is this entity and what is it responsible for?
2. **States** — What are all the valid states? For each, when is the entity in it?
3. **Terminal states** — Which states are terminal (no further transitions)?
4. **Allowed transitions** — For every transition, fill the row: `| From | To | Trigger | Actor | Notes |`. List only transitions that are intended; anything else is invalid by definition.
5. **Role guards** — For each transition/action, who is allowed to perform it (Buyer, Seller, Reseller, Admin, Moderator, System)? What authorization check enforces it?
6. **Business invariants** — What must always be true (e.g. amounts non-negative, balance ≥ held, one active escrow per trade)? What must never happen?
7. **Ledger effects** — Which transitions post ledger entries / move money? Debit what, credit what, to whom, in what currency? Is it ledger-only or on-chain?
8. **Fee effects** — Which transitions accrue or charge fees (platform fee, reseller commission, affiliate commission, network fee)? Who pays, who receives, when?
9. **Timeout / job behavior** — What background jobs or timeouts act on this entity? After how long, and what transition do they trigger?
10. **Open questions** — Anything undecided. Record each with an owner and date.

Write tables (not prose) for transitions and role guards so drift detection is deterministic. Add the `<!-- design-entity: {entity} -->` marker and every `<!-- design-anchor: {entity}.{section} -->` anchor.

## Design Doc Agent Operating Rule

When changing entity lifecycle, role guards, ledger effects, fee behavior, timeout behavior, or state transitions:

1. Run `cargo run --bin design-check`.
2. Read any generated drift report.
3. If drift exists, ask the dev: "This change drifts from the intended design for {entity}.{section}. Is this an intentional design change or a bug?"
4. If **intentional design change**: update `docs/design/{entity}.md`, regenerate as-built and drift, rerun `cargo run --bin design-check`.
5. If **bug**: fix the code to match the intended design, regenerate as-built and drift, rerun `cargo run --bin design-check`.
6. Do not silently leave unresolved drift. Never hand-edit a generated file under `docs/design/.generated/`.

## Design System

**Read `DESIGN.md` before any frontend/UI work.** It is the single source of truth for QICTRADER's visual identity — colors, typography, spacing, components, motion, and anti-patterns.

- All UI changes must conform to the tokens and patterns defined in `DESIGN.md`
- Do not introduce fonts, colors, shadows, or radius values outside the design system
- Use CSS variables and Tailwind tokens — no hardcoded hex/rgb in components
- When in doubt, check the anti-patterns section (Section 11) before writing markup

## Deployment

**Full deployment guide: [`DEPLOYMENT.md`](DEPLOYMENT.md) — read it before any deploy work.**

Everything is on **Heroku** (no Vercel). Deploys are **PR-driven and automatic** —
you never run a deploy command by hand:

- Branch → open a PR → a **Heroku review app** is created (seeded from staging).
- Merge to `main` → **staging auto-deploys** (`staging.qictrader.com` / backend staging).
- **Promote** the verified staging release in the Heroku pipeline → **production**
  (`www.qictrader.com` / `qictrader-backend-rs`). Production is manual + gated.

Rules:
- All changes go through a **PR**. Never push directly to `main`. Never force-push.
- `git pull --rebase origin main` is **mandatory** before you start and before you push.
- Commit messages follow: `TICKET-ID: Short description` (no emoji prefixes).
- GitHub text (PR titles, PR descriptions, PR/issue comments, review comments) is
  written in the **first person** ("I fixed the header alignment", not "this PR
  fixes" or "Marcello flagged") and **never contains em dashes**. Use plain
  sentences, commas, or parentheses instead.
- Backend is **Heroku buildpack only** (Heroku compiles server-side). The old local
  cross-compile + Slug-API path (`scripts/fast-deploy-backend.sh`) was **removed**
  after the 2026-06-15 prod outage — do not reintroduce it.

`./commit-all.sh` is a **commit/push helper only** (it never deploys):

```
./commit-all.sh "TICKET-ID: message"                 # commit submodules + update root pointer
./commit-all.sh "TICKET-ID: message" --push          # commit + push current branch(es)
./commit-all.sh "TICKET-ID: message" --frontend-only  # frontend submodule only
./commit-all.sh "TICKET-ID: message" --backend-only   # backend submodule only
./commit-all.sh "TICKET-ID: message" --dry-run        # preview without making changes
```

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

### Hosting (Heroku)

The frontend runs on Heroku as a Next.js **standalone** server (`output: "standalone"`
in `next.config.ts`, booted via the `Procfile`; static assets copied by the
`heroku-postbuild` script). It is built from the `Qictrader2/Frontend` repo via the
`qictrader-frontend` Heroku pipeline. There is **no Vercel** — see
[`DEPLOYMENT.md`](DEPLOYMENT.md) for the full deploy process and Config Var setup.
