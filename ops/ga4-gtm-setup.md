# GA4 + GTM Setup — Manual Runbook (QIC-231)

Single source of truth for the **Google Analytics 4** and **Google Tag Manager** properties owned by **QIC Trade Systems Limited**. The frontend ships GTM via `@next/third-parties/google` with Consent Mode v2 defaulting to **all-denied**; everything below is the manual work in Google's UI that has to happen before the tag fires anything useful.

Forward reference: parent runbook is [`google-entity-accounts.md`](./google-entity-accounts.md) (QIC-228) — that doc owns the Workspace tenant and entity-level Google identity. This doc only owns analytics-stack identifiers.

> **Rule:** every analytics property gets owner, purpose, and an identifier that lives in `ops/credentials-google.md`. Anything not listed here is either unclaimed or undocumented — both are bugs.

---

## 0. Pre-flight

- Workspace super-admin `marcello@qictrader.com` is signed in.
- 2FA enrolled per `google-entity-accounts.md` §1.2.
- `admin@qictrader.com` super-admin already exists as break-glass.
- Domain `qictrader.com` is verified in Search Console (gives GA4 the demographic enrichment hook later).

You should already have:
- The QIC Trade Systems Limited billing profile attached (for paid Ads later, no charge for GA4/GTM).
- A "QIC Trader" Google Marketing Platform organisation. If not, you'll create it on the way through GA4 setup.

---

## 1. Create the GA4 property

Goal: a single GA4 property covering `qictrader.com` (production) and Vercel staging previews. Separate streams per environment, single property so reports aren't fragmented.

### 1.1 Property

1. Sign in to <https://analytics.google.com> as `marcello@qictrader.com`.
2. **Admin → Create → Property**.
3. **Property details**:
   - Property name: `QicTrader — Web`
   - Reporting time zone: `(GMT+02:00) Johannesburg`
   - Currency: `South African Rand (ZAR)`
4. **Business details**:
   - Industry category: `Finance`
   - Business size: `Small`
5. **Business objectives**: pick `Generate leads` + `Examine user behavior`. Don't pick "Drive online sales" — that switches GA4's default reports into ecommerce mode, which doesn't match our P2P trade model.
6. **Data collection → Web**:
   - Stream name: `QicTrader Production`
   - Website URL: `https://www.qictrader.com`
   - Enhanced measurement: **On** (we'll prune events we don't want later).
7. **Stream details → Configure tag settings**:
   - **Define internal traffic**: add rule `internal_traffic` → IP equals office/home IPs (whatever Marcello uses for QA). Mark these as "internal" so they're filterable in DebugView.
   - **List unwanted referrals**: add `firebaseapp.com`, `localhost`, `vercel.app` so payment-redirect referrers don't pollute attribution.
   - **Adjust session timeout**: leave default 30 min.

### 1.2 Capture the Measurement ID

After the stream is created, copy the **Measurement ID** (`G-XXXXXXXXXX`). Record it in `ops/credentials-google.md` under `Google Analytics 4 / Production`. We don't put this in any env var — GTM owns the tag firing, the frontend only knows the GTM container ID.

### 1.3 Staging stream

Repeat §1.1 step 6 for a second stream on the same property:

- Stream name: `QicTrader Staging`
- Website URL: `https://*.vercel.app`
- Enhanced measurement: **On**

Copy that Measurement ID too — record it as `Google Analytics 4 / Staging` in `credentials-google.md`. Two streams means we can split staging vs prod traffic with the `stream_id` dimension in reports.

### 1.4 Privacy / retention settings

Still in **Admin → Property → Data settings**:

1. **Data collection** → toggle **Google signals data collection** to **Off**. Google signals adds advertising features that pull from logged-in Google users; we keep it off until Ads is plumbed in (separate ticket).
2. **Data retention** → set **Event data retention** to **14 months**. Default is 2 months which is too short for cohort/retention reports. POPIA caps it at "no longer than necessary for the purpose" — 14 months is GA4's max for free tier and is the GDPR-compatible ceiling, so we set it now and don't revisit.
3. **Data retention** → tick **Reset user data on new activity** to extend retention on returning users.
4. **Data filters** → confirm `Internal Traffic` filter exists and is set to **Active** (not Testing).
5. **IP anonymization**: GA4 anonymizes IPs by default at ingest — confirm no override is in place. The "Universal Analytics" anonymization toggle no longer exists; GA4 does this implicitly.

---

## 2. Create the GTM container

Goal: a Web container `GTM-XXXXXXX` that the Next.js frontend loads via `<GoogleTagManager />`. Every GA4 hit, future Ads pixel, and future conversion goes through here.

