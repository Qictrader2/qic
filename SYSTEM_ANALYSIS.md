# Qictrader — System Analysis

**In-depth analysis of all 4 repositories: security risks, architectural flaws, and code quality.**

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Repository Overview](#2-repository-overview)
3. [Backend — `backend/`](#3-backend--backend)
4. [Frontend — `Frontend/`](#4-frontend--frontend)
5. [Mobile App — `mobile-app/`](#5-mobile-app--mobile-app)
6. [Telegram Bot — `telegram-bot/`](#6-telegram-bot--telegram-bot)
7. [Cross-System Vulnerabilities](#7-cross-system-vulnerabilities)
8. [Master Vulnerability Register](#8-master-vulnerability-register)
9. [Architecture Debt Summary](#9-architecture-debt-summary)

---

## 1. Executive Summary

The Qictrader platform consists of 4 repositories comprising an estimated **80,000+ lines of TypeScript** handling cryptocurrency trading, custodial wallets, escrow, and real money. The system has **significant security vulnerabilities and architectural flaws** that create real risk for user funds and data.

### Critical Numbers

| Metric | Count |
|--------|-------|
| **CRITICAL security vulnerabilities** | 12 |
| **HIGH security vulnerabilities** | 19 |
| **MEDIUM security vulnerabilities** | 14 |
| **LOW security vulnerabilities** | 9 |
| **Total vulnerabilities** | **54** |
| Backend `console.log` statements (should be logger) | 150+ |
| Backend `any` type usage | 200+ |
| Full Firestore collection scans (DoS risk) | 10+ |
| Validation schemas defined but never applied | 6 |
| Committed production secrets (across repos) | 4 files |
| Firestore security rules files | **0** |
| Test coverage percentage | Unknown (17 test files for 337 source files) |

### Top 5 Risks to User Funds

1. **No Firestore security rules** — Any authenticated user could directly read 2FA secrets, modify wallet balances, or elevate their role via the Firebase client SDK
2. **Race condition on wallet transfers** — Balance checked outside transaction; concurrent requests can double-spend
3. **Hardcoded encryption key fallback** — Wallet mnemonics encrypted with `'dev-fallback-key-change-me'` if env var missing
4. **TOTP codes logged to console** — Every 2FA verification attempt writes the valid code to server logs
5. **Production secrets committed to git** — Firebase API keys, QuickNode RPC URLs, and backend URLs in version-controlled env files across 3 repos

---

## 2. Repository Overview

| Repository | Stack | Files | Purpose |
|-----------|-------|-------|---------|
| `backend/` | Node.js, Express 5, TypeScript, Firestore, Redis | 337 | REST API, WebSocket, blockchain integration |
| `Frontend/` | Next.js 16, React 19, TypeScript, Tailwind, Redux+Zustand | 400+ | Web application |
| `mobile-app/` | Expo 54, React Native 0.81, TypeScript, Redux | 392 | iOS/Android application |
| `telegram-bot/` | Grammy 1.38, Bun, TypeScript, Redis | 100+ | Telegram interface |

All repos share:
- TypeScript (strict mode enabled)
- Firebase for authentication
- Socket.IO for real-time features
- The same backend API (`/api/v1`)

---

## 3. Backend — `backend/`

### 3.1 Architecture Summary

**Stack:** Node.js + Express 5.1.0 + TypeScript 5.9.3 + Firebase (Firestore, Auth, Storage) + Redis (ioredis)

**Structure:**
```
backend/src/
├── config/        # Firebase, environment, Redis
├── controllers/   # Route handlers (35+ files)
├── middleware/     # Auth, validation, upload, error handling
├── routes/        # Express routers (18 files)
├── services/      # Business logic (40+ files)
├── types/         # TypeScript interfaces (15+ files)
├── validators/    # Zod schemas
├── websocket/     # Socket.IO handlers
├── db/            # Unused Firestore utility class
├── models/        # Minimal model definitions
├── utils/         # Helpers, logging
├── lib/           # State machine, ledger types
└── scripts/       # Migration scripts
```

### 3.2 Security Vulnerabilities

#### CRITICAL

**SEC-B01: No Firestore Security Rules**

No `firestore.rules` file exists anywhere in the project. The Firebase client SDK is used on both the web and mobile frontends. Without security rules, **every Firestore collection is potentially accessible to any authenticated user** via the client SDK.

Impact: A user could:
- Read `settings` collection → steal 2FA secrets for any user
- Write to `users` collection → set their own `role` to `super_admin`
- Write to `wallets` collection → inflate their balance
- Read `user_wallets` collection → access encrypted mnemonics (then attempt to crack the weak encryption key)
- Delete `ledger_entries` → erase financial audit trail

**SEC-B02: Hardcoded Encryption Key Fallback**

File: `src/services/encryption.ts`

```typescript
const ENCRYPTION_KEY = process.env.WALLET_ENCRYPTION_KEY || 'dev-fallback-key-change-me'
const ENCRYPTION_SALT = process.env.WALLET_ENCRYPTION_SALT || 'qic-trader-salt'
```

If `WALLET_ENCRYPTION_KEY` is not set in the environment, every user's custodial wallet mnemonic is encrypted with a known key. This is the master key to every wallet on the platform.

The `.env.example` file **comments out** `WALLET_ENCRYPTION_KEY`, making it likely a deployment will miss it.

**SEC-B03: 2FA Token Secret Fallback**

File: `src/controllers/auth/login.ts`

```typescript
const TWO_FA_TOKEN_SECRET = process.env.TWO_FA_TOKEN_SECRET || 'qic-trader-2fa-temp-secret-change-me'
```

`TWO_FA_TOKEN_SECRET` is not listed in `.env.example`. Every deployment that doesn't manually add it uses this known secret. An attacker can forge 2FA temporary tokens.

**SEC-B04: 2FA Token Leaks Firebase Credentials**

File: `src/controllers/auth/login.ts`

When a user has 2FA enabled, the login flow generates a temporary JWT. This JWT's payload contains the user's actual Firebase `idToken` and `refreshToken` in plaintext. JWTs are base64-encoded, not encrypted. Anyone intercepting this token can decode it and use the live credentials to bypass 2FA entirely.

**SEC-B05: TOTP Code Logged to Console**

File: `src/controllers/auth/verify2FALogin.ts`

```typescript
console.log('2FA Debug:', {
  expectedCode: authenticator.generate(settings.twoFactorSecret),
  // ...
})
```

Every 2FA verification attempt writes the current valid TOTP code to the server logs. Anyone with log access can bypass 2FA for any user.

**SEC-B06: Rate Limiting Not Applied**

`express-rate-limit` is listed as a dependency. `RATE_LIMIT_WINDOW_MS` and `RATE_LIMIT_MAX_REQUESTS` are defined in `config/environment.ts`. But the rate limiting middleware is **never mounted** on any route or globally in `server.ts`. Every endpoint is unlimited.

**SEC-B07: 2FA Secrets Stored in Plaintext**

File: `src/controllers/users/twoFactor.ts`

TOTP secrets are stored as plaintext strings in the Firestore `settings` collection. Combined with SEC-B01 (no security rules), any authenticated user can read any other user's 2FA secret via the client SDK.

**SEC-B08: Placeholder JWT Utility**

File: `src/utils/jwt.ts`

```typescript
export const signToken = (payload: any): string => {
  return 'placeholder-token'
}
export const verifyToken = (token: string): any => {
  return 'placeholder-verify'
}
```

If any code path calls these functions, authentication is completely bypassed. The functions exist and are importable.

**SEC-B09: Placeholder Authentication Middleware**

File: `src/middleware/authenticate.ts`

```typescript
export const authenticate = (req, res, next) => { next() }
```

A no-op middleware. If accidentally used on any route, it provides zero authentication.

#### HIGH

**SEC-B10: Race Condition on Wallet Transfers (Double-Spend)**

File: `src/controllers/wallet/transfer.ts`

The sender's balance is read *outside* the Firestore batch. Two concurrent transfer requests can both pass the balance check and proceed, resulting in a negative balance.

```typescript
const senderBalance = senderWallet.balances[currency] || 0  // Read outside transaction
if (senderBalance < amount) { /* reject */ }
// ... later, batch.update() deducts — no transactional read lock
```

**SEC-B11: Missing Input Validation on Financial Routes**

Comprehensive Zod schemas exist in `src/validators/index.ts` — `createOfferSchema`, `createTradeSchema`, `sendTokenSchema`, `exportMnemonicSchema`, `linkOnChainEscrowSchema`. None of them are applied to their routes:

| Route | Schema Exists | Applied | Risk |
|-------|:---:|:---:|------|
| `POST /offers` | Yes | **No** | Malformed offer data accepted |
| `POST /trades` | Yes | **No** | Invalid trade amounts accepted |
| `POST /wallet/withdraw` | Yes | **No** | Unvalidated withdrawal requests |
| `POST /wallet/transfer` | Yes | **No** | Unvalidated transfer requests |
| `POST /custodial-wallet/send` | Yes | **No** | Unvalidated send requests |
| `POST /custodial-wallet/export` | Yes | **No** | Unvalidated export requests |

**SEC-B12: Weak Password Policy on Signup**

File: `src/routes/auth.ts`

The signup route uses `userSchemas.create` which requires only 6 characters. A stronger schema (`signupSchema` requiring 8+ chars, mixed case, number) exists in `src/validators/index.ts` but is not used.

**SEC-B13: Firebase API Key Exposed in URLs**

Files: `src/controllers/auth/login.ts`, `src/controllers/auth/refreshToken.ts`, `src/services/passwordVerification.ts`

The Firebase API key is passed as a URL query parameter in REST API calls. URL parameters are logged by web servers, proxies, and CDNs.

**SEC-B14: Missing Authorization on User CRUD**

File: `src/controllers/userController.ts`

- `POST /users/` — creates users **without authentication**
- `PUT /users/:id` — any authenticated user can update **any** user
- `DELETE /users/:id` — any authenticated user can delete **any** user

**SEC-B15: Missing Moderator Check on Escrow Refund**

File: `src/routes/escrow.ts`

`POST /escrow/:id/refund` is missing `moderatorOnly` middleware. Any authenticated user can trigger a refund.

**SEC-B16: Mnemonic Export Without Rate Limiting**

File: `src/controllers/custodialWallet/export.ts`

The mnemonic export endpoint requires password re-verification but has no rate limit. An attacker with a valid session can brute-force the password.

**SEC-B17: Sensitive Data in Logs**

Multiple files log partial API keys, mnemonic decryption results, and user data via `console.log`:

- `src/controllers/auth/login.ts` — logs partial Firebase API key
- `src/services/wallet/send.ts` — logs mnemonic decryption success/failure
- `src/controllers/auth/verify2FALogin.ts` — logs valid TOTP code (SEC-B05)

**SEC-B18: Full Collection Scan on User Search (DoS)**

File: `src/controllers/users/search.ts`

```typescript
const profilesSnapshot = await db.collection('profiles').get()
```

Loads the **entire** `profiles` collection into memory. This endpoint is unauthenticated. An attacker can trigger repeated calls to exhaust server memory.

#### MEDIUM

**SEC-B19: CORS Allows Wildcard with Credentials**

File: `src/server.ts`

If `ALLOWED_ORIGINS` is `*`, the server allows any origin with `credentials: true`. This enables cross-origin requests from malicious sites to make authenticated API calls.

**SEC-B20: `/health/detailed` Exposes System Info Without Auth**

Exposes memory usage, uptime, Redis status, WebSocket connection count, and process stats to anyone.

**SEC-B21: Role Cache Not Invalidated on Change**

File: `src/middleware/roleAuth.ts`

User roles are cached in memory for 5 minutes. A demoted admin retains privileges until cache expires. In multi-instance deployments, the cache is per-process and cannot be invalidated.

**SEC-B22: Redis TLS Disabled**

File: `src/services/redis/RedisClient.ts`

```typescript
tls: { rejectUnauthorized: false }
```

Certificate validation is disabled on Redis TLS connections. Vulnerable to man-in-the-middle attacks.

**SEC-B23: Account Deletion Without Password/2FA Re-verification**

File: `src/controllers/auth/deleteUser.ts`

Account deactivation requires only a valid session token, not password or 2FA. A stolen session can permanently deactivate an account.

**SEC-B24: Trade Messages Not Sanitized**

File: `src/controllers/trades/tradeMessages.ts`

Message `content` is trimmed but not sanitized before storage. If the frontend renders this without escaping, stored XSS is possible.

**SEC-B25: Collection Name Mismatch**

`syncEscrowFromBlockchainController` uses `'escrow'` (singular) while all other code uses `'escrows'` (plural). This endpoint silently fails — it reads from and writes to a non-existent collection.

**SEC-B26: Excessive JSON Body Limit**

File: `src/server.ts`

`JSON_LIMIT` defaults to `10mb`. For a trading API that handles small JSON payloads, this allows resource exhaustion via oversized requests.

#### LOW

**SEC-B27:** Stack traces exposed in error responses when `NODE_ENV !== 'production'`
**SEC-B28:** `uncaughtException` handler logs but doesn't exit — process left in undefined state
**SEC-B29:** No explicit request timeout middleware
**SEC-B30:** Firebase config falls back to `firebase-service-account.json` file — risk of accidental commit

### 3.3 Database & Schema Flaws

**SEC-DB01: No Schema Enforcement (HIGH)**

Firestore is schemaless. TypeScript interfaces exist only at compile time. Any code path writing to Firestore can store arbitrary fields, wrong types, or missing required fields. All type casts like `doc.data() as Trade` are unchecked at runtime.

**SEC-DB02: `users` vs `profiles` Dual Collection (MEDIUM)**

User data is split across two collections with overlapping fields (email, username, displayName, status). Updates to one don't propagate to the other. Role checks use `users` but public endpoints use `profiles`. Signup writes to both independently (not atomically).

**SEC-DB03: Floating-Point Money (MEDIUM)**

All monetary values are JavaScript `number` (IEEE 754 float64). `0.1 + 0.2 === 0.30000000000000004`. Over thousands of transactions, rounding errors accumulate.

**SEC-DB04: No Referential Integrity (HIGH)**

No foreign keys between trades/escrows/wallets/ledger. A write failure mid-operation leaves orphaned records. Many operations use batches (which don't support reads) instead of transactions.

**SEC-DB05: Ledger Not Truly Immutable (MEDIUM)**

`ledger_entries` is designed as append-only, but nothing prevents Admin SDK from updating or deleting entries.

**SEC-DB06: `db/index.ts` Client SDK Dead Code (LOW)**

`FirestoreUtils` class imports from `firebase/firestore` (client SDK) instead of `firebase-admin/firestore`. Unused but dangerous if adopted.

### 3.4 Architectural Flaws

**ARCH-B01: 150+ `console.log` Statements**

The codebase mixes `console.log`, `console.error`, `console.warn`, and the structured `logger` utility inconsistently. Even `src/middleware/errorHandler.ts` (the error handler itself) uses `console`. Critical security data is logged via `console.log` which has no redaction, rotation, or access controls.

**ARCH-B02: 200+ `any` Type Usages**

Despite `strict: true` in `tsconfig.json`, the codebase has 200+ instances of `any`, including:
- `catch (error: any)` — 50+ instances
- `(req as any).requestId` — 30+ instances
- `as any` type assertions — 100+ instances

This defeats TypeScript's safety guarantees.

**ARCH-B03: 10+ Full Collection Scans**

Multiple endpoints fetch entire Firestore collections and filter in memory:

| Endpoint | Collection | Impact |
|----------|-----------|--------|
| `GET /users/search` | `profiles` | All profiles loaded — DoS risk, unauthenticated |
| `GET /users/top` | `profiles` + `offers` | Two full scans in one request |
| `GET /escrow` (list) | `escrows` | All escrows, filter in memory |
| `GET /wallet/transactions` | `wallet_transactions` | No pagination, all docs |
| `GET /mod/stats` | `escrows` | Full scan for statistics |
| `GET /reseller/active` | (reseller offers) | No limit |
| `GET /payment-methods` | (payment methods) | No limit |

**ARCH-B04: Business Logic in Controllers**

Controllers contain complex business logic (BTC wallet locking, escrow validation, balance calculations) instead of delegating to services. This makes logic hard to test and reuse.

**ARCH-B05: Inconsistent Error Handling**

Three different error handling patterns coexist:
1. `handleControllerError` from `responseFormatter.ts`
2. Direct `try/catch` with `res.status().json()`
3. `asyncHandler` wrapper (inconsistently used)

**ARCH-B06: 50+ Direct `process.env` Accesses**

Despite a centralized `config/environment.ts`, many files access `process.env` directly, bypassing validation.

**ARCH-B07: In-Memory State (Scalability Blocker)**

- WebSocket connection tracking falls back to `Map<string, Set<string>>` in memory
- Top traders cache is in-memory with a timer
- Role cache is per-process

None of these work in multi-instance deployments.

**ARCH-B08: 17 Test Files for 337 Source Files**

Test coverage is minimal. No coverage reporting configured. Financial operations (transfers, escrow, wallet sends) appear untested.

---

## 4. Frontend — `Frontend/`

### 4.1 Architecture Summary

**Stack:** Next.js 16.0.10 + React 19.2.3 + TypeScript 5.9.3 + Tailwind 4.1.18 + Redux Toolkit + Zustand + React Query

**Structure:**
```
Frontend/src/
├── app/          # Next.js App Router (45+ routes)
│   ├── (auth)/   # Login, signup, forgot/reset password
│   ├── (dashboard)/ # Dashboard, wallet, trades, settings
│   ├── (main)/   # Landing, marketplace, profiles, help
│   ├── (offers)/ # Offer browse and detail
│   └── layout.tsx
├── components/   # UI components (200+ files)
├── hooks/        # Custom React hooks (30+ files)
├── lib/          # API client, Firebase, auth, env
├── services/     # API service layer (15+ files)
├── store/        # Redux + Zustand stores
├── types/        # Type definitions
└── utils/        # Utilities
```

### 4.2 Security Vulnerabilities

#### CRITICAL

**SEC-F01: Production Secrets Committed to Git**

File: `Frontend/.env.production` (committed, not gitignored)

```env
NEXT_PUBLIC_FIREBASE_API_KEY=AIzaSyD2EIF8GRyBpKEP0rps6RWl4FLgvm7ejGA
NEXT_PUBLIC_FIREBASE_PROJECT_ID=qic-trader
NEXT_PUBLIC_QUICKNODE_BTC_RPC=https://silent-compatible-butterfly.btc.quiknode.pro/50006949f085f404eb7e4b02b80428660aa88137/
NEXT_PUBLIC_TRON_RPC=https://bold-restless-thunder.tron-mainnet.quiknode.pro/c686547078e738e0a0b502b1c2d08fbfb8c0eb8a/jsonrpc
```

Production Firebase API key and **authenticated QuickNode RPC endpoints** (with API keys embedded in the URL path) are committed to version control. These RPC endpoints give direct blockchain access. An attacker can:
- Make unlimited RPC calls billed to your QuickNode account
- Query blockchain data about your platform's wallets
- Potentially submit transactions if the endpoints allow writes

These secrets exist in git history even if later removed.

#### HIGH

**SEC-F02: No Content Security Policy**

File: `Frontend/next.config.ts`

Security headers are set (X-Frame-Options, X-Content-Type-Options, Referrer-Policy) but **no Content-Security-Policy** header. Without CSP, XSS attacks can load arbitrary scripts, exfiltrate data, and steal tokens.

**SEC-F03: Auth Tokens in localStorage**

File: `Frontend/src/store/auth-store.ts`

The auth store uses Zustand with `persist` to localStorage. The `backendToken` (Firebase ID token) is stored in `localStorage` under key `auth-storage`. Any XSS vulnerability can read this token and make authenticated API calls.

**SEC-F04: `dangerouslySetInnerHTML` Usage**

Found in:
- `src/components/ui/chart.tsx` (line 81)
- `src/app/layout.tsx` (line 61)

If the injected HTML comes from user input or unsanitized API responses, this creates XSS attack vectors.

**SEC-F05: Development Firebase Credentials Committed**

File: `Frontend/.env.development`

Development Firebase credentials committed. While less critical than production, this exposes the dev environment and could be used to access dev data.

#### MEDIUM

**SEC-F06: Triple State Management Complexity**

The frontend uses three state management solutions simultaneously:
- Redux Toolkit (UI, offers, prices, WhatsApp, direct trade)
- Zustand (auth, user, wallet, trade, notification, moderation)
- React Query (server state caching)

This creates complexity where state can get out of sync between stores. Auth was migrated from Redux to Zustand mid-project, suggesting ongoing architectural uncertainty.

**SEC-F07: Mixed API Error Handling**

Some API calls handle errors in the service layer, others rely on React Query's error handling, and some have bare try/catch blocks. Inconsistent error handling can lead to silent failures where financial operations fail without user notification.

### 4.3 Architectural Issues

**ARCH-F01: No CI/CD Pipeline**

No `.github/workflows/` directory, no CI configuration. The codebase has `typecheck`, `lint`, and `validate` scripts but nothing enforcing them automatically.

**ARCH-F02: Some `any` Types**

- `src/store/auth-store.ts:164` — `const userAny = state.user as any`
- `src/services/bug-reports-api.ts:195` — `Record<string, any>`
- Various cast-to-any patterns

**ARCH-F03: Broken E2E Tests**

A directory `e2e/tests/broken/` exists, suggesting tests that were abandoned rather than fixed.

---

## 5. Mobile App — `mobile-app/`

### 5.1 Architecture Summary

**Stack:** Expo 54 + React Native 0.81.5 + React 19.1.0 + TypeScript + Redux Toolkit + React Query + NativeWind

**Structure:**
```
mobile-app/
├── app/          # Expo Router (file-based routing)
│   ├── (auth)/   # Login, signup, verify-2fa, forgot-password
│   ├── (tabs)/   # Main tab navigation
│   ├── offers/   # Offer screens
│   ├── trades/   # Trade screens
│   ├── wallet/   # Wallet screens
│   ├── settings/ # Settings screens
│   └── kyc/      # KYC screens
├── components/   # Reusable components
├── config/       # Environment, Firebase, API config
├── services/     # API + WebSocket clients
├── store/        # Redux store + slices
├── hooks/        # Custom hooks
├── types/        # Type definitions
└── utils/        # Utilities
```

### 5.2 Security Vulnerabilities

#### HIGH

**SEC-M01: No Certificate Pinning**

The app makes HTTPS requests to the backend API without certificate pinning. On compromised networks (public WiFi, corporate proxies), an attacker can intercept all API traffic including authentication tokens and financial operations using a proxy tool (mitmproxy, Charles).

**SEC-M02: No Root/Jailbreak Detection**

The app runs on rooted Android or jailbroken iOS devices without restriction. On a compromised device:
- Frida can hook into the app and intercept encryption keys
- The SecureStore can be dumped
- The app binary can be modified
- SSL pinning (if added) can be bypassed

**SEC-M03: No Code Obfuscation**

React Native bundles are JavaScript — readable, debuggable, and modifiable. Without obfuscation (Hermes bytecode + ProGuard), an attacker can:
- Reverse-engineer the API client
- Discover hidden endpoints
- Understand business logic and find bypasses

**SEC-M04: Hardcoded Backend URL**

File: `mobile-app/app.config.ts` (lines 57, 66, 75)

```typescript
API_URL: 'https://qic-trader-backend-f44367dc781f.herokuapp.com'
```

The production backend URL is hardcoded in the build config. This is baked into the app binary and cannot be changed without a new release.

**SEC-M05: Placeholder Apple ID in Build Config**

File: `mobile-app/eas.json` (line 88)

The iOS submit config contains a placeholder Apple ID that needs updating before production release.

#### MEDIUM

**SEC-M06: Redux Persistence in AsyncStorage**

File: `mobile-app/store/index.ts`

Redux state (including auth slice with token) is persisted to `@react-native-async-storage/async-storage`. While the primary token management uses `expo-secure-store` (encrypted), the Redux auth slice also stores token-related state in unencrypted AsyncStorage.

**SEC-M07: `@ts-ignore` Suppressing Type Errors**

File: `mobile-app/config/firebase.ts:20`

TypeScript error suppression in Firebase configuration could mask real issues.

**SEC-M08: 30+ Files with `any` Types**

Across the codebase, ~30+ files contain `any` type usage, reducing type safety for a financial application.

### 5.3 Architectural Issues

**ARCH-M01: No Push Notification Implementation**

Push notification infrastructure exists (types, UI components, settings screen) but `expo-notifications` is not installed. The notification system is incomplete.

**ARCH-M02: No `.env.example` in Mobile Repo**

The mobile app has no `.env.example` documenting required environment variables. Developers must discover them from code.

**ARCH-M03: Mixed State Management**

Same issue as the frontend — Redux + React Query coexist. Some components use Redux, others use React Query hooks.

### 5.4 Strengths

- Token storage uses `expo-secure-store` (hardware-backed encryption)
- HTTP client has request throttling (300ms), rate limit handling, retry logic
- Request deduplication for concurrent identical requests
- Typed routes enabled via Expo Router
- E2E tests configured with Maestro

---

## 6. Telegram Bot — `telegram-bot/`

### 6.1 Architecture Summary

**Stack:** Grammy 1.38.4 + Bun runtime + TypeScript + Redis (ioredis) + Zod 4.2.1

**Structure:**
```
telegram-bot/src/
├── bot/         # Grammy bot initialization
├── commands/    # 30+ command handlers
├── config/      # Zod-validated environment config
├── middleware/   # Security, rate limiting, auth, errors
├── redis/       # Redis client + rate limiter
├── server/      # Optional HTTP server for testing
├── services/    # Business logic (20+ services)
├── templates/   # Message templates
├── types/       # Type definitions
└── utils/       # Logger (Winston), QR code, helpers
```

### 6.2 Security Vulnerabilities

#### MEDIUM

**SEC-T01: Redis TLS Validation Disabled**

File: `telegram-bot/src/redis/client.ts:56-58`

```typescript
tls: { rejectUnauthorized: false }
```

Same issue as the backend (SEC-B22). Heroku Redis uses self-signed certificates, so validation is disabled. MITM attacks on the Redis connection are possible.

**SEC-T02: Rate Limiter Fails Open**

File: `telegram-bot/src/middleware/rateLimitMiddleware.ts`

If Redis is unavailable, the rate limiter allows all requests through instead of blocking. Under a DoS attack that also takes down Redis, the bot becomes unlimited.

**SEC-T03: Production Backend URL in `.env.example`**

File: `telegram-bot/.env.example:25`

```env
BACKEND_URL=https://qic-trader-backend-f44367dc781f.herokuapp.com/api/v1
```

The production backend URL is in the example file. New developers may accidentally connect to production.

#### LOW

**SEC-T04: Polling Instead of Webhooks**

The bot uses long-polling to receive updates. This is less efficient and has slightly higher latency than webhooks. For a financial bot handling trades and escrow, webhook mode with HTTPS is more appropriate for production.

**SEC-T05: All Data in Redis (Ephemeral)**

User sessions, wizard state, and temporary data are stored only in Redis with TTLs. Redis restarts cause all users to be logged out. There's no persistent storage — the bot relies entirely on the backend API for durable data.

### 6.3 Strengths

The telegram bot is the **best-structured** of all 4 repos:

- Zod validation on all environment config (fails to start if invalid)
- Credential masking in logs
- Input validation with injection pattern blocking
- Passwords deleted from Telegram messages immediately after processing
- Session-based auth with Redis (7-day TTL)
- Comprehensive error middleware with custom error types
- Structured logging with Winston (not `console.log`)
- Rate limiting with sliding window algorithm
- Role-based access control for moderator/admin commands
- Password validation (8+ chars, mixed case, number, special char) — stronger than the backend's signup validation

### 6.4 Architectural Issues

**ARCH-T01: No Dockerfile or Deployment Config**

No containerization, no CI/CD configuration. The startup script (`scripts/start.sh`) is macOS-specific (opens new terminal for Redis).

**ARCH-T02: HTTP Test Server is Feature-Complete**

`src/server/index.ts` exposes most bot functionality as a REST API on port 3000. This test server could be accidentally left running in production, providing an unauthenticated interface to bot operations.

---

## 7. Cross-System Vulnerabilities

These vulnerabilities span multiple repositories and represent systemic risks.

### CROSS-01: No Firestore Security Rules (CRITICAL)

**Affected:** Backend, Frontend, Mobile App

The backend uses Firebase Admin SDK (bypasses rules). The frontend and mobile app use Firebase client SDK (subject to rules). With **zero security rules deployed**, the client SDKs have unrestricted access to every Firestore collection.

This is the single most dangerous vulnerability. It means every other backend security measure (auth middleware, role checks, validation) can be bypassed by calling Firestore directly from the frontend.

### CROSS-02: Production Secrets in Version Control (HIGH)

| Repository | File | Secrets Exposed |
|-----------|------|-----------------|
| Frontend | `.env.production` | Firebase API key, QuickNode BTC RPC (with auth token), Tron RPC (with auth token) |
| Frontend | `.env.development` | Dev Firebase API key |
| Mobile App | `app.config.ts` | Production backend URL, EAS project ID |
| Telegram Bot | `.env.example` | Production backend URL |

These secrets are in git history. Even if deleted now, they require rotation.

### CROSS-03: Inconsistent Authentication Between Clients (MEDIUM)

| Client | Token Storage | 2FA Implementation | Password Policy |
|--------|-------------|-------------------|-----------------|
| Frontend | localStorage (XSS-vulnerable) | Full (setup, verify, disable) | Uses backend's weak 6-char policy |
| Mobile App | SecureStore (encrypted) | Verify only (setup via web) | Uses backend's weak 6-char policy |
| Telegram Bot | Redis sessions (encrypted in transit) | Full (setup, verify, disable) | **8+ chars, mixed case, number, special** |
| Backend API | N/A (issues tokens) | Logs valid TOTP codes | **6 chars minimum** |

The Telegram bot enforces a **stronger** password policy than the actual backend API. The mobile app stores tokens more securely than the web app.

### CROSS-04: No API Contract Enforcement (HIGH)

All 3 clients (web, mobile, bot) consume the same backend API, but there is:
- No OpenAPI specification
- No contract tests
- No versioned API schema
- No generated client code

Each client hand-codes its own API types and request/response handling. When the backend changes a response shape, clients break silently.

### CROSS-05: Different Backend URL Patterns (LOW)

| Client | Backend URL | Source |
|--------|-----------|--------|
| Frontend (dev) | `http://localhost:5050/api/v1` | `.env.development` |
| Frontend (prod) | `https://api.qictrader.com/api/v1` | `.env.production` |
| Mobile (all envs) | `https://qic-trader-backend-f44367dc781f.herokuapp.com/api/v1` | `app.config.ts` |
| Telegram (default) | `http://localhost:5050/api/v1` | `.env.example` |

The mobile app points to a Heroku URL while the frontend points to a custom domain. If these are different servers, there could be data inconsistency. If they're the same (Heroku behind custom domain), the mobile app bypasses any CDN/WAF on the custom domain.

---

## 8. Master Vulnerability Register

All vulnerabilities across all 4 repositories, sorted by severity.

### CRITICAL (12)

| ID | Repo | Summary |
|----|------|---------|
| SEC-B01 | Backend | No Firestore security rules — all collections exposed to client SDK |
| SEC-B02 | Backend | Hardcoded wallet encryption key fallback (`'dev-fallback-key-change-me'`) |
| SEC-B03 | Backend | Hardcoded 2FA token secret fallback |
| SEC-B04 | Backend | 2FA temporary token embeds live Firebase idToken and refreshToken |
| SEC-B05 | Backend | Valid TOTP code logged to console on every 2FA attempt |
| SEC-B06 | Backend | Rate limiting middleware never applied despite being configured |
| SEC-B07 | Backend | 2FA TOTP secrets stored as plaintext in Firestore |
| SEC-B08 | Backend | Placeholder JWT utility returns hardcoded strings |
| SEC-B09 | Backend | Placeholder auth middleware that calls `next()` unconditionally |
| SEC-F01 | Frontend | Production Firebase API key + QuickNode RPC URLs committed to git |
| CROSS-01 | All | No Firestore security rules across entire platform |
| SEC-B10 | Backend | Double-spend race condition on wallet transfers |

### HIGH (19)

| ID | Repo | Summary |
|----|------|---------|
| SEC-B11 | Backend | 6 validation schemas defined but never applied to financial routes |
| SEC-B12 | Backend | Signup password policy is 6 chars (stronger schema exists, unused) |
| SEC-B13 | Backend | Firebase API key exposed in URL query parameters |
| SEC-B14 | Backend | User CRUD endpoints missing authentication/authorization |
| SEC-B15 | Backend | Escrow refund missing moderator authorization |
| SEC-B16 | Backend | Mnemonic export endpoint not rate-limited |
| SEC-B17 | Backend | Sensitive data logged via console.log |
| SEC-B18 | Backend | User search loads entire profiles collection — DoS |
| SEC-DB01 | Backend | No schema enforcement — Firestore is schemaless |
| SEC-DB04 | Backend | No referential integrity between financial collections |
| SEC-F02 | Frontend | No Content Security Policy header |
| SEC-F03 | Frontend | Auth tokens stored in localStorage (XSS-accessible) |
| SEC-F04 | Frontend | `dangerouslySetInnerHTML` in chart and layout components |
| SEC-F05 | Frontend | Development Firebase credentials committed |
| SEC-M01 | Mobile | No certificate pinning — MITM possible |
| SEC-M02 | Mobile | No root/jailbreak detection |
| SEC-M03 | Mobile | No code obfuscation — JS bundle readable |
| SEC-M04 | Mobile | Production backend URL hardcoded in build config |
| CROSS-04 | All | No API contract enforcement between clients and backend |

### MEDIUM (14)

| ID | Repo | Summary |
|----|------|---------|
| SEC-B19 | Backend | CORS allows wildcard origin with credentials |
| SEC-B20 | Backend | `/health/detailed` exposes system info without auth |
| SEC-B21 | Backend | Role cache not invalidated on role change |
| SEC-B22 | Backend | Redis TLS certificate validation disabled |
| SEC-B23 | Backend | Account deletion without password/2FA re-verification |
| SEC-B24 | Backend | Trade messages not sanitized for XSS |
| SEC-B25 | Backend | Collection name `'escrow'` vs `'escrows'` mismatch |
| SEC-B26 | Backend | 10MB JSON body limit — resource exhaustion risk |
| SEC-DB02 | Backend | `users` vs `profiles` dual collection data split |
| SEC-DB03 | Backend | Floating-point money (JavaScript `number`) |
| SEC-DB05 | Backend | Ledger entries not truly immutable at DB level |
| SEC-F06 | Frontend | Triple state management (Redux+Zustand+ReactQuery) complexity |
| SEC-M06 | Mobile | Auth state in unencrypted AsyncStorage (via Redux persist) |
| SEC-T01 | Telegram | Redis TLS certificate validation disabled |

### LOW (9)

| ID | Repo | Summary |
|----|------|---------|
| SEC-B27 | Backend | Stack traces in non-production error responses |
| SEC-B28 | Backend | uncaughtException doesn't exit process |
| SEC-B29 | Backend | No request timeout middleware |
| SEC-B30 | Backend | Firebase config falls back to local JSON file |
| SEC-DB06 | Backend | Unused `db/index.ts` uses client SDK |
| SEC-M05 | Mobile | Placeholder Apple ID in EAS submit config |
| SEC-T02 | Telegram | Rate limiter fails open when Redis unavailable |
| SEC-T03 | Telegram | Production backend URL in `.env.example` |
| SEC-T04 | Telegram | Uses polling instead of webhooks |

---

## 9. Architecture Debt Summary

### Per-Repository Health Score

| Dimension | Backend | Frontend | Mobile | Telegram |
|-----------|:-------:|:--------:|:------:|:--------:|
| **Security** | 2/10 | 4/10 | 5/10 | 7/10 |
| **Code Quality** | 4/10 | 6/10 | 6/10 | 8/10 |
| **Type Safety** | 3/10 | 7/10 | 6/10 | 7/10 |
| **Error Handling** | 3/10 | 5/10 | 6/10 | 8/10 |
| **Testing** | 2/10 | 4/10 | 5/10 | 6/10 |
| **Documentation** | 4/10 | 5/10 | 5/10 | 7/10 |
| **Deployment** | 4/10 | 5/10 | 7/10 | 3/10 |
| **Scalability** | 2/10 | 6/10 | 6/10 | 6/10 |
| **Overall** | **3/10** | **5/10** | **6/10** | **7/10** |

### Key Takeaways

**Backend (3/10):** The most critical component has the most issues. 12 critical vulnerabilities, 200+ `any` types, 150+ unstructured log statements, 10+ full collection scans, no rate limiting, and the wallet encryption key has a hardcoded fallback. This is a financial system handling real money.

**Frontend (5/10):** Well-structured code with good XSS sanitization utilities, but production secrets are committed to git, tokens are in localStorage, no CSP header, and no CI/CD pipeline. The triple state management adds unnecessary complexity.

**Mobile App (6/10):** Best token storage (SecureStore), good HTTP client with retry/throttling, but missing fundamental mobile security (certificate pinning, jailbreak detection, obfuscation). Push notifications are half-implemented.

**Telegram Bot (7/10):** The strongest codebase. Zod validation on config, structured logging with Winston, input sanitization, proper rate limiting, session management, and the strongest password policy of all 4 repos. Main gaps are deployment configuration and Redis TLS.

### What Needs to Happen Before Production

**Immediate (block deployment):**
1. Deploy Firestore security rules that deny all client-side writes to sensitive collections
2. Remove hardcoded encryption key fallbacks — fail if env vars missing
3. Stop logging TOTP codes and sensitive data
4. Apply rate limiting middleware globally
5. Rotate all committed secrets (Firebase keys, QuickNode RPC tokens)
6. Apply existing validation schemas to all financial routes
7. Fix the wallet transfer race condition with a proper Firestore transaction

**Soon (within 2 weeks):**
1. Add Content Security Policy to the frontend
2. Move tokens from localStorage to httpOnly cookies (requires backend changes)
3. Implement certificate pinning in the mobile app
4. Add root/jailbreak detection to the mobile app
5. Encrypt 2FA secrets at rest
6. Replace 150+ console.log with structured logger
7. Add pagination to all collection queries

**Planned (within 1 month):**
1. Eliminate 200+ `any` types
2. Consolidate `users`/`profiles` collections
3. Add Firestore security rules tests
4. Set up CI/CD across all repos
5. Add contract tests between clients and backend
6. Write integration tests for financial operations
7. Add code obfuscation to mobile builds
