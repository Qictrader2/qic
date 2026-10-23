# E-910 live check (backend built in target-910, DB qic_631_910_r1_15, port 18910)

Deviations from PLAN.md: none found in behaviour. Notes:
- design-check failed on its first run ("generated design docs were missing or stale and have been refreshed": as-built/drift for merchant-tier, money-recovery-operation, treasury-sol-swap, 6 files under docs/design/.generated). Second run: OK (6 intended docs, 12 generated artifacts). Logs: E-910-design-check.log, E-910-design-check2.log. The refreshed files show as modified in the worktree.
- Escrow fee here is 0.25% (fee_config platform_fee_bps 25), not 0.7%.
- First live attempt used the shared target and is discarded (discarded/). Signup is rate limited to 4 per IP, so users were e910_buyer, e910_seller, e910_poor and e910_ops (admin role by SQL). FX needed several minutes of warm-up before offers worked.
- Step 6 first call used an invalid network name (422 deserialisation, not the refusal); rerun as 6b with ethereum_mainnet is the real result.

Calls and DB before/after: E-910-calls.jsonl, E-910-db.txt.
1. POST /offers on_chain and offer_escrow, unkeyed, on_chain with K1 twice, offer_escrow with K2 twice: all 422 unsupported_escrow_type "This escrow type is not supported. Offers use custodial escrow." offers 0, offer_versions 0 after. money_request_keys: K1 and K2 offer_publish refused 422 unsupported_escrow_type; GET /money-requests/e910-K1 outcome refused.
2. Custodial offer 200 (fabde5ea-a553-4614-8667-a365ad812006). Buyer POST /trades 200 trade f2ef4cb2-b90a-4598-9685-f44f21d1e4d7 status escrow_funded. Seller 900000000/0 -> 849875000/50125000 (locked +50.125 = amount 50 + fee 0.125). wallet_locks 1 of 50125000; escrow custodial/held amount 50000000 fee 125000; one ledger escrow_lock debit 50000000 (ledger_entries 2 -> 3); trade_events trade_created. trade_messages: one system message "You're buying 50.000000 USDT (~818.50 ZAR market value) for 1000.00 ZAR via Bank transfer. Your funds are being securely held in escrow." followed by 4 numbered steps (wait for seller's payment details, pay, mark as paid, seller releases).
3. on_chain offer 26641cd9-90d4-48ca-9354-c33437285de8 (copied by SQL with an offer_versions row). POST /trades unkeyed, keyed K3, K3 repeat: all 422 unsupported_escrow_type. Counts for trades/escrows/wallet_locks/ledger/trade_messages/trade_events/money_request_keys unchanged; balances unchanged (see S3 before/after).
4. Offer by e910_poor (5 USDT) 200 ffb6163c-673e-42ac-b618-8e2a2804fc9b; buyer POST /trades 400 VALIDATION_ERROR "This offer cannot be filled right now - the seller's USDT balance is insufficient."; counts unchanged (S4 before/after).
5. Created trade inserted by SQL (payment_reference QT-E910-CREATED). PATCH /trades/{id}/status escrow_funded by buyer, seller, admin: each 400 "A trade is funded only when it is created. PATCH /status cannot fund a trade."; status stays created, trade_events for it 0 (S5 after).
6. POST /escrow/{funded trade}/link (6b) by buyer, seller, admin: each 422 unsupported_escrow_type; escrow row byte-identical before/after (E-910-db.txt S6b).

Processes stopped: backend (bk910) killed, no cargo running from this worker. Postgres on 5433 is the shared one and was left running.
