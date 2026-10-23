# 968 acceptance map

Card 968 (Trello xcgVXhJY). Accepted intent: card text and checklist, owner decisions of 21 Oct (destinations) and
22 Oct (replicate the v5.2 artboards exactly; artboards win over card text). Board-derived decisions are listed in
/Users/starbird/Workspace/Code/qic/.scratch/968/reports/boil-brief.md (sections A to M) and in phase-1-boil.md.
Source pins: frontend base origin/redesign/906 f44c50dc, branch 968-bottom-bar-header-overlays (BOIL head 2fc674d3). QA environment: review app https://qictrader-fe-pr-303.herokuapp.com after merge (signed in).

| Criterion and requirement | Accepted behavior and owner decisions | Real route, consumer or artifact | Environment and source pins | Exact assertion | Evidence path | State and responsible phase |
| --- | --- | --- | --- | --- | --- | --- |
| AC1.1 five destinations in order | Wallet, Market, Trades, Earnings, Account (owner 21 Oct) | dock (<768) and island (>=768) | local prod build | five links in order with hrefs /wallet, /marketplace, /trade-history, /earnings, /profile | evidence/dock, evidence/island, jest | verified (BOIL): bottom-bar.test, main-header-customer.test; proof/s1, s6 |
| AC1.2 active item on every customer route | card rules (brief D); /offer/resell -> Earnings per C16 | getBottomBarActive over every page.tsx | jest | exactly one or no item active per route; aria-current=page | jest bottom-bar-routes | verified (BOIL): bottom-bar-routes.test over every page.tsx; proof/s1, s6 aria-current |
| AC1.3 dock only on top-level pages on phones | boards (brief C) override card | isDockPage | jest + local build | dock absent on level-2/task pages, present on dock pages | jest, evidence/dock | verified (BOIL): isDockPage tests; proof/s2-absence-level2.json (11 level-2 pages, no dock, back control shown) |
| AC1.4 Trades badge, 9+ | G2, G15, D4a | dock and island Trades item | local build (seeded) | badge absent at 0/error/loading, digit 1-9, "9+" above 9, inside its pane at 320 | evidence/dock | verified (BOIL) with fixtures: "2", "9+" inside the pane at 320, none at 0 (proof/s1-dock.json); badge counts trades waiting on you (A13, D4a) |
| AC1.5 signed-in spot checks on review app | | review app | after merge | each item opens its route, active correct | QA folder | deferred-to-QA (review app, signed in) |
| AC2.1 labels fit at 320 and 360, safe area, 6 px insets, 440 cap | G2, G15 | dock | local build | label rects inside cells; scrollWidth == viewport | evidence/dock | verified (BOIL) for the dock: labels inside cells at 320/360/390, insets 6/12, bottom 8 (42 with 34 px inset), 440 cap at 600; scrollWidth fails at 320 on /wallet (343) and /profile (357) from page content outside 968 (WalletCard, ProfileHeader): open risk for 958/956 |
| AC2.2 keyboard open hides the dock | C5 frame 11 | dock + hook | local build | data-bottom-bar removed, dock inert, field visible | evidence/overlays | verified (BOIL, coarse-pointer path): inert, aria-hidden, 16 px drop, opacity 0, data-bottom-bar=keyboard, no layout jump (proof/s345-overlays.json); real iOS/Android devices deferred-to-QA |
| AC2.3 toast 12 px above the dock | G6, G15; 969 container reads the variable | --qt-overlay-bottom | local build | toast bottom edge 12 px above dock top | evidence/overlays | deferred: --qt-overlay-bottom = calc(env(safe-area-inset-bottom)+8px+64px+12px) set and measured (84 px with no inset); the toast container is 969 and still bottom-0 pb-6 on this branch, so the toast check waits for 969 to read the variable |
| AC2.4 consent banner above the dock, steps aside for keyboard | card (no board) | ConsentBanner | local build | banner bottom >= dock top; never dismissed | evidence/overlays | verified (BOIL): banner 12 px above the dock, nothing stored (proof/s345-overlays.json) |
| AC2.5 sheet/dialog open: dock fades and is inert | G1 rule 3, G3, G5 | dock | local build | inert + aria-hidden + opacity 0 | evidence/overlays | verified (BOIL): Radix Customise dialog and chat panel give opacity 0, inert, aria-hidden, data-bottom-bar=covered, 180 ms ease-in; reduced motion opacity only 150 ms linear |
| AC2.6 no floating chat on customer pages | D0, D1a | SupportChatWidget | local build | no support-chat-fab on customer pages; present on /help | evidence/overlays, jest | verified (BOIL): support-chat-placement.test; proof/s2 (FAB only on staff/public pages) |
| AC2.7 absent signed out, >=768, staff, public, excluded | card + boards | dock | local build + jest | no bottom-bar element, no data-bottom-bar | evidence/absence | verified (BOIL): proof/s2-absence-*.json (signed out, staff, public, excluded, 1024/1440); /login, /auth/callback/google, /account/kyc redirect a signed-in user by pre-existing code (unchanged), so their dock state is n/a |
| AC3.1 balance in customer band | boards draw none (G8, G9, D6b); card says once | customer header | local build | 0 HeaderBalance, 0 BalanceStrip on customer pages; staff/public unchanged | evidence/header | verified (BOIL) against the boards: 0 balance controls and 0 BalanceStrip on customer pages at 768/1024/1440 (G8, G9, D6b, C3 draw none); /help keeps today header with balance and strip. Card text says "once": owner decision needed (see phase-1-boil.md) |
| AC3.2 staff entries role-gated | card AC3, D0 | Account hub, Jump palette, desktop menu | jest (user, moderator, admin, super_admin, investor) | exact entry sets per role | jest | verified (BOIL): staff-entries-gating, account-hub, user-profile-dropdown-staff, jump-palette tests for user, moderator, admin, super_admin, investor; proof/s7 browser-seeded roles; real server roles deferred-to-QA |
| AC4.1 Back through the bar keeps id and query | Next Link push | dock/island links | local build | history back returns to previous URL with query | evidence/history | verified (BOIL): proof/s8-history.json (?status=active and ?action=withdraw survive Back) |
| AC4.2 /earnings redirect keeps query, Back skips it | brief M | /earnings | jest + local build | redirect to /dashboard/affiliate?query | jest | verified (BOIL): earnings-redirect.test; browser: lands on /dashboard/affiliate?query, Back returns to /marketplace; hard load answers 200 with a streamed NEXT_REDIRECT replace 307 (no Location header) |
| AC5.1 staff/public/excluded pages unchanged | card AC5 | MainHeader default path | local build | same header markup as base | evidence/absence | verified (BOIL): default branch of mainHeader.tsx, FAB markup, NotificationsDropdown default variant unchanged (git diff); proof/s2 staff and public pages show today header |
| AC5.2 seven route handlers respond | card AC5 | api routes | local build | status codes match base | evidence/handlers | verified (BOIL): src/app/api diff empty; all seven respond locally (proof/s9-route-handlers.json; /api/v1/health 500 BACKEND_URL not configured locally) |
| AC5.3 2FA and KYC flows submit as before | card AC5 | /login, /account/kyc | review app | | QA folder | deferred-to-QA |
| DES.1 board replication | owner 22 Oct | dock, island, capsule, palette, phone header, Account hub, overlays | local build vs .dc.html | side-by-side + computed styles match | evidence/compare | verified (BOIL) with recorded exceptions: evidence/dock, header, palette, account-hub, proof/s10 pairs and computed styles; exceptions listed in phase-1-boil.md |

