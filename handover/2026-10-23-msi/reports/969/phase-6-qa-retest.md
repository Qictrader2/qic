# 969 phase 6 QA re-test (/qic-qa)

Date: 2026-10-22, 17:16 to 17:36 UTC. Re-test of the two findings in `phase-6-qa.md` after Frontend #316 (squash e3e1dab8), plus a short regression pass.

## Verdict: PASSED (card moved to QA Approved / Ready For Deployment, as the owner asked for a pass)

1. **Finding 1 (alertdialog) is fixed.** A MutationObserver recorded every dialog node from page load. Withdraw is `role="alertdialog"` (solid) on the first node added, and stays so on the authenticator code step and the emailed step-up code step, three rounds each. Transfer is `dialog` (glass) on the form, switches to `alertdialog` (solid) on review, and stays `alertdialog` after Back to the form, on review again, on the 2FA verify step and after Back from verify. Lock is `dialog` (glass) on select, `alertdialog` (solid) from confirm, and stays so after Back. All of this holds at 390x844 (sheet) and 1440x900 (centred), for both accounts.
2. **Finding 2 (quote resume after Back) is fixed.** On starbirdbeats@gmail.com with authenticator 2FA: TRON, 3.5 USDT, a note. Continue, code prompt, Back (address, amount, note and network kept), Continue, code prompt again. I did three rounds at each size. Each Continue fetched a new quote id and `POST /gas/withdrawal-fee/quote/{id}/continue` returned 200: 1440 b5d0587e, 0aa5aa93, 9170fd3d; 390 5d819953, 837f8426, 6f2bad95. No 409 and no inline error. The quote id after Back is new by design (969-qafix). The emailed step-up account marcello@unlimited-beats.com behaves the same way (six continue calls, all 200, all new ids).

Jev (jev-1.14.0): verdict pass, confidence 0.91 (pass 0.94); severity 1.30 (confidence 0.49); needs_product_decision 0.08; reproducible_from_evidence 0.80. These clear every threshold, so the pass flow applies. Request and response: `/Users/starbird/.config/qic/qa-artifacts/969-retest-20261022T171632Z/jev-request.json`, `jev-response.json`.

## Target and identity

