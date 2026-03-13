# FIX-003: KYC function not working — KYC-002 to KYC-007

**Source:** Tester feedback — Phase 2 (75%, "KYC function isn't working as it should ~ KYC-002 to 007")  
**Severity:** HIGH  
**Story IDs:** KYC-001 through KYC-007

## Affected stories

| ID | Tester guide expectation |
|----|---------------------------|
| KYC-001 | View KYC status (None, Pending, Approved, Rejected) |
| KYC-002 | Submit government ID — upload succeeds, shows "Pending" |
| KYC-003 | Submit selfie — upload succeeds |
| KYC-004 | Submit proof of address — upload succeeds |
| KYC-005 | Trading limits displayed based on KYC level |
| KYC-006 | Resubmit rejected documents (upload new docs and resubmit) |
| KYC-007 | Blocked from trading without KYC on offers that require KYC |

## Problem

KYC flows (submit ID, selfie, proof of address, view status, limits, resubmit, trading gate) are not working as described. Testers report ~25% failure in Phase 2 attributable to KYC.

## Acceptance criteria

- [ ] **KYC-001:** KYC/verification page shows current status (None, Pending, Approved, Rejected).
- [ ] **KYC-002:** User can upload government ID (passport, national ID, or driver's licence); upload succeeds and status shows "Pending" where applicable.
- [ ] **KYC-003:** User can upload selfie; upload succeeds.
- [ ] **KYC-004:** User can upload proof of address; upload succeeds.
- [ ] **KYC-005:** Trading limits are shown based on KYC level (e.g. on KYC page or trading limits section).
- [ ] **KYC-006:** If KYC was rejected, user can upload new documents and resubmit.
- [ ] **KYC-007:** User without completed KYC is blocked from starting a trade on an offer that requires KYC, with clear message to complete KYC first.

## Implementation notes

- Confirm backend endpoints for upload (ID, selfie, proof of address) and status/limits exist and return expected shapes.
- Confirm frontend sends correct payloads (e.g. file upload content-type, field names) and handles success/error and status updates.
- Ensure "offer requires KYC" is defined (e.g. offer flag or global rule) and enforced at trade creation.

## Priority

P1 — Registration & Identity is HIGH PRIORITY; KYC blocks trading for restricted offers and compliance.
