# Google Credentials — Identifiers Only (QIC-228)

Template doc for capturing the **public identifiers** of every Google product owned by **QIC Trade Systems Limited**. Fill in the `<TBD>` values manually after each product is set up per `ops/google-entity-accounts.md`.

> **Hard rule:** this file contains **identifiers only**. No passwords. No API keys. No OAuth secrets. No 2FA backup codes. No card PANs or CVVs. Anything secret goes in Bitwarden (or whichever password manager the entity standardises on) under the `QIC Trade Systems Limited` vault. See §Security Notes at the end.

---

## Workspace

| Field | Value |
|---|---|
| Primary domain | `qictrader.com` |
| Secondary domains | `<TBD: list any `qictrader.co.za` / `qictrade.com` aliases added to Workspace, or "None">` |
| Workspace Customer ID | `<TBD: C0xxxxxxxx — Admin Console → Account → Account settings → top of page>` |
| Workspace edition | `<TBD: Business Starter / Standard / Plus — Admin Console → Billing → Subscriptions>` |
| Number of seats | `<TBD: count of active users — should match users listed in §Ownership matrix>` |
| Tenant created | `<TBD: YYYY-MM-DD>` |
| Super admins | `marcello@qictrader.com`, `admin@qictrader.com` |

### Admin recovery (owners only — values in Bitwarden)

| Account | Recovery email owner | Recovery phone owner |
|---|---|---|
| `marcello@qictrader.com` | `<TBD: e.g. Marcello personal Gmail>` | `<TBD: Marcello SA mobile>` |
| `admin@qictrader.com` | `<TBD: same as above, or distinct break-glass mailbox>` | `<TBD: same as above, or distinct break-glass SIM>` |

---

## Billing

| Field | Value |
|---|---|
| Legal entity name | QIC Trade Systems Limited |
| CIPC registration number | `<TBD: 2024/XXXXXX/07>` |
| Registered address | `<TBD: full street address, postal code, country>` |
| VAT number (SARS) | `<TBD: 10-digit, or "Not VAT-registered">` |
| Billing contact email | `billing@qictrader.com` |
| Billing contact phone | `<TBD: SA business number>` |
| Google Billing account ID | `<TBD: 01ABCD-1234EF-567890 format — Billing → Account management>` |
| Payment method type | `<TBD: Visa / Mastercard / SEPA debit>` |
| Payment method last 4 | `<TBD: last 4 digits only — never the full PAN>` |
| Card expiry month/year | `<TBD: MM/YYYY — for renewal reminders>` |
| Statement delivery | `<TBD: email-only / email + paper>` |

---

## Google Ads

| Field | Value |
|---|---|
| Account name | `<TBD: e.g. "QIC Trader — ZAR">` |
| Customer ID | `<TBD: XXX-XXX-XXXX — top right of Ads UI>` |
| Manager (MCC) ID | `<TBD: XXX-XXX-XXXX if linked to an MCC, otherwise "None — standalone account">` |
| Currency | ZAR |
| Time zone | `Africa/Johannesburg` |
| Billing profile | Same as §Billing (Google Billing account ID above) |
| 2FA enforced on account | `<TBD: Yes/No — should be Yes per runbook §4.1>` |

---

## Search Console

| Field | Value |
|---|---|
| Property type | Domain property |
| Verified property | `qictrader.com` |
| Verification method | DNS TXT |
| DNS provider hosting the TXT record | `<TBD: Cloudflare / Route 53 / Namecheap / etc.>` |
| Verification date | `<TBD: YYYY-MM-DD>` |
| Sitemap submitted | `<TBD: https://qictrader.com/sitemap.xml — or "Not yet, frontend sitemap not implemented">` |

---

## GA4 (QIC-231 — partially provisioned)

| Field | Value |
|---|---|
| Account name | `<TBD: verify in GA4 UI; expected "QIC Trade Systems Limited">` |
| Property name | `<TBD: verify/create; expected "QicTrader — Web">` |
| Property ID | `<TBD: 9-digit numeric, e.g. 387654321 — set during QIC-231>` |
| Production web stream name | `Qictrader` |
| Production web stream URL | `https://www.qictrader.com` |
| Production web stream ID | `14911868847` |
| Production Measurement ID | `G-8J1XWX5ZKQ` |
| Staging web stream | `<TBD: create/verify for Vercel preview domain>` |
| Staging Measurement ID | `<TBD: G-XXXXXXXXXX — set during QIC-231>` |
| Browser verification status | `Not yet proven live: page_view and sign_up still need GTM/GA4 DebugView verification` |
| BigQuery export linked | `<TBD: Yes/No — set during QIC-231>` |

See `ops/ga4-gtm-setup.md` once QIC-231 ships.

---

## GTM (QIC-231 — pending browser verification)

| Field | Value |
|---|---|
| Account name | `<TBD: verify/create; expected "QIC Trade Systems Limited">` |
| Container name | `<TBD: verify/create; expected "qictrader.com">` |
| Container public ID | `GTM-5X3M5QCS` |
| Container internal ID | `<TBD: numeric, used in GTM REST API calls>` |
| Workspace name | `<TBD: e.g. "Default Workspace">` |
| Environments configured | `<TBD: Live, Preview/Staging — set during QIC-231>` |
| Vercel env var | `NEXT_PUBLIC_GTM_ID=GTM-5X3M5QCS` set for Preview and Production |
| Publish status | `Published v1 — GA4 page views and sign_up` |

See `ops/ga4-gtm-setup.md` once QIC-231 ships.

---

## Google Business Profile

