<!-- qic-plan
{
  "body_sha256": "80f3ba016559ae9af4fe667bff7e1c7ea7849838698ccb2f4bd76873ebb6def9",
  "budget_minutes": 300,
  "card_id": "6acd983a914864e091d819ad",
  "ci_allowance_minutes": 45,
  "created_at": "2026-10-22T14:07:55.836817+00:00",
  "designs": [],
  "frontend": false,
  "implementation_gates": [],
  "local_evidence": [],
  "risk_areas": [
    "money",
    "authorization",
    "state-transition"
  ],
  "severity": "financial",
  "size": "L",
  "size_table": "2026-10-16",
  "source": {
    "backend": "8fda17fa8f713b5cdc5d1bfef494bf72dd54edc6",
    "frontend": "a73d2708f634a4d9ace4cdcff79ffddd3b64ade2",
    "root": "ecf5b1ba0574a12c4c401314ea7cc4efe3363d89"
  },
  "status": "ready",
  "ticket_hash": "8d9cf0a49b9fdd9ad0ad8796068fe3e5a044c1c55b60b6d49c44ed6275b0803d",
  "track": "FULL",
  "version": 1,
  "watches": [
    {
      "path": "docs/intended-entity-state-machines.md",
      "repo": "backend",
      "requested_path": "docs/intended-entity-state-machines.md",
      "sha256": "bd386f02b2c7b55d5cf36d27da9b29a7c19017edcb29939dfa8137df23f26ecb"
    },
    {
      "path": "migrations",
      "repo": "backend",
      "requested_path": "migrations",
      "sha256": "bdeab595af4c5b0fe3b207ba95fb984042570d3efe2af392b4b081da91fb3d04"
    },
    {
      "path": "src/api/admin.rs",
      "repo": "backend",
      "requested_path": "src/api/admin.rs",
      "sha256": "3812f980f239851bdf9af9fd42442fc7b93ddfb579ef9e191cfdca315396aa16"
    },
    {
      "path": "src/api/escrow.rs",
      "repo": "backend",
      "requested_path": "src/api/escrow.rs",
      "sha256": "d69ac9fbbfbc964e1a5403b6dde876d71e618e44bd0a0bcd3d0ab1d95b855e26"
    },
    {
      "path": "src/api/offers.rs",
      "repo": "backend",
      "requested_path": "src/api/offers.rs",
      "sha256": "2ca5e61961248dbfd193c4eced828062d1d726f6e37f0e962eb4ed48f34d70ba"
    },
    {
      "path": "src/api/reseller.rs",
      "repo": "backend",
      "requested_path": "src/api/reseller.rs",
      "sha256": "c546a3fafbb1b9289ec4bca9521d62b1ae9dbe44f77a0447c80878d040b4a280"
    },
    {
      "path": "src/api/trades.rs",
      "repo": "backend",
      "requested_path": "src/api/trades.rs",
      "sha256": "65e296a5b063bd25aa6e3110fa009cddbc7e8d10884f0a6f8b609851d9011552"
    },
    {
      "path": "src/error.rs",
      "repo": "backend",
      "requested_path": "src/error.rs",
      "sha256": "865a63384399195c68fe7b3089a1111a788f2a3ac4f70f4da76f576659915d23"
    },
    {
      "path": "src/repo/escrow.rs",
      "repo": "backend",
      "requested_path": "src/repo/escrow.rs",
      "sha256": "b59afa3bca6a535176f76e812a8d7b33c5a93855e917f8eb6b3f90ec559fe736"
    },
    {
      "path": "src/repo/money_request.rs",
      "repo": "backend",
      "requested_path": "src/repo/money_request.rs",
      "sha256": "d43a12f02046c30ae04744fd5e4ec013d6061fb0ac7c22056465c37eef1130cc"
    },
    {
      "path": "src/repo/offer.rs",
      "repo": "backend",
      "requested_path": "src/repo/offer.rs",
      "sha256": "cb3a6d342920e57d869a85ccd1e489d36d7c0b3d74fa65fc232a8fb4a4892b9b"
    },
    {
      "path": "src/repo/trade.rs",
      "repo": "backend",
      "requested_path": "src/repo/trade.rs",
      "sha256": "b5188b2b884ab43aeb9705b0ace8548040a6b98ccc6f812532263743674a41d6"
    },
    {
      "path": "src/repo/wallet.rs",
      "repo": "backend",
      "requested_path": "src/repo/wallet.rs",
      "sha256": "1fbdfae3049a08aa3a0ca5945753b0a1cdd0048b6433640df61b2ecf5cdc2d49"
    },
    {
      "path": "src/services/crypto_quote.rs",
      "repo": "backend",
      "requested_path": "src/services/crypto_quote.rs",
      "sha256": "432559e560e01291f8c373b4b25c96f324a97672e92046671925aaeb58eff31f"
    },
    {
      "path": "src/services/escrow_fee_quote.rs",
      "repo": "backend",
      "requested_path": "src/services/escrow_fee_quote.rs",
      "sha256": "2bc8a9f1509d1da8ad19fa1527d8784ada4422fc8ec14f758ce5b4b72aa5973d"
    },
    {
      "path": "src/services/escrow_refund.rs",
      "repo": "backend",
      "requested_path": "src/services/escrow_refund.rs",
      "sha256": "2c41a74354265a59551677d93df70d65901c79823238b26d38ca5e28294490dd"
    },
    {
      "path": "src/services/escrow_release.rs",
      "repo": "backend",
      "requested_path": "src/services/escrow_release.rs",
      "sha256": "7c401b7de59b83193e615be442c8774f45e2e5f0fee4358e8f432ec88b6cac25"
    },
    {
      "path": "src/services/ledger.rs",
      "repo": "backend",
      "requested_path": "src/services/ledger.rs",
      "sha256": "72820fd18b5df6c5c3b8ce9c452f5a53978c136761f98df14a962e408dac19d8"
    },
    {
      "path": "src/services/money_request.rs",
      "repo": "backend",
      "requested_path": "src/services/money_request.rs",
      "sha256": "6fe53c97aef86fcf3d3862cbbd844f38159129f2d16f853208307d035bc13527"
    },
    {
      "path": "src/services/trade_participants.rs",
      "repo": "backend",
      "requested_path": "src/services/trade_participants.rs",
      "sha256": "04d2b4b8dd0c887403c9a90c49cf812b7e268f58183c7a4853b733b4f4b507c3"
    },
    {
      "path": "src/types/enums.rs",
      "repo": "backend",
      "requested_path": "src/types/enums.rs",
      "sha256": "bc77a559661c5d63091d20835a2debf760afabbda044b6da9f3177f211b1ba58"
    },
    {
      "path": "src/types/money_request.rs",
      "repo": "backend",
      "requested_path": "src/types/money_request.rs",
      "sha256": "23b76ba1abaa796531982b0574d9707d8a3c6f6e2162abf35f7801742e1ece3f"
    },
    {
      "path": "tests",
      "repo": "backend",
      "requested_path": "tests",
      "sha256": "6209d85515d2e8f63914eba591f1c404cdaade481215ab40d20076998ad61229"
    },
    {
      "path": "src/hooks/api/use-escrow.ts",
      "repo": "frontend",
      "requested_path": "src/hooks/api/use-escrow.ts",
      "sha256": "eaeb7e6ec5667858e94be95047ed4f325cba7ba0051180b6bc9d8544ecfd0f6d"
    },
    {
      "path": "src/hooks/api/use-trade-detail-page.ts",
      "repo": "frontend",
      "requested_path": "src/hooks/api/use-trade-detail-page.ts",
      "sha256": "817044483ff325bad4a54f408cd8847abce62f0e96de02cdb94f03ddf0216d72"
    },
    {
      "path": "src/hooks/api/use-trades.ts",
      "repo": "frontend",
      "requested_path": "src/hooks/api/use-trades.ts",
      "sha256": "261f1199d18c92e8657b6c11499176b71b79a72ed82f8d9b2ea7e04e35084875"
    },
    {
      "path": "src/services/escrow-api.ts",
      "repo": "frontend",
      "requested_path": "src/services/escrow-api.ts",
      "sha256": "bc93ee9399bd894d55766f5fcbb988b07cbd321d8092b6f07e0ad2d5ac2d9b9d"
    },
    {
      "path": "src/services/offers-api.ts",
      "repo": "frontend",
      "requested_path": "src/services/offers-api.ts",
      "sha256": "16474d3f8155057f2c9c9eda752a935a019e92b404bc77bc9fe3a78b39cf961e"
    },
    {
      "path": "src/services/trades-api.ts",
      "repo": "frontend",
      "requested_path": "src/services/trades-api.ts",
      "sha256": "d4b3da0396c9f82aefb7f2e08c6ea314eac34b7a2fc586f61b257be7960998e9"
    },
    {
      "path": "DESIGN.md",
      "repo": "root",
      "requested_path": "DESIGN.md",
      "sha256": "3c1f6971404627c8cd5d608c698e448965ec88c4b001d5806663d278cd16126d"
    }
  ]
}
-->

