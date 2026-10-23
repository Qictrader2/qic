# 910 PHASE 2 TEMPER review, round 2 (opus55 independent reviewer)

Verdict: APPROVED

I re-reviewed the whole `git diff origin/main` (8fda17fa) in .scratch/910/wt/qictrader-backend-rs plus the untracked tests/ticket_910_trade_funding_invariant.rs. Diff sha at review time: 190411687dbb; new test file sha e387fe0acf1f.

## Round-1 findings, rechecked

1. MEDIUM (exact amounts), fixed. `assert_funded_trade` (tests/ticket_910_trade_funding_invariant.rs:558) now takes a literal `ExpectedFunding` and asserts escrows.amount, fee_amount, buyer_crypto_amount and wallet_locks.amount exactly. It also checks wallet deltas against the lock and the EscrowLock ledger against the escrow amount. I checked the values against the fixtures and the `Config::test_default()` fees (70 bps platform and vendor-resell):
   - Sell, buy and resell: 250_000_000 + 1_750_000 = 251_750_000.
   - BTC: 500_000 + 3_500 + 10_000 gas reserve = 513_500.
   - Swap: 10_000_000 + 70_000 = 10_070_000.

   These match the AC2 0.7% worked example. On the baseline question: the trades.rs diff changes only the lock conditional and the PATCH guard. `lock_amount`, `escrow_fee`, the quote, the `escrow::create` arguments and the ledger amount are identical to origin/main for lock-backed offers, so values taken from the current fixtures equal the baseline. I accept the builder's inference.
2. LOW (PATCH order), fixed. The `EscrowFunded` refusal now sits after the participant check and before `can_transition_to` (src/api/trades.rs:2631). The new Paid-trade test asserts `PATCH_STATUS_FUND_REFUSAL`, status `paid` and unchanged row counts.
3. LOW (dead code), fixed. `update_escrow_type` and `set_contract_address` are deleted from src/repo/escrow.rs. No references remain, and the `EscrowType` import is still used.

Nothing else in the source changed since round 1, so the round-1 grading of root cause, the 920 contract, unchanged flows, money effects, roles and fixtures stands.

## Gate receipt

phase-2-gate.log ends `GATE FAIL` / `EXIT=1`. The only failure is cargo-test-lib `quote_expiry_864_tests::persisted_database_admission_and_offer_expiry_contract`, which panics with "refusing a database outside the local task or isolated CI allowlist". That is the environment-bound test on the skill's acceptable list. 4314 lib tests passed. All other gate checks passed: suppression scans, migrations, cargo-check, clippy, and cargo-test-targets with 71 of 71 targets, including all 6 changed test files. The gate started at 22:24Z, after the last edits at 21:41Z, so it covers the reviewed tree.

## Findings (none blocking)

| Severity | File:line | Finding | Fix |
|---|---|---|---|
| LOW | frontend/e2e (e.g. smoke/critical-path.test.ts:59, system/full-trade-journey.test.ts:98) | 19 Playwright fixtures use on_chain offers or PATCH escrow_funded. They will fail on staging, and there is no frontend CI to catch it. | Follow-up card or companion frontend PR. |
| LOW | tests/ticket_910_trade_funding_invariant.rs (swap test) | The test drops trades_fiat_amount_check. The owner ruled the bug out of scope, but no card tracks it. | Card the swap constraint bug. |
| LOW | tests/common/mod.rs:547 vs ticket_910 test file:103 | Two wallet-seeding helpers do the same job. | Optional consolidation. |
| LOW | docs/design/.generated/* (6 files) | Regenerated, hash-only changes. | Commit them with the change. |

No HIGH or MEDIUM findings. Required actions: none.