| Field | Value |
|---|---|
| Business name | QIC Trader |
| Category | Financial Service |
| Service area | South Africa |
| Business Profile account ID | `<TBD: accounts/{accountId} — from URL when managing>` |
| Location ID | `<TBD: locations/{locationId} — from URL when managing>` |
| Verification status | `<TBD: Verified / Pending / Rejected>` |
| Verification method | `<TBD: Video / Phone / Postcard>` |
| Verification date | `<TBD: YYYY-MM-DD>` |

---

## YouTube Brand Channel

| Field | Value |
|---|---|
| Channel name | QIC Trader |
| Channel handle | `<TBD: @qictrader if available, else alternate>` |
| Channel ID | `<TBD: UCxxxxxxxxxxxxxxxxxxxxxx — Studio → Settings → Channel → Advanced>` |
| Brand Account ID | `<TBD: numeric — myaccount.google.com/brandaccounts>` |
| Country of residence | South Africa |
| Monetisation status | `<TBD: Not eligible / Pending / Enabled — track for tax compliance>` |

---

## Ownership matrix

Roles per person per product. Update whenever access changes. Use `ops/google-access-offboarding.md` when removing a person.

Role legend:
- **Super Admin** — full control, can change billing, can delete the resource, can manage other admins
- **Admin** — full operational control, cannot change billing or delete the resource
- **Editor** — can create/modify content but cannot manage users
- **Viewer** — read-only
- **None** — no access

| Person | Workspace | Billing | Ads | Search Console | GA4 | GTM | YouTube |
|---|---|---|---|---|---|---|---|
| Marcello Haupt (`marcello@qictrader.com`) | Super Admin | Admin | Admin | Owner | `<TBD: set in QIC-231>` | `<TBD: set in QIC-231>` | Manager |
| JP van Zyl (`<TBD: confirm jp@qictrader.com vs personal>`) | `<TBD: Admin / None>` | `<TBD: None recommended — billing is Marcello+admin@>` | `<TBD: Admin / Editor>` | `<TBD: Owner / Editor>` | `<TBD: set in QIC-231>` | `<TBD: set in QIC-231>` | `<TBD: Manager / None>` |
| `admin@qictrader.com` (break-glass) | Super Admin | Admin | Admin | Owner | `<TBD: set in QIC-231>` | `<TBD: set in QIC-231>` | Owner |
| Future ops hire | None (until provisioned) | None | None | None | None | None | None |

**Rule:** any cell upgraded from None requires a documented reason in this file (add a line below the table). Any cell downgraded to None triggers the offboarding checklist.

### Access change log

Append-only. Newest at the top. Format: `YYYY-MM-DD | person | product | from → to | reason | actioned by`.

- `<TBD: first entry will be the initial provisioning per QIC-228>`

---

## Security Notes

### What lives here (and only here)

- Account IDs (numeric or alphanumeric public identifiers)
- Verified domain names
- Role assignments per person
- Recovery contact **owners** (not the email addresses or phone numbers themselves)
- Last 4 of payment cards, expiry month/year (for renewal tracking — not full PAN, not CVV)

### What does NOT live here — ever

- Passwords for any Google account
- 2FA backup codes
- OAuth client secrets, refresh tokens, service-account JSON keys
- API keys (Ads API, Analytics Data API, YouTube Data API, etc.)
- Full credit card numbers, CVVs
- Recovery email addresses or phone numbers as values (only the owner's name)
- Anything tagged "secret" in a Google product UI

### Where secrets DO live

- **Password manager** — Bitwarden (or 1Password if the entity migrates). Vault: `QIC Trade Systems Limited`. Folder per product. Item naming: `{product} / {account email}` (e.g. `Workspace / admin@qictrader.com`).
- **App passwords for transactional senders** (e.g. `noreply@` SMTP) — same vault, separate item, tagged `app-password`.
- **Service-account JSON keys** (if any are ever generated for GA4 Data API, BigQuery, etc.) — encrypted at rest, mounted into Heroku via config vars, never committed. See `ops/heroku-apps.md` for the credential-hygiene rule that applies to all secrets in Heroku config.

### If a secret leaks (e.g. someone pastes a password in Slack or this repo)

1. **Rotate at the issuer** — change the password / regenerate the key / revoke the token in the Google product UI **before** worrying about cleanup. Leak duration is what matters.
2. **Audit access** — `Admin Console → Security → Investigation tool` for Workspace, "Account access" tab for each downstream product. Look for sessions from unexpected IPs.
3. **Remove from the leak channel** — delete the message / force-push the commit removed (and accept that it may still be cached; rotation in step 1 is what protects you, not cleanup).
4. **Log it** — append an entry to the §Access change log above with `compromise → rotated`, the affected account, and the rotation date.

### Quarterly review (calendar this)

Every quarter, sit down with this file open and walk top-to-bottom:

- [ ] Every `<TBD>` either filled in or escalated as a blocker
- [ ] Every row in the ownership matrix matches reality (cross-check against each product's user-access UI)
- [ ] 2FA still enforced on the admin OU
- [ ] Billing card not expiring within 60 days
- [ ] Recovery email/phone owners still reachable
- [ ] No new Google products created outside this doc (use `Admin Console → Reports → Audit and investigation → Login audit log` filtered by `admin@` and `marcello@` to spot product activations)

---

## Cross-references

- `ops/google-entity-accounts.md` — setup runbook for each product.
- `ops/google-access-offboarding.md` — what to do when a person's access changes.
- `ops/ga4-gtm-setup.md` — analytics setup (QIC-231).
- Trello ticket: QIC-228.
