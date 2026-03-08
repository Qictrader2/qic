# Node Backend (`backend/`) — Issue List

> Extracted from `docs/BACKEND-PROFIT-GAP-ANALYSIS.md`

---

## Critical

| ID | Issue |
|----|-------|
| CRIT-1 | Refunds deduct 1% platform fee — seller loses money on cancellation |
| CRIT-2 | Non-atomic fee split on Tron/ETH — buyer can permanently lose ~1% on transfer failure |
| CRIT-3 | Cancel trade doesn't release escrow on-chain — money stuck in escrow wallets |
| CRIT-4 | Admin dashboard fees hardcoded to 0 — zero revenue visibility for investors |
| CRIT-7 | Reseller profit never distributed — feature is non-functional |

## High

| ID | Issue |
|----|-------|
| HIGH-1 | Ledger entries don't match on-chain amounts — accounting discrepancies |
| HIGH-2 | Reseller trades non-atomic, no pricing snapshot — data corruption risk |
| HIGH-3 | Reseller trade status never completed — stats always show 0 |
| HIGH-4 | Affiliate commissions never paid out — broken promises to referrers |
| HIGH-5 | Non-escrow trades collect no fee — revenue leak |

## Medium

| ID | Issue |
|----|-------|
| MED-1 | BTC dust threshold silently skips platform fee — small trade revenue loss |
| MED-2 | Dashboard queries wrong Firestore collection — escrow metrics always 0 |
| MED-3 | Treasury sponsor function doesn't actually sponsor gas — incorrect usage tracking |
| MED-4 | EVM gas reimbursement deducts from buyer but never sends to treasury — buyer loses money to nowhere |
| MED-5 | Platform fee addresses hardcoded in source — no emergency rotation |
| MED-6 | Reseller network hardcoded to Solana — wrong chain for non-SOL crypto |
| MED-7 | Firestore full-collection scans in reseller — won't scale |

## Low

| ID | Issue |
|----|-------|
| LOW-1 | Excessive console.log of financial data — security concern |
| LOW-2 | Solana SOL release no dust check for fee output — potential tx failure |
| LOW-3 | Stored platformFeeAmount is informational only — fee can change between creation and release |
