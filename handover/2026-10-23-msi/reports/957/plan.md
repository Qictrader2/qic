# 957 (906-C) implementation plan: marketplace, offers and public profiles

Planning only. No code, commit or Trello change was made. Written 22 Oct 2026.

Sources read:
- Trello card 957 (ogMKY85s): description, five-item Acceptance Criteria checklist, no comments.
- Sibling and dependency cards by number: 959, 958, 956, 955, 967, 968, 965, 966, 970, 920, 918, 911, 963, 910, 919, 917, 964, 656, 926, 927.
- Design pack v5.2: `~/Downloads/906-design-pack-v5.2/CHANGES.md` and the C PDFs, plus `v5.2-src/C-discovery/` (design-notes.md including "Change log: 21 Oct v5.2", the C0 route table in `project/Main.dc.html`, every `project/*.dc.html`, register.json, tests.md). Also `index/project/tickets.dc.html` and the B0 board's Home notes.
- Code: the read-only checkout `.scratch/906-redesign/base-ro/frontend`, on redesign/906 at 366dca2f (970, 968 and 969 merged). Also the 970 follow-up `origin/970-v52-replicate` (1ad8734d), assumed merged. Root `DESIGN.md`.
- Backend intent (from origin/main): `docs/intended-entity-state-machines.md` §1.2 Offer and §5.4 ResellOffer, `docs/design/merchant-tier.md` (the quote and fee disclosure contract), and `src/api/offers.rs` (sort and visibility).
- The 906 hand-over: `.scratch/906-redesign/HANDOVER.md`.

Build order from the hand-over: 970, then 969 and 968 (all merged), then **957, 956 and 955**, then 959 and 958, then 967 and 966, then 965. This card goes before 959. The offer page therefore has to leave a clean seam for 959's start-trade confirmation.

---

## a. Boards and routes

### Routes (C0 route table)

| Route | Placement (C0) | Completeness | Boards |
|---|---|---|---|
| `/marketplace` | Market, top level, dock on phone | Complete | C1 (390), C2 (768), C3 (1440), C4 (signed out), C5 (phone states), C21a/b/c (large states, rates strip) |
| `/offer/[id]` | Market, task page, no dock | Complete at 390 | C6 (buying), C7 (states), C8 (seller, large), C20b 30-31 (verification), C22a/b (money states, large) |
| `/profile/[userId]` | Market, deep page, no dock | Complete at 390 | C9, C20b 10, 15 |
| `/my-offers` | Market, section pill, dock on phone | Complete at 390 | C10, C11, C20b 11-14 |
| `/offer/create` | Market › My offers, stepped task, no dock | Complete at 390 | C12 (phone), C13, C14 (large), C20a |
| `/offer/resell` | Earnings › Reseller, deep page, no dock | Complete at 390 | C15 (resell sheet from the row menu), C16 (page) |
| `/dashboard/listing` | 308 to `/marketplace` | Complete | C17 |
| `/prices`, `/prices/live`, `/prices/gas` | 308 to `/marketplace` | Complete | C17 |

Cross-cutting boards:
- C18: accessibility, the B4 checks (320, 360, 200%, query parameters, sign-in round trip, the 308s) and B6 (glass).
- C19a/b/c: decisions, product rules, components K1 to K14, tests, the B3 states table and completeness.
- C0: overview.

### Changed in v5.2

- **Individual edits:** C0, C1, C4, C6, C7, C8, C9, C16, C17, C19a/b/c, C22a, C22b.
- **Shared-control colour only:** C5, C12 to C15, C20a/b, C21b. These also map to 967.
- **Unchanged, carried from v5:** C2, C3, C10, C11, C21a, C21c.

The CHANGES.md row titled "C21b ... mkt-states-large-2" covers frames 17 to 22.

### Build-relevant rules on the boards

- **A1 sort:**
  - Buy USDT is lowest price first; Sell USDT is highest price first.
  - The order applies inside each fiat group, with the selected currency first, then the rest alphabetically (M16, confirmed). For a ZAR user that is ZAR, KES, NGN; for an NGN user it is NGN, KES, ZAR.
  - The viewer's own offer stays in its price place, labelled "Your offer", with Manage. It is never pinned.
  - The same sequence holds at 390, 768 and 1440.
  - Count captions are exact: "8 offers · lowest price first", "6 buyers · highest price first", "9 offers · lowest price first · yours included", and for guests "11 offers · Rand first, lowest price first".
- **C-2, URL compatibility:**
  - `?tab=` values do not change. `?tab=buy` keeps listing buy offers under the label "Sell USDT"; `?tab=sell` keeps listing sell offers under "Buy USDT".
  - With no `?tab`, the page opens on Buy USDT. That is today's `?tab=sell` list, so the default tab flips.
  - Sorting desc is requested only for `?tab=buy` with `sort=best-rate`.
- **Tabs and section pills:**
  - The pills read "Buy USDT | Sell USDT | My offers"; guests do not see My offers.
  - At 768 and 1440 the marketplace header has "Create offer", plus "Find a trader" at 1440.
  - The phone marketplace has no Create button.
- **Rates strip:**
  - "1 USDT" with NGN, ZAR and KES, and a clock rather than a counter: "Live · 14:02" (M14).
  - States: stale ("Rates from 13:58, not updating", or "Not live · 13:58" at 768) and one missing ("No rate").
- **Row anatomy (K1):**
  - Trader line "98% completion · 412 trades", price "R 18.20 per USDT" with "1.4% above market" from 1440, and "Limits R 500 – R 20,000".
  - Payment: two method chips then "+2". "30 min to pay" (768) or a Time to pay column (1440).
  - One action (Buy or Sell) and a ⋯ menu.
- **Row ⋯ menu:**
  - Signed in: Resell this offer, Copy link, View profile, Report this offer.
  - Guests get Copy link only. Resell is hidden on your own offers and on resales.
- **Money (A2):**
  - Rows read Trade amount, Escrow fee 1.00%, Held in escrow, Your balance. Worked example: 120.00 USDT holds 121.20 with a 1.20 fee.
  - Max ("Use the most you can") comes from the quote's `sellerEscrowFee`, never a constant.
  - The buyer pays no escrow fee.
