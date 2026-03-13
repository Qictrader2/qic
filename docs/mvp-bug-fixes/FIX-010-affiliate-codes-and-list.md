# FIX-010: Affiliate — Codes not working; no list of affiliates (Phase 12)

**Source:** Tester feedback — Phase 12 (0%, "No affiliate codes are working. Not showing a list of affiliates. Big block for testing escrow function with regards to affiliate reward payouts and other affiliate related functions.")  
**Severity:** HIGH  
**Story IDs:** AFF-001 (Affiliate code on signup), AFFILIATE-001 to AFFILIATE-007

## Problem

1. Affiliate/referral codes do not work (e.g. at signup or when applying code).  
2. Affiliate list is not shown — users (or admins) cannot see a list of affiliates or referrals.  
3. Escrow/trade flows that credit affiliate rewards cannot be tested until escrow works (FIX-001); once escrow is fixed, affiliate payouts and tiers must work.

## Acceptance criteria

- [ ] **AFF-001:** On registration page, there is a field for affiliate/referral code. Entering a valid code and completing signup correctly links the new user to the referrer (affiliate).
- [ ] **Affiliate codes work:** Valid affiliate codes are accepted; invalid or expired codes show a clear error. Signup with valid code registers the referral relationship.
- [ ] **AFFILIATE-001 / dashboard:** Affiliate dashboard loads and shows referral stats (e.g. referral link, number of referrals, earnings).
- [ ] **List of affiliates/referrals:** Affiliate can see a list of users who signed up with their referral (AFFILIATE-003 View referrals). If "list of affiliates" means admin list of all affiliates, provide an admin view or endpoint as needed.
- [ ] **AFFILIATE-002:** User can generate a referral link; link is unique and tracks signups.
- [ ] **Earnings and payouts (after FIX-001):** Once escrow works, affiliate commission from referred users' trades is calculated and displayed (AFFILIATE-004, AFFILIATE-005); payout request and history (AFFILIATE-007) work.
- [ ] **Root cause:** If affiliate features hit the wrong backend (e.g. old Node backend), fixing API URL (MVP-001) may resolve; verify affiliate endpoints and frontend base URL.

## Related docs

- MVP-001 / PROD-BUGS BUG 7: Vercel frontend hitting old backend — affiliate dashboard was blank; confirm affiliate API calls use correct backend.
- FIX-001: Escrow must work for affiliate reward payouts to be testable.

## Priority

P1 — Phase 12 is HIGH; affiliate codes and list are baseline; payouts depend on escrow (FIX-001).
