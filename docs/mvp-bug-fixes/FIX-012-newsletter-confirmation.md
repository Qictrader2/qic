# FIX-012: Newsletter — No confirmation or newsletters received (Phase 14)

**Source:** Tester feedback — Phase 14 (75%, "No confirmation or newsletters received. Everything else works during this phase.")  
**Severity:** MEDIUM  
**Story ID:** VISITOR-011 (Newsletter signup)

## Tester guide expectation (VISITOR-011)

1. Find the newsletter signup form (e.g. in footer).  
2. Enter email and subscribe.  
3. **Check:** User sees a confirmation message.

## Problem

After subscribing to the newsletter, users do not receive a confirmation (in-app or email) and do not receive newsletters. This may be due to: no confirmation step in UI, no email provider (see SUPPORT-TICKET-GAPS), or newsletter sending not implemented.

## Acceptance criteria

- [ ] **Confirmation message:** After submitting the newsletter form, the user sees a clear in-app confirmation (e.g. "Thanks, you're subscribed" or "Check your email to confirm") so they know the signup was accepted.
- [ ] **No duplicate signups:** Submitting the same email again shows a friendly message (e.g. "Already subscribed") and does not error unnecessarily.
- [ ] **Newsletters received (if in scope):** If the product promises email newsletters, implement or integrate sending (e.g. email provider + campaign tool) so subscribers receive them; otherwise document "confirmation only" and add email sending later.
- [ ] **Discord/webhook (optional):** If internal process uses Discord for new signups, ensure DISCORD_NEWSLETTER_WEBHOOK_URL is set so signups are logged (see SUPPORT-TICKET-GAPS).

## Related docs

- `docs/SUPPORT-TICKET-GAPS.md` — no email provider; DISCORD_NEWSLETTER_WEBHOOK_URL blank. At minimum, in-app confirmation and persistent storage of signups; email confirmation and sending when provider is added.

## Priority

P2 — Phase 14 is LOW; in-app confirmation is quick win; email delivery depends on provider and roadmap.
