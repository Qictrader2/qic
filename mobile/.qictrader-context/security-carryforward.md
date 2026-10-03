# Security & Hardening Carry-Forward — QicTrader → Next Apps

**Date compiled:** 2026-05-20
**Compiled from:** Trello backlog (Schalk + Alfred), shipped Trello tickets, codebase inventory, `architecture/QIC_TRADER_PENTEST_REPORT.md`, `.cursor/rules/`, and the migration history.
**Purpose:** Catalog of security patterns, controls, and lessons we baked into this build so they can be replicated in the new apps being scaffolded tomorrow. Also lists the open security wishlist that Schalk and Alfred raised but have not been closed yet.

---

## TL;DR — what to copy on day 1 of a new app

1. **Engineering rules first.** Copy `.cursor/rules/credential-hygiene.mdc`, `.cursor/rules/engineering-principles.mdc`, `.cursor/rules/tests-before-staging.mdc`, `CLAUDE.md` and `AGENTS.md` before writing any feature code. These rules carried more weight than any single fix during this build.
2. **Auth scaffolding.** Argon2id passwords + JWT with separate `aud` for access / refresh / 2FA + DB-backed sessions with server-side revocation + rate-limit on `/auth/*` from the first commit. Don't bolt on later.
3. **Money plumbing.** Append-only ledger (DB trigger forbids UPDATE/DELETE) + typed `LedgerEntryType` enum + pure settlement functions + property tests for conservation of value. Add `incidents` table for edge cases that can't settle (KS-001 pattern).
4. **Impossible states.** Domain enums (TradeStatus, EscrowStatus, OfferStatus, KycStatus) with `can_transition_to()` + `is_terminal()` + transition tests. No `_ =>` catch-alls.
5. **Boot guards.** Refuse to start if `JWT_SECRET` is weak/default, if mainnet RPC is configured on staging, if KYC provider is unwired in production, if `WALLET_ENCRYPTION_KEY` differs in length/strength from spec.
6. **Tests as security infra.** Every pentest finding gets a regression test file named `pentest_NNN_*.rs`. Two-user IDOR fixture is mandatory for any resource API.
7. **Credential rotation calendar.** Every secret has a rotation procedure documented before it's ever set in prod.

---

## A. OPEN SECURITY BACKLOG — Schalk + Alfred wishlist (not yet closed)

These are the security-flavoured cards Schalk or Alfred created that are still in **Backlog** or **Acknowledged** across the Qictrader / Project One boards. Treat this as the "still to do on QicTrader, also worth scoping into the new apps" list.

### Schalk Dormehl — SEC- series (2026-03-23)

Schalk's review charter — three top-down audits we never formally closed:

- **SEC-001: Full logging security review** — `https://trello.com/c/SK75yL15`
  > Make sure there is no logging to the front end or unintentional logging by the back end of any secure information or any sensitive information.
- **SEC-002: Blockchain funds security review** — `https://trello.com/c/YaAAxuic`
  > Confirm that there is absolutely no path for unauthorized movement of funds out of the treasury wallets.
- **SEC-003: Full app flow security review** — `https://trello.com/c/EcPvNmEM`
  > Ensure that users can't cause events that move funds that don't belong to them, and that users can't access funds or indirectly move funds that don't belong to them. Must be extremely comprehensive.

**Carry-forward:** Treat these three as the standing audit checklist for any custodial/fintech build. Run each at v1.

### Alfred — operational + protocol gaps

- **Rotate ALL production keys** — `https://trello.com/c/QiUTLtKO`
  Full 14-step rotation plan (Phase 1 API keys → Phase 4 treasury keys). Includes JWT_SECRET, Firebase, Google OAuth, SMTP, Redis, DATABASE_URL, S3/R2, Treasury wallets, Telegram bot. The plan itself is the deliverable — copy it as the rotation runbook.
- **Audit Trail of trade fees** — `https://trello.com/c/W61BFWa1`
  Confirm escrow fees + affiliate funds are reconciled and affiliates actually receive their funds.