# Ticket 910: Trade creation: non-custodial offers can leave a trade unfunded in Created

Severity: financial
Track: FULL
Size: L
Budget: 300 min
CI allowance: 45 min
Size table: 2026-10-16
Risk areas: money, authorization, state-transition
Frontend: False

## Expected behaviour
Sources: ticket 910 Expected behaviour and acceptance criteria; owner decision of 13 Oct 2026 recorded on the card ("on_chain and offer_escrow are not supported product modes. Escrow is not a separate wallet; it is a ledger lock on the seller's balance"); intended design docs/intended-entity-state-machines.md:101 (Created → EscrowFunded by System only) and :156-182 (escrow funded at creation, → Held by System only); 920 replay contract (same doc, 7.1 MoneyRequestKey).

1. Offer creation. POST /api/v1/offers with `escrowType: "on_chain"` or `"offer_escrow"` returns HTTP 422 with `code: "unsupported_escrow_type"` and a message such as "This escrow type is not supported. Offers use custodial escrow." No offer row and no offer version row is written, and no `offer:new` event or notification is sent. With an idempotency key, the refusal is recorded and a replay of the same key and body returns the same 422 and code, still writing nothing. `custodial` (and `btc_wallet_lock`) offers are created exactly as today; the app keeps sending `custodial`.

