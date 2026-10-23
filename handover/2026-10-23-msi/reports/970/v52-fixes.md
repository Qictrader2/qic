# Ticket 970: v5.2 board-check fixes

PR https://github.com/Qictrader2/Frontend/pull/304 into `redesign/906`. Fixes the gaps in `v52-board-check.md`.
Not merged, Trello not touched, no force-push.

## Commits (fast-forward pushes)

| Repo | Branch | Before | After |
|---|---|---|---|
| Frontend | 970-daylight-tokens-controls | 419b721b | 0172e59a |
| Root (qic) | 970-daylight-tokens-controls | 93f3493 | 08d4b56 (DESIGN.md, frontend pointer at 0172e59a) |

Message on both: "970: Match the v5.2 boards for glass field focus, outline label and steps".

## Gaps

1. Row 20, blocking, glass field focus. Fixed. `frontend/src/app/globals.css:1009`: the first-child
   `box-shadow:none !important` rule's `:not()` list now also exempts `[data-testid=qt-input],input,textarea,select`.
   A field that is the first child of a `.gl-sheet`, `.gl-dialog`, `.gl-popover` or `.gl-fade` body (including `.gl-solid`)
   keeps its 1 px focus ring and 1 px error ring. New test `src/__tests__/design/glass-field-focus.test.tsx`: it extracts
   the real rule from globals.css, prefixes the customer scope, and uses jsdom `matches` to check that the React Input
   (valid and `aria-invalid`), a textarea and a select as first child do not match in sheet, dialog, popover and
   dialog+solid, while a plain div still does (control case). It also asserts the Input's focus and error ring classes
   and the glass ring colour overrides (`--foreground` in `.gl`, `--focus` in `.gl-solid` and `.gl-plate`). On the old
   rule the test fails, because the rule then lacks `[data-testid=qt-input]`.
2. Rows 3 and 53, outline label. Fixed to the G0 board. `button.tsx` `outline`, customer classes only: label
   `--action-pressed` at rest and hover, hover fill `--surface`, pressed fill `--surface` plus the base scale .98.
   Pressed moved off `--surface-hover` because `--action-pressed` on it is 4.20:1, below 4.5. On `--card` it is 5.18 and
   on `--surface` 4.73, and `contrast.test.ts` already asserts both. `secondary` keeps the `--foreground` label
   (qt-v5.css `.btn-secondary`). DESIGN.md button table now has one row each for `outline` and `secondary`, so section 2
   ("links and outline labels") and section 7 agree.
3. Row 14, Steps. Fixed to G0 and G4. `progress.tsx`: list `gap-0` became `gap-1.5` (6 px); the done-step check icon and
   its flex wrapper are gone, so labels are text only. There is no "Step n of N" text, matching the boards. State stays
   readable without colour, as the boards show it: the current label is 600 against 500 for the rest, plus
   `aria-current="step"`, plus a visually hidden ", completed" suffix on done steps. `controls.test.tsx` covers the gap,
   the missing icon and the suffix. DESIGN.md stepper item updated.
4. No other in-scope minor gaps: the report lists exactly three minor gaps (rows 3, 14, 53), all fixed above.

Default-surface class strings are unchanged. Steps is new in this branch, and every button change is under `customer:`.

## Verification

1. `bun run test -- src/__tests__/design`: 13 suites, 484 tests, all passed.
2. Gate: `QIC_GATE_ROOT=.../970-root .claude/scripts/gate.sh --base origin/redesign/906` reported GATE PASS
   (typecheck, lint-no-new-errors, build). Receipts:
   `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root/.scratch/gate/20261021T213637`.
3. Full frontend Jest run: 392 suites, 5099 tests, with 7 tests failing in 5 suites (TradingLimitsSection,
   AdminUserDetailTabs, merchant-tier-panel-582, ticket-727-war-room-tooltips, ui-fixes). The same 7 fail with the HEAD
   419b721b versions of progress.tsx, button.tsx and globals.css, so this change did not cause them. ui-fixes reads
   a backend file that is missing from the empty backend submodule in this worktree.
4. Not re-rendered in a browser: the fix is proved by selector matching in jsdom, not by a fresh screenshot.
