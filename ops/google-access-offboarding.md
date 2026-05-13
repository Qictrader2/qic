# Google Access Offboarding (QIC-228)

Checklist for revoking a person's access to QIC Trade Systems Limited's Google products. Triggered when someone leaves the company, changes role, or has their access compromised.

> **Why order matters:** if you suspend the Workspace account first, the person loses sign-in to Ads / GA4 / GTM — but those products may still list them as the **sole owner** of campaigns, properties, or containers. You can't transfer ownership from a suspended user. Always strip ownership from downstream products first, then suspend Workspace last.

---

## When to run this checklist

- Employee or contractor leaving
- Role change that removes need-to-know (e.g. moving off ops)
- Suspected credential compromise (run immediately, ask questions after)
- Vendor / agency engagement ending (Ads agency, SEO consultant, etc.)

Pull `ops/credentials-google.md` §Ownership matrix open while you work — every row that mentions the leaving person is a step.

---

## Pre-flight

Before touching anything:

1. **Confirm the trigger in writing.** Slack DM, email, or HR ticket. Don't offboard someone on a hallway conversation.
2. **Identify all accounts in scope.** A person may have:
   - A primary Workspace user (`firstname@qictrader.com`)
   - A separate admin account (`firstname.admin@`)
   - Personal Google accounts granted access to QIC products (e.g. `firstname@gmail.com` as a viewer on Ads)
   - Service accounts they created but didn't document (audit — see §5)
3. **Schedule a window.** Aim for 30 minutes uninterrupted. Each step is fast but stopping halfway leaves the org in a weird half-revoked state.
4. **Open `ops/credentials-google.md`.** You'll update the §Access change log as you go.

---

## 1. Transfer ownership of resources the person uniquely owns

Do this FIRST. Once a Workspace account is suspended, you can't reassign anything they own from inside Google's UI — only Google Support can, and that takes days.

### 1.1 Google Ads

For each Ads account where the leaving person is the sole admin:

1. Sign in to <https://ads.google.com> as `admin@qictrader.com`.
2. **Tools and settings → Access and security → Users**.
3. Find the leaving person's row. Confirm there is **at least one other Admin** before proceeding (if not, promote `admin@qictrader.com` or `marcello@qictrader.com` to Admin first).
4. Note any campaigns, conversion actions, or audience lists they created — Ads doesn't have per-resource ownership, so account-level admin removal is enough. But document any campaigns that should pause/end if the person was their sole operator.

### 1.2 Search Console

Domain properties don't have a single "owner" — they have multiple Owners. But if the leaving person is the **last remaining Owner** verified via their personal email (rare; should be DNS-verified through the entity), the property has to be re-verified before revoke.

1. <https://search.google.com/search-console> → property `qictrader.com` → **Settings → Users and permissions**.
2. Confirm DNS-based verification is still in place (`dig +short TXT qictrader.com @8.8.8.8` should still show `google-site-verification=...`).
3. Confirm `admin@qictrader.com` is listed as **Owner**.
4. If the leaving person was the only Owner, add `admin@qictrader.com` as Owner first.

### 1.3 GA4

Set during QIC-231; once that ticket ships, the offboarding step here is:

1. <https://analytics.google.com> → **Admin → Account access management** (account-level) **and Property access management** (property-level — they're separate).
2. Confirm at least one other Administrator exists at both account and property level.
3. If the leaving person created any **custom audiences**, **conversion events**, or **explorations**, audit them. Audiences and conversion events are property-level and survive user removal. Explorations are user-scoped — they disappear when the user is removed. If any exploration is important, copy it to a shared report or screenshot it before revoking.

### 1.4 GTM

GTM has hard per-container ownership. A user with "Publish" permission can deploy tags; only an "Administrator" can manage users. If the leaving person is the sole Administrator on a container, you cannot reassign — escalate to Google Support.

1. <https://tagmanager.google.com> → container → **Admin → User Management**.
2. Confirm at least one other user has the **Administrator** role.
3. Check **Workspaces** — if the leaving person had an open workspace with uncommitted changes, either commit or discard those changes before revoking access. Their workspace becomes orphaned otherwise.

### 1.5 YouTube Brand Channel

Brand channels are tied to a Google Brand Account, which has Owner / Manager roles. There must always be at least one Owner.

1. <https://studio.youtube.com> → channel → **Settings → Permissions**.
2. Confirm `admin@qictrader.com` is **Owner**.
3. If the leaving person uploaded videos as themselves, the videos remain on the channel (uploads belong to the channel, not the uploader user) — no transfer needed. Confirm by checking video metadata.

### 1.6 Google Business Profile

