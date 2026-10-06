# QIC Trader — Design Brief & Functionality Walkthrough

> Single-source design + functionality reference for the QIC Trader app. Generated from the live codebase (`frontend/` + `qictrader-backend-rs/`) — May 2026.
>
> Companion to `DESIGN.md` (visual system / tokens). Where `DESIGN.md` says *how a button looks*, this document says *where the button sits on each screen and what it does when you press it*.

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Information Architecture](#2-information-architecture)
3. [Global Chrome — Header, Footer, Navigation](#3-global-chrome--header-footer-navigation)
4. [Per-Screen Design Briefs](#4-per-screen-design-briefs)
5. [Component Library Reference](#5-component-library-reference)
6. [Public Asset Manifest](#6-public-asset-manifest)
7. [Functionality Walkthrough — Domain Model](#7-functionality-walkthrough--domain-model)
8. [State Machines](#8-state-machines)
9. [End-to-End User Journeys](#9-end-to-end-user-journeys)
10. [API Surface (Reference)](#10-api-surface-reference)
11. [Background Jobs & Side Effects](#11-background-jobs--side-effects)
12. [Limits, Fees, Currencies](#12-limits-fees-currencies)
13. [Auth, MFA, Security](#13-auth-mfa-security)
14. [State Management (Frontend)](#14-state-management-frontend)
15. [Empty / Loading / Error States](#15-empty--loading--error-states)

---

## 1. Product Overview

**QIC Trader** is a custodial peer-to-peer crypto marketplace built for the South African market (ZAR primary, NGN behind a feature flag). It connects fiat sellers and crypto sellers through escrow-protected trades.

| Aspect | Detail |
|---|---|
| Target market | South Africa (ZAR), expanding to Nigeria (NGN) |
| Supported crypto (UI) | BTC, ETH, USDT (TRC-20 primary, SPL via flag), SOL |
| Trade model | P2P with custodial escrow (default) and on-chain escrow (advanced) |
| Revenue | Platform fee (default 0.7% / 70 bps) on the crypto seller; reseller markup cut (25%) |
| Identity model | Tiered KYC L0–L3 with USD daily/monthly volume gates |
| Settlement | Fiat off-platform (bank transfer, payment methods); crypto via custodial wallets |
| Two landing variants | `/` consumer P2P · `/lp` B2B reseller/operator pitch |

**User personas:**

- **Buyer** — sends fiat, receives crypto. Marks payment when bank transfer is sent.
- **Seller (vendor)** — locks crypto in escrow, releases on payment confirmation.
- **Reseller** — relists existing offers with markup; takes a commission on settlement.
- **Moderator** — works dispute, KYC, and report queues.
- **Admin** — operates treasury, configuration, KYC overrides, war room.
- **Investor** — read-only access to war-room KPIs.

---

## 2. Information Architecture

The app uses Next.js App Router with route groups that **do not appear in URLs**:

| Route group | Layout | Auth gate |
|---|---|---|
| `(auth)` | Slim header (`authHeader`), no footer | Public |
| `(main)` | `MainHeader` + `Footer` | Public unless explicitly marked authed |
| `(offers)` | `MainHeader` + `Footer` | Public browse, authed actions |
| `(dashboard)` | `MainHeader` + `RequireAuth` + `Footer` + `TestAccountBanner` | Authed |

**Auth model is layered, not single-gate:**

1. **`AuthProvider`** (`src/components/providers/auth-provider.tsx`) — global redirector. Unauthenticated users on protected routes go to `/login`; logged-in users on `/login` or `/signup` go to `/dashboard`.
2. **`RequireAuth`** wrapper inside `(dashboard)` layout.
3. **`AdminGuard`** / **`ModeratorGuard`** — role checks on `/admin/*` and `/moderator/*`.
4. **KYC gate** — *not* a route gate. Enforced at action-time (`useKYCGate` + `KYCRequirementModal`) and on the backend via the `KycVerifiedUser` extractor when `KYC_MANDATORY_GATE_ENABLED=true`.

**Total page count: 86 `page.tsx` routes.**

### Route Map (grouped)

```
PUBLIC MARKETING               AUTH                          MAIN APP (authed)
/                              /login                         /dashboard
/lp                            /signup                        /wallet
/marketplace                   /forgot-password               /wallet/deposit/[currency]
/about                         /reset-password                /wallet/withdraw/[currency]
/affiliate                     /verify-email                  /wallet/manage/[currency]
/contact-us                    /verify-email-pending          /settings
/faqs                          /auth/callback/google          /trade-history
/help                                                         /trade-history/[id]
/how-escrow-works              OFFER & TRADE                  /notifications
/trading-guide                 /offer/create                  /escrow
/security-tips                 /offer/resell                  /whatsapp
/testimonials                  /offer/[id]                    /my-tickets
/terms /privacy /cookies       /offer/[id]/fund-escrow        /my-tickets/[id]
/status                        /trade/[id]                    /my-offers
/prices /prices/live           /profile                       /reseller
/prices/gas                    /profile/[userId]              /report
/withdrawal-fees                                              /bug-report
                               MODERATOR (gated)              ADMIN (gated)
                               /moderator                     /admin
                               /moderator/disputes            /admin/war-room (also: investor)
                               /moderator/kyc-reviews         /admin/analytics
                               /moderator/users               /admin/users
                               /moderator/reports             /admin/verifications
                               /moderator/support-tickets     /admin/config
                               /moderator/trades/[id]         /admin/treasury (+ /transfer, /recovery)
                               /moderator/treasury-recovery   /admin/reconciliation
                               /moderator/logs                /admin/withdrawal-fees
                                                              /admin/banned-words
                                                              /admin/logs /admin/diagnostics
                                                              /admin/testing
                                                              /admin/incidents/reseller-settlement
```

---

## 3. Global Chrome — Header, Footer, Navigation

### 3.1 Main Header (`src/components/common/mainHeader.tsx`)

Sticky top, full-width, `bg-background/80 backdrop-blur` on scroll. Used by `(main)`, `(dashboard)`, `(offers)` layouts and the `/` landing page.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  [Logo]   [NavLinks (desktop only)]    [Bell] [ZAR Bal] [Avatar▾]   ☰   │
└─────────────────────────────────────────────────────────────────────────┘
│  [BalanceStrip — horizontal scroll of per-asset balances, authed only]  │
└─────────────────────────────────────────────────────────────────────────┘
```

| Zone | Component | Asset / behavior |
|---|---|---|
| Logo (left) | `src/components/navigation/Logo.tsx` → `src/components/brand/QicLogo.tsx` | **Inline SVG** (theme-aware), not `public/logo.svg`. Mobile: icon only. Desktop: lockup. Click → `/` |
| Desktop nav (center) | `src/components/navigation/NavLinks.tsx` | Items from `MAIN_NAV_ROUTES` in `src/config/routes.ts` |
| Notifications (right) | `src/components/navigation/NotificationsDropdown.tsx` | Bell icon with red dot for unread; opens dropdown list |
| ZAR balance (right) | `src/components/navigation/HeaderBalance.tsx` | Shows ZAR equivalent of total portfolio |
| Profile avatar (right) | `src/components/navigation/UserProfileDropdown.tsx` | Avatar (initials fallback) → dropdown of `PROFILE_MENU_ITEMS` |
| Mobile hamburger (right) | `src/components/navigation/MobileMenuButton.tsx` | Opens right slide-over `MobileNav` |
| Padding | — | `px-3 py-2.5` mobile, `px-24 py-3` desktop (per `DESIGN.md` §4.2) |

**Desktop nav items** (`MAIN_NAV_ROUTES`):

| Label | Path | Visibility |
|---|---|---|
| Marketplace | `/marketplace` | Always |
| My Offers | `/my-offers` | Authed |
| Create Offer | `/offer/create` | Authed |
| Wallet | `/wallet` | Authed |
| Affiliate | `/affiliate` | Always |
| About | `/about` | Always |
| Support | `/contact-us` | Authed |

### 3.2 Mobile Drawer (`src/components/common/mobile-nav.tsx`)

Right slide-over (Sheet primitive). **No bottom tab bar** — mobile uses the hamburger drawer.

Drawer contents (top → bottom):

1. User card (avatar + display name + verification badge)
2. Main nav links (`MAIN_NAV_ROUTES`)
3. Profile menu (`PROFILE_MENU_ITEMS`):
   - Profile, Verification (`/settings?tab=verification`), Notifications, My Trades, Settings, Security & 2FA, Payment Methods, Wallet, Reseller Profile, My Tickets, Help & Support
4. Privileged links (mobile-only convenience):
   - Moderator Dashboard → `/moderator` (mod email allowlist)
   - Admin Panel → `/admin` (admin role)
5. Theme toggle + logout

### 3.3 Auth Header (`src/components/common/authHeader.tsx`)

Used by `(auth)` route group only.

```
┌─────────────────────────────────────────────────────────────────┐
│  [Logo]                              [Sign Up]  [Support]       │
└─────────────────────────────────────────────────────────────────┘
```

No footer. No notifications. Page content typically `bg-hero-bg` with the form column + an illustration column on signup/forgot/reset/verify pages.

### 3.4 Footer (`src/components/landing/Footer.tsx`)

5-column marketing footer used by all main route groups.

```
┌────────────────────────────────────────────────────────────────────────┐
│  [Brand Lockup]   Product       Company       Legal       Connect       │
│  Tagline + ZA     Marketplace   About         Terms       Twitter       │
│  badge            Trading guide Affiliate     Privacy     Telegram      │
│                   FAQs          Contact       Cookies     YouTube       │
│                   Help          Testimonials               Instagram    │
│                                                                          │
│  © 2026 QIC Trade Systems Limited                                       │
└────────────────────────────────────────────────────────────────────────┘
```

- Brand lockup uses **inline `QicIcon` / `QicLockup` SVG** from `src/components/brand/QicLogo.tsx`.
- Social icons sourced from `public/assets/icons/` (`facebook.svg`, `twitter.svg`, `ig.svg`, `tg.svg`, `yt.svg`, `whatsapp.svg`).

### 3.5 Global Overlays (root layout, `src/app/layout.tsx`)

Mounted regardless of route:

- `SupportChatWidget` (`src/components/support/`) — floating bottom-right bubble on all pages.
- `ConsentBanner` — GDPR-style cookie consent at first visit; state in `consent-store` (Zustand).
- `ReacceptanceModal` — fires when a new legal document version is detected for the user.
- Toast container (Sonner) — bottom-right of viewport.

---

## 4. Per-Screen Design Briefs

For each major screen: layout zones, components, asset positions, primary states, and the backend interaction it triggers.

### 4.1 `/` — Home Landing (consumer)

**File:** `src/app/page.tsx`
**Layout:** custom — `MainHeader` → `<main>` (full-width) → `Footer`. Does **not** use the `(main)` route group layout.
**Auth:** public. Logged-in users see authed nav state; CTAs route to `/marketplace`.

```
┌──────────────────────────────────────────────────────────────────────┐
│                          Main Header                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ╔════════════════════ HERO ═══════════════════════════════════╗    │
│   ║  Headline (left)              │  Hero image (right, lg+)     ║    │
│   ║  Subhead                      │  public/landing/hero.png     ║    │
│   ║  [Start Trading]  [How It…]   │                              ║    │
│   ╚═══════════════════════════════════════════════════════════════╝    │
│                                                                      │
│   ── How It Works  (3 numbered steps with icons)                    │
│                                                                      │
│   ── Why QicTrader (feature grid: escrow, KYC, liquidity)           │
│                                                                      │
│   ── Feature spotlight                                              │
│                                                                      │
│   ── YouTube feed (latest 3 videos)                                 │
│                                                                      │
│   ── StartCta (full-width gradient band, primary CTA)               │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│                          Footer                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Zone | Component | Asset |
|---|---|---|
| Hero | `src/components/landing/Hero.tsx` | `public/landing/hero.png` (right column lg+) |
| Steps | `src/components/landing/HowItWorks.tsx` | Inline numbered SVG/icons |
| Features | `src/components/landing/WhyQictrader.tsx` | `public/landing/*.svg` (legacy art) |
| Feature row | `src/components/landing/Feature.tsx` | — |
| Video feed | `src/components/landing/YouTubeFeed.tsx` | YouTube embed iframes |
| CTA band | `src/components/landing/StartCta.tsx` | Brand-blue gradient, single CTA |

**Animations** (Framer Motion): hero H1 fade + translateY(30→0) over 0.7s; subtitle and CTAs staggered 0.5s and 0.7s delay; hero image floats over 5s loop.

**No backend interaction** other than fetching public crypto prices via `/api/v1/prices` for the optional price strip.

---

### 4.2 `/lp` — V8 Reseller Landing (B2B)

**File:** `src/app/lp/page.tsx`
**Auth:** public.
**Purpose:** book a Calendly call with the QIC team — pitches the reseller/operator tooling.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Main Header                                                         │
├──────────────────────────────────────────────────────────────────────┤
│  V8Hero — bold headline + Calendly CTA                              │
│  Spread mock ── shows offer pricing on multiple resellers            │
│  Dashboard mock ── reseller earnings dashboard preview               │
│  Video block (embedded explainer)                                    │
│  "What happens next" — 3-step onboarding                             │
│  Calendly inline embed                                               │
│  Closing CTA (mirror of hero)                                        │
│  YouTubeFeed                                                         │
├──────────────────────────────────────────────────────────────────────┤
│  Footer                                                              │
└──────────────────────────────────────────────────────────────────────┘
```

Components live in `src/components/landing/v8/*`. CSS scoped by `v8-landing` class.

---

### 4.3 `/marketplace` — Offer Book

**File:** `src/app/(main)/marketplace/page.tsx`
**Auth:** public (browse). Authed for "Find a Trader" and direct-trade actions.
**Backend:** `GET /api/v1/offers` with filters; `GET /api/v1/prices/fx` for the reference bar.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Main Header                                                         │
├──────────────────────────────────────────────────────────────────────┤
│  bg-background-secondary, lg:px-24                                   │
│                                                                      │
│   ── Title row: "Marketplace"          [+ Find a Trader] (authed)   │
│                                                                      │
│   ┌─────────────── FiatReferenceBar ─────────────────────────┐      │
│   │  USD → ZAR  •  USDT → ZAR  •  Last updated 12:34          │      │
│   └────────────────────────────────────────────────────────────┘      │
│                                                                      │
│   ┌─ [Buy Crypto] [Sell Crypto] ─── (segmented tab) ────────┐       │
│   │                                                           │       │
│   │  FilterControls    SortControls        [Grid/List]        │       │
│   │  Crypto · Fiat · Payment method · Min/max amount          │       │
│   ├───────────────────────────────────────────────────────────┤       │
│   │  Desktop column headers (table mode):                     │       │
│   │  Trader · Price · Limits · Payment · Trade →              │       │
│   ├───────────────────────────────────────────────────────────┤       │
│   │  ┌──────────────────────────────────────────────────┐     │       │
│   │  │ DesktopOfferCard                                  │     │       │
│   │  │ Avatar · Username · Stars · Premium %             │     │       │
│   │  │ Price · Limits · Payment icons · [Buy/Sell] CTA   │     │       │
│   │  └──────────────────────────────────────────────────┘     │       │
│   │  (repeat)                                                 │       │
│   ├───────────────────────────────────────────────────────────┤       │
│   │  Pagination                                                │       │
│   └───────────────────────────────────────────────────────────┘       │
│                                                                      │
│   AffiliateBanner (occasional row, gradient strip)                   │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│  Footer                                                              │
└──────────────────────────────────────────────────────────────────────┘
```

| Zone | Component | Notes |
|---|---|---|
| Tabs | `MarketplaceOffers` (parent) | Buy / Sell. Tab is a URL state (`?type=buy`) so it deep-links. |
| Filters | `src/components/features/marketplace/FilterControls.tsx` | Crypto, fiat, payment method, min/max amount. |
| Sort | `SortControls` | Best price, most reputable, fastest, lowest min. |
| List | `OffersList` → `offer-card/DesktopOfferCard.tsx` + skeleton | Mobile collapses to single-column compact card. |
| Reference bar | `FiatReferenceBar` | Last-known FX (cached server-side, refreshed by `run_fx_poller`). |
| Modals | `UserSearchModal`, `CreateDirectOfferModal` | Reachable via "Find a Trader" CTA when authed. |

**Crypto icons** resolved by `prices-api.ts` to `/assets/icons/{usdt,bitcoin,eth,sol,trx}.svg`. Payment-method icons use `/assets/icons/wallet4.svg`.

**Hooks:** `use-marketplace-page` orchestrates filters, pagination, debounce, and data fetching. URL state is persisted (see `__tests__/hooks/use-marketplace-page-url-state.test.ts`).

---

### 4.4 `/offer/[id]` — Offer Detail

**File:** `src/app/(offers)/offer/[id]/page.tsx`
**Auth:** public (view). Starting a trade requires authed + KYC at the action level.
**Backend:** `GET /api/v1/offers/:id`, `POST /api/v1/offers/:id/quote` (deterministic quote on amount change), `POST /api/v1/trades` to start.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Main Header                                                         │
├──────────────────────────────────────────────────────────────────────┤
│  bg-background-secondary, lg:px-24                                   │
│  Breadcrumb: Marketplace > Offer #abc123                             │
│                                                                      │
│   ┌─────────────────────────────────┬──────────────────────────────┐ │
│   │  LEFT (~65%)                    │  RIGHT (~35%, sticky)         │ │
│   │                                 │                                │ │
│   │  Trader card                    │  Price breakdown               │ │
│   │   - Avatar / initials           │   - Price/unit                 │ │
│   │   - Username + verified badge   │   - Min / Max                  │ │
│   │   - Online dot                  │   - Premium %                  │ │
│   │   - Star rating + trade count   │                                │ │
│   │                                 │  ZAR amount input  ⇄  USDT     │ │
│   │  Payment methods badges         │  amount input (linked, live)   │ │
│   │                                 │                                │ │
│   │  Terms (markdown render)        │  Escrow shield messaging       │ │
│   │                                 │   /assets/icons/shield.svg     │ │
│   │  Version history accordion      │                                │ │
│   │   (`offer_versions` snapshots)  │  [ Start Trade ] (primary)     │ │
│   │                                 │                                │ │
│   │                                 │  Report offer (link, low key)  │ │
│   └─────────────────────────────────┴──────────────────────────────┘ │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│  Footer                                                              │
└──────────────────────────────────────────────────────────────────────┘
```

| Zone | Component | Asset |
|---|---|---|
| Trader card | inline in page | `/assets/icons/star.svg`, `bluecheck.svg` |
| Trade form | inline | `/assets/icons/wallet4.svg` (input adornment) |
| Escrow messaging | inline | `/assets/icons/shield.svg`, `greencheck.svg` |
| Back link | inline | `/assets/icons/arrowleft.svg` |
| KYC modal | `KYCRequirementModal` | Fires when CTA clicked and tier insufficient |
| Report | `ReportButton` → `/api/v1/users/:id/report` | — |

**Quote behavior:** every change to ZAR or crypto amount POSTs `/api/v1/offers/:id/quote` (debounced 250ms) and updates the breakdown atomically. The opposite field is computed from the response, never from a client-side multiply (preserves ledger parity).

**Mobile layout:** stacks. Right column drops below trader card. CTA becomes sticky-bottom.

---

### 4.5 `/offer/[id]/fund-escrow` — Escrow Funding Step

**File:** `src/app/(offers)/offer/[id]/fund-escrow/page.tsx`
**Auth:** authed.
**Purpose:** seller funding an offer-level escrow (vendor stakes upfront so the listing is hot).
**Backend:** `POST /api/v1/escrow/offer/create`, `POST /api/v1/escrow/offer/:offer_id/confirm-deposit`.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Title: Fund escrow for Offer #abc123                                │
│  Step indicator: Address ─ Send ─ Confirm                            │
│                                                                      │
│  Network selector (BTC / ETH / SOL / TRX) ── icons                  │
│  QR code (deposit address) /assets/icons/qrcode.svg                 │
│  Address (mono, click-to-copy)                                       │
│  Required amount (with safety reserve for BTC: gas reserve)          │
│                                                                      │
│  Live balance poll (5s interval) — shows confirmations               │
│  EscrowDepositCard / EscrowFundedStatus                              │
│                                                                      │
│  [ I have sent the funds ] (primary)                                 │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 4.6 `/trade/[id]` — Trade Page (the core room)

**File:** `src/app/(main)/trade/[id]/page.tsx`
**Auth:** authed + participant (or moderator).
**Backend:** `GET /api/v1/trades/:id`, `GET /messages`, `POST /messages`, `POST /mark-paid`, `POST /complete`, `POST /dispute`, Socket.IO `trade:*` events.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Main Header                                                         │
├──────────────────────────────────────────────────────────────────────┤
│  bg-background-secondary, px-4 lg:px-24                              │
│  Breadcrumb: Trades > Trade #xyz789                                  │
│  Title row: Trade #xyz789  ·  Started 12:34                          │
│                                                                      │
│  TradeStatusBanner (full-width, color-coded)                         │
│   - Pending escrow / Paid / Released / Disputed / Completed          │
│   - Countdown timer when applicable                                  │
│                                                                      │
│   ┌─────────────────────────────────┬──────────────────────────────┐ │
│   │  LEFT (flex-1)                  │  RIGHT (~320px fixed)         │ │
│   │                                 │                                │ │
│   │  TradeHeader                    │  TradeSidebar                  │ │
│   │   - Counterparty card           │   - Action buttons:            │ │
│   │   - Amounts (fiat / crypto)     │     [Mark Paid] / [Release]   │ │
│   │   - Payment method              │     [Open Dispute] / [Cancel]  │ │
│   │                                 │   - EscrowStatusBar            │ │
│   │  TradeChat                      │     /assets/icons/shield.svg   │ │
│   │   - Messages (own/peer)         │   - Timer to expiry            │ │
│   │   - Attachments inline          │   - ParticipantsCard           │ │
│   │   - Input + paperclip + send    │   - CounterpartyTradeHistory   │ │
│   │                                 │                                │ │
│   │  ProofOfPaymentDisplay          │                                │ │
│   │   - Image / PDF preview         │                                │ │
│   │                                 │                                │ │
│   │  TradeEventTimeline             │                                │ │
│   │   - Audit log of state changes  │                                │ │
│   │                                 │                                │ │
│   │  DisputeEvidenceUpload          │                                │ │
│   │   (visible only when disputed)  │                                │ │
│   │                                 │                                │ │
│   │  TradeReviewForm                │                                │ │
│   │   (visible after Completed)     │                                │ │
│   │                                 │                                │ │
│   │  DownloadReceiptButton          │                                │ │
│   │   (visible after Completed)     │                                │ │
│   │                                 │                                │ │
│   └─────────────────────────────────┴──────────────────────────────┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Component | File | Behavior |
|---|---|---|
| `TradeHeader` | `src/components/features/trade/TradeHeader.tsx` | Counterparty + amounts + payment method |
| `TradeStatusBanner` | `.../TradeStatusBanner.tsx` | Color-coded by status, includes countdown |
| `TradeChat` | `.../TradeChat.tsx` | Real-time via Socket.IO `trade:message:new` |
| `ChatAttachment` | `.../ChatAttachment.tsx` | Image / PDF preview, click-to-zoom |
| `TradeSidebar` | `.../TradeSidebar.tsx` | Conditional CTAs based on user role + status |
| `EscrowDepositCard` / `EscrowStatusBar` | `.../EscrowDepositCard.tsx` | Funding UX, on-chain confirmations |
| `ProofOfPaymentDisplay` | `.../ProofOfPaymentDisplay.tsx` | Buyer's bank-transfer screenshot |
| `TradeEventTimeline` | `.../TradeEventTimeline.tsx` | Reads `GET /trades/:id/events` |
| `DisputeEvidenceUpload` | `.../DisputeEvidenceUpload.tsx` | Multi-file upload to `/dispute/evidence` |
| `TradeReviewForm` | `.../TradeReviewForm.tsx` | Star + comment, posts to `/rating` |
| `DownloadReceiptButton` | `.../DownloadReceiptButton.tsx` | `GET /trades/:id/receipt.pdf` |

**Action visibility matrix:**

| Status | Buyer sees | Seller / vendor sees | Moderator+ sees |
|---|---|---|---|
| `created` | (waiting) | Cancel | Cancel, Force-cancel |
| `escrow_funded` | Mark Paid, Cancel | Cancel | Release, Refund |
| `paid` | Open Dispute, Cancel(*) | Release, Open Dispute | Release, Refund |
| `disputed` | Upload evidence | Upload evidence | Resolve to Buyer / Seller |
| `released` | (auto-completes) | (auto-completes) | — |
| `completed` | Rate, Download receipt | Rate, Download receipt | — |

(*) Cancel after `paid` is windowed (TF-005 rules).

---

### 4.7 `/wallet` — Wallet Overview

**File:** `src/app/(dashboard)/wallet/page.tsx`
**Auth:** authed.
**Backend:** `GET /api/v1/wallet`, `GET /api/v1/wallet/transactions`, `GET /api/v1/prices`, plus per-currency endpoints when modal opens.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Main Header + BalanceStrip                                          │
├──────────────────────────────────────────────────────────────────────┤
│  bg-hero-bg, max-w-7xl mx-auto, px-4                                 │
│  Breadcrumb: Home > Wallet                                           │
│                                                                      │
│   Title: "My Wallet"  [info ⓘ]                          [☀/🌙 theme]│
│                                                                      │
│   Show zero balances [toggle]    CardThemeSelector [● ● ●]          │
│                                                                      │
│   ┌──────────── WalletCard grid (1 / 2 / 3 cols) ──────────────┐    │
│   │  ┌─USDT─────┐ ┌─BTC──────┐ ┌─ETH──────┐ ┌─SOL──────┐      │    │
│   │  │ Icon      │ │ Icon     │ │ Icon     │ │ Icon     │      │    │
│   │  │ Balance   │ │ Balance  │ │ Balance  │ │ Balance  │      │    │
│   │  │ ZAR equiv │ │ ZAR eq   │ │ ZAR eq   │ │ ZAR eq   │      │    │
│   │  │ [Deposit] │ │ [Deposit]│ │ [Deposit]│ │ [Deposit]│      │    │
│   │  │ [Withdraw]│ │ [W/draw] │ │ [W/draw] │ │ [W/draw] │      │    │
│   │  └───────────┘ └──────────┘ └──────────┘ └──────────┘      │    │
│   └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│   WalletActions row: [Deposit] [Withdraw] [Transfer]                 │
│                                                                      │
│   PortfolioDistribution donut + PriceCards (live ticker)             │
│                                                                      │
│   TransactionHistory (table) ─ filter by type/asset/status           │
│   - Deposit, Withdrawal, Trade lock/release, Sweep, Fee              │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Zone | Component | Modal triggered |
|---|---|---|
| Wallet cards | `WalletCard` / `CustodialWalletCard` | `DepositModal`, `WithdrawModal` |
| Bottom actions | `WalletActions` | `TransferModal` (P2P internal transfer) |
| Distribution | `PortfolioDistribution` | — |
| Price tiles | `PriceCards` | — |
| TX list | `TransactionHistory` | `TransactionStatusModal` (per-row) |
| Lock funds | `LockFundsModal` | Pre-locks for offer creation |

**Modals are mobile bottom-sheets, desktop centered dialogs** (per `DESIGN.md` §7.4).

---

### 4.8 `/settings` — Account Settings

**File:** `src/app/(dashboard)/settings/page.tsx`
**Auth:** authed.
**Layout:** narrow `max-w-4xl` centered. Tab bar persists in URL via `?tab=`.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Title: "Settings"                                                   │
│  ┌─[ General ]─[ Security ]─[ Payments ]─[ Verification ]─┐          │
│  │ Active tab content                                       │          │
│  │                                                           │          │
│  │ GENERAL:                                                  │          │
│  │  ProfileSection (avatar upload, display name, username)   │          │
│  │  TradingPreferences (default fiat, rounding)              │          │
│  │  NotificationSettings (email/in-app per event type)       │          │
│  │  BlockedUsersSection                                      │          │
│  │  TradingLimitsSection (read-only display of tier limits)  │          │
│  │                                                           │          │
│  │ SECURITY:                                                 │          │
│  │  Password change                                          │          │
│  │  TwoFactorSetup (TOTP wizard, QR + backup codes)          │          │
│  │  SessionsList (revoke individual / all)                   │          │
│  │  ConnectedAccounts (Google OAuth link/unlink)             │          │
│  │                                                           │          │
│  │ PAYMENTS:                                                 │          │
│  │  PaymentMethodsSection — saved bank details, encrypted    │          │
│  │                                                           │          │
│  │ VERIFICATION:                                             │          │
│  │  KYCVerificationContent → IdentityVerification            │          │
│  │   (Didit hosted session iframe)                           │          │
│  │  KYCL3UpgradeSection (proof of address upload)            │          │
│  │  KYCStatusBadge + tier benefits table                     │          │
│  └───────────────────────────────────────────────────────────┘          │
│                                                                      │
│   Footer actions: [Logout]  [Deactivate]  [Delete account]           │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 4.9 `/login` & `/signup`

**Files:** `src/app/(auth)/login/page.tsx`, `src/app/(auth)/signup/page.tsx`
**Layout:** `(auth)` group — `authHeader`, no footer, `bg-hero-bg`.

**Login** — single centered column `max-w-md` card:

```
┌──────────────────────────────────────────────────────────────────────┐
│  Auth Header (Logo · [Sign Up] [Support])                            │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│            ┌─────────────────────────────────────────┐              │
│            │  Welcome back                           │              │
│            │  Email [____________________]           │              │
│            │  Password (PasswordInput, eye toggle)   │              │
│            │  [ Sign in ] (LoadingButton, brand-blue)│              │
│            │  ─────── or ────────                     │              │
│            │  [ Continue with Google ]               │              │
│            │   /assets/icons/google.svg              │              │
│            │  Forgot password? · Need an account?    │              │
│            └─────────────────────────────────────────┘              │
│                                                                      │
│  TwoFactorLoginModal (overlays when 2FA challenge returned)          │
└──────────────────────────────────────────────────────────────────────┘
```

**Signup** — split layout:

```
┌──────────────────────────────────────────────────────────────────────┐
│  Auth Header                                                         │
├──────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────┬─────────────────────────────────────┐ │
│  │  FORM (left, lg:50%)     │  ILLUSTRATION (right, lg:50%)        │ │
│  │                          │                                       │ │
│  │  Create your account     │  /assets/signupbanner.svg            │ │
│  │  Email                   │                                       │ │
│  │  Username (live check)   │  Tagline:                            │ │
│  │   /username/:u/available │  "Trade ZAR ⇄ Crypto, securely"      │ │
│  │  Password                │                                       │ │
│  │  Referral code? (opt)    │                                       │ │
│  │  [✓] I accept terms      │                                       │ │
│  │  [Create account]        │                                       │ │
│  │  Already have one? Login │                                       │ │
│  └──────────────────────────┴─────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

**Verify-email / forgot / reset** all use `/assets/loginbanner.svg` in the right column.

---

### 4.10 `/dashboard` — User Home

**File:** `src/app/(dashboard)/dashboard/page.tsx`

```
┌──────────────────────────────────────────────────────────────────────┐
│  Greeting: "Hi, {firstName}"                                          │
│                                                                      │
│  StatsGrid (4 cards): Active trades · Completed · Volume · Earnings  │
│                                                                      │
│  QuickActions: Buy · Sell · Deposit · Withdraw                       │
│                                                                      │
│  ┌────────────────────────────┬────────────────────────────────┐    │
│  │  WalletOverview (donut)    │  EscrowOverview (status pie)    │    │
│  └────────────────────────────┴────────────────────────────────┘    │
│                                                                      │
│  ActiveTradesTable                                                   │
│                                                                      │
│  MyOffersTable (top 5, link to /my-offers)                          │
│                                                                      │
│  ┌────────────────────────────┬────────────────────────────────┐    │
│  │  ProfitTrend (line chart)  │  MonthlyTradingVolume (bars)    │    │
│  └────────────────────────────┴────────────────────────────────┘    │
│                                                                      │
│  AffiliateSummary (compact card, link to full affiliate page)        │
│  RecentActivity feed                                                 │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 4.11 `/admin/war-room` — Operations Dashboard

**File:** `src/app/(dashboard)/admin/war-room/page.tsx`
**Auth:** Admin or Investor role.
**Backend:** `GET /api/v1/admin/war-room/*` endpoints (summary, trades, activity, heatmap, leaderboard, treasury, kyc-funnel, affiliates, cohorts).

Layout follows the brief in `architecture/WAR-ROOM-DASHBOARD-SPEC.md`. KPI tiles top, time-series charts middle, ops drilldowns below. Read-only for investors; admins see action affordances (treasury transfer link, escrow recovery shortcuts).

---

### 4.12 Moderator Pages (`/moderator/*`)

**Files:** `src/app/(dashboard)/moderator/**/page.tsx`
**Guard:** `ModeratorGuard` (email allowlist).

Common pattern across pages:

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sidebar nav (queue counts):                                         │
│   Disputes (12)  ·  KYC reviews (3)  ·  Reports (5)  ·  Tickets (7) │
├──────────────────────────────────────────────────────────────────────┤
│  Filter bar (priority / age / assigned)                              │
│  Queue table (latest first)                                          │
│  Detail drawer / page on row click                                   │
└──────────────────────────────────────────────────────────────────────┘
```

Detail pages (e.g. `/moderator/disputes/[id]`) follow a left-content / right-actions split: trade context + chat + evidence on the left, resolution actions on the right.

---

## 5. Component Library Reference

Components organized by feature. File paths relative to `frontend/src/`.

### 5.1 Marketplace (`components/features/marketplace/`)

| Component | Purpose |
|---|---|
| `MarketplaceOffers` | Page shell — title, tabs, filters, list, pagination |
| `FilterControls` | Crypto / fiat / payment method / amount range filters |
| `SortControls` | Sort + view-mode (grid/list) |
| `OffersList` | Renders cards or table rows |
| `offer-card/DesktopOfferCard` | Full row card for desktop |
| `offer-card/OfferCardSkeleton` | Loading placeholder |
| `FiatReferenceBar` | USD/USDT → ZAR ticker |
| `AffiliateBanner` | Inline gradient strip pitching the affiliate program |
| `MarketplaceHero` / `CryptoCard` | Optional hero / featured-crypto tiles |

### 5.2 Trade (`components/features/trade/`)

| Component | Purpose |
|---|---|
| `TradeHeader` | Counterparty + amounts |
| `TradeStatusBanner` | Color-coded status banner with timer |
| `TradeChat` | Real-time chat (Socket.IO) |
| `ChatAttachment` | File preview / download |
| `TradeSidebar` | Right column, conditional CTAs |
| `TradeActions` | Action button cluster |
| `EscrowDepositCard` | Funding UX (QR + address) |
| `EscrowStatusBar` | On-chain status indicator |
| `EscrowFundedStatus` | Confirmation state |
| `ProofOfPaymentDisplay` | Buyer's bank-transfer proof viewer |
| `PaymentProofCard` | Compact POP card (used in chat) |
| `TradeEventTimeline` | Audit log of state changes |
| `TradeStatusTimeline` | Visual progress bar |
| `DisputeEvidenceUpload` | Multi-file dispute upload |
| `TradeReviewForm` | Star rating + comment |
| `DownloadReceiptButton` | PDF receipt download (`ATF78HG5`) |
| `ParticipantsCard` | Buyer/seller mini-profiles |
| `CounterpartyTradeHistory` | Recent trades with this counterparty |

### 5.3 Wallet (`components/features/wallet/`)

| Component | Purpose |
|---|---|
| `WalletCard` | Per-asset balance card |
| `CustodialWalletCard` | Multi-chain custodial card variant |
| `WalletActions` | Deposit / Withdraw / Transfer buttons |
| `DepositModal` | Per-currency deposit flow |
| `WithdrawModal` | Per-currency withdrawal (TOTP gated) |
| `TransferModal` | Internal P2P transfer |
| `ManageWalletModal` | Per-asset settings |
| `TransactionHistory` | Filterable ledger view |
| `TransactionStatusModal` | Per-transaction detail |
| `PortfolioDistribution` | Donut chart |
| `PriceCards` | Live ticker tiles |
| `SecurityTip` | Inline tip cards |
| `GasFeeDisplay` | Fee estimator (uses `services/gas.rs`) |
| `LockFundsModal` | Pre-lock funds for offer creation |

### 5.4 Auth (`components/features/auth/`)

| Component | Purpose |
|---|---|
| `TwoFactorLoginModal` | 2FA challenge after password login |
| `TwoFactorCodeInput` | 6-digit auto-tab input |
| `TwoFactorTimer` | Code-validity countdown |

### 5.5 KYC (`components/features/kyc/`)

| Component | Purpose |
|---|---|
| `IdentityVerification` | Didit hosted-session iframe + status polling |
| `KYCWizard` | Tier-aware step-through |
| `KYCL3UpgradeSection` | Proof of address upload |
| `KYCVerificationProgressCard` | Status card |
| `KYCStatusBadge` | Inline verification badge |
| `KycRetakeCard` | Retake/resubmit affordance |

### 5.6 Settings (`components/features/settings/`)

| Component | Purpose |
|---|---|
| `ProfileSection` | Avatar + display name + username |
| `SecuritySettings` (with `security/*`) | Password / 2FA / sessions / OAuth |
| `PaymentMethodsSection` | Saved bank details |
| `KYCVerificationContent` | Wraps KYC for the settings tab |
| `BlockedUsersSection` | Manage blocks |
| `TradingLimitsSection` | Tier limit display |
| `NotificationSettings` | Per-event email/in-app prefs |

### 5.7 Dashboard (`components/features/dashboard/`)

| Component | Purpose |
|---|---|
| `StatsGrid` | KPI tiles |
| `QuickActions` | Buy / Sell / Deposit / Withdraw shortcuts |
| `WalletOverview` | Compact wallet card |
| `EscrowOverview` | Active escrow summary |
| `AffiliateSummary` | Affiliate KPIs |
| `MyOffersTable` | Top offers preview |
| `ActiveTradesTable` | In-flight trades |
| `RecentActivity` | Activity feed |
| `ProfitTrend` | Line chart |
| `MonthlyTradingVolume` | Bar chart |
| `PriceChart` | Sparkline tiles |

### 5.8 Navigation (`components/navigation/`)

| Component | Purpose |
|---|---|
| `Logo` | Home link, theme-aware SVG |
| `NavLinks` | Desktop + mobile nav buttons |
| `UserProfileDropdown` | Avatar menu (`PROFILE_MENU_ITEMS`) |
| `HeaderBalance` | ZAR portfolio total in header |
| `BalanceStrip` | Horizontal scroll of per-asset balances |
| `BalanceDisplay` | Currency-formatted balance text |
| `NotificationsDropdown` | Bell + recent notifications |
| `MobileMenuButton` | Hamburger trigger |

### 5.9 Brand (`components/brand/`)

| Component | Purpose |
|---|---|
| `QicLogo.tsx` | Inline SVG `QicIcon`, `QicLockup` — primary brand assets |

### 5.10 Common (`components/common/`)

| Component | Purpose |
|---|---|
| `mainHeader.tsx` | Header used by `(main)`, `(dashboard)`, `(offers)` |
| `authHeader.tsx` | Slim header for `(auth)` |
| `mobile-nav.tsx` | Right slide-over drawer |

### 5.11 UI Primitives (`components/ui/`)

43 Shadcn components: `button`, `input`, `textarea`, `select`, `checkbox`, `switch`, `tabs`, `dialog`, `sheet`, `dropdown-menu`, `popover`, `tooltip`, `toast`, `skeleton`, `table`, `card`, `badge`, `avatar`, `breadcrumb`, `field`, `password-input`, `loading-button`, `theme-toggle`, `user-avatar`, `error-boundary`, etc. See `DESIGN.md` §7 for tokens.

---

## 6. Public Asset Manifest

Paths relative to `frontend/public/`.

### 6.1 Brand & SEO

| File | Used by |
|---|---|
| `logo.svg`, `logo.png`, `wordmark.svg`, `logo-small.svg/png` | Favicon generation, SEO tests. **UI itself uses inline SVG**, not these files |
| `icon.svg`, `favicon.ico` | Root favicon |
| `og-image.png`, `og-source-icon.jpg` | `src/lib/seo/metadata.ts`, social cards |
| `icons/favicon-*.png`, `icon-192/512*.png`, `apple-touch-icon.png` | `metadata.ts`, `manifest.ts` |
| `manifest.webmanifest`, `robots.txt` | PWA / crawlers |

### 6.2 Landing

| File | Used by |
|---|---|
| `landing/hero.png` | `/` Home `Hero.tsx` (right column lg+) |
| `landing/*.svg` (trade, secure, resell, etc.) | Legacy landing art |
| `landing/wave.svg`, `wave_light.svg` | Decorative section dividers |

### 6.3 Auth Marketing

| File | Used by |
|---|---|
| `assets/signupbanner.svg` | `/signup` (right column) |
| `assets/loginbanner.svg` | `/verify-email`, `/verify-email-pending`, `/forgot-password`, `/reset-password` |
| `assets/logo.svg` | Duplicate lockup (tests/docs) |

### 6.4 Icons (`assets/icons/`)

| Set | Files | Used by |
|---|---|---|
| Crypto | `usdt.svg`, `bitcoin.svg`, `eth.svg`, `sol.svg`, `trx.svg` | `prices-api.ts`, wallet, marketplace, trade |
| UI | `shield`, `star`, `wallet4`, `greencheck`, `arrowleft`, `checkmark`, `eye`, `calendar`, `google`, `help`, `quote`, `chat`, `qrcode`, `search`, `alert`, `timer-outline`, `wallets`, `tips`, `bluecheck`, `lock` | Trade, offer, wallet, affiliate, auth, testimonials |
| Social | `facebook`, `twitter`, `ig`, `tg`, `yt`, `whatsapp` | Footer / marketing |

### 6.5 Email-only

| File | Used by |
|---|---|
| `email-assets/*`, `logo-email-2x.png`, `logo-shield-email.png` | Backend `services/email.rs` Brevo templates |
| `logo-google-512/1024.png` | Google OAuth branding |

### 6.6 Legal PDFs

| File | Used by |
|---|---|
| `legal/terms-*.pdf`, `privacy-*.pdf`, `cookies-*.pdf` | Download links from `/terms`, `/privacy`, `/cookies` pages |

### 6.7 Likely-unused (audit candidates)

| File | Notes |
|---|---|
| `next.svg`, `vercel.svg`, `file.svg`, `globe.svg`, `window.svg` | Next.js scaffold artifacts |
| `wave.svg`, `whats_icon.svg` | Legacy |
| `assets/icons/wallet3.svg` | Referenced in `AffiliateBanner` — verify file exists |

---

## 7. Functionality Walkthrough — Domain Model

The platform's behaviour is defined by six core entities. Each one has a state machine; transitions are the audit-bearing units of work.

| Entity | Purpose | Source of truth |
|---|---|---|
| **User** | Identity, role, KYC status | `users` table, `Role` / `KycStatus` enums |
| **Offer** | A standing buy or sell intent listed on the marketplace | `offers` + `offer_versions` |
| **Trade** | A specific bilateral execution against an offer | `trades` |
| **Escrow** | Custodial or on-chain hold of crypto for a trade or offer | `escrows` + `escrow_wallets` |
| **Wallet** | Per-user, per-crypto balance ledger | `wallets`, `wallet_locks`, `wallet_transactions` |
| **LedgerEntry** | Append-only audit trail of every credit/debit | `ledger_entries` |

**Invariants enforced in code (`tests/mpr001_*`, `tests/escrow_balance.rs`):**

1. Every value-bearing state transition writes a `ledger_entries` row in the same DB transaction.
2. `wallets.locked_balance` equals the sum of unreleased `wallet_locks.amount` for that user/crypto.
3. Escrow released = exactly one fee ledger entry + exactly one credit to the buyer wallet (or refund to seller).
4. A trade's `offer_versions` snapshot is immutable once the trade is created.

---

## 8. State Machines

### 8.1 Trade (`TradeStatus`)

**Terminal states:** `completed`, `cancelled`, `resolved`.

```
                 ┌──────────────┐
                 │   created    │
                 └──────┬───────┘
                        │ (auto: lock crypto in custodial escrow)
                        ▼
                 ┌──────────────┐
       ┌─────────│ escrow_funded│─────────────┐
       │         └──────┬───────┘             │
       │ (cancel        │ buyer marks paid    │ seller releases early
       │  window)       ▼                     │
       │         ┌──────────────┐             │
       │         │     paid     │─────────────┤
       │         └──┬─────┬─────┘             │
       │            │     │ open dispute      │
       │            │     ▼                   ▼
       │            │  ┌───────────┐    ┌──────────┐
       │            │  │ disputed  │    │ released │
       │            │  └─────┬─────┘    └────┬─────┘
       │            │        │ moderator     │ (auto)
       │            │        ▼               ▼
       │            │  ┌──────────┐    ┌──────────┐
       │            │  │ resolved │    │completed │
       │            │  └──────────┘    └──────────┘
       │            │ seller releases
       │            ▼
       │     ┌──────────┐
       │     │completed │
       │     └──────────┘
       ▼
  ┌──────────┐
  │cancelled │
  └──────────┘
```

| Transition | Allowed actor | Side effect |
|---|---|---|
| `created → escrow_funded` | System | Custodial wallet lock + `wallet_locks` row + ledger `escrow_lock` |
| `escrow_funded → paid` | Buyer / admin | No money moves; just marker |
| `paid → released` | Seller / vendor / admin | Escrow `release` service runs |
| `released → completed` | Auto | Affiliate commission payouts; PDF receipt available |
| `* → disputed` | Buyer or seller | Triggers `services/dispute.rs` snapshot capture |
| `disputed → resolved` | Moderator+ | `resolve_to_seller` or `resolve_to_buyer` |
| `escrow_funded/paid → cancelled` | Participant (TF-005 window) or admin | Refund via `services/escrow_refund.rs` |

Implemented in `src/types/enums.rs::TradeStatus::can_transition_to()` — exhaustive match, tested in `tests/triage_stories.rs`.

### 8.2 Escrow (`EscrowStatus`)

**Terminal states:** `released`, `refunded`, `expired`.

```
   pending → awaiting_deposit → held → released
                              ↓        ↓
                          (custodial)  refunded
                              ↓
                          disputed → released | refunded

   pending | awaiting_deposit → expired   (admin only, no funds at risk)
```

**Escrow types:**

| Type | When | Mechanism |
|---|---|---|
| `custodial` | Default for marketplace trades | Lock in user's custodial wallet via `wallet_locks` |
| `on_chain` | Advanced / vendor offers | Real on-chain multisig or escrow contract |
| `offer_escrow` | Pre-funded offer (vendor stakes upfront) | Address tied to offer, drawn down per trade |
| `btc_wallet_lock` | BTC-specific | UTXO lock with gas reserve (`BTC_GAS_RESERVE_SATS`) |

### 8.3 Offer (`OfferStatus`)

```
   active ⇄ paused → closed
                   ↘ deleted (soft)
```

| Action | Endpoint | Notes |
|---|---|---|
| Create | `POST /offers` | KYC-gated; `offer_versions` row created |
| Edit | `PUT /offers/:id` | New version snapshotted |
| Pause | `PATCH /offers/:id/pause` | Hidden from marketplace, in-flight trades unaffected |
| Resume | `PATCH /offers/:id/resume` | Reverts to `active` |
| Close | `PATCH /offers/:id/close` | Terminal, listing removed |
| Delete | `DELETE /offers/:id` | Soft-delete |

**Offer visibility:** `public` (marketplace), `private` (link only), `direct` (sent to specific user).

**Auto-pause job** (`run_offer_auto_pause`, every 15 min): pauses **resell offers** when the parent vendor offer is unavailable or vendor balance is too low to honour any size.

### 8.4 KYC (per-user `KycStatus`)

```
   none → initialized → pending → approved
                              ↘ rejected (resubmit allowed)
```

**Tier progression** (separate `kyc_submissions.level` field):

| Level | Name | Daily / Monthly USD | Documents |
|---|---|---|---|
| 0 | Unverified | $100 / $500 | none |
| 1 | Basic | $1,000 / $5,000 | Government ID |
| 2 | Verified | $10,000 / $50,000 | ID + selfie liveness |
| 3 | Premium | $100,000 / $500,000 | ID + selfie + proof of address |

Providers: **SumSub** (incumbent) and **Didit** (in migration — see `architecture/DIDIT-MIGRATION-SPEC.md`). Provider chosen by `services/kyc_provider.rs`.

### 8.5 User Account

| Field | States | Effect |
|---|---|---|
| `role` | `user`, `moderator`, `admin`, `super_admin`, `investor` | Gates moderator/admin/war-room routes |
| `kyc_status` | None / Pending / Approved / Rejected | Action-time gate |
| `is_active` | bool | Soft-delete protection |
| `is_suspended`, `suspended_until` | timestamp | Mod suspension |
| `is_banned` | bool | Mod ban; admin to unban |
| `deleted_at` | timestamp | Soft-deleted, can be reactivated by admin |

### 8.6 Wallet Transactions (`WalletTransactionStatus`)

```
   pending → confirmed | failed | completed
```

Terminal: `confirmed`, `failed`, `completed`. Type-specific behaviour:

| Type | Notes |
|---|---|
| `deposit` | Detected by webhook (Alchemy / Helius) or polled by `run_deposit_monitor` |
| `withdrawal` | Built and broadcast by `services/blockchain/*`; TOTP-gated when 2FA enabled |
| `internal_transfer` | Off-chain instant move between users |
| `escrow_lock` / `escrow_release` | Created by escrow services; debits/credits `wallets.locked_balance` |
| `sweep` | `deposit_sweep.rs` moves user funds → platform hot wallet |
| `fee_sweep` | `fee_sweep.rs` moves accumulated platform fees → fee wallets |

### 8.7 Deposit / Sweep flow

1. User views deposit address (`GET /wallet/deposit-address/:crypto/:network`).
2. **Detection:**
   - **ETH / ERC-20** — Alchemy webhook → `POST /webhooks/alchemy` (HMAC verified).
   - **Solana / SPL** — Helius webhook → `POST /webhooks/helius` (auth header).
   - **BTC / TRX** — polled every **60s** by `run_deposit_monitor`.
3. **Credit:** atomic compare on-chain balance vs already-credited; if delta found, append ledger `deposit` + credit wallet + emit notification.
4. **Sweep:** `deposit_sweep.rs` moves token from user custodial wallet → platform hot wallet, writes ledger `sweep` entry.
5. **Fee sweep:** `fee_sweep.rs` consolidates platform fees to dedicated fee wallets every 5 min.

---

## 9. End-to-End User Journeys

### 9.1 Onboarding (signup → first KYC)

1. **`/signup`** → `POST /auth/signup` (rate-limited 5/IP/15min).
   - Validates email format, username uniqueness (live `/users/check-username/:username`), password strength.
   - Records `referralCode` if provided (validated via `/auth/validate-referral`).
   - Captures `acceptedLegal` flag against current legal doc version.
   - Sends verification email via `services/email.rs` (Brevo).
   - User redirected to `/verify-email-pending`.
2. **Email link** → `GET /auth/verify-email?token=...` → confirms email.
3. **`/login`** → `POST /auth/login`.
   - Returns access token (24h) + refresh token (30d) + user profile.
   - HttpOnly cookies `qic_access`, `qic_refresh`, plus `qic_csrf` for double-submit (SEC-COOKIES-001).
   - If 2FA enabled: returns `requiresTwoFactor: true` + `twoFactorToken`; client opens `TwoFactorLoginModal`.
4. **First action that needs KYC** → `KYCRequirementModal` opens.
   - Tier required is computed from intended action's USD value vs current tier limits.
   - User taken to `/settings?tab=verification` → `IdentityVerification` component creates a Didit (or SumSub) session.
   - Submits documents → status polls every 5s on `POST /kyc/didit/refresh` or via webhook (`POST /webhooks/kyc/didit`).
   - On approval, `users.kyc_status = approved` and the originating action becomes available.

**Backend touch points:** `services/auth.rs`, `services/kyc_provider.rs`, `services/legal.rs`, `services/email.rs`.

### 9.2 Buyer journey — buying USDT with ZAR

1. Browse **`/marketplace`** with filters (Buy USDT, ZAR, payment = bank transfer, max 5,000 ZAR).
2. Click an offer → **`/offer/[id]`**.
3. Type ZAR amount → frontend POSTs `/offers/:id/quote` → updates the USDT amount + price breakdown atomically.
4. Click **Start Trade** → KYC gate evaluates intended USD value vs tier limit.
5. `POST /trades` with `{ offerId, fiatAmount, paymentMethod }` → backend:
   - Verifies offer is active, buyer ≠ seller, KYC sufficient.
   - Creates `trades` row with `status=created`.
   - Snapshots `offer_versions` (immutable record of price + terms).
   - Locks crypto from seller's custodial wallet (`wallet_locks`) → `status=escrow_funded`.
   - Writes `ledger_entries` (`escrow_lock`).
   - Returns trade ID; frontend navigates to **`/trade/[id]`**.
6. On `/trade/[id]`:
   - Buyer receives bank account details from seller (chat or POP card).
   - Buyer transfers fiat off-platform.
   - Buyer uploads bank-transfer screenshot (`POST /trades/:id/attachments`) and clicks **Mark Paid** (`POST /trades/:id/mark-paid`).
7. Seller confirms receipt off-platform, clicks **Release** (`POST /trades/:id/complete` or `POST /escrow/:id/release`):
   - Escrow service runs:
     - Computes platform fee (`PLATFORM_FEE_BPS=70` bps default).
     - Credits buyer wallet with `amount - fee`.
     - Credits platform fee accumulator.
     - Writes ledger entries: `escrow_release` (debit), `transfer` (credit buyer), `platform_fee` (credit fee wallet).
     - Releases `wallet_locks` row.
     - Triggers `services/affiliate_commission.rs` for any referrals up the 3-level chain.
   - Trade status: `released → completed`.
8. Both parties can rate (`POST /trades/:id/rating`) and download PDF receipt (`GET /trades/:id/receipt.pdf`).

### 9.3 Seller journey — listing an offer

1. **`/offer/create`** wizard (`src/components/offer/create/*`):
   - Step 1: Offer type (Buy or Sell crypto).
   - Step 2: Crypto + fiat + pricing mode (`fixed` or `floating`/`premium`).
   - Step 3: Min / max amount, payment methods, time limit.
   - Step 4: Escrow type (custodial default; on-chain advanced).
   - Step 5: Terms & visibility.
2. `POST /offers` → KYC-gated, validated against tier `max_amount`.
3. For sell offers with custodial escrow: optional pre-fund via `/offer/[id]/fund-escrow` so listing is instantly tradeable.
4. Offer appears on `/marketplace` (visibility = public). Seller can pause/resume/close from `/my-offers`.

### 9.4 Reseller journey — relisting with markup

1. Reseller browses public sell offers from a vendor whose offer is "resellable".
2. Clicks **Resell** → `/offer/resell` → sets markup % (default from `reseller_profiles.default_markup_pct`).
3. `POST /reseller/resell/:offer_id` creates a child offer with `parent_offer_id` set.
4. Markup commission split:
   - Reseller earns `markup_amount * (1 - RESELLER_FEE_BPS/10000)` (75% by default).
   - Platform earns the `RESELLER_FEE_BPS` portion (25%).
5. On trade completion, escrow release service splits proceeds three ways: vendor (cost), reseller (markup minus cut), platform (cut + base fee).

### 9.5 Dispute journey

1. Either participant on `paid` status clicks **Open Dispute** → `POST /trades/:id/dispute` with reason.
2. `services/dispute.rs` snapshots trade state, sets `trades.status=disputed`, creates `disputes` row with priority + deadlines.
3. Both parties upload evidence via `DisputeEvidenceUpload` → `POST /trades/:id/dispute/evidence`.
4. Notifications sent to moderator queue. `/moderator/disputes` shows priority-sorted queue.
5. Moderator reviews trade context, chat, evidence, ledger entries.
6. Moderator clicks **Resolve to Buyer** (`POST /escrow/:id/resolve-to-buyer`) or **Resolve to Seller** (`/resolve-to-seller`).
7. `run_dispute_deadline_monitor` (every 15 min) auto-resolves expired disputes by refunding seller and cancelling the trade.

### 9.6 Withdrawal journey

1. **`/wallet`** → click **Withdraw** on asset → `WithdrawModal` opens.
2. Frontend pre-checks `GET /wallet/withdraw/check` → returns network availability per crypto.
3. User enters destination address + amount + selects network.
4. Network fee estimate displayed via `services/gas.rs` (live or cached).
5. If 2FA enabled, TOTP code field required.
6. `POST /wallet/withdraw` with `{ cryptocurrency, amount, network, toAddress, totpCode? }`:
   - Backend verifies balance sufficient (available, not locked).
   - Verifies TOTP if enabled.
   - Creates `wallet_transactions` row (status=pending).
   - Builds + signs + broadcasts via `services/blockchain/{ethereum,bitcoin,solana,tron}.rs`.
   - Writes ledger `withdrawal` entry.
   - Polling job updates status to `confirmed` once on-chain confirmations sufficient.

---

## 10. API Surface (Reference)

Base prefix: `/api/v1`. JSON wire format is `camelCase` (`#[serde(rename_all = "camelCase")]`).

### 10.1 Auth — `src/api/auth.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| POST | `/auth/signup` | Public, rate-limited | Email/password registration |
| POST | `/auth/login` | Public, rate-limited | Login (returns 2FA challenge if enabled) |
| POST | `/auth/2fa/verify-login` | Public | Complete 2FA challenge |
| POST | `/auth/refresh-token` | Refresh | Rotate access token |
| POST | `/auth/logout` | Authed | Revoke this session |
| POST | `/auth/logout-all` | Authed | Revoke all sessions |
| POST | `/auth/forgot-password` | Public | Send reset email |
| POST | `/auth/reset-password` | Public | Set new password from token |
| GET | `/auth/verify-email` | Public | Confirm email |
| POST | `/auth/resend-verification` | Public | Resend verify email |
| POST | `/auth/oauth-login` / `/oauth-signup` | Public | Google / Apple OAuth |
| GET | `/auth/sessions` | Authed | List active sessions |
| DELETE | `/auth/sessions/:id` | Authed | Revoke a session |
| DELETE | `/auth/delete/:user_id` | Authed | Soft-delete account (password required) |

### 10.2 Profile / Settings — `src/api/users.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET / PATCH | `/users/me` | Authed | Read / update profile |
| POST | `/users/me/avatar` | Authed | Avatar upload |
| GET | `/users/me/stats` | Authed | Trade stats |
| GET / PATCH | `/users/me/notifications` | Authed | Notification prefs |
| GET / PUT | `/users/profile` | Authed | Extended profile (`user_profiles`) |
| GET / PUT | `/users/settings` | Authed | User settings |
| GET | `/users/2fa/status` | Authed | Enabled? |
| POST | `/users/2fa/setup` | Authed | Start TOTP setup |
| POST | `/users/2fa/verify` | Authed | Confirm + receive backup codes |
| POST | `/users/2fa/disable` | Authed (password + TOTP) | Disable 2FA |
| POST | `/users/2fa/backup-codes/regenerate` | Authed | Regenerate backup codes |
| POST | `/users/password/change` | Authed | Change password |
| GET | `/users/top-traders` | Public | Leaderboard |
| GET | `/users/search` | Public | User search |
| GET | `/users/:user_id/public` | Public | Public profile |
| GET | `/users/:user_id/ratings` | Public | Ratings |
| POST | `/users/:user_id/report` | Authed | Report user |
| POST / DELETE | `/users/:user_id/block` | Authed | Block / unblock |
| GET | `/users/blocked` | Authed | Block list |
| GET | `/users/check-username/:username` | Public | Availability |

### 10.3 KYC — `src/api/kyc.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/kyc/tiers` | Public | L0–L3 catalog |
| GET | `/kyc/limits/:level` | Public | Static limits |
| GET | `/kyc/my-limits` | Authed | Limits + usage |
| GET | `/kyc/status` | Authed | Status + submissions |
| POST | `/kyc/submit` | Authed | Manual submission |
| POST | `/kyc/sumsub/token` | Authed | SumSub SDK token |
| POST | `/kyc/sumsub/webhook` | Webhook | SumSub callbacks |
| POST | `/kyc/session` | Authed | Didit hosted session URL |
| POST | `/kyc/didit/refresh` | Authed | Poll Didit status |
| POST | `/kyc/upgrade-to-l3` | Authed | Request L3 |
| POST | `/kyc/documents/upload` | Authed | Upload doc |
| GET / DELETE | `/kyc/documents/:doc_id` | Authed/Mod | Get / delete |
| GET | `/kyc/documents/:doc_id/download` | Authed/Mod | Download (IDOR-protected) |

### 10.4 Wallet — `src/api/wallet.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/wallet` | Authed | All balances |
| GET | `/wallet/balance` | Authed | Summary |
| GET | `/wallet/transactions` | Authed | Tx history |
| GET | `/wallet/pending` | Authed | Pending |
| GET | `/wallet/transfers` | Authed | Internal transfers |
| GET | `/wallet/deposit-address` | Authed | Default deposit address |
| GET | `/wallet/deposit-address/:crypto` | Authed | Per-crypto |
| GET | `/wallet/deposit-address/:crypto/:network` | Authed | Per-network |
| GET | `/wallet/locks` | Authed | Active wallet locks |
| GET | `/wallet/withdraw/check` | Authed | Network availability |
| POST | `/wallet/deposit` | Authed | Manual deposit claim |
| POST | `/wallet/withdraw` | Authed (+ TOTP) | On-chain withdrawal |
| POST | `/wallet/transfer` | Authed | Internal P2P transfer |
| POST | `/wallet/locks/lock` / `unlock` / `check` | Authed | Pre-trade locks |

### 10.5 Custodial Wallet — `src/api/custodial_wallet.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| POST | `/custodial-wallet/generate` | Authed | Generate HD wallet |
| GET | `/custodial-wallet` | Authed | Primary wallet |
| GET | `/custodial-wallet/all` | Authed | All chains |
| GET | `/custodial-wallet/balance` | Authed | Combined balance |
| POST | `/custodial-wallet/send` | Authed | Send on-chain |
| POST | `/custodial-wallet/export` | Authed | Export mnemonic (high-risk action) |

### 10.6 Offers — `src/api/offers.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/offers` | Optional | Marketplace listing |
| GET | `/offers/buy` / `/sell` | Optional | Filtered |
| POST | `/offers` | KYC | Create |
| GET | `/offers/me` | Authed | Own offers |
| GET | `/offers/user/:user_id` | Public | User's public offers |
| GET | `/offers/:offer_id` | Optional | Detail |
| PUT | `/offers/:offer_id` | Owner | Edit (versioned) |
| DELETE | `/offers/:offer_id` | Owner/admin | Soft-delete |
| GET | `/offers/:offer_id/versions` | Authed | Version history |
| POST | `/offers/:offer_id/quote` | Authed | Deterministic quote |
| PATCH | `/offers/:offer_id/pause` / `/resume` / `/close` | Owner | Lifecycle |

### 10.7 Direct Offers — `src/api/direct_offers.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| POST | `/direct-offers` | Authed | Send private offer to user |
| GET | `/direct-offers/received` / `/sent` | Authed | Inbox / outbox |
| POST | `/direct-offers/:offer_id/accept` / `/decline` / `/cancel` | Authed | Lifecycle |

### 10.8 Trades — `src/api/trades.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| POST | `/trades` | KYC | Start trade |
| GET | `/trades` / `/active` / `/completed` | Authed | Lists |
| GET | `/trades/:trade_id` | Participant/mod | Detail |
| PATCH | `/trades/:trade_id/status` | Participant/mod | State transition |
| POST | `/trades/:trade_id/cancel` | Participant/admin | Cancel (TF-005 rules) |
| POST | `/trades/:trade_id/complete` | Seller/vendor/admin | Mark completed |
| POST | `/trades/:trade_id/mark-paid` | Buyer/admin | Buyer marked fiat sent |
| POST | `/trades/:trade_id/dispute` | Participant | Open dispute |
| GET | `/trades/:trade_id/events` | Participant/mod | Audit timeline |
| GET | `/trades/:trade_id/ledger` | Participant/mod | Ledger entries |
| GET / POST | `/trades/:trade_id/messages` | Participant | Chat |
| POST | `/trades/:trade_id/attachments` | Participant | Upload |
| GET | `/trades/:trade_id/attachments/:filename/download` | Authed | Download |
| GET | `/trades/:trade_id/receipt.pdf` | Participant | PDF receipt |
| POST | `/trades/:trade_id/rating` | Participant | Rate counterparty |
| POST | `/trades/:trade_id/dispute/evidence` | Participant | Upload evidence |

### 10.9 Escrow — `src/api/escrow.rs`

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/escrow` / `/active` / `/stats` / `/overview` | Authed | Lists |
| POST | `/escrow/custodial/create` | Authed | Create custodial escrow |
| POST | `/escrow/custodial/:id/deposit` | Authed | Fund |
| POST | `/escrow/custodial/:id/release` | Seller/admin | Release |
| POST | `/escrow/offer/create` | Authed | Offer-level escrow |
| GET | `/escrow/offer/:offer_id/balance` | Authed | On-chain balance |
| POST | `/escrow/offer/:offer_id/confirm-deposit` | Authed | Confirm deposit |
| GET | `/escrow/trade/:trade_id` | Participant | Trade escrow |
| POST | `/escrow/trade/:trade_id/confirm-deposit` | Participant | Confirm |
| POST | `/escrow/:escrow_id/release` / `/refund` / `/dispute` | Role-gated | Escrow lifecycle |
| POST | `/escrow/:escrow_id/resolve-to-seller` / `/resolve-to-buyer` | Mod | Dispute resolution |

### 10.10 Other API Areas

| Prefix | File | Notes |
|---|---|---|
| `/payment-methods` | `payment_methods.rs` | Encrypted bank details CRUD |
| `/dashboard` | `dashboard.rs` | User dashboard data |
| `/prices` | `prices.rs` | Coin prices, FX, alerts |
| `/config` | `platform_config.rs` | Platform fees, limits, supported assets |
| `/notifications` | `notifications.rs` | In-app notifications |
| `/affiliate` | `affiliate.rs` | Stats, payouts, leaderboard |
| `/reseller` | `reseller.rs` | Reseller markup, settlement |
| `/mod/*` | `moderation.rs` | Moderator queues + actions |
| `/admin/*` | `admin.rs` | Admin operations |
| `/admin/war-room/*` | `admin_war_room.rs` | KPIs, treasury, cohorts |
| `/webhooks/alchemy` | `webhooks.rs` | ETH deposits |
| `/webhooks/helius` | `webhooks.rs` | Solana deposits |
| `/webhooks/kyc/didit` | `webhooks.rs` | Didit KYC status |
| `/support` | `support.rs` | Tickets |
| `/help` | `help.rs` | Articles |
| `/legal-acceptances` | `legal.rs` | Legal doc versioning |
| `/chatbot` | `chatbot.rs` | Support chatbot |
| `/reports` | `reports.rs` | User/trade reports |
| `/bug-reports` | `bug_reports.rs` | Bug submissions |
| `/whatsapp` | `whatsapp.rs` | WhatsApp escrow link tracking |
| `/gas` | `gas.rs` | Fee estimation, treasury |

---

## 11. Background Jobs & Side Effects

Defined in `src/jobs.rs`, spawned from `src/main.rs`.

| Job | Frequency | What it does |
|---|---|---|
| `warm_price_history_cache` | Once at startup | Pre-warm CoinGecko history |
| `run_fx_poller` | Continuous loop | Live FX rates for enabled fiats |
| `run_price_alert_job` | 30s | Evaluate price alerts, broadcast Socket.IO |
| `run_dispute_deadline_monitor` | 15 min | Auto-resolve expired disputes (refund seller, cancel trade) |
| `run_trade_expiry_monitor` | 60s | Cancel expired trades, refund escrow |
| `run_treasury_monitor` | configurable | TRX auto-swap when treasury low |
| `run_fee_sweep_monitor` | 5 min | Sweep platform fees to fee wallets |
| `run_deposit_monitor` | 60s | Poll custodial addresses, credit deposits |
| `run_offer_auto_pause` | 15 min | Pause resell offers when parent unavailable |
| `run_reconciliation_job` | 6 hours | Ledger / wallet reconciliation |

Real-time push events use Socket.IO via `src/ws.rs` and `services/notify.rs`:

| Event | Triggered by | Consumed by |
|---|---|---|
| `notification:new` | Any service that calls `notify::push` | `NotificationsDropdown`, `notification-store` |
| `trade:status:changed` | Trade state transitions | `/trade/[id]` page |
| `trade:message:new` | `POST /trades/:id/messages` | `TradeChat` |
| `wallet:balance:updated` | Deposit credit, withdrawal confirm, escrow release | `BalanceStrip`, wallet page |
| `price:alert:fired` | `run_price_alert_job` | Notifications |

---

## 12. Limits, Fees, Currencies

### 12.1 Currencies

**User-facing crypto (MVP):** BTC, ETH, USDT (TRC-20 primary; SPL on Solana via feature flag), SOL.
**Backend-only / plumbing:** TRX (gas), USDC.
**Fiat:** ZAR (always); NGN when `ENABLE_NGN=true`. USD/EUR/GBP exist as types but are not marketplace-enabled.
**Networks:** `bitcoin_mainnet`, `ethereum_mainnet`, `solana_mainnet`, `tron_mainnet` (+ extras in enum for future).

### 12.2 KYC tier limits (USD)

| Level | Daily | Monthly |
|---|---|---|
| L0 | $100 | $500 |
| L1 | $1,000 | $5,000 |
| L2 | $10,000 | $50,000 |
| L3 | $100,000 | $500,000 |

Enforced on offer create/update (`max_amount` vs daily limit) and per-trade.

### 12.3 Platform limits (`src/config.rs`)

| Env | Default | Purpose |
|---|---|---|
| `MIN_TRADE_ZAR` | ~50 | Min trade |
| `MAX_TRADE_ZAR` | ~100,000 | Max trade |
| `DAILY_LIMIT_ZAR` | ~500,000 | Platform daily cap |
| `MIN/MAX/DAILY_LIMIT_NGN` | — | NGN equivalents |

### 12.4 Fees (`FeeConfig`)

| Fee | Env | Default | Who pays |
|---|---|---|---|
| Platform escrow fee | `PLATFORM_FEE_BPS` | 70 (0.7%) | Crypto seller |
| Taker fee | `TAKER_FEE_BPS` | 0 | Buyer (usually zero) |
| Reseller platform cut | `RESELLER_FEE_BPS` | 2500 (25%) | Of reseller markup |
| Reseller escrow fee | `RESELLER_ESCROW_FEE_BPS` | 0 | Resell trades |
| Vendor fee on resells | `VENDOR_RESELL_FEE_BPS` | 0 | Resell trades |
| BTC gas reserve | `BTC_GAS_RESERVE_SATS` | 10,000 sats | Locked with BTC escrow |
| Network withdrawal fees | dynamic | `services/gas.rs` | User (may be sponsored) |

Admins can mutate fee bps at runtime via `PUT /api/v1/admin/config`.

---

## 13. Auth, MFA, Security

### 13.1 JWT & Sessions (`services/auth.rs`)

| Token | TTL | Audience | Notes |
|---|---|---|---|
| Access | 24h | `qictrader-api` | Carries `session_id` matching live `user_sessions` row |
| Refresh | 30d | `qictrader-refresh` | Rotated on use (SEC-REFRESH-ROTATION-001 stores JTI) |
| 2FA session | short-lived | separate audience | Used between password and TOTP step only |

Logout, password change, or admin action revokes sessions immediately.

### 13.2 Cookies (SEC-COOKIES-001)

`qic_access` and `qic_refresh` are HttpOnly, Secure, SameSite=Lax. `qic_csrf` is a non-HttpOnly companion read by the frontend and submitted as `X-CSRF-Token` header (double-submit pattern, enforced by `middleware/csrf.rs`).

### 13.3 MFA Flow (TOTP)

1. `POST /users/2fa/setup` → returns shared secret + QR URL.
2. User adds in authenticator app (Authy / Google Auth).
3. `POST /users/2fa/verify` with first 6-digit code → enables 2FA, returns 10 backup codes.
4. From now on, login returns `requiresTwoFactor`.
5. `POST /auth/2fa/verify-login` completes login.
6. Withdrawals require `totpCode` whenever 2FA is enabled (`services/totp.rs`).

### 13.4 Rate Limits (`services/rate_limit.rs`, in-memory fixed-window)

| Endpoint | Policy |
|---|---|
| Login | 5/IP/5min, 10/email/hour |
| Signup | 5/IP/15min |
| Forgot password | 1/email/10min, 10/IP/hour |

Returns 429 with generic message + `Retry-After` header (PENTEST-002).

### 13.5 IDOR Protections

| Resource | Check |
|---|---|
| Trades | `is_trade_participant()` (buyer / seller / vendor); mods bypass where appropriate |
| Trade attachments | Token validated even when passed in query string |
| KYC documents | Owner-or-moderator download authz (`tests/kyc_l3_doc_download_authz.rs`) |
| Legal acceptances | Per-user only (`tests/legal_idor.rs`) |
| Escrow / wallet | Participant or role checks in handlers |
| User reads | `require_self_or_admin()` for sensitive fields |
| Role parsing | `Role::from_access_token_claim` rejects `refresh` / `2fa_session` / unknown audiences |

### 13.6 Other Middleware

- **CORS** — explicit origin allowlist (`src/app.rs`).
- **Security headers** — CSP, HSTS, X-Robots-Tag by environment (`middleware/security_headers.rs`).
- **Mainnet guard** — non-prod refuses mainnet RPC URLs (PENTEST-004).
- **Webhook signing** — HMAC for Alchemy, auth header for Helius, signature for Didit.
- **KYC mandatory gate** — feature-flagged `KycVerifiedUser` extractor on offer/trade create.

---

## 14. State Management (Frontend)

Three tiers: Redux (legacy / persisted UI), Zustand (most stores), React Query (server state).

### 14.1 Redux (`src/store/redux/`)

| Slice | Persists | Purpose |
|---|---|---|
| `uiSlice` | theme, sidebarOpen | Theme + sidebar |
| `offerSlice` | draftOffers | Offer creation drafts |
| `sellOfferHeaderSlice` | currency, tab | Sell-tab header state |
| `whatsappSlice` | drafts, recent | WhatsApp escrow links |
| `directTradeSlice` | formData | Direct/custom trade modal |
| `pricesSlice` | rates cache | Price display cache |

**Auth is explicitly not in Redux.**

### 14.2 Zustand (`src/store/`)

| Store | Persists | Purpose |
|---|---|---|
| `auth-store` | profile fields (not JWT) | Current session user; tokens kept in memory |
| `user-store` | profile | Extended profile / photoURL |
| `offer-store` | resell flow | Resell/escrow modal state |
| `trade-store` | trades cache | Trade list + current trade + messages |
| `wallet-store` | wallet UI | Wallet page state |
| `notification-store` | notifications | Cache + unread count |
| `ui-store` | theme, tabs, modals | UI prefs (overlap with Redux ui slice) |
| `moderation-store` | mod filters | Moderator UI filters |
| `consent-store` | consent UI | GDPR banner state |

### 14.3 React Query (`src/hooks/api/`)

Page-level orchestrators (each returns the full data + loading/error a page needs):

`use-marketplace-page`, `use-trade-detail-page`, `use-offer-detail-page`, `use-profile-page`, `use-wallet-page`, `use-settings-page`, `use-dashboard`, `use-backend-dashboard`.

Domain hooks: `use-users`, `use-offers`, `use-trades`, `use-escrow`, `use-escrow-balance`, `use-wallet`, `use-payment-methods`, `use-prices`, `use-affiliate`, `use-reseller`, `use-notifications`, `use-support`, `use-moderation`, `use-kyc`, `use-kyc-gate`, `use-whatsapp`, `use-user-search`, `use-direct-offers`, `use-admin`, `use-war-room`, `use-me`, `use-trade-history`, `use-platform-fees`, `use-platform-config`, `use-gas-fees`, `use-market-prices`, `use-zar-conversion`, `use-crypto-conversion`, `use-custodial-wallet`, `use-escrow-recovery`, `use-reports`, `use-testimonials`, `use-transaction-status`, `use-active-listings`, `use-mod-support`, `use-offer-escrow`, `moderation/*`.

### 14.4 Auth Provider

`AuthProvider` (`src/components/providers/auth-provider.tsx`) wraps the app:

- Hydrates `auth-store` from refresh cookie.
- Redirects unauthenticated users on protected routes → `/login`.
- Redirects authed users on `/login` or `/signup` → `/dashboard`.
- Shows full-screen loader during initial auth check.

`useAuth` hook (`src/hooks/auth/`) composes: `useAuthState`, `useEmailAuth`, `useOAuth`, `useAuthActions`, `useIdleTimeout`.

---

## 15. Empty / Loading / Error States

Conventions used across the app — reference for when designing new screens.

### 15.1 Loading

| Surface | Pattern |
|---|---|
| Lists (offers, trades, transactions) | Skeleton rows (`OfferCardSkeleton`, etc.) — same shape as the loaded row |
| Page-level | Top progress bar + skeleton blocks |
| Buttons during action | `LoadingButton` with inline spinner; button disabled |
| Modal data | Skeleton inside modal, not closed |
| Auth check on protected route | Full-screen spinner + brand mark |

Animation: `shimmer` keyframe (1.5s, see `DESIGN.md` §8.3).

### 15.2 Empty

| Surface | Pattern |
|---|---|
| Marketplace | Centered illustration + "No offers match these filters" + "Reset filters" CTA |
| My offers / trades | Centered icon + "No offers yet" + "Create your first offer" CTA |
| Notifications | Centered bell icon + "You're all caught up" |
| Wallet (zero balance) | Card stays visible; "Deposit" CTA prominent; "Show zero balances" toggle controls visibility |
| Search results | Inline "No matches" within the list area |

### 15.3 Error

| Surface | Pattern |
|---|---|
| Form field | Red text below field, red border, `aria-invalid` |
| Form submit | Top-of-form red banner + field-level errors |
| API failure (non-critical) | Toast (Sonner) bottom-right, dismissible, action-link "Retry" |
| API failure (page) | Centered card with icon + message + "Try again" / "Go home" |
| Boundary crash | `error-boundary.tsx` shows fallback page with bug-report link |
| 404 | Centered "Page not found" + nav back to `/marketplace` |
| Permission denied | "You don't have access to this page" + "Go home" |
| KYC required | `KYCRequirementModal` — explains tier required + CTA to `/settings?tab=verification` |
| Rate limited | Toast: "Too many attempts. Try again in {retry-after}." |

### 15.4 Confirmation / Success

| Surface | Pattern |
|---|---|
| Action complete | Green toast bottom-right + state update |
| Trade completed | Inline success card on `/trade/[id]` + receipt button |
| Offer created | Redirect to `/my-offers` + green toast |
| Withdrawal sent | Toast + transaction appears in `TransactionHistory` with status="pending" → "confirmed" |

---

## Appendix A — Source Map

Use this when extending the app to know where each concern lives.

| Concern | Frontend | Backend |
|---|---|---|
| Auth flow | `src/app/(auth)/*`, `src/hooks/auth/*`, `src/components/providers/auth-provider.tsx` | `src/api/auth.rs`, `src/services/auth.rs`, `src/services/auth_cookies.rs` |
| KYC | `src/app/(dashboard)/settings/page.tsx?tab=verification`, `src/components/features/kyc/*` | `src/api/kyc.rs`, `src/services/kyc_provider.rs`, `src/services/sumsub.rs`, `src/services/didit.rs` |
| Marketplace | `src/app/(main)/marketplace/page.tsx`, `src/components/features/marketplace/*` | `src/api/offers.rs` |
| Trade room | `src/app/(main)/trade/[id]/page.tsx`, `src/components/features/trade/*` | `src/api/trades.rs`, `src/services/dispute.rs`, `src/services/notify.rs` |
| Escrow | `src/components/features/trade/EscrowDepositCard.tsx`, `src/app/(offers)/offer/[id]/fund-escrow/page.tsx` | `src/api/escrow.rs`, `src/services/escrow_release.rs`, `src/services/escrow_refund.rs`, `src/services/escrow_balance.rs` |
| Wallet | `src/app/(dashboard)/wallet/*`, `src/components/features/wallet/*` | `src/api/wallet.rs`, `src/api/custodial_wallet.rs`, `src/services/blockchain/*`, `src/services/wallet_crypto.rs` |
| Ledger | (read-only via trade page) | `src/services/ledger.rs`, `src/repo/ledger.rs` |
| Affiliate | `src/app/(main)/affiliate/page.tsx`, `src/app/(dashboard)/dashboard/affiliate/*` | `src/api/affiliate.rs`, `src/services/affiliate_commission.rs`, `src/services/affiliate_tier.rs` |
| Reseller | `src/app/(main)/reseller/page.tsx`, `src/app/(main)/offer/resell/page.tsx` | `src/api/reseller.rs` |
| Moderation | `src/app/(dashboard)/moderator/*`, `src/components/features/moderator/*` | `src/api/moderation.rs` |
| Admin / War Room | `src/app/(dashboard)/admin/*` | `src/api/admin.rs`, `src/api/admin_war_room.rs` |
| Real-time | Socket.IO client, `src/services/socket.ts` (or equivalent) | `src/ws.rs`, `src/services/notify.rs` |
| Receipts | `DownloadReceiptButton` | `src/services/receipt.rs` (genpdf) |

---

## Appendix B — Design Brief Notes

Things to keep in mind when extending the visual design or adding screens.

1. **Two landing experiences** — `/` is consumer-facing P2P. `/lp` is B2B reseller pitch. Don't conflate them.
2. **Logo is SVG-in-code** (`src/components/brand/QicLogo.tsx`), not from `public/logo.svg`. This is so the lockup is theme-aware. Keep it that way.
3. **Trade page lives under `(main)`**, not `(dashboard)`, but still requires auth via the global provider. Cosmetic difference: no `TestAccountBanner`.
4. **KYC is a tab + modal, not separate routes** — `/settings?tab=verification` for self-service; offer/trade actions trigger an inline modal. Don't add a `/kyc` route.
5. **No mobile bottom tab bar** — mobile uses the right-drawer (`mobile-nav.tsx`) with hamburger trigger. Keep parity.
6. **Modals are bottom-sheet on mobile, centered on desktop** (`DESIGN.md` §7.4). Always.
7. **All amounts pass through `services/quote.rs`** (backend) for trade-time pricing — never multiply on the client. This preserves the ledger invariant.
8. **Locked balance vs available balance** — wallet UI must distinguish them. Withdrawals only from `available_balance`.
9. **Admin actions are double-confirmed** — destructive admin buttons require a typed confirmation phrase or password re-entry.
10. **Tests gate staging** — every new screen needs at least one test in `frontend/e2e/tests/regression/` before deploy (see `.cursor/rules/tests-before-staging.mdc`).

---

*This document is generated from the live codebase as of May 2026. Update it when screens, state machines, or major features change. The companion `DESIGN.md` covers visual tokens; this brief covers layout, behaviour, and end-to-end flow.*
