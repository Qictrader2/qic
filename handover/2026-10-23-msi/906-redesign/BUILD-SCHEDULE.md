# 906 page groups: build schedule (959, 958, 957, 956, 955)

Written 22 Oct 2026 from the five plans (`.scratch/{959,958,957,956,955}/reports/plan.md`), `HANDOVER.md`, and read-only source at `origin/redesign/906` 179f342f (umbrella checkout). Nothing was built, committed or written to Trello.

Ground rules used:
- Decision rule (owner, 22 Oct): nothing goes to Marcello. Card vs board: board wins. Board vs board: the board for that exact screen or state wins, else the majority of frames. Backend: build against backend origin/main as it is today; leave the AC unticked naming the gap and report the gap.
- Card priority 959 > 958 > 957 > 956 > 955. One PR per slice, squash-merged into `redesign/906`. No two slices in one wave share a file.
- Locked until they merge (checked in the two worktrees):
  - **970-v52-replicate** (1ad8734d plus uncommitted fixes): `app/(dashboard)/notifications/page.tsx`, `app/(main)/my-offers/page.tsx`, `app/(offers)/offer/[id]/page.tsx`, `app/globals.css`, `components/ui/{amount-input,amount,button,checkbox,input,loading-button,radio-group,select,spinner,status-chip}.tsx`, `__tests__/design/*`.
  - **968-staff-menu-d0** (uncommitted on 9bbd3d88): `config/routes.ts`, `components/navigation/{AccountHub,Capsule,CustomerHeader,Island,JumpPalette,PhoneHeader,StaffMenu(new),index,jump-search}.ts(x)`, `components/support/return-focus.ts`, `__tests__/navigation/*` (account-hub, jump, main-header-customer, staff-entries).
- Wave 1 needs neither follow-up merged. From wave 2, slices import the new 970 pieces (AmountInput, CheckboxRow, held Amount, 22px chip, spinner ring), so **970-v52-replicate must merge before any wave 2 PR merges**. Wave 1 slices that end up importing those pieces (likely 959-S2, 957-S2) build on the follow-up head and merge after it.
- 955-S1 is dropped: 956-S1 builds the shared password module (see table 1). 957-S9 is split so M15 gets its own PR, **957-S9m** (decision 2). Result: 53 PRs in 10 waves.

---

## 1. Shared files: who owns them

"Owner" builds or first edits the file. "Appends after" slices touch only their own hunk and rebase after the earlier slice merges. "Consumers" import and never edit.

