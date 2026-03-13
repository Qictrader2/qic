# MVP Bug Fixes — Tester Feedback (Phases 0–14)

Stories prepared from tester feedback. Each fix is mapped to Tester Guide story IDs and phases. **FIX-001 (escrow) unblocks Phases 0, 5, 6, 7** and should be prioritised first.

## Summary by phase

| Phase | Tester % | Main issues | Fix stories |
|-------|----------|-------------|-------------|
| 0 | 10% | Escrow broken | FIX-001 |
| 1 | 90% | AUTH-010 (logout all devices) | FIX-002 |
| 2 | 75% | KYC-002 to KYC-007 | FIX-003 |
| 3 | 75% | Withdraw/transfer; escrow | FIX-004, FIX-001 |
| 4 | 50% | Fixed vs floating rate pricing UX | FIX-005 |
| 5 | 0% | Escrow block | FIX-001 |
| 6 | 0% | Escrow block | FIX-001 |
| 7 | 0% | Escrow block | FIX-001 |
| 8 | 75% | Blocked users/offers visible; tickets | FIX-006, FIX-008 |
| 9 | 0% | No payment method edit; %/markup rules | FIX-007 |
| 10 | 75% | Ticket generation/numbers | FIX-008 |
| 11 | 25% | No notifications; 2FA no QR | FIX-009 |
| 12 | 0% | Affiliate codes; no affiliate list | FIX-010 |
| 13 | 0% | No reseller portal; code option | FIX-011 |
| 14 | 75% | Newsletter confirmation | FIX-012 |

## Fix list (priority order)

| ID | Title | Severity | Blocks |
|----|--------|----------|--------|
| [FIX-001](FIX-001-escrow-blocks-phase-0-5-6-7.md) | Escrow broken — blocks Phase 0, 5, 6, 7 | P0 / CRITICAL | ES-*, MP-*, RS-*, TF-*, TRADE-*, ESCROW-*, MOD-* |
| [FIX-002](FIX-002-auth-010-logout-all-devices.md) | AUTH-010 — Logout all devices | P1 | Phase 1 |
| [FIX-003](FIX-003-kyc-002-to-007-not-working.md) | KYC-002 to KYC-007 not working | P1 | Phase 2 |
| [FIX-004](FIX-004-wallet-withdraw-transfer.md) | Wallet — Withdraw/transfer | P1 | Phase 3 |
| [FIX-005](FIX-005-offer-pricing-fixed-vs-floating-rate.md) | Offer pricing — Fixed vs floating rate | P1 | Phase 4 |
| [FIX-006](FIX-006-block-user-offers-still-visible.md) | Block user — Blocked accounts/offers visible | P1 | Phase 8 |
| [FIX-007](FIX-007-payment-methods-edit-offers.md) | Payment methods — Edit on offers; %/markup rules | P2 | Phase 9 |
| [FIX-008](FIX-008-support-ticket-generation-and-numbers.md) | Support — Ticket generation and numbers | P1 | Phase 10 |
| [FIX-009](FIX-009-notifications-and-2fa-qr.md) | Notifications + 2FA QR code | P1 | Phase 11 |
| [FIX-010](FIX-010-affiliate-codes-and-list.md) | Affiliate — Codes and list | P1 | Phase 12 |
| [FIX-011](FIX-011-reseller-portal-and-code-option.md) | Reseller — Portal and code option | P1 | Phase 13 |
| [FIX-012](FIX-012-newsletter-confirmation.md) | Newsletter — Confirmation | P2 | Phase 14 |

## Dependencies

- **FIX-001 (escrow)** blocks FIX-010 (affiliate payouts) and full regression of Phase 0, 5, 6, 7.
- **MVP-001** (correct backend URL) may already address some affiliate/signup issues; verify before deep changes.
- **MVP-003** and **PROD-BUGS BUG 2b** are the main technical inputs for FIX-001.

## Related docs

- `../PROD-BUGS-2026-03-08.md` — production bugs (escrow wallet, gas, deposit monitor, etc.)
- `../mvp-bugs/` — MVP-001 (backend URL), MVP-002 (auth), MVP-003 (escrow auto-fund)
- `../SUPPORT-TICKET-GAPS.md` — support/email/Discord gaps
