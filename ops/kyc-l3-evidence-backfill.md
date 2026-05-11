# KYC-L3-ADMIN-EVIDENCE-001 — Historical L3 evidence backfill

**Trello:** [ZXoiUaNP](https://trello.com/c/ZXoiUaNP)
**Migration:** `20260511170000_KYC_L3_ADMIN_EVIDENCE_001_approval_evidence`
**Shipped:** 2026-05-11

## Context

The 2026-05-11 migration adds a `kyc_submissions.approval_evidence`
column. From that release onward the API enforces that every admin
L3 grant carries a structured pointer to the offline proof-of-address
(URL or note). Historical L3 admin grants pre-date this rule and
land with `approval_evidence IS NULL` — those are the rows ops needs
to backfill.

## Finding affected rows

Run this against the prod DB to get the list:

```sql
SELECT
  u.id      AS user_id,
  u.email,
  ks.id     AS submission_id,
  ks.reviewed_at,
  ks.rejection_reason AS admin_note
FROM kyc_submissions ks
JOIN users u ON u.id = ks.user_id
WHERE ks.level = 3
  AND ks.status = 'approved'
  AND ks.approval_source = 'admin'
  AND ks.approval_evidence IS NULL
ORDER BY ks.reviewed_at;
```

As of 2026-05-10 (the audit that produced this ticket) there were 5
rows — all approved between 2026-04-29 and 2026-05-05 by
`admin@qictrader.com`:

| user_id | email |
| --- | --- |
| 11ae4bf5-3d0a-487f-b63c-138a9a23f8bc | youshanv@ |
| 76147978-ddd2-4baf-a1ac-147a386bf890 | (internal) |
| 986cf009-b8b6-4d4f-9ceb-85182530c2ba | (internal) |
| de7a1cc6-0990-4fce-963e-fc5e1df34899 | marcellohaupt@ |
| f74f284f-ccd0-4592-ae93-2c1d29275ac7 | jp.vanzyl@ |

## Backfill workflow

Per row above:

1. Locate the offline POA (email thread, Slack permalink, S3 link,
   archived PDF).
2. Re-run the admin override endpoint with the evidence — the repo
   layer's idempotency check now patches `approval_evidence` onto
   the existing row when the value differs (no duplicate insert):

   ```bash
   curl -X POST "https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/admin/users/{user_id}/kyc/override" \
     -H "Authorization: Bearer $ADMIN_JWT" \
     -H "Content-Type: application/json" \
     -d '{
       "level": 3,
       "action": "grant",
       "reason": "Backfilling approval_evidence per KYC-L3-ADMIN-EVIDENCE-001",
       "evidenceUrl": "https://...",
       "evidenceNote": "POA archived in SharePoint /compliance/2026/..."
     }'
   ```

3. Verify the row updated:

   ```sql
   SELECT id, approval_evidence, updated_at
   FROM kyc_submissions
   WHERE id = '<submission_id>';
   ```

## Notes

- **Do NOT** auto-revoke L3 status on rows missing evidence. That
  would break trading limits for active users mid-flow. Rows stay
  approved; the compliance ask is only that we record WHERE the
  paperwork lives, not that the paperwork didn't exist.
- The `kyc_l3_admin_override::admin_grant_backfill_evidence_onto_existing_row`
  integration test pins the idempotent-patch behaviour so this path
  can't regress.
- Once the 5 rows are backfilled, the same SQL above should return
  zero rows. A monitoring item could fail loudly if any L3 admin
  grant lands with NULL evidence post-shipment (the API rejects this
  now, so it shouldn't happen — defense in depth).
