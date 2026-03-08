# Feature status: FEATURES.md & NEW_FEATURES.md

Status key: **Done** | **Partial** | **Not started**

---

## A. FEATURES.md — Platform inventory (33 sections)

| # | Section | Status | Notes |
|---|---------|--------|-------|
| 1 | Authentication & Account Management | **Done** | Signup, login, 2FA, refresh, logout, delete, reactivate in Rust; frontend wired. Referral code at signup done. |
| 2 | User Profiles & Social | **Partial** | Backend endpoints exist. Profile bio/avatar/trade-name display (NEW_FEATURES) not all done. |
| 3 | Marketplace — Offers | **Done** | List, create, buy/sell, versions, pause/resume/close in Rust; frontend exists. |
| 4 | Trading System | **Done** | Create, list, status, cancel, complete, events, ledger, messages, attachments, rating in Rust. |
| 5 | Escrow System | **Partial** | Custodial, trade/offer escrow, release, refund, dispute in Rust. **Escrow :id/sync** is stub only ("blockchain sync not yet available"). |
| 6 | Wallet System | **Done** | Balance, transactions, deposit/withdraw, locks, transfer in Rust. |
| 7 | Custodial Multi-Chain Wallets | **Done** | Generate, balance, send, export, networks, tokens in Rust. |
| 8 | Gas & Treasury Management | **Done** | Estimate, treasury, sponsored withdrawal in Rust. Auto-swap job not in Rust (see §27). |
| 9 | Price Feeds & Conversion | **Done** | Prices, convert, CoinGecko in Rust. |
| 10 | Payment Methods | **Done** | CRUD in Rust. |
| 11 | KYC & Verification | **Done** | Tiers, status, submit, documents upload/list/download in Rust. No document verification software (VER-001) or manual review queue (VER-002) yet. |
| 12 | Affiliate & Referral Program | **Partial** | Stats, referrals, earnings, generate-link, tiers, payouts in Rust. **Novice→Diamond tiers and commission-from-escrow (NEW_FEATURES)** not implemented. |
| 13 | Reseller Program | **Done** | Stats, trades, resell, buy in Rust. |
| 14 | WhatsApp Escrow Links | **Done** | Create, list, get, delete, stats, track in Rust. |
| 15 | Notifications | **Done** | List, read, delete, read-all in Rust. |
| 16 | Real-Time (WebSocket) | **Partial** | Exists in old backend; Rust may have limited or different WS support. |
| 17 | Support & Help | **Partial** | Create ticket, list, message, close in Rust. **Guest contact (REG-004/SUP-001) done**: POST /support/contact, /contact-us page for guests. |
| 18 | Bug Reporting | **Done** | POST/GET bug-reports (user), mod list/stats/status/assign in Rust. Frontend submits to Rust when authenticated; guests still use Firestore. |
| 19 | Newsletter | **Done** | Subscribe/unsubscribe in Rust. |
| 20 | Moderator Tools | **Done** | Reports, disputes, users (warn/suspend/ban), escrow release/refund, stats, audit, logs in Rust. Mod bug-reports done. |
| 21 | Admin Tools | **Done** | Dashboard, resolve, logs, treasury, diagnostics in Rust. Real volume/trades on admin dashboard done. |
| 22 | Dashboard & Analytics | **Done** | GET dashboard, GET summary with **this_month_trades / this_month_volume_usd** in Rust; frontend "This month" stat and price alerts link done. |
| 23 | Settings & Preferences | **Partial** | Backend profile/settings exist. Trade name display, phone region, bio, avatar (NEW_FEATURES) not all done. |
| 24 | Blockchain Integrations | **Done** | BTC, ETH, SOL, TRX in Rust. XMR/BNB (WLT-001) not added. |
| 25 | Frontend UI/UX | **Partial** | Theme, nav, components exist. **Task bar hover (REG-001)** done on auth header. Dropdown simplification varies. |
| 26 | Platform Configuration | **Done** | Cryptos, networks in Rust. |
| 27 | System & Infrastructure | **Partial** | **GET /health/detailed** (Redis, WS, memory) done in Rust. **Background jobs** (auto-swap, dispute deadline monitor) not in Rust. |
| 28 | Reputation System | **Done** | Calculate + update on trade/rating/cancel/dispute in Rust; persisted on profile. |
| 29 | Append-Only Financial Ledger | **Partial** | Ledger repo (create_entry, list_by_user, etc.) in Rust; trade ledger endpoint. Full balance-from-ledger everywhere may vary. |
| 30 | Offer Event Tracking | **Partial** | Offer versioning in Rust; dedicated offer-event service/events as in doc may be implied not explicit. |
| 31 | Soft Delete & Data Preservation | **Partial** | Soft delete on offers/users etc. in places; not necessarily a single documented service. |
| 32 | Platform Fees | **Partial** | Fee logic in escrow/trades; 1% escrow fee and affiliate commission from it (AFL-006) not implemented. |
| 33 | Client Applications | **Partial** | Web frontend; mobile/Telegram reference price alerts — price alerts now in Rust + web. |