| File or piece | Owner | Appends after the owner merges (in order) | Consumers (import only) | Resolution notes |
|---|---|---|---|---|
| `config/routes.ts` | **968-staff-menu-d0** (lock; also adds `/earnings` to `CUSTOMER_SURFACE_RULES` for 958-S9, so 958-S9 merges after it) | 959-S3 (w3: `/trade/[id]` back link from a whitelisted `back`, drop `/trade-history/[id]`) → 957-S1 (w4: `/prices*`, `/dashboard/listing` rows, `CURRENCY_OPTIONS`) → 958-S8 (w5: `/dashboard`, `/dashboard/resell`, `/wallet/manage/[currency]` rows, `AUTH_NAV_ROUTES` `/dashboard` href) → 957-S8a (w6, only if close-to-origin needs `PHONE_SUB_HEADERS`) → 955-S3 (w7: six auth routes in `CUSTOMER_SURFACE_RULES`) → 956-S4a (w8: `SETTINGS_TAB_TITLES.preferences`, hub row and Jump to `?tab=preferences`) | everyone | 958-S9 needs no routes.ts edit (the `/earnings` dock, title and Jump rows already exist). Route-table tests (`bottom-bar-routes`, `phone-header`, `customer-surface`, `theme-provider-surface`) follow the same order |
| `next.config.ts` `redirects()` | **959-S8** (w1: `/trade-history/:id` → `/trade/:id`) | 957-S1 (w4: `/dashboard/listing`, `/prices`, `/prices/live`, `/prices/gas`) → 958-S8 (w5: `/dashboard`, `/wallet/manage`, `/wallet/manage/:currency`, `/wallet/deposit/:c`, `/wallet/withdraw/:c`, `/dashboard/resell`) | | 959-S8 lays the array out as one commented block per card so later appends don't conflict. All `permanent: true` (308), query kept |
| `app/sitemap.ts` | **957-S1** (drop the three `/prices*` entries) | none | | 958 adds no sitemap entry (N7); 955 none |
| `__tests__/.../sitemap.test.ts`, `e2e/smoke/seo-metadata.test.ts` | **958-S9** (w1: `/earnings` absent) | 957-S1 (w4: `/prices` absent) | | |
| `app/robots.ts`, `public/robots.txt` | **958-S9** | none | | `app/robots.ts` already disallows `/earnings`; the served file is `public/robots.txt` (decision 13) |
| `lib/auth/login-redirect.ts`, `components/providers/auth-provider.tsx` | **955-S2** (default `/wallet`, `authSwitchHref`, /signup honours `redirect`) | none | 957 (login round trips), 956-S8 (`/login?redirect=/report`) | 958-S8 **drops** its constant edit. Between w5 and w6 a sign-in lands on `/dashboard` and the 958-S8 308 takes it to `/wallet`, so nothing breaks. 955-S2 owns `login-redirect.test.ts` and `ticket-935-login-redirect` |
| `lib/api/http-client.ts`, `lib/auth/session-idle.ts`, `services/auth-api.ts`, `hooks/auth/use-auth-actions.ts` | **955-S2** | none | 958-S4, S5 (`retryAfterSeconds` for step-up limits), 959 | |
| `hooks/use-online-status.ts` (new) | **956-S2** (w6) | none | 955-S4, S5, S6c, 956-S7b, S8 | Both 955-S2 and 956-S2 planned to create it; 955-S2 drops it |
| `components/ui/offline-banner.tsx` (new) | **956-S2** | none | 955 pages (E1d N, E3d H, E4d I) | |
| `components/ui/banner.tsx` (new, four tones) | **955-S3** (w7) | none | 956 later slices if they need a tone banner | 958 builds `HarcStatusCard` (its own, w2) and does not wait for it. An older `ui/rate-limit-banner.tsx` exists; 955-S3 checks whether it can retire it |
| `lib/state-machine/status-display.ts` | **959-S1** (w1, rewrite, `getTradeStatusDisplay` signature kept) | none | 957-S4 (`OffersList`), 958-S6 (Your turn card), 958-S7 (trade rows), 958-S10 (reseller rows) | |
| `lib/state-machine/migration-map.ts` (`normalizeTradeStatus`), `TradeState` | **959-S1** (added 22 Oct: pass backend `completed` and unknown statuses through; `services/trades-api.ts` unchanged) | none | every trade reader | Before the fix every finished trade read "USDT released, Step 4 of 4" and an unknown status read Funded with I've paid and Cancel |
| `features/affiliate/tier-config.ts` fallback table | **958-S11** (add "Priority support" to the fallback rows, found by 958-S12) | none | 958-S12 pages | |
| `components/features/offer-kit/OfferRow.tsx` | 957-S2 (merged) | **957-S4** adds optional title and requirements props (C5 frame 7) | 957-S4, S10 | |
| `e2e/pages/create-offer.page.ts` `findUserOfferOnMarketplace` (L866, L893) | **957-S8b** | | | Still clicks role=tab and looks for "Your Offer"; stale after 957-S4's list; S8b updates it |
| `e2e/tests/report-user.test.ts` | **956-S8** | | | Expects a visible Report button; since 957-S6 Report sits in the offer page's more menu |
| Chat composer selectors in `e2e/pages/trade.page.ts` (L42, L226), `ticket-SNtRWcvP-chat-socket-blocked.test.ts`, `trade-chat.test.ts` | 959-S9 owns the files | **959-S6** updates only the composer selector to the board's "Message {name}" (aria-label) | | Coordinator ruling 22 Oct |
| `WithdrawModal` amount while a request key is held | **958-S4** | | | Freeze the sent amount while a key is held (found in the main sync, #330) |
| Held funds sheet: Check status path for a kept unlock key whose offer no longer shows Unlock | **958-S6** | | | From 958-S2's review |
| `lib/api/http-client.ts` 429 handling | **955-S2** | | 958-S5 step-up code note | Put the Retry-After header into the thrown error as `data.retryAfterSeconds` (958-S5 already reads it there); http-client also replaces every 429 message |
| `MyOfferCard.tsx` row menu over the cookie banner | **957-S10** | | | Use the shared dropdown-menu (which now hides the banner, #343) or register the layer; found by 968-qafix2 |
| Marketplace offer row menu at 390 opens flush to the left screen edge | **957-S5** (filters and row menu) | | | From the 968 second re-test |
| `lib/affiliate-payout.ts` and the ticket-471 test (unused since 958-S11) | **958-S8** (delete) | | | |
| Activity wording "Affiliate commission" | **958-S7** | | | Board copy no mapper produces yet |
| `lib/effective-max.ts`, `__tests__/hooks/buy-max-fee-headroom.test.ts` | **nobody in these five cards** (card 911) | | 957-S7 reads it for "Use the most you can" | |
| Password module: `lib/password-rules.ts`, `components/ui/password-requirements.tsx`, `lib/constants.ts` (`PASSWORD_MIN_LENGTH` 12), `components/ui/password-input.tsx` (customer look: 44px toggle, `aria-pressed`, hide on submit) | **956-S1** (w6), built to **955-S1's spec** (E's component: debounced count, cross after submit, fold at 5 of 5, `.is-all` success colour and icon, sign-up's symbol class) | none | 956-S5a, 955-S5, 955-S6c | 955-S1 is removed. One path: `src/lib/password-rules.ts`. The D-vs-E look difference goes to Marcello (decision 15) |
| Money outcome screens (outcome unknown, Check status result, request reference) | **959-S2** (w1): generic pieces in `components/money/{OutcomeUnknown,CheckStatusResult,RequestReference}.tsx` and `lib/request-reference.ts` (prefix argument covers T/M/R/X/D and W/S/P/L); trade-specific bodies stay in `components/features/trade/commit/*` | none | 958-S1 (w2) and every 958 flow; 957-S7 (start outcomes) | Resolves 958's "unless 959 lands first". 958-S1 keeps its pure libs, `HarcStatusCard`, `SecurityCheckStep`, `CodeBoxes`, `NetworkRadioRows` |
| `StartOutcomeBody`, `lib/trade/start-errors.ts`, `hooks/use-trade-commit.ts` | **959-S2** | none | 957-S7 mounts them (decision 1) | |
| New read calls for Check status (`GET /wallet/transfers`, `GET /wallet/locks/{offer_id}`, `GET /affiliate/payouts`) in `services/wallet-api.ts`, `services/affiliate-api.ts`, new `hooks/api/use-money-check-status.ts` | **958-S1** | none | 958-S2, S5, S11 | Keeps S2, S4 and S5 (same wave) out of the service files. Only 958-S4 edits `hooks/api/use-wallet.ts` (`useWithdraw` key binding) |
| `components/features/wallet/NetworkDropdown.tsx`, `StepUpPrompt.tsx` (delete) | **958-S6** (w4) | | | Moved from 958-S4: DepositModal (S3) and TransferModal (S5) still import them during w3 |
| `components/ui/section-pills.tsx` (new, generic qt `.pill`, `aria-current`, `.hit44`) | **958-S9** (w1) | none | 959-S7 (Active, History), 957-S4 (`MarketPills`), 956-S2 (`AccountPageHeader`) | 956-S2 drops its own `SectionPills`; 958-S9's `EarningsSectionPills` is a thin config over it |
| `components/features/offer-kit/SegmentedControl.tsx` (K11, counts, roving focus) | **957-S2** (w1) | none | 957-S10, 956-S6 (All, Unread) | |
| Toast Undo action: `components/ui/toast.tsx`, `store/ui-store.ts` (`action: {label, onClick}`, 6s) | **957-S10** (w5, pause with Undo) | none | 956-S6 (delete with Undo) | Moved from 956-S2 because 957-S10 needs it two waves earlier |
| `components/ui/error-summary.tsx` (role=alert, focus, links move focus) | **957-S8b** (w7, "Fix 2 things") | none | 956-S4a (save bar validation summary), 955-S6b/S6c (H, H2) | One component, built to E2d H's spec |
| `components/profile/ReputationBlock.tsx` (D4, M18: Trades, Completion rate, Typical reply, Rating, honest empty words) | **957-S11** (w6) | none | 956-S3 (w7) | Both plans built it. Neutral path, not under `public/` |
| `app/(main)/profile/[userId]/page.tsx` | **957-S11** (rebuild, includes `?view=public` skipping the own-id redirect) | none | 956-S3 links to it | 956-S3 no longer edits it |
| `app/(offers)/offer/[id]/page.tsx`, `hooks/api/use-offer-detail-page.ts` | **957-S6** (w4, after 970-v52-replicate merges) | 957-S7 (w5: `StartTradeAction` body and the `createTrade` call site only) | 959-S2 pieces | 959 slices never edit these files |
| `components/support/support-chat-state.ts`, `SupportChatPanel.tsx` (trade context for "Get help", D1c note 11) | **959-S3** (w3, after 968-staff-menu-d0 merges) | none | 958 "Get help with this withdrawal" links `/report?withdrawal=` instead | `return-focus.ts` stays 968's |
| `components/navigation/PhoneHeader.tsx` (right-hand actions slot) | **959-S3** (after 968-staff-menu-d0 merges) | none | | No other slice edits it |
| `app/(main)/report/**`, `lib/report-query.ts` | **956-S8** (w9: `?trade=`, `?user=`, and `?withdrawal=` per decision 12) | none | 959-S3 and 958 link to it | |
| `lib/notification-route.ts`, `NotificationsGlass.tsx`, `NotificationsDropdown.tsx`, `notifications/page.tsx` | **956-S6** (w10, after 970-v52-replicate merges) | none | | Fixes the unknown-type crash (#130) |
| Dashboard tables `features/dashboard/{MyOffersTable,ActiveTradesTable,WalletOverview,…}.tsx`, `skeletons/{Dashboard,Table}Skeleton.tsx`, `app/(dashboard)/dashboard/page.tsx` | **958-S8** (w5, delete; only the dashboard page imports the two tables) | none | | 959 and 957 asked for label or link fixes on these tables; deleting them settles both |
| `components/ui/escrow-badge.tsx`, `marketplace/offer-card/*` | **957-S4** (delete with offer-card) | none | | |
| `features/merchant-tier/EscrowFeeDisclosure.tsx` and test `escrow-fee-disclosure-608` | component: nobody edits; test fixture → 100 bps: **957-S7** | | 959-S4 if `TradeSummary` uses it | 959-S3 deletes `TradeSidebar`, so the component may lose its last importer; delete only if unimported after w5 |
| `components/offer/create/AddPaymentMethodModal.tsx` | nobody restyles in these cards | | 957-S8a, 956-S5b | |
| `lib/seo/metadata.ts` (`interactiveWidget: "resizes-content"`) | **956-S2** | | | Global keyboard change; regression look on the 959 room composer (decision 11) |
| `__tests__/ui-fixes.test.ts` | **957-S1** (w4, delete L102-118) | 955-S6a (w6, helper path) → 956-S3 (w7, L195-198) → 955-S7 (w10, Google L78-84 kept) | | Line-disjoint hunks, one per wave |
| e2e page objects | `trade.page.ts`: **959-S9**; `marketplace.page.ts`: **957-S4**; `offer-detail.page.ts`: **957-S6**; `create-offer.page.ts`: **957-S8b**; `login.page.ts`: **955-S7** | | | 959-S9 waits for 957-S6 (C's page objects first, tests.md order) |
| e2e `btc-escrow-real`, `sol-escrow-real`, `trade-flow-no-escrow` (delete, USDT only) | **957-S1** | | | 959-S9 does not touch them |
| e2e `affiliate-referral-flow.test.ts` | **955-S7** (single owner across B, C, D, E) | | | Expect it red between 958-S8 and 955-S7; e2e is not in the merge gate |
| e2e `ticket-405`, `ticket-nav-affiliate-active` (`?tab=preferences`) | **956-S4a** | | | |

---

## 2. Waves

Each wave's slices touch no common file. A slice can start once its dependencies have merged. "968" = after 968-staff-menu-d0 merges; "970" = after 970-v52-replicate merges.

| Wave | Slices (card priority order) | Locks and merge notes |
|---|---|---|
| **1** | 959-S1 trade model · 959-S2 money commit core (+ shared outcome pieces) · 959-S8 legacy merge and 308 · 958-S9 earnings overview (+ `section-pills`) · 957-S2 offer kit · 957-S3 marketplace ordering | None locked. 959-S2 and 957-S2 merge after 970 if they import its new pieces. 968 and 970 should merge during this wave |
| **2** | 959-S4 status cards · 959-S5 commit steps · 959-S6 chat, documents, rating · 959-S7 trades list · 958-S1 money primitives · 958-S10 reseller | All 970. 958-S1 needs 959-S2 merged |
| **3** | 959-S3 room shell **(968)** · 958-S2 held funds · 958-S3 deposit · 958-S4 withdraw · 958-S5 send and gas · 958-S11 affiliate and payout | 959-S3 is the first routes.ts appender after 968 |
| **4** | 958-S6 wallet page · 958-S12 tiers and loyalty · 957-S1 retire and redirect **(968)** · 957-S4 marketplace list · 957-S6 offer page **(970)** · 957-S9 resell sheet and page | 958-S6 also deletes NetworkDropdown and StepUpPrompt, and takes the 908 fix if decision 3 is yes |
| **5** | 959-S9 e2e and evidence · 958-S7 activity and detail · 958-S8 redirects and landing **(968)** · 957-S5 filters and row menu · 957-S7 offer money and start · 957-S10 my offers **(970)** | Merge 958-S8 before 958-S7 if S7 changes `TransactionHistory` props (the manage page that imports it is deleted by S8) |
| **6** | 957-S8a create shell and steps 1-2 (**968** only if it edits routes.ts) · 957-S11 public profile · 956-S1 password module · 956-S2 account kit · 955-S2 auth plumbing · 955-S6a sign-up helpers | 955-S2 drops `use-online-status` |
| **7** | 957-S8b create steps 3-5 (+ `error-summary`) · 957-S9m M15 guest redirect · 956-S3 /profile body (**968** only if the 320px fix lands in AccountHub) · 956-S7a support list and new ticket · 955-S3 auth shell and header **(968)** | Merge 955-S3 with 955-S4 and S5 close together (new header over old bodies in between) |
| **8** | 956-S4a settings shell **(968)** · 956-S7b ticket thread · 955-S4 /login · 955-S5 forgot and reset · 955-S6b sign-up components | |
| **9** | 956-S4b preferences and alerts · 956-S5a security · 956-S5b payment methods and limits · 956-S8 /report · 955-S6c /signup page | |
| **10** | 956-S6 notifications and bell **(970)** · 955-S7 auth tests and evidence | |

10 waves, 53 PRs. Per card: 959 finishes in w5, 958 in w5, 957 in w7, 956 in w10, 955 in w10. 956 and 955 start late because the six slots in waves 1 to 5 go to higher-priority cards. If you want 956 and 955 done sooner, start 956-S1/S2 and 955-S2/S6a as a seventh and eighth lane in wave 1 (they touch no shared file). That cuts two waves and breaks the 6-per-wave aim, not the priority order.

Review strictness (from the plans): two sign-offs for 959-S2, S5 and S3; 958-S1, S2, S4, S5 and S11; 957-S7; 955-S2 and S4 (auth handlers).

---

## 3. Decisions to make or relay

| # | Decision | Recommended answer | Who |
|---|---|---|---|
| 1 | Who builds "Start this trade?" and its outcomes on `/offer/[id]`? 959 says 957 mounts them; 957 says 959 owns them and keeps today's direct start meanwhile. No slice in either plan owns the wiring | **957-S7** replaces `StartTradeAction`'s body with 959-S2's `useTradeCommit` and `StartOutcomeBody` (confirm, pending, failed, couldn't confirm, not enough USDT, funding failed copy). 959-S2 merges in w1 and 957-S7 builds in w5, so the interim direct start never ships. Never offer start retry (920) | coordinator |
| 2 | 957 M15 (`/offer/resell` sign-in first) "in its own commit for Christian" | Squash-merge erases an inner commit, so make it **its own PR, 957-S9m**, merged after 957-S9 and listed in Christian's approval pack. Revert = one commit on redesign/906 | coordinator (Christian already approved v5.2) |
| 3 | 958-S6 takes card 908's one-hook fix in `use-balance-fiat-rates.ts` | **Yes.** It is a one-hook frontend change on a page 958-S6 rebuilds, and B4's "Rand estimate unavailable" depends on it. Add 908's case to S6's tests and mark 908 as covered by 958 | coordinator |
| 4 | 955-S2 changes the thrown error class in `auth-api.ts` and `use-auth-actions.ts` (card says keep them unchanged) | **Accept.** `ApiResponseError` with byte-identical messages keeps every existing assertion green; the alternative is message-string matching. Align the card to "messages and endpoints unchanged" | coordinator |
| 5 | 956: earnings in the Account hub (card) vs dock only (D1a notes 2, 9) | **Board wins: no earnings on Account.** Card alignment only, not for Marcello | coordinator |
| 6 | 956: "Preferences and alerts" link `?tab=general` (968 as built) vs `?tab=preferences` (D1a note 5) | **`?tab=preferences`.** 956-S4a re-points the hub row and Jump after 968 merges; 956-S6 re-points the bell. `?tab=general` and legacy `?tab=profile` stay valid (fall back to Personal details) | coordinator |
| 7 | 956: System alerts "Always on" while some accounts store `system=false` | Backend `origin/main` never reads `notification_preferences` outside the GET/PUT endpoints (no sender checks it), so System alerts **are** always delivered. **Build the "Always on" row as drawn, no client write.** The catch: the other in-app switches are not enforced either (backend gap 12) | coordinator; owner for gap 12 |
| 8 | 957: "Best completion" sort (C5 frame 5); the server sorts by reputation | **Leave the option out** until a backend sort key exists; don't put a completion label on a reputation sort. Card 977 adds the sort; name it on the unticked AC | coordinator |
| 9 | 957: buyer-side seller fee figure (C6 frame 1 "thabo_k pays the 1.00% fee") vs the merchant-tier rule that a seller's tier is not disclosed (16 Sep) | **DECIDED 22 Oct: show the figure** as C6, C7, C18, C22a, C22b and A2 draw it, using the real fee for the seller's tier from the server (main sends seller_merchant_tier and the public tier ladder; C9 shows the tier publicly). Drop "The seller pays the escrow fee" | decided |
| 10 | 957: currency groups have no drawn pagination (C4, C16) | Use 957's proposal: each group shows its first page with the total in the header, and the header links to `?fiat=` | coordinator |
| 11 | 956-S2 global viewport `interactive-widget=resizes-content` | **Accept**, with a keyboard regression check on the 959 room composer and 958 withdraw steps in the 956-S2 review (both built by then) | coordinator |
| 12 | 958 needs `/report?withdrawal=` prefilled, and 956's plan reads only `?trade` and `?user` | **Add `?withdrawal=` to 956-S8's `report-query.ts`** (reference only, no user data) | coordinator |
| 13 | `/earnings` robots rule: `app/robots.ts` already has it, but `public/robots.txt` is the served file (HANDOVER) | **958-S9 adds `Disallow: /earnings` to `public/robots.txt`.** Don't delete the static file: it carries PENTEST-012 entries that `app/robots.ts` lacks | coordinator |
| 14 | 956: support-reply notification opens `/support/tickets/{id}` (915) | **Add the frontend alias to `/my-tickets/{id}` in 956-S6's resolver** (same pattern as 936). Keep 915 open for the server fix | coordinator |
| 15 | Password requirements look: D's copy (d.css `.a-req-n`) lacks E's `.is-all` success colour, icon and `.is-wide` row (auth.css). D's notes call it "group E's component" | **DECIDED 22 Oct:** build E's version once (956-S1) for both | decided |
| 16 | 959: A2 Cancelled frame labels ⋯ "…, cancel" but A8 "More menu · closed trade" has no Cancel | **DECIDED 22 Oct:** follow A8, no Cancel | decided |
| 17 | 959: release confirm width A9a and A19a vs G16b 520px (969 built G16b) | Measure at 959-S5 build time. If they differ, the board drawn for that exact screen wins (A9a/A19a for the trade room), else the majority of frames; log it in HANDOVER | coordinator |
| 18 | 955: rate-limit clock needs `Retry-After`; backend CORS exposes only `content-disposition` (`src/app.rs`) | First check in the network tab whether the review app calls `/api/v1` same-origin through the proxy. If yes, nothing to decide. If cross-origin, raise gap 13. No board draws a timeless banner, so get copy from the owner before building one | coordinator |
| 19 | 955: dormant Google sign-in branches (`isGoogleConfigured`) | **Keep them unchanged and unstyled**; deleting them is an auth-policy change for 761's owner | coordinator |
| 20 | 955: `/verify-email-pending` is not in `PUBLIC_ROUTES`, so a signed-out new customer can be bounced to /login after sign-up (frontend, no card) | Raise a small frontend card for the verification routes. 955 AC1/AC3 sign-up hand-off stays unticked until then | coordinator |
| 21 | 957: margin bounds (−50% to +100% sell, −10% to +50% buy) enforced in the form only | Build in the form as drawn; raise gap 9 | coordinator |
| 22 | Christian items already open | 957 CH6 verification-limit copy and M15 (decision 2) go in the final approval pack | Christian |

---

## 4. Backend gaps (cards 972-980 created 22 Oct; build against main today, name the card on the unticked AC)

1. Participants can't see the moderator's resolution note or decision text on a resolved trade (`dispute.resolution` is not exposed; resolve only notifies). (959) **Card 973.**
2. No trade events are written for "Dispute resolved", "balance restored" or document review, so the room timeline can't show them. (959) **Card 974.**
3. `TradeResponse` doesn't return `payment_reference` ("QT-…"), so the short trade and payment reference can't be shown. (959) **Card 975.**
4. `GET /trades/:id/ledger` returns the counterparty's rows including `balance_after` (data exposure; the UI shows only the viewer's rows). (959) **Card 972 (fix-first, not a redesign dependency).**
5. No customer endpoint exposes the enforced per-network withdrawal minimums; `/config/cryptos` returns a hard-coded "5" for USDT (FE1). (958) **Card 976.**
6. No server sort by completion rate; `sort` uses `users.reputation` ("Best completion", C5). (957) **Card 977.**
7. `OfferResponse` has no weekly trade count ("3 trades this week", "Trades this week"). (957) **Card 978.**
8. Desc sort puts unpriced buy offers first (`compare_offers`, `src/api/offers.rs`, one-line fix). (957) **Card 979.**
9. Offer margin bounds aren't enforced on create or update (C11, C13 say a backend ticket does it; none exists). (957)
10. Withdrawn 22 Oct: the figure is shown from the seller's tier, which main already sends. (957)
11. Notification texts with amounts (D-B4 owner build note) need server copy changes. (956)
12. In-app notification preferences are stored but never enforced: no sender reads `notification_preferences`, so every switch on Preferences and alerts is decorative. (956, found while checking decision 7) **Card 980.**
13. Resolved 22 Oct: frontend only, main's same-origin proxy already passes Retry-After, so 955 reads it. (Was: only if decision 18 finds the review app calling cross-origin: CORS should expose `Retry-After` (card 64's area). (955)
13b. The buyer's escrow breakdown ("Where the 121.20 USDT went") has no field in the trade response; only the seller gets the fee split. (959; since 22 Oct also the trades list's seller "held" figure) **Card 981.**
13c. `/reseller/active` doesn't return the parent offer's owner and margin; the page reads each parent via GET /offers/{id} (cached) until then. (958) **Card 982, low priority.**
13d. Paused loyalty programme returns no month volume (build_status returns dormant before using it), so B17's "October volume so far" is left out. (958) **Card 983.**
13e. Marketplace rows: no seller completion rate, trades, verified flag or last active in OfferResponse, so the trader line is the name only. (957) **Card 984.**
13f. Offer effective maximum ignores an empty or locked seller balance, so C7 "can't fill" never shows. (957) **Card 985.**
13g. Resell trades: no reseller earnings, platform share or markup in TradeResponse. (959) **Card 986.**
13h. No counterparty legal name during an open trade (username stands in). (959) **Card 987.**
13i. No escrow or refund status after a cancel, so "balance restored" can't show. (959) **Card 988.**
13j. Chat "Seen": no read state on trade messages. (959) **Card 990.**
13k. Your own rating isn't readable after a reload. (959) **Card 991.**
13l. Trade documents store and return no original file name. (959) **Card 992.**
13m. Unlocking a lock that funds a Held offer escrow (money safety). (958) **Card 996.**
13n. GET /wallet/locks has no trade or escrow id; unlock releases the newest lock, not a named one. (958) **Card 995.**
13o. A second manual lock on one offer returns 500. (958) **Card 994.**
13p. Closing an offer keeps its lock and offer escrow. (958) **Card 993.**
13q. Affiliate earnings rows have no referred customer name. (958) **Card 997.**
13r. pending_balance is never written by the withdrawal flow, so the wallet sums GET /wallet/pending. (958) **Card 998.**
14. Backend intent doc (`intended-entity-state-machines.md` 1.1 and 4.2) still describes split dispute resolution. Card 963 should update it; raise it there, not as a new card. (959)

