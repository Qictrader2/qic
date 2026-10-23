# 968 Phase 2 TEMPER fixes

Worktree: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-root/frontend, branch 968-bottom-bar-header-overlays.
Before: 2fc674d3. After: 8a1d7859 "968: Address review: dock hooks only where the dock shows, header hydration and back target". Not pushed. No PR and no Trello change.

## 1. Finding 1 (MEDIUM): fixed

- src/components/navigation/BottomBar.tsx:36-56: the outer `BottomBar` computes `showDock = hydrated && isDockPage(pathname, { signedIn }) && largeShell !== true` (line 41) with the SSR-safe `useMediaQuery(LARGE_SHELL_QUERY)`. While the dock is hidden, its effect removes `data-bottom-bar` and `data-qt-keyboard`, which also unsets `--qt-overlay-bottom` (globals.css sets it only under `html[data-bottom-bar="visible"]`). It returns null unless the dock shows.
- BottomBar.tsx:60 onward: the inner `DockView` is now the only place that calls useChromeCovered, useKeyboardOpen and `useTradesBadge(true)`, sets the two attributes and removes them on unmount. Markup, classes, testids and aria are unchanged.
- src/hooks/use-trades-badge.ts:41: `useTradesBadge(enabled = true)`. When it is false it sends no request and returns `{ count: null }`. Island.tsx:18-19 passes `useMediaQuery(LARGE_SHELL_QUERY) === true`, so the Island fetches only at 768 px and up, where it is visible. The query key is still shared.
- Behaviour note: `data-bottom-bar` is now set even when matchMedia is missing (null). That case exists only in jsdom, because a real browser always answers after hydration.
- Tests: bottom-bar.test.tsx, "starts nothing where the dock is not shown". It shows that no MutationObserver starts, no focusin/focusout or visualViewport listener is added and no fetch is sent when signed out, on /admin, /moderator, /, /login, /settings and at desktop width. It shows they do start on /wallet at phone width, and that both attributes are cleared after a route change to a staff page, after sign-out and after a switch to desktop width. Two more tests are in use-trades-badge.test.tsx (enabled false: no fetch, count null). The new island.test.tsx (2 tests) shows no fetch below 768 px and a fetch at 768 px.
- Red check: with the old BottomBar.tsx temporarily in place, 9 of the new tests failed. The new file was restored afterwards and diff-checked.

## 2. Finding 2 (LOW): fixed

- src/app/globals.css:1161: deleted the dead `:where(.gdock.m3 .dock-item.is-focus){outline:2px solid var(--action-pressed);...}` rule. The `--foreground` focus-visible rule at about line 779 is unchanged. No test referenced it.

## 3. Finding 3 (LOW): fixed

- New src/hooks/use-auth-hydrated.ts: `useSyncExternalStore` over `useAuthStore.persist.onFinishHydration` and `hasHydrated()`, with a server snapshot of false (the same pattern MarkupPresetControl uses, which was left as it is).
- src/components/common/mainHeader.tsx:40 and :59: `CustomerHeader` renders only when `authHydrated && shouldShowCustomerChrome(...)`. Until hydration finishes, the current header renders.
- Tests (main-header-customer.test.tsx): (1) signed in on a customer path but not hydrated, the current header renders. After the hydration listener fires, CustomerHeader renders. This test fails without the gate. (2) renderToString, then hydrateRoot with the store signed in: no recoverable error, and no customer band on the first client commit. The worker could not reproduce a real mismatch on the old code under zustand 5 in jsdom, so test 1 is what enforces the gate.

## 4. Finding 5 (LOW): fixed

- New src/lib/navigation/in-app-history.ts: `hasInAppHistory()` is true after a client-side pathname change in this document, or when `document.referrer` has the same origin. The URL parse is wrapped in try/catch and returns false on failure. New src/components/navigation/InAppHistoryTracker.tsx renders null and has one pathname effect. It is mounted once in src/app/layout.tsx after `<BottomBar />`.
- src/components/navigation/PhoneHeader.tsx:248: Back calls `router.back()` only when `hasInAppHistory()` is true. Otherwise it calls `router.push(back.href)`, which is the parent route (for example /notifications to /profile, "back to Account").
- Tests: in-app-history.test.tsx (9). Covered: external or empty referrer pushes the parent href; an in-app change goes back; a same-origin referrer goes back; an unparsable referrer pushes. The key two fail with the old `history.length > 1` condition.

## 5. Not fixed, as instructed

Findings 4 and 6 are recorded and out of scope.

## 6. Verification at 8a1d7859 (working tree identical before commit)

- `bun run test -- src/__tests__/navigation src/__tests__/design` plus every importer of the touched files: 33 suites, 1,236 tests pass.
- Named importers run explicitly (header-live-fx, support-chat-placement, header-fiat-864, both sign-out zoqyJwfY suites, use-trades-badge, staging-gate, ticket-nav-affiliate-active, ticket-nav-wallet-dup): 9 suites, 95 tests pass.
- ConsentBanner alone: 1 suite, 7 tests pass.
- ui-fixes.test.ts is not run because of the environment: the backend file it needs is missing in this frontend-only worktree, and it imports nothing that changed.
- Added lines scanned: no code comments.
- `QIC_GATE_ROOT=.../968-root .claude/scripts/gate.sh --base origin/redesign/906`: GATE PASS (typecheck 3 s, lint with no new errors 56 s, build 35 s). Receipt: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-root/.scratch/gate/20261022T090946.

## 7. Seen, not changed (outside the findings)

- CustomerHeader.tsx:24 still runs useChromeCovered at every width on signed-in customer pages, for the band's `covered` state. It is limited to customer pages, so it was left alone.
