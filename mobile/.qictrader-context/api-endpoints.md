# Backend API endpoints — quick reference

Curated summary of the key endpoints the mobile app will consume. **Always cross-reference the Rust source** in `/Users/jpvanzyl/Workspaces/Qictrader/qictrader-backend-rs/src/api/` before writing any API call — this doc may be stale.

## Base URLs

| Env | URL |
|---|---|
| Production | `https://qictrader-backend-rs-13eab0516d9a.herokuapp.com` (will move to `api.qictrader.com`) |
| Staging | (resolve via web's `.env.staging` or ask JP — name pattern `qictrader-backend-rs-staging-*.herokuapp.com`) |
| Local dev | `http://localhost:8080` |

All endpoints prefixed with `/api/v1`.

## Auth

| Endpoint | Method | Auth | Purpose |
|---|---|---|---|
| `/auth/signup` | POST | none | Create account |
| `/auth/login` | POST | none | Email+password login; returns 200 + tokens OR 200 + `requires2fa: true` + 2fa challenge token |
| `/auth/verify-2fa-login` | POST | 2fa-token | Submit TOTP code |
| `/auth/refresh-token` | POST | refresh-token | Get new access token |
| `/auth/logout` | POST | bearer | Revoke current session |
| `/auth/logout-all` | POST | bearer | Revoke all sessions for user |
| `/auth/forgot-password` | POST | none | Send reset email |
| `/auth/reset-password` | POST | none | Set new password via token |
| `/auth/verify-email` | POST | none | Verify email via token |
| `/auth/resend-verification` | POST | bearer (unverified ok) | Resend verification email |
| `/auth/2fa/setup` | POST | bearer | Generate TOTP secret + QR |
| `/auth/2fa/enable` | POST | bearer | Confirm TOTP code, enable 2FA |
| `/auth/2fa` | DELETE | bearer + TOTP | Disable 2FA |
| `/auth/oauth/apple` | POST | none | Apple identity token → session |
| `/auth/oauth/google` | POST | none | Google ID token → session |

**Auth header:** `Authorization: Bearer <jwt>` OR cookie `qic_access=<jwt>; HttpOnly; Secure`. Mobile uses Bearer.

**Token audiences:**
- Access: `aud=qictrader-api`
- Refresh: `aud=qictrader-refresh`
- 2FA challenge: `aud=qictrader-2fa`

## User / Profile

| Endpoint | Method | Purpose |
|---|---|---|
| `/me` | GET | Current user profile + KYC tier + flags |
| `/me` | PATCH | Update display name, bio, country |
| `/me/avatar` | POST | Upload avatar image |
| `/me/preferences` | GET/PUT | Currency, theme, notification prefs |
| `/me/payment-methods` | GET/POST/PUT/DELETE | Bank / mobile money methods |
| `/me/sessions` | GET | List active sessions |
| `/me/sessions/{id}` | DELETE | Revoke specific session |
| `/me/legal-acceptances` | GET/POST | T&Cs / privacy versioned acceptances |
| `/me/change-password` | POST | Current + new password |

## Wallet

| Endpoint | Method | Purpose |
|---|---|---|
| `/wallet` | GET | All wallet balances (BTC, ETH, USDT*, SOL, TRX, ZAR, NGN-if-flagged) |
| `/wallet/transactions` | GET | Tx history with filters (type, status, currency, dateRange, page, limit) |
| `/wallet/transactions/{id}` | GET | Tx detail |
| `/wallet/deposit/{currency}` | GET | Deposit address (query: `network=erc20\|trc20\|spl`) |
| `/wallet/withdraw` | POST | Withdraw to external address (body: `currency`, `network`, `amount`, `address`, `memo?`, `twoFactorCode?`) |
| `/wallet/withdraw/fee-preview` | POST | Preview fee + receivable amount |
| `/wallet/transfer` | POST | Internal transfer between user's own wallets |

**Honor `SOLANA-FEATURE-FLAG-001`** — don't paint SOL routes for non-allowed users (backend rejects with 403).

## Market / Prices

| Endpoint | Method | Purpose |
|---|---|---|
| `/market/prices` | GET | All supported crypto prices in user's fiat |
| `/market/prices/{symbol}/history` | GET | Historical chart data (range: 24h, 7d, 30d, 90d, 1y) |
| `/market/snapshot` | GET | Aggregate snapshot (prices + fx_per_currency map) |

## Marketplace / Offers

| Endpoint | Method | Purpose |
|---|---|---|
| `/offers` | GET | List active offers (filters: `offerType`, `crypto`, `fiat`, `paymentMethod`, `minAmount`, `sort`, `page`) |
| `/offers/{id}` | GET | Offer detail |
| `/offers` | POST | Create offer |
| `/offers/{id}` | PUT | Edit offer (limited fields) |
| `/offers/{id}/status` | PUT | Pause / resume / close |
| `/offers/{id}` | DELETE | Soft-delete |
| `/offers/me` | GET | User's own offers |
| `/offers/{id}/resell` | POST | Create a resold offer with markup |

## Trades / Escrow

| Endpoint | Method | Purpose |
|---|---|---|
| `/trades` | POST | Initiate trade from an offer (body: `offerId`, `amount`, `paymentMethodId`) |
| `/trades` | GET | User's trades (filters: `status`, `role`, `dateRange`) |
| `/trades/{id}` | GET | Trade detail (incl. state machine, counterparty, payment instructions, audit) |
| `/trades/{id}/mark-paid` | POST | Buyer marks payment sent |
| `/trades/{id}/release` | POST | Seller releases escrow (body: `twoFactorCode?`) |
| `/trades/{id}/cancel` | POST | Cancel (body: `reason`) |
| `/trades/{id}/dispute` | POST | Open dispute (body: `reason`, `description`, `evidenceUrls`) |
| `/trades/{id}/proof-upload-url` | POST | Get presigned S3 URL for POP upload |
| `/trades/{id}/messages` | GET | Chat history (paginated) |

**Real-time chat:** Socket.IO at `/socket.io/`. Auth via `?token=<jwt>` query string (or `Authorization` header in handshake). Same JWT as REST.

## KYC

| Endpoint | Method | Purpose |
|---|---|---|
| `/kyc/status` | GET | Current tier + per-tier requirements + benefits |
| `/kyc/didit/session` | POST | Start a Didit verification session — returns URL to open in WebView |
| `/kyc/sumsub/access-token` | POST | Issue SumSub WebSDK token (fallback provider) |
| `/kyc/documents` | POST | Direct upload (legacy / overrides) |

**Webhooks** (NOT consumed by mobile, but relevant context):
- `/webhooks/kyc/didit` — Didit completion → updates user tier
- `/webhooks/kyc/sumsub` — SumSub completion → updates user tier
- `/webhooks/alchemy` — Deposit detection (EVM)
- `/webhooks/helius` — Deposit detection (Solana, recently added — TKT-214.4)

## Affiliate

| Endpoint | Method | Purpose |
|---|---|---|
| `/affiliate/dashboard` | GET | Stats: earnings, pending, referrals, rank |
| `/affiliate/link` | GET | User's referral link + code |
| `/affiliate/commissions` | GET | Commission ledger (paginated) |

## Notifications

| Endpoint | Method | Purpose |
|---|---|---|
| `/notifications` | GET | In-app notification list (paginated) |
| `/notifications/{id}/read` | POST | Mark notification read |
| `/notifications/read-all` | POST | Mark all read |
| `/notifications/preferences` | GET/PUT | Per-topic / per-channel toggles |
| `/notifications/devices` | POST | Register FCM/APNs token (mobile-specific; **may need to be added — coordinate with backend**) |
| `/notifications/devices/{token}` | DELETE | Unregister on logout |

## Config (public, no auth)

| Endpoint | Method | Purpose |
|---|---|---|
| `/config/public` | GET | Public flags: `enableNgn`, `supportedNetworks`, `solanaFlagEnabled`, etc. |

## Health

| Endpoint | Method | Purpose |
|---|---|---|
| `/health` | GET | Liveness probe |
| `/health/ready` | GET | Readiness probe (DB connectivity) |

## Error response shape

All errors return:

```json
{
  "error": {
    "code": "VALIDATION_ERROR" | "UNAUTHORIZED" | "FORBIDDEN" | "NOT_FOUND" | "RATE_LIMITED" | "INTERNAL_ERROR" | ...,
    "message": "Human-readable summary",
    "details": { /* per-error context, e.g. field errors */ }
  }
}
```

Mobile API client should:
- Parse this shape into a typed `ApiError` discriminated union
- Map `UNAUTHORIZED` → trigger token refresh → retry once → if still 401, route to login
- Map `RATE_LIMITED` → respect `Retry-After` header
- Map `VALIDATION_ERROR` → display field errors inline in forms
- Map `FORBIDDEN` → show 'You do not have access' screen (and check KYC tier requirements where relevant)

## Backend repo paths (for source-of-truth lookups)

| Want | Read |
|---|---|
| Route definitions | `qictrader-backend-rs/src/app.rs` |
| Auth logic | `qictrader-backend-rs/src/api/auth.rs`, `src/services/auth.rs` |
| Wallet logic | `qictrader-backend-rs/src/api/wallet.rs`, `src/services/wallet.rs` |
| Trade logic | `qictrader-backend-rs/src/api/trades.rs`, `src/services/escrow_release.rs` |
| Offer logic | `qictrader-backend-rs/src/api/offers.rs` |
| KYC | `qictrader-backend-rs/src/api/kyc.rs`, `src/services/didit.rs`, `src/services/sumsub.rs` |
| Domain enums | `qictrader-backend-rs/src/types/enums.rs` |
| DB models | `qictrader-backend-rs/src/models/` |
| Config (env vars) | `qictrader-backend-rs/src/config.rs` |