---

## 5. Card text to align to the boards

Back up each card first (`card-backups/`, as for 968, 969, 970). Every card: AC widths say 390 x 844 and 1440 x 900, but the boards also draw 768 (and E draws 720 and 320/360 keyboard); WCAG 2.2 AA carries the documented CTA exception (white on `--action` 2.77:1, kept by Christian).

**959 (trade room)**
- "I have paid" → "I've paid"; the confirm reads "I've paid R 2,184.00" (A2, A8, A12).
- "Release USDT" → "Release 120.00 USDT" with the amount (A3, A8).
- Remove the HARC cancel wording and AC3's HARC clause: cancel restores "121.20 USDT back in Available" in the same step; HARC is withdrawals only (A1a, A1b, A10, A21a).
- Escalated dispute: the customer sees the same "In dispute" state, with no deadline or escalated frame (A1a, A18a).
- "A moderator's cancel returns 121.20" → outcomes "Released to the buyer" and "Returned to the seller" under the chip "Dispute resolved" (A1a, A2, A3).
- "Keeping both URLs" → `/trade-history/[id]` is a 308 to `/trade/[id]`; the legacy page goes (A16).
- AC5 "existing query and filter parameters": none exist today; the board adds tab and filter state to the URL and replaces the crypto filter with "Paid in" and the seven labels (A13 to A15).
- Status-name sources: drop `ui/escrow-badge` and `utils/format.ts` (no status map there); the sources are the room components and `status-display.ts`.
- Start-trade screens and "Max includes the fee" live on 957's offer page (A21a "layer for C"); 959 supplies the shared step (decision 1).
- Add: request reference "X-XXXX-XXXX · sent hh:mm:ss" on every pending, failed and unknown step (A21a, A21b, A22); Report a problem to `/report?trade=`, rating restyle, Reseller watching, 1,000-character dispute reason (A8, A10, A18a; backend 921).

