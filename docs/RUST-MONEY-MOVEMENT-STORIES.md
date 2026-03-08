# Rust Backend — Money Movement Story List

> **Date:** 2026-03-06
> **Goal:** Make the Rust backend (`qictrader-backend-rs/`) move real crypto, collect the 1% platform fee, and give investors visibility into revenue.
> **Source:** Gap analysis from `docs/BACKEND-PROFIT-GAP-ANALYSIS.md` and `docs/NODE-BACKEND-ISSUES.md`, plus full audit of both backends.
> **Non-negotiable:** The system must move funds, take profit, and send it to the right accounts.
> **No stubs. No mocked data. No shortcuts.** Every story ships with real logic, real tests, and a clean deploy.

---

## Workflow — Every Story

Every story follows this exact process. No exceptions.

```
1. IMPLEMENT   — Write the real code. No stubs, no hardcoded responses, no mimicked data.
2. UNIT TEST   — Rust #[cfg(test)] tests for every function. Assert specific values.
3. INTEG TEST  — tests/ integration tests against a real DB (and testnet where applicable).
4. E2E TEST    — Puppeteer/Playwright tests that hit the API and verify the full flow.
5. FULL SUITE  — Run `cargo test` + `npm run test:e2e`. Every existing test must still pass.
6. DEPLOY      — Deploy to staging. Commit message: "STORY-XX: <title>".
7. VERIFY      — Smoke test on staging. Confirm real behaviour, not just green tests.
8. NEXT STORY  — Only after all above pass. Never skip ahead.
```

**Test rules:**
- Unit tests assert exact amounts, exact statuses, exact tx hashes — not just "status != error"
- Integration tests use a real Postgres database, not mocks
- Blockchain tests use testnets (Solana devnet, Tron Nile, Sepolia, BTC testnet) — real signing, real broadcast, real confirmation
- E2E tests (Puppeteer) cover the frontend → API → blockchain round-trip where applicable
- If a test fails, fix the code. Never skip or `#[ignore]` a failing test.

---

## Current State of Rust Backend

**What exists (read-only):**
- Balance checks on-chain (TRON TRC-20, ETH, SOL, TRX) via RPC
- Gas/energy estimates (all chains)
- Key derivation and wallet generation (BIP-39, HD derivation for ETH/BTC/SOL/TRON)
- Custodial wallet creation with encrypted private keys
- Escrow DB state machine (create, update status, release/refund — DB only)
- Trade DB state machine (create, complete, cancel — DB only)
- Wallet lock/unlock DB operations
- Ledger `create_entry()` function (exists, never called)
- RPC URLs in config (ETH, SOL, BTC, TRON)

**What does NOT exist:**
- No transaction signing on any chain
- No transaction broadcasting on any chain
- No blockchain SDK dependencies (no ethers, no solana-sdk, no bitcoin crate)
- No platform fee addresses configured
- No platform fee percentage wired (hardcoded to 0)
- No escrow release that moves crypto
- No escrow refund that moves crypto
- No wallet withdrawal that moves crypto
- No lock release on trade completion or cancellation
- No ledger entries created during trade flow
- No admin fee aggregation

---

## Priority: IMMEDIATE (System Cannot Function Without These)

### STORY-01: Platform Fee Configuration

**What:** Add platform fee addresses and percentage to config, readable from env vars.

**Why:** Without this, the system has no destination for profit and no fee rate.

**Acceptance criteria:**
- [ ] Env vars: `PLATFORM_FEE_BPS` (default 100 = 1%), `PLATFORM_FEE_ADDRESS_BTC`, `PLATFORM_FEE_ADDRESS_ETH`, `PLATFORM_FEE_ADDRESS_SOL`, `PLATFORM_FEE_ADDRESS_TRON`
- [ ] Config struct in `src/config.rs` with all fee addresses and bps
- [ ] `AppState` exposes fee config
- [ ] `build_quote_input` in `services/quote.rs` reads `platform_fee_bps` from config instead of hardcoded `0`
- [ ] Fee amount is calculated and stored on trade and escrow at creation time

**Node reference:** `config/platformFees.ts` — but we fix the "hardcoded in source" issue by using env vars.

**Tests:**
- [ ] Unit: `Config::from_env()` parses all fee env vars correctly, defaults `PLATFORM_FEE_BPS` to 100
- [ ] Unit: `Config::from_env()` fails if fee address is missing for any chain
- [ ] Unit: `build_quote_input` returns `platform_fee_bps` from config, not 0
- [ ] Unit: `calculate_quote` produces correct `fee_amount` (e.g. 100 USDT trade → 1 USDT fee)
- [ ] Integration: create a trade via API → escrow `fee_amount` is non-zero
- [ ] Integration: `GET /api/v1/platform-config` returns correct fee percentage