2. Trade creation. Every trade the server accepts is returned with `status: "escrow_funded"` from POST /api/v1/trades. Worked example (USDT sell offer, default 0.7% escrow fee): the seller has 1,000.000000 USDT available and a buyer opens a trade for 100.000000 USDT. The response status is `escrow_funded`. The seller's available balance becomes 899.300000 and held (`locked_balance`) rises by 100.700000 (trade amount plus the staker's escrow fee, as today). One `wallet_locks` row of 100.700000 is attached to the trade. The escrow row is `custodial`, status `held`, amount 100.000000, fee 0.700000. One EscrowLock ledger entry of 100.000000 is written. If the staker's fee resolves to 0 (merchant tier), available and held move by exactly 100.000000.
   A trade request on an offer row that already carries `on_chain` or `offer_escrow` is refused with HTTP 422 `unsupported_escrow_type` before any write: no trade, escrow, wallet lock, ledger entry, trade event, chat message or completed money request.
   If the staker's available balance is below amount + fee, the request is refused with the existing insufficient-balance 400 and nothing is written (atomic, as today).
   Two simultaneous requests against one offer whose seller can fund only one: exactly one succeeds as `escrow_funded`, the other is refused, the seller's balance never goes negative and no `created` trade remains.

3. Manual funding. PATCH /api/v1/trades/{id}/status with `{"status": "escrow_funded"}` is refused for every caller (buyer, seller, vendor, admin) with HTTP 400 and a message such as "A trade is funded only when it is created. PATCH /status cannot fund a trade." The trade status, escrow row, wallet and events are unchanged and no notification is sent. No admin or service override is kept: the intended design names System at creation as the only actor, and creation now always funds. POST /api/v1/escrow/{trade_id}/link is refused with HTTP 422 `unsupported_escrow_type` for every caller and changes nothing on the escrow.

4. Unchanged flows. Custodial offers (sell and buy), resell trades (sell-side and buy-side), BTC wallet-lock trades and crypto-to-crypto swaps produce the same wallet lock, escrow row, trade status, ledger entries and keyed-request behaviour as before the change. Existing `on_chain`/`offer_escrow` escrow rows keep their release and refund handling so any in-flight row can still settle.

## Diagnosis
Verified by source reading against snapshot fa0e5f1c9ff7d7fb (root ecf5b1ba, backend 8fda17fa, frontend a73d2708). Not reproduced on any shared environment; no staging or production rows have been counted yet. Reconciled with the prior draft (snapshot dbf23dee6dc6f20f): the only watched change since then is ticket 920 (money request idempotency keys), which wraps offer publish and trade start in a keyed transaction but leaves the funding logic, the PATCH status route and the link route unchanged. Line numbers below are current.

1. Entry point. `CreateOfferRequest.escrow_type` (qictrader-backend-rs/src/api/offers.rs:47) deserialises any `EscrowType` variant (`Custodial`, `OnChain`, `OfferEscrow`, `BtcWalletLock`, src/types/enums.rs:782-786). `validate_business_rules` (offers.rs:137-177) only reaches `validate_offer_kyc_compatibility` (offers.rs:191-211), which rejects `OnChain` only when `require_id_doc`/`require_bank_letter` is set and always accepts `OfferEscrow`. Since 920, `create_offer` (offers.rs:1857) parses the optional idempotency key (whose fingerprint includes `escrow_type`, offers.rs:587) and calls `publish_offer_unreplayed` (offers.rs:1930), which runs `validate_business_rules` first (offers.rs:1937) and then persists `req.escrow_type` unchanged (offers.rs:2056) inside the publish transaction. Crypto-to-crypto offers are already forced to `Custodial` by services/crypto_quote.rs, and resell parents with `OnChain`/`OfferEscrow` are refused by api/reseller.rs. Other insert paths hardcode custodial. The frontend always sends `escrowType: "custodial"` (frontend/src/services/offers-api.ts:316, now also sending `idempotencyKey`), so only direct API callers reach the gap.

2. Unfunded trade. POST /trades now routes to `create_trade_request` (api/trades.rs:1419), which loads the offer, parses the optional key and calls the inner `create_trade` (trades.rs:1477). There the wallet lock is taken only when `needs_wallet_lock = is_custodial || is_btc_wallet_lock` (trades.rs:1969-1971, lock at :1988-2050, `else { None }` at :2051-2053). The escrow row is created with the offer's type in status `pending` (trades.rs:2059-2072), the trade row is inserted with the DB default `created` (repo/trade.rs create; migrations/20260301185445_005_create_trades.up.sql:6), and the escrow→`held` / trade→`EscrowFunded` / `record_escrow_lock_in_tx` block runs only under `if needs_wallet_lock` (trades.rs:2132-2172). For `OnChain`/`OfferEscrow` offers the transaction commits (trades.rs:2198) with trade `created`, escrow `pending`, no `wallet_locks` row and no ledger entry, and the buyer still receives the system message "Your funds are being securely held in escrow" (trades.rs:2290). With a key, 920 also marks the money request `completed` for that unfunded trade (trades.rs:2174-2185). The integration test `create_trade_on_chain_success` (tests/trade_creation_mpr013.rs:122-236, OnChain offer at :165) asserts this outcome, so the gap is codified, not accidental.

