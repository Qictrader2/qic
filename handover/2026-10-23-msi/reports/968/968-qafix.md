# 968-qafix: consent banner over the chat sheet

- PR: https://github.com/Qictrader2/Frontend/pull/337 (base redesign/906, not merged)
- Branch 968-qafix-consent-over-chat, head 1a7f29d86352e5f365f459bd7515cb5bad53a590 (3 commits on 28645d0c)
- Worktree /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-qafix-root/frontend (.next deleted)

## Fix
The banner steps aside like the dock while any layer is open (modal dialog, sheet, alertdialog, Jump, chat sheet, Customise dialog, and the non-modal chat popover at 768+): inert, aria-hidden, data-covered, opacity 0, visibility hidden after the fade, z-index 39 under the z-40 scrims. It returns in place and works when the last layer closes. Nothing is stored while hidden. Motion per G1: 180 ms --ease-in out, 200 ms --ease-out back, reduced motion opacity only --dur-quick linear. --qt-overlay-bottom block untouched (banner 12 px above the dock, keyboard step-aside kept).

## Board call (logged per owner rule)
No board draws the consent banner. Used G1 layer budget rules 3, 5, 6 (the open layer is the only live layer; chrome under a scrim is covered and inert; a popover is never over other chrome), G1 motion table, G14 rule 3, I7 968 line. Chose "hidden and inert" over "under the scrim" because a banner under a scrim stays visible but dead, which the requester ruled out, and at 768+ the chat popover has no scrim and overlaps the banner. Rule applies on every surface (non-customer modals and role=dialog popovers such as notifications also hide it while open).

## Files
- src/components/consent/ConsentBanner.tsx (card mounts only while shown; uses the hook)
- src/hooks/use-bottom-bar-state.ts (opt-in includeNonModal, initialFromDom, uncoverSync; defaults keep dock and CustomerHeader unchanged)
- src/app/globals.css (covered rules after the 968 block)
- tests: src/__tests__/components/consent-banner-covered.test.tsx, src/__tests__/hooks/is-chrome-covered.test.ts, src/__tests__/design/consent-covered-css.test.ts
All files are 968's own; nothing outside ownership.

## Tests and gate
- gate.sh --base refs/remotes/origin/redesign/906: PASS (typecheck, lint-no-new-errors, build); receipt 968-qafix-root/.scratch/gate/20261023T081819
- 11 suites, 221 tests pass (new 10 + 21 + 7; ConsentBanner, bottom-bar, support-chat-panel, jump-palette, dialog-shell, support-chat-openers, main-header-customer, dock-scroll-padding-css)
- Fail-without-fix checks: uncoverSync off fails 4 of 10 focus tests (inert-aware focus shim); initialFromDom off fails the first-render test; includeNonModal off fails the popover test.
- Real Chrome 390x844 and 1440x900 (public harness, deleted): every layer hides the banner (opacity 0, hidden, inert, nothing hits it); after close Reject all is hit again; 6 chat options clear at 390; focus back on Customise after Escape, Cancel and X; real Reject all stores deny-all. Evidence: /private/tmp/claude-501/-Users-starbird-Workspace-Code-qic/853e4c8b-e6be-48cd-8930-893197f52192/scratchpad/968qafix-proof/ (measurements-head.json, head-*.png). Scratchpad, so it may be swept.

## Reviews (opus55-high, foreground)
1. NEEDS_FIXES: MEDIUM untested focus net. Fixed (net removed), plus LOW first-render cover and z-39 comment.
2. NEEDS_FIXES: MEDIUM focus lands on body on non-customer pages (legacy dialog has no exit animation). Fixed with uncoverSync and an inert-aware focus test.
3. APPROVE. LOWs left: coveredRef comment says "committed" but holds the last requested value; a close-then-open in one frame uncovers for about one frame; real Chrome never reproduced the focus loss (race fix, shown in jsdom); focus return on the customer surface is checked by reading the code and in jsdom, not in Chrome (needs sign-in).

## Not done / backend
No backend dependency. Not re-tested on the review app (QA does that after merge).