## Phase 1 (BOIL), 22 Oct 2026

Head 2fc674d3 on base f44c50dc. Gate PASS, receipt /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-root/.scratch/gate/20261022T065343. Local production build on port 3968 (port 3000 belonged to another session). Proof index: /Users/starbird/Workspace/Code/qic/.scratch/968/evidence/proof/proof-index.json (133 entries). Report: phase-1-boil.md.

## Phase 5 (ACCHECK), 22 Oct 2026

Review app release v7 (frontend 366dca2f = 968 fe023ad2 + 969), staging backend, buyer-906 / seller-906 / third-906 (temporarily moderator, restored). Report: phase-5-accheck.md. Evidence: ~/.config/qic/qa-artifacts/968/.

| AC | Verdict | Jev | Trello | State |
| --- | --- | --- | --- | --- |
| AC1 | MET by overseer (24 card routes, 0 mismatches at 390/768/1440) | met 0.53 after retry, below gate | unticked | held for arbiter |
| AC2 | MET with limits (no real-device keyboard, safe-area inset 0 in capture); AC2.3 toast now verified 12 px above dock at 390/360/320 on v7 | met 0.64 after retry, below gate | unticked | held for arbiter |
| AC3 | MET (AC3.2 real server roles verified: user none, moderator Moderator only, super_admin both; hub, Jump, desktop menu) | met 0.89 (retry) | ticked | verified |
| AC4 | MET (trade ID, ?status=active, ?action=withdraw survive Back; hard-load back stays in app; /earnings keeps query) | met 0.91 | ticked | verified |
| AC5 | MET with limits (real submits and 2FA step not run; AC5.3 deferred-to-QA) | met 0.43 after retry, below gate | unticked | held for arbiter |

