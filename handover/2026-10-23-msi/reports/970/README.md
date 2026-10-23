# Ticket 970 local observed proof (BOIL)

Build: frontend worktree at commit b1cbfe6e (970: Daylight tokens, motion and shared controls) plus the temporary harness
route `/offer/qt-harness-970`, built with `bun run build` (log `build-2.log`) and served by `bun run start -p 3000` with
BACKEND_URL and NEXT_PUBLIC_WS_URL pointed at the staging backend. The harness file, server and browser profiles were
removed afterwards; `git status --short` in the worktree is empty.

Safety: no sign-in anywhere. Signed-out runs let only GET /api/v1/* through the local proxy (public reads). Fake-auth
runs (`/my-offers`, `/notifications`) seeded a fixture `auth-storage` user (fixture970, fixture-970@example.test) and
answered every /api/* and /socket.io request from local fixtures; WebSocket and EventSource were stubbed in-page, and
every request to a host other than localhost:3000 was aborted (only connect.facebook.net and calendly.com were tried).

## Top level

- `build-1.log`: earlier orchestrator build of the 970 tree (not from this run).
- `build-2.log`: this run's `bun run build`, includes the `/offer/qt-harness-970` static route; exit 0.
- `server.log`: `next start` output of the server used for every "after" and customer capture.
- `keyframe-grep.txt`: earlier slice-B reference grep for the 24 removed keyframes (not from this run).
- `qt-v5-fixture-check.txt`: earlier slice-B sha256 check of qt-v5.css v5.2 against the token fixture (not from this run).

## unchanged/ (AC1 staff and public pages unchanged)

- `before/*.png|json`: staging.qictrader.com captures (base tree) of /, /login, /contact-us, /help, /admin (signed out), /lp, /marketplace at 390 and 1440, stored light and dark, plus reduced motion for / and /lp.
- `after/*.png|json`: the same captures from the local 970 build.
- `diff/*.png`: pixel diff per capture, red = channel delta above 24.
- `compare.json`: per capture: theme class, data-surface, token, theme-color, body colour, toggle, animation and pixel-diff comparison.
- `compare.md`: one row per capture, same or different and why. All public pages same; /marketplace different by design (now customer).

## customer/ (AC1 to AC4 on the local build)

- `a-marketplace.json`: /marketplace signed out, stored dark and light, both viewports: html class light, data-surface customer, 4 theme-color metas (#F8FAFC), all 58 v5.2 tokens, no theme toggle in page or drawer, ui-storage still 'dark'.
- `a-marketplace-{390,1440}-{dark,light}.png`: full-page Daylight captures with stored dark and stored light.
- `a-marketplace-390-{dark,light}-drawer.png`: open mobile drawer on /marketplace, no theme toggle.
- `a-token-check.txt`: the 58 computed tokens equal qt-v5.2-tokens.css after normalising serialisation (#fff, hex alpha, .1s).
- `g-drawer-toggle-control.json`, `g-drawer-390-{help,marketplace}-dark.png`: positive control, /help drawer shows "Toggle theme menu" in dark, /marketplace drawer shows none.
- `b-firstpaint.json`: stored dark, CPU 4x, /marketplace: screenshots at 0/50/100/200/400 ms, every screencast frame sampled, rAF log of body colour from the first frame (no dark body frame).
- `b-firstpaint-{390,1440}-{0,50,100,200,400}ms.png`: the timed screenshots (all Daylight).
- `b-screencast-{390,1440}/*.png`: every screencast frame; frame 000 is #121212.
- `b-control/control.json` and PNGs: control showing the same #121212 first screencast frame for a plain white data: URL and /help stored light, so it is a headless pre-paint frame, not the page.
- `c-clientnav.json`, `c-nav-*.png`: stored dark, real link clicks /help -> /marketplace -> /help in one document; class, data-surface, body colour per hop and per rAF frame during each hop (no wrong-theme frame).
- `d-states.json`: /my-offers and /notifications (fake auth, full fixtures) and /offer/{id} (signed out) in loading (request held), error (500), empty (empty list, 404 for the offer), data, and retry; qt-* testids with computed styles, request log, clicks counted.
- `d-<page>-<vp>-<state>.png`: full-page capture of each state; `-retry-after.png` after one Try again click.
- `_summary-d.json`: short form of d-states.json (Try again caused exactly 1 request in all 6 runs, data rendered).
- `e-harness.json`: harness computed styles per control (rest, hover, pressed, keyboard focus), hit-area probes, Amount text and colours, chips, steps, progress, checkbox tick, radio dot, field error.
- `e-harness-{390,1440}.png`: full harness.
- `e-focus-<vp>-<id>.png`: crop of each control while keyboard-focused (paper, glass, glass-solid, ink).
- `_summary-e.json`: short form of e-harness.json.
- `f-motion.json`: reduced motion off and on for the harness, /marketplace, / and /lp at both viewports: getAnimations after 1500 ms, skeleton and spinner animation, qt-motion-only and qt-reduced-motion-only display, button and input transitions.
- `f-{home,lp}-{390,1440}-{motion,rm}.png`: landing pages with and without reduced motion.

## tools/ (in ../tools)

- `capture.ts`: before/after capture of unchanged pages (pre-existing).
- `compare-unchanged.ts`: pixel and JSON comparison writing unchanged/compare.json and diff/.
- `crop.ts`: side-by-side crop helper used for visual review.
- `customer.ts`: tasks a to h above (`--tasks=a,b,c,d,e,f,g`).
- `firstframe-control.ts`: the b-control run.
- `fixtures/offer.json`: one public staging offer (GET /api/v1/offers) used as the shape for fixture offers.

## recheck/ (re-verification at frontend commit b1645747)

Same safety rules; harness recreated at the same path, built (`recheck/build-3.log`), served (`recheck/server.log`), then
deleted with the server stopped and `git status --short` empty. Script: `../tools/recheck.ts`.

- `recheck.json`: all six items. item1 skeleton animations/background per qt-skeleton; item2 LoadingButton busy rest/hover/down; item3 Tab focus outline on paper, .gl band, .gl-solid, ink; item4 Mark All Read presence in loading/error/data; item5 theme-color metas on /marketplace and /login with the staging values; item6 full input focus box-shadow and border on paper, error, .gl, .gl-solid.
- `1-harness-{390,1440}.png`: harness with static skeletons.
- `1-{my-offers,notifications}-{390,1440}-loading.png`: loading states with the request held.
- `4-notifications-{390,1440}-{error,data}.png`: error state without Mark All Read; data state with it back.
- `unchanged/before/`: staging captures of /login and /contact-us copied from ../unchanged/before.
- `unchanged/after/`: local captures at b1645747 (all pages captured; only /login and /contact-us compared).
- `unchanged/compare.json`, `unchanged/diff/`: /login and /contact-us at 390 and 1440, light and dark: 0% pixel difference, tokens, class, metas identical.

## recheck2/ (/notifications cards at frontend commit df28d32e)

Existing gate build in `.next` (BUILD_ID 21:02, after the df28d32e commit); no rebuild, no harness. Server log
`recheck2/server.log`; script `../tools/recheck2.ts`; fake auth with full interception as before.

- `recheck.json`: per viewport (390, 1440) and state (loading, empty, data): every `bg-background-light` card's background, all four borders, skeleton background and animation count, unread class, plus --primary and --card on the page.
- `notifications-{390,1440}-{loading,empty,data}.png`: full-page captures; cards white with --border edges, static skeletons visible, matching B7.