3. Hand funding. `update_trade_status` (PATCH /trades/{id}/status, trades.rs:2613) admits buyer, seller, vendor or admin (trades.rs:2625-2633), checks only `can_transition_to` (trades.rs:2636; `Created → EscrowFunded` is legal in enums.rs:663), refuses only Released/Disputed/Cancelled targets (trades.rs:2650-2672), and the role match has an empty arm for `EscrowFunded` (trades.rs:2714). It then writes the status (trades.rs:2717) with no lock check. A participant can therefore move an unfunded `created` trade to `escrow_funded`, after which `mark_paid` (trades.rs:4594) and release paths treat it as funded. The intended design allows `Created → EscrowFunded` only for System at creation (qictrader-backend-rs/docs/intended-entity-state-machines.md:101) and escrow `→ Held` only for System (same doc :182).

4. Related hazard (source-verified, not reproduced): `link_onchain_escrow` (POST /escrow/{trade_id}/link, api/escrow.rs:1455, route :2349) lets any buyer, seller, vendor or admin rewrite an existing escrow's type to `OnChain` (escrow.rs:1477-1479) and set contract/deposit fields (:1482-1485) whatever the escrow's current type or status. On a funded custodial trade that switches release to `release_on_chain` (services/escrow_release.rs) and refund to `refund_on_chain` (services/escrow_refund.rs) instead of the ledger-lock paths. The route has no app caller: `useLinkEscrow` (frontend/src/hooks/api/use-escrow.ts:75) is never imported. It is fix-sketch item 4 and is in scope as a refusal because it contradicts the owner's 13 Oct 2026 rule.

5. Other paths checked and unaffected: `create_crypto_swap_trade` (trades.rs:1080) always locks, creates a `Custodial` escrow and sets `EscrowFunded` in the same transaction (trades.rs:1309-1315); 920 only wrapped it in the keyed claim. `lock_balance_for_trade_on` (repo/wallet.rs) is a conditional `UPDATE ... WHERE balance >= $1`, so concurrent creations serialise on the wallet row and cannot overdraw; no concurrent-creation test exists. The only `TradeStatus::EscrowFunded` writers are trades.rs:1315, :2140 and the PATCH at :2717. The offer-level route `create_offer_escrow` (escrow.rs:968) writes an `OfferEscrow` escrow row tied to an offer, never a trade, and now funds it in a savepoint inside a keyed transaction; its new frontend hook `useCreateOfferEscrow` (use-escrow.ts:42) is used only by a 920 test.

6. 920 interaction. Business refusals raised inside `create_trade` or `publish_offer_unreplayed` are recorded against a supplied key only if `services/money_request.rs::is_recordable_refusal` (:247-295, an exhaustive match) returns true, and replays re-render the stored HTTP status and code through `AppError::ReplayedRefusal` (error.rs:479, `rendered_code` :674). A new `AppError` variant therefore has to be added to that match, to `rendered_code`'s list and to the existing status/code/message matches, or the crate will not compile.

Root cause: trade funding was modelled as optional per escrow type (to leave room for on-chain/offer-escrow modes that the owner has since ruled are not product modes), and the generic PATCH status route was never restricted to exclude the system-only funding transition.

## Scope
Included (backend only, qictrader-backend-rs):
1. Offer creation refuses `OnChain` and `OfferEscrow` with a stable error code (fix sketch 1).
2. `create_trade` refuses offers whose escrow type is not lock-backed before any write, and the lock branch becomes mandatory so no code path can commit a trade without a lock (fix sketch 2).
3. PATCH /trades/{id}/status refuses the `escrow_funded` target for every caller (fix sketch 3; no admin path kept, matching the intended design).
4. POST /escrow/{trade_id}/link is refused (fix sketch 4, the 'flag as unsupported' option) because it rewrites funded escrows to an unsupported type.
5. The new error is a recordable refusal under the 920 idempotency contract, so keyed requests replay it consistently.
6. Integration-test fixtures that relied on `OnChain` offers to open trades without funding a wallet move to funded custodial offers; new ticket tests cover every acceptance case.
7. The intended entity design doc records the 13 Oct 2026 owner rule.