- **INCIDENT-001: Investigate unexplained balance inflation** — `https://trello.com/c/mUpNeWtL`
  User balance went 200 → 1000 ZAR with no deposit. Story explicitly lists the investigative checks (duplicate ledger rows, double payout path, race in `complete_trade`, duplicate WalletTransaction). Add a DB constraint or guard to prevent recurrence. **Replicate the diagnostic checklist as standing health checks in the new apps.**
- **GAP-022 / GAP-023** — `U7yxXhzi`, `192Sy3HQ`
  Update intent doc with new entities/state changes; property-based tests for dispute split conservation law. Schalk-style invariants.
- **AUTH-015H: Align Telegram signup with email verification gate** — `https://trello.com/c/hGEaHidu`
  Telegram signup currently bypasses email verification (returns tokens immediately with `email_verified=false`). **General principle: every signup channel must funnel through the same verification gate, no shortcuts.**
- **AUTH-006 / 012 / 013: Apple OAuth, Telegram login/signup** — `A6wNW4xx`, `HtswTyjs`, `XMfVzOVv`
  Carry over the JWT audience pinning rule from PENTEST-006 when wiring these.
- **TELEGRAM-008: Manage Security Settings via Telegram** — `https://trello.com/c/soYsKELn`
  `/security` command surfaces 2FA status, last login, active session count + actions (enable 2FA, view backup codes, logout all devices, view security logs). Good shape for a security-control surface on any bot channel.
- **MPR-016: DFD 8 — Eligibility + Visibility Gating (Fundable + Enabled + Allowed)** — `https://trello.com/c/rra7WG24`
  Three-way gate (Fundable + Enabled + Allowed) for offer visibility. Replicate as the canonical "may this user even see/use this resource" predicate.
- **MOD: Add user wallet/ledger activity tab to moderator user review page** — `https://trello.com/c/KQjeyA0i`
  Moderator visibility into user's financial activity for dispute work.
- **Map out moderator dispute flow** — `https://trello.com/c/ikigLo8M`
  Moderator visibility, notifications, escalation, management tracking, audit trail of moderator actions, dashboard stats.
- **SWEEP-REVIEW-001: Independent review of deposit-sweep code path (Tron + Solana)** — `https://trello.com/c/BrQ4tp6V`
- **RPC-NODES-AUDIT-001: Review node structure for client accounts (public vs private RPCs) and related security** — `https://trello.com/c/gQ1Zar8X`
- **KYC-STUCK-001: 2 users in partial-Didit-completion limbo (UX gap, not webhook bug)** — `https://trello.com/c/Xot1N63P`
- **Need a Design System** — `https://trello.com/c/xlsJjec6`
  Indirect security: consistent auth/error UI reduces phishing UX drift.

**Read full descs in Trello — many include acceptance criteria worth copying verbatim into new tickets.**

---

## B. SHIPPED SECURITY WORK — what we actually built during this build

Every entry below is **already in the QicTrader codebase**. Use the file paths as recipes.

### B1. Authentication & sessions

