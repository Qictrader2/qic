# 969 PHASE 1 BOIL report

Ticket 969 (https://trello.com/c/DqJFKFVH), "Customer redesign 906-0b: one dialog style and working notifications". Native mode, size L. Built locally. Nothing pushed, no PR, no Trello change, no self-review.

Verdict: BUILT. The fast gate passes on the final commit 746ea018, 85 targeted Jest suites pass, and the local production builds were compared with the v5.2 boards and exercised in a real browser. What remains is staging-only proof (signed-in accounts, staging money, screen readers) and seven board differences recorded for the owner (section 7).

## 1. Base and merge-base

| Repo | Worktree | Branch | Base | merge-base HEAD base |
| --- | --- | --- | --- | --- |
| frontend | `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root/frontend` | `969-dialogs-toasts` | origin/redesign/906 = f44c50dcf55162cf2f5a14cf689c91675312e3b3 (970 merged) | f44c50dc (equal) |

No root or backend change. DESIGN.md was not changed; the plan does not require it.

## 2. Commits (local only)

| SHA | Message |
| --- | --- |
| 49cee000 | 969: Shared v5.2 dialog shell, dialog stack and one toast with routing while a dialog is open |
| 26746d6a | 969: Count only open dialogs in the stack, one error look in dialogs, April overlay for staff confirms |
| e79a9e38 | 969: Wallet money dialogs on the shared shell with inline field errors and the opaque commit step |
| 66703591 | 969: Offer, direct-trade, KYC and resell dialogs on the shared shell, sonner replaced by the one toast, legacy Modal removed |
| b19b96a6 | 969: One report error inside the open dialog on customer pages |
| 7fac4ff4 | 969: Match the v5.2 boards for toast offset, actions row, field look, error icon, sheet band and deposit plate |
| 8c52a94b | 969: Band row padding only on opaque sheets, report actions on the shared footer |
| f20195ac | 969: Escape the apostrophe in the solid sheet test |
| 746ea018 | 969: Keep the excluded proof of residence dialog on its default look |

No AI attribution trailer. One history wart: the `modal.tsx` deletion was staged early and landed in 26746d6a, so 26746d6a and e79a9e38 do not build on their own (their wallet and offer importers still referenced it). Every commit from 66703591 on builds. A squash merge, as #304 used, removes the wart.

## 3. Files and size (origin/redesign/906..746ea018)

Per-file numstat: `/Users/starbird/Workspace/Code/qic/.scratch/969/evidence/overseer/size-numstat.tsv`.

| Part | Files | Added | Deleted |
| --- | --- | --- | --- |
| Product code | 28 | 2,219 | 892 |
| Tests and e2e page object | 18 | 2,609 | 35 |
| package.json and lockfiles (sonner only) | 3 | 0 | 15 |
| Total | 49 | 4,828 | 942 |

Main product files: ui/dialog.tsx +617/-65, ui/toast.tsx +289/-54, WithdrawModal +203/-29, KYCRequirementModal +181/-219, EditTradeLimitModal +177/-78, store/ui-store.ts +120/-5, EditOfferMarginModal +86/-48, dialog-stack.ts +77 (new), hooks/use-customer-surface.ts +76 (new), confirm-dialog.tsx +71/-35, DepositModal +61/-52, lib/customer-field-classes.ts +48 (new), ui/modal.tsx -182 (deleted).

Size stop: product additions are 2,219 lines, about 3,100 changed lines counting deletions, across 28 product files (49 in all). That sits at the edge of the "about 3,000 lines or 50 files" stop but not clearly past it, so I continued. The reviewer should know it is at the line.

## 4. Gate receipt

`QIC_GATE_ROOT=/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root /Users/starbird/Workspace/Code/qic/.claude/scripts/gate.sh --base origin/redesign/906` on 746ea018: GATE PASS. Receipts: `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root/.scratch/gate/20261022T083938/`. Transcript: `/Users/starbird/Workspace/Code/qic/.scratch/969/evidence/overseer/gate-run-final.txt`.
- bun-typecheck ok, bun-lint-no-new-errors ok (48 changed files, no file's error count rose), bun-build ok.
- Earlier runs: PASS on b19b96a6 (`20261022T062217`), FAIL on 8c52a94b (`20261022T083438`): a test I added had an unescaped apostrophe. Fixed in f20195ac, then PASS on f20195ac (`20261022T083636`).
- The backend submodule is empty in this worktree, so no backend steps ran (no backend change).
- Lint debt the gate tolerates as pre-existing: `react-hooks/set-state-in-effect` in KYCRequirementModal, EditOfferMarginModal, EditOfferRequirementsModal, EditTradeLimitModal and TransactionStatusModal (1 each, same count as base).

## 5. Tests

Final run on 746ea018: 85 suites, 1,297 passed, 1 skipped. The skip already exists on base, in a suite this card does not touch ("includes TRC-20 (TRON) network option"). Log: `/Users/starbird/Workspace/Code/qic/.scratch/969/evidence/overseer/jest-final.log`. The full suite was not run.

New suites:
- dialog-shell (25)
- toast-v52
- ticket-969-withdraw-state-survival (17)
- ticket-969-offer-dialog-field-errors
- ticket-969-board-field-look
- ticket-969-silent-toasts
- ticket-969-kyc-dialog-shell
- ticket-969-no-legacy-overlays

Mutation checks: builder B broke dismissible, the material latch and the 819 `dismissible={false}` one at a time, and 4 tests failed (`evidence/builder-b/new-suite-mutation-check.log`). Builder A's two stack tests failed on 49cee000 (`evidence/builder-a/stack-test-on-49cee000.log`).

Existing-suite edits (I checked every diff line):
1. Query root only (`const { baseElement: container }`, no `expect()` change): ticket-769-withdraw-modal-runtime, 776, 839, 9dl9xx7a, withdraw-fiat-864, KYCRequirementModal-827-no-limit. In 9dl9xx7a and KYC-827, the `not.toContain`/`not.toMatch` checks would otherwise pass trivially on an empty container.
2. Owner decision D2 (in-dialog validation becomes inline), same message strings:
   - `ticket-zoqyJwfY-withdraw-amount-precision.test.tsx:277-278`. It used to expect a toast.error starting "Insufficient balance". It now expects an inline role=alert in the dialog with that text, aria-invalid and aria-describedby on the field, and no toast.error. Stronger than before.
   - `ticket-769-withdraw-network-choice.test.tsx:242`. A source regex, changed from `toast.error("Please select a network")` to `withFieldError("network", "Please select a network")`. Same strength.
3. Added assertions only: ticket-zoqyJwfY-transfer-amount-precision (field look classes).
4. ticket-645-wallet-manage-modals (static source shapes) and ticket-645-wallet-manage-runtime pass unchanged.

## 6. Observed proof (local production builds, Puppeteer via Bun)

Server `bun run start` on port 3100 (3000 was held by another agent's server), with a temp Chrome profile created and deleted. A fixture `auth-storage` user, every /api and socket request answered from fixtures, no sign-in, no cookies or passwords, no backend writes, no money. A temporary harness route outside (main) was created and deleted each run, and `git status --short` was clean afterwards. Tools: `/Users/starbird/Workspace/Code/qic/.scratch/969/tools/`.

- `evidence/compare/` (b19b96a6): first board comparison and the full behaviour pass for 15 dialogs at 390 and 1440, plus staff and public.
- `evidence/compare2/` (7fac4ff4): re-comparison after the fix round, actions rows at 390 and 768, toast position, and a regression pass.
- `evidence/compare3/` (8c52a94b): band-row padding per material, Report actions, Report 500 and 201.

Results:
1. Dialog shell and keyboard (all 15 dialogs, 390 and 1440): pass.
   - Glass or solid as specified, scrim rgba(15,23,42,.48) with no blur, the sheet bottom equals the viewport, focus starts inside, Tab and Shift+Tab wrap, Escape closes, and focus returns to the opener.
   - KYC by design: the first Escape opens its own opaque "Skip verification?" confirm.
2. 819 notice: pass. Opaque alertdialog, no close button, Escape inert.
3. Withdraw: pass.
   - An invalid TRON address shows inline under the field with role=alert, and no toast.
   - A server 500 shows as a form-level error directly above the actions row.
   - With the request held: Escape and the scrim are inert, the close button is hidden, and a double click sends 1 POST.
   - The success chain opens TransactionStatus, and closing it returns focus to the page Withdraw opener.
4. Transfer and Lock: pass. Glass, then solid from confirm, still solid after Back.
5. UserSearch to CreateDirectOffer: pass. The new dialog stays glass through the search dialog's exit animation (builder C's stack finding, fixed in 26746d6a).
6. Report: pass. A 500 shows the form error above the actions row, the old red box is hidden on customer pages, and no toast renders. On a 201, no toast shows while the dialog is open; the success toast appears in the polite region once it closes.
7. Toast: pass.
   - Bottom edge at 24px with no `--qt-overlay-bottom`, at 118px with 118px set, and at 24px on /contact-us.
   - success, info and warning are role=status in the polite region; error is role=alert in the assertive region; both regions stay mounted while empty.
   - Lifetime about 6.2s; hover pauses it and leaving resumes it.
   - Reduced motion: opacity only, 150ms linear, no progress bar.
8. Staff and public: unchanged. The admin banned-words ConfirmDialog and the Add word dialog use the base class strings and carry no `gl` classes. The /contact-us toast keeps the dark look.

## 7. v5.2 board comparison (owner rule of 22 Oct: replicate)

Boards rendered from the editable sources, checked against the PDFs: G16b at 0.009% pixel difference, G4 at 0.28%, text anti-aliasing only. Each board was compared with the build at the same size and state, side by side and by computed style (`evidence/compare2/side/*-side.png`, `*-diff-visible.json`, `evidence/compare3/side/`).

Fixed after the comparison:
- toast 8px too high
- EditTradeLimit maximum field overflowing at 768
- actions rows squeezing the primary button to 44px on sheets (fixed once in the shared footer)
- error icon stroke 2 to 1.8
- invalid field look now equals tb-dialog `.input.is-error`: 1px #B91C1C border, 12px radius, white fill, outer 1px ring, 14px padding, 48px height
- rest-state field look
- solid-sheet band row 6px padding (ph-release, G16b 390), kept at 0 on glass sheets (ph-sheet)
- deposit plate: flex, 14px gap, 12px padding
- Report on the shared footer, so its form error sits above the actions

Recorded for the owner's decision (contradicted between boards, or not changeable inside 969's scope):

| # | Boards | Element and property | Values | What I built |
| --- | --- | --- | --- | --- |
| 1 | tb-dialog, content-zone, content-zone-2 (shared G) against 87 error icons on the A to E group boards (B-money and C-discovery included) | error icon shape | triangle (shared G) / circle (all groups) | Circle everywhere, the same as 970's FieldError, so field and form errors match |
| 2 | G16b 1440 opaque commit step against tb-confirm | ConfirmDialog panel width and actions | 520px with the primary growing to 368px / 440px right-aligned (primary 191px) | tb-confirm (size sm, 440, right-aligned). TradeSidebar's release confirm can pass `size="lg"` when 959 redesigns the trade room |
| 3 | tb-dialog (top 168), tb-confirm (top 300) against G16b 1440 (vertically centred) | panel vertical position | fixed top / centred | Centred |
| 4 | all boards | button vertical padding | 0 / 8px (970's shared Button). Height is 44px on both, no visible difference | Left as 970 built it; changing it touches every Button on the customer surface |
| 5 | ph-release, G16b 390, ph-toast | amounts inside copy set in the number font and bold (description 61 vs 60px tall, toast text 21 vs 20px) | board markup inside caller copy | Copy is plain text from the callers. TradeSidebar (959) and the toast callers own that markup |
| 6 | tb-dialog, B boards | Withdraw address field padding-right 80px (Paste button overlay), Withdraw amount rows taller than 48px | board 14px / 48px | Unchanged. Both need a content restructure owned by 958 (wallet steps) |
| 7 | ph-sheet, tb-dialog, tb-confirm, release | dialog body content and copy (Deposit layout, labels, "Edit offer" vs "Edit trade limit", the well, release details) | board content | Group cards 956 to 959 own page content (plan exclusion 2). 969 replicates the shell and the elements it adds |

## 8. Owner decisions, as handled

1. v5.2 artboards win. Built straight from tb-dialog, tb-confirm, ph-sheet, ph-toast, ph-release (G4) and content-zone-2 (G16b) through 970's glass classes. Sizes follow the boards (confirm 440, offer edit 520).
2. B6 outlet rejected (D2). No toast is ever drawn in a dialog portal. Client-side validation that names a field is a FieldError under that field, with aria-invalid and aria-describedby:
   - Withdraw: address, network, amount and code
   - Transfer and Lock: amount
   - EditTradeLimit and EditOfferMargin

   Other errors raised while a customer dialog is open are tagged to the top dialog and shown as its form-level error above the footer (role=alert, immediate). Success and info toasts are held and released when no customer dialog is open (closing dialogs do not count). Off the customer surface nothing is routed or held.
3. No design-expert package was used.
4. QA rows that need sign-in, staging money or a screen reader are `deferred-to-QA` for the review app.
5. Replicate rule (22 Oct): see section 7.
6. Coordinator approvals relayed during the run:
   - builder B added optional `invalid` and `errorId` props to StepUpPrompt and NetworkDropdown (aria pass-through only, defaults unchanged)
   - builder C's stack finding was fixed by builder A

Coordination with 968: the toast container uses `bottom: var(--qt-overlay-bottom, 24px)`. 969 does not define the variable and did not edit globals.css.

## 9. Acceptance map

`/Users/starbird/Workspace/Code/qic/.scratch/969/reports/acceptance-map.md`: 27 rows. 18 are fully verified-local, counting D5 (board replication), which is verified with the section 7 differences recorded. 8 are verified-local with a staging-only part deferred to QA: 320px and screen-reader announcement, chained focus on real data, real Nile withdrawal, 2FA on staging, real staging report and review, staff side-by-side with staging and the moderator ConfirmDialog, live re-acceptance and session gates, and the proof-of-residence look in a browser. 1 row is fully deferred: the transfer, lock and deposit outcomes on staging. None failed.

## 10. Open risks

1. Size is at the L line (section 3).
2. A narrower opt-out than planned: ProofOfResidenceSection (excluded) sat on a customer route and would have picked up the shell automatically. It now passes `look="default"` (one prop). Two other untouched ui/dialog consumers, ConnectWalletButton and KycMediaPanels, render only off the customer surface today, so they keep their look; any future customer route for them would pick up the shell.
3. Behaviour changes for reviewer attention:
   - DialogContent now returns focus to its opener on every surface (Radix did this only for DialogTrigger).
   - Staff ConfirmDialog is now a Radix dialog: it gains a focus trap and focus return and keeps its April overlay (bg-black/60 with blur). It loses Modal's phone handle and entry animation.
   - KYC: Escape on the "Skip verification?" confirm now closes that confirm instead of re-asking.
4. Pre-existing, not changed:
   - Escape inside an open NetworkDropdown list closes the whole Deposit or Withdraw dialog (the old Modal did the same).
   - An early run showed the consent banner drawn above the dialog scrim (ConsentBanner z-index). That capture was overwritten, so it is not proven here; worth a look in QA.
5. Not proven in a browser: Transfer and Lock with a request in flight (Jest only), the trade room's own TradeSidebar mount (ConfirmDialog checked with the board copy), and a toast raised on /wallet itself (harness page used).
6. Parallel work: 968 edits globals.css and relies on the same toast container. Whichever merges second rebases and re-runs toast-v52 and dialog-shell.
7. The frontend repo has no CI. The local gate plus the targeted suites are the merge proof.


## Owner decision, 22 Oct 2026
Toast lifetime board clash (G1, B18 at 6s vs D3, E5 at 4s) settled by Marcello: 6 seconds everywhere. The build already uses 6s (DEFAULT_DURATION 6000, no caller overrides), so nothing changes.
