# Backend codebase — pointer + reading map

The Rust backend lives at `/Users/jpvanzyl/Workspaces/Qictrader/qictrader-backend-rs/`. Treat as **read-only** from this mobile repo.

## Stack

- Rust (stable)
- Axum (HTTP framework)
- SQLx (PostgreSQL, compile-time-checked queries)
- Tokio
- `tracing` for logging
- `utoipa` for OpenAPI generation (in flight)
- Deployed on Heroku (`qictrader-backend-rs` app)

## Project structure

```
qictrader-backend-rs/
  src/
    types/           Domain types, enums, Money — start here for any new feature
    models/          Database row structs (sqlx::FromRow)
    repo/            Database queries (pure SQL, no logic)
    services/        Business logic (pure where possible)
    api/             Axum handlers (thin, delegate to services)
    extractors/      Axum extractors (auth, validation)
    middleware/      Axum middleware (CSRF, security headers, etc.)
    config.rs        Env var parsing + boot guards
    app.rs           Route definitions + router builder
    main.rs          Entry point
  migrations/        SQLx migrations (filename pattern: YYYYMMDDHHMMSS_TICKET-ID_desc.up.sql)
  tests/             Integration tests (per ticket: pentest_NNN_*.rs, etc.)
  docs/              State machines, schema docs, bug write-ups
```

## Map: where to read for what

### For auth contract

| Read | What's there |
|---|---|
| `src/types/enums.rs` | `Role` enum + `has_*_access()` methods |
| `src/api/auth.rs` | Login, signup, 2FA, refresh, logout handlers |
| `src/services/auth.rs` | Argon2id, JWT issuance, audience validation |
| `src/extractors/auth.rs` | `AuthUser` extractor, `OptionalAuthUser`, role gates |
| `src/middleware/csrf.rs` | CSRF rules (exemption list, double-submit cookie pattern) |
| `src/services/totp.rs` | TOTP / 2FA logic |

### For wallet contract

| Read | What's there |
|---|---|
| `src/api/wallet.rs` | `/wallet`, `/wallet/withdraw`, etc. handlers |
| `src/services/wallet.rs` | Balance computation, wallet creation |
| `src/services/wallet_crypto.rs` | AES-256-GCM encryption, key versioning |
| `src/services/withdrawal.rs` | Withdraw flow + fee calc |
| `src/services/deposit_webhook.rs` | Alchemy webhook handler |
| `src/services/helius_webhook.rs` | Helius (Solana) webhook handler (TKT-214.4) |
| `src/repo/wallet.rs` | Wallet queries |
| `src/repo/wallet_transactions.rs` | Tx history queries |

### For trade / escrow contract

| Read | What's there |
|---|---|
| `src/api/trades.rs` | Trade lifecycle handlers (initiate, mark paid, release, cancel, dispute) |
| `src/services/escrow_release.rs` | Settlement math (including resold-trade 3-way split) |
| `src/services/escrow_refund.rs` | Refund flow |
| `src/services/ledger.rs` | Append-only ledger (DB-enforced) |
| `src/types/enums.rs` | `TradeStatus`, `EscrowStatus` with `can_transition_to()` + `is_terminal()` |
| `src/repo/trades.rs` | Trade queries |
| `docs/intended-entity-state-machines.md` | Design intent for trade/escrow |
| `docs/as-built-state-machines.md` | Implementation reality (flag divergences) |

### For offers / marketplace

| Read | What's there |
|---|---|
| `src/api/offers.rs` | Offer CRUD handlers |
| `src/services/offers.rs` | Offer lifecycle + visibility rules |
| `src/types/enums.rs` | `OfferStatus`, `OfferType` |
| `src/repo/offers.rs` | Offer queries |

### For KYC

| Read | What's there |
|---|---|
| `src/api/kyc.rs` | KYC status + provider handoff handlers |
| `src/services/kyc_provider.rs` | Provider abstraction (Didit + SumSub) |
| `src/services/didit.rs` | Didit integration |
| `src/services/sumsub.rs` | SumSub integration |
| `src/types/enums.rs` | `KycStatus`, `KycTier` |

### For notifications

| Read | What's there |
|---|---|
| `src/api/notifications.rs` | Notification list + preferences |
| `src/services/notifications.rs` | Notification dispatch (email, push, in-app) |
| `src/services/push.rs` | Push token registration + send (may need extension for mobile) |

### For state machines (READ THIS FIRST when working on trade-related features)

| Read | What's there |
|---|---|
| `docs/intended-entity-state-machines.md` | Design intent — what we're building toward |
| `docs/as-built-state-machines.md` | Reality — what actually exists today |

If reality differs from intent, **flag it** — don't silently perpetuate divergence. File a ticket on the backend board.

### For DB schema

| Read | What's there |
|---|---|
| `docs/database-schema.md` | Generated schema summary |
| `migrations/` | Ordered migration history (filenames tell the story) |
| `src/models/` | Rust structs mirroring tables |

## Key boot guards (relevant for mobile env-handling)

These are enforced at backend startup. They tell you what env vars need to be set per environment:

- `JWT_SECRET` ≥ 32 chars, not the known leaked default
- `WALLET_ENCRYPTION_KEY` required + validated
- `ALLOWED_ORIGINS` explicit allow-list (empty ⇒ no CORS, never `*`)
- `APP_ENV` ∈ {production, staging, development}
- Mainnet RPC URLs refused in non-prod envs
- KYC provider required when DB has users

If mobile sees a 5xx that looks like a config issue, check these. The backend refuses to boot if any are wrong.

## Pentest report

`Qictrader/architecture/QIC_TRADER_PENTEST_REPORT.md` — the full 2026-04-17 white-box pentest. Read it before scoping any new auth/security feature. Findings became `pentest_NNN_*.rs` test files.

## Sample lookup workflow

When mobile needs to call `/api/v1/wallet/withdraw`:

```bash
# 1. Find the handler
rg "fn .*withdraw" /Users/jpvanzyl/Workspaces/Qictrader/qictrader-backend-rs/src/api/

# 2. Read the handler signature
Read src/api/wallet.rs (lines around the handler)

# 3. Find the request struct
rg "struct WithdrawRequest" /Users/jpvanzyl/Workspaces/Qictrader/qictrader-backend-rs/src/

# 4. Read the struct — note camelCase serde rename
Read src/api/wallet.rs (struct definition)

# 5. Find the response struct
# Usually defined adjacent to the handler

# 6. Check the service layer for business rules
Read src/services/withdrawal.rs

# 7. Check validation rules
Read src/services/validation.rs (or look for inline checks in the handler)

# 8. Confirm auth check exists
# Handler should call .require_*() or check resource ownership

# 9. Now you know what to send + expect — implement on mobile
```
