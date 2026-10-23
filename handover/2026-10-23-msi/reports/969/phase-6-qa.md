# 969 phase 6 QA (/qic-qa)

Date: 2026-10-22, 10:00 to 10:56 UTC. Owner instruction (Marcello, 22 Oct): "send 969 to /qic-qa he has to do it".

## Verdict: FAILED (card moved to QA Failed)

1. **AC2, 969 defect.** The withdraw dialog (opaque from its first frame, and its 2FA code step) and the transfer review and 2FA verify steps have `role="dialog"`. The card's Expected behaviour and board G1 ("role=alertdialog from the commit step"; "withdrawal and send review onward") require `role="alertdialog"`. Only TransactionStatusModal:176, the DepositModal 819 notice and ConfirmDialog set it. WithdrawModal.tsx:1156 and TransferModal.tsx:366 pass no role. Same at 390 and 1440. The opacity part of AC2 passes.
2. **AC2, older bug that 969 did not cause.** On an account with authenticator 2FA, a TRON withdrawal goes Continue (quote continue 200), then the code prompt, then Back (fields kept). After that, Continue re-sends the same quote id and gets 409 `WITHDRAWAL_FEE_QUOTE_EXPIRED`, shown inline as role=alert. Every further Continue repeats the 409, so the withdrawal cannot resume without closing the dialog. `holdQuoteAndContinue` and the gas services are identical on origin/main, so this predates 969. Decision needed: fix it inside 969, or raise a separate card (958 or backend).

Jev (jev-1.14.0): verdict fail, confidence 1.0; severity 1.41 (confidence 0.25); needs_product_decision 0.27; reproducible_from_evidence 0.70. The fail flow applies. Request and response: `/Users/starbird/.config/qic/qa-artifacts/969-20261022T100011Z/jev-request.json`, `jev-response.json`.

## Target and identity

- Review app qictrader-fe-pr-303 on the staging backend (`/health` ok). Releases during the run: v9 179f342f (exploration only), v10 dd59fb9e, v11 b562b4e3, **v12 d3cd77a0** (most evidence, from 10:21Z), and v13 9a561b0d (live from about 10:41; it adds only trade-ledger files and touches no dialog, toast or wallet file). Every result in this report was re-run on v12 or v13. Release log: `/Users/starbird/.config/qic/qa-artifacts/969-20261022T100011Z/release-log.txt`, `release-receipt.txt`.
- Accounts, by email: marcello@unlimited-beats.com (buyer-906, user, KYC 2), marcellohaupt@gmail.com (seller-906, super_admin, KYC 3) and starbirdbeats@gmail.com (third-906, user, KYC 3). The qic-qa-signin agent supplied every session. Browser: Puppeteer with plantSession, one Chrome for Testing at a time, temporary profiles under the scratchpad, each deleted. An in-page guard blocked every money and order write (withdraw, transfer, locks, trades, escrow, direct-offers, offer create and edit).

## Results (390x844 and 1440x900)

