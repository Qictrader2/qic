# 968 Phase 1 BOIL report

Card 968 (Trello xcgVXhJY), 906-0c: phone bottom bar, one balance and clear overlays. Size L, native mode, risk area authorization.
Verdict: built to completeness locally. No push, no PR, no Trello write, no self-review.

## 1. Base and merge-base

- Worktree: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-root/frontend, branch 968-bottom-bar-header-overlays.
- Base origin/redesign/906 = f44c50dcf55162cf2f5a14cf689c91675312e3b3 (970 merged). `git merge-base HEAD origin/redesign/906` = f44c50dc. Match.
- Backend untouched.

## 2. Commits (oldest first)

1. 63c26df7 968: Bottom bar routes, dock pages, earnings redirect and glass chrome styles
2. 44bb04f9 968: Trades badge and dock state hooks
3. 77866acd 968: Account hub on /profile with role-gated staff rows
4. 3a00c3ab 968: Customer header band, phone header and Jump palette
5. b93ef3ea 968: Phone dock with board geometry, badge, keyboard and overlay states
6. ad660d62 968: Match the boards for Jump focus return, narrow back control and bell popover rows
7. 2fc674d3 968: Tests for the customer header, phone header and Jump palette

## 3. Size

- Product: +2,497 / -20 lines in 25 files. Tests: +2,694 / -2 in 17 files. Under the 3,000-line / 50-file stop.
- Largest product files: config/routes.ts +599, AccountHub.tsx 319, JumpPalette.tsx 297, PhoneHeader.tsx 260, NotificationsGlass.tsx 224, use-bottom-bar-state.ts 138, BottomBar.tsx 132, globals.css +94 (one 968 block at the end of the file).
- Full list: /Users/starbird/Workspace/Code/qic/.scratch/968/evidence/logs/numstat.txt. No comment lines added (scanned).

## 4. Gate receipt

`QIC_GATE_ROOT=.../968-root .claude/scripts/gate.sh --base origin/redesign/906` at 2fc674d3: GATE PASS (typecheck 3 s, lint no new errors 32 s, build 32 s).
Receipt: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-root/.scratch/gate/20261022T065343. Log: .scratch/968/evidence/logs/gate-1.log.

## 5. Tests (targeted, never the full suite)

- navigation + design folders and every suite importing a touched file: 24 suites, 1,019 tests, all pass (logs/jest-targeted.log).
- Other importers (ConsentBanner, header-fiat-864, header-live-fx, both last-known-label-864, sign-out entry points, ticket-405, ticket-556 x4, ticket-936, ticket-nav-affiliate-active, ticket-nav-wallet-dup, support-chat-placement, use-trades-badge, staging-gate, seo metadata, dsp-63): 21 suites, 180 tests pass (logs/jest-importers.log).
- ui-fixes.test.ts fails to run: it reads ../qictrader-backend-rs/src/services/validation.rs, which this frontend-only worktree does not have. Environmental; it imports nothing 968 changed.
- ticket-nav-* and header-live-fx tests are unchanged. customer-surface.test.ts gained the /earnings row and its page count went from 97 to 98 (exhaustive table, needed for the new page).

## 6. The three artboard decisions (owner rule: the artboards decide)

1. **Breakpoint: dock below 768 px, island and capsule from 768 px (card said bar below 1024).**
   Evidence: G1 Q7 "Below 768 px is the phone shell ... the dock is capped at 440 and centred"; B0 "Below 768 CSS px is the phone shell"; D7b section 8; E8 "the pack's single breakpoint ... below 768". Every 768 x 1024 frame draws the island with a search icon and bell and no dock (D6b, C2, B2, A14, D1b, D4b). No board draws a dock at 768+ or an island below 768. The only "1024" is card 968's own row on I7.
   Built: dock below 768; band at 768-1023 with shield logo, text-only island, search-icon capsule; from 1024 wordmark and icon+label island.
2. **No dock on /offer/create, /offer/[id], /offer/resell, /profile/[userId] (card said bar shown, Market active).**
   Evidence: C12 title "Create offer as five short steps, with no dock"; C6/C7 "A focused task: no dock"; C9 "Deep page, no dock"; C16 "the phone has no dock" and "the island marks Earnings"; G1 rule 1 "Task pages (trade room, offer create, auth) have none". Large frames of the same routes draw the island (C6, C9, C16, C12 via C13/C14).
   Built: no dock on those four on phones; island at 768+ with Market active (Earnings on /offer/resell per C16). The same board rule removes the dock from every level-2 and task page (settings, notifications, tickets, report, trade room, trade-history/[id], wallet manage/deposit/withdraw, affiliate tiers per B16a) and gives them the board's back header instead.