---

## B. NEW_FEATURES.md — User stories (33 stories)

### Epic 1: Registration Page — Task Bar

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| REG-001 | Task bar icon hover effect | **Done** | Auth header task bar: icons + contrasting hover overlay (bg-foreground/10), smooth transition; E2E in system/registration-taskbar.test.ts. |
| REG-002 | Task bar icon sizing & alignment | **Done** | Task bar: min height, h-10/h-11 per item, gap-4, icons h-5 w-5; centered with justify-center; responsive (hidden on small, md:flex). |
| REG-003 | Registration page borders/boxes | **Not started** | Frontend only. |
| REG-004 | Pre-registration support access | **Done** | POST /api/v1/support/contact (no auth), /contact-us form for guests, E2E in system/guest-contact.test.ts. |

### Epic 2: Support Window

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| SUP-001 | Support without registration | **Done** | Same as REG-004; guest_contact_requests table + API + frontend. |
| SUP-002 | Categorised support emails (Fraud, General, Support) | **Not started** | Config + UI. |
| SUP-003 | Complaints, compliments & suggestions option | **Not started** | New category + storage. |
| SUP-004 | AI-powered live chat bot | **Not started** | New service + summary on ticket. |
| SUP-005 | Call option with IVR-style menu | **Not started** | Integration. |
| SUP-006 | Office locations display | **Not started** | Static/config. |
| SUP-007 | 5-level support ticket priority | **Not started** | Auto-classify + mod override. |

### Epic 3: Username Generation & Validation

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| USR-001 | Auto-generated username suggestions | **Done** | Backend endpoint + frontend "Generate random" on signup. |
| USR-002 | Username restrictions info bubble | **Not started** | Frontend + optional config. |
| USR-003 | Username validation warnings (symbols, profanity, duplicate) | **Partial** | Duplicate check exists; profanity/symbols warnings and real-time bubbles not fully done. |

### Epic 4: Email & Password

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| EML-001 | AI-based email validation (format, DNS, typos) | **Not started** | Optional backend/validation. |
| EML-002 | Strong password generator | **Not started** | Backend suggest or frontend-only. |
| EML-003 | Password complexity hint bubble (11 chars, upper, lower, symbol, number) | **Partial** | Rules may be enforced; visible hint bubble until satisfied not fully done. |

### Epic 5: Affiliate Code at Registration

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| AFF-001 | Affiliate code/link at signup | **Done** | Backend accepts referral_code; frontend ref in URL + signup. |

### Epic 6: My Profile — Username & Email Display

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| PRF-001 | Display username and email on profile | **Partial** | Profile API likely returns them; UI may not show both prominently as specified. |

### Epic 7: Notifications Simplification

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| NTF-001 | Dropdown: only "My Profile" + "Notifications" link | **Not started** | Frontend only. |

### Epic 8: Profile Information Enhancements

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| PRF-002 | Trade name display (full / initial+surname / hidden) | **Not started** | Profile setting + apply in trade/chat. |
| PRF-003 | International region code for phone | **Not started** | Profile + validation. |
| PRF-004 | Trader bio (500 chars, profanity filter) | **Partial** | Bio field may exist in backend; limit and profanity filter may be partial. |
| PRF-005 | Profile picture upload | **Partial** | Avatar upload may exist; JPG/PNG 5MB and crop/resize as specified may vary. |

### Epic 9: Identity Verification

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| VER-001 | Identity document verification software (OCR, liveness) | **Not started** | Integration + reject invalid; manual fallback. |
| VER-002 | Manual verification review team / admin panel | **Partial** | KYC flow exists; dedicated pending queue and SLA/audit as specified may vary. |