| Pattern | Where | What to copy |
|---|---|---|
| Argon2id password hashing | `qictrader-backend-rs/src/services/auth.rs` | Per-password random salt via `SaltString::generate`; default Argon2 params; `PasswordVerifier`. |
| JWT with typed audiences | `services/auth.rs`, `extractors/auth.rs` | `aud=qictrader-api` for access, `qictrader-refresh` for refresh, `qictrader-2fa` for the 2FA challenge token. `Role::from_access_token_claim` rejects refresh/2FA tokens on API routes. |
| iss/aud/jti claims (JWT-AUDIT-2026-05) | `services/auth.rs`, `tests/jwt_audit_2026_05_claims.rs` | Pin issuer + audience, add `jti` early even if revocation isn't wired yet. |
| Generic JWT error responses (PENTEST-006) | `services/auth.rs` | Client gets "invalid or expired token"; library details only in server logs. |
| Required strong `JWT_SECRET` | `qictrader-backend-rs/src/config.rs`, `src/main.rs` | clap requires the env var (no dev default); runtime rejects the known leaked default and any secret < 32 chars. Belt-and-suspenders. |
| DB-backed sessions + server-side revocation | `extractors/auth.rs`, `repo/user.rs`, `api/auth.rs` | Access tokens embed `session_id`; `find_active_session` runs on every request; logout/admin-deactivate/password-change all `DELETE FROM sessions`. |
| Centralized token validation across transports | `AuthUser::validate_token()` | One function used by REST handlers, Socket.IO handshake, `/ws`, and attachment `?token=` downloads. |
| Refresh tokens (30-day TTL) | `services/auth.rs`, `tests/auth_refresh_tokens.rs` | Refresh JWT with own audience; refresh endpoint enforces the audience. |
| 2FA / TOTP gate (post-password) | `services/totp.rs`, `api/auth.rs`, `Frontend/e2e/tests/two-factor-auth.test.ts` | 5-min `2fa_session` token after password; TOTP verification before issuing access+refresh; property tests on `require_2fa_if_enabled`. |
| Email verification gate (separate from KYC) | `api/auth.rs`, migrations `20260402000001_auth015_email_verified.*`, `20260402000002_auth015_email_verification_tokens.*` | Login blocked until verified; OAuth users backfilled as verified at first link. |
| Auth endpoint rate limiting (PENTEST-002) | `services/rate_limit.rs`, `api/auth.rs`, `tests/pentest_002_auth_rate_limit.rs` | Fixed-window per-IP **and** per-email on login/signup/forgot-password; 429 + `Retry-After`; generic body; bucket reset on successful login. |
| Password-change session hygiene | `api/users.rs`, `tests/jwt_audit_2026_05_revocation.rs` | `delete_other_sessions` keeps the current device, revokes everywhere else. |
| Admin role change / deactivation → wipe sessions | `api/admin.rs`, `tests/jwt_audit_2026_05_revocation.rs` | `delete_all_sessions` whenever privilege changes. |
| Frontend auth store with safe rehydrate | `Frontend/src/store/auth-store.ts`, `lib/auth.ts` | Zustand persist; rehydrate checks expiry and logs out; no token preview in logs. |

### B2. Authorization & roles

| Pattern | Where | What to copy |
|---|---|---|
| `Role` enum + explicit capability methods | `qictrader-backend-rs/src/types/enums.rs` | `User`, `Moderator`, `Admin`, `SuperAdmin`, `Investor`; methods like `has_moderator_access`, `has_admin_access`, `has_super_admin_access`, `has_war_room_access`. |
| Role gate extractors on `AuthUser` | `extractors/auth.rs` | `require_moderator`, `require_admin`, `require_super_admin`, `require_war_room_access`, `require_self_or_admin`. Thin, reusable, composable. |
| Read-only `Investor` role (INVESTOR-ROLE-001) | migration `20260518090000_INVESTOR_ROLE_001_user_role_investor.up.sql`, `api/admin_war_room.rs`, `Frontend/src/components/features/admin/AdminGuard.tsx`, `Frontend/src/hooks/use-admin-auth.ts` | Separate "owner metrics" reader from ops admin. Moderators explicitly excluded from GMV/revenue. |
| Trade participant IDOR checks | `api/trades.rs` | `is_trade_participant()` on every trade/escrow/ledger handler; moderator/admin bypass where intentional. |
| Role-specific action guards | `api/trades.rs` | Only buyer marks paid; only seller/vendor/admin releases; dispute rules encoded per role. |
| Public-listing `OptionalAuthUser` | `extractors/auth.rs` | Marketplace serves anonymous, personalizes when logged in (blocked-user filter, etc.). |
| KYC mandatory gate on offers (KYC-MANDATORY-001) | `tests/kyc_mandatory_001_gate.rs` | Feature-flagged 403 `KYC_REQUIRED` for money-moving actions. |
| Legal acceptance gate (QICT-262) | migration `20260514120000_QICT_262_user_legal_acceptances.up.sql`, `services/legal.rs`, `Frontend/src/store/auth-store.ts` | Versioned ToS / privacy / cookies with IP + UA audit columns; re-acceptance modal blocks app. |
| Admin user restriction fields | migration `20260313140000_admin002_user_restriction_fields.up.sql` | `is_suspended`, `is_banned`, `suspended_until` with partial indexes. |
| KYC L3 document download authz | `tests/kyc_l3_doc_download_authz.rs` | Treat KYC evidence as the most sensitive PII in the system; explicit auth on downloads. |
| Frontend admin guard from JWT role only (PENTEST-016) | `AdminGuard.tsx`, `use-admin-auth.ts` | No `NEXT_PUBLIC_ADMIN_EMAILS` shipped in client bundles. |

