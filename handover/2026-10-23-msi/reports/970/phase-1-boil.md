# 970 PHASE 1 BOIL report

Ticket 970 (https://trello.com/c/pY8CFeEI), "Customer redesign 906-0a: Daylight colours, motion and shared controls". Native mode, size L, FULL track. Built locally; nothing pushed, no PR, no Trello change.

Verdict: BUILT. The fast gate passes on the final tree, all targeted Jest suites pass, and the local observed proof is captured. The size exceeds the L line limit (section 6). The coordinator decided to continue (option A) on that basis.

## 1. Bases and merge-base confirmation

| Repo | Worktree | Branch | Base | merge-base HEAD base |
| --- | --- | --- | --- | --- |
| frontend | `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root/frontend` | `970-daylight-tokens-controls` | origin/redesign/906 = 04fed798fa32b735dd0b6a56bbf7a77da3f0351a (freshly fetched) | 04fed798 (equal) |
| root | `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root` | `970-daylight-tokens-controls` | redesign/906 = 63ec4ecc0f3d1fffb80fcd446cd8ce77b6bd8ef9 | 63ec4ecc (equal) |

04fed798 is an empty commit on 1ac41624, which is the staging frontend's running release (qictrader-frontend-staging v331 "Deploy 1ac41624"). That makes staging.qictrader.com a valid "before" for unchanged surfaces.

## 2. Commits (local only, not pushed)

| Repo | SHA | Message |
| --- | --- | --- |
| frontend | b1cbfe6e | 970: Daylight tokens, motion and shared controls on the customer surface |
| frontend | b1645747 | 970: Static customer skeletons, steady busy buttons, inset glass focus, settled mark-all-read, metas in place |
| frontend | df28d32e | 970: White bordered cards on /notifications so customer skeletons stay visible |
| frontend | 09b3eb16 | 970: Match the read notification row's left edge to the customer border |
| root | 9604fc0 | 970: Rewrite DESIGN.md for the v5.2 Daylight customer system and point frontend at the 970 build (DESIGN.md + submodule pointer 09b3eb16) |

No commit carries an AI attribution trailer.

## 3. Files changed (frontend origin/redesign/906..09b3eb16, plus root DESIGN.md)

Per-file table: `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/size-split.tsv`.

Product code, 29 files, +1,714 / -651:
- globals.css +454/-363
- notifications/page +84/-35
- my-offers/page +36/-24
- offer/[id]/page +12/-9
- layout +55/-63
- theme-provider +112/-28
- routes.ts +87
- mobile-nav +10/-4
- wallet/page -4
- motion-provider +8
- ui/amount +174
- ui/status-chip +74
- ui/error-state +98
- ui/radio-group +138
- ui/progress +92/-5
- ui/field +63/-2
- ui/button +38/-12
- ui/skeleton +24/-3
- ui/loading-button +16/-2
- ui/spinner +14/-7
- ui/empty-state +11/-6
- ui/checkbox +11/-2
- ui/input +6
- lib/exact-amount +53
- lib/customer-classes +42
- contact-us and bug-report +1/-1 each (import only)
- common/button.tsx deleted (-38, no importers)
- common/input.tsx deleted (-42)

Tests, 15 files, +3,890 / -2:
- design/controls.test.tsx 811
- design/design-md-tokens.test.ts 707
- design/theme-provider-surface.test.tsx 443
- design/reduced-motion-css.test.ts 421
- design/customer-surface.test.ts 327
- design/amount.test.tsx 298
- design/contrast.test.ts 205
- design/error-state.test.tsx 143
- design/notifications-mark-all-read.test.tsx 156
- design/status-chip.test.tsx 138
- design/shared-states.test.tsx 90
- design/customer-classes.test.ts 86
- fixtures/qt-v5.2-tokens.css 60
- TradeParametersSection.test.tsx 1/1: the order assertion now names the input by its new qt-input testid
- admin-merchant-tiers-741.test.tsx 4/1: its id-or-text helper ignores qt-* default testids, and every assertion is kept

Docs, 1 file: root DESIGN.md +293 / -483. It is a full rewrite of 313 lines in 13 sections. Anti-patterns stays as section 11, so the CLAUDE.md pointer still holds, and the contrast exception is section 12.

## 4. Gate receipts

Command: `QIC_GATE_ROOT=/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root /Users/starbird/Workspace/Code/qic/.claude/scripts/gate.sh --base origin/redesign/906`

Final run on 09b3eb16: GATE PASS. Receipts are in `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root/.scratch/gate/20261021T210357/`, and the transcript is `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/gate-run-final.txt`.
- bun-typecheck: ok (3s)
- bun-lint-no-new-errors: ok (36s)
- bun-build: ok (30s)

The backend submodule is empty in this worktree, so the backend steps did not run (no backend change). Earlier passes: b1645747 at `.scratch/gate/20261021T205228`, df28d32e at `.scratch/gate/20261021T210056`.

## 5. Targeted tests

Final run on 09b3eb16: 20 suites, 605 tests, all pass. Log: `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/jest-targeted-final.log`.
- All of `src/__tests__/design` (amount, contrast, controls, customer-classes, customer-surface with 97 routes, design-md-tokens, error-state, notifications-mark-all-read, reduced-motion-css, shared-states, status-chip, theme-provider-surface).
- The at-risk existing tests: slider-class-overrides, sign-out-entry-points-zoqyJwfY, TradeParametersSection, header-live-fx, admin-merchant-tiers-741, active-resells-earned-864, wallet-display-fiat-864, ticket-wallet-manage-fiat-864.
- Builders also ran every existing suite that imports a changed file. Builder C ran the design, components, regression and hooks directories: 283 suites passed and 4 failed with 7 tests. All 7 also fail on an archived base copy, so they existed before this branch:
  - TradingLimitsSection
  - ticket-727
  - merchant-tier-panel-582
  - AdminUserDetailTabs. Its NGN case expects a `<p>` with exactly "₦ 1 500 000.00", but the base component renders "All assets valuation: ₦ 1 500 000.00". The copy comes from base commit e99961b5 (card 930).
- ui-fixes.test.ts reads ../qictrader-backend-rs, which is empty in this worktree, so it cannot run here (environment, not code).

Negative proofs:
- customer-surface fails naming an unclassified route. This is a permanent test, and it also happened live when the uncommitted harness route `/offer/qt-harness-970` was present (`evidence/jest-targeted.log`).
- design-md-tokens fails on --focus #0284C7 and on a missing exception row (permanent in-memory tests).
- Builders mutation-checked contrast and reduced-motion-css.
- The full suite was not run.

## 6. Size against the L limit

The L limit is 3,000 lines and 50 files; the plan estimated 2,300 lines and 34 files. The actual diff exceeds the L line limit:

| Part | Files | Added | Deleted |
| --- | --- | --- | --- |
| Product code | 29 | 1,714 | 651 |
| Tests (incl. fixture) | 15 | 3,890 | 2 |
| Docs (root DESIGN.md) | 1 | 293 | 483 |
| **Total** | **45** | **5,897** | **1,136** |

Product code plus docs is about 3,140 changed lines; with tests it is about 7,000. Reasons:
1. The tests are the ones the plan mandates (steps 12a to 12h). Each pins exact values, both directions of the DESIGN.md comparison, every Button variant × size against today's class strings, the 97-route classification table and the fast-check property.
2. globals.css's 363 deletions are the 24 unreferenced keyframes and their classes, which plan step 6d says to delete.
3. The DESIGN.md rewrite replaces the April document.

Scope did not grow beyond plan steps 1 to 12, and the file count stays under 50. The coordinator chose to continue (A) on 21 Oct.

## 7. Where v5.2 and the plan or approved package differ, and how it was resolved (v5.2 wins)

1. `--font-sans` and `--font-num` use the next/font variables (`var(--font-poppins)`, `var(--font-figtree)`) instead of the literal 'Poppins' and 'Figtree'. next/font serves hashed family names, so the literals would not resolve. This is the only token not copied word for word; the Jest tests and DESIGN.md document it. To make the variables resolve in the html-scoped block, the font variable classes moved from body to html.
2. Steps segment gap stays 0, as qt-v5.css v5.2 `.steps{gap:0}` sets in all 8 shared copies. The approved package board B2 draws 6 px. The v5.2 contrast figure "4.20:1 beside the empty step segment" assumes segments that touch. The 6 px gaps in v5.2 come only from group C's c.css and two inline group-board styles, which are local. A consumer can pass a gap through className.
3. The planner's reduced-motion block is used: it cuts every transition and animation to 0.01 ms. Package decision 3 suggested keeping 150 ms layer fades. The glass layers keep their own 150 ms linear opacity fades through glass v3.2.1's `!important` block, which satisfies the card's "opacity cross-fades capped at 150 ms".
4. Plan step 4d says page-level overrides still win. That is false for same-property utilities, because `customer:` utilities sort after plain utilities. Fix: `withCustomerLook` (lib/customer-classes.ts) drops any `customer:` class whose property the caller's className already sets, so caller overrides win on the customer surface as they do today. Covered by tests.
5. Spinner: plan 7g says "never pulse the label". The label keeps `animate-pulse` on the default surface, so staff and auth pages look the same, and gets `customer:animate-none` on the customer surface. The global reduced-motion block stops the pulse everywhere.
6. Customer skeletons are static, per package decision 1 and the card's "static --surface blocks". The plan's wording only stops the shimmer under reduced motion.
7. ErrorState's Try again is secondary by default, per boards B5 to B7, and primary on /offer/[id], per board B8 and package decision 8.
8. theme-color metas: the head script no longer rewrites them, because pre-hydration writes made React 19 append two more. ThemeProvider sets them to #F8FAFC after hydration on the customer surface. Count stays 2, as on base; the browser chrome shows the server colour until hydration.
9. Tailwind's built-in `ease-out` and `ease-in` read `--ease-out` and `--ease-in`, which qt-v5.css redefines. On the customer surface they therefore become cubic-bezier(.2,0,0,1) and (.4,0,1,1).
10. Toast placement classes keep the pack's phone-frame offsets; 906-0c / 968 sets real safe-area offsets. The ink focus scope class for 906-0c is `.qt-on-ink`.

## 8. Observed proof

The full index is `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/README.md`.

How it was run:
- Local `next build` + `next start` on localhost:3000, with the staging backend behind the same-origin proxy (BACKEND_URL).
- Puppeteer under Bun, on Chrome for Testing with temporary profiles that were deleted afterwards. Only processes this run started were ended.
- No sign-in anywhere. Signed-in sample pages used a fixture session with every /api/v1 and socket request answered locally.
- The harness route `/offer/qt-harness-970` was temporary, never committed, and deleted after each round. Builds: build-2, recheck build-3, and the gate build for recheck2.

Results:
- Unchanged pages (/, /login, /contact-us, /help, /lp, /admin signed-out redirect at 390 and 1440, light and dark) against staging:
  - html class, data-surface (null), tokens and metas are identical.
  - Pixels are identical except the autoplay carousel on / and the /lp badge pulse phase.
- Customer surface:
  - /marketplace has the Daylight tokens (58 equal v5.2), no theme toggle, and no dark first frame.
  - Client navigation /help ↔ /marketplace flips with no wrong-theme frame.
  - Harness controls match the v5.2 values: focus #0369A1 on paper, #0F172A on glass (inset on bands), #0369A1 in .gl-solid, #38BDF8 on ink; fills #0073AD; input border plus 1 px ring.
  - Amount and StatusChip texts and colours are exact, and Figtree loads.
  - Each Try again click sent exactly one request.
  - Reduced motion stops every infinite animation, and the spinner shows "Loading...".

Gaps found and fixed:
- The customer skeleton still shimmered.
- A busy LoadingButton changed colour on hover and press.
- Glass band focus offset was +2 px instead of the inset -2 px.
- Mark All Read showed during loading and error.
- There were 4 theme-color metas instead of 2.
- /notifications skeletons were invisible on surface-coloured cards; fixed with customer white bordered cards per B7.

The rechecks in `evidence/recheck/` and `evidence/recheck2/` confirm the fixes. The final one-token change in 09b3eb16 (read rows' left edge in --border) is covered by Jest and the gate build. It was not re-rendered in a browser; recheck2 had already shown the other three edges at rgb(226,232,240) on the df28d32e build.

Comparison with the v5.2 boards and the approved package exports: the gaps left are listed in section 9.

## 9. Risks and items not closed locally

1. Signed-in renders are deferred to QA on the review app: token values in a Radix dialog portal, AC1 first paint on /dashboard, /settings and /offer/{id} signed in, staff pages, /loyalty-programme signed in, control states on /offer/create and /settings, and real-data states. Each is a deferred-to-QA row in `reports/acceptance-map.md`.
2. Public and staff pages under reduced motion: the global block (plan 6b, Risks) stops /lp's two endless pulses, and other decorative loops, only for visitors who ask for reduced motion. The coordinator confirmed this is intended. With reduced motion off, / and /lp match staging.
3. Pre-existing on base, not 970's doing; evidence in `evidence/preexisting-findings.md`:
   - /offer/{id} shows "Price (Market) 0.00 ZAR/USDT" when the rate is missing (use-offer-detail-page.ts:63 plus formatCurrencyByCode).
   - /notifications throws React #130 on an unknown notification type (icon map unchanged).
   - The signed-in header is under width pressure at 1440; its source is unchanged by 970, base was not measured, and QA should compare signed in.
4. The legacy alias remap changes every customer page that uses those tokens. Example: `bg-background-light` (197 uses) now computes to --surface, the well colour. That keeps today's panel-on-page relationship but is a visible palette shift on pages not yet redesigned. The group cards 906-A to E own page-level restyles.
5. Not captured locally: forced-colours and reduced-transparency glass fallbacks (Jest asserts the rules exist), Safari, Firefox and real phones.
6. /notifications cards are white with a 1 px --border edge on the customer surface (board B7). Unread rows keep base `border-l-primary border-l-4`, which is navy there because --primary maps to --foreground. Read rows get a matching --border left edge (fixed in 09b3eb16 after recheck2).

## 10. Size beyond L

See section 6. Nothing else pushes the size beyond L. No backend change, no new npm dependency (Figtree via next/font/google only), no added code comments.
