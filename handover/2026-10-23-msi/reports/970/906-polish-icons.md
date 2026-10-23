# 906-polish-icons (card 970, shared controls)

- PR: https://github.com/Qictrader2/Frontend/pull/327 (base `redesign/906` at cc201b88, not merged)
- Branch `906-polish-button-icons`, head 75624f81a36c90e44d368b4a965685fc42cad4e1 (commits a419869b, d45dde90, merge 293f49d8 of redesign/906 at c04cac53, 75624f81)
- Worktree: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/906-polish-icons-root/frontend

## Files changed
- `src/components/ui/button.tsx`: one `customer:[&_svg]:size-*` token per size (classes only)
- `src/lib/trade/timeline.ts`: `expired` line
- `src/__tests__/lib/trade/timeline.test.ts`: expectation for that line
- `src/__tests__/design/button-icon-sizes.test.tsx`: new, 55 tests

## Prior 970 decision
None. replicate.md and contradictions.md compare button boxes, not the svg inside them (item 10 is the 52px square only). HANDOVER logged the 16 vs 18 px gap as open.

## Measurement
- All 188 v5.2 `.dc.html` sources rendered in headless Chrome at canvas size: 3,623 svgs in button-like elements, 2,870 in `.btn` / `.icon-btn`.
- PDF spot check on A2, B6a, C1, D3a, E1d, G0, G8: 12 icons agree to within 0.5 CSS px.
- Evidence: `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/906-polish-icons/` (`button-icons.json`, `divergence.json`, `out/tables.txt`, `spot/`, `scripts/`).

## Board calls (decision rule: exact screen first, then majority of frames)
| Code size | Built | Frames |
|---|---|---|
| sm | 18px | 18 in 38, 16 in 24 (screen groups: 18 in 38, 16 in 10) |
| default | 18px | 18 in 337, 20 in 46, 16 in 7 |
| lg | 20px | 20 in 81, 18 in 29 |
| icon | 20px | All `.icon-btn`: 24 in 674, 20 in 367. Without the top bar bell and Jump to (built with the `icon-btn` CSS class in `navigation/Capsule.tsx`, not Button) it is 20 in 367, 24 in 340. Without dialog Close (hand-built in `ui/dialog.tsx`) as well, 20 in 278, 24 in 243 |
| icon-lg | 20px | 20 in 12, 18 in 1 |

- Shared boards do not settle it: G0 has only one icon button ("Delete offer" danger md, 16px); the G control boards mix 16, 18 and 20 at md.
- The staff look is unchanged. A test pins the staff tokens for all 8 variants x 5 sizes, and a before/after diff of all 40 class strings is empty.
- Production CSS (`.next/static/chunks/b458d730ea83a904.css`): `.[&_svg]:size-4 svg` at offset 377744, `customer:[&_svg]:size-4.5` at 378206, `size-5` at 378339. Same specificity, so the customer rules win only under `[data-surface=customer]`. `--spacing` is 0.25rem, so 4.5 = 18px and 5 = 20px.