Excluded:
1. Removing the `OnChain`/`OfferEscrow` enum variants, the `escrow_type` Postgres enum values, `release_on_chain`/`refund_on_chain`, or the `link_onchain_escrow` handler body. Existing rows may still reference them; removal waits for the diagnosis counts and is a follow-up (the ticket recommends item 4 only after counts show no live rows). No migration in this ticket.
2. Any data change to existing offers or trades (pausing live unsupported offers, cancelling stale `created` trades). If counts show live rows, that is a separate owner-approved operation.
3. The offer-level escrow route POST /escrow/offer/create (escrow.rs:968), which writes `OfferEscrow` escrow rows tied to an offer, never a trade (see risks).
4. Frontend: no change. The app already sends `custodial` (frontend/src/services/offers-api.ts:316), never sends `escrow_funded` to PATCH /status (`handleUpdateStatus` in frontend/src/hooks/api/use-trade-detail-page.ts:668 is defined but never returned or called) and never calls the link route (`useLinkEscrow` is never imported). Since 920 the create-trade error path throws `ApiResponseError` carrying the backend code (frontend/src/services/trades-api.ts:329), so the new 422 surfaces as a normal error. The card 906 redesign's 'Funded first' trade status becomes true server-side. No design package is required because nothing renders differently.
5. Restricting `btc_wallet_lock` to BTC offers, buyer-driven escrow `Held` updates on `custodial_deposit`/`confirm_trade_deposit`, and `create_trade_escrow` (see risks).

Dependencies and rollout: backend deploy only, no migration, no config. Builds on 920 (already in the backend pin). Forward-compatible with old frontend builds. Before closing, the read-only diagnosis counts on staging and production must be recorded on the card (mandatory QA). Production counting is separately authorised release work: agents stop at staging and hand the production read to the owner. The counts are not an implementation gate because the refusals are correct whatever they show.

## Implementation
1. qictrader-backend-rs/src/error.rs: add `AppError::UnsupportedEscrowType` following the `UnsupportedNetwork` pattern: variant near :252, `status_code` arm near :645 → `StatusCode::UNPROCESSABLE_ENTITY`, add it to the `rendered_code` passthrough list (:674-723), `error_code` arm near :767 → "unsupported_escrow_type", `user_message` arm near :850 → "This escrow type is not supported. Offers use custodial escrow." Add a render unit test like `unsupported_network_is_422_with_stable_code` (:1206).
2. qictrader-backend-rs/src/services/money_request.rs: add `AppError::UnsupportedEscrowType` to the `true` arm of `is_recordable_refusal` (:248-253) as a business refusal, and add an assertion to the `recordable` unit test (:338-366). This keeps keyed offer-publish and trade-start replays returning the same 422 code.
3. qictrader-backend-rs/src/types/enums.rs: add `EscrowType::is_lock_backed(&self) -> bool` (true for `Custodial | BtcWalletLock`) next to the enum at :782 with a unit test over all four variants, so offers.rs, trades.rs and escrow.rs share one definition of a supported mode.
4. qictrader-backend-rs/src/api/offers.rs: in `CreateOfferRequest::validate_business_rules` (:137) return `AppError::UnsupportedEscrowType` when `!self.escrow_type.is_lock_backed()`, before the KYC compatibility call (:155). `publish_offer_unreplayed` calls it first (:1937), before the publish transaction, so nothing is written. Keep `validate_offer_kyc_compatibility` (:191-211) unchanged because `update_offer` still calls it on stored rows (:2213). Update the unit test at :3442-3456 (on_chain + requireBankLetter) to assert the `unsupported_escrow_type` code, and add `offer_escrow` without KYC flags.
5. qictrader-backend-rs/src/api/trades.rs inner `create_trade` (:1477): immediately before the crypto-swap dispatch (:1633), after the existing eligibility checks so paused, self-trade, blocked and suspended refusals keep their messages, return `AppError::UnsupportedEscrowType` when `!offer.escrow_type.is_lock_backed()`. This precedes the quote/FX fetch, the keyed claim (:1962) and the transaction (:1957), so nothing is written and a keyed caller gets a recorded refusal. Then make funding mandatory: replace `needs_wallet_lock` (:1971) with the lock-backed invariant so the `else { None }` arm (:2051-2053) and the conditional block (:2132-2172) cannot commit an unfunded trade (for example, return `AppError::internal` inside the `business` block if `wallet_lock_id` is `None` before the keyed `complete` at :2174, so the rollback at :2193 runs). Keep `is_custodial`/`is_btc_wallet_lock` for the BTC gas reserve (:1979-1985) and the later post-commit branch (:2436). Leave `create_crypto_swap_trade` (:1080) and `create_trade_request` (:1419) unchanged.
6. qictrader-backend-rs/src/api/trades.rs `update_trade_status` (:2613): next to the Released/Disputed/Cancelled refusals (:2650-2672) add `if req.status == TradeStatus::EscrowFunded { return Err(AppError::validation(PATCH_STATUS_FUND_REFUSAL)); }` with a new `pub const PATCH_STATUS_FUND_REFUSAL` beside `PATCH_STATUS_CANCEL_REFUSAL` (:2870). It applies to admins too and returns before `update_status` (:2717), the event (:2720) and notifications. Leave `TradeStatus::can_transition_to` (enums.rs:663) unchanged because `create_trade` (:2134) and the swap path (:1309) depend on it. Keep the exhaustive role arm at :2714 compiling (the `EscrowFunded` value becomes unreachable there).
7. qictrader-backend-rs/src/api/escrow.rs `link_onchain_escrow` (:1455): after the participant check (:1465-1473) return `AppError::UnsupportedEscrowType` without calling `update_escrow_type`, `set_contract_address` or `set_deposit` (:1477-1485). Keep the route registered (:2349) so callers get a stable 422 rather than a 404. Do not touch the `release_on_chain`/`refund_on_chain` dispatch in services/escrow_release.rs and services/escrow_refund.rs, or `create_offer_escrow` (:968).
8. Test fixtures that open trades on `OnChain` offers without a funded wallet move to `EscrowType::Custodial` with a seeded staker wallet (amount + fee): tests/trade_creation_mpr013.rs (`create_trade_on_chain_success` :122-236 becomes a refusal test asserting 422 `unsupported_escrow_type` and zero trade/escrow/lock/ledger rows; the paused :541 and self-trade :631 fixtures are refused earlier and may stay, but switch them to custodial for clarity), tests/mpr001_trade_escrow_ledger_invariants.rs (:162, :299), tests/mpr002_offer_version_trade_snapshot.rs (:175, offer created through the API with "on_chain"), tests/uz_seller_escrow_fee_quote.rs (`make_offer` near :183; the taker wallet must cover amount + fee on Buy offers), tests/ticket_741_merchant_tier_override_grants.rs (`make_offer` near :1795). Leave tests/mpr003_index_premium.rs (:91, no trade), tests/escrow_offer_auto_fund.rs (:150, offer-level escrow row) and the escrow_release.rs/escrow_refund.rs tests (legacy escrow rows inserted directly). Add a shared funded-wallet helper in tests/common/mod.rs if two or more files need it.
9. New tests/ticket_910_trade_funding_invariant.rs (sqlx, runs in CI with REQUIRE_TEST_DB) covering every acceptance case below: the concurrent case (two `tokio::join!`ed POST /trades from two buyers against one seller funded for one trade), keyed replays of the 422 for offer publish and trade start, and regressions for custodial sell and buy offers, a sell-side and a buy-side resell, a BTC wallet-lock offer and a crypto-to-crypto swap, each asserting lock amount, escrow row, trade status and ledger entries.
10. qictrader-backend-rs/docs/intended-entity-state-machines.md: in 1.1 (role guards at :101) note that PATCH /status never funds a trade, and in 1.2/1.3 (:121, :156-182) record the owner rule of 13 Oct 2026 that only `custodial` and `btc_wallet_lock` are product escrow modes; `on_chain`/`offer_escrow` remain only for legacy rows and the offer-level escrow route.
11. Targeted local checks: `cargo check`, `cargo clippy` on the crate, unit tests in enums.rs/error.rs/money_request.rs/offers.rs/trades.rs, and the new and edited integration test files against the local Postgres on 5433. The full suite runs once in CI.
12. After the staging deploy, run the read-only diagnosis counts on staging and hand the production read to the owner. Queries: `SELECT escrow_type, status, count(*) FROM offers WHERE escrow_type IN ('on_chain','offer_escrow') GROUP BY 1,2`; `SELECT t.status, e.escrow_type, e.status, count(*) FROM trades t LEFT JOIN escrows e ON e.id = t.escrow_id WHERE t.status = 'created' GROUP BY 1,2,3`; `SELECT escrow_type, status, (trade_id IS NULL) AS offer_level, count(*) FROM escrows WHERE escrow_type IN ('on_chain','offer_escrow') GROUP BY 1,2,3`. Record the results on the card.

