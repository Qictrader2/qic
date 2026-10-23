# 969 PHASE 2 TEMPER fixes

Frontend worktree `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root/frontend`, branch `969-dialogs-toasts`. Local only: nothing pushed, no PR, no Trello change, no self-grading.

## 1. Commits and integration

| Step | SHA | Notes |
| --- | --- | --- |
| Review HEAD | 746ea018 | on origin/redesign/906 f44c50dc |
| Fix commit | c85b20da | "969: Address review: held toasts, release confirm, chained focus and fee refresh" |
| Safety ref | `969-dialogs-toasts-pre-rebase-c85b20da` | local branch at c85b20da |
| Integrated HEAD | **c54ea8fe** | the 10 commits rebased onto origin/redesign/906 fe023ad2 (968 merged). No conflicts. The 969 delta is byte-identical before and after (`git diff f44c50dc..c85b20da` equals `git diff fe023ad2..c54ea8fe`), and no file is touched by both 968 and 969 |

## 2. Findings

| # | Verdict | Change (file:line on c54ea8fe) | Test |
| --- | --- | --- | --- |
| 1 | Fixed | Toasts already showing when a customer dialog opens are suspended: the container draws nothing while a customer panel is mounted, and each toast stops its timer, clears its hover/focus pause reasons, and resumes with the time it had left. `ui/toast.tsx:112-184` (BannerToast `suspended`), container reads `useCustomerDialogMounted` | toast-v52: a toast shown before the dialog opens is gone while the dialog is open, its timer does not run, and it returns and expires after the remaining time |
| 2 | Fixed | A mounted registry is kept apart from the open stack: `ui/dialog-stack.ts:65-96`, registered from an unmount-only layout effect in `ui/dialog.tsx:271-283`. Held toasts are released only when no customer panel is mounted (`store/ui-store.ts:183`) | dialog-shell (the old "released as soon as that dialog starts closing" test is rewritten to the mounted rule with a real Radix exit animation: the panel stays mounted after open=false, nothing is released, then release follows animationend and unmount) |
| 3 | Fixed | Customer surface: an inline FieldError (role=alert) under the Available/Fee line with the exact string, shown immediately and never through the toast store (`features/wallet/WithdrawModal.tsx:829`, `:1592`). It clears on submit, edits and close. The default surface keeps `toast.info` | new `__tests__/regression/ticket-969-withdraw-fee-refresh.test.tsx` (8 tests, real ui-store, ToastContainer and useWithdraw; only walletApi.withdraw is mocked). Mutation: putting the old toast.info back fails 6 of them |
| 4 | Fixed | `ui/confirm-dialog.tsx:23-24, 80-81, 100`: optional `size` (default sm) and `stretchPrimary` (default false), customer path only. `features/trade/TradeSidebar.tsx:433-434` passes `size="lg" stretchPrimary` on the release confirm (copy unchanged) | new `__tests__/components/ticket-969-confirm-dialog-geometry.test.tsx` (8). Browser measurement against the rendered boards: release at 1440 is 520 wide, row 470, cancel 92.14, primary 367.86 with grow 1; at 390 the primary is 245.86 with grow 1. Default "Delete this offer?" is 440 wide with flex-end, buttons 115.44/125.33. These equal G16b and tb-confirm. Evidence: `evidence/phase-2/confirm-geometry/` |
| 5 | Fixed | Each panel carries `data-dialog-stack-id` (`ui/dialog.tsx:394`). The mounted entries store each panel's opener. `resolveOpener`/`handOffFocus` (`ui/dialog-stack.ts:105-127`) inherit a closing panel's opener, recursively. Customer surface only; returnFocusRef still wins | dialog-shell: the old "cannot find the first opener" test is now a passing chained-return test, plus a UserSearch-to-CreateDirectOffer-shaped test |
| 7 | Fixed | A customer panel that is not dismissible renders Close as `disabled` with `aria-disabled="true"` (`ui/dialog.tsx:372-379`). hideCloseButton still renders none, and the default path is unchanged. The 819 notice passes `hideCloseButton` (`features/wallet/DepositModal.tsx:605`) | dialog-shell (disabled Close does not close the dialog), `deposit-modal-819-minimum-notice` (no Close at all on the customer surface) |
| 8 | Fixed | New `features/wallet/dialog-panel-classes.ts` (base, `DEFAULT_SURFACE_PANEL_CLASSES` md, `COMPACT_SURFACE_PANEL_CLASSES` sm for ManageWalletModal), imported by Withdraw, Transfer, Lock, Deposit, TransactionStatus and ManageWallet. `INVALID_FIELD_RING_CLASS` is exported once from `lib/customer-field-classes.ts` and imported by EditTradeLimit and EditOfferMargin. The rendered strings are byte-identical | ticket-645 static and runtime suites pass unchanged |
| 9 | Fixed | `role="region"` on the labelled viewport (`ui/toast.tsx:373`) | toast-v52 |
| 6, 10 | Not in scope | 6 is handled at merge with --squash. 10 is a QA watch item (OfferPreviewModal width) | |

