# FIX-005: Offer creation — Fixed vs floating rate pricing UX (Phase 4)

**Source:** Tester feedback — Phase 4 (50%, "Fixed rate should give you the option to set a market price to build your mark-up percentage on, but floating rate shouldn't give you an option at all to select a market rate because the market rate is floating and will be determined by the current market price of the digital asset being offered.")  
**Severity:** HIGH  
**Story IDs:** OFFER-004, OFFER-005, OFFER-006 (offer create/edit)

## Problem

- **Fixed rate:** User should be able to set a market/reference price to build their mark-up percentage on. This option is missing or unclear.
- **Floating rate:** User must not be given an option to "select" a market rate — the rate is floating and determined by current market price. Showing or allowing a market-rate selection is wrong.

## Acceptance criteria

- [ ] **Fixed rate offers:** UI offers a way to set a reference/market price (or equivalent) that the mark-up percentage is applied to. Flow is clear and validated.
- [ ] **Floating rate offers:** No control to "select" or "set" market rate. UI makes clear that the effective rate will be the current market price of the asset (and optionally show live or last-known rate for transparency only).
- [ ] Offer type (fixed vs floating) is clearly indicated; switching type updates the form so fixed shows market-price input, floating does not.
- [ ] Backend accepts and stores fixed vs floating and any reference price only when applicable (fixed).

## Implementation notes

- Confirm offer model has `pricingType` (or similar) and optional `referencePrice` / `marketPrice` for fixed.
- Frontend: show/hide or enable/disable "market price" (or equivalent) based on fixed vs floating.
- Validation: floating offers must not send a user-selected market rate; fixed offers should require or default reference price for mark-up calculation.

## Priority

P1 — Phase 4 is CRITICAL (Offers & Marketplace); wrong pricing UX affects all offer creation.