- Review app qictrader-fe-pr-303 on the staging backend (`staging-api.qictrader.com/health` 200). **Release v30, build 80f50bf3 (succeeded 17:11Z), source ce1f24bc90e776a8758fb2c40f6fff991e663a69** (redesign/906 head, PR #303). `git merge-base --is-ancestor e3e1dab8 ce1f24bc` passes, so #316 is in the build. The release did not change between the start (17:20Z) and the end of testing. Since e3e1dab8 the only changes to wallet dialogs come from #321 (request keys) and the sync of nS5CVEIN; the wallet modal files have no diff after #316. Log: `release-log.txt`.
- Accounts, by email: marcello@unlimited-beats.com (buyer-906, user, KYC 2, step_up_2fa flag on, no authenticator), marcellohaupt@gmail.com (seller-906, super_admin, KYC 3, balance check only), starbirdbeats@gmail.com (third-906, user, KYC 3, authenticator 2FA turned on for this test). The qic-qa-signin agent (signin969r) supplied every session. Browser: Puppeteer with plantSession, one Chrome for Testing at a time, a temporary profile per run under the scratchpad, each deleted afterwards. An in-page guard blocked every money and order write; it recorded no blocked attempt because none was made.

## Regression pass (current build, 390x844 and 1440x900)

| Area | Result |
| --- | --- |
| Sheet / centred | Pass. Deposit at 390: bottom 844, 20 px top corners, handle, max-height 759.6, own scroll. At 1440: centred, 480 wide, 20 px radius. Scrim rgba(15,23,42,.48), no blur. Withdraw, Transfer and Lock are sheets at 390 and dialogs at 1440 |
| Focus at open | Pass. Withdraw address field, Transfer search, Lock title, then the 819 notice title over Deposit; the code steps focus the code field |
| Trap | Pass. Tab and Shift+Tab never left Deposit, Withdraw, Transfer or Lock (0 escapes) and wrapped |
| Escape and focus return | Pass. Escape closed Deposit, Withdraw (form and code step), Transfer (form, review, verify) and Lock (select, confirm), and focus returned to each opener. The 819 notice ignores Escape and the scrim; only OK closes it |
| Deposit plate | Pass. White opaque plate (rgb 255,255,255, no backdrop) holds the Solana address EM9p158P...a9F2c and the QR code |
| Toasts | Pass. Success toast role=status/polite, errors role=alert/assertive ("Failed to copy link" at 1440, "Failed to copy. Please try again." at 390). Pane rgb(230,246,255), 1 px white edge, 16 px radius, no backdrop filter, 44x44 Dismiss. Untouched about 6.2 s. Hover and focus pause past 10 s, then gone about 4.2 s after leaving. Entry 0 to 1 opacity with 8 px rise. Reduced motion is opacity only, 150 ms linear, no translate, no bar. Placement at 1440: right 32, bottom 32, width 360. At 390: 12 px above the dock. A toast raised before Transfer opened stayed hidden while the dialog was open, then came back (about 4.9 s left). One at a time |

## Notes (not findings)

- The newly merged nS5CVEIN notice "Transaction verification starts 30 October" opens once per day per browser profile, on the first page load, as a shared-shell `dialog` (glass). While it is open it holds toasts back, which is the intended "never over a dialog" rule. My first toast run at 1440 hit this; I reran after "Got it" and it passed.
- After Back from a code step, focus goes to the dialog panel rather than a field. No criterion covers this, and it is inside the trap.
- The emailed step-up code request returned 429 RATE_LIMITED on the 390 run, because my 1440 run had already requested three codes within minutes. The code step still rendered; the cause is test volume, not a 969 problem.

## Money steps

Not run (agent safety rule). Each one stopped at its final screen:
- Withdrawals (TRON, 3.5 USDT): authenticator code step and emailed code step reached, no code entered, Confirm Withdrawal not pressed. Only quote holds (`/continue`) and step-up code requests were sent.
- Transfer of 1 USDT to marcello_haupt_qic (marcellohaupt@gmail.com): review screen and, for starbirdbeats@gmail.com, the 2FA verify step reached; Confirm Transfer (send) not pressed.
- Lock of 1 USDT: confirm screen reached, Confirm Lock not pressed.

Balances and locks before and after are equal for all three accounts (`api/balance-*`, `api/locks-*`; only key order and the USD estimate moved). No tx hashes from this run.

## Account changes (qic-qa-signin agent)

The sign-in agent's report, which I confirmed through the API myself (buyer `/users/me` featureFlags.step_up_2fa false; third `/users/2fa/status` enabled false):

1. **2FA on starbirdbeats@gmail.com.** Turned on for the test at about 17:18Z (audit 907ca2db) and restored off at 17:34:37Z (audit 880f34ca). The TOTP secret was not touched. After: DB `two_factor_enabled=false`; API enabled=false, hasPendingSetup=true, as it was before the earlier run. `users.updated_at` moved because a trigger rewrites it. Ledger `~/.config/qic/qa-sessions/change-ledger/969-retest-20261022T171757Z-2fa.json`, marked restored.
2. **step_up_2fa on marcello@unlimited-beats.com.** Another run turned it on at 08:56Z, and this re-test used it for the emailed code step.
   - Before: DB `user_feature_flags` row enabled=true, enabled_at 2026-10-22 08:56:46Z, enabled_by marcello_haupt_qic; API `featureFlags.step_up_2fa=true`.
   - After: turned off at 17:35:03Z through `POST /api/v1/admin/users/{id}/feature-flags` (enabled=false, HTTP 200) as seller-906. DB enabled=false with a 969-retest note; API `featureFlags.step_up_2fa=false` (solana and trade_receipt still true).
   - That endpoint writes no audit_entries row, so the change is recorded on the flag row itself. Before the 08:56Z run there was no row, so the effective value was false. The row now exists with enabled=false and was not deleted.
   - Ledgers: `969-retest-20261022T173503Z-stepup-off.json`; `969-20261022T085633Z.json` is marked restored.
3. Residue: the test sent six quote holds per account and six step-up code emails for marcello@unlimited-beats.com, three of which were rate-limited. No other changes.

## Trello

The card moved from QA Failed to QA / Testing at 17:16:41Z (read back), then to QA Approved / Ready For Deployment at 17:36Z on the owner's explicit instruction for a pass (fresh GET: idList 69d535d2, list name read back). Seven attachments were uploaded and read back: 969-retest-AC2-evidence-roles.png, -1-withdraw-2fa-after-back-1440, -2-withdraw-email-step-390, -3-transfer-after-back-390, -4-lock-confirm-1440, -5-clip-contact-sheet, and the clip 969-retest-AC2-2fa-back-continue-1440.webm (300 frames, inspected as a contact sheet; Puppeteer screencast because `qic_record.ts` accepts only staging.qictrader.com). One comment was posted (read back 17:35:57Z).

Evidence: `/Users/starbird/.config/qic/qa-artifacts/969-retest-20261022T171632Z/` (JSON per run, PNGs, `scripts/`).