## Risks
- Money: create_trade's lock/escrow/ledger block is restructured to make funding mandatory, and it now sits inside 920's keyed `business` block. A mistake could change lock amounts (amount + fee, + BTC gas reserve for btc_wallet_lock), the escrow fee or the EscrowLock ledger amount for custodial, resell or BTC trades, or leave a keyed claim without a rollback. AC4 regression tests must compare wallet balance, locked_balance, wallet_locks.amount, escrows.amount/fee_amount/status and ledger entries before and after.
- Authorization: PATCH /trades/{id}/status loses the escrow_funded target for every role, including admin, and POST /escrow/{trade_id}/link is refused for every role. Any undocumented operator script or support playbook that hand-funded trades or linked on-chain escrows stops working. No app caller exists (verified in the frontend snapshot).
- State machine: TradeStatus::can_transition_to keeps Created → EscrowFunded because the two system funding paths use it. The guard lives in the PATCH handler, so a future generic status route could reopen the gap; the new test pins the PATCH refusal.
- Idempotency (920): the new error must be added to the exhaustive matches in error.rs and services/money_request.rs. Classifying it as recordable means a keyed request that hit the 422 replays the same 422 for that key forever. That is correct for a business refusal, but a caller who fixes the body must use a new key (a changed body under the same key already returns 409 idempotency_key_reused).
- Legacy rows: offers already stored with on_chain/offer_escrow become untradeable (422) but stay listed until someone pauses them. Existing trades in created with escrow pending can still be cancelled by the buyer or expire; they can no longer be hand-funded. Existing on_chain/offer_escrow escrow rows keep their release/refund paths. Counts are unknown until the diagnosis runs; any data clean-up is separate owner-approved work.
- Wording mismatch: the ticket's AC2 QA line says held balance rises by 'the trade amount'. Current, intended behaviour (intended-entity-state-machines.md 3.6.1, unchanged by AC4) locks amount + the staker's escrow fee, while the EscrowLock ledger entry records the amount only. QA must assert held delta = escrows.amount + escrows.fee_amount and ledger entry = escrows.amount, not a single figure.
- USDT-only QA rule (owner, 2026-10-06): the ticket's mandatory BTC wallet-lock regression is covered by the CI integration test only, with no staging BTC asset QA. Hands-on staging QA uses USDT custodial offers.
- Test fixture churn: five integration test files used OnChain offers to open trades without funding a wallet. Moving them to funded custodial offers changes their setup, not what they assert. Reviewers should check that no assertion was weakened.
- Out-of-scope observations, not fixed here: (a) btc_wallet_lock is accepted for non-BTC offers and would add btc_gas_reserve_sats to a USDT lock (trades.rs:1979-1985); (b) custodial_deposit and confirm_trade_deposit (escrow.rs:489, :812) let the buyer set an escrow to held with no lock, though they never change the trade status; (c) create_trade_escrow (escrow.rs:1405) can attach a fresh custodial escrow to a trade that has none; (d) POST /escrow/offer/create (escrow.rs:968) still writes OfferEscrow escrow rows for an offer-level lock and, when the wallet is short, generates a dedicated deposit wallet. It never creates or funds a trade, so it does not affect these criteria. It does sit awkwardly with the 13 Oct rule and is worth its own ticket.