1. <https://business.google.com> → profile → **Menu → Business Profile settings → People and access**.
2. Confirm at least one other Owner exists.
3. If the leaving person was the only Owner, transfer ownership to `admin@qictrader.com` and wait 7 days (Google enforces a cooldown on ownership transfer to prevent hostile takeover). During this window the leaving person still has Owner access — assess the compromise risk and decide whether to escalate to Google Support for emergency transfer.

---

## 2. Revoke access from downstream products

Now that ownership is safe, remove the person from each product. Do these in any order, but get through all of them.

### 2.1 Google Ads → remove user

`Tools and settings → Access and security → Users` → click leaving person → **Remove access**.

### 2.2 Search Console → remove user

`Settings → Users and permissions` → leaving person → **trash icon** → confirm.

### 2.3 GA4 → remove from both Account and Property

`Admin → Account access management` → remove. `Admin → Property access management` → remove. Both. They are independent — removing only one leaves dangling access.

### 2.4 GTM → remove user

`Admin → User Management` → leaving person → **Remove user**. Repeat for every container they had access to (there should only be one for QIC, but verify by checking the container picker).

### 2.5 YouTube → remove user

Studio → **Settings → Permissions** → leaving person → **trash icon**.

### 2.6 Google Business Profile → remove user

`Menu → Business Profile settings → People and access` → leaving person → **Remove**.

### 2.7 Any Google Cloud projects

If GCP projects exist (none as of QIC-228, but check anyway):

