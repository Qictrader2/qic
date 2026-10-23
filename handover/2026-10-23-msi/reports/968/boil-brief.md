# 968 BOIL brief for builders (input file, owned by the 968 overseer)

Card: QicTrader 968 (Trello xcgVXhJY), "Customer redesign 906-0c: phone bottom bar, one balance and clear overlays".
Worktree (the only place you edit): /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-root/frontend
Branch 968-bottom-bar-header-overlays, base origin/redesign/906 f44c50dc. Never switch branches. Never touch
/Users/starbird/Workspace/Code/qic/frontend. bun only. No new npm dependency. Never symlink node_modules.
A hook forbids git stash, hard reset, checkout -- / restore (discard) and clean: use commits.
Commit only your own files with `git add <exact paths>` (never `git add -A` or `git add .`; other builders work
in the same worktree at the same time). If the index is locked, wait a few seconds and retry.
Commit message format: `968: <short description>`. No push. No PR. No Trello writes.
Write NO code comments in added lines (no //, /* */, {/* */}). Meaning goes into names and test names.

## Owner rules (binding)

1. The v5.2 artboards are the spec, and the build must REPLICATE them, not interpret them (owner, 22 Oct).
   Every value the boards show is required: colours, sizes, spacing, radius, type, borders, shadows, focus
   rings, every state (each active item, badges, keyboard open, sheet open, signed out) and copy.
   Where a board and the card text, qt-v5.css or the planner's investigation differ, the board wins.
   A difference may be left unfixed only if it is impossible in code or two boards contradict each other.
   List every such case in your final message with board ids and values.
2. Bottom-bar destinations, in order: Wallet, Market, Trades, Earnings, Account (owner, 21 Oct).
3. Authorization: moderator and admin entries appear only for those roles, exactly as useModeratorAuth
   (moderator, admin, super_admin) and useAdminAuth (isAdmin: admin, super_admin; isInvestor for War Room,
   shown only when not admin) filter today. Server and layout guards unchanged.
4. Keep isRouteActive, isNavRouteActive, MAIN_NAV_ROUTES, PROFILE_MENU_ITEMS, composeMobileMenuItems unchanged.
5. Staff, public and excluded pages (admin, moderator, landing, help, FAQ, legal, contact, /escrow, /whatsapp,
   /verify-email(-pending), OAuth callback, 2FA, KYC, auth) keep today's header, drawer, footer, strip, FAB and
   behaviour. Default-surface class strings stay byte for byte. The seven non-page route handlers keep responding.
6. Do NOT edit src/components/ui/toast.tsx or any dialog or sheet component (card 969 owns them).
7. DESIGN.md at 968-root/DESIGN.md is a read-only copy; report needed changes instead of editing.
8. Use what 970 shipped: isCustomerSurface and normaliseSurfacePath (src/config/routes.ts), the customer: Tailwind
   variant, v5.2 tokens in globals.css (--ink-surface, --foreground, --action, --action-pressed, --action-subtle,
   --attention, --warning, --gl-* glass variables, --dur-*, --ease-*), the glass v3.2.1 rules already in
   globals.css (.gdock.m3 deltas, .gl, .gl-palette, .gl-mi, .gl-group, .gcap.m3 trigger state, .hit44, fallbacks).
   Do not duplicate them.

## Board sources (render these; never open hosted claude.ai canvases)

PDFs: ~/Downloads/906-design-pack-v5.2/906-v5.2-<id>-*.pdf (only changed boards have PDFs).
Editable HTML sources (exact CSS values live here):
/Users/starbird/Workspace/Code/qic/.scratch/customer-app-redesign/research/design-pack/v5.2-src/<group>/project/
- shared/project: dock-fix.dc.html (G2 dock at 2x, 390/360/320, badges), v311.dc.html (G15 dock cap, 9+ rule,
  toast placement), ph-sheet.dc.html (G3 sheet over wallet, dock faded), ph-menu.dc.html (G5 action sheet, dock
  faded), ph-toast.dc.html (G6 toast 12 px above the dock), lg-menu.dc.html (G8 1440 island band), lg-palette.dc.html
  (G9 Jump palette), lg-popover.dc.html (G10 bell popover over affiliate), glass.css (v3 runtime: base .gdock,
  .gisle, .gcap, .m3 .frost/.grain/.skin, .isl-item, .dock-item), glass-v3.1.css (v3.2.1 deltas), qt.css.
- C-discovery/project: mkt-390.dc.html (C1), mkt-768.dc.html (C2), mkt-1440.dc.html (C3), listing (C17), myoffers-bp (C10).
- B-money/project: wallet-390/768/1440.dc.html (B1/B2/B3), earnings-1.dc.html (B11a).
- A-trading/project: history-390/768/1440.dc.html (A13/A14/A15).
- D-account/project: profile-entry-390/768/1440.dc.html (D1a/D1b/D1c Account hub, chat panel, Jump),
  notif-390/768/1440 (D4a/b/c bell and popover), tickets-390 (D5a).
Design evidence already gathered: /Users/starbird/Workspace/Code/qic/.scratch/968/evidence/board-survey.md (read it).
Planner investigation (card rules, superseded where boards differ): /Users/starbird/Workspace/Code/qic/.scratch/968/reports/investigation.json

## Accepted design (board-derived; the overseer recorded these decisions)

A. Breakpoint (G1 Q7, B0, D7b, E8; every 768 frame): the phone dock shows below 768 px only. From 768 px the
   customer band shows: logo | glass island | glass capsule. 768 to 1023: shield-only logo, text-only island
   items, capsule = Jump search icon button (aria-label "Jump to, Control K") + bell (D6b, C2, D1b). 1024 and up:
   wordmark logo, icon + label island items, capsule = "Jump to..." field with a "Ctrl K" key cap + bell (G8, G9,
   C3, D1c). The switch between the 768 and 1440 variants at 1024 is not drawn; it is the overseer's choice.
B. Material (G0, D7a, G1, glass-v3.1.css 5i; C1, G8, D6b drawn): dock, island and capsule are option-3 pale
   frosted glass (.m3), never navy ink. Focus on dock and island cells is a 2 px --foreground outline, offset -2 px.
C. Dock presence on phones (G1 rule 1, D0 levels, C12, C6/C7, C9, C16, A2/A3/A8/A16, D3a, D4a, D5a, D6a, B5a):
   the dock shows only on top-level pages: /wallet, /dashboard (until 958 redirects it), /marketplace, the
   Market-level pages the boards draw with the dock (verify /my-offers on C10 and /dashboard/listing on C17 and
   /prices* on C17), /trade-history, /earnings, /reseller, /dashboard/affiliate, /dashboard/affiliate/tiers,
   signed-in /loyalty-programme (B group), /profile. No dock on level-2 and task pages: /trade/[id],
   /trade-history/[id], /offer/*, /profile/[userId], /settings, /notifications, /my-tickets*, /report,
   /dashboard/price-alerts, /wallet/manage|deposit|withdraw/*. The island shows at 768+ on every signed-in
   customer page.
D. Active item (card 968 rules, used by the island everywhere and by the dock where it shows), one at most:
   Wallet: /wallet and children, /dashboard exactly. Market: /marketplace, /offer/[id], /offer/create,
   /my-offers, /dashboard/listing, /profile/[userId], /prices*. Trades: /trade-history(/*), /trade/[id].
   Earnings: /earnings, /reseller, /dashboard/resell, /dashboard/affiliate(/tiers), /offer/resell (C16: "the
   island marks Earnings"), signed-in /loyalty-programme. Account: /profile exactly, /settings, /notifications,
   /my-tickets(/*), /report, /dashboard/price-alerts.
E. No balance in the customer band or phone header (no board draws one); BalanceStrip and HeaderBalance are not
   rendered on customer pages. No avatar, burger or Support button on customer pages. Staff and public pages
   keep today's header exactly.
F. Phone header on customer pages (C1, B1, A13, B11a, D1a, D3a, D4a, D5a, D6a, C6, C9, C12, C16, A2): no logo,
   balance, burger or avatar. Top-level pages: large title (Wallet, Marketplace, Trades, Earnings, Account) + the
   bell at the right (the bell links to /notifications on phones, D4a). Level-2/3 pages: back control with its
   board label + centred title, no bell. Page-specific actions drawn in those rows (Market filter, Mark all read,
   trade Help and more menu) belong to the page cards (955 to 959) and are not built here.
G. Support chat: no floating button on customer pages at any width (D0, D1a note 6). Chat opens from the
   Account hub "Chat with support" row and the Jump action "Chat with support", through the existing
   open-support-chat window event. The FAB stays on staff and public pages, classes unchanged.
H. Jump palette (G9, D1c frame 3): Ctrl K or "/" from 768 up on customer pages; GO TO and ACTIONS groups, rows
   with sub-labels, query match in bold, selected row in the active pane with an "Enter" key cap, bottom hints
   "↑ ↓ move", "Enter open", "Esc close and return focus", "Also opens with /". Esc returns focus to the trigger.
   The index holds the five destinations, every Account destination and action with synonyms ("support",
   "help", "ticket", "report", "logout" finds Sign out), and role-gated staff entries (Moderator Dashboard,
   Admin Panel, War Room) only for those roles.
I. Account hub on /profile (D1a phone, D1b 768, D1c 1440): SETTINGS (Personal details, Preferences and alerts,
   Security and sign-in, Payment methods, Verification, Price alerts), ACTIVITY (Notifications, Support tickets),
   HELP (Chat with support, Report a problem, Help centre and FAQs), Sign out, then Terms of Service, Privacy
   Notice, Cookie preferences. Plus role-gated staff rows (not drawn; D0 "staff keep their menu"; the card
   requires staff entries for staff only): Moderator Dashboard, Admin Panel, War Room, with data-testids.
J. Overlays: the dock fades out (180 ms --ease-in) and goes inert + aria-hidden while any dialog, sheet, drawer,
   the chat panel or the Jump palette is open, and fades back (200 ms --ease-out); reduced motion: opacity in
   --dur-quick linear. Keyboard open (C5 frame 11): the dock drops 16 px and fades out in 200 ms ease-in,
   inert, and returns on blur in 200 ms ease-out. At 768+ under a modal, island and capsule take .is-covered.
K. Geometry (G2, G15, glass css): dock 64 px tall, bottom = env(safe-area-inset-bottom) + 8 px, side insets 12 px
   (6 px below 360 px), max-width 440 px centred from 441 px, radius 32, cells per glass-v3.1 v3.2.1, "9+" badge
   rule, badge position right:-10px below 375 px. Island 52 px tall. Page clearance on dock pages:
   safe-area + 8 + 64 + 16 px (D1a "page ends 122 px above the bottom" with a 34 px inset).
L. Overlay offset contract with 969: html[data-bottom-bar="visible"] is set only while the dock is visible
   (removed for keyboard open, covered, signed out, >= 768, non-dock pages). globals.css (968 block only)
   defines under it --qt-overlay-bottom: calc(env(safe-area-inset-bottom) + 8px + 64px + 12px) (G15: 118 at a
   34 px inset, 84 in a window). 969's toast container reads var(--qt-overlay-bottom, today's offset). The
   consent banner (which no board draws) sits above the dock with the same variable and steps aside while the
   keyboard is open (html[data-qt-keyboard="open"]); it is never dismissed and never stores a choice.
M. /earnings: protected server route on redesign/906 that redirects to /dashboard/affiliate keeping the query
   string; '/earnings' added to robots DISALLOWED_PATHS.

## Proof method for every visual piece

Render the board source (.dc.html, file:// in headless Chrome via Puppeteer + Bun) and the implemented screen
(local production build) at the same viewport and state. Capture both, put them side by side, and compare
computed styles of the chrome elements (height, width, insets, radius, background layers, border, box-shadow,
font family/size/weight/line-height, colours, gaps, padding, outline). Fix every difference. Keep captures under
/Users/starbird/Workspace/Code/qic/.scratch/968/evidence/<area>/. Puppeteer runs on a temp Chrome profile you
create and delete; never sign in, never handle cookies or passwords; seed signed-in UI only through request
interception and localStorage seeding inside your own test browser. Stop only processes you start.
