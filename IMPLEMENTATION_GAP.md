# QicTrader — Implementation Gap Analysis

**Purpose:** Compare FEATURES.md (full platform inventory) and NEW_FEATURES.md (user stories) against the **Rust backend** (`qictrader-backend-rs`) and the **old backend** (`backend/` Node/TypeScript). This list is what has **not** been done yet and is the backlog for this chat.

**Principles:** Pure functional style, Result types, types-first, no mocks/TODOs, implement what’s needed now.

---

## Understanding the Two Documents

### FEATURES.md

- **What it is:** Full catalog of the platform — 33 sections covering every backend API, frontend surface, WebSocket events, client apps (web, mobile, Telegram), and cross-cutting systems.
- **Sections:** Auth, profiles, marketplace/offers, trading, escrow, wallet, custodial multi-chain, gas/treasury, prices, payment methods, KYC, affiliate, reseller, WhatsApp escrow links, notifications, WebSocket, support, bug reporting, newsletter, mod tools, admin, dashboard, settings, blockchain, frontend UI, platform config, system/infra, **reputation**, **append-only ledger**, **offer event tracking**, **soft delete**, **platform fees**, client apps (mobile, Telegram).
- **Backend:** 150+ endpoints; WebSocket events; background jobs (auto-swap, dispute deadline monitor); reputation service; ledger; offer versioning; soft delete; platform fee config.

### NEW_FEATURES.md

- **What it is:** 33 user stories across 13 epics — enhancements and fixes, not the base inventory.
- **Epics:** Registration page (task bar, support access), support window (unauthenticated, categories, AI chat, call IVR, office locations, 5-level priority), username (auto-suggest, restrictions info, validation warnings), email/password (AI validation, strong generator, complexity hint), affiliate code at registration, profile (username/email display, notification simplification, trade name display, region code, bio, profile picture), identity verification (document verification software, manual review team), help (step-by-step guides), wallet (XMR/BNB, Solana icon, timestamps, card design), **affiliate programme** (Novice→Diamond tiers, commission from escrow, progression UI, badges, private stats, leaderboard, dashboard overhaul).

---

## Rust Backend — What Exists Today

The Rust API is under `/api/v1/`. The following are **implemented** (routes and handlers exist):

