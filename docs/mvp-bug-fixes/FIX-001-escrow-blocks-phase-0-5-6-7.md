# FIX-001: Escrow function broken — blocks Phase 0, 5, 6, 7

**Source:** Tester feedback — Phase 0 (10%), Phase 5 (0%), Phase 6 (0%), Phase 7 (0%)  
**Severity:** CRITICAL — blocks most core flows  
**Related:** MVP-003 (escrow auto-fund), PROD-BUGS BUG 2b (no escrow wallet available)

## Affected story IDs

| Phase | Stories blocked |
|-------|-----------------|
| Phase 0 | ES-001 to ES-009 (Escrow Logic), MP-001 to MP-008 (Marketplace), RS-001 to RS-010 (Reseller), TF-001 to TF-010 (Trade Flow) |
| Phase 5 | TRADE-001 to TRADE-013 (Trading Flow) |
| Phase 6 | ESCROW-004, ESCROW-006, GAP-003, LDG-*, AUD-* (Escrow & Ledger) |
| Phase 7 | MOD-001 to MOD-017, ADMIN-007, GAP-008, MPR-008 (Disputes & Moderation) |

## Problem

Broken escrow function stops most Phase 0 testing and makes Phases 5, 6, and 7 untestable. Testers report: "CAN'T USE ESCROW SERVICE SO CAN'T TEST. BIG BLOCK."

## Root causes (from existing docs)

1. **MVP-003:** Sell offers do not auto-fund escrow from custodial wallet; users are sent to "Fund Escrow" and escrow stays `pending`.
2. **PROD-BUGS BUG 2b:** After placing an offer, "no escrow wallet available" — backend creates escrow record but escrow wallet creation/assignment fails (blockchain service integration).
3. **PROD-BUGS BUG 4:** Deposit monitor rate-limited (429) — BTC/TRON/ETH deposits not detected, so manual funding may also fail.

## Acceptance criteria (fix complete when)

- [ ] **ES-001:** Trade opens immediately; seller's balance shows amount locked in escrow within 1–2 seconds (no waiting for blockchain confirmations).
- [ ] **ES-002:** On trade completion, buyer receives crypto; seller's locked amount is released (minus platform fee).
- [ ] **ES-003:** On cancellation, seller's locked funds return to available balance instantly.
- [ ] **ES-004:** During dispute, funds stay locked; neither party can release or refund.
- [ ] **ES-005:** Moderator resolution releases/refunds correctly (buyer or seller).
- [ ] No "no escrow wallet available" after offer creation when user has sufficient custodial balance.
- [ ] Sell offer creation either auto-funds escrow from custodial balance (MVP-003) or clearly guides user and succeeds when wallet is assigned.

## Implementation references

- `docs/mvp-bugs/MVP-003-escrow-auto-fund.md` — auto-fund from custodial wallet
- `docs/PROD-BUGS-2026-03-08.md` — BUG 2b (escrow wallet), BUG 4 (deposit monitor)

## Priority

P0 — Unblock Phase 0, 5, 6, 7 before full regression of core flows is possible.
