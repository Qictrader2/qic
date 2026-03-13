# MVP-003: Sell offers should auto-fund escrow from custodial wallet

**Severity:** HIGH — broken core flow  
**Status:** Not implemented  
**Date:** 2026-03-08

## Problem

When a user creates a **sell** offer, they are redirected to a "Fund Escrow" page and asked to manually deposit crypto to an on-chain escrow wallet. The escrow stays `pending` until an external deposit is detected by the deposit monitor.

**Expected behavior:** The seller's custodial wallet balance should be checked, and if sufficient, the escrow should be auto-funded via an internal transfer (no on-chain transaction needed for custodial wallets).

## Business Rules

### SELL offers (user is selling crypto for fiat)
1. **Before creation:** Check user's custodial wallet has enough balance
2. **Confirmation popup:** "Are you sure you want to sell X USDT for ZAR? This amount will be locked in escrow until the trade completes."
3. **On confirm:** Create offer + internally move funds from custodial wallet → escrow (DB-level transfer, no blockchain tx)
4. **Escrow status:** Immediately `funded`
5. **If insufficient balance:** Show error "Insufficient balance. You have X USDT, need Y USDT."

### BUY offers (user is buying crypto with fiat)
1. **No escrow needed from buyer** — buyer deposits fiat via agreed payment method
2. **When a seller matches the trade:** The seller funds escrow (same auto-fund logic from their wallet)
3. **Confirmation popup:** "Are you sure you want to buy X USDT for ZAR?"

## Current DB State (example)

```
offer: 4a9a6583-8492-474f-a603-8674867d95a0
type: sell, crypto: USDT, fiat: ZAR
escrow_status: pending, escrow_amount: 12084592 (12.08 USDT)
escrow wallet balance: 0 (waiting for manual deposit)
```

## Implementation Required

### Backend
1. Add `POST /api/v1/escrow/{id}/auto-fund` endpoint
2. Check user's custodial wallet balance (DB `wallets` table)
3. Debit user wallet, credit escrow — single DB transaction
4. Set escrow status to `funded`
5. On offer creation for sell offers, auto-call this internally

### Frontend  
1. Add confirmation dialog before offer creation (different message for buy/sell)
2. For sell: check wallet balance client-side first, show warning if insufficient
3. Remove redirect to "Fund Escrow" page for custodial sell offers
4. Show success: "Offer created and escrow funded!"

## Files
- `qictrader-backend-rs/src/api/escrow.rs` — escrow handlers
- `qictrader-backend-rs/src/api/offers.rs` — offer creation
- `Frontend/src/app/(main)/offer/create/page.tsx` — offer creation page
- `Frontend/src/app/(offers)/offer/[id]/fund-escrow/page.tsx` — fund escrow page
