# Mac mini → MSI handover, 23 Oct 2026

Marcello stopped all work on the Mac mini on 23 Oct 2026 around 13:30 Dubai, and the MSI continues from here. Nothing is running on the Mac. Every piece of work in flight is pushed as a branch, and nothing was merged or deployed as part of the stop.

To pick this up on the MSI:

```bash
git fetch origin handover/msi-2026-10-23
git checkout handover/msi-2026-10-23 -- handover/2026-10-23-msi
```

This branch exists only to carry these notes. Don't merge it into main.

## 1. Standing owner rules (apply to everything below)

1. **The v5.2 boards are the source of truth** for every 906 frontend card (955 to 960, 965 to 970). If a card disagrees with a board, build what the board shows and align the card. If two boards disagree, the board drawn for that exact screen or state wins, then the majority of frames. Christian approved v5.2 on 21 Oct. Don't bring design questions to Marcello.
2. **Backend:** build the frontend against what backend `main` returns today. If a screen needs something main doesn't send, check it in the code and card it as a backend gap. Every new backend need gets a card straight away.
3. **The redesign never merges to main.** Slices squash-merge into `redesign/906` (umbrella draft PR Qictrader2/Frontend#303). Merge first, then QA on the review app https://qictrader-fe-pr-303.herokuapp.com, which runs on the staging backend.
4. **Never deploy to production.** Backend tickets stop at staging plus QA.
5. **Silly blockers:** follow the retro skill steps, put the fix in place, then retry.
6. **QA:** only the `qic-qa-signin` agent signs accounts in. QA accounts may move staging money without asking. QA agents won't make real money submits on hosted environments, so those stay with Marcello.
7. **GitHub text:** first person, no em dashes, no AI attribution. Commit format: `TICKET: Short description`.
8. **USDT only.** No tickets or QA cases for other coins.

## 2. Where every ticket stands

### Frontend: 906 redesign (Qictrader2/Frontend, branch `redesign/906`, head 5ac36d0d)

| Card | Trello state | Done | Left |
|---|---|---|---|
| 970 (0a) colours and controls | QA Approved / Ready For Deployment | all 5 ACs, QA passed | nothing |
| 969 (0b) dialogs and toasts | QA Approved / Ready For Deployment | all 4 ACs, QA passed | **Marcello:** about 10 minutes of real money submits on the review app (2FA withdrawal, transfer, lock and unlock, open and cancel a trade, direct offer) |
| 968 (0c) phone bar, header, overlays | QA Approved / Ready For Deployment | all 5 ACs, QA passed twice | re-check the wallet More menu live (now placed by 958-S6) |
| 959 (A) trade room | Development (In Progress) | all slices merged, incl. S9b (#347) | ACCHECK on the review app (was starting when stopped, no ticks yet; map `reports/959/acceptance-map.md`), then QA |
| 958 (B) wallet and earnings | Development (In Progress) | S1, S1b, S2, S3, S4, S5, S6, S8, S9, S10, S11, S12 | **S7** (branch `958-s7-activity-detail` b8b3ceaf: built, needs gate, review, PR, merge), then ACCHECK and QA |
| 957 (C) marketplace and offers | Development (In Progress) | S1, S2, S3, S4, S6 | **S7** (branch `957-s7-offer-money-start` 674d29e1, WIP: re-gate, review round 2 needs TWO approvals), **S9** (branch `957-s9-resell` just started, = redesign/906), then S5, S10, S8a, S11, S8b, S9m |
| 956 (D) profile, settings, support | Backlog | none | all 11 slices |
| 955 (E) sign in, sign up, reset | Backlog | none | all 8 slices |
| 967, 966 colour sweeps; 965 screenshot register and a11y | Backlog | none | after 957+959 (967), 958+956 (966), then 965 last |
| 960 | Backlog | split into 965 to 970 | closes with them |
| 906 umbrella | Development Complete | draft PR #303 | Christian signs off, then Marcello merges |

Full per-slice detail, file ownership, waves and decisions are in `906-redesign/BUILD-SCHEDULE.md`. The process (3 builders at most, gate, reviews, disk rules) is in `906-redesign/SLICE-BRIEF.md`. The full log is `906-redesign/HANDOVER.md`; start at its "Stopped 23 Oct for MSI handover" section. Per-card plans and slice reports are in `reports/95x/`.

The root repo's `redesign/906` pointer is behind (it points at frontend 28645d0c). Bump it with the next root PR.

### Backend

| Card | Branch / PR | Phase reached | Next step |
|---|---|---|---|
| 917 + 964 password rules | `964-password-policy` in both repos. Backend PR #324 (459c2a2), CI **green**. Frontend PR #334 (c2bd883c), local gates green | GOLIVE opened; gates 0 to 4 passed | Take the staging lock. Squash-merge #324 then #334, then the root pointer PR. Confirm staging carries the SHAs (baselines: backend-staging v640 8fda17fa, frontend-staging v333 a73d2708). Then ACCHECK and staging QA with **throwaway accounts only** (never change the seeded QA passwords), then move both cards to QA. Run notes: `ship-runs/917-964-20261022/` |
| 910 fully funded trade starts | `910-full-funding-check` (9fe34845, on main 8fda17fa). **No PR yet** | PUSH; gates 0 to 3 passed; gate 4 has Jev's score-only rework but no reviewer verdict (cut off) | Re-run the gate 4 review from `ship-runs/910/phase-4.json`. Open the PR, wait for one CI run, merge, confirm staging. Then the read-only diagnosis counts (on_chain and offer_escrow offers and escrows, trades in created), ACCHECK and QA. **Before or with the merge:** the frontend e2e Playwright suites with `escrowType: 'on_chain'` fixtures and PATCH `escrow_funded` (full-trade-journey, dispute-resolution-journey) will fail on staging after 910 deploys. Fix them alongside or card them. |
| 919 held withdrawals | not started | plan only (`plans/919-PLAN.md`, size L) | Start the pipeline with `--slice QA=30 --slice TEMPER=25`. Owner decision (card comment 6ada4a81): the v5.2 boards B8, B9 and B19 satisfy the design gate. Ship the backend `customerOutcome: "held"` (from backend state, never elapsed time, distinct from failed and unknown) **and** minimal current-app labels with the exact board wording: "Taking longer than usual", then "Cancelled, balance restored" only after the backend confirms. Include the withdrawal reference and the in-context support link. Merge only after 910's staging checks release the staging lock. |
| 996 money safety | not started (top of Backlog) | carded 23 Oct | Unlocking a lock that funds a Held offer escrow must be refused. Verified on main and production b7dcc2ed. Ship to staging. |

Suggested order for the MSI: 917+964 merge and QA → 910 → 919 and 996, alongside the redesign slices.

## 3. Backend gap cards created 22 to 23 Oct (all in Backlog, none started)

972 to 992 (from the v5.2 gap audit; meanings in `906-redesign/BUILD-SCHEDULE.md` section 4), plus:
- **993** closing an offer leaves its lock and offer escrow held
- **994** a second manual lock on one offer returns 500 (the ledger reference index reuses the offer id)
- **995** `GET /wallet/locks` has no trade or escrow id, and unlock releases the newest lock rather than a named one
- **996** unlock frees USDT behind a Held offer escrow (money safety)
- **997** affiliate earnings don't name the referred customer
- **998** `wallets.pending_balance` never tracks customer withdrawals ("Withdrawing" figure)

Closed: **908** moved to Done 23 Oct (already fixed on production by 861 and 864; the redesign added a regression test).

## 4. Staging state left behind

- **Staging lock:** none held.
- **Step-up flag:** `step_up_2fa` on marcello@unlimited-beats.com is OFF, restored 23 Oct by the redesign QA. The 910 session's note says it's still on from 969; that note is older, so check the API once.
- **2FA:** off on starbirdbeats@gmail.com (restored; audit rows 907ca2db, 880f34ca).
- **Failed ledger entry to reconcile:** `failed_ledger_entries` 7035b1ce-3a84-456e-8b82-c7769703e386 (wallet_unlock, 2 USDT, pending) on the QA seller account. This is card 994's bug.
- **920 leftovers on the QA seller:** closed offer 5230d104, cancelled trade 072f50ff, and escrow 7295f895 still `held` with no lock (card 993).
- **Test records kept on purpose:** a support ticket from marcello@unlimited-beats.com, a QA report on offer 4375ae39, a 5-star review on buyer-906, a 3 USDT Nile withdrawal, a 1 USDT transfer from buyer to seller, and a 600 KES trade (cancelled).
- **Review-app QA sessions** (buyer-906, seller-906, third-906) may have expired. Ask the `qic-qa-signin` agent for fresh ones.

## 5. Process fixes made on the Mac that the MSI needs

These were made under the retro rule on 22 to 23 Oct:
1. **jev-qic-ship skill:** a per-worktree cargo target clone; `budget phase --name`; the `--slice` flags; two known-flaky tests (`quote_expiry_864` is CI-only and environment-bound). agentsync should have carried it; diff before running.
2. **`ops/ticket-preparation/qic_prepare.py`:** `scope:` labels no longer invalidate a plan, and an expired CLI login is refreshed once on a 401. These edits are **uncommitted on the Mac** and copied here as `mac-uncommitted/ops-ticket-preparation.patch`. Apply that on the MSI or land it through a PR.
3. **`.claude/scripts/gate.sh`:** the suppression scan now covers new untracked files. The Mac copy is `mac-uncommitted/gate.sh`. The MSI runs a WSL-fixed gate.sh, so merge only that change.
4. **`budget.py`:** the `--slice` flag.
5. **Never share one cargo target dir across worktrees.** Freshness goes by mtime, so tests link other worktrees' code. Give each worktree its own target (a copy of a warm one) and set `CARGO_TARGET_DIR` explicitly.
6. **Frontend:** never use `bun run dev` for previews (it kill -9s whatever holds port 3000). Use `bunx next dev -p <port>`. Builders stop only PIDs they started; a broad `pkill -f next-server` stopped other sessions' servers on 23 Oct.
7. **Disk:** remove a slice worktree as soon as its PR merges, and delete `.next` after each gate. Don't start a builder below 12 GB free.

On the MSI itself: cargo and bun live in WSL. Never run `wsl --terminate` or `--shutdown`, because other sessions' builds and Postgres share it.

## 6. Only Marcello can do

1. The 969 real-money submits on the review app (about 10 minutes).
2. Real-phone checks for 965.
3. Christian's sign-off on the finished redesign, then the merge of `redesign/906`.
4. Any production deploy.

## 7. Files in this folder

- `906-redesign/`: HANDOVER.md (full log), BUILD-SCHEDULE.md, SLICE-BRIEF.md
- `reports/955` to `reports/959`: card plans and per-slice reports. `reports/968`, `969`, `970`: acceptance maps, QA reports and board contradictions.
- `plans/919-PLAN.md`: the current 919 plan
- `ship-runs/910`, `ship-runs/917-964-20261022`: plans, phase JSON and gate receipts. Live logs are left out because they hold one-time tokens.
- `mac-uncommitted/`: the uncommitted `ops/ticket-preparation` patch and the Mac's gate.sh

The v5.2 design pack is already on the MSI (`scratchpad/v5.2/`, republished via REPUBLISH.md), and the PDFs are in `~/Downloads/906-design-pack-v5.2/` on the Mac.