| Area | Result |
| --- | --- |
| Sheet below 768 / centred dialog | Pass. Sheets: bottom 844, 20 px top corners, handle, max-height 759.6 (90%), own scroll. 1440: centred, 20 px radius. Scrim rgba(15,23,42,.48), no blur. Deposit, 819 notice, Withdraw, Transfer, Lock, Report Offer, Find a Trader, Create Direct Offer (5 steps), Transfer verify, Withdraw 2FA step |
| Focus at open | Pass. First field (Withdraw address, Transfer search, Report reason, Find a Trader search, Withdraw code field) or the title (Lock, 819 notice, Create Direct Offer) |
| Trap / Escape / return | Pass. Tab and Shift+Tab never left and wrapped; Escape closed and returned focus to the opener; Create Direct Offer returned to Find a Trader; the 819 notice ignores Escape and the scrim, only OK closes it; in-flight quote hold disables Close and ignores Escape and the scrim |
| Shift+Tab on a focused title | Stays on the title (Lock, 819 notice, three presses); Tab goes to the first control. Not a defect: no trap under WCAG 2.1.2 (Tab and Escape leave), no order fault under 2.4.3 (the title is not a sequential stop), and G1 prescribes the tabindex -1 title and "stay inside". Optional polish: wrap to the last control |
| Materials | Pass. Withdraw solid from frame 1; Transfer and Lock glass, then solid from review, still solid after Back; Deposit glass with a white opaque plate holding the address and QR; network listbox opaque |
| Roles | **Fail**, finding 1 |
| Step-up | Code prompt inside the dialog, Back keeps the fields (pass); resume after Back fails (finding 2, older bug); transfer verify step works and Back returns to a solid review |
| Deposit address and network | Pass. Solana EM9p158Po4sfuKTh6GFAeYzjDT2TNRgMp2k5o36a9F2c and TRON TTcQR7SjFyghqPXrknknmzER4EapSnWxL1 match `/custodial-wallet/chains` |
| Toast look | Pass. Pane rgb(230,246,255), 1 px white edge, 16 px radius, shadow, no backdrop filter, 44x44 Dismiss |
| Toast roles | Pass. Success in role=status / polite; errors in role=alert / assertive |
| Toast timing | Pass. 6.2 s untouched; hover and focus pause past 10 s; gone about 4.2 s after leaving; enter 200 ms with translate 8 px to 0; reduced motion is opacity only, 150 ms linear, no translate, no bar |
| Toast placement | Pass. 1440: right 32, bottom 32, width 360. 390: toast bottom 760, dock top 772 (12 px) |
| Never over a dialog | Pass. A toast raised on /wallet stayed hidden 7 s while Transfer was open, then came back with its remaining time (about 4.9 s); one toast at a time |
| Silent messages | Pass. "Referral link copied" plus "Paste it anywhere you share links." (both sizes); "Failed to copy link" as an alert (1440; the 390 /dashboard/affiliate page says "Failed to copy. Please try again."); Report Offer real submit (POST /offers/4375ae39.../report 200), dialog closes, then the "Report submitted successfully" toast; a duplicate (400) shows "Failed to submit report" inside the dialog |
| Staff dialog | Pass. /admin/banned-words as marcellohaupt@gmail.com: Add word (dark default look, trap, Escape return, POST 200, dark toast); Remove ConfirmDialog (alertdialog, dark, trap, Escape return, DELETE 200). Both test words removed; read back by API |

## Money steps

Not run (agent safety rule). Each was taken to its final screen and stopped:
- Withdrawal through live 2FA: code prompt reached, no code entered.
- 1 USDT transfer from marcello@unlimited-beats.com to marcellohaupt@gmail.com: review screen reached, Confirm Transfer not pressed. The reverse direction was not attempted.
- 1 USDT lock: review screen reached, Confirm Lock not pressed. The unlock was not attempted.
- Trade: amount 650 KES entered on offer 26112496, "Confirm & Start Trade" not pressed. One script slip: an early version pressed that button once at 10:30Z, and the in-page guard stopped `POST /api/v1/trades` inside the browser. No request reached the server (network log), and the trades list shows no new trade.
- Direct offer: "Send Offer" review step reached at both sizes and not pressed.

Balances and locks before and after are equal for all three accounts (`api/balance-*`, `api/locks-*`). No tx hashes from this run. The earlier audit's Nile withdrawal is 072c9c1a00f226ed887026d4b7603f5d89bd7dc510d7d9cf2b8d744b34492e7f.

## Account changes and residue

- starbirdbeats@gmail.com: the sign-in agent turned 2FA on at 10:04Z and **restored it off** at 10:52Z. Read back from the database and the API (enabled=false, hasPendingSetup=true as before). The audit rows are 0d0eb91c and efa9c84f. users.updated_at moved (a trigger rewrote it). Ledger: `~/.config/qic/qa-sessions/change-ledger/969-20261022T100400Z-2fa.json`.
- The step_up_2fa flag on marcello@unlimited-beats.com was turned on by another run at 08:56Z and is still on. I did not touch it.
- One pending QA report by marcello@unlimited-beats.com on offer 4375ae39. The test words were removed.

## Not covered

Review toast and "Please select a rating" (no completed trade available without opening one). Release and cancel confirms (no live trade). Reacceptance and session-conflict dialogs. A real screen reader. Outcome recording: `qic_record.ts` accepts only staging.qictrader.com, so I recorded one clip with Puppeteer screencast (`969-AC2-stepup-back-continue-1440.webm`, 299 frames, inspected as a contact sheet).

## Trello

The card moved to QA / Testing at the start, then to **QA Failed**, confirmed by a fresh GET. One comment was posted (read back 10:55:35Z). 12 attachments were uploaded and read back: 969-AC2-1 to -5, the clip, and six pass shots.

Evidence: `/Users/starbird/.config/qic/qa-artifacts/969-20261022T100011Z/` (JSON per run, PNGs, `scripts/`, proof spec).