3. **Material: option-3 pale frosted glass with --foreground focus (card said ink surface with --focus-on-ink).**
   Evidence: C1, G8, G9, D6b, B11a, B12, B14, A4a, A6a draw pale frosted glass; G0 "The glass dock and island take --foreground"; D7a and glass-v3.1.css 5i (`.m3 .dock-item.is-focus ... outline:2px solid var(--foreground)`). --focus-on-ink is for navy section pills. A17b has one sentence saying the dock uses --focus-on-ink; it contradicts G0, D7a, D0, G17 and the CSS, so the majority wins.
   Built: .gdock.m3 / .gisle.m3 / .gcap.m3 with frost, grain and skin layers; 2 px --foreground outline at -2 px.

## 7. Other v5.2 differences from the card, built the board's way (owner decision needed on 1 and 2)

1. **No balance in the customer header at any width.** No board draws a balance in the band, capsule, phone header or dock (G8, G9, D6b, C1-C3, B1-B3, D1a-D1c). AC3's "the desktop header shows the balance once" therefore reads as zero on customer pages; the balance lives on the Wallet page. Staff and public pages keep today's header with HeaderBalance and BalanceStrip unchanged (their double HeaderBalance mount, hidden by CSS, is left as today).
2. **No header Support button.** Chat opens from the Account hub "Chat with support" row and the Jump action "Chat with support" (D0, D1a, G9). The floating button is gone from customer pages at every width (D0, D1a note 6), not only on phones; it stays on staff and public pages with today's classes (#00A8FF kept byte for byte under the default-surface rule; the raw-hex sweep is 906-0d/0e).
3. Phone header: no logo, balance, avatar or burger on customer pages. Top-level pages show the title and the bell (a link to /notifications on phones, D4a); level-2 pages show the board's back control and title.
4. Large screens: island + Jump palette (G8, G9) replace NavLinks and the avatar menu on customer pages; staff reach /moderator and /admin through role-gated rows in the Account hub and the palette.
5. Bell at 768+: glass popover per D4b/D4c via an opt-in `variant="glass"` on NotificationsDropdown; default variant unchanged for staff/public pages.
6. Trades badge counts trades waiting on you (A13 "counts the trades waiting on you (2)", D4a note 2): buyer on escrow_funded (or created with escrowRequired false), seller on paid. Accessible name "Trades, 2 need you".
7. Keyboard open: the dock drops 16 px and fades (C5 frame 11), rather than the card's "never animate position".
8. /dashboard/affiliate/tiers has no dock (B16a "level-3 page: no dock").

## 8. Board values not replicated (impossible in code or contradicted between boards), for the owner

