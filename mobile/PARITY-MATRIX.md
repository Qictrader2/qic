# Mobile ↔ Web/Backend Parity Matrix

**Status: read-only API smoke pass complete against PRODUCTION (2026-06-11). Visual walkthrough + write-flow verification still pending.**

Backend: `qictrader-backend-rs` on Heroku (prod). Test accounts: buyer + seller (credentials in `ops/local-secrets/`, gitignored), both email-verified, KYC-approved, no 2FA.

> ⚠️ This document does NOT yet support a "verified 1:1" claim. It records:
> (1) which screen-backing endpoints are confirmed working against prod,
> (2) endpoint drift found and fixed, (3) known gaps, (4) what remains.

## 1. Read-only smoke pass (buyer account, prod)

| Screen | Endpoint | Status |
|---|---|---|
| App boot: platform config | `GET /api/v1/config` | ✅ 200 (fixed, was `/config/public` → 404) |
| Login | `POST /api/v1/auth/login` | ✅ 200, tokens + user payload |
| Dashboard/profile | `GET /api/v1/users/me` | ✅ 200 (fixed, was `/me` → 404) |
| Wallet: balances | `GET /api/v1/wallet` | ✅ 200 `{balances}` |
| Wallet: transactions | `GET /api/v1/wallet/transactions` | ✅ 200 `{data, pagination}` |
| Wallet: portfolio card | `GET /api/v1/prices` (client-side derivation) | ✅ 200 (fixed, was `/market/portfolio-history` → 404, no such feature on web/backend) |
| Wallet: sparklines | `GET /api/v1/prices/{coingecko_id}/history?days=7&mode=line` | ✅ 200 (fixed, was `/market/price-history?currency=BTC` → 404) |
| Fiat balance screen | `GET /api/v1/prices` + `GET /api/v1/prices/fx` | ✅ 200 (fixed, was `/wallets/fiat-equivalent` → 404) |
| Marketplace | `GET /api/v1/offers` | ✅ 200 `{data, pagination}` (empty marketplace at test time) |
| My offers | `GET /api/v1/offers/me` | ✅ 200 (fixed, was `/offers/mine` → 400) |
| Trades: active | `GET /api/v1/trades/active` | ✅ 200 `{trades, total, hasMore}` |
| Trades: history | `GET /api/v1/trades/completed` | ✅ 200 (fixed, was `/trades/history` → 400) |
| Notifications | `GET /api/v1/notifications` | ✅ 200 |
| Notification prefs (read) | `GET /api/v1/notifications/preferences` | ✅ 200 |
| Payment methods | `GET /api/v1/payment-methods` | ✅ 200 (fixed, was `/me/payment-methods` → 404) |
| Affiliate stats | `GET /api/v1/affiliate/stats` | ✅ 200 |
| KYC status | `GET /api/v1/kyc/status` | ✅ 200 |
| Sessions | `GET /api/v1/auth/sessions` | ✅ 200 |
| Support tickets | `GET /api/v1/support/tickets` | ✅ 200 |
| Reseller (stats/active/trades) | `GET /api/v1/reseller/*` | ✅ 200 |
| Withdraw: fee preview | `GET /api/v1/gas/withdrawal-fee/network` | ✅ 200 (fixed, was `/wallet/withdraw/fee-preview` → 404; fee-on-top semantics matched to web) |

## 2. Endpoint drift fixed (mobile → real backend route)

| Was (never existed) | Now | Files |
|---|---|---|
| `GET/PATCH /me` | `/users/me` | `AuthProvider.tsx`, `use-session-lifecycle.ts`, `profile.service.ts` |
| `POST /me/avatar` | `/users/me/avatar` | `profile.service.ts` |
| `POST /me/change-password` | `/users/password/change` | `profile.service.ts` |
| `* /me/payment-methods` | `/payment-methods` | `profile.service.ts` |
| `GET /config/public` | `/config` | `platform-config.ts` |
| `GET /offers/mine` | `/offers/me` | `market.service.ts` |
| `GET /trades/history` | `/trades/completed` | `trade.service.ts` |
| `POST /auth/2fa/setup` (resp `qrCodeUrl`) | `/users/2fa/setup` (resp `qrCode`) | `2fa-setup.tsx` |
| `POST /auth/2fa/verify` | `/users/2fa/verify` | `2fa-setup.tsx` |
| `DELETE /auth/sessions/all` | `POST /auth/logout-all` | `security-settings.tsx` |
| `PATCH /notifications/preferences` | `PATCH /users/me/notifications` | `notifications-settings.tsx` |
| `POST /kyc/didit/start`, `/kyc/sumsub/start` | `POST /kyc/session` (provider-agnostic; Sumsub retired on web) | `kyc.service.ts` |
| `GET /wallet/withdraw/fee-preview` | `GET /gas/withdrawal-fee/network` | `wallet.service.ts` |
| `GET /market/price-history` | `GET /prices/{coingecko_id}/history?days=N&mode=line` | `wallet.tsx` |
| `GET /market/portfolio-history` | none — derived client-side from `/prices` (web does the same) | `wallet.tsx` |
| `GET /wallets/fiat-equivalent` | none — derived from `/prices` + `/prices/fx` (web does the same via CoinGecko) | `fiat-balance.tsx` |

## 3. Known gaps (explicit, deferred)

| Gap | Detail |
|---|---|
| Push device registry | Backend has no `/notifications/register-device` / `unregister-device`. Mobile registration is a documented no-op until backend support lands. |
| Backend analytics relay | No `/analytics/event` route (web uses GA4/GTM directly). Mobile events are Sentry breadcrumbs only. |
| Fiat display currencies | Backend FX reference is USD→ZAR only (`/prices/fx`). NGN/EUR/GBP/KES/GHS have no rate source; fiat screen shows ZAR + USD only. |
| Response-shape audit | Smoke pass verified status + top-level keys, not full field-by-field shape per screen. Visual walk will surface remaining mismatches (e.g. `GET /wallet` returns `{balances}` — mobile wallet list mapping needs visual confirmation). |
| ESLint broken in repo | `eslint-plugin-react-native` is incompatible with ESLint 9 (`context.getScope` crash) — pre-existing; typecheck + jest are the working gates. |

## 4. Remaining for "verified 1:1" sign-off

- [ ] Visual walkthrough on simulator/device, every screen (build ready; human-driven)
- [ ] Write flows on **staging**, not prod: offer create/edit/pause, trade initiate → escrow → mark-paid → release, withdraw (2FA), internal transfer, dispute
- [ ] Trade/escrow two-account walk (buyer + seller accounts exist; need staging funding)
- [ ] Side-by-side copy/validation/fee comparison vs web per flow
