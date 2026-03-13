# FIX-009: Notifications not received; 2FA QR code option (Phase 11)

**Source:** Tester feedback — Phase 11 (25%, "Not receiving any notifications so unable to test notification functions because they all stem from existing notifications on your page to list. Not getting notifications so can't test things, e.g. GAP-014 deleting notifications. Receiving 2FA but not able to generate a QR code as opposed to using the code.")  
**Severity:** HIGH  
**Story IDs:** GAP-009 (real-time updates), GAP-014 (delete notifications); AUTH-007 / PROFILE-005 (2FA)

## Problem

1. **Notifications:** Users do not receive in-app (and possibly push/email) notifications. Notification list is empty, so flows that depend on it (e.g. GAP-014 delete notifications, real-time updates) cannot be tested.
2. **2FA:** Users can use a code for 2FA but cannot generate or use a QR code (e.g. for adding to authenticator app); only manual code entry is available.

## Acceptance criteria

### Notifications

- [ ] **Delivery:** In-app notifications are created and displayed when events occur (e.g. new trade, payment marked sent, message, trade completed). User sees them in the notifications list without needing to refresh.
- [ ] **Real-time (GAP-009):** When another user sends a message or starts a trade, the recipient sees the notification appear without refreshing the page (e.g. WebSocket or polling).
- [ ] **List:** Notifications list shows existing notifications so users can manage them.
- [ ] **GAP-014:** User can delete one or more notifications; they are removed from the list.
- [ ] If push or email notifications are in scope, document and implement; otherwise ensure in-app + list + delete work so Phase 11 is testable.

### 2FA

- [ ] **QR code:** When enabling 2FA, user is shown a QR code (e.g. TOTP provisioning URI) that can be scanned with an authenticator app, in addition to or instead of only showing a manual secret/code.
- [ ] **Code still works:** User can still complete 2FA setup and login using the code from the authenticator app (AUTH-007).
- [ ] **PROFILE-005:** Set up 2FA — scan QR or enter code; 2FA is required on login after setup.

## Implementation notes

- Notifications: backend must create notification records for relevant events; frontend must subscribe (WebSocket or polling) and render list; delete must soft-delete or remove from list.
- 2FA: backend should return TOTP secret and provisioning URL for QR; frontend should render QR (e.g. qrcode library) and optionally show manual entry for fallback.

## Priority

P1 — Phase 11 (Dashboard & Notifications) is MEDIUM but notifications are core to trade UX; 2FA QR is expected for security setup.