1. Jump field from 1024: logo + 576 px island + 296 px capsule only fit from 1232 px. Built: compact capsule (search icon + bell) 1024-1279, full Jump field from 1280. The switch is not drawn on any board.
2. 9+ badge at 360: G2 dock-fix puts it at left:18px; G15 v311 `is-badge-in` puts it at right:-10px below 375 px. Built G15 (the later rule).
3. Glass edge colour: B-group b.css uses color-mix(--action-pressed 25%); shared glass.css and groups A, C, D use rgba(3,105,161,.25). Built the shared value.
4. Back control style differs by group (D 15/500 in --action-pressed, C 16/600, A 15/600 at 52 px, B icon button). Each route uses its own group's style.
5. Edge fade behind the band: the board hard-codes rgba(244,246,249,...) (old background); built with var(--background) at the same opacities. D1c draws no edge fade while G8/C3 do; built with the fade.
6. Arrow key caps in the palette hint band are 20.7 px wide instead of 22.7 px (fallback glyph font).
7. Account hub: at 768 the hub is 728 px against 720 because today's /profile page padding is 20 px (board 24, page owned by 956); at 1440 it sits at x 984 / y 116 against the board's 960 / 228 (page padding and the title/pills above it are 956's). Chip radius 9999px vs rounded-full (same shape). Version "1.1" hard-coded as the dropdown already does.
8. Bell popover: no "Your turn" chip (no data marks a notification as waiting on you), no security-type icon, loading and error states not built.
9. Not drawn anywhere, so the overseer chose: Jump "No matches" row, empty-query list, list height cap min(456px, 100vh - 280px), phone header background and scrolled hairline (from C ph-head.is-scrolled and D1a), the 768/1024 band variant switch at 1024, the hub 1280 column breakpoint.
10. Page-owned pieces left to their cards: Market filter icon (C1), Trades export (A13), Mark all read (D4a), trade Help and more menu (A2), offer and public-profile more menus (C6, C9), "Step n of 5" and progress bar (C12), Activity export (B), page titles "Resell an offer" (C16) and "Affiliate tiers" (B16), the Account title row, "See how others see you", "Edit details" and the Profile/Settings/Notifications/Support pills (D1c, 956).

## 9. Authorization tests

- Unit: staff-entries-gating.test.ts (visibleStaffEntries over the real role mapping), account-hub.test.tsx, user-profile-dropdown-staff.test.tsx and jump-palette.test.tsx each run user, moderator, admin, super_admin and investor. Results: user none; moderator Moderator Dashboard only; admin and super_admin Moderator Dashboard + Admin Panel; investor War Room only (not when admin). Same gating as UserProfileDropdown today (its role checks are byte for byte; it only gained data-testids and lost the two email console.log calls).
- Browser (seeded roles in the test browser only): proof s7, 15 of 15 pass at 390/768/1440.
- Server and layout guards unchanged (no diff under admin, moderator, auth or api). Real server roles deferred to QA.

## 10. Observed proof (local production build of 2fc674d3, port 3968 because 3000 belonged to another session)

Index: /Users/starbird/Workspace/Code/qic/.scratch/968/evidence/proof/proof-index.json (133 entries). Board comparisons: evidence/dock (14 pairs), evidence/header (band and phone header pairs for G8, G9, C1-C3, B1-B3, A2, A13, A14, B11a, C6, C9, C12, C16, D1a-D1c, D3a, D4a, D4b, D5a, D6a, D6b), evidence/palette (G9), evidence/account-hub (D1a-D1c), proof/s10 integrated pairs. Logo, island, capsule, dock and badge geometry, colour, radius, shadow and type match within 0.5 px; the differences left are those in section 8.

Verified locally: dock at 320/360/390 per active state; 9+ inside its pane at 320; absent at 768+, signed out, staff, public, excluded and level-2 pages; keyboard and covered states; consent banner 12 px above the dock with nothing stored; customer band with zero balance controls and one bell; Jump palette Ctrl K and "/"; Back keeps ?status=active and ?action=withdraw; Earnings lands on /dashboard/affiliate and Back skips /earnings; seven route handlers respond with an empty src/app/api diff.

## 11. Acceptance

Map: /Users/starbird/Workspace/Code/qic/.scratch/968/reports/acceptance-map.md. 17 rows verified locally (some with recorded limits), deferred: AC1.5 and AC5.3 to QA (signed in on the review app), AC2.3 toast placement until 969's container reads --qt-overlay-bottom, real-device keyboard and real server roles to QA.

## 12. Open risks

1. Owner decisions needed: no balance in the customer header (section 7.1) and no Support button (7.2).
2. 969 contract: --qt-overlay-bottom is calc(env(safe-area-inset-bottom) + 8px + 64px + 12px) (G15: 118 at a 34 px inset, 84 in a window), set only under html[data-bottom-bar="visible"]. While the dock is hidden for the keyboard or covered, data-bottom-bar is "keyboard" or "covered" (clearance kept, variable unset, so 969 falls back). The toast container on this branch is still bottom-0 pb-6 and would sit over the dock until 969 reads the variable.
3. At 320 px /wallet (WalletCard swatches, scrollWidth 343) and /profile (ProfileHeader reputation row, 357) overflow sideways. Page content outside 968's files; AC2 "long names and amounts do not overflow" needs 958/956 or a follow-up.
4. Phone headers add a title above pages that still render their own old heading until the page cards land (interim duplication on redesign/906).
5. Keyboard detection is heuristic (visualViewport or coarse pointer); verified on the coarse-pointer path only. Needs iOS and Android checks in QA.
6. /earnings on a hard load answers 200 with a streamed NEXT_REDIRECT (replace, 307) instead of a Location header; client navigation replaces correctly. It exists only on redesign/906 until 958 builds the Overview.
7. src/hooks/use-moderator-auth.ts:34 still logs the user's email to the console (pre-existing, not in 968's scope).
8. Hub counts: support tickets count only reads the first 10 tickets.
9. DESIGN.md: no change made; the read-only copy was not edited. The new customer chrome follows the v5.2 pack rather than DESIGN.md's current header description; whoever owns DESIGN.md should add the dock, island and Jump palette.