**958 (wallet and money)**
- Fix sketch "rebuild `wallet/manage/[currency]` and `dashboard`" → both become 308s to `/wallet` (B0, B13).
- "Keep the redirect pages" → deposit and withdraw entries are one 308 each to `/wallet?action=…` (B0).
- Example "10.500000 USDT" → 2 decimals, up to 6 when present, never rounded; QA compares to 6 decimals (B7a).
- "Rate unavailable" → "≈ R" and "Rand estimate unavailable", never "R 0.00"; NGN and KES the same (B4, M10).
- "Backup-code access where supported" → backup codes only at sign-in; "Can't use your authenticator?" opens account recovery (B7a).
- "Emailed code for flagged accounts" → emailed code only for accounts without an authenticator in the pilot (v5.2 scope correction).
- Transfers "double submit and two tabs never send twice" → stays unticked, blocked by 920.
- Add `/earnings` overview (B11) replacing 968's temporary redirect.

**957 (marketplace and offers)**
- Problem and AC3 fee label (656) are stale → "no taker platform-fee line; the escrow fee shows in USDT".
- "Once JP confirms the rate" → 1.00% decided (F1, A2); only 911 and 918 remain.
- Entry points "from Marketplace and Home" and "in the header" → Create offer in the marketplace header (768, 1440), on My offers and its empty state, the My offers pill and Manage links; nothing on Home (B0, C1 to C3, C10).
- Add the `/prices`, `/prices/live`, `/prices/gas` retirement (C17); "rebuild dashboard/listing" → 308 and delete.
- Start-trade confirmation and outcomes are drawn on the offer page (C6 20-29, C7, C8, C22a/b); state the 957/959 split as in decision 1.
- Three edit modals → one Edit sheet with price, limits and documents (C11 frame 5).
- File list: create components live in `components/offer/create/*`, moving to `features/offers/create/`.
- Note in AC4: a bare `/marketplace` now opens Buy USDT (today's `?tab=sell` list), per C-2.

**956 (account and support)**
- Account hub covering "earnings (Reseller, Affiliate, Loyalty)" → Account shows no money or earnings; Earnings is a dock destination (D1a notes 2, 9).
- `settings/notifications` route → doesn't exist; in-app alerts move to `/settings?tab=preferences` and the notifications page loses its settings tab (D3, D4).
- "General, payment and ordinary tabs" → five sections incl. Preferences and alerts; Security card rebuilt around the unchanged 2FA panel (D3).
- "Attach files to support tickets" → files attach on replies only; the new-ticket sheet has three fields (D5).
- "Read and clear notifications" → no clear-all; "Mark all read" and per-item Delete with Undo (D4).
- Related section claims the hosted canvases show v5.2 → they still show v5; build from v5.2 sources and PDFs (CHANGES.md).
- AC3 server enforcement (D8b 4b) stays unticked: 917, 964.

**955 (sign-in and sign-up)**
- "Weak/Fair/Strong meter" → "Password requirements" with "n of 5 requirements met" (E2d C, E4d B).
- AC4 "locked" → one rate-limit message for both limits, no separate lockout frame (E1d D).
- "Keep `use-auth`, `auth-api` … unchanged" → endpoints and messages unchanged; `http-client.ts` and `session-idle.ts` gain Retry-After and `&redirect=` (E6), and the thrown error class changes (decision 4).
- Scope: the compact header also applies to `/verify-email` and `/verify-email-pending` (D-E3).
- Landing after sign-in is `/wallet` (E6, PR-E3); 935 AC3's "lands on the dashboard" becomes `/wallet`.
- `/signup` with a live session goes to `?redirect`, else `/wallet` (E1d K, E2d N).
- Large-screen states reuse the phone column; QA captures 768 and 1440 default frames only (D-E9, E7).
- Toast 4s on E1d J → 6s by owner decision (no card text today; nothing to change).
- 755 has no description; close it as covered by 955 or link it.

### Frontend items moved into slices (22 Oct)
- 955-S2: add `/verify-email-pending` and `/verify-email` to PUBLIC_ROUTES, and test that the emailed token is not spent before the redirect (was decision 20; no card needed).
- 955-S2/S4: read `Retry-After` through main's same-origin `/api/v1` proxy for the "Try again at" time (decision 18 resolved).
- 958-S4: the withdrawal key replay fix now belongs to backend card 920; 958-S4 still binds the client key to the withdrawal details as planned and names 920.

### Cleanup sweep list (one small slice near the end, 23 Oct)
- `e2e/pages/trade.page.ts` `confirmReceipt()` has no callers (since 957-S1).
- `ticket-958-s8-redirects.test.ts` has a test name that still calls /dashboard/listing live.
- `use-prices` and `use-crypto-conversion` have no product user; delete once no other card's test imports them.
- Files nothing imports after 958-S8 (listed in .scratch/958/reports/958-S8.md) and links that still reach /wallet/manage or /dashboard through one redirect.
- 958-S1 SecurityCheckStep LOWs (code boxes editable and Back hidden rather than greyed while a send waits); 958-S4 LOWs needing components/money changes.