**Estimated scope:** Small — config + quote wiring.

---

### STORY-02: Blockchain Transaction Signing & Broadcasting — Solana (SPL + SOL)

**What:** Implement Solana transaction signing and broadcasting for SPL tokens (USDT/USDC) and native SOL.

**Why:** Solana is the primary chain for USDT trades. This is the most critical chain to get working first.

**Acceptance criteria:**
- [ ] Add `solana-sdk`, `solana-client`, `spl-token` crate dependencies
- [ ] New service: `src/services/blockchain/solana.rs`
- [ ] `send_spl_token(private_key, to_address, mint_address, amount, decimals, fee_address, fee_bps)` → returns tx signature
- [ ] `send_sol(private_key, to_address, amount, fee_address, fee_bps)` → returns tx signature
- [ ] **Single atomic transaction** with two transfer instructions (recipient + fee) — NOT two separate txs
- [ ] ATA creation if destination/fee ATAs don't exist
- [ ] Rent-exempt minimum check (890,880 lamports)
- [ ] Retry logic (3 attempts, exponential backoff) for transient RPC errors
- [ ] Returns `{ signature, recipient_amount, fee_amount }`

**Node reference:** `services/solana/splToken.ts` `executeSPLTransfer` — already atomic, copy the approach.

**Key constants:**
```
SOL_RENT_EXEMPT_MINIMUM = 890_880 lamports
SOL_TRANSACTION_FEE = 5_000 lamports
USDT_MINT = Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
USDC_MINT = EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

**Tests:**
- [ ] Unit: fee calculation — 100 USDT with 100 bps → 99 USDT to recipient, 1 USDT to fee address
- [ ] Unit: fee calculation — 0 bps → 100% to recipient, no fee instruction
- [ ] Unit: ATA derivation matches known addresses
- [ ] Integration (devnet): `send_spl_token` with real devnet USDT mint → tx confirms, recipient ATA receives tokens, fee ATA receives tokens
- [ ] Integration (devnet): `send_sol` → tx confirms, both recipients receive correct lamport amounts
- [ ] Integration (devnet): retry logic — simulate transient RPC timeout, verify retry succeeds
- [ ] Integration (devnet): insufficient balance → returns error, no partial send
- [ ] Integration (devnet): rent-exempt check — tiny SOL amount → error, not silent failure

**Estimated scope:** Medium — new crate deps + service implementation.

---

### STORY-03: Blockchain Transaction Signing & Broadcasting — Tron (TRC-20 + TRX)

**What:** Implement Tron transaction signing and broadcasting for TRC-20 tokens (USDT/USDC) and native TRX.

**Why:** Tron is the second most used chain for USDT.

**Acceptance criteria:**
- [ ] Tron transaction signing via raw HTTP to TronGrid/QuickNode (no Rust TronWeb SDK exists — use `triggersmartcontract` + `broadcasttransaction` JSON-RPC)
- [ ] New service: `src/services/blockchain/tron.rs`
- [ ] `send_trc20(private_key, to_address, contract_address, amount, decimals, fee_address, fee_bps)` → returns tx hash
- [ ] `send_trx(private_key, to_address, amount, fee_address, fee_bps)` → returns tx hash
- [ ] **FIX from Node:** Make fee split atomic where possible, or implement retry-safe logic that checks balances before retry (Node has non-atomic two-tx split that can lose money)
- [ ] Fee limit: 100 TRX (100,000,000 sun) per contract call
- [ ] Returns `{ tx_hash, recipient_amount, fee_amount }`

**Node reference:** `services/tron.ts` — but FIX the non-atomic fee split (CRIT-2).

**Key constants:**
```
TRON_USDT_CONTRACT = TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t
TRON_USDC_CONTRACT = TEkxiTehnzSmSe2XqrBj4w32RUN966rdz8
TRON_FEE_LIMIT = 100_000_000 sun
```

**Tests:**
- [ ] Unit: fee split math — 100 USDT at 100 bps → 99 to recipient, 1 to fee
- [ ] Unit: raw amount conversion — 100.5 USDT × 10^6 = 100_500_000 sun
- [ ] Integration (Nile testnet): `send_trc20` → tx hash returned, both recipients receive correct amounts
- [ ] Integration (Nile testnet): `send_trx` → tx confirms
- [ ] Integration (Nile testnet): insufficient TRX for gas → clear error before any tx is sent
- [ ] Integration: retry-safe — if second tx fails, verify first tx amount and state are recoverable

**Estimated scope:** Medium-Large — raw JSON-RPC, protobuf signing.

---

### STORY-04: Blockchain Transaction Signing & Broadcasting — Bitcoin

**What:** Implement Bitcoin PSBT-based transaction building, signing, and broadcasting.

**Why:** Required for BTC escrow release.

**Acceptance criteria:**
- [ ] Add `bitcoin` crate (rust-bitcoin), `secp256k1` crate dependencies
- [ ] New service: `src/services/blockchain/bitcoin.rs`
- [ ] `send_btc(private_key, from_address, to_address, fee_address, fee_bps, sweep_all)` → returns txid
- [ ] **Single atomic transaction** with multiple outputs (recipient + fee) — same as Node
- [ ] Dynamic fee rate from mempool.space API (`halfHourFee` + 10% buffer, capped at 500 sat/vbyte, fallback 15)
- [ ] UTXO fetching from QuickNode/Blockstream
- [ ] Dust threshold: skip fee output if < 546 sats (but LOG it for accounting)
- [ ] P2WPKH (bech32) signing
- [ ] Returns `{ txid, recipient_amount, fee_amount, network_fee }`

**Node reference:** `services/bitcoin.ts` `sendBTCFromPrivateKey` — already atomic, copy the approach.

**Key constants:**
```
BTC_DUST_THRESHOLD = 546 sats
BTC_DEFAULT_FEE_RATE = 15 sats/vbyte
BTC_MAX_FEE_RATE = 500 sats/vbyte
```

**Tests:**
- [ ] Unit: PSBT construction — 2 UTXOs, 2 outputs (recipient + fee), correct amounts
- [ ] Unit: dust threshold — fee < 546 sats → single output, fee skipped, warning logged
- [ ] Unit: fee rate capping — rate > 500 → capped to 500
- [ ] Unit: sweep mode — all UTXOs consumed, no change output
- [ ] Integration (testnet): `send_btc` → txid returned, confirmed on testnet explorer
- [ ] Integration (testnet): no UTXOs → clear error
- [ ] Integration: dynamic fee rate fetch from mempool.space, fallback to default on failure

**Estimated scope:** Medium — Rust bitcoin ecosystem is mature.

---

### STORY-05: Blockchain Transaction Signing & Broadcasting — Ethereum (ERC-20 + ETH)

**What:** Implement Ethereum ERC-20 and native ETH transfers with signing and broadcasting.

**Why:** Required for ETH/ERC-20 escrow release.

**Acceptance criteria:**
- [ ] Add `alloy` or `ethers-rs` crate dependencies
- [ ] New service: `src/services/blockchain/ethereum.rs`
- [ ] `send_erc20(private_key, to_address, token_address, amount, decimals, fee_address, fee_bps)` → returns tx hash
- [ ] `send_eth(private_key, to_address, amount, fee_address, fee_bps)` → returns tx hash
- [ ] **FIX from Node:** Attempt single-tx pattern if possible (e.g. via multicall/batch), or implement retry-safe logic with balance checks (Node has non-atomic two-tx split — CRIT-2)
- [ ] Gas estimation and price fetching
- [ ] Decimal truncation to avoid precision errors
- [ ] Returns `{ tx_hash, recipient_amount, fee_amount, fee_tx_hash }`

**Node reference:** `services/ethereum.ts` — but FIX the non-atomic fee split (CRIT-2).

**Tests:**
- [ ] Unit: fee split math — 100 USDT at 100 bps → 99 to recipient, 1 to fee
- [ ] Unit: decimal truncation — no precision overflow for 18-decimal tokens
- [ ] Integration (Sepolia): `send_erc20` → tx hash returned, both recipients receive correct amounts
- [ ] Integration (Sepolia): `send_eth` → tx confirms
- [ ] Integration (Sepolia): insufficient ETH for gas → clear error before any tx
- [ ] Integration: retry-safe — verify balance check before retry prevents double-send

**Estimated scope:** Medium — alloy/ethers ecosystem is mature.

---

### STORY-06: Escrow Release Service — Move Crypto to Buyer + Fee to Platform

**What:** When a trade is completed, actually send crypto from the escrow wallet to the buyer, and the 1% fee to the platform wallet.

**Why:** This is the core money movement. Without it, trades "complete" but nothing happens.

**Acceptance criteria:**
- [ ] New service: `src/services/escrow_release.rs`
- [ ] `release_to_buyer(escrow, buyer_address, chain_config, fee_config)`:
  1. Load escrow wallet and decrypt private key (`wallet_crypto.rs` already has decryption)
  2. Determine chain from `escrow.network`
  3. Call appropriate chain send function (STORY-02 through STORY-05) with fee split
  4. Persist real `release_tx_hash` on escrow (not a label like `"custodial_release"`)
  5. Update escrow status to `Released`
  6. Create ledger entry: `escrow_release` / `credit` for buyer
  7. Create ledger entry: `fee` / `credit` for platform (with `user_id` = platform sentinel or config)
  8. Update wallet balances if custodial
- [ ] `complete_trade` handler in `api/trades.rs` calls this service instead of just updating DB status
- [ ] On success: trade status → `Completed`, escrow → `Released`
- [ ] On failure: trade stays in current status, error is logged and returned, no partial state

**Node reference:** `services/escrow/release.ts` `releaseEscrowToBuyer` + `controllers/trades/completeTrade.ts`

**Tests:**
- [ ] Unit: `release_to_buyer` dispatches to correct chain send based on `escrow.network`
- [ ] Unit: ledger entries created with correct amounts (from actual transfer result, not trade amount)
- [ ] Unit: on send failure → escrow status unchanged, error returned
- [ ] Integration: create trade → fund escrow → complete → verify escrow status = `Released`, real `release_tx_hash` stored
- [ ] Integration: verify two ledger entries created (escrow_release + fee)
- [ ] Integration: verify `complete_trade` API returns the tx hash in the response
- [ ] E2E (Puppeteer): create trade flow → complete → verify trade page shows "Completed" with tx link

**Estimated scope:** Medium — orchestration of existing DB code + new chain sends.

---

### STORY-07: Escrow Refund Service — Return Crypto to Seller (No Fee)

**What:** When a trade is cancelled with escrow held, actually send crypto back to the seller. **Do NOT deduct the 1% fee** — this is a fix for Node CRIT-1.

**Why:** Currently, cancelled trades leave money stuck in escrow wallets forever.

**Acceptance criteria:**
- [ ] New service function: `refund_to_seller(escrow, seller_address, chain_config)`
- [ ] Same chain send functions but with `fee_bps: 0` (no platform fee on refunds)
- [ ] Persist real `refund_tx_hash` on escrow
- [ ] Update escrow status to `Refunded`
- [ ] Create ledger entry: `escrow_refund` / `credit` for seller (full amount, no fee)
- [ ] `cancel_trade` handler in `api/trades.rs` calls this service when `escrow.status == Held`
- [ ] On failure: cancel proceeds (DB status updates) but refund is marked as pending for retry

**Node reference:** `services/escrow/release.ts` `refundEscrowToSeller` — but FIX CRIT-1 (no fee on refund).

**Tests:**
- [ ] Unit: `refund_to_seller` calls chain send with `fee_bps: 0` — seller gets 100%, platform gets 0
- [ ] Unit: ledger entry type is `escrow_refund`, direction is `credit`, full amount (no deduction)
- [ ] Integration: create trade → fund escrow → cancel → verify escrow status = `Refunded`, real `refund_tx_hash`
- [ ] Integration: verify seller receives **full** amount (regression test for Node CRIT-1)
- [ ] Integration: cancel without escrow held → no refund attempted, no error
- [ ] E2E (Puppeteer): cancel a funded trade → verify seller sees refund tx on trade page

**Estimated scope:** Small-Medium — reuses chain send functions with fee_bps=0.

---

### STORY-08: Wallet Lock Lifecycle — Release on Complete and Cancel

**What:** When a custodial trade completes or is cancelled, release the seller's wallet lock so their balance is available again.

**Why:** Currently, wallet locks are permanent. Users' available balances only decrease (CRIT-6).

**Acceptance criteria:**
- [ ] `complete_trade` handler releases the wallet lock after successful escrow release
- [ ] `cancel_trade` handler releases the wallet lock and restores `locked_balance`
- [ ] `repo::wallet::release_lock(db, user_id, offer_id, cryptocurrency, amount)` — decrements `locked_balance`, marks `wallet_locks.released_at`
- [ ] Store `wallet_lock_id` on the trade or escrow so it can be looked up on completion
- [ ] Handle edge case: lock was already released (idempotent)

**Node reference:** Node doesn't have DB wallet locks (it uses on-chain escrow wallets). This is Rust-specific.

**Tests:**
- [ ] Unit: `release_lock` decrements `locked_balance` by exact amount, sets `released_at`
- [ ] Unit: `release_lock` on already-released lock → no-op (idempotent), no error
- [ ] Unit: `release_lock` with wrong amount → error
- [ ] Integration: create custodial trade → verify `locked_balance` increased → complete trade → verify `locked_balance` back to 0
- [ ] Integration: create custodial trade → cancel → verify `locked_balance` back to 0
- [ ] Integration: `available_balance` = `balance - locked_balance` is correct throughout lifecycle

**Estimated scope:** Small — DB operations only.

---

### STORY-09: Ledger Entries for All Money Movements

**What:** Create ledger entries for every financial event in the trade lifecycle.

**Why:** `create_entry()` exists but is never called. No audit trail exists (HIGH-7).

**Acceptance criteria:**
- [ ] Trade creation (custodial): `escrow_lock` / `debit` for seller
- [ ] Escrow release: `escrow_release` / `credit` for buyer
- [ ] Platform fee: `fee` / `credit` for platform
- [ ] Escrow refund: `escrow_refund` / `credit` for seller
- [ ] Trade cancellation (custodial): lock release / `credit` for seller
- [ ] Withdrawal: `withdrawal` / `debit` for user
- [ ] Deposit: `deposit` / `credit` for user
- [ ] All entries include: `trade_id`, `escrow_id`, `amount`, `currency`, `balance_after`
- [ ] `balance_after` is computed from the actual on-chain or DB balance at the time

**Node reference:** `services/ledger/createEntry.ts` — entries exist but amounts don't match reality (HIGH-1). Fix by using actual transfer amounts.

**Tests:**
- [ ] Unit: each entry type creates correct `entry_type`, `direction`, `amount`, `currency`
- [ ] Unit: `balance_after` is computed correctly (previous balance ± amount)
- [ ] Integration: complete a trade → verify exactly 2 ledger entries (escrow_release + fee)
- [ ] Integration: cancel a trade → verify exactly 1 ledger entry (escrow_refund)
- [ ] Integration: deposit → verify 1 ledger entry (deposit / credit)
- [ ] Integration: `GET /api/v1/trades/{id}/ledger` returns entries for that trade
- [ ] Integration: ledger `amount` matches the on-chain transfer return value, not the trade's pre-computed amount

**Estimated scope:** Medium — wire into every handler that moves money.

---

### STORY-10: Admin Revenue Dashboard — Aggregate Fees from Ledger

**What:** Admin endpoint that sums platform fee ledger entries to show actual revenue.

**Why:** Both backends show `0` for platform fees (CRIT-4). Investors need to see what the platform earns.

**Acceptance criteria:**
- [ ] `GET /api/v1/admin/treasury/balance` returns actual aggregated fees per currency
- [ ] Query: `SELECT currency, SUM(amount) FROM ledger_entries WHERE entry_type = 'fee' GROUP BY currency`
- [ ] Include: total all-time, last 24h, last 7d, last 30d
- [ ] Include: count of fee-generating trades per period
- [ ] `GET /api/v1/admin/treasury/health` returns platform wallet on-chain balances (read-only, using existing balance check services)
- [ ] `GET /api/v1/admin/treasury/transactions` returns paginated ledger entries of type `fee`

**Node reference:** `controllers/admin/dashboard.ts` — but fix the hardcoded `0` by actually querying the ledger.

**Tests:**
- [ ] Integration: complete 3 trades → `GET /admin/treasury/balance` returns correct sum per currency
- [ ] Integration: verify 24h/7d/30d breakdowns are accurate
- [ ] Integration: `GET /admin/treasury/transactions` returns paginated fee entries
- [ ] Integration: `GET /admin/treasury/health` returns on-chain balances for platform wallets
- [ ] Integration: non-admin user → 403
- [ ] E2E (Puppeteer): admin logs in → navigates to treasury dashboard → sees non-zero fee totals

**Estimated scope:** Small — SQL queries + existing admin auth.

---

## Priority: HIGH (Required for Full Operation)

### STORY-11: Wallet Withdrawal — Send Crypto Out

**What:** Implement the `POST /api/v1/wallet/withdraw` endpoint to actually send crypto to an external address.

**Why:** Currently returns `"withdrawal requires blockchain service"`. Users can deposit but cannot withdraw.

**Acceptance criteria:**
- [ ] Validate: sufficient balance, valid address for the network, amount > minimum
- [ ] Deduct balance atomically (DB transaction)
- [ ] Call appropriate chain send function (no fee split — user pays network fee only)
- [ ] Create ledger entry: `withdrawal` / `debit`
- [ ] Update `wallet_transactions` with real `tx_hash`
- [ ] On blockchain failure: rollback balance deduction
- [ ] Rate limiting / daily withdrawal limits (configurable)

**Tests:**
- [ ] Unit: validate address format per network (reject invalid addresses)
- [ ] Unit: insufficient balance → error before any chain call
- [ ] Unit: on blockchain failure → balance rollback, no deduction
- [ ] Integration: deposit 100 USDT → withdraw 50 → verify balance = 50, real tx hash stored
- [ ] Integration: withdraw more than balance → 400 error, balance unchanged
- [ ] Integration: verify ledger entry created with correct amount and tx hash
- [ ] E2E (Puppeteer): user deposits → goes to wallet → withdraws → sees tx hash and updated balance

**Estimated scope:** Medium — reuses chain send functions.

---

### STORY-12: Custodial Wallet Send — User-Initiated Transfers

**What:** Implement the `send_transaction` handler in `custodial_wallet.rs`.

**Why:** Currently returns `"Send transactions are not yet enabled"`. Users can generate custodial wallets and receive deposits but cannot send.

**Acceptance criteria:**
- [ ] Decrypt user's custodial wallet private key
- [ ] Validate: sufficient on-chain balance, valid destination
- [ ] Call appropriate chain send function (no platform fee — this is a user send, not a trade)
- [ ] Create wallet transaction record with real tx hash
- [ ] Create ledger entry

**Tests:**
- [ ] Unit: private key decryption works (round-trip: encrypt → decrypt → sign → verify)
- [ ] Unit: insufficient on-chain balance → error before send
- [ ] Integration (testnet): send from custodial wallet → verify tx confirms on-chain
- [ ] Integration: verify wallet transaction record has real tx hash
- [ ] Integration: verify no platform fee deducted (this is a user send, not a trade)

**Estimated scope:** Medium — reuses chain send functions + key decryption.

---

### STORY-13: Deposit Balance Crediting

**What:** When a user deposits crypto (detected on-chain), credit their DB wallet balance.

**Why:** The `deposit` handler creates a `wallet_transaction` row but never calls `update_balance()`. The user's DB balance never increases.

**Acceptance criteria:**
- [ ] After confirming deposit on-chain, call `repo::wallet::update_balance` to increment `balance`
- [ ] Idempotent: don't double-credit if called twice for the same tx
- [ ] Create ledger entry: `deposit` / `credit`
- [ ] Consider: polling service or webhook for automatic deposit detection (stretch)

**Tests:**
- [ ] Unit: `update_balance` increments balance by exact deposit amount
- [ ] Unit: double-credit guard — same tx hash twice → second call is no-op
- [ ] Integration: call deposit endpoint → verify `wallets.balance` increased
- [ ] Integration: call deposit endpoint twice with same tx → balance only increases once
- [ ] Integration: verify ledger entry created (deposit / credit)
- [ ] Integration: `GET /wallet/balance` reflects the new balance

**Estimated scope:** Small — wire `update_balance` into deposit handler.

---

### STORY-14: Trade Completion — Custodial Lock-at-Creation Flow (End-to-End)

**What:** Wire the full custodial trade flow: create → lock → buyer pays → seller completes → escrow releases from seller's locked balance to buyer → lock released → fee to platform.

**Why:** The custodial flow is partially built (lock at creation works) but completion does nothing.

**Acceptance criteria:**
- [ ] On `complete_trade`:
  1. Verify trade is in `Paid` or `Released` status
  2. Load the escrow and the seller's custodial wallet
  3. Call `escrow_release::release_to_buyer` (STORY-06) to send crypto from seller's wallet to buyer
  4. Deduct from seller's DB balance (`balance -= amount`)
  5. Release the wallet lock (STORY-08)
  6. Credit buyer's DB balance if they have a platform wallet
  7. Create all ledger entries (STORY-09)
  8. Update trade status to `Completed`
- [ ] Atomic: if blockchain send fails, don't update DB. If DB update fails after send, log for manual reconciliation.

**Tests:**
- [ ] Integration: full custodial E2E — create offer → buyer buys → seller locked_balance increases → buyer marks paid → seller completes → crypto sent on-chain → locked_balance released → fee in ledger → trade status = Completed
- [ ] Integration: complete fails if trade not in Paid status
- [ ] Integration: blockchain send failure → trade stays in Paid, no DB state change
- [ ] Integration: verify seller's `balance` decreased by trade amount
- [ ] Integration: verify buyer's balance increased (if custodial wallet exists)
- [ ] Integration: verify platform fee ledger entry amount = 1% of trade amount
- [ ] E2E (Puppeteer): full buyer journey — find offer → buy → mark paid → seller completes → both see "Completed"

**Estimated scope:** Medium — orchestration of STORY-06, 08, 09.

---

### STORY-15: Trade Completion — On-Chain Escrow Flow (End-to-End)

**What:** Wire the full on-chain escrow flow: create → seller deposits to escrow address → balance detected → buyer pays → seller completes → escrow releases from escrow wallet to buyer → fee to platform.

**Why:** On-chain escrow is the other main flow. Currently, confirm-deposit works but release does nothing.

**Acceptance criteria:**
- [ ] On `complete_trade`:
  1. Verify trade is in `Paid` status and escrow is `Held`
  2. Load the escrow wallet (address + encrypted private key)
  3. Decrypt private key
  4. Call `escrow_release::release_to_buyer` (STORY-06) to send from escrow wallet to buyer
  5. Create all ledger entries (STORY-09)
  6. Update trade and escrow status
- [ ] Pre-flight: verify on-chain balance >= expected amount before attempting release
- [ ] Handle partial deposits (balance < expected): reject release, notify seller

**Tests:**
- [ ] Integration: full on-chain E2E — create trade → escrow wallet created → seller deposits (testnet) → balance detected → buyer marks paid → seller completes → crypto released on-chain → fee in ledger → trade = Completed
- [ ] Integration: complete fails if escrow is not Held
- [ ] Integration: partial deposit (balance < expected) → release rejected with clear error
- [ ] Integration: blockchain send failure → escrow stays Held, no partial state
- [ ] Integration: verify release_tx_hash is a real tx hash, not a placeholder string
- [ ] E2E (Puppeteer): seller deposits → confirms deposit → buyer pays → seller completes → trade page shows tx link

**Estimated scope:** Medium — orchestration of STORY-06, 09 + key decryption.

---

## Priority: MEDIUM (Needed for Feature Completeness)

### STORY-16: Gas Sponsorship — Tron TRX for USDT Transfers

**What:** When releasing USDT on Tron, the escrow wallet needs TRX for gas. Implement treasury-funded gas sponsorship.

**Why:** Without TRX in the escrow wallet, TRC-20 transfers will fail. The Node backend has this (partially broken — MED-3).

**Acceptance criteria:**
- [ ] Treasury wallet config (address + encrypted key for each chain)
- [ ] Before TRC-20 escrow release: check TRX balance of escrow wallet
- [ ] If insufficient: send TRX from treasury to escrow wallet
- [ ] After release: optionally reimburse treasury from the released amount
- [ ] Track sponsorship in `treasury_transactions` table

**Tests:**
- [ ] Unit: TRX balance check → below threshold → sponsorship triggered
- [ ] Unit: TRX balance check → above threshold → no sponsorship
- [ ] Integration (Nile): sponsor TRX to escrow wallet → escrow wallet TRX balance increases
- [ ] Integration: verify `treasury_transactions` record created with correct amount and tx hash
- [ ] Integration: full flow — escrow has USDT but no TRX → sponsor → release succeeds

**Estimated scope:** Medium — treasury wallet management + TRX sends.

---

### STORY-17: Gas Sponsorship — Ethereum ETH for ERC-20 Transfers

**What:** Same as STORY-16 but for Ethereum — escrow wallets need ETH for gas.

**Acceptance criteria:**
- [ ] Before ERC-20 escrow release: check ETH balance of escrow wallet
- [ ] If insufficient: send ETH from treasury to escrow wallet
- [ ] Minimum gas: 0.002 ETH
- [ ] Track in `treasury_transactions`

**Tests:**
- [ ] Unit: ETH balance check → below 0.002 → sponsorship triggered
- [ ] Integration (Sepolia): sponsor ETH → escrow wallet balance increases
- [ ] Integration: full flow — escrow has ERC-20 but no ETH → sponsor → release succeeds

**Estimated scope:** Medium.

---

### STORY-18: Affiliate Commission Recording on Trade Completion

**What:** When a trade completes, calculate and record affiliate commissions as a share of the escrow fee.

**Why:** The tier system and `commission_from_escrow_fee()` exist in `types/enums.rs` but are never called. Affiliates should see their earnings.

**Acceptance criteria:**
- [ ] On trade completion, after fee is collected:
  1. Look up seller's and buyer's referral chains (L1, L2, L3)
  2. Calculate commission per level using `AffiliateTier::commission_from_escrow_fee`
  3. Create `affiliate_earnings` records with `status: 'pending'`
  4. Increment referrer's `pending_earnings`
- [ ] Commission is a share OF the 1% fee, not additional
- [ ] Create ledger entries for each commission

**Node reference:** `services/affiliateService.ts` — records but never pays. We record now, payout is STORY-19.

**Tests:**
- [ ] Unit: `commission_from_escrow_fee` with each tier returns correct bps (existing tests already cover this — verify they still pass)
- [ ] Unit: L1/L2/L3 chain lookup returns correct referrers
- [ ] Integration: complete trade where buyer has a referrer → `affiliate_earnings` record created with correct amount
- [ ] Integration: commission amount is a share of the 1% fee, not additional
- [ ] Integration: no referrer → no commission records, no error

**Estimated scope:** Small-Medium.

---

### STORY-19: Affiliate Commission Payout

**What:** Admin-triggered or automated payout of pending affiliate commissions.

**Why:** Node backend records commissions but has NO payout mechanism (HIGH-4). We need to close the loop.

**Acceptance criteria:**
- [ ] `POST /api/v1/admin/affiliates/payout` — admin triggers payout for a specific affiliate or batch
- [ ] Sends crypto from platform/treasury wallet to affiliate's wallet
- [ ] Updates `affiliate_earnings.status` from `pending` to `paid`
- [ ] Creates ledger entries
- [ ] Stretch: automated payout when pending balance exceeds threshold

**Tests:**
- [ ] Integration: create affiliate earnings → admin triggers payout → `status` changes to `paid`, real tx hash stored
- [ ] Integration: payout to affiliate with no wallet → clear error
- [ ] Integration: verify ledger entry created for payout
- [ ] Integration: non-admin → 403
- [ ] Integration: double payout attempt → idempotent, no double-send

**Estimated scope:** Medium — reuses chain send functions + admin auth.

---

### STORY-20: Non-Escrow Trade Fee Collection

**What:** For trades where `escrow_required = false`, still collect the 1% platform fee.

**Why:** Node backend collects zero fee on non-escrow trades (HIGH-5). Revenue leak.

**Acceptance criteria:**
- [ ] On non-escrow trade completion, create a fee invoice or deduct from seller's platform wallet balance
- [ ] Create ledger entry for the fee
- [ ] If seller has no platform wallet: record as receivable

**Tests:**
- [ ] Integration: complete non-escrow trade → verify fee ledger entry created
- [ ] Integration: seller has platform wallet balance → fee deducted from balance
- [ ] Integration: seller has no balance → receivable record created (or trade blocked — TBD)

**Estimated scope:** Small — depends on business decision (deduct from balance vs invoice).

---

## Story Dependency Graph

```
STORY-01 (fee config)
    ├── STORY-02 (Solana send)
    ├── STORY-03 (Tron send)
    ├── STORY-04 (Bitcoin send)
    └── STORY-05 (Ethereum send)
            │
            ├── STORY-06 (escrow release service)
            │       ├── STORY-14 (custodial trade E2E)
            │       └── STORY-15 (on-chain trade E2E)
            │
            ├── STORY-07 (escrow refund service)
            │
            ├── STORY-11 (wallet withdrawal)
            │
            └── STORY-12 (custodial wallet send)