### B3. Wallet / custodial key management

| Pattern | Where | What to copy |
|---|---|---|
| AES-256-GCM wallet encryption | `qictrader-backend-rs/src/services/wallet_crypto.rs` | Per-user HMAC-derived keys, random nonces, domain separation string `qictrader-wallet-v1`. |
| JWT ↔ wallet key decoupling + key versioning | `services/wallet_crypto.rs`, migration `20260507081100_JWT_WALLET_DECOUPLE_001_wallet_key_version.up.sql` | v0 = derived from `JWT_SECRET` (legacy), v1 = `WALLET_ENCRYPTION_KEY`; `wallet_key_version` column on custodial + escrow rows. **Rotating the JWT secret no longer requires re-encrypting wallets.** |
| Re-encryption CLI | `qictrader-backend-rs/src/bin/reencrypt_wallets.rs` | Idempotent paginated v0→v1 migration; loud abort on decrypt failure. |
| Dual-key pure tests | `tests/jwt_audit_2026_05_dual_key.rs`, `tests/custodial_wallet_crypto.rs` | Proves v0/v1 round-trip, re-encrypt path, JWT rotation independence. |
| BIP-39 / BIP-32 / BIP-44 / SLIP-10 HD derivation | `services/wallet_crypto.rs` | Mnemonic generation + ed25519 paths for chain keys. |
| Per-escrow private key storage | migration `20260306170000_add_escrow_wallet_private_key.up.sql` | Escrow keys encrypted with same versioned scheme as custodial. |
| Config redaction in `Debug` | `config.rs` | Hand-written `Debug` redacts JWT, DATABASE_URL, treasury keys, RPC URLs, API keys. |

### B4. Money movement & audit trail

| Pattern | Where | What to copy |
|---|---|---|
| Append-only ledger via DB trigger (ES-006) | migration `20260312120000_es006_ledger_append_only.up.sql`, `services/ledger.rs`, `repo/ledger.rs` | UPDATE/DELETE forbidden at DB level. Single source of truth for money. |
| Typed `LedgerEntryType` enum | `services/ledger.rs`, `types/enums.rs` | Escrow lock / release, fees, deposits, withdrawals, refunds, reseller fees, reversals — all typed. No string types. |
| Property / invariant tests on ledger | `tests/ledger_property_tests.rs`, `tests/ledger_stories.rs`, `tests/mpr001_trade_escrow_ledger_invariants.rs` | Conservation of value tested as property, not just happy path. |
| Wallet balance debit-at-lock (ARCH-003) | migration `20260313130000_arch003_wallet_balance_debit_at_lock.up.sql`, `tests/arch003_wallet_balance_props.rs` | Funds locked in DB **before** escrow promises them. |
| Quote audit trail (MPR-005) | migration `20260305100000_025_mpr005_quote_audit.up.sql` | Persisted quote inputs to settle pricing disputes deterministically. |
| Reseller settlement incident table (KS-001) | migration `20260507070700_KS001_reseller_settlement_incidents.up.sql`, `services/escrow_release.rs`, `tests/reseller_settlement_incident.rs` | When fee math returns `None`, buyer still settles atomically and an incident row (UNIQUE on `trade_id`) is created. Admin queue resolves later with ledger link. **Never silently drop commission on edge cases.** |
| Pure 3-way reseller settlement | `services/escrow_release.rs` (`compute_reseller_settlement()`), `tests/escrow_release.rs` | Atomic split between buyer / reseller / platform in one transaction; settlement math is pure + tested. |
| Treasury swap ledger types (GAP-004) | migration `20260315180000_gap004_treasury_swap_ledger_type.up.sql` | Treasury ops belong in the same audit trail. |
| Failed ledger entry archive (GAP-005) | migration `20260318130000_gap005_failed_ledger_entries.up.sql` | Failed attempts are persisted, not just successes. |
| Trade forensics playbook | `architecture/TRADE-FORENSICS-2026-04-29.md` | Documented procedure for reconstructing any trade from DB + logs. **Write this on day 1, not after an incident.** |
| Reconciliation regression | `tests/reconciliation_decode_regression.rs`, ticket `QMdCQOXG` | Regression-test reconciliation after schema changes. The original bug was a NUMERIC → i64 decode failure. |
| Auto-sweep platform fees → fee wallet | ticket `MIjqryYg` | Separate the pool wallet from the fee receive wallet. |

