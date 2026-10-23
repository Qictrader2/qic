# 970 Phase 2 TEMPER fixes

Frontend worktree: `.scratch/906-redesign/970-root/frontend` (branch 970-daylight-tokens-controls)
Frontend commit: 419b721bfb41b0e534fd77f69491ea025f05edc0 (parent 09b3eb16)
Root commit: 93f349351c14d59ba66ce2831a83eb2ae4eadd85 (parent 9604fc0), frontend pointer = 419b721b
Nothing pushed.

## 1. MEDIUM: RadioItem and the current Step used hex fallbacks with no customer: scope. Fixed.

- `src/components/ui/radio-group.tsx:8-12`: the v5.2 looks moved into `CUSTOMER_RADIO_LABEL_CLASSES` and
  `CUSTOMER_RADIO_CLASSES`. Every token is `customer:` scoped and uses theme keys: `border-input-border`, `bg-card`,
  `checked:bg-action-pressed`, `after:bg-action-foreground`, `focus-visible:outline-focus`, `disabled:bg-surface`,
  `checked:disabled:bg-border`, `checked:disabled:after:bg-text-secondary`, `after:duration-press`, `after:ease-qt-out`.
  No `#` value and no `var(..., #hex)` fallback remain.
- `radio-group.tsx:114-115`: the label keeps its 44 px row (`min-h-11 min-w-11`) on every surface. Off the surface a
  disabled row now dims with `has-[:disabled]:opacity-50`, the same neutral treatment Checkbox uses. On the surface it gets
  `customer:has-[:disabled]:opacity-100 customer:has-[:disabled]:text-text-secondary`.
- `radio-group.tsx:130-131`: the neutral off-surface default is a native radio, `size-4 shrink-0 cursor-[inherit]
  accent-brand-blue`. `withCustomerLook` drops any customer token that a caller's className overrides, as Button,
  Checkbox and Input already do.
- Small extra fix on the same line: the dot transition is now `after:transition-[scale,opacity]`. Tailwind 4's `scale-*`
  writes the `scale` property, so the earlier `[transform,opacity]` would have made the dot jump instead of animating.
- `src/components/ui/progress.tsx:57`: the current step bar is now `bg-brand-blue customer:bg-action-pressed`. Off the
  surface the Steps use only root tokens (success, brand-blue, border).
- Guards in `src/__tests__/design/controls.test.tsx`:
  - :452 checks that no radio input or label class string matches a hex colour, that no v5.2 token sits outside
    `customer:`, that the named customer tokens are present, and that the neutral default is present.
  - :634 checks every Step element (li, bar, label, icon) for hex colours and for an unscoped `action-pressed`.
  - :441 and :614 were updated for the new scoped classes.
- Red check: the four tests failed against the HEAD 09b3eb16 components and pass now.

## 2. LOW: DESIGN.md said the checkbox uses no-min-touch. Fixed.

- Root `DESIGN.md:173`: the Checkbox row now says
  `customer:min-h-0 and customer:min-w-0, and a centred ::before extender`, which matches `ui/checkbox.tsx:11`.
- `DESIGN.md:162-164`: the intro now names both opt-outs (`no-min-touch`, or `customer:min-h-0 customer:min-w-0` inside a
  component).
- Guard: `design-md-tokens.test.ts` stays green (17/17). Its hit-area check reads the Target, Visual and Hit area
  columns and the "at least 44 x 44 px" text, and none of those changed.

## 3. LOW: the LoadingButton lost the button's gap-2 while loading with no loadingText. Fixed.

- `src/components/ui/loading-button.tsx:43`: the children cell is now
  `col-start-1 row-start-1 inline-flex items-center justify-center gap-2 motion-reduce:invisible`. A child icon and its
  text keep the 8 px gap on every surface, and the overlap with the reduced-motion "Loading..." stays the same.
- Guard: `controls.test.tsx:776` renders `<LoadingButton isLoading><svg/>Save</LoadingButton>` and checks that the
  icon's cell has `inline-flex`, `items-center`, `gap-2` and `motion-reduce:invisible` and no `customer:` token. It
  failed on the old code and passes now.

## 4. LOW: qt-theme-swap froze transitions on every stored-theme toggle. Fixed.

- `src/components/providers/theme-provider.tsx:105-121`: the swap class now comes from its own layout effect keyed only on
  `[onCustomerSurface]`. It is declared before the apply effect, so it is set in the same commit, before the classes
  change. The apply effect at :123-149 still runs on `[onCustomerSurface, storedResolvedTheme]` but no longer touches
  qt-theme-swap. A public-page light/dark toggle therefore keeps its 300 ms fade.
- Guards in `src/__tests__/design/theme-provider-surface.test.tsx`:
  - :197 toggles the stored theme on /help (light, then dark), checks the html class flips, and checks qt-theme-swap is
    never added. This failed on the old code and passes now.
  - :215 goes /wallet to /help (swap added, removed after two frames), then /help to /markets (no swap).
  - The existing :174 test still covers crossing onto the surface.

## Not fixed (note only, as instructed)

LOW 5 (design-md-tokens.test.ts reads ../DESIGN.md) and LOW 6 (/notifications additions match B6/B7).

## Gate and tests

- `bun run test -- src/__tests__/design`: 12 suites, 459 tests, all passed.
- Other suites that import the touched files (transaction-history-859, ticket-556 x5, ticket-818-ac5, ticket-401):
  8 suites, 46 tests, all passed.
- Fast gate:
  `QIC_GATE_ROOT=.../970-root gate.sh --base origin/redesign/906`: GATE PASS (bun-typecheck, bun-lint-no-new-errors,
  bun-build). Receipt: `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root/.scratch/gate/20261021T211525`.
  `.next` was removed before the run.
- No code comments in added lines. No stash, reset or clean was used.