### Epic 10: Help & Support (Profile Section)

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| HLP-001 | Step-by-step help articles / guides | **Not started** | CMS or static; categorised, searchable. |

### Epic 11: Wallet Overview

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| WLT-001 | Additional blockchain support (XMR, BNB) | **Not started** | Backend + deposit/withdraw flow. |
| WLT-002 | Solana wallet icon | **Not started** | Frontend only. |
| WLT-003 | Transaction timestamps (date + time, local TZ) | **Partial** | Backend may return timestamps; full date+time in local TZ on all tx types may vary. |
| WLT-004 | Card design variety / colour options | **Not started** | Frontend + optional setting. |

### Epic 12: Affiliate Programme — Tier System & Commissions

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| AFL-001 | Novice affiliate tier ($10 vol, 0–4 affiliates, 5% A1) | **Not started** | Backend tier model. |
| AFL-002 | Bronze tier ($50 vol, 5–10 affiliates, 10% A1 + 4% A2) | **Not started** | |
| AFL-003 | Silver tier ($8k vol, 10–50 affiliates, 15% A1 + 8% A2 + 3% A3) | **Not started** | |
| AFL-004 | Gold tier ($50k vol, 50–100 affiliates, 20% A1 + 10% A2 + 5% A3) | **Not started** | |
| AFL-005 | Diamond tier ($300k vol, 100–200 affiliates, 25% A1 + 12% A2 + 7% A3 + gift cards) | **Not started** | |
| AFL-006 | Commission from 1% escrow fee (A1/A2/A3 split on trade complete) | **Not started** | Backend calculation + ZAR/USDT. |

### Epic 13: Affiliate Programme — UI, Cosmetics & Leaderboard

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| AFL-007 | Tier progression visualisation (progress bars) | **Not started** | Backend stats for targets + frontend. |
| AFL-008 | Tier badges & emblems on profile | **Not started** | Backend tier on profile + frontend. |
| AFL-009 | Private tier stats (detailed for self, only badge for others) | **Not started** | Backend + API privacy. |
| AFL-010 | Affiliate leaderboard (rank by lifetime earnings) | **Not started** | Backend GET /affiliate/leaderboard + frontend. |
| AFL-011 | Affiliate dashboard visual overhaul | **Not started** | Frontend only. |

---

## C. Summary counts

| Status | FEATURES.md (sections) | NEW_FEATURES.md (stories) |
|--------|------------------------|----------------------------|
| **Done** | 18 | 3 (USR-001, AFF-001, plus price alerts / dashboard / bug reports / reputation / health detailed from FEATURES) |
| **Partial** | 11 | 6 (USR-003, EML-003, PRF-001, PRF-004, PRF-005, VER-002, WLT-003) |
| **Not started** | 4 (explicit gaps) | 24 |

*FEATURES.md counts are approximate: many sections are "Done" in Rust with small gaps; "Partial" = main area done but specific sub-items or NEW_FEATURES not done. NEW_FEATURES "Done" here is strict (story fully met).*

---

## D. Quick reference — what’s done vs not

**Done (backend and/or frontend):**  
Auth (incl. referral at signup), Users, Offers, Trades, Escrow (except sync), Wallet, Custodial wallet, Gas, Prices + **price alerts**, Payment methods, KYC (basic), Reseller, WhatsApp links, Notifications, **Bug reports (user + mod, frontend to Rust for logged-in)**, Newsletter, Mod tools (incl. mod bug-reports), Admin (incl. real dashboard stats), Dashboard (incl. **this month**), Platform config, **Health detailed**, **Reputation**.

**Partial or gap:**  
Support (no unauthenticated tickets), Escrow sync (stub), Background jobs (not in Rust), Affiliate (no Novice→Diamond tiers or commission from escrow), Profile (trade name, phone region, bio/avatar as per stories), Username (info bubble, profanity/symbols warnings), Password (hint bubble, generator), Wallet (XMR/BNB, Solana icon, timestamps, card themes), Verification (document verification software, manual queue), WebSocket (may be partial in Rust).

**Not started (NEW_FEATURES):**  
REG-001 to REG-004, SUP-001 to SUP-007, USR-002, EML-001, EML-002, NTF-001, PRF-002, PRF-003, VER-001, HLP-001, WLT-001, WLT-002, WLT-004, AFL-001 to AFL-011.