### B5. State machines & impossible-state prevention

| Pattern | Where | What to copy |
|---|---|---|
| `can_transition_to()` + `is_terminal()` on every lifecycle enum | `types/enums.rs` | TradeStatus, EscrowStatus, OfferStatus, KycStatus, ResellOfferStatus, etc. + transition tests + proptest. |
| Newtype IDs | `types/` | Compile-time prevention of UUID mix-ups. |
| Wallet transaction status enum (WAL-001) | migration `20260316140000_wal001_wallet_transaction_status_enum.up.sql` | Money statuses are DB enums, not strings. |
| No `_ =>` catch-all on domain enums | `.cursor/rules/engineering-principles.mdc`, `CLAUDE.md`, `AGENTS.md` | Compiler forces every new variant to be handled. |

### B6. Input validation & error handling

| Pattern | Where | What to copy |
|---|---|---|
| Clippy deny `unwrap` / `expect` / `panic` in non-test code | `qictrader-backend-rs/src/lib.rs` | `#![cfg_attr(not(test), deny(...))]` block. |
| `let _ = fallible_call()` ban on money/auth paths | `CLAUDE.md`, `AGENTS.md`, `qictrader-backend-rs/.cursor/rules/engineering.mdc` | Zero tolerance. Pre-push grep `grep -rn 'let _ =' src/` is mandatory. |
| Central `AppError` HTTP error type | `error.rs`, every `api/*` handler | Typed variants for rate-limit, validation, forbidden, etc. No ad-hoc strings. |
| Shared validation helpers | `services/validation.rs` | Validation pushed to reusable pure functions. |
| Fee config bounds at boot | `config.rs::validate()` | Asserts fee BPS ≤ 10000, gas reserve non-negative, etc. **Validate economic params at startup.** |
| "Tests must not silently pass" rule | `Frontend/.cursor/rules/testing.mdc`, `CLAUDE.md` | Bans `if (!res._ok) { return }`, empty catches, `not.toBe(404)` weak assertions. |
| Injection probing suite | `Frontend/e2e/tests/security/injection.test.ts` | API injection regression coverage. |

### B7. Webhooks & external integrations

| Pattern | Where | What to copy |
|---|---|---|
| Alchemy webhook HMAC-SHA256 verification | `api/webhooks.rs`, `services/deposit_webhook.rs` | Sign on **raw body**; constant-time compare; **503 (fail-closed) when signing key unset**. |
| SumSub webhook + outbound HMAC | `services/sumsub.rs`, migration `20260324100000_kyc008_sumsub_integration.up.sql` | `verify_webhook_signature` with constant-time eq; outbound API signed; broad unit test coverage. |
| Didit KYC provider abstraction | `services/didit.rs`, `services/kyc_provider.rs`, `architecture/DIDIT-MIGRATION-SPEC.md`, migration `20260417000001_KYC-DIDIT-006_user_kyc_provider.up.sql` | KYC behind interface; per-provider HMAC; path selects provider; SumSub kept as fallback. |
| KYC session idempotency | `tests/kyc_didit_idempotency.rs` | Prevents duplicate `kyc_submissions` on repeat start. |
| KYC L3 review + admin override + evidence note | `tests/kyc_l3_review.rs`, `tests/kyc_l3_admin_override.rs`, migrations `20260430120100_*`, `20260511170000_*`, ticket `ZXoiUaNP` | Manual L3 review with evidence storage; admin override requires evidence note. |
| Deposit detection vs settlement separation | migration `20260323000000_webhook_deposits.up.sql`, `services/deposit_webhook.rs`, `services/deposit_sweep.rs` | Webhook detects; sweep service settles. Independent retry semantics. |
| Email (Brevo) hygiene | `.cursor/rules/credential-hygiene.mdc` | Documented rotation runbook; SMTP secrets never logged. |

### B8. Operational / boot-time guards

