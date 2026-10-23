# 969 PHASE 5 ACCHECK (audit only)

Date: 2026-10-22, 06:20 to 07:10 UTC. Overseer: fresh, did not build, review or merge 969.

## What was audited

- Card 969 (https://trello.com/c/DqJFKFVH), current text (aligned with the v5.2 boards on 22 Oct), 4 Acceptance Criteria checklist items, list "Development Complete". Card text saved at `~/.config/qic/qa-artifacts/969/card-969-description.md`.
- Deployed: frontend `366dca2f1097d2e2184cf09ba88038f5af766363` = origin/redesign/906 (PR Qictrader2/Frontend#306), Heroku review app `qictrader-fe-pr-303` release v7 "Deploy 366dca2f", web.1 up, staging backend `/health` ok. Its tree `a0050f83` equals the gated worktree `c54ea8fe`.
- Owner decisions applied as given in the brief (no toast on a dialog or sheet, one toast at a time, 12 px above the dock, bottom right 32/32/360 from 768 px, sheet below 768, release confirm 520 per G16b, other confirms 440 per tb-confirm, circle error icon, G7 referral copy).
- Test plan: `~/.config/qic/qa-artifacts/969/htt-plan.md` (AC1 -> proof 2, AC2 -> proof 3, AC3 -> proof 4, AC4 -> proof 5). Worker brief: `~/.config/qic/qa-artifacts/969/worker-brief.md`.
- Sessions: buyer-906 (unlimited_beats, user, KYC 2) and seller-906 (marcello_haupt_qic, super_admin, KYC 3) from the qic-qa-signin agent, both passed `--check`; the agent changed nothing. Browser runs used plantSession in Puppeteer with temp profiles under the session scratchpad, all deleted.

## Verdicts

| AC | Worker | Jev (jev-1.14.0) first / retry | Gate | Result |
| --- | --- | --- | --- | --- |
| 1 Shared v5.2 dialog, sheet <768 / dialog >=768, focus kept and returned | MET | met 0.56 / met 0.76, admissible 0.97, partly 0.28 / 0.21 | below 0.85 | Not ticked, for the arbiter |
| 2 Money dialogs same outcomes, opaque from review, deposit glass with opaque plate | MET | met 0.67 / met 0.80, admissible 0.99, partly 0.09 / 0.08 | below 0.85 | Not ticked, for the arbiter |
| 3 One toast style, pauses on hover or focus, formerly silent messages show | MET | met 0.79 / met 0.79, admissible 0.99, partly 0.18 / 0.16 | below 0.85 | Not ticked, for the arbiter |
| 4 Staff and public dialogs and toasts open, submit, close as before | MET | met 0.91, admissible 0.97, partly 0.09 | passed | Ticked (read back complete) |

Jev answered all four with "met"; AC1 to AC3 stayed under the 0.85 confidence bar after one retry that added decisive evidence (AC1: the remaining dialogs and the panel class code; AC2: the unchanged request layer; AC3: the store routing code). Per the brief I did not decide those myself. Packets and answers: `/private/tmp/claude-501/-Users-starbird-Workspace-Code-qic/853e4c8b-e6be-48cd-8930-893197f52192/scratchpad/969ac/jev-state-ac*.json` and `jev-resp-ac*.json` (excerpts re-read from the pinned commit and the captured JSON before sending).

### AC1 evidence (review app, 390x844 and 1440x900)

Sheet at 390 (data-layout sheet, bottom 844, 20 px top radius) and centred dialog at 1440, Tab and Shift+Tab stayed inside, Escape or close returned focus to the opener for: Deposit, 819 notice (Escape and outside click ignored, only OK closes, distinct labelledby from Deposit), Withdraw, Transfer, Lock funds, Transaction Status (390), Edit limit, Edit price, Edit document requirements, Find a Trader then Create Direct Offer (focus back on Find a Trader), resell Create Resell Offer, KYC Verification Required plus nested solid Skip verification? (distinct labels), Report Offer, release and cancel confirms. Scrim rgba(15, 23, 42, 0.48), no blur. `ui/modal.tsx` deleted, no importer left; breakpoint `(min-width: 768px)`. ManageWalletModal has no caller in the pinned source. Files: `ac1-*.json`, `ac1-*.png`, `ac1-ac2-deposit.json`, `ac1-transfer-trap.json`.

Deviations from the Expected behaviour text that the checklist item does not state (reported, not decided):
1. Initial focus lands on the header Close button in every dialog except Find a Trader; the card says focus moves to the first field, or the title when there is none.
2. With the Withdraw network list open, Escape closes the whole Withdraw dialog instead of only the list (`ac1-withdraw-escape-in-network-list-*.png`). The old `ui/modal.tsx` at fe023ad2 did the same, so it is not new in 969.

### AC2 evidence

- Withdrawal: buyer-906, 3 USDT (Tron minimum; fee 4.50), to TEKxYnZeNLWvxib32wTSWvTkTTE6U5DXg4. Double click on Continue sent one `/wallet/withdraw` POST carrying `feeQuoteId` 8cdf2dd7-b405-4633-bcf9-6d5de48d7973, the id the quote-hold call returned. Escape and outside click ignored in flight, Close disabled. Response confirmed, tx `072c9c1a00f226ed887026d4b7603f5d89bd7dc510d7d9cf2b8d744b34492e7f`; Nile: contractRet SUCCESS, block 72039112, USDT contract TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf, from hot wallet TATv2fTGVMpWC6LsJcjnUUUCjSxFQLBVYC, 3000000 units. Balance 81.986926 -> 74.486926 USDT. Transaction Status (solid alertdialog) then Done returned focus to the page Withdraw button; success toast only after close.
- Transfer: buyer-906 to seller-906, 1 USDT, one POST, completed, fee 0. Buyer 74.486926 -> 73.486926, seller 172769.008969 -> 172770.008969. Form glass, review solid, still solid after Back, in-flight Escape and outside click ignored.
- Lock: buyer-906, 1 USDT on own sell offer 2294ad3b, one POST, available 73.486926 -> 72.486926, success toast after close; then unlocked through `POST /wallet/locks/unlock` (200), locks empty, available back to 73.486926.
- Deposit: glass panel, address on `gl-plate` with background rgb(255, 255, 255); 819 notice solid.
- Confirms on a real trade (2f35547f, 600 KES = 4.63786 USDT escrow from seller-906, cancelled by buyer-906 through the cancel confirm; seller balance back to 172770.008969): release confirm 520 wide, Go Back 100.27 grow 0, primary 359.73 grow 1; cancel confirm 440 wide, flex-end.
- Request layer unchanged: `git diff --stat fe023ad2 366dca2f -- src/services src/hooks/api` is empty. ticket-769 and ticket-645 suites 4/4, 38/38 pass on the merged tree.
- Not run: a step-up or 2FA withdrawal (neither QA account has 2FA or the step_up_2fa flag), a credited deposit.

### AC3 evidence

One opaque pane (rgb(230, 246, 255), white 0.85 edge, no blur, 16 px radius, 44 px Dismiss), one at a time, success role=status in the polite region, errors role=alert in the assertive region. Lifetime 6.3 s; hover and focus each hold it past 7 s, gone about 4.7 s after leaving; reduced motion opacity 150 ms linear. 390: toast bottom 760, dock top 772, gap 12 px. 1440: right 32, bottom 32, width 360. "Referral link copied" + "Paste it anywhere you share links."; "Report submitted successfully" + "Our moderation team will review your report." after the dialog closed; duplicate report rejected 400 and shown inside the dialog (qt-dialog-form-error, circle icon, above the actions, no toast); "Thank you for your feedback!" on a real review; Copy address inside Deposit drew no toast while open and showed it after close. Before 969 the three sources imported sonner with no Toaster mounted. The "Please select a rating" path is unreachable because Submit is disabled until a star is chosen.

### AC4 evidence

/admin/banned-words as seller-906: Add blocked word opened in the default look (no gl classes, bg-black/80), focus in the field, trapped, submitted (200), closed with a toast; Remove banned word? ConfirmDialog kept bg-black/60 with blur(8px), trapped, DELETE 200, closed with a toast; at 390 Add word opened and Escape closed it with focus returned. /contact-us dark default toast, role alert, at both sizes. Default-branch class strings unchanged from fe023ad2. Reacceptance, session-conflict and session-timeout dialogs not raised live (need server state or a fresh login); their files are unchanged by 969; SessionConflictGate, SessionConflictModal, session-timeout-warning-578 and legal review-routes suites pass on the merged tree (`jest/jest-related.log`, `jest/jest-gates.log`).

## Trello writes

- Ticked AC4 (checkItem 6ad887d0e8c247ca9cd975f9), fresh GET shows complete; AC1 to AC3 left incomplete.
- One summary comment.
- Card not moved (stays in Development Complete). No attachment.

## Staging changes made for proof

- Money: withdrawal 3 USDT + 4.5 fee from buyer-906 (Nile tx above); transfer 1 USDT buyer-906 -> seller-906 (not reversed); lock 1 USDT then unlocked; trade 2f35547f created and cancelled, escrow returned.
- Records: offer report b868957c-f5b9-4363-9c2b-4ea8e9904473 by buyer-906 on seller-906's offer 26112496 (pending, marked as a QA audit); 5-star review on completed QA trade 10bf1a24 by buyer-906; banned word qa969probezz added then removed.
- Accounts: no KYC, role, permission or 2FA change. Test-browser-only stub: `/kyc/my-limits` answered with level 0 to raise the KYC dialog, trade POST blocked in that run.

## For the arbiter

AC1, AC2, AC3: worker MET with literal evidence, Jev "met" both times but under 0.85 confidence. Open items worth weighing: the initial-focus deviation (AC1 note 1), the Escape-in-network-list behaviour (pre-existing), no step-up path exercised (AC2), info-type toast covered by code only (AC3).

Evidence root: `/Users/starbird/.config/qic/qa-artifacts/969/`.

## AC1 re-check after the focus fix (22 Oct 2026)

Target: frontend 179f342f (PR Qictrader2/Frontend#308, merged into redesign/906), review app `qictrader-fe-pr-303` on the staging backend, 390x844 and 1440x900. Sessions buyer-906 and seller-906 came from the qic-qa-signin agent (no account changes). Puppeteer with plantSession on temporary profiles, all deleted.

Result: every condition on the arbiter's list held at both sizes. I ticked AC1 (checkItem 6ad887cd471443f4be7fec6c, read back complete) and posted one comment (6ad9c96a278c1b2f3879f0c1). Card not moved.

| Dialog | Focus at open | Trap | Escape / return |
| --- | --- | --- | --- |
| Withdraw | address field | in | closes, Withdraw |
| Transfer | username search | in | closes, Transfer |
| Edit trade limit | Min field | in | closes, Edit limit |
| Offer margin | margin field | in | closes, Edit price |
| Documents required per trade | identity switch | in | closes, Edit document requirements |
| Create Resell Offer | margin percentage | in | closes, Resell |
| Report Offer | reason picker | in | closes, Report offer |
| Find a Trader | username search | in | chain below |
| Deposit | title, then the 819 notice takes it | in | closes, Deposit |
| 819 Minimum deposit | title | in | Escape ignored, OK only |
| Lock Funds | title | in | closes, Lock |
| Create Direct Offer | title | in | closes, focus on Find a Trader |
| Verification Required / Skip verification? | title / title | in | Escape opens Skip confirm by design; Skip for Now returns to Confirm & Start Trade |
| Transaction Status | title | in | Done returns to Withdraw |
| Release Without Confirmation? | title | in | closes, Release Crypto |
| Cancel This Trade? | title | in | closes, Cancel Trade |

Close never had focus at open. Mid-commit Escape on Withdraw left the dialog open. Sheet (390 wide, bottom 844) at 390 and centred dialog at 1440 for all.

No money moved. In the test browser only: the quote-continue and `/wallet/withdraw` POSTs were answered locally to reach Transaction Status; GETs of cancelled trade 2f35547f and its escrow were rewritten to escrow_funded / held (fixed expiresAt) so the confirms render; `/kyc/my-limits` answered level 0. Every other write was blocked. A first trade run with a moving expiresAt re-rendered the trade buttons and broke focus return; that came from my stub, and the rerun with a fixed expiresAt is the one recorded.

Notes, not failures: after OK on the 819 notice focus lands on the Deposit Done button (same as before the fix); Shift+Tab while the title has focus keeps focus on the title instead of wrapping.

Evidence: `~/.config/qic/qa-artifacts/969/ac1-recheck/` (`wallet-*.json`, `pages-*.json`, `trade-confirms-*.json`, `kyc-skip-return-*.json`, `deposit-focus-sequence-*.json`, one PNG per dialog per size). Probe scripts: scratchpad `969rc/`.