## Acceptance and evidence

### AC1: An offer that uses an escrow mode the platform does not support is refused when it is created, with a clear error to the caller.
- Setup: KYC-verified seller with a funded USDT wallet calls POST /api/v1/offers directly with a valid sell-offer body and escrowType "on_chain"; repeat with "offer_escrow". Run once without an idempotency key and once with a key, then replay the keyed call with the same key and body.
  Expected: Each call returns HTTP 422 with code "unsupported_escrow_type" and a readable message, including the keyed replay. No offers row and no offer_versions row exists for the seller afterwards; no offer:new event or 'Offer Published' notification is emitted; the keyed money_request_keys row is state refused with status 422, never completed.
  Evidence: CI integration test in tests/ticket_910_trade_funding_invariant.rs counting offers, offer_versions and money_request_keys rows before and after; staging QA curl transcript of both unkeyed calls with the QA seller account plus a read-only row count.
- Setup: Offer row inserted directly (repo::offer::create) with escrow_type on_chain, then a second with offer_escrow, both active; a buyer calls POST /api/v1/trades on each, once without and once with an idempotency key.
  Expected: Each request returns HTTP 422 unsupported_escrow_type. Trades, escrows, wallet_locks, ledger entries, trade_events and trade_messages counts are unchanged; seller wallet balance and locked_balance are unchanged; no money_request_keys row is completed.
  Evidence: CI integration test (replaces create_trade_on_chain_success in tests/trade_creation_mpr013.rs) asserting row counts and wallet balances before and after.
- Setup: Seller creates a USDT custodial sell offer through the app on staging, and a second custodial offer through POST /api/v1/offers with escrowType "custodial".
  Expected: Both offers are created and listed exactly as before (HTTP 200, escrowType custodial, status active).
  Evidence: CI integration test for the API path; staging QA screenshot of the app offer-creation success and the curl response for the API path.
- Setup: POST /api/v1/escrow/{trade_id}/link by the buyer, the seller and an admin on a funded custodial trade, with a contract address and tx hash.
  Expected: Each call returns HTTP 422 unsupported_escrow_type. The escrow row keeps escrow_type custodial, status held, and contract_address, deposit_address and deposit_tx_hash unchanged.
  Evidence: CI integration test reading the escrow row before and after.
- Setup: Diagnosis (mandatory QA): read-only count of offers with escrow_type on_chain or offer_escrow, grouped by status, on staging and production, plus escrows with those types grouped by status and by whether they are trade-linked or offer-level.
  Expected: Counts recorded on the Trello card before closing, with environment and timestamp. Production read performed by or handed to the owner.
  Evidence: Card comment with the SQL text and results for staging (agent) and production (owner-run or owner-authorised read-only query).

### AC2: No trade can open without the seller's USDT being locked in the same step, so no customer sees an open trade that is not funded.
- Setup: USDT custodial sell offer; seller wallet 1,000 USDT available; buyer opens a trade for about 100 USDT via POST /api/v1/trades.
  Expected: Response status "escrow_funded". Seller available balance falls by escrows.amount + escrows.fee_amount and locked_balance rises by the same; one wallet_locks row of that amount is attached to the trade; escrow row custodial, status held; one EscrowLock ledger entry equal to escrows.amount.
  Evidence: CI integration test asserting the response and wallet, wallet_locks, escrows and ledger rows; staging QA with QA buyer and seller accounts (USDT) showing the trade page and wallet before and after.
- Setup: Custodial offer whose seller's available balance is below amount + fee; buyer calls POST /api/v1/trades.
  Expected: HTTP 400 with the existing insufficient-balance message. No trade, escrow, wallet_locks or ledger row is written; seller balance and locked_balance unchanged.
  Evidence: CI integration test (extends create_trade_custodial_insufficient_balance_returns_400 at tests/trade_creation_mpr013.rs:240) asserting zero new rows.