## Phase 6 QA (22 to 23 Oct 2026, review app v30 to v32, frontend 1679d151)

- AC1: verified hands-on at 390, 768 and 1440. /earnings is now the real page.
- AC2: **failed**. On a first visit at 390, the consent banner (z-50) covers the open chat sheet (z-41) and its buttons cannot be used while the chat is open. Every other AC2 case passed: no-dock pages, 12 px toast and banner gaps, Tab focus at 700x560, labels at 320 and 360.
- AC3: verified, including the staff menu per D0 for super_admin and no staff menu for plain users.
- AC4: verified.
- AC5: verified. /admin and /moderator are unchanged, and the real bio save and report submit worked.
- Evidence: /Users/starbird/.config/qic/qa-artifacts/968-20261022T174013Z/. Report: phase-6-qa.md. Card moved to QA Failed.

## Phase 6 re-test (23 Oct 2026, review app v35, frontend ca8fbe25 with #337)

- AC2: **failed**. The original finding is fixed: the banner is covered, inert and unreachable under the chat sheet (390), chat popover (768, 1440), Jump, Cookie preferences, notifications, Export trades, Filters and the daily notice, returns in place, and Reject all, Accept all and Customise are stored and survive a reload. New failing case: at 390 the super_admin staff menu panel (UserProfileDropdown, no dialog role or data-qt-overlay) opens under the unanswered banner, which hides 6 of its 14 entries including Sign Out. Evidence: `~/.config/qic/qa-artifacts/968-retest-20261023T051541Z/`. Report: `phase-6-qa-retest.md`.
- Regression on v35 (dock, island, Jump, chat focus return, staff menu placement, /wallet hard load, 12 px banner and toast gaps): verified.

## Phase 6 re-test 2 (23 Oct 2026, review app v41, frontend 3e37fe68 with #343)

- AC2: **verified**. Staff menu (super_admin) at 390, 768 and 1440: 14 of 14 entries hit-test, banner covered, inert and unreachable while open, back and working after close, choice stored. Trade room more menu, marketplace offer row menu and notifications popover: same. Wallet More menu has no mount point in this build; proved by the fix's unit tests (24 of 24 at 3e37fe68). Customise focus returns to Customise 32 of 32; Accept, Reject and Save return focus to the prior element or main 18 of 18. Known, not 968's: /my-offers row menu (MyOfferCard, 957-S10) still opens with the banner showing. Evidence: `~/.config/qic/qa-artifacts/968-retest2-20261023T081832Z/`. Report: `phase-6-qa-retest-2.md`.
- Regression on v41 (chat, Jump, dock, island, 12 px banner and toast gaps, staff menu at four sizes, /wallet hard load): verified, same values as the last run.
- Card moved to QA Approved / Ready For Deployment on the owner's instruction.