STORY-08 (wallet lock lifecycle) ← standalone, wire into STORY-14

STORY-09 (ledger entries) ← wire into STORY-06, 07, 11, 12, 13

STORY-10 (admin revenue dashboard) ← depends on STORY-09

STORY-13 (deposit balance crediting) ← standalone

STORY-16 (Tron gas sponsorship) ← depends on STORY-03
STORY-17 (ETH gas sponsorship) ← depends on STORY-05

STORY-18 (affiliate recording) ← depends on STORY-06
STORY-19 (affiliate payout) ← depends on STORY-18 + chain sends

STORY-20 (non-escrow fee) ← standalone
```

## Recommended Build Order

| Phase | Stories | What it unlocks |
|-------|---------|-----------------|
| **Phase 1: Foundation** | STORY-01, STORY-08, STORY-09, STORY-13 | Fee config, lock lifecycle, ledger, deposit crediting |
| **Phase 2: Solana (primary chain)** | STORY-02, STORY-06, STORY-07 | Solana escrow release + refund with fee split |
| **Phase 3: End-to-end trades** | STORY-14, STORY-15 | Custodial + on-chain trades fully working on Solana |
| **Phase 4: Revenue visibility** | STORY-10 | Admin can see platform fees earned |
| **Phase 5: Remaining chains** | STORY-03, STORY-04, STORY-05 | Tron, BTC, ETH escrow release |
| **Phase 6: Gas sponsorship** | STORY-16, STORY-17 | Tron + ETH trades work without users needing gas |
| **Phase 7: User wallet ops** | STORY-11, STORY-12 | Users can withdraw and send from custodial wallets |
| **Phase 8: Affiliate + extras** | STORY-18, STORY-19, STORY-20 | Affiliate commissions, non-escrow fees |

## Node Backend Bugs We Are Fixing (Not Copying)

| Node Bug | How Rust Fixes It |
|----------|-------------------|
| CRIT-1: Refund takes 1% fee | STORY-07: Refund uses `fee_bps: 0` — seller gets 100% back |
| CRIT-2: Non-atomic Tron/ETH fee split | STORY-03/05: Implement retry-safe logic, check balance before retry, idempotency keys |
| CRIT-3: Cancel doesn't refund on-chain | STORY-07: `cancel_trade` calls `refund_to_seller` |
| CRIT-4: Dashboard shows $0 | STORY-10: Aggregate from ledger entries |
| HIGH-1: Ledger ≠ on-chain amounts | STORY-09: Use actual transfer return values for ledger amounts |
| HIGH-5: Non-escrow = free | STORY-20: Collect fee on all trades |
| HIGH-6: `platform_fee_bps` = 0 | STORY-01: Read from config/env |
| HIGH-7: Ledger never called | STORY-09: Wire into every money movement |
| MED-5: Fee addresses hardcoded | STORY-01: Env vars, rotatable without deploy |