| Area | Routes / behaviour |
|------|---------------------|
| **Auth** | signup, login, forgot-password, 2fa/verify-login, refresh-token, oauth-sync, verify, logout, logout-all, delete/:userId, reactivate/:userId |
| **Users** | me, me/stats, me/notifications, profile, settings, 2fa (status, setup, verify, disable), password/change, sessions, activity-log, wallet (get, link, unlink), top, top-traders, search, check-username, username/:name/available, :id/public, :id/ratings, :id/report, :id/block, :id/untrust, :id (get, put, delete), POST / (create_user) |
| **Dashboard** | GET /, GET /summary |
| **Offers** | list, create, buy, sell, user/:id, :id (get, put, delete), :id/versions, :id/pause, :id/resume, :id/close |
| **Trades** | create, list, active, completed, :id (get), :id/status, :id/cancel, :id/complete, :id/events, :id/ledger, :id/messages, :id/attachments (single/multiple), :id/rating |
| **Escrow** | list, active, stats, custodial (create, deposit, status, release, refund), trade/:id (get, wallet, balance, confirm-deposit), wallet/:id, offer (create, get, balance, confirm-deposit), :id (get), :tradeId/create, :tradeId/link, :id/release, :id/dispute, :id/refund, :id/resolve-to-seller, :id/resolve-to-buyer, :id/sync (stub) |
| **Wallet** | get, balance, transactions, pending, transfers, deposit-address(es), locks (get, by offer, estimate-fee, check, lock, unlock), available-balance, deposit, withdraw, transfer |
| **Custodial wallet** | generate, get, all, chains, balance, balance/all, deposit-address, history, history/:network, send, export, networks, tokens |
| **Gas** | estimate, estimate/live, estimate/breakdown, estimate/:type, chain-params, price, validate, withdrawal-fee, system-status, withdraw/estimate, withdraw/sponsored, treasury/* |
| **Prices** | get all, get by coin, convert/to-usd, convert/from-usd |
| **Payment methods** | list, create, update, delete |
| **Notifications** | list, :id/read, :id delete, read-all |
| **KYC** | tiers, limits/:level, status, submit, documents (upload, list, get, delete, download) |
| **Support** | ticket (create), tickets (list, get), tickets/:id/message, tickets/:id/close |
| **Reports** | POST / (submit user/trade/offer report) |
| **Affiliate** | stats, referrals, earnings, generate-link, tiers, payouts, request-payout |
| **Reseller** | stats, trades, resell/:offerId, active, buy/:resellOfferId |
| **Newsletter** | subscribe, unsubscribe |
| **Mod** | reports (list, stats, get, action, assign, dismiss, claim), disputes (full set including evidence/upload, evidence/verify, audit, assign, resolve, escalate, claim), users (review, history, warn, suspend, ban, unban, lift-suspension), escrow release/refund, stats (dashboard, moderator, platform-health), audit, logs |
| **Admin** | dashboard, resolve, logs, treasury/balance, treasury/health, treasury/transactions, treasury/user-eligibility, diagnostics, test-encryption, test-solana-rpc |
| **Platform config** | cryptos, cryptos/:symbol, cryptos/:symbol/networks, networks |
| **Direct offers** | (nested under api) |
| **WhatsApp** | links (create, list), links/:id (get, delete), stats, track/:short_code |
| **Health** | GET /health (liveness), GET /health/ready (readiness), GET /api/v1/version |

**Rust also has:** Ledger repo (create_entry, list_by_user, count_by_user, treasury_transactions); offer repo with versioning (OfferVersion); no reputation calculation; no dedicated “offer event” service (events implied by version/status changes).

---

## Old Backend — What Exists

- Same feature areas as above; **additionally**: bug reports (POST/GET /bug-reports, GET /bug-reports/:id) and mod bug reports (GET /mod/bug-reports, stats, status, assign) — **but** `bugReportsRouter` is **not** mounted in `backend/src/routes/index.ts`, so those routes are **dead** unless mounted elsewhere.
- **Reputation:** `calculateReputation` (pure) and `updateUserReputation` (called on trade complete, cancel, rating submit, admin resolve).
- **Health:** GET /health, GET /health/detailed (Redis, WS, memory).
- **Background jobs:** Auto-swap (treasury), dispute deadline monitor.

---

## Gap 1: FEATURES.md — Not (Fully) in Rust Backend

| # | Item | Notes |
|---|------|--------|
| 1 | **Bug reports (user-facing)** | FEATURES §18: POST /bug-reports, GET /bug-reports, GET /bug-reports/:id. Not in Rust. Old backend has code but route not mounted. |
| 2 | **Mod bug reports** | GET /mod/bug-reports, GET /mod/bug-reports/stats, POST /mod/bug-reports/:id/status, POST /mod/bug-reports/:id/assign. Not in Rust. |
| 3 | **Price alerts** | FEATURES Appendix + mobile: GET/POST/DELETE /price-alerts (and toggle). Never in old backend; must be new in Rust (table + job + optional push). |
| 4 | **Reputation system** | FEATURES §28: 0–100 score from trades, rating, success rate, age; penalties; recalc on complete/cancel/rating/dispute. Old backend has it; Rust has **no** reputation module (no calculate, no update on trade/rating). |
| 5 | **GET /health/detailed** | FEATURES §27: detailed health (Redis, WS, memory). Rust has /health and /health/ready only; no “detailed” with Redis/WS/memory. |
| 6 | **Background jobs in Rust** | Auto-swap monitor and dispute deadline monitor run in old backend; need equivalent in Rust (or doc that they run elsewhere). |
| 7 | **Escrow :id/sync** | Rust has route but returns “blockchain sync not yet available”. Implement or remove. |
| 8 | **Dashboard/summary “this month”** | Rust dashboard/summary returns this_month_trades / this_month_volume_usd as 0. Not implemented. |
| 9 | **Admin dashboard volume/trades** | Rust admin dashboard returns total_trades, active_trades, total_volume_usd as 0. Needs real aggregates. |

---

## Gap 2: NEW_FEATURES.md — User Stories Not Implemented

Stories are **not** implemented unless both backend and frontend (and any jobs) exist. Assumption: we build backend first; frontend may lag.

### Epic 1: Registration Page (REG-001–004)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| REG-001 Task bar icon hover | Frontend only | No |
| REG-002 Icon sizing & alignment | Frontend only | No |
| REG-003 Registration page borders/boxes | Frontend only | No |
| REG-004 Pre-registration support access | Support + routing | Support ticket/create usable without auth (Rust has auth on support — need unauthenticated ticket creation or dedicated “contact” endpoint). |

### Epic 2: Support Window (SUP-001–007)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| SUP-001 Support without registration | Same as REG-004 | Unauthenticated support/contact. |
| SUP-002 Categorised support emails | Config + UI | Backend: category → email mapping or config. |
| SUP-003 Complaints/compliments/suggestions | New category flow | Backend: new category/type and storage. |
| SUP-004 AI live chat bot | New service | Backend: optional summary attachment to ticket. |
| SUP-005 Call option with IVR menu | Integration | Backend: store selection, optional webhook. |
| SUP-006 Office locations | Static/config | Backend: config or static content. |
| SUP-007 5-level ticket priority | Auto-classify + override | Backend: priority enum, auto-assignment from category, mod override. |

### Epic 3: Username (USR-001–003)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| USR-001 Auto-generated username suggestions | New endpoint | GET or POST endpoint returning 3+ available suggestions. |
| USR-002 Username restrictions info bubble | Frontend + optional config | Backend: optional config for rules text. |
| USR-003 Validation warnings (symbols, profanity, duplicate) | Validation + profanity list | Backend: validation rules; profanity check (list or service); duplicate check (exists). |

### Epic 4: Email & Password (EML-001–003)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| EML-001 AI-based email validation | Optional external/backend | Format + optional DNS/MX/typo suggestion. |
| EML-002 Strong password generator | New endpoint or frontend | Backend: optional GET /auth/password-suggest. |
| EML-003 Password complexity hint | Validation + config | Backend: same rules as signup (11 chars, upper, lower, symbol, number); return requirements in config or error. |

### Epic 5: Affiliate Code at Registration (AFF-001)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| AFF-001 Affiliate code/link at signup | Signup accepts code | Backend: signup accepts referral_code or link; validate and persist referral relationship. |

### Epic 6: Profile Display (PRF-001)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| PRF-001 Username and email on profile | Profile API | Backend: ensure profile includes username and email (Rust likely already does). |

### Epic 7: Notifications Simplification (NTF-001)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| NTF-001 Dropdown: only “My Profile” + “Notifications” link | Frontend only | No backend change. |

### Epic 8: Profile Enhancements (PRF-002–005)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| PRF-002 Trade name display (full / initial+surname / hidden) | Profile settings + trade payloads | Backend: user setting; trade/chat responses apply masking. |
| PRF-003 Intl region code for phone | Profile + validation | Backend: store full intl number; optional validation. |
| PRF-004 Trader bio | Profile field | Backend: bio field, length limit, profanity filter. |
| PRF-005 Profile picture upload | Avatar storage | Backend: avatar upload (Rust may have or need storage). |

### Epic 9: Identity Verification (VER-001–002)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| VER-001 Document verification software | Integration | Backend: integrate provider (e.g. OCR + liveness); reject invalid; fallback to manual. |
| VER-002 Manual verification review team | Mod/admin panel | Backend: list pending, approve/reject/request re-upload; notify user; audit. |

### Epic 10: Help (HLP-001)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| HLP-001 Step-by-step help articles | CMS or static | Backend: list/categories + content (or static). |

### Epic 11: Wallet (WLT-001–004)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| WLT-001 Monero (XMR) and BNB | New chains | Backend: new networks, addresses, send/receive. |
| WLT-002 Solana wallet icon | Frontend | No backend. |
| WLT-003 Transaction timestamps (date + time) | Response shape | Backend: ensure timestamps in user TZ or ISO with offset. |
| WLT-004 Card design/colour options | Frontend + optional setting | Backend: optional user preference. |

### Epic 12: Affiliate Tiers & Commissions (AFL-001–006)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| AFL-001 Novice tier | Tier model | Backend: Novice tier (e.g. $10 vol, 0–4 affiliates, 5% A1). |
| AFL-002–005 Bronze/Silver/Gold/Diamond | Tiers + commissions | Backend: tier thresholds and A1/A2/A3 rates; auto-promotion. |
| AFL-006 Commission from escrow fee | Commission source | Backend: commission = % of 1% escrow fee; on trade complete; split A1/A2/A3; ZAR/USDT. |

### Epic 13: Affiliate UI & Leaderboard (AFL-007–011)

| Story | Status | Backend needed? |
|-------|--------|------------------|
| AFL-007 Tier progression (progress bars) | Stats | Backend: affiliate stats include current vs next-tier targets. |
| AFL-008 Tier badges on profile | Profile + public | Backend: tier on profile; public profile returns tier/badge. |
| AFL-009 Private tier stats | Auth + profile | Backend: detailed stats only for self; public only tier/badge. |
| AFL-010 Affiliate leaderboard | New endpoint | Backend: GET /affiliate/leaderboard (rank by lifetime earnings). |
| AFL-011 Dashboard visual overhaul | Frontend | No backend. |

---

## Summary: What to Build (Backend-First)

**From FEATURES.md (Rust):**

1. Bug reports: user CRUD + mod list/stats/assign/status.
2. Price alerts: CRUD + background job + optional push.
3. Reputation: pure `calculate_reputation`, call on trade complete/cancel/rating/dispute resolve; persist on profile.
4. GET /health/detailed (Redis, WS, memory) or document that readiness is sufficient.
5. Background jobs: auto-swap, dispute deadline monitor (in Rust or doc).
6. Escrow sync: implement or remove.
7. Dashboard summary “this month” and admin dashboard real volume/trades.

**From NEW_FEATURES.md (backend work):**

- Support: unauthenticated ticket/contact; categories; 5-level priority; optional AI summary.
- Username: suggestion endpoint; profanity + duplicate validation.
- Password: optional suggest endpoint; enforce complexity.
- Signup: affiliate code/link parameter and persistence.
- Profile: trade name display setting; phone region; bio; avatar if missing.
- KYC: document verification integration + manual review workflow.
- Wallet: XMR/BNB support; timestamps in responses.
- Affiliate: Novice→Diamond tiers; commission from escrow (1%); A1/A2/A3 split; leaderboard endpoint; tier in profile and stats for progression.

---

## Do I Understand Each Feature?

Yes.

- **FEATURES.md:** I treat it as the single source of truth for “what the platform is”: every endpoint, event, and system (reputation, ledger, offer events, soft delete, fees). I used it to list what the Rust backend already has and what’s missing.
- **NEW_FEATURES.md:** I treat it as the backlog of 33 user stories across 13 epics. Each story has clear acceptance criteria. I marked what requires backend work and what is frontend-only, and aligned affiliate tiers and commission (AFL-001–011) with FEATURES §12 and §32.

If you want to proceed in this chat, we can pick one gap (e.g. reputation, bug reports, or affiliate tiers + commission) and implement it in Rust following your architectural principles (types first, Result, pure functions, no mocks).