| Pattern | Where | What to copy |
|---|---|---|
| `APP_ENV` tier signaling (PENTEST-001) | `config.rs`, `middleware/security_headers.rs`, `app.rs` (`/robots.txt`) | Adds `X-Environment` header; non-prod returns `X-Robots-Tag: noindex` + `Disallow: /` robots. **Staging must self-identify and refuse to be indexed.** |
| Security response headers (PENTEST-005) | `middleware/security_headers.rs` | HSTS, X-Content-Type-Options, X-Frame-Options DENY, Referrer-Policy, Permissions-Policy, CSP `default-src 'none'`. Apply on JSON APIs too. |
| Mainnet-in-non-prod boot guard (PENTEST-004) | `config.rs`, `main.rs`, `tests/pentest_004_mainnet_guard.rs` | Refuses boot if staging/dev RPCs include `mainnet`. **Prevents accidental mainnet spend from staging.** |
| Production Solana RPC guard (TKT-214.3) | `config.rs`, `main.rs` | Production requires Helius or an explicit non-default `SOL_RPC_URL`. |
| KYC provider boot requirement | `main.rs` | Refuses boot without SumSub or Didit when DB has users; blocks `KYC_PROVIDER_OVERRIDE_ENABLED` in production. |
| Custom domain for backend (PENTEST-009) | ticket `SFpqNIfD` | `api.qictrader.com` with cert scoped to it. Don't expose Heroku/Vercel default domains. |
| CORS from explicit allow-list (PENTEST-003) | `app.rs`, ticket `RaxqSlus` | Empty `ALLOWED_ORIGINS` ⇒ no CORS. Never `*` in non-dev. |
| Heroku app destroy + key rotation (PENTEST-015) | ticket `Ac2Umcxs` | When decommissioning an orphan environment, **rotate every secret it touched**. |
| Credential hygiene as workspace rule | `.cursor/rules/credential-hygiene.mdc` | `heroku config:get NAME --app …` over full dump; pre-push secret grep; rotation table by env var. |
| Deploy pull-before-deploy | `CLAUDE.md`, `.cursor/rules/vercel-deploy-hook-only.mdc`, `ops/deployment.md` | Mandatory `git pull --rebase` before deploy; CLI-only Vercel prod. **Multi-agent safety.** |
| Migration timestamp collision avoidance | `.cursor/rules/engineering-principles.mdc`, `CLAUDE.md` | Multi-agent migration naming + collision check. Prevents silent migration skips. |

### B9. Tests as security infrastructure

**Backend (`qictrader-backend-rs/tests/`):**

- `jwt_audit_2026_05_claims.rs` — iss/aud/jti claims + backward compat
- `jwt_audit_2026_05_dual_key.rs` — wallet encryption v0/v1 + JWT rotation independence
- `jwt_audit_2026_05_pure.rs` — pure JWT/auth logic
- `jwt_audit_2026_05_revocation.rs` — session revocation across admin/password/Socket.IO/attachments
- `auth_refresh_tokens.rs`, `auth_middleware.rs`
- `pentest_001_environment_signals.rs`, `pentest_002_auth_rate_limit.rs`, `pentest_004_mainnet_guard.rs`, `pentest_017_socket_io_auth.rs`
- `reseller_settlement_incident.rs` — KS-001 incident idempotency
- `legal_idor.rs`, `legal_acceptance.rs` — legal endpoint authz
- `kyc_mandatory_001_gate.rs`, `kyc_didit_idempotency.rs`, `kyc_l3_doc_download_authz.rs`, `kyc_l3_review.rs`, `kyc_l3_admin_override.rs`, `kyc_sumsub.rs`
- `gap_stories.rs` — GAP-001 through GAP-017 functional security gaps
- `ledger_property_tests.rs`, `ledger_stories.rs`, `mpr001_trade_escrow_ledger_invariants.rs`
- `arch003_wallet_balance_props.rs`
- `war_room.rs` — war-room auth (investor/admin/moderator boundaries)
- `wallet_withdrawal.rs`, `wallet_lock_lifecycle.rs`
- `escrow_release.rs`, `escrow_refund.rs`
- `custodial_wallet_crypto.rs`
- `reconciliation_decode_regression.rs`

