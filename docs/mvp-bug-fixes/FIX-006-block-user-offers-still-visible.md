# FIX-006: Block user — Blocked accounts/offers still visible; no receiving tickets

**Source:** Tester feedback — Phase 8 (75%, "Can still see accounts/offers that are blocked and no receiving tickets.")  
**Severity:** HIGH  
**Story IDs:** PROFILE-011 (Block a user), possibly support/ticket visibility

## Tester guide expectation (PROFILE-011)

1. Go to a user's profile and block them.
2. **Check:** You can no longer see their offers or trade with them.

## Problem

After blocking a user, testers can still see that user's accounts/offers. Blocked users' content should be hidden from the blocker. "No receiving tickets" may mean: (a) blocked user's support tickets are not visible where expected, or (b) ticket list/numbers not received — if (b), see FIX-008.

## Acceptance criteria

- [ ] **PROFILE-011:** After blocking a user, the blocking user no longer sees that user's offers on the marketplace (filter or exclude blocked users' offers).
- [ ] Blocked user's profile/page is hidden or shows "blocked" state to the blocker (no trading, no viewing offers).
- [ ] Trading with a blocked user is not possible (cannot start trade from their offer; existing trade handling per product rules).
- [ ] **Receiving tickets:** Clarify with testers: if "no receiving tickets" means support tickets not appearing in a list or no ticket number shown, treat under FIX-008; if it means "cannot receive ticket from blocked user" or similar, define expected behaviour and implement (e.g. blocked users cannot open tickets to blocker, or tickets from blocked users are filtered).

## Implementation notes

- Backend: ensure marketplace and offer list endpoints filter out offers (or users) blocked by the requesting user.
- Frontend: hide or disable blocked users' offers; profile view for blocked user shows blocked state.
- If "receiving tickets" is support-related, ensure ticket list and ticket number generation work (FIX-008).

## Priority

P1 — Phase 8 (Profile & User Management) is HIGH; block must be enforced for trust/safety.