- **Start trade (C6 20-29, C7 5, 12, 13, 45, 46; C8 18, 49, 50; C22a/b):**
  - "Start this trade?", Pending, Failed, outcome unknown, Check status and funding failed. The card assigns these to 959 (906-A); see g.
- **CH6:** "Verify your identity to trade this amount ... 250 USDT a day (≈ R 4,480)". The button text carries no "≈". Product rule Open for Christian (final approval).
- **Create offer:**
  - Five steps: Type and price, Limits and payment, Your terms, Who can trade, Check your offer. Then Done, Discard, verification limit, Publishing… and publish failed.
  - No crypto picker. The wallet sidebar is removed.
- **My offers:**
  - The intro line and a segmented filter "All 4 / Active 2 / Paused 1 / Closed 1".
  - A health warning such as "Hidden from buyers. Your 12.40 USDT won't cover…".
  - One action per status (Edit, Resume or none), and a ⋯ menu with Pause, Copy link, See it as buyers do, Version history and Delete.
  - One Edit sheet holds price, limits and documents. Pause has no confirmation and shows an Undo toast for 6 s. Delete asks for confirmation.
  - Empty copy differs per tab.
- **Resell:**
  - The sheet carries "QicTrader's share, 25%" from the server preview. The page groups offers by currency.
  - Guests are sent to `/login?redirect=%2Foffer%2Fresell` and keep `?offerId=`. This is M15, which the board itself says needs Christian's separate approval.
- **Profile:**
  - Reputation block: Trades, Completion rate, Typical reply, Rating. This is the same block as the user's own /profile (D4, M18).
  - ⋯ menu: Copy profile link, Report, Block. The block dialog is followed by a "Blocked by you" state with Unblock (M11, needs BE8 = card 926).
  - Not found reads "This profile isn't available".
- **C17:**
  - Four `permanent: true` rules in `next.config.ts` redirect to /marketplace, keeping the query string.
  - Delete `components/landing/listing.tsx`, `hooks/api/use-active-listings.ts`, `app/(main)/prices/**` and `PricesNav`.
  - `sitemap.ts` drops its three /prices entries. The backend `/prices/fx` API stays.

---

## b. Current code map (redesign/906, 366dca2f, plus 970-v52-replicate)

### /marketplace

`app/(main)/marketplace/page.tsx` (173 lines):
- Renders `MarketplaceOffers` (485).
- Lazy modals: escrow, user search, `CreateDirectOfferModal`.

`hooks/api/use-marketplace-page.ts` (699) holds:
- URL state `MARKETPLACE_URL_PARAMS`: tab, q, crypto, fiat, payment, min, max, sort, type, country, view, page.
- localStorage keys `marketplace-prefs-v1` and `marketplace-fiat-choice-v1`.
- `buildOfferParams` (:430-464): `best-rate` maps to price **asc on both tabs**. That is the A1 bug.
- Client-side filtering after the server page (:570-636), the offers socket, and dead fake BTC/ETH `cryptoPrices` (:255-258, :564-568).

Server side:
- `GET /offers`, `offers-api.ts:411-443`, perPage 20. The server sorts by the effective `price_per_unit` (`src/api/offers.rs:1102-1125`).
- "All currencies" omits `fiatCurrency`, so ZAR, KES and NGN prices are mixed in one list. **There is no grouping.**

Components:
- `features/marketplace/` (4,634 lines in all): `OffersList` (471, virtualised above 15 rows), `FilterControls` (316, three dropdowns including a crypto picker), `SortControls` (70), `MarketplaceFiatReferenceRail` (231, a per-second "updated ago" counter), `DemandSignal*` (empty state).
- `offer-card/*`: Desktop and Mobile cards, CreatorSection, PriceSection, ActionButtons, ActionMenu, PaymentMethods, PremiumBadge, OfferKycTags, skeleton.
- `TradeStats.tsx` is dead. `FiatReferenceBar.tsx` is only used by /prices/live.

Labels:
- Tabs read "Buy Offers (n)" and "Sell Offers (n)", named from the offer maker's side.
- The own offer gets a "Your Offer" badge with a disabled main button; it is not pinned (`OfferCard.tsx:60`).
- Non-USDT leftovers: the crypto dropdown, the "Cryptocurrency" column and the "buy or sell cryptocurrency" text.

