# QicTrader Backend — Profit & Money Flow Gap Analysis

> **Date:** 2026-03-06
> **Scope:** Both backends — `backend/` (Node/TS, original) and `qictrader-backend-rs/` (Rust, rewrite)
> **Purpose:** Critical audit of where money sits, how profit is collected, and why investors are not receiving returns

---

## Table of Contents

1. [Where Funds Are Held](#1-where-funds-are-held)
2. [What "Profit" Is](#2-what-profit-is)
3. [Side-by-Side Backend Comparison](#3-side-by-side-backend-comparison)
4. [Node Backend (`backend/`) — Deep Audit](#4-node-backend-deep-audit)
   - [Trade Lifecycle](#41-trade-lifecycle)
   - [Critical Issues](#42-critical-issues)
   - [High Severity Issues](#43-high-severity-issues)
   - [Medium Severity Issues](#44-medium-severity-issues)
   - [Low Severity Issues](#45-low-severity-issues)
5. [Reseller / Flip-Deals — Completely Broken](#5-reseller--flip-deals--completely-broken)
6. [Rust Backend (`qictrader-backend-rs/`) — Summary](#6-rust-backend--summary)
7. [Why Investors Are Not Receiving Profit](#7-why-investors-are-not-receiving-profit)
8. [Full Issue Tracker](#8-full-issue-tracker)

---

## 1. Where Funds Are Held

When money enters the system, it can sit in three places:

| Location | Description | Backend |
|----------|-------------|---------|
| **User wallets (DB)** | `wallets` table — per user, per cryptocurrency: `balance`, `locked_balance`, `pending_balance`. Used for custodial lock-at-creation flow. | Rust |
| **Custodial wallets (on-chain)** | `custodial_wallets` table — per user, per network. Real blockchain addresses with encrypted private keys. Balances read on-chain. | Both |
| **Escrow wallets (on-chain, per trade)** | `escrow_wallets` / `escrows` — a temporary wallet created per trade. Seller deposits crypto here; released to buyer on completion. | Both |

**Platform fee wallets** (where profit should go) — hardcoded in the Node backend (`config/platformFees.ts`):

| Chain | Address |
|-------|---------|
| Bitcoin | `bc1qs5622yulyj6atsuvzttu3d396xgtt2lndp7775` |
| Ethereum | `0x4747B5b654a4CF3dF120e9a2204ec26fa695B36A` |
| Solana | `9QxPH5oAUDQWBWTZTrdLHCc42g62D8KGNVacPn6t9p2Q` |
| Tron | `TEKxYnZeNLWvxib32wTSWvTkTTE6U5DXg4` |

**In the Rust backend, these addresses do not exist.** There is no fee collection.

---

## 2. What "Profit" Is

The platform earns from:

| Revenue Stream | Rate | Status |
|----------------|------|--------|
| **Escrow fee** | 1% of trade value on release | Node: partially working (see issues). Rust: not implemented. |
| **Gas markup** | ~1% on sponsored gas fees | Node: treasury service. Rust: stub only. |
| **Reseller cut** | None currently — platform only takes the standard 1% | Reseller logic is broken (see §5). |
| **Affiliate commissions** | Share of the 1% escrow fee (5–25% depending on tier) | Recorded but never paid out. |

---

## 3. Side-by-Side Backend Comparison

| Concern | Node Backend (`backend/`) | Rust Backend (`qictrader-backend-rs/`) |
|---------|---------------------------|----------------------------------------|
| **Trade creation** | Creates trade + escrow atomically in Firestore | Creates trade + escrow, locks wallet balance (custodial) |
| **Escrow release** | Real on-chain transfer (BTC/ETH/SOL/TRX) | DB status change only — **no crypto transfer** |
| **Platform fee %** | 1% deducted at release, sent to platform wallets | Always **0** — `platform_fee_bps: 0` in quote builder |
| **Fee destination** | 4 hardcoded wallet addresses per chain | Nowhere — fee is never collected |
| **Fee tracking** | Ledger entries created (`platform_fee`) but not aggregated | `create_entry()` exists but is **never called** |
| **Lock release on complete** | Yes (escrow wallet emptied on-chain) | **No** — wallet lock stays forever |
| **Lock release on cancel** | Ledger entry created, but no refund transfer | **No** — wallet lock stays forever |
| **Wallet balance updates** | Managed via escrow wallets (on-chain) | `update_balance()` never called in trade flow |
| **Reseller profit** | Calculated and stored but never distributed | Not implemented |
| **Admin fee dashboard** | Hardcoded to `0`, comment says "Would need fee tracking" | Treasury endpoints return "requires treasury service" |

---

## 4. Node Backend Deep Audit

### 4.1 Trade Lifecycle

**Step 1 — Trade creation** (`controllers/trades/createTrade.ts`, `services/trade/createTradeAtomically.ts`)
- Validates offer, buyer, trade limits
- Computes pricing snapshot with `platformFeeAmount` (1% of crypto amount)
- Creates trade document with status `pending`
- Creates `escrow_lock` ledger entry (debit seller)
- If custodial: creates escrow wallet, `escrowStatus: 'pending'`
- If BTC wallet-lock: `escrowStatus: 'held'`

**Step 2 — Escrow funding** (custodial only)
- Seller deposits crypto to escrow wallet address
- Frontend polls `checkEscrowBalance` (on-chain balance)
- `confirmEscrowDeposit` updates `escrowStatus: 'held'`, trade status to `active`

**Step 3 — Buyer marks paid** (`controllers/trades/updateTradeStatus.ts`)
- `PATCH /trades/:id/status` with `status: 'paid'`
- Verifies escrow is funded before allowing

**Step 4 — Trade completion / release** (`controllers/trades/completeTrade.ts`)
- Seller calls `POST /trades/:id/complete`
- Three paths:
  - **BTC wallet-lock:** `releaseBTCFromWalletLock` — single atomic transaction, 99% to buyer + 1% to platform address
  - **Custodial escrow:** `releaseEscrowToBuyer` — sends 99% to buyer + 1% to platform address (two separate transactions on Tron/ETH — **non-atomic, dangerous**)
  - **Non-escrow:** Status change only — **no fee collected**

**Step 5 — Cancellation** (`controllers/trades/cancelTrade.ts`)
- Sets DB status to `cancelled` and `escrowStatus: 'refunded'`
- Creates `escrow_refund` ledger entry
- **But never actually calls `refundEscrowToSeller`** — crypto stays locked in escrow wallet

---

### 4.2 Critical Issues

#### CRIT-1: Refunds Deduct 1% Platform Fee — Seller Loses Money on Cancellation

**File:** `services/escrow/release.ts`

`refundEscrowToSeller` calls `executeTransfer`, which uses the same blockchain send functions as buyer release. Every send function deducts 1% to the platform fee address. On a refund, the seller gets back only 99% of their own escrowed funds.

**Impact:** Seller loses 1% of their crypto when a trade is cancelled — even though neither party completed a trade.

---

#### CRIT-2: Non-Atomic Fee Splitting on Tron and Ethereum — Money Can Be Lost

**Files:** `services/tron.ts`, `services/ethereum.ts`

On Tron and Ethereum, the platform fee is sent as a **separate transaction before** the recipient transfer. If the fee tx succeeds but the recipient tx fails:

- Platform has already taken 1%
- Buyer receives nothing
- Escrow wallet now has only 99% of the original balance
- On retry, amounts are recalculated on the reduced balance — buyer permanently loses ~1%

**Bitcoin and Solana are safe** — both use single atomic transactions with multiple outputs/instructions.

---

#### CRIT-3: Cancel Trade Does NOT Actually Refund Escrow

**File:** `controllers/trades/cancelTrade.ts`

When a trade is cancelled:
1. DB says `escrowStatus: 'refunded'`
2. Ledger entry says funds were refunded
3. **But `refundEscrowToSeller()` is never called**

Crypto remains locked in the escrow wallet. The seller must manually call `POST /escrow/:id/refund` separately — and even then loses 1% (see CRIT-1).

**Impact:** Money sits in abandoned escrow wallets after cancellation. Sellers may not know they need to manually request a refund.

---

#### CRIT-4: Admin Dashboard Shows Zero Platform Fees — No Revenue Visibility

**File:** `controllers/admin/dashboard.ts`

```typescript
platformFees: 0, // Would need fee tracking
platformFeesToday: 0,
```

Platform fees are hardcoded to `0`. The admin dashboard never queries ledger entries or any source for actual fee revenue. The platform owner / investors have **zero visibility** into how much money the platform is making.

---

### 4.3 High Severity Issues

#### HIGH-1: Ledger Entries Don't Match Actual On-Chain Amounts

**File:** `controllers/trades/completeTrade.ts`

Ledger entries use `trade.cryptoAmount * PLATFORM_FEE_PERCENTAGE`, but actual on-chain transfer uses `escrowWallet.depositedAmount` (which can be reduced by gas reimbursement). Ledger and blockchain diverge.

If BTC platform fee is skipped due to dust threshold, the ledger still records a fee deduction.

---

#### HIGH-2: Reseller Trades Missing Pricing Snapshot, Not Atomic

**File:** `controllers/reseller/buyResellOffer.ts`

- No `pricingSnapshot` on the trade — `completeTrade` may behave unexpectedly
- No `escrowRequired` field set
- Two separate Firestore writes (trade + reseller_trade) without a transaction — data inconsistency
- Network hardcoded to `'solanaMainnet'` regardless of actual cryptocurrency

---

#### HIGH-3: Reseller Trade Status Never Updated on Completion

`completeTrade` updates the trade document but has **no code** to update the corresponding `reseller_trades` document. `reseller_trade.status` stays `'pending'` forever.

`reseller/stats.ts` will always report **0 completed trades and 0 profit**.

---

#### HIGH-4: Affiliate Commissions Are Never Paid Out

**File:** `services/affiliateService.ts`

Commissions are recorded in `affiliate_earnings` with `status: 'pending'`. There is **no payout mechanism** anywhere — no scheduled job, no admin action, no API endpoint to transfer crypto to the referrer.

---

#### HIGH-5: Non-Escrow Trade Completion Collects No Fee

**File:** `controllers/trades/completeTrade.ts`

For trades where `escrowRequired === false`, no money moves, no platform fee is collected, and no financial ledger entries are created. This is a revenue leak.

---

### 4.4 Medium Severity Issues

#### MED-1: BTC Dust Threshold Silently Skips Platform Fee

**File:** `services/bitcoin.ts`

For small BTC trades where 1% < 546 satoshis (trades under ~0.000546 BTC / ~$30), the platform fee is set to 0. No alert or logging for accounting purposes.

---

#### MED-2: Dashboard Queries Wrong Firestore Collection for Escrow Data

**File:** `controllers/admin/dashboard.ts`

Code queries `db.collection('escrows')` but escrow wallets are in `db.collection('escrow_wallets')`. `totalEscrowHeld` and `activeEscrows` will always be 0.

---

#### MED-3: Treasury Sponsor Function Doesn't Actually Sponsor Gas

**File:** `services/treasury.ts`

`sponsorUserTransaction` creates a TronWeb instance with the **user's** private key. The user pays gas from their own TRX. The treasury tracks usage it didn't actually fund.

---

#### MED-4: EVM Gas Reimbursement Deducts From Buyer But Never Sends to Treasury

**File:** `services/escrow/release.ts`

For EVM chains, gas "reimbursement" deducts from the buyer's payout but no USDT/token is sent to any treasury address. Comment: "we don't have an auto-swap mechanism yet" — money disappears from the buyer with no corresponding credit.

---

#### MED-5: Platform Fee Addresses Hardcoded in Source Code

**File:** `config/platformFees.ts`

All four fee addresses are hardcoded. Changing them requires a code change and deploy. If compromised, no emergency rotation possible without a deploy.

---

### 4.5 Low Severity Issues

| ID | Issue | File |
|----|-------|------|
| LOW-1 | Excessive sensitive data in console logs | Multiple blockchain services |
| LOW-2 | Solana SOL escrow release doesn't handle dust for fee output | `services/solana/transactions.ts` |
| LOW-3 | Stored `platformFeeAmount` in pricing snapshot is informational only — never read on release | `services/trade/createTradeAtomically.ts` |

---

## 5. Reseller / Flip-Deals — Completely Broken

The reseller feature is **UI scaffolding only**. The financial settlement is entirely unimplemented.

### How It's Supposed to Work

1. Seller creates offer at market rate + 1% (e.g. R19.50/USDT)
2. Reseller creates a resell offer at market rate + 3% (e.g. R19.89/USDT)
3. Buyer purchases from resell offer
4. Seller's crypto is held in escrow
5. Buyer pays reseller the higher price (fiat, off-platform)
6. Reseller pays seller the original price (fiat, off-platform)
7. Escrow releases crypto to buyer
8. **Reseller's profit = difference in fiat price × crypto amount**
9. Platform takes 1% escrow fee

### How It Actually Works (Broken)

1. Reseller creates a resell offer → works
2. Buyer buys from resell offer → **creates a normal trade with `sellerId = resellerId`**
3. **The reseller must fund escrow from their own wallet** — there is no link to the original seller's offer
4. On completion, crypto goes from reseller's escrow to buyer
5. **The original seller is never involved, never paid, never referenced**
6. `profit` field is calculated but **never paid out**
7. `reseller_trades.status` stays `'pending'` **forever** — `completeTrade` has zero knowledge of the reseller layer

### What's Missing

| What Should Happen | What Actually Happens |
|---|---|
| Original seller funds escrow | Reseller funds escrow from own pocket |
| Reseller collects fiat markup from buyer | No mechanism — just a number in the DB |
| Reseller pays original seller the base price | Never happens |
| `reseller_trades.status` updates on completion | Status is `'pending'` forever |
| `resell_offers.totalSales` increments | Always `0` |
| `resell_offers.totalProfit` increments | Always `0` |
| `reseller_stats` shows actual numbers | `completedResells`, `totalSales`, `totalProfit`, `totalVolume` are all `0` |

### Additional Reseller Bugs

- **Network hardcoded to `solanaMainnet`** (`buyResellOffer.ts` line 136) regardless of the actual cryptocurrency
- **Firestore full-collection scans** in `trades.ts`, `stats.ts`, `active.ts` — fetches ALL documents then filters in-memory. Will not scale.
- **Two Firestore writes without a transaction** in `buyResellOffer.ts` — trade and reseller_trade can be inconsistent

---

## 6. Rust Backend — Summary

The Rust backend (`qictrader-backend-rs/`) has **no money movement at all**:

| Aspect | Status |
|--------|--------|
| Fee collection | **Not implemented** — `platform_fee_bps: 0`, no fee distribution |
| Fee storage | `fee_amount` on trade/escrow is always `0` |
| Escrow release | DB status change only — **no crypto transfer** |
| Custodial completion | No wallet transfer, no lock release on completion |
| Ledger entries | `repo::ledger::create_entry` exists but is **never called** |
| Wallet balances | `update_balance()` exists but is **never called** during trades |
| Wallet lock on cancel | **Not released** — lock remains forever |
| Wallet lock on complete | **Not released** — lock remains forever |
| Withdrawal | Returns `"withdrawal requires blockchain service"` |
| Treasury | Returns `"requires treasury service"` |

### Stubs / TODOs in Rust

| Location | Issue |
|----------|-------|
| `api/trades.rs` | `get_trade_ledger` — "TODO: integrate with repo::ledger" |
| `api/escrow.rs` | `get_escrow_wallet_for_trade` — "escrow wallet service not yet implemented" |
| `api/escrow.rs` | `sync_escrow` — "blockchain sync not yet available" |
| `api/wallet.rs` | `withdraw` — "withdrawal requires blockchain service" |
| `api/wallet.rs` | `unlock_funds` — "unlock requires lock_id lookup - not yet implemented" |
| `api/gas.rs` | Multiple treasury endpoints — "requires treasury service" |

---

## 7. Why Investors Are Not Receiving Profit

Here is every reason, from most impactful to least:

### A. The 1% Fee Revenue Is Invisible

Even in the Node backend where fees are actually collected on-chain, the admin dashboard shows `0`. There is no aggregation of ledger entries, no revenue report, no way for investors to see what the platform has earned. The fee addresses receive crypto, but nobody is monitoring them programmatically.

### B. The Rust Backend Collects Zero Fees

If any trades are running through the Rust backend, the platform earns exactly nothing. `platform_fee_bps` is set to `0`, escrow release only changes a DB flag, and no crypto is transferred to any platform address.

### C. Cancelled Trades Lock Up Money

In the Node backend, cancellation marks the DB as "refunded" but never actually refunds the seller. Crypto sits in orphaned escrow wallets that nobody can access without manual intervention. This is money that should be liquid but is effectively lost.

### D. Reseller Feature Generates Zero Revenue

The entire reseller flow is broken. Reseller profit is a write-only number — never paid out, never aggregated, never visible. The original seller in a resell chain is never involved. Stats always show 0.

### E. Affiliate Commissions Are Promises, Not Payments

Affiliate earnings are recorded with `status: 'pending'` but there is no payout mechanism. No scheduled job, no admin action, no API endpoint to transfer commissions to referrers. This is a liability on the books that cannot be settled.

### F. Non-Escrow Trades Are Free

Any trade marked `escrowRequired: false` completes without collecting any platform fee. If a significant portion of trades are non-escrow, that's direct revenue leakage.

### G. Refunds Cost Sellers 1%

When the platform does process a refund, the same send functions split 99/1, so the platform takes 1% even on refunds. This is money the platform shouldn't be collecting — it creates trust issues and may have legal implications.

### H. Ledger ≠ Reality

The ledger entries record amounts based on `trade.cryptoAmount`, but actual on-chain transfers use `escrowWallet.depositedAmount` (which can differ due to gas reimbursement). The books don't match the blockchain.

---

## 8. Full Issue Tracker

| # | Severity | Backend | Issue | Impact |
|---|----------|---------|-------|--------|
| CRIT-1 | **CRITICAL** | Node | Refunds deduct 1% platform fee | Seller loses money on cancellation |
| CRIT-2 | **CRITICAL** | Node | Non-atomic fee split on Tron/ETH | Buyer can permanently lose ~1% on transfer failure |
| CRIT-3 | **CRITICAL** | Node | Cancel trade doesn't release escrow on-chain | Money stuck in escrow wallets |
| CRIT-4 | **CRITICAL** | Node | Admin dashboard fees hardcoded to 0 | Zero revenue visibility for investors |
| CRIT-5 | **CRITICAL** | Rust | No money movement at all | Zero fee collection, zero escrow release, zero everything |
| CRIT-6 | **CRITICAL** | Rust | Wallet locks never released | Users' available balances only decrease |
| CRIT-7 | **CRITICAL** | Both | Reseller profit never distributed | Feature is non-functional |
| HIGH-1 | HIGH | Node | Ledger entries don't match on-chain amounts | Accounting discrepancies |
| HIGH-2 | HIGH | Node | Reseller trades non-atomic, no pricing snapshot | Data corruption risk |
| HIGH-3 | HIGH | Node | Reseller trade status never completed | Stats always show 0 |
| HIGH-4 | HIGH | Node | Affiliate commissions never paid out | Broken promises to referrers |
| HIGH-5 | HIGH | Node | Non-escrow trades collect no fee | Revenue leak |
| HIGH-6 | HIGH | Rust | `platform_fee_bps` hardcoded to 0 | No fees even if release was implemented |
| HIGH-7 | HIGH | Rust | Ledger `create_entry` never called | No audit trail |
| MED-1 | MEDIUM | Node | BTC dust threshold silently skips fee | Small trade revenue loss |
| MED-2 | MEDIUM | Node | Dashboard queries wrong Firestore collection | Escrow metrics always 0 |
| MED-3 | MEDIUM | Node | Treasury sponsor function doesn't actually sponsor | Incorrect usage tracking |
| MED-4 | MEDIUM | Node | EVM gas reimbursement deducts but doesn't transfer | Buyer loses money to nowhere |
| MED-5 | MEDIUM | Node | Fee addresses hardcoded in source | No emergency rotation |
| MED-6 | MEDIUM | Node | Reseller network hardcoded to Solana | Wrong chain for non-SOL crypto |
| MED-7 | MEDIUM | Node | Firestore full-collection scans in reseller | Won't scale |
| LOW-1 | LOW | Node | Excessive console.log of financial data | Security concern |
| LOW-2 | LOW | Node | SOL release no dust check for fee output | Potential tx failure |
| LOW-3 | LOW | Node | Stored platformFeeAmount is informational only | Fee can change between creation and release |

---

## Appendix: Files Audited

### Node Backend (`backend/src/`)

| File | Role |
|------|------|
| `config/platformFees.ts` | Fee percentage and platform wallet addresses |
| `controllers/trades/createTrade.ts` | Trade creation entry point |
| `controllers/trades/completeTrade.ts` | Trade completion and escrow release |
| `controllers/trades/updateTradeStatus.ts` | Status transitions (paid, etc.) |
| `controllers/trades/cancelTrade.ts` | Trade cancellation |
| `controllers/escrow/escrowByTrade.ts` | Custodial escrow by trade, refund |
| `controllers/reseller/resell.ts` | Create resell offer |
| `controllers/reseller/buyResellOffer.ts` | Buyer purchases from resell offer |
| `controllers/reseller/trades.ts` | Reseller trade history |
| `controllers/reseller/stats.ts` | Reseller dashboard stats |
| `controllers/reseller/active.ts` | Active resell offers |
| `controllers/admin/dashboard.ts` | Admin dashboard (fees hardcoded to 0) |
| `services/trade/createTradeAtomically.ts` | Atomic trade creation in Firestore |
| `services/escrow/release.ts` | Escrow release and refund transfers |
| `services/escrow/balance.ts` | Balance check and deposit confirmation |
| `services/bitcoin.ts` | BTC send with fee split |
| `services/tron.ts` | Tron send with fee split (non-atomic) |
| `services/solana/splToken.ts` | Solana SPL send with fee split |
| `services/ethereum.ts` | ETH send with fee split (non-atomic) |
| `services/ledger/createEntry.ts` | Ledger entry creation |
| `services/treasury.ts` | Gas sponsorship |
| `services/affiliateService.ts` | Affiliate commission recording |
| `types/reseller.ts` | Reseller type definitions |
| `types/trades.ts` | Trade type definitions |
| `types/ledger.ts` | Ledger type definitions |

### Rust Backend (`qictrader-backend-rs/src/`)

| File | Role |
|------|------|
| `api/trades.rs` | Trade handlers (create, complete, cancel, dispute) |
| `api/escrow.rs` | Escrow handlers (create, confirm, release, refund) |
| `api/wallet.rs` | Wallet handlers (balance, deposit, withdraw, lock) |
| `api/custodial_wallet.rs` | Custodial wallet handlers |
| `api/admin.rs` | Admin handlers (treasury stub) |
| `api/gas.rs` | Gas fee handlers (treasury stubs) |
| `repo/wallet.rs` | Wallet DB queries |
| `repo/escrow.rs` | Escrow DB queries |
| `repo/ledger.rs` | Ledger DB queries (never called from trade flow) |
| `services/escrow_balance.rs` | On-chain balance fetching |
| `services/quote.rs` | Quote calculation (fee_bps: 0) |
| `types/enums.rs` | State machines, affiliate tiers, escrow fee constants |
| `models/wallet.rs` | Wallet DB models |
| `models/escrow.rs` | Escrow DB models |
| `migrations/006_create_escrow.up.sql` | Escrow schema |
| `migrations/007_create_wallets.up.sql` | Wallet schema |
| `migrations/013_create_ledger.up.sql` | Ledger schema |
| `migrations/014_create_platform.up.sql` | Platform config (platform_fee_bps: 100) |
