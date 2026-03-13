# FIX-004: Wallet — Can't withdraw or transfer (off-platform or onboard)

**Source:** Tester feedback — Phase 3 (75%, "Can't withdraw crypto or transfer off platform or to an onboard wallet. Escrow broken also affects part of the test.")  
**Severity:** HIGH  
**Story IDs:** WALLET-003, WALLET-004, WALLET-006, GAP-013, GAP-017 (partial)

## Affected stories

| ID | Expectation |
|----|-------------|
| WALLET-003 | Withdraw crypto — balance decreases, crypto arrives at external wallet |
| WALLET-004 | Internal transfer (e.g. trading wallet ↔ main) — completes instantly, balances update |
| WALLET-006 | Fee estimate shown before withdrawal confirm |
| GAP-013 | Wallet transfers complete and balances update |
| GAP-017 | Unlock funds — funds move back to available balance |

## Problem

Users cannot withdraw crypto or transfer (off-platform or to another onboard wallet). Escrow issues also affect some wallet tests (e.g. locked balance display).

## Acceptance criteria

- [ ] **Withdraw (WALLET-003):** User can enter external address and amount, confirm withdrawal; platform balance decreases by amount + gas/fee; crypto arrives at external address (or clear error if network/validation fails).
- [ ] **Fee before confirm (WALLET-006):** Withdrawal flow shows fee/gas estimate before user confirms; no surprise deductions.
- [ ] **Internal transfer (WALLET-004, GAP-013):** If the product has multiple wallet types (e.g. trading vs main), user can transfer between them; transfer completes in-app without blockchain wait; both balances update.
- [ ] **Unlock (GAP-017):** Where funds can be unlocked (e.g. after cancelled trade), unlock moves funds back to available balance.
- [ ] Escrow-related wallet behaviour (locked balance, post-trade updates) is fixed per FIX-001 where it affects Phase 3.

## Related docs

- PROD-BUGS BUG 3: Withdrawal gas fee endpoint returns 500 (fallback used) — may affect WALLET-006 UX.
- FIX-001: Escrow fixes may restore locked-balance and post-trade wallet behaviour.

## Priority

P1 — Phase 3 is CRITICAL (Wallet & Money); withdraw and transfer are core.