Entry points:
- `routes.ts:74-75` `MAIN_NAV_ROUTES` (Create Offer, My Offers) only reaches the legacy header and mobile-nav.
- 968's `CustomerHeader`, Island, dock and Jump have no Create offer or My offers entry. The marketplace page has none either.
- `/dashboard` (Home, Wallet dock) renders `MyOffersTable`. Its Edit pushes to `/offer/{id}/edit`, which does not exist (dead link, 958's file).

### /offer/[id]

- `app/(offers)/offer/[id]/page.tsx` (1,059) and `hooks/api/use-offer-detail-page.ts` (386).
- The quote is `POST /offers/{id}/quote` (`services/quote-api.ts`). It returns `cryptoAmount` and `effectiveRate`, plus `sellerEscrowFee` (minor units, base and effective bps, saving) only for the caller who stakes.

Max:
- `lib/effective-max.ts:17` `STAKER_ESCROW_FEE_BPS = 70`, pinned by `__tests__/hooks/buy-max-fee-headroom.test.ts`. This is card 911.

Fees:
- 656 is already resolved: the line was deleted in PR #199, with the guard test `offer-page-no-platform-fee-Q1RkYiMt.test.tsx`.
- The merchant-tier doc forbids any taker platform-fee line.

Known defects:
- **"Price (Market) 0.00"** (page :473-475, :550, :638; `formatCurrencyByCode(0)`).
- **Signed-out "Login to Trade"** does nothing: the hook returns at :224 before the redirect at :236-242.
- **Unavailable copy:** "seller doesn't have enough funds" is computed from the *viewer's* wallet on buy offers.
- There is no start-trade confirmation: "Confirm & Start Trade" calls `createTrade` directly, and errors show as a toast only.
- There is no separate not-found state.

KYC:
- `use-kyc-gate.ts` and `KYCRequirementModal` (310 lines, shared with create). Limits come as `dailyLimitUsd`.

Other:
- `components/offer/EscrowSection.tsx` (319) is dead.
- The 970 follow-up changes two Back buttons on this page to `variant="secondary"`.

### /offer/create

- `app/(main)/offer/create/page.tsx` (1,170): one long form, not a stepper.
- `components/offer/create/*` (3,706 lines):
  - `TradeParametersSection` (452) and `MarkupPresetControl` (519) are used.
  - `BasicInfoSection` lists BTC/ETH and snaps to USDT.
  - `CreateOfferSidebar` is rendered without props, so it always defaults to BTC on the buy side, which is wrong.
  - `OfferPreviewModal` shows the fee rate from `useMerchantTier`.
  - `EscrowToggle`, `BTCLockInfo` and `LockedOffersManager` are dead.
  - `AddPaymentMethodModal` is shared with settings (956).
  - `TestDataButton` is shared with resell.
- The create page has no fee math. KYC is checked at publish with `freshCheckGate`. Publish goes to `/marketplace`.

### /my-offers

- `app/(main)/my-offers/page.tsx` (320; the 970 follow-up restyles its Create Offer button) and `features/offers/MyOfferCard.tsx` (558).
- Three edit modals (Margin 183, TradeLimit 251, Requirements 121, all restyled by 969) and `OfferDocumentRequirementsControls`.
- Delete uses the browser's `confirm()`. Signed-out shows "Please log in" without a redirect.
- Edit permission is `!offer.parentOfferId`. Ownership comes from `/offers/me` and the backend.

### /offer/resell

- `page.tsx` (453) plus `_components/*` (about 2,000 lines): TraderCard, MarkupControls, EscrowModal, ActiveResellsSidebar.
- The share comes from the server preview `platformShareBps` (GET `/reseller/resell/:id/preview`). No 25% is hard-coded.
- The route is public: there is no redirect, only a "Sign in" link in the sidebar.
- The BTC "Not included" line and per-coin colours are leftovers.

### /profile/[userId]

- `page.tsx` (47; redirects you to /profile on your own id) and `loading.tsx` (89).
- `components/profile/PublicProfileView.tsx` (695):
  - Stats grid "Success Rate". Report and Block, with Block on the legacy Dialog and no Unblock.
  - Staff moderation actions.
- `ProfileReviews` (275) is also used by the own /profile (956).

### /dashboard/listing and /prices*

- Live pages, with no redirects. `next.config.ts:193-204` has only `/terms-and-condition`.
- `routes.ts` lists them in `CUSTOMER_SURFACE_RULES`, `PHONE_TOP_TITLES` and the Market dock prefixes.
- `sitemap.ts:39-41` lists /prices*. `sitemap.test.ts:27` and `e2e/smoke/seo-metadata.test.ts:53` expect /prices.

### Config

- `config/routes.ts:237-241` `CURRENCY_OPTIONS` still lists BTC. **Nothing imports it**, so the change is cosmetic.
- `USDT_ONLY = true` in `crypto.config.ts:235`.
- `app/(offers)/offer/layout.tsx:15` metadata says "BTC, USDT, ETH and SOL".

### Kit available (970, 969, 968 and the 970 follow-up)

- From 970: Button variants, `StatusChip` (28 and 22 px), `Amount` (in, out and held, six decimals, "Unavailable"), `AmountInput`, `CheckboxRow`, `RadioGroup`, `Field`, `EmptyState`, `ErrorState`, `Skeleton`, `Spinner`/`LoadingButton`, `Progress`, `Select`.
- From 969: `DialogShell`, which renders as a sheet below 768, plus `ConfirmDialog` and `toast`. A toast never shows over a dialog, and lasts 6 s.
- From 968: CustomerHeader, Island, PhoneHeader, BottomBar, Jump and `routes.ts` tables.
- **No section-pill or segmented-control component exists**, and no K12 group header.

### Tests touching these routes

- About 150 files under `src/__tests__` and `e2e/`.
- The most load-bearing:
  - `use-marketplace-page-url-state.test.ts` and `use-marketplace-page-tab.test.tsx`
  - `ticket-779-landing-search` (C-2 keeps it green)
  - `buy-max-fee-headroom` (911's)
  - `offer-page-no-platform-fee-Q1RkYiMt`, `escrow-fee-disclosure-608`, `use-offer-detail-kyc-retry-864`
  - `TradeParametersSection.test.tsx` (1,254)
  - `resell-modal-math`
  - the route-table tests (`bottom-bar-routes`, `phone-header`, `customer-surface`, `theme-provider-surface`)
  - the e2e page objects `offer-detail.page.ts` and `create-offer.page.ts`, which use brittle selectors.

---

## c. Gap per board

| Board | Today | Gap to build |
|---|---|---|
| C1/C2/C3 | One list with "Buy/Sell Offers (n)" tabs, dropdown filters, desktop/mobile cards, no Create or My offers on the page | Page header (title, subtitle, Create offer at 768+, Find a trader at 1440); pills Buy USDT, Sell USDT, My offers with the C-2 mapping; K1 rows (phone cards, two columns from 600 px; 768 four-column table; 1440 six-column table with market delta and Time to pay); filter chips; exact count caption; ⋯ menu; one action |
| C4 | Same page for guests | Guest variant: no My offers pill, no Find a trader, ⋯ menu with Copy link only, note above the list, **All currencies grouped** with K12 headers "Rand · ZAR · 8", default currency first then alphabetical |
| C5 (phone states), C21a/b/c | Per-second counter; Live/Degraded labels; generic empty state; no keyboard state | Rates strip clock with stale and missing states (M14); Sell tab caption; Filters and sort sheet (spend amount with helper, Paid with, Currency, Sort by radio, live "Show N offers"); 768 popover; 1440 inline filter card; row menu as a phone action sheet or glass menu; truncation rules; own row "Your offer" chip, stats line and Manage; skeleton after 300 ms; empty "No offers match" with Clear filters, Tell me when one is live and the demand count; error with "Trying again…"; keyboard-open caption |
| C6 | Inline buyer form, no "What happens" section, no aside card | Header "Buy USDT from thabo_k" with price age; trader line; price with market delta; "You pay" AmountInput with limits helper; "You get"; "No escrow fee for you…"; How you pay; terms; the three-step "What happens when you start"; sticky footer (phone and 768) and sticky "Start a trade" card (1440); ⋯ with Copy link and Report. Frames 20 to 29 are 959's (see g) |
| C7 | Generic validation toasts, broken signed-out button, misattributed unavailable state, 0.00 price | Seller money rows (frame 4) from the quote; below min and above max field errors; signed out "Log in to trade" returning with the amount kept; own-offer frame with "Manage in My offers"; CH6 verification dialog; unavailable variants (paused, closed, expired, can't fill) with "See similar offers" keeping tab, fiat and min; loading skeleton and load error; market-following "≈" and "Rand estimate unavailable" (fixes "0.00"); not enough USDT (frame 6) with "Use the most you can" and Deposit. Frames 5, 12, 13, 45 and 46 are 959's |
| C8 | n/a | Seller layout at 768 and 1440 (frames 16, 17, 19, 32). Frames 18, 49 and 50 are 959's |
| C9, C20b 10/15 | Stats grid, legacy block dialog, no Unblock, local error | Header chips "ID verified" and "Gold merchant"; "Online now · joined"; the reputation block (shared with 956); primary "See thabo_k's offers"; secondary "Send a direct offer"; review summary and list; ⋯ menu; block dialog in the 969 shell; "Blocked by you" with Unblock (data from 926); no reviews; not found; load error with Try again; sticky left card at 1440 |
| C10, C11, C20b 11-14 | Two-row cards, wrench menu, three separate modals, `confirm()` | Intro line; segmented filter with counts; card (phone) and table (1440) layouts; health warning row; one action per status; ⋯ menu; **one Edit sheet** with price, limits and documents (margin bounds −50% to +100% sell, −10% to +50% buy, 0.1% steps); pause with Undo toast; Delete confirm; per-tab empty copy; error card; guest login round trip; Resale chip with markup-only edit |
| C12 to C14, C20a | Long single form, BTC defaults, broken sidebar, preview modal | Five-step task with header "Step n of 5" and bar, sticky Back and Continue, step list at 768 and 1440, live preview card at 1440 and preview sheet at 768; fields per step as drawn; inline errors with a "Fix 2 things" summary; review rows with Edit; fee sentence with the tier name; Publishing…; Done (private variant); Discard; CH6 verification limit; publish failed (three variants); close returns to where the user started |
| C15 | `EscrowModal` from a trader card | Resell sheet (phone) or dialog (520 px at 1440, 560 px at 768): three explanation lines, markup with policy bounds, server preview rows (Buyers pay, Difference, QicTrader's share 25%, You earn), per-trade range, errors, Creating…, Done with Resale card. Opened from the marketplace row menu and from /offer/resell |
| C16 | Tabs, trader cards and sidebar; public route | Breadcrumb; summary strip; side tabs with counts; search; currency groups; "At +2%, per trade" column; empty, error and shared-link states; guest redirect (M15) |
| C17 | Live pages | Four 308 rules, deletions, sitemap, tests; own offer in place marked "Your offer"; one sign-post "All your offers are in My offers" after the first screen |
| C18/C19 | n/a | Exact copy, 44 px targets, focus and dialog trap and return, reduced motion, 320 and 200%, query-param survival, 308 query survival. Test verdicts from C19c (delete the btc/sol escrow suites, rewrite `resell-modal-math` to 25%, move the `escrow-fee-disclosure-608` fixture to 100 bps, /prices tests to 308) |

---

## d. Slice plan

Each slice is one reviewable PR into `redesign/906` and each stays at or under about 1,500 product lines. "Product lines" means added or changed non-test lines; deletions are listed apart. The file sets are disjoint. Where two slices meet, the earlier one creates a mount file and the later one only edits its own folder.

Folder convention for new code:
- `components/features/offer-kit/` (shared pieces)
- `components/features/marketplace/` (page parts)
- `components/features/offers/offer-detail/`
- `components/features/offers/my-offers/`
- `components/features/offers/create/`
- `components/features/resell/`
- `components/profile/public/`

| # | Slice | Files it owns | Product lines (est.) | Depends on |
|---|---|---|---|---|
| 1 | **Retire and redirect (C17), USDT-only config.** Four 308 rules (query kept). Delete `dashboard/listing/page.tsx`, `landing/listing.tsx`, `use-active-listings.ts`, `app/(main)/prices/**` (6 files), `features/prices/PricesNav.tsx` and `features/marketplace/FiatReferenceBar.tsx`. Drop the three sitemap entries. Remove BTC from `CURRENCY_OPTIONS`. Remove /prices* and /dashboard/listing from the routes.ts surface, title and Market-prefix tables. Fix the offer layout metadata. Tests: new 308 test per path, sitemap and seo e2e assert /prices absent, feature-validation L321 expects 308, delete ui-fixes L102-118 and the currency-converter tests, update the route-table tests. Delete the e2e btc-escrow-real, sol-escrow-real and trade-flow-no-escrow suites (C19c CH2). | `next.config.ts`, `app/sitemap.ts`, `config/routes.ts` (those tables only), `app/(offers)/offer/layout.tsx`, the deleted files | about 150 added, about 1,950 deleted | none |
| 2 | **Offer kit (K1, K6, K7, K10, K11, K12).** `TraderLine` (initials, name, verified mark, merchant tier chip, "98% completion · 412 trades"; breaks after "completion" below 360), `OfferPrice` ("R 18.20 per USDT", market delta "1.4% above market" or "at market"), `LimitsText`, `PaymentChips` ("+2" with an accessible name), `YourOfferTag`, `OfferRow` (card, table row and own-offer variants, one action plus ⋯ slot), `MoneyRows`, `Notice`, `SegmentedControl` (counts, roving focus), `CurrencyGroupHeader`, `FiatEstimate` (≈, rounded down from limits, "Rand estimate unavailable", M10/W4), `formatFiatPrice` that never turns 0 or null into "0.00". Pure helpers have property tests. | `components/features/offer-kit/**`, `lib/offer-display.ts` | about 850 | 1 is not needed; can start at once |
| 3 | **Ordering and marketplace data (A1, M16, C-2).** `lib/marketplace-order.ts`: tab to visitor side; order per tab (`?tab=buy` with best-rate gives desc, otherwise asc); group order (selected or default currency first, then alphabetical by code); stable partition of unpriced offers to the end (guards the server's Desc-puts-None-first behaviour); caption builder with the exact board strings; sort labels ("Best price", "Most trades", "Best completion", "Newest") over unchanged URL slugs. Hook: default tab becomes `sell` (C-2), keeping `?tab=` meanings and the localStorage keys; "All currencies" grouping by one query per enabled fiat in group order (counts from each `pagination.total`); remove the fake `cryptoPrices`; expose `isOwnOffer`. Tests: per-tab order, group order for ZAR and NGN users, own offer never pinned, the URL round-trip tests kept green, `ticket-779-landing-search` unchanged. | `hooks/api/use-marketplace-page.ts`, `lib/marketplace-order.ts`, marketplace hook tests | about 500 | none (parallel with 2) |
| 4 | **Marketplace list, header and states (C1 to C4 lists, C5 1-4, 7-12, C17 landing, C21a/b loading, empty, error, own offer, C21c strip).** Page header with Create offer (768+) and Find a trader (1440); `MarketPills` (Buy USDT, Sell USDT, My offers; guests without My offers); rates strip with the clock and stale and missing states; list rendering through K1 (phone cards, 768 and 1440 tables, two phone columns from 600 px), group headers, caption, own row with Manage to `/my-offers`; skeleton after 300 ms; "No offers match" with the demand signal; error with "Trying again…"; the C17 sign-post; guest note (C4). Mounts `./filters/MarketplaceFilters` and `./row-menu/OfferRowMenu`, initially thin wrappers over today's FilterControls and ActionMenu. Deletes `offer-card/*`, `OfferCard.tsx` and `TradeStats.tsx`. | `app/(main)/marketplace/page.tsx`, `features/marketplace/{MarketplaceOffers,OffersList,MarketplaceFiatReferenceRail,MarketplaceHeader,MarketPills,…}.tsx`, the two wrapper files (created) | about 1,200 added, about 1,700 deleted | 2, 3 |
| 5 | **Filters, sort and row menu (C5 5, 6, 11; C21b 21, 22; C3 filter card).** Filters and sort sheet (below 768), popover (768), inline card (1440): "I want to spend" or "I want to receive" with helper text, Paid with, Currency (ZAR, KES, NGN, All), Sort by radio, live "Show N offers", Clear all, removable chips, keyboard-open caption. Row ⋯ menu as phone action sheet or glass menu (Resell this offer opens the slice 9 sheet; Copy link; View profile; Report this offer; guest variant). Deletes the crypto picker and `SortControls`. | `features/marketplace/filters/**`, `features/marketplace/row-menu/**`, `FilterControls.tsx`, `SortControls.tsx` | about 950 added, about 400 deleted | 4, 9 |
| 6 | **Offer page, layout, buyer and states (C6 1-3, C7 7-11, 14, 15; C20b 30-31).** Split the 1,059-line page into `offer-detail/*`: header, trader card, price block, `AmountField` (AmountInput, limits helper, errors on blur and Continue), "You get", How you pay, terms, What happens, sticky footer (phone and 768) and aside card (1440), ⋯ menu, version history (owner only, kept). States: signed out (fix the dead button: login with return URL and amount kept), own offer (Manage in My offers), CH6 verification dialog in DialogShell, unavailable variants with See similar offers (tab, fiat and min kept), not found separate from load error, loading, market estimate and "Rand estimate unavailable" (fixes "Price (Market) 0.00"). Start seam: `StartTradeAction` calls today's `createTrade` path unchanged until 959 mounts its confirmation; Continue keeps today's behaviour and copy is recorded in g. Hook fixes: signed-out redirect, unavailability decided from the offer, not the viewer's wallet. | `app/(offers)/offer/[id]/page.tsx`, `hooks/api/use-offer-detail-page.ts`, `features/offers/offer-detail/**` (excluding `money/`) | about 1,150 added, about 800 deleted | 2 |
| 7 | **Offer page money (C7 4, 6; C8 16, 17, 19, 32).** Seller rows from the quote: Trade amount = `cryptoAmount`; Escrow fee x.xx% = `sellerEscrowFee.effectiveEscrowFeeBps` and `effectiveEscrowFeeAmount` (minor units, six decimals); Held in escrow when you start = their sum; Available now and after you start from `useBalance`. Tier sentence only when `merchantTier` is present. Not enough USDT: "This trade needs 121.20 USDT: 120.00 plus the 1.20 escrow fee", "Use the most you can" (value from `effective-max.ts` until 911) and Deposit USDT. Footer "You receive R … Held … USDT". Buyer side: "No escrow fee for you." Wording in g. Keeps the `Q1RkYiMt` guard. | `features/offers/offer-detail/money/**` | about 550 | 6 |
| 8a | **Create offer shell and steps 1-2 (C12 1, 2, 7; C13; C14 step 2).** Stepper task header ("Step n of 5", bar, close as discard), sticky Back and Continue, step list (768 and 1440), preview card (1440) and preview sheet (768). Step 1 (I want to; Paid in; Fixed or Follows market; price with market line; margin presets and bounds). Step 2 (min and max with USDT range; Time to pay 15, 30, 60, Other; payment methods with Add a method through the shared `AddPaymentMethodModal`, unchanged). Discard dialog. A form state hook keeps every step's input. | `app/(main)/offer/create/page.tsx`, `features/offers/create/{CreateOfferTask,steps/Step1*,steps/Step2*,preview/**,use-create-offer-form.ts}` | about 1,100 added | 2 |
| 8b | **Create offer steps 3-5 and outcomes (C12 3-6, 8, 20, 28; C14; C20a).** Your terms (counters, blocked-word errors), Who can trade (visibility, requirements, direct-offer target search, Closes, documents disabled on Buy), Check your offer (rows with Edit, fee sentence from the tier and platform config, terms checkbox), Publishing…, Done (live or private), publish failed (three variants), CH6 verification limit with "Lower the maximum to R 4,480". The publish payload is unchanged (`offersApi.createOffer`). Deletes `components/offer/create/*` except `AddPaymentMethodModal`, and the `TestDataButton` crypto-escrow type. | `features/offers/create/steps/Step{3,4,5}*`, `features/offers/create/outcomes/**`, `components/offer/create/**` (deletions) | about 1,000 added, about 3,300 deleted | 8a |
| 9 | **Resell sheet and page (C15, C16).** `ResellSheet` (server preview `platformShareBps`, refreshed 250 ms after typing stops; markup bounds from `markupPolicies`; errors; Creating…; wallet setup; Done) mounted by /offer/resell and exported for slice 5. Page: breadcrumb, summary, side tabs, search, currency groups (reuses `marketplace-order`), estimate column, empty, error and shared-link states. Guest redirect (M15) in its **own commit** so it can be reverted (see g). Removes the BTC "Not included" line and per-coin colours. | `app/(main)/offer/resell/**`, `features/resell/**` | about 1,200 added, about 1,900 deleted | 2, 3 (for grouping) |
| 10 | **My offers (C10, C11, C20b 11-14).** Page with intro, segmented filter (K11), card and table layouts, health warning (balance against the minimum, with "≈ … USDT with the x.xx% escrow fee" from the platform-fee config), one action per status, ⋯ menu, one Edit sheet replacing the three modals (same PUT `/offers/:id` fields; margin bounds; documents through `offerDocumentRequirementsAvailable`), pause and resume with Undo toast, Delete through ConfirmDialog, per-tab empty states, error card, guest login round trip, Resale chip with markup-only edit. | `app/(main)/my-offers/**`, `features/offers/my-offers/**`, `features/offers/{MyOfferCard,Edit*Modal}.tsx` (replaced or deleted) | about 1,100 added, about 1,100 deleted | 2 |
| 11 | **Public profile (C9, C20b 10, 15).** `PublicProfileView` rebuilt under `components/profile/public/`: header chips, presence line, `ReputationBlock` (exported for 956's own /profile), actions, review summary and list (a new public reviews list; `ProfileReviews` is left to 956), ⋯ menu (Copy profile link, Report through the existing ReportButton flow, Block), block dialog (969 shell), "Blocked by you" with Unblock rendered only when the profile carries a blocked flag (926), no reviews, not found, load error, sticky card at 1440. Staff moderation actions stay as today (boards are silent). | `app/(main)/profile/[userId]/**`, `components/profile/PublicProfileView.tsx`, `components/profile/public/**` | about 900 added, about 600 deleted | 2 |

### Order and parallelism

1. Slices 1, 2 and 3 can run at once.
2. After 2 and 3: slices 4, 6, 8a, 9, 10 and 11 in parallel.
3. Then 5 (needs 4 and 9), 7 (needs 6) and 8b (needs 8a).

Two rules carry across every slice:
- Rebase every slice on redesign/906 after `970-v52-replicate` merges, because slices 6 and 10 replace the two page files that follow-up touches.
- Every slice adds `data-testid`s that the e2e page objects can rely on, and updates the e2e page objects it breaks.

---

## e. Acceptance mapping

| AC (checklist item) | Proved by | Slices | Blocked or partly blocked by |
|---|---|---|---|
| **AC1** Browse, filter, open, create, edit and resell with QA buyer, seller and reseller accounts; loading, empty, populated, validation and error at 390 × 844 and 1440 × 900 | Review-app journeys per role; captures per state | 4, 5, 6, 7, 8a, 8b, 9, 10, 11 | (1) "Start this trade?", outcome unknown and Check status reached from the offer page belong to **959**, with request keys from **920**; until then Continue keeps today's direct start. (2) Funding failed (C7 45, 46; C8 49, 50; C22a 47, 48) needs **910** and 959. (3) Profile "Blocked by you" with Unblock needs **926** (BE8). (4) "Best completion" sort has no server sort key (no card; see g). (5) "3 trades this week" and "Trades this week" have no API field (no card; see g). |
| **AC2** Buy ascending, Sell descending within each currency group, selected currency first; own offer in price position, labelled, never pinned; same order at 390, 768 and 1440 | Unit tests on `marketplace-order` and the hook; seeded offers on the review app (staging backend) at three widths, with the API order compared to the DOM order | 3, 4 | Partly: on the Sell USDT tab (`?tab=buy`, desc) the server's `compare_offers` reverses the whole ordering, so **buy offers with no market price sort first** (buy offers are always visible, `offer_visibility.rs`). Slice 3 moves them last within each page, but a page made only of unpriced offers can still lead. A backend fix is a one-line change in `src/api/offers.rs` with no card today. Seeded QA data must include one unpriced buy offer to prove the guard. |
| **AC3** Limits, fee and held amount on offer detail and create equal the server quote to six decimals; no 0.7% client default (911); fee currency label (656) | Unit tests that the rows render `sellerEscrowFee` minor units unchanged; review-app quote request compared to the DOM; grep proves no 0.007, 1.007 or 70 bps in 957 files | 7, 8b, 10 | (1) **911**: "Use the most you can" still comes from `effective-max.ts` at 70 bps; this card does not touch that file (owner rule). (2) **918**: environments without the `platform_fee_bps` row charge 0.7%; staging and production hold 100, so the review app shows 1.00%. (3) Create page and My offers health line: no quote exists for an unpublished offer, so the fee sentence uses the platform-fee config plus the tier rate, with rounding owned by **911/918**. (4) **656 is already done** (line removed, guard test kept), so this part of AC3 is "no taker platform-fee line appears". (5) Buyer-side seller-fee figures are withheld by the backend (see g). |
| **AC4** Filter and sort URLs, /offer/[id] and /profile/[userId] links and /dashboard/listing keep working; KYC limits and role rules as today; no editing of another customer's offer | URL round-trip tests (kept); C-2 tab mapping; 308 tests with query survival; KYC retry test 864 kept; an IDOR API test (PUT, pause and delete `/offers/:id` as another user returns 403 or 404; template `e2e/tests/security/idor.test.ts`) | 1, 3, 6, 10 | Not backend-blocked. Note: the no-`?tab` default flips to Buy USDT (C-2), so a bare `/marketplace` link now opens today's `?tab=sell` list. That is intended by the board and must be stated in QA. CH6 copy is a product rule still open for Christian. |
| **AC5** Only USDT shown; current, proposed and implemented captures for every changed page and state; WCAG 2.2 AA, 320 px, 200% zoom, long names, large lists | Grep for BTC, ETH and SOL in the rebuilt files; register entries with three captures each (965 format); axe plus keyboard on each route | all | Not backend-blocked. The CTA contrast (2.77:1) is the one accepted exception (kept by Christian). Captures depend on 965's register format if it lands first; otherwise follow 965's spec by hand. |

QA runs on the review app https://qictrader-fe-pr-303.herokuapp.com (staging backend), after each merge into redesign/906.

---

## f. Risks

### Fees

- Never derive a fee from bps on the offer page. Use `sellerEscrowFee.effectiveEscrowFeeAmount` (minor units, as `escrows.fee_amount`) and `cryptoAmount`, and show the sum with six decimals.
- `quote.feeAmount` is the taker's fiat fee and must never be shown (merchant-tier doc).
- `sellerEscrowFee` is absent for non-stakers, so the absence is the role gate. Do not fill it with a default.
- Max at 70 bps can still fill an amount the backend refuses until 911 ships (card 911's R 1,511.67 example). The not-enough-USDT and refusal copy must cover that case.
- Keep `offer-page-no-platform-fee-Q1RkYiMt.test.tsx` and `escrow-fee-disclosure-608`. C19c moves the latter's fixture to 100 bps.
- The create page and the My offers health line read the fee rate from `/config/platform-fees` and `useMerchantTier`. They must say "≈" where rounding is not the server's.

### Limits

- Fiat min and max are offer fields; the list's `capped_max_amount` reflects the staker's balance.
- Resells inherit the parent's fiat limits unchanged (intent §5.4.1). Do not scale them in the resell UI.
- Limit-derived estimates round down (M10). Prices in the offer's own currency are exact.

### KYC

- Keep `checkKycGate` and `freshCheckGate` and the "checking limits" wait. Keep test 864 (a retry creates no trade).
- The limit arrives as `dailyLimitUsd` and the backend caps in USDT 1:1, so "250 USDT a day" is the same number.
- CH6 copy is a product rule open for Christian. Build it as drawn, in its own component, so a copy change is local.

### Roles

- Own offer: `offer.userId === user.uid`.
- A resale offer (`parentOfferId`) gets a "Resale" chip, markup-only edit and no Resell.
- The backend already hides resells of the viewer's own offers.
- Guests: no My offers pill, no Resell, Copy link only. Login round trips keep the query.
- Moderator and admin actions on profiles stay as today; no board draws them. Record this, do not remove them.
- Editing another customer's offer relies on the backend PUT, pause and delete checks. Slice 10 adds the IDOR test.

### Sort correctness

- Per-tab direction (desc only for `?tab=buy` and best-rate).
- The server's Desc places unpriced offers first (see e).
- Client-side payment and search filters run after server pagination, so captions must count what is shown and the page total must not double count.
- "All currencies" grouping needs N queries, one per enabled fiat. Watch the load (enaWsB18 performance tests) and prefetch per tab.
- Socket offer updates must re-run the same comparator. The virtualised list must keep DOM order equal to data order. The same data source feeds every breakpoint, which is the C17 and AC2 three-width rule.

### URL and storage

- The `?tab` default flips (C-2).
- `marketplace-prefs-v1` may restore an old tab, which is still a valid meaning.
- `NavLinks` replays prefs onto /marketplace.
- `LandingSearch` writes `max`, which still has no on-screen control. The boards' "I want to spend" maps to `?min`; keep `max` parsing.

### Scale of test churn

- About 150 test files touch these routes. `TradeParametersSection.test.tsx` (1,254 lines) and the create-offer e2e page object (1,128) need rewriting with slice 8, and the offer-detail e2e page object with slice 6.

### Overlap with 959

- Slices 6 and 7 must not build the start confirmation. Keep the `StartTradeAction` seam to one call site so 959 replaces one component.

---

## g. Card-vs-board drift (for the coordinator)

1. **656 is stale.** The card's Problem and AC3 say the offer page renders a fiat fee with a crypto label. 656 is in Sprints Completed: PR #199 deleted the taker platform-fee line, and the merchant-tier design doc forbids it. No group C board shows it. AC3 should read "no taker platform-fee line; the escrow fee shows in USDT".
2. **"Once JP confirms the rate" is stale.** The boards record 1.00% as decided (F1, 16 Oct) and A2 as Confirmed (Christian, 21 Oct). Only 911 (Max) and 918 (code default) remain.
3. **Entry points.**
   - The card says Create offer and My offers are reached "from Marketplace and Home". No board draws them on Home: B0 moves the Home "My offers summary" to Market › My offers, and the dock has no Home.
   - Boards draw Create offer in the marketplace header (768 and 1440), on My offers and in its empty state, plus the My offers pill and the Manage links.
   - The Problem line "sit in the header" is also stale: 968's customer header has neither entry.
4. **/prices retirement is missing from the card.** C17 retires /prices, /prices/live and /prices/gas with a 308, but the card names only /dashboard/listing. The fix sketch says "rebuild … dashboard/listing", while the board says 308 and delete.
5. **Start-trade ownership.**
   - The card says 906-A (959) owns the start-trade confirmation and funding outcome. The group C boards draw those frames on the offer page (C6 20-29, C7 5, 12, 13, 45, 46; C8 18, 49, 50; C22a/b).
   - The index board's 957 criteria include "An unfunded start is refused with the approved message".
   - Plan: 957 builds the pre-start states (not enough USDT, limits, verification); 959 builds the confirmation, outcome unknown, Check status and funding failed. AC1 on 957 cannot cover those frames until 959 lands. Because 957 ships first, Continue keeps today's direct start in between.
6. **Sort label "Best completion".** C5 frame 5 lists Best price, Most trades, Best completion, Newest. The server sorts by `users.reputation`, not completion rate (`repo/offer.rs:214`), so "Best completion" cannot be honoured. Options: a backend sort key (no card) or keep "Best reputation". Needs a decision; the board cannot be built exactly on today's API.
7. **"Trades this week" has no data.** The own-offer row ("Active · 3 trades this week"), the own offer page ("Trades this week") and the My offers card ("3 trades this week · 41 in total") need a weekly count that `OfferResponse` does not carry (only `totalTrades` and `completedTrades`). This is impossible on today's API and no backend card exists.
8. **Buyer-side seller fee.**
   - C6 frame 1 ("thabo_k … pays the 1.00% fee") and frames 20-22 ("Held in escrow from thabo_k 121.20 USDT", "thabo_k's 1.20 fee") show the seller's fee to the buyer.
   - The backend returns `sellerEscrowFee` only to the staker, so that one merchant's tier is never disclosed (merchant-tier doc, product owner 16 Sep). A Gold seller's fee would be 0.84, not 1.20.
   - This is board-contradicted by backend intent. The buyer copy needs either a tier-neutral wording or a new backend field (no card). Frames 20-22 are 959's, but frame 1 is 957's.
9. **M15 guest redirect on /offer/resell.** It is drawn on C16 frame 7, but C19a says making the route sign-in-first is an authentication-policy change needing Christian's separate approval. Plan: build it in its own revertible commit on redesign/906 (which reaches production only after Christian's final approval) and list it in the approval pack.
10. **M11 Unblock.** The board draws it, but it needs 926's blocked flag. Until then the profile looks unchanged, as the board itself says.
11. **WCAG 2.2 AA in AC5** must carry the CTA exception (white on --action 2.77:1), kept as approved by Christian on 21 Oct.
12. **One Edit sheet.** C11 frame 5 draws one sheet (price, limits, documents). Today there are three modals that 969 just restyled. The board wins, so slice 10 replaces them; tell 969's QA.
13. **Margin bounds.** C11 and C13 say "a backend ticket enforces the same bounds on create and update"; no card number is given and the backend checks none today. 957 enforces them in the form only.
14. **Index estimate.** The index board estimates 957 at 56 h. See i: the honest figure is higher.
15. **Group pagination is not drawn.** C4 and C16 draw currency groups but no pagination or "show more" inside a group. The page has 20-row pagination today. Proposed least-invention rule: in grouped mode show each group's first page with its total in the header, and make the header a link to that currency (`?fiat=`). Needs a nod.
16. **Card file list.** The card names `features/offers` and `offer/create`, but the create page's components live in `components/offer/create/*`. Slice 8 moves them to `features/offers/create/` and deletes the dead BTC ones.

---

## h. Files that other page-group cards also touch

| File or area | 957 slice | Other card | What to agree |
|---|---|---|---|
| `app/(offers)/offer/[id]/page.tsx`, `hooks/api/use-offer-detail-page.ts` | 6 | **959** (start confirmation, outcome unknown, funding failed), **967** (colour sweep skips rebuilt files), 970 follow-up (two Button variants) | 957 lands first and leaves one `StartTradeAction` seam; 959 replaces only that component and its call in the hook |
| `components/features/merchant-tier/EscrowFeeDisclosure.tsx` | 7 (reads only) | **959** (`TradeSidebar` uses it) | 957 renders its own money rows; it does not change this component |
| `components/features/escrow/KYCRequirementModal.tsx`, `hooks/api/use-kyc-gate.ts` | 6, 8b (CH6 dialog wraps or replaces the modal content) | **959** (trade context) | Keep the `context` prop and gate API; put the new dialog beside it rather than editing the shared modal |
| `app/(dashboard)/dashboard/page.tsx`, `features/dashboard/MyOffersTable.tsx` (Home) | none | **958** | B0 moves the My offers summary to Market › My offers; 958 removes or keeps it. `MyOffersTable`'s Edit link to the non-existent `/offer/{id}/edit` should go to `/my-offers` |
| `app/(main)/reseller/**`, Earnings › Reseller "Resell an offer" | 9 exports `ResellSheet` | **958** | 958 links to `/offer/resell` or mounts the sheet; it does not rebuild it |
| `components/profile/ProfileReviews.tsx`, `ProfileHeader.tsx` (links to /offer/create and /offer/resell), the reputation block on own /profile, `features/settings/BlockedUsersSection.tsx` (unblock in Settings), `/report?user=` page | 11 (exports `ReputationBlock`; uses ReportButton) | **956** | One `ReputationBlock` (D4, M18), built by 957 and consumed by 956. `ProfileReviews` stays 956's |
| `components/offer/create/AddPaymentMethodModal.tsx` | 8a (uses unchanged) | **956** (settings payment tab) | 956 owns its restyle; 957 keeps it where it is |
| `config/routes.ts` | 1 (CURRENCY_OPTIONS; /prices* and /dashboard/listing rows); 8a may change `PHONE_SUB_HEADERS` so close returns to the origin | **968** follow-ups, **958** (/earnings), **956** (Account hub), **955** (public routes) | Small separate hunks; rebase before merge |
| `next.config.ts` `redirects()`, `app/sitemap.ts` | 1 | **958** (says it keeps redirect pages; confirm it adds no rules), **955** | Append-only rule list |
| `features/marketplace/**`, `features/offers/*`, `components/offer/create/*`, `profile/FlipDeals.tsx`, `profile/ActiveTrades.tsx` | 4, 5, 8, 10 | **967** (cleans what page cards leave) | 957 leaves zero raw hex in rebuilt files; 967 skips them. `FlipDeals` and `ActiveTrades` are not 957's |
| `lib/fiat-display.ts`, `features/marketplace/fiat-reference-display.ts` | 4 (strip) | **958** (wallet fiat display), admin pages | Do not change exported signatures |
| `/login?redirect=` handling | 9 (M15), 6 and 10 (login round trips) | **955** | Uses the 935 behaviour already merged; no change in 955's files |
| Register and captures | all | **965** | Use 965's register format for AC5 captures |

---

## i. Honest size estimate

- **Product code:**
  - About 10,650 lines added across 12 PRs (slices 1, 2, 3, 4, 5, 6, 7, 8a, 8b, 9, 10, 11). The largest is 4 at about 1,200, and none is over about 1,200.
  - About 11,750 lines deleted. Dead and replaced code is about half the current surface: offer-card, the create sidebar, BTC lock, EscrowSection, the resell components, the prices pages and the listing page.
- **Tests:** about 4,500 to 5,500 lines written or rewritten. The biggest pieces are the create-offer and offer-detail e2e page objects, `TradeParametersSection.test.tsx` and the marketplace hook tests. Add the new ordering, 308, IDOR and money-row tests.
- **Effort:** about 80 to 100 hours of build, review and QA, against the index board's 56 h. The extra comes from four things:
  1. grouped "All currencies" fetching
  2. the five-step create rebuild
  3. the one-sheet My offers editor
  4. about 150 affected test files
- **Calendar:** with three parallel builders after slices 2 and 3, about 4 to 5 working days to merge, then QA on the review app.
- **Uncertainty:** the outcome of the drift items in g, especially 5, 6, 7, 8 and 15. Each answered "build as drawn with backend support" adds a backend card outside 957.