1. <https://console.cloud.google.com> → **IAM & Admin → IAM**.
2. Find the leaving person → **Remove principal**.
3. Repeat per project — IAM is per-project, not org-wide, unless org-level policies are in place (they aren't yet).

---

## 3. Revoke external integrations granted by the leaving person

People grant OAuth access to third-party apps under their own user, often without telling anyone. Those grants survive user removal in some cases — the third party caches the refresh token.

1. Sign in to the leaving person's Workspace account (you have super-admin; use `Admin Console → Users → leaving person → Sign in as user`).
2. Go to <https://myaccount.google.com/permissions>.
3. Screenshot the list of third-party apps with access. Note anything QIC-related (Heroku, Vercel, Sentry, Cloudflare, GitHub, etc.).
4. **Revoke every app on the list.** The leaving person should not retain any OAuth tokens issued in QIC's name.
5. Sign back out of "sign in as user" mode.

---

## 4. Forward role inboxes (Groups membership)

Per `ops/google-entity-accounts.md` §2, customer-facing addresses are Groups. Removing the leaving person from Workspace does NOT automatically remove them from Groups they're a member of (especially if they joined as an external owner at any point).

For each of `ops@`, `support@`, `billing@`, `privacy@`:

1. **Admin Console → Directory → Groups → {group} → Members**.
2. If the leaving person is listed, **Remove member**.
3. Confirm at least one other member remains. If not, add `marcello@qictrader.com` (or whoever inherits the role) before completing the removal.
4. If the leaving person was the **Owner** of any group, promote another member to Owner first, then remove.

For `noreply@qictrader.com` — this is a user account, not a group. The app password issued to the transactional sender (SendGrid/Postmark) does **not** depend on the leaving person. Confirm by checking that no Workspace user logged in to `noreply@` recently (`Admin Console → Reports → User reports → Login activity` filtered by `noreply@`).

---

## 5. Service account audit

People sometimes create service accounts in GCP / Workspace to glue products together, then forget to document them. If the leaving person did this, the service account survives them — but the **billing** for any GCP usage by that service account still bills the entity, and there's no human to ask "what is this for?".

1. <https://console.cloud.google.com/iam-admin/serviceaccounts> for each GCP project (none as of QIC-228 — `<TBD: re-run this step the moment any GCP project is created>`).
2. List all service accounts. For each, check **Created by** in the audit log:
   - Workspace `admin@` or `marcello@` → keep, document in a future `ops/service-accounts.md` if not already.
   - Leaving person → assess: is the service account still in use (check IAM bindings, recent activity)?
     - In use → transfer "owned by" knowledge: reassign monitoring to a remaining team member, document purpose in this repo, rotate the JSON key (revoke old, generate new, update wherever it's consumed).
     - Not in use → delete after a 14-day cooldown (in case something pings it during off-hours).
3. <https://admin.google.com> → **Security → API controls → Domain-wide delegation**: list every entry. For any entry granted by the leaving person, follow the same in-use / not-in-use decision tree.

---

## 6. Workspace account — suspend, transfer data, then delete

LAST step. Only after §1–§5 are complete.

### 6.1 Suspend immediately (do not wait)

`Admin Console → Directory → Users → leaving person → More options → Suspend user`. This kills their active sessions and prevents new sign-ins. Mail still flows to the mailbox; nothing is deleted yet.

### 6.2 Set up email forwarding (optional but usually right)

For 30–90 days after departure, forward inbound mail to whoever inherits the role:

`User → User information → Account → Forward mail to → enter forwarding address`. Or set a Gmail filter under their account (you'll need to "Sign in as user" first).

Auto-reply: "I no longer work at QIC Trader. Please email `support@qictrader.com` for assistance."

### 6.3 Transfer file ownership

`User → More options → Transfer data → Drive and Docs → select new owner` → run transfer. This moves every Drive file owned by the leaving person to the new owner. Without this step, when the user is deleted (step 6.5), every file they owned is also deleted, including files shared with the team.

### 6.4 Export mailbox if required (POPIA / legal hold)

If the person was involved in any regulator-facing correspondence, or there's a possibility of legal hold, export their mailbox before deletion:

1. <https://takeout.google.com> while signed in as them (use "Sign in as user"), or
2. `Admin Console → Security → Data export` (org-level export, exports the whole tenant — heavy hammer, use only if needed).

Store the export in encrypted offsite storage with a retention period defined in writing.

### 6.5 Delete the user

After 30+ days of suspension (gives time to catch missed handoffs):

`User → More options → Delete user`. The mailbox is destroyed. The recovery email/phone for the account is unlinked. The Workspace seat is freed.

If the user was a super-admin: **demote them before deleting**. Workspace refuses to delete the last super-admin and refuses to delete super-admins via the bulk-delete path. Demote first, then delete.

### 6.6 Update `ops/credentials-google.md`

- §Ownership matrix: change every cell for the leaving person to "None"
- §Access change log: append entry for each product, format `YYYY-MM-DD | person | product | <prev role> → None | offboarding | actioned by <your name>`

---

## 7. Verification

After completing all steps, do a final sweep:

1. <https://myaccount.google.com> as `admin@qictrader.com` → **Security → Your devices** — confirm no devices listed under the leaving person's name.
2. `Admin Console → Reports → Audit and investigation → Login audit log` filtered by the leaving person's email — confirm no successful logins after the suspension timestamp. Failed logins after suspension are normal and indicate someone (probably the leaving person) trying to access the account; not a concern unless they're frequent.
3. Re-open `ops/credentials-google.md` §Ownership matrix and read every row top-to-bottom. Every cell that previously said the leaving person now reads "None" or shows the new owner.
4. Run one production-impact smoke check per product the leaving person was operationally responsible for (e.g. Ads — confirm campaigns still serve; GA4 — confirm data still flows; YouTube — confirm scheduled uploads still publish).

If anything looks off after 24 hours, the offboarding wasn't clean — re-walk the relevant section.

---

## 8. Edge cases

### The leaving person owned the credit card

If the corporate card was issued in the leaving person's name (which §3 of `google-entity-accounts.md` forbids — but historical accidents happen), Google will keep charging the card until the next billing cycle but the card will fail when the issuing bank revokes it on departure. Result: every Google product simultaneously goes into suspension for non-payment, including Workspace itself.

Pre-empt this: BEFORE the leaving person's last day, update §3 of the entity runbook — replace the payment method on the Google Billing account with a new card issued in the entity's name. Verify the next charge succeeds against the new card before the old one is revoked.

### The leaving person was the sole Workspace super-admin

Should never happen because §1.1 of the entity runbook mandates `admin@qictrader.com` as a second super-admin. But if it ever does:

1. Use Workspace account recovery: <https://admin.google.com/accountrecovery>. Requires DNS verification of the domain — same TXT record method as Search Console. Google verifies you control the domain and reinstates super-admin access via a recovery process that can take 1–7 business days.
2. After recovery, immediately create `admin@qictrader.com` and document the lesson in the access change log.

### Compromise vs. departure

If the trigger is suspected compromise (not departure), the order is the same but the speed is different:

1. **Suspend Workspace account immediately** — even before §1 ownership transfer. The attacker keeps any session tokens they already exfiltrated, but no new sign-ins succeed.
2. Run §3 (revoke external integrations) IMMEDIATELY. OAuth tokens are the attacker's persistence mechanism.
3. Then back to §1 (ownership transfer) — slower but no longer time-critical.
4. Rotate every API key, OAuth client secret, and service-account JSON key that the compromised account had access to. Anything they could read once, assume they exfiltrated.

---

## Cross-references

- `ops/google-entity-accounts.md` — setup runbook (this is the reverse).
- `ops/credentials-google.md` — ownership matrix to update.
- `ops/heroku-apps.md` §Decommission procedure — same rotate-at-issuer principle.
- Trello ticket: QIC-228.