- Setup: Seller funded for exactly one trade; two different buyers send POST /api/v1/trades on the same offer at the same time (tokio::join!), each with its own idempotency key.
  Expected: Exactly one response is escrow_funded and the other is refused. Seller balance never goes below zero, locked_balance equals one lock, exactly one trade exists for the offer and no trade is in created.
  Evidence: CI integration test in tests/ticket_910_trade_funding_invariant.rs, run repeatedly in a loop inside the test to catch races.
- Setup: Every lock-backed creation path (custodial sell, custodial buy, sell-side resell, buy-side resell, BTC wallet-lock, crypto swap) opened once each.
  Expected: No created response appears; every trade row has status escrow_funded and an attached wallet_locks row written in the same transaction.
  Evidence: CI integration test asserting status and lock presence per path.
- Setup: Diagnosis (mandatory QA): read-only count of trades in status created joined to their escrow type and escrow status, on staging and production.
  Expected: Counts recorded on the card before closing; any nonzero result is listed for owner follow-up rather than altered.
  Evidence: Card comment with the SQL and results for staging (agent) and production (owner-run or owner-authorised).

### AC3: A trade cannot be marked funded by hand, so funded always means the seller's USDT is locked.
- Setup: A trade row in created (inserted directly to simulate a legacy unfunded trade); PATCH /api/v1/trades/{id}/status {"status":"escrow_funded"} sent by the buyer, by the seller, by the vendor of a resell trade and by a non-party admin.
  Expected: Every call returns HTTP 400 with the PATCH_STATUS_FUND_REFUSAL message. Trade status stays created; no status_changed trade event, no notification, and no escrow or wallet change.
  Evidence: CI integration test iterating the four actors and asserting status, trade_events and notifications counts.
- Setup: Admin or service path: confirm none is kept. A non-party admin PATCHes escrow_funded on a created trade; then a custodial trade is created normally.
  Expected: The admin PATCH is refused like any participant. The only route to escrow_funded is trade creation, which writes the wallet lock first, then the status, and logs the trade_created event after commit.
  Evidence: CI integration test plus code review that trades.rs:1315 (swap) and the create_trade funding block (:2140) are the only EscrowFunded writers.
- Setup: Funded custodial trade; buyer calls POST /api/v1/escrow/{trade_id}/link to rewrite the escrow to on_chain before completion; seller then completes the trade.
  Expected: The link call is refused with 422 and the escrow stays custodial; completion releases through the custodial path and clears the wallet lock exactly as for any custodial trade.
  Evidence: CI integration test asserting escrow_type after the link attempt and wallet_locks.released_at after completion.

### AC4: Custodial offers, resells and crypto swaps are created and funded exactly as they are today.
- Setup: Regression: sell-side resell offer (reseller markup over a custodial parent) and a buy-side resell; a buyer opens a trade on each.
  Expected: Each trade is escrow_funded; lock owner, lock amount, escrows.amount, escrows.fee_amount, buyer_crypto_amount and the EscrowLock ledger entry equal the values produced before this change for the same inputs.
  Evidence: CI integration test with fixed price and fee config asserting the exact minor-unit values.
- Setup: Regression: BTC offer with escrow_type btc_wallet_lock and a seller BTC wallet covering amount + fee + btc_gas_reserve_sats.
  Expected: Trade escrow_funded; lock amount = amount + fee + btc_gas_reserve_sats; escrow btc_wallet_lock, held; EscrowLock ledger entry = escrows.amount.
  Evidence: CI integration test only (no staging BTC asset QA under the 2026-10-06 USDT-only rule).
- Setup: Regression: crypto-to-crypto swap offer (custodial, fixed price); buyer holds the quote currency and calls POST /api/v1/trades with baseAmount and an idempotency key, then replays the same key.
  Expected: Trade escrow_funded; seller lock = legs.base_locked; escrow custodial, held, amount legs.base_amount, fee legs.base_fee; EscrowLock ledger entry = legs.base_amount; response unchanged; the replay returns the same trade and writes nothing new.
  Evidence: CI integration test asserting the swap legs and rows.
- Setup: Ledger reconciliation for a new custodial USDT trade: compare the EscrowLock ledger entry, the escrow row and the seller's locked_balance delta.
  Expected: Ledger entry = trade crypto amount in minor units = escrows.amount, and locked_balance delta = escrows.amount + escrows.fee_amount = wallet_locks.amount.
  Evidence: CI integration test; staging QA read-only query of the new trade's ledger, escrow and wallet_locks rows after the USDT trade in AC2.
- Setup: Custodial USDT sell offer and buy offer on staging, each traded by QA accounts through the app up to release.
  Expected: Both trades open as Funded, can be marked paid and released, and wallet balances settle as before.
  Evidence: Staging QA screenshots of the trade pages and wallet balances (USDT) with the QA buyer and seller accounts.

## Designs
No frontend work.

## Design decision
Not applicable.

## Implementation approval
No additional implementation approval gate recorded by preparation.