**Frontend (`Frontend/e2e/tests/`):**

- `security/idor.test.ts` — two-user IDOR on offers/trades/users (reference template)
- `security/auth-bypass.test.ts`, `security/injection.test.ts`, `security/legal-idor.test.ts`, `security/rate-limiting-headers.test.ts`
- `regression/pentest-012-robots-meta.test.ts`, `regression/pentest-013-credential-hygiene.test.ts`, `regression/pentest-016-no-admin-email-allowlist.test.ts`
- `regression/investor-role-001-war-room-access.test.ts`
- `two-factor-auth.test.ts`, `moderator-auth-guard.test.ts`

**Rule:** `.cursor/rules/tests-before-staging.mdc` — no staging deploy without passing ticket-N regression tests.

### B10. Engineering rules & process — copy verbatim

These rules carried more weight than any individual fix:

- `.cursor/rules/credential-hygiene.mdc` — rotation triggers, redacted Debug, `heroku config:get NAME` not dump, secret rotation matrix.
- `.cursor/rules/engineering-principles.mdc` — Types First, Make Impossible States Impossible, Auth must authorize specific resource, `let _ =` ban, multi-agent migration safety.
- `.cursor/rules/tests-before-staging.mdc` — no deploy without passing regression tests.
- `.cursor/rules/vercel-deploy-hook-only.mdc` — CLI-only deploys, mandatory pull-before-deploy.
- `.cursor/rules/ops-checklist.mdc` — read `ops/` before any ticket; Trello card movement procedure.
- `CLAUDE.md` + `AGENTS.md` — the same content in both files (Claude and OpenAI agent rules). Project-wide NO SUPPRESSION policy, frontend↔backend contract, design-intent doc pointers.
- `architecture/QIC_TRADER_PENTEST_REPORT.md` + `architecture/PENTEST-SCOPE.md` — the 2026-04-17 white-box pentest. Read it before scoping any new product. Pentest findings became `pentest_NNN_*` test files; that loop is the standard process.

---

## C. KNOWN GAPS — do not replicate as "done"

From the pentest report and code review, these are documented but **not fully closed** in QicTrader. Treat them as the "phase 2 hardening" checklist when scaffolding the new apps:

- **OAuth `aud` pinning incomplete.** `api/auth.rs` (~line 1785) still uses `validate_aud = false` for Apple OAuth. Pin from day 1 in the new builds.
- **`session_id` not mandatory on all access tokens.** Legacy tokens without `session_id` still authenticate during the backward-compat window. New apps: require `session_id` from token v1.
- **Refresh-token rotation not enforced on every refresh.** New apps: rotate the refresh token on every refresh + maintain a revocation list.
- **Tokens in localStorage (frontend).** Ticket exists for httpOnly cookie + CSRF migration but not shipped. New apps: cookies from day 1.
- **Rate limiting is process-local.** Acceptable for a single dyno; needs Redis-backed rate limiter when scaling horizontally.
- **SEC-001 / SEC-002 / SEC-003** (Schalk's three) — never formally closed. Run them.
- **INCIDENT-001** root cause never formally documented in a postmortem. Add a "balance inflation guard" property test on day 1: for any user, ∑(ledger credits) − ∑(ledger debits) == wallet.balance.

---

## D. Carry-forward "starter kit" (distilled)

When you spin up a new app tomorrow, paste this checklist into the first ticket:

1. **Rules layer.**
   - `.cursor/rules/credential-hygiene.mdc`
   - `.cursor/rules/engineering-principles.mdc`
   - `.cursor/rules/tests-before-staging.mdc`
   - `.cursor/rules/vercel-deploy-hook-only.mdc` (or equivalent)
   - `CLAUDE.md` + `AGENTS.md`
2. **Auth layer.**
   - Argon2id passwords
   - JWT with `iss` + typed `aud` (access / refresh / 2FA) + `jti` + `session_id`
   - DB-backed sessions, `find_active_session` on every request
   - Refresh token with own audience, rotated on each refresh
   - 2FA / TOTP gate post-password
   - Email verification gate (separate from KYC)
   - Rate limit `/auth/login`, `/auth/signup`, `/auth/forgot-password` — per-IP and per-identifier — 429 + `Retry-After` + generic body
   - Generic JWT error messages, library details server-side only
   - Session wipe on password change / role change / deactivation
   - httpOnly cookies, not localStorage (don't repeat QicTrader's deferred fix)
3. **AuthZ layer.**
   - `Role` enum with explicit `has_*_access()` methods
   - Role gate extractors (`require_admin`, `require_*_access`)
   - Resource-level IDOR check (participant / ownership) on **every** handler
   - `OptionalAuthUser` for public reads
   - Per-channel signup gate uniformity (Telegram bug from QicTrader)
   - Legal acceptance gate with versioned docs + IP/UA audit
4. **Wallets / secrets.**
   - AES-256-GCM with per-user HMAC-derived key
   - Encryption key **separate** from `JWT_SECRET` (give yourself the JWT-WALLET-DECOUPLE freedom)
   - `wallet_key_version` column on every encrypted row
   - Re-encryption CLI from day 1
   - Custom `Debug` impl on config that redacts every secret
5. **Money.**
   - Append-only `ledger_entries` table with DB-level UPDATE/DELETE trigger
   - Typed `LedgerEntryType` enum, exhaustive match
   - Pure settlement function with property tests for conservation of value
   - Incident table with UNIQUE on the failing entity (trade_id, etc.)
   - Balance debit-at-lock (don't promise funds you haven't deducted)
   - Quote audit trail
   - Failed-ledger-entries archive table
6. **State machines.**
   - Domain enums with `can_transition_to()` + `is_terminal()` + tests
   - No `_ =>` catch-alls
   - Newtype IDs
7. **Boot guards.**
   - Required `JWT_SECRET` ≥ 32 chars, reject default
   - `WALLET_ENCRYPTION_KEY` required and validated
   - Mainnet-in-staging refusal
   - KYC provider boot requirement
   - Fee config bounds (`bps ≤ 10000`, etc.)
   - Required `ALLOWED_ORIGINS` (empty ⇒ no CORS, never `*`)
   - `APP_ENV` tier header + non-prod `noindex` robots
8. **Webhooks.**
   - HMAC verification on the raw body
   - Constant-time compare
   - 503 (fail-closed) when signing key is unset
   - Provider abstraction (KYC) so SumSub-style + Didit-style can coexist
9. **Tests.**
   - Two-user IDOR fixture from day 1
   - `pentest_NNN_*` regression file per finding
   - Property test for ledger conservation
   - Property test for state-machine transition consistency
   - "Tests before staging" rule enforced in CI
10. **Process.**
    - Pentest → ticket → regression test → deploy
    - `commit-all.sh`-style monorepo discipline
    - Tagged prod releases from staging-verified commits
    - Trade forensics playbook written before incidents
    - Rotation calendar (90-day max) + rotation runbook stored next to the keys

---

## E. Pointers — files worth re-reading tomorrow morning

- `architecture/QIC_TRADER_PENTEST_REPORT.md` — full 2026-04-17 pentest report.
- `architecture/PENTEST-SCOPE.md` — scoping doc for a new pentest engagement.
- `architecture/TRADE-FORENSICS-2026-04-29.md` — production trade audit methodology.
- `architecture/DIDIT-MIGRATION-SPEC.md` — KYC provider migration playbook.
- `architecture/HEROKU-MIGRATION-SPEC.md` — backend migration / cutover playbook.
- `architecture/LOAD-TEST-SCOPE.md` — load test scoping.
- `qictrader-backend-rs/docs/intended-entity-state-machines.md` — design intent.
- `qictrader-backend-rs/docs/as-built-state-machines.md` — implementation reality (flag any divergence early).
- `.cursor/rules/credential-hygiene.mdc` — rotation triggers + matrix.

---

## F. Cards I did NOT move or action (per instruction)

This document only **reads** Trello — no cards were moved, edited, archived, or commented on while compiling it. The 39 backlog cards in section A are still in Backlog/Acknowledged exactly where Schalk and Alfred left them.

If you decide tomorrow you want to bulk-clone the SEC-001/002/003 cards into the new project's board, or convert the "Rotate ALL production keys" runbook into a recurring quarterly card, that's a separate ticket-pipeline run.