1. Sign in to <https://tagmanager.google.com> as `marcello@qictrader.com`.
2. **Create Account**:
   - Account name: `QIC Trade Systems Limited`
   - Country: `South Africa`
   - Don't tick "Share data anonymously with Google" — we don't owe them benchmarking data.
3. **Container setup**:
   - Container name: `qictrader.com`
   - Target platform: `Web`
4. Accept the GTM Terms of Service (EU data processing terms appear because we're going to advertise into EU later — accept).
5. After creation, copy the **Container ID** (`GTM-XXXXXXX`). Record it in `ops/credentials-google.md` under `Google Tag Manager / Web Container`.

### 2.1 Container settings

- **Admin → Container Settings → Container Notes**: paste the link to this runbook (`ops/ga4-gtm-setup.md`) and the parent Workspace doc.
- **Admin → User Management → Add user**: invite `admin@qictrader.com` as **Administrator** (break-glass account). No other humans yet.

---

## 3. Inside GTM — variables, tags, triggers

The frontend code (`src/lib/analytics/gtm.ts`) pushes events to `window.dataLayer`. GTM has to pick those up, map their fields onto GA4 parameters, and forward to the GA4 property. **Names matter** — keep them identical to what the frontend pushes.

### 3.1 Built-in variables

**Variables → Configure**, enable these:

- `Page URL`, `Page Path`, `Page Hostname`, `Referrer`
- `Click URL`, `Click Element`, `Click Text` (for future button tracking)
- `Container ID`, `Container Version`, `Random Number` (debug helpers)

### 3.2 User-defined dataLayer variables

Frontend pushes payloads like `{ event: 'first_deposit', currency: 'ZAR', amount: 250 }`. Each non-`event` field needs a Data Layer Variable so GTM can read it.

For each variable below: **Variables → User-Defined Variables → New → Data Layer Variable**.

| Variable Name (GTM) | Data Layer Variable Name | Default Value | Notes |
|---------------------|--------------------------|---------------|-------|
| `dlv.currency` | `currency` | `(empty)` | ZAR / BTC / USDT / ETH / SOL — frontend already constrains via TS union |
| `dlv.amount` | `amount` | `0` | Numeric, in `currency` units |
| `dlv.method` | `method` | `(empty)` | Sign-up method: `email` / `google` / `wallet` |
| `dlv.user_id_hash` | `user_id_hash` | `(empty)` | Pseudonymous user identifier; **never** push raw email or wallet address into the data layer |
| `dlv.offer_type` | `offer_type` | `(empty)` | `buy` / `sell` — set on `first_offer_created` |
| `dlv.cryptocurrency` | `cryptocurrency` | `(empty)` | ZAR / BTC / USDT / ETH / SOL — set on `first_offer_created` |
| `dlv.fiat_currency` | `fiat_currency` | `(empty)` | Three-letter fiat code (e.g. `ZAR`) — set on `first_offer_created` |

Version 2: leave Data Layer Variable type as **Version 2** (default). Version 1 only reads the most recent push — we want the merged state.

### 3.3 GA4 Configuration tag

This is the single tag that initialises GA4 inside GTM. Every event tag references it.

1. **Tags → New → Tag Configuration → Google Analytics: GA4 Configuration**.
2. **Measurement ID**:
   - Don't hardcode. Use a Constant variable: `Variables → User-Defined → New → Constant → const.ga4_measurement_id` with value `G-XXXXXXXXXX` (production stream).
   - For staging we'll add a `Lookup Table` variable later if we use one GTM container across both environments. For now: production-only.
3. **Fields to set**:
   - `send_page_view`: `true`
   - `cookie_flags`: `SameSite=None;Secure` (required for cross-site contexts behind Cloudflare)
4. **Consent Settings** (the v2 bit):
   - Click **Advanced Settings → Consent Settings → Require additional consent for tag to fire**.
   - Tick `analytics_storage`. Do NOT tick the ad signals here — GA4 itself only needs analytics consent. Ads tags (added in a later ticket) will require the ad signals separately.
5. **Triggering**: `Initialization — All Pages`.

Name the tag: `GA4 — Configuration`.

### 3.4 GA4 Event tags

Three conversion events, one tag each.

For all three: **Tags → New → Tag Configuration → Google Analytics: GA4 Event**. Set:

- **Configuration Tag**: the `GA4 — Configuration` tag from §3.3
- **Event Name**: the exact GA4 reserved name listed below
- **Event Parameters**: from the dataLayer variables in §3.2
- **Consent Settings**: require `analytics_storage` (same as the config tag).
- **Triggering**: a Custom Event trigger named `ce.<event_name>` matching `Event equals <event_name>`.

| GA4 Event Name | Custom Event Trigger | Parameters (from dataLayer) | Mark as Conversion? |
|----------------|----------------------|-----------------------------|---------------------|
| `sign_up` | `ce.sign_up` | `method = {{dlv.method}}` | Yes |
| `onboarding_complete` | `ce.onboarding_complete` | `user_id_hash = {{dlv.user_id_hash}}` | Yes |
| `first_deposit` | `ce.first_deposit` | `currency = {{dlv.currency}}`, `amount = {{dlv.amount}}` | Yes |
| `kyc_verification_completed` | `ce.kyc_verification_completed` | (none) | Yes |
| `first_offer_created` | `ce.first_offer_created` | `offer_type = {{dlv.offer_type}}`, `cryptocurrency = {{dlv.cryptocurrency}}`, `fiat_currency = {{dlv.fiat_currency}}` | Yes |

`sign_up` is GA4-reserved and shows up in default reports — don't rename it. The custom events use snake_case to match GA4 conventions.

### 3.4.1 Frontend emission status

The frontend event schema already defines all three conversion events in `frontend/src/lib/analytics/events.ts`, and `frontend/src/lib/analytics/gtm.ts` only pushes them when `NEXT_PUBLIC_GTM_ID` is configured.

Current wiring:

| Event | Frontend status | Notes |
|-------|-----------------|-------|
| `sign_up` | Live in code | Fired after successful email signup and Google OAuth signup. It is not fired for login. |
| `onboarding_complete` | Not emitted yet | Needs a settled product definition for "onboarding complete" plus a pseudonymous `user_id_hash` source. Do not send raw email, wallet address, or raw user ID. |
| `first_deposit` | Not emitted yet | Needs a backend/API contract that says this accepted deposit is the user's first deposit. Do not infer this from the generic deposit mutation, or repeat deposits will be counted as first deposits. |
| `kyc_verification_completed` | Live in code | Fired by `IdentityVerification.tsx` on the transition into the verified terminal state. Stateless — repeated verifications (re-KYC) will re-fire; consider this when interpreting funnel reports. |
| `first_offer_created` | Live in code | Backend snapshots offer count before insert and returns `isFirstOffer: true` on the create response only when the user has zero prior offers. Frontend forwards that to GA4. No client-side state. |

### 3.5 Mark as Conversions in GA4

GTM only fires the events. To make them count as conversions in GA4 reports:

1. In GA4 → **Admin → Property → Events**.
2. After at least one of each event has fired (publish GTM first, see §4, then trigger from staging), each event will appear in the list.
3. Toggle **Mark as conversion** for `sign_up`, `onboarding_complete`, `first_deposit`, `kyc_verification_completed`, `first_offer_created`.

Note: GA4 renamed "Conversions" to "Key events" in 2024 — same toggle, different label depending on when you read this.

### 3.6 Future-proofing: Ads tag stub (do NOT publish yet)

Create a placeholder tag so the consent requirement is documented:

1. **Tags → New → Tag Configuration → Google Ads Conversion Tracking**.
2. Leave Conversion ID/Label blank (will fill when Ads account is provisioned in a later ticket).
3. **Consent Settings → Require additional consent**: tick **all three** — `ad_storage`, `ad_user_data`, `ad_personalization`.
4. **Triggering**: pick a placeholder trigger (e.g. a custom event `ce.first_deposit_ads_only`).
5. **Pause** the tag (Tag Settings → three-dot menu → Pause). Do not publish.

This makes the consent contract explicit for the next agent: GA4 needs `analytics_storage`; Ads needs all three ad signals.

---

## 4. Publish + verify with DebugView

GTM changes don't fire until they're published.

### 4.1 First publish

1. In GTM, top-right: **Submit**.
2. **Publish Configuration**:
   - Version Name: `v1 — GA4 + 3 conversion events`
   - Version Description: `Initial setup, consent-mode v2 enforced, see ops/ga4-gtm-setup.md`
3. **Publish**.

The container is now live.

### 4.2 Wire the GTM ID into Vercel

The frontend reads `process.env.NEXT_PUBLIC_GTM_ID`. Add it to Vercel:

```bash
cd frontend/
# production
echo "GTM-XXXXXXX" | npx vercel env add NEXT_PUBLIC_GTM_ID production
# preview (staging)
echo "GTM-XXXXXXX" | npx vercel env add NEXT_PUBLIC_GTM_ID preview
```

(There is no separate `staging` env in Vercel — `preview` covers all non-prod deploys.)

Then redeploy to pick up the new var:

```bash
./commit-all.sh "QIC-231: wire GTM ID env var" --deploy --frontend-only
```

If `NEXT_PUBLIC_GTM_ID` is unset the frontend silently skips GTM injection — no broken pages. So you can verify in two steps: set it on preview first, smoke-test, then push to production.

### 4.3 DebugView verification

Two parallel verifications: tag-side and consent-side.

**Tag side (GA4 DebugView):**

1. Install [Google Tag Assistant](https://chromewebstore.google.com/detail/tag-assistant-companion/jmekfmbnaedfebfnmakmokmlfpblbfdm) in Chrome.
2. Visit the deployed Vercel preview URL.
3. Open Tag Assistant → **Add domain** → preview URL → **Connect**.
4. In another tab open GA4 → **Admin → DebugView**.
5. Accept all cookies in QicTrader's banner.
6. Trigger `sign_up` (or just navigate around — `page_view` should fire immediately).
7. DebugView should show the event within ~10 seconds, with the parameters you defined.

If nothing appears in DebugView:
- Tag Assistant probably never connected — check for the green tick.
- Check `window.dataLayer` in the browser console — should contain a `gtm.js` push, a `consent default`, a `consent update`, and your event push.

**Consent side (Network tab):**

1. Browser DevTools → Network tab, filter `collect`.
2. Load the page **without** clicking Accept on the banner.
3. You should see **no** `collect?v=2&...` requests to `google-analytics.com` (analytics_storage = denied blocks them).
4. Click Accept All on the banner.
5. Refresh.
6. Now `collect` requests appear, and `dataLayer` has a `consent update` entry with all four signals `granted`.
7. Reload again — refresh, click Reject All. `collect` requests should stop within the page.

If `collect` fires before Accept is clicked, the consent default isn't loading early enough. The frontend ships `<ConsentDefault>` with `strategy="beforeInteractive"` so it must execute before GTM bootstraps — check the rendered HTML `<head>` order: consent default script must appear **before** the GTM `<script>` tag.

### 4.4 Tag Assistant browser check

In Tag Assistant (real-time mode, not the Connect flow above):

- `GA4 — Configuration` tag should show as "Fired"
- Each event tag fires only when the matching dataLayer push happens
- Consent state should display as `analytics_storage: granted/denied` per user choice
- No "Tag failed" rows

If a tag shows as "Not fired" but you triggered it: open the tag's row, check the "Blocking trigger" column. Most likely cause: consent setting isn't satisfied.

---

## 5. Ongoing ops

### 5.1 Who owns what

| Asset | Owner role | Where the ID lives |
|-------|-----------|--------------------|
| GA4 Property `QicTrader — Web` | `marcello@qictrader.com` (admin), `admin@qictrader.com` (admin, break-glass) | `ops/credentials-google.md` |
| GA4 stream `Production` (`G-XXXXX`) | same | same |
| GA4 stream `Staging` (`G-YYYYY`) | same | same |
| GTM container `qictrader.com` (`GTM-XXXXX`) | same | same |
| Vercel env var `NEXT_PUBLIC_GTM_ID` | DevOps (set via `vercel env add`) | Vercel dashboard, Production + Preview |

### 5.2 Adding new events

Same playbook every time:

1. Add the event name to `src/lib/analytics/events.ts` as a const + add its payload to the discriminated union.
2. Push it via `pushEvent({...})` from `src/lib/analytics/gtm.ts`.
3. In GTM: add a Data Layer Variable per new payload field, create a Custom Event trigger, create a GA4 Event tag wired to the `GA4 — Configuration` tag.
4. Set Consent Settings appropriately (`analytics_storage` for measurement events; ad signals for any tag that talks to Ads).
5. Submit + publish.
6. Verify in DebugView.
7. Mark as Conversion in GA4 if it's a key business event.

### 5.3 Changing the GTM container ID

If we ever need to rotate the container (e.g. account migration):

1. Create new container per §2.
2. Re-export the workspace from old container (Admin → Export Container) and import into new (Admin → Import Container, merge mode = Overwrite).
3. Update `NEXT_PUBLIC_GTM_ID` in Vercel for both Production and Preview.
4. Redeploy.
5. Verify both environments fire to the new container.
6. Leave the old container paused (don't delete) for 90 days, then archive.

### 5.4 POPIA / GDPR posture

- Default consent is **all-denied**. Consent Mode v2 sends "denied" pings (no PII) so GA4 still gets modelled conversions but no cookies are written until the user opts in.
- We surface the cookie banner on first visit and a "Cookie preferences" link in the footer for re-opt.
- Retention is 14 months — the GA4 maximum. After 14 months event data is purged.
- Google signals is **off** until Ads is provisioned (separate ticket).
- POPIA is the applicable regime today; the all-denied default + explicit-consent design also satisfies GDPR, so we can advertise into EU later without rework.

---

*Linked docs: [`google-entity-accounts.md`](./google-entity-accounts.md) (entity-level Google), `credentials-google.md` (all identifiers).*
