# FIX-002: AUTH-010 — Logout all devices not working

**Source:** Tester feedback — Phase 1 (90%, "Perfect except for story code AUTH-010")  
**Severity:** HIGH  
**Story ID:** AUTH-010

## Tester guide expectation (AUTH-010)

1. Log in on two different browsers (e.g., Chrome + Firefox).
2. On one browser, use "Logout all devices" (or equivalent).
3. Go to the other browser and refresh the page.
4. **Check:** User should be logged out on all browsers.

## Problem

"Logout all devices" does not invalidate sessions on other devices; after using it, the second browser remains logged in after refresh.

## Acceptance criteria

- [ ] "Logout all devices" (or equivalent) is visible and discoverable in security/settings or profile.
- [ ] When user clicks "Logout all devices", all other sessions are invalidated (e.g. refresh tokens or server-side session store cleared for that user).
- [ ] After invoking it, refreshing any other tab/device shows user as logged out (redirect to login or clear auth state).
- [ ] Current device may either stay logged in (single session) or also log out — document intended behaviour and ensure it matches UI copy.

## Areas to check

- Backend: session/token invalidation (e.g. revoke all refresh tokens for user, or clear server-side session entries).
- Frontend: clear local storage/session storage and any in-memory auth state; redirect to login when API returns 401 after invalidation.

## Priority

P1 — Phase 1 is otherwise 90%; this is the only failing story in Platform Foundation.