A correction to the premise of finding 2: aria-hidden 1.2.6 `hideOthers` explicitly skips `[aria-live]` nodes (`node_modules/aria-hidden/dist/es2015/index.js`, issue #10), so the live regions were never hidden. What is real while a panel closes is body `pointer-events: none` and the still-mounted aria-modal panel. The fix holds regardless, and the test asserts the release happens after unmount.

## 3. Board alignment requested by the coordinator during the round

| Item | Verdict | Change and test |
| --- | --- | --- |
| a. Sheet below 768, dialog from 768 (G1 Q7, G3, G10) | Fixed | `hooks/use-customer-surface.ts:7` is now `(min-width: 768px)`. New `ticket-969-board-alignment.test.tsx`: 767 gives a sheet, 768 a dialog, ConfirmDialog follows, and the default surface has no data-layout. 640 test titles renamed in dialog-shell and kyc-dialog-shell |
| b. Deposit glass, address on an opaque plate | Already matched | withdraw-state-survival:561-573 |
| c. Escape ignored while a commit is in flight (G1 s10) | Already matched | Withdraw, Transfer (new test), Lock, ConfirmDialog, TransactionStatus and KYC. Offer edit and CreateDirectOffer still allow Escape because they are not commit steps (G1 s5) |
| d. One toast at a time, at least 24px from the edge | One at a time and 24px already matched. Desktop position fixed per the coordinator's decision | `ui/toast.tsx:379` customer branch from md (768): right 32, bottom 32, width 360, not centred (shared board G7). Phones keep `--qt-overlay-bottom`. toast-v52:102 (exact default string, phone and md classes). Board-vs-board recorded: D3 has right 24, bottom 24, width 380, and B6 is another variant; the shared board wins for a shared component |
| e. Referral-copied toast per G7 | Fixed | New `lib/referral-toast-copy.ts`: title "Referral link copied", line "Paste it anywhere you share links." Used by AffiliateDashboard and ReferralSection (through a new optional `successDescription` on CopyButton). ticket-969-silent-toasts updated |
| f. Lifetime | Kept at 6s, recorded | G1 and B18 say 6s; D3 and E5 say 4s. DEFAULT_DURATION stays 6000 |

## 4. Gate and tests

- Before the rebase (fix tree on c85b20da): targeted plus importing suites 118 passed / 1,100 tests passed. GATE PASS, receipts `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root/.scratch/gate/20261022T091749`.
- Integrated c54ea8fe: 135 suites / 1,742 tests passed (the gate list ticket-969, dialog-shell, toast-v52, 769, 645, 776, 839, 864, 9dl9xx7a, drcwuijn, zoqyJwfY and withdraw-fiat-864, plus every suite importing a touched file, plus 968's navigation, ConsentBanner, support-chat, customer-surface, theme-provider-surface and trades-badge suites). `QIC_GATE_ROOT=/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root /Users/starbird/Workspace/Code/qic/.claude/scripts/gate.sh --base origin/redesign/906` (base fe023ad2): GATE PASS (bun-typecheck, bun-lint-no-new-errors, bun-build), receipts `.../969-root/.scratch/gate/20261022T092017`.
- No code comments in added lines (scanned). The full jest suite was not run.

## 5. Browser check on integrated c54ea8fe

Fixture auth, every /api and socket call answered from fixtures, temp Chrome profile, dev server stopped, temp route removed, `git status` empty afterwards. Evidence: `/Users/starbird/Workspace/Code/qic/.scratch/969/evidence/phase-2/toast-dock/` (measurements.json and four screenshots).

- 390x844 on /wallet with `data-bottom-bar="visible"`: toast bottom 760, dock top 772, gap **12px**, side insets 12/12, width 366. Pass.
- 1440x900: right 32, bottom 32, width 360, centre x 1228 (not centred). Pass.
- Opening Deposit at 390 hides the toast, and it returns at the same place after Escape. Pass.
- The dev overlay badge and support widget in the 390 shot come from the dev server and fixtures.
