# FIX-007: Payment methods — Edit on offers (not active trades); allow %/markup on offers

**Source:** Tester feedback — Phase 9 (0%, "No way to change payment methods and unable to test because feature doesn't exist to change payment methods on existing trade offers. Not a problem though. No need to allow them to change payment methods for existing trades. This will force them to create a new offer. Changing percentages and markups on existing trades should be allowed but disallowed on active trades.")  
**Severity:** MEDIUM (clarification + small feature)  
**Story IDs:** PAYMENT-001, PAYMENT-002; offer edit behaviour

## Problem

1. No way to change payment methods on **existing offers** — testers couldn't test payment method management. Product decision: **do not** allow changing payment methods for **active trades** (force new offer instead).  
2. **Allow:** Changing percentages and markups on **existing offers**.  
3. **Disallow:** Changing percentages and markups on **active trades** (trades in progress).

## Acceptance criteria

- [ ] **Payment methods on offers:** User can add/edit/delete payment methods in settings (PAYMENT-001, PAYMENT-002). When editing an **offer** (not an active trade), user can change which payment methods the offer accepts — save and persist.
- [ ] **No payment method change on active trades:** Once a trade is started, payment method for that trade is fixed. No UI to change payment method for the active trade (user must create a new offer for different payment methods).
- [ ] **Percentages/markup on offers:** User can edit an existing **offer** and change premium, markup, or other percentages; changes apply to the offer and to future trades only.
- [ ] **Percentages/markup on active trades:** User cannot change premium/markup/percentages for an **active trade**; existing trade keeps original terms. UI does not allow editing these fields on the trade (or API rejects).

## Implementation notes

- Offer edit: ensure payment method list and percentage/markup fields are editable for offers in draft/active-offer state; persist and reflect on marketplace.
- Trade model: ensure trade records snapshot payment method and terms at creation; no update endpoints that allow changing payment method or price terms for an active trade.
- Frontend: offer edit page shows payment methods and %; trade detail page does not offer edit for payment method or price terms.

## Priority

P2 — Phase 9 was 0% due to missing feature; implementing offer-level payment method and %/markup edit (and locking for active trades) unblocks testing and matches product intent.
