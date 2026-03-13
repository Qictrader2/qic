# FIX-011: Reseller — No portal/stats; code option missing (Phase 13)

**Source:** Tester feedback — Phase 13 (0%, "Registering as a resell feature doesn't exist so can't view my reselling stats. Possibly create this portal under 'My Profile' or 'My Trades' or both. Only link is available and option to use a code does not exist.")  
**Severity:** HIGH  
**Story IDs:** RESELLER-002 (Reseller profile/dashboard), RS-001–RS-010 (resell flows); referral/code entry

## Problem

1. **No reseller portal:** Users cannot find a reseller dashboard or "my reselling stats" — no dedicated place to see resell offers, volume, or commission.  
2. **Placement:** Testers suggest putting the reseller portal under "My Profile" or "My Trades" (or both).  
3. **Code vs link only:** Reseller/referral entry is only via link; there is no option to enter a reseller (or referral) **code** when starting a trade or signing up.

## Acceptance criteria

- [ ] **Reseller portal / RESELLER-002:** A reseller can access a dedicated reseller view (dashboard or section) that shows: active resell offers, resell trade count, volume, commission earned, and any stats required by RS-006 (reseller dashboard stats).
- [ ] **Discoverability:** Reseller portal is reachable from "My Profile" and/or "My Trades" (or clearly linked from nav) so users can find "my reselling stats" and manage resell offers.
- [ ] **RS-006:** Dashboard shows: number of active resell offers, total trade volume, total commission earned, active trades count; updates after new trades (RS-007 trade history with breakdown).
- [ ] **Code option:** Where a reseller or referral is applied (e.g. at signup, or when starting a trade from a resell offer), the user can either use a **link** or enter a **code** (e.g. reseller code, referral code). Both link and code identify the same reseller/affiliate and work correctly.
- [ ] **RS-001–RS-010:** Resell offer creation, pricing, trading, settlement, and dispute behaviour remain correct; reseller portal surfaces the data for these flows.

## Implementation notes

- Add a "Reseller" or "My Reselling" section under Profile or Trades; or a nav item that routes to reseller dashboard. Backend may already expose reseller stats; frontend needs to consume and render.
- Add optional "Reseller code" or "Referral code" input where referral/reseller context is applied; validate code and associate user/session with reseller same as when coming from link.

## Priority

P1 — Phase 13 is blocked without a reseller portal and code option; reseller flow is part of core design (Phase 0).
