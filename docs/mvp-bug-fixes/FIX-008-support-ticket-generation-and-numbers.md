# FIX-008: Support — Ticket generation and ticket numbers not working

**Source:** Tester feedback — Phase 10 (75%, "Works perfectly except for generating tickets and ticket numbers.")  
**Severity:** HIGH  
**Story IDs:** SUPPORT-001 (Create support ticket), SUPPORT-003, SUPPORT-004; "no receiving tickets" (Phase 8) may overlap

## Tester guide expectation (SUPPORT-001)

1. Go to support section, create a new ticket with subject and description, submit.
2. **Check:** Ticket is created and user receives a confirmation.
3. Implicit: ticket has a unique identifier/number for reference.

## Problem

Tickets are not being generated correctly and/or ticket numbers are not shown or sent. Users cannot rely on ticket creation or reference a ticket by number.

## Acceptance criteria

- [ ] **Create ticket (SUPPORT-001):** Submitting the support form creates a ticket in the system and returns success to the user.
- [ ] **Ticket number/ID:** Each ticket has a unique identifier (e.g. ticket number or ID) that is displayed to the user after creation and in ticket list/detail views.
- [ ] **Confirmation:** User sees a clear confirmation after creation (e.g. "Ticket #12345 created") and can view the ticket in "My tickets" or equivalent.
- [ ] **Reply (SUPPORT-003):** User can reply to a ticket; reply is stored and associated with the ticket; ticket number/ID remains visible.
- [ ] **Close (SUPPORT-004):** User can close a ticket; status updates; ticket number/ID still visible in history.
- [ ] If "receiving tickets" in Phase 8 means "receiving ticket number/confirmation" — ensure confirmation and ticket number are visible and (if applicable) sent by email when email is implemented (see SUPPORT-TICKET-GAPS).

## Related docs

- `docs/SUPPORT-TICKET-GAPS.md` — no email provider; staff not notified; Discord webhooks missing. Ticket creation and numbering should work even without email; email can be added later for "receiving" confirmation.

## Priority

P1 — Phase 10 is MEDIUM but support is user-facing; ticket creation and numbers are baseline for support flow.