## Timeline
- `expired` now reads "Time ran out, the trade cancelled itself" (A11a room-states-lg-1, A20a room-states-768-1, A22c/A22d room-unknown-lg-3/4, A1b overview-2). The brief named `lib/state-machine`, but the string lives in `lib/trade/timeline.ts` (959-S1's model); `lib/state-machine` has no such label.
- Left alone on purpose: room-model step caption, status card countdown and commit "Time ran out at {time}". The boards draw these as bare "Time ran out".

## Gate and tests
- Gate PASS (`gate.sh --fast`, under `.gate-lock`): typecheck, lint-no-new-errors, build. Receipts: `906-polish-icons-root/.scratch/gate/20261022T182046`.
- Tests: timeline + button-icon-sizes 75/75. Also controls 85, replicate-owner-sizes 5, replicate-fixes 21, all passing.

## Review
- Round 1, opus55-high: NEEDS_FIXES. One MEDIUM regression (TradeSummary copy, 16 to 20px) and two board gaps (PayPlate 18px, OfferMenu 24px), all in caller files. The coordinator approved fixing them in this PR.
- Round 2 (the three fixes plus an audit of every customer Button whose icon set its own size class): APPROVE, four LOW findings. The LOW on dead `h-4 w-4` icon classes is left: they still describe the 16px staff size.
- Base moved to c04cac53 (959-S6). I merged it (no overlapping files, no force-push) and audited S6's Buttons.
- Round 3: NEEDS_FIXES. One MEDIUM (phone chat sheet back chevron is 24px on room-sheets-390 and room-states-390; the measurement missed it because that board draws an `<a class="back">`, not a button) and two LOWs (lightbox Close, staged and KYC remove). All fixed.
- Round 4: APPROVE. The two LOWs: one test name corrected; one note that the Copy all test guards the size default.

## Caller overrides (`customer:[&_svg]:size-N` on the Button)
| Control | File | px | Board evidence |
|---|---|---|---|
| Copy trade reference | trade/room/status/TradeSummary.tsx | 16 | room boards, 136 frames |
| Pay plate copies | trade/room/status/PayPlate.tsx | 18 | 168 frames |
| More for this offer | offers/offer-detail/OfferMenu.tsx | 24 | every offer page board; dead svg `size-6` removed |
| Sign out | app/(dashboard)/settings/page.tsx:304 | 20 | profile boards 11/11; no /settings frame draws it |
| Create new ticket | app/(main)/my-tickets/page.tsx:114 | 20 | 8 frames against 4 at 18 |
| Add payment method (x2) | settings/PaymentMethodsSection.tsx:130, :526 | 20 | settings 390/1440; code label "Add" vs board "Add payment method" |
| Filters | app/(dashboard)/trade-history/page.tsx:213 | 20 | history 390/768, 16 frames |
| Chat payment detail copies | trade/chat/ChatMessages.tsx:265 | 18 | 42 frames |
| Remove file (evidence) | trade/DisputeEvidenceUpload.tsx:144 | 18 | 7 frames |
| Retry, proof can't load | trade/documents/doc-ui.tsx:187 | 16 | 3 frames |
| Trade back, phone chat sheet | trade/chat/ChatHeaders.tsx:137 | 24 | room-sheets-390, room-states-390 |
| Close image | trade/ChatAttachment.tsx:366 | 24 | no lightbox board; room Close 103/103 at 24 |
| Remove staged chat file, remove KYC file | trade/ChatAttachment.tsx:232, trade/TradeKycPanel.tsx:153 | 18 | no board; every room remove-file is 18 |

- Audited and already matching: Try again (error boundaries, KYC, ProofOfResidence, MarketplaceStates), Submit proof of residence, Export CSV, Receipt, Copy all, Upload photo, reseller More, Attach, Send, View, Download.
- No board: notifications Mark all read and row actions, MyOffersTable/ActiveTradesTable icon buttons, ReportButton, trade-history Chat, my-tickets Back/Close Ticket, prices/gas/swap refresh, wallet Transfer/LockFunds, and others listed in the audit (`scratchpad/hits.json`, `audit2.py`).
- Not customer surface (`CUSTOMER_SURFACE_RULES` in src/config/routes.ts): affiliate, help, login/forgot-password, WhatsApp, KYC callback, admin/moderator Pagination/CopyButton/BackendUnavailable.
- Tie left at the 18px default: my-tickets/[id] Send Reply (2 frames at 16, 2 at 18).
- Still open: 959-S5a (#326, open) and later slices land after this PR. Their icon Buttons get the size default and need the same check.

## Gate and tests (final)
- Gate PASS at 20261022T200307 (24 changed files): typecheck, lint-no-new-errors, build. 14 suites, 352 tests pass (status-cards, button-icon-sizes, timeline, controls, replicate-*, sign-out, all 959-S6 suites). After the gate, the only change was one test name; that suite ran again (26/26).

## Backend
None.

## Environment notes
- `budget phase` needs `--ticket` as well as `--file` (the brief's example leaves it out).
- Measuring every board took one worker about 25 minutes.
