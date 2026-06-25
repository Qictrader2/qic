# Meta Pixel Setup - QIC Trader LP

Single source of truth for the Meta Pixel browser-side setup used by the
QicTrader acquisition landing page at `https://www.qictrader.com/lp`.

This is the v1 browser Pixel setup. Conversions API is intentionally a follow-up
after Pixel events are verified in Meta Events Manager.

> Rule: record public identifiers in `ops/credentials-meta.md`. Never commit
> access tokens, API tokens, app secrets, or downloaded diagnostics containing
> customer data.

## 1. Create or Verify the Meta Dataset

1. Sign in to Meta Business Suite / Events Manager with the QicTrader business
   owner account.
2. Open **Events Manager -> Data sources**.
3. Create or select the QicTrader website dataset / Pixel.
4. Set the website domain to `qictrader.com`.
5. Copy the public Pixel/Dataset ID into `ops/credentials-meta.md`.
6. Do not create or store a Conversions API access token for v1.

## 2. Browser Pixel Install, Events, and Consent

The QicTrader Meta Pixel is installed directly in the frontend because the MSI
browser session could not access the live GTM container. GTM remains present for
GA4, but Meta Pixel v1 does not depend on GTM publishing access.

Pixel ID: `1598536338721831`

Deployment note: the direct Pixel install is present in Vercel deployment
`dpl_J8PFVYBXq3rGixLTj4ZdAB61b9CJ`
(`https://frontend-b2a8ct90r-qictraders-projects.vercel.app`). Updating
`www.qictrader.com` to that deployment is currently blocked because the Vercel
token can deploy the `frontend` project but cannot manage the
`www.qictrader.com` domain alias.

### 2.1 Consent Gate

The frontend only loads `https://connect.facebook.net/en_US/fbevents.js` when
all of these consent switches are enabled:

- `ads`
- `ad_user_data`
- `ad_personalization`

Before consent, no Meta Pixel script or `PageView` request should fire. If a
visitor later grants all ad consent, the frontend initializes the Pixel and
fires `PageView`.

### 2.2 Frontend Event Mapping

| Frontend Event             | Meta Event                    | Trigger Source                      | Primary Fields                            |
| -------------------------- | ----------------------------- | ----------------------------------- | ----------------------------------------- |
| Page load after ad consent | `PageView`                    | `MetaPixel` component               | Pixel ID `1598536338721831`               |
| `lp_demo_cta_click`        | `LpDemoCtaClick` custom event | `/lp` demo CTA clicks               | `eventID`, `page_path`, `location`        |
| `calendly_event_scheduled` | `Schedule` standard event     | Calendly completed booking listener | `eventID`, `page_path`, `calendly_widget` |

## 3. Optional GTM Tags, Triggers, and Consent

The frontend already loads GTM globally with Consent Mode v2. Meta tags must be
configured in the existing `GTM-5X3M5QCS` container only if the direct frontend
Pixel install is removed or replaced later.

### 3.1 Built-in Variables

Enable these if they are not already enabled:

| Variable            | Purpose                                   |
| ------------------- | ----------------------------------------- |
| `Page URL`          | Filter/report `/lp` page visits           |
| `Page Path`         | Build `/lp` custom conversion or audience |
| `Page Hostname`     | Confirm production vs preview             |
| `Container ID`      | Debugging                                 |
| `Container Version` | Debugging                                 |

### 3.2 Data Layer Variables

Create these user-defined Data Layer Variables:

| GTM Variable Name     | Data Layer Variable Name | Default Value | Notes                                     |
| --------------------- | ------------------------ | ------------- | ----------------------------------------- |
| `dlv.event_id`        | `event_id`               | `(empty)`     | Used now by Pixel and later by CAPI dedup |
| `dlv.page_path`       | `page_path`              | `(empty)`     | `/lp` for landing-page events             |
| `dlv.location`        | `location`               | `(empty)`     | CTA location, e.g. `hero`                 |
| `dlv.calendly_widget` | `calendly_widget`        | `(empty)`     | `inline` for the embedded Calendly widget |

### 3.3 Triggers

Create these triggers:

| Trigger Name                  | Type         | Condition                               |
| ----------------------------- | ------------ | --------------------------------------- |
| `ce.lp_demo_cta_click`        | Custom Event | Event equals `lp_demo_cta_click`        |
| `ce.calendly_event_scheduled` | Custom Event | Event equals `calendly_event_scheduled` |

### 3.4 Meta Pixel Base Tag

Create a Custom HTML tag named `Meta Pixel - Base`.

Trigger: `Initialization - All Pages`

Consent settings: require all of:

- `ad_storage`
- `ad_user_data`
- `ad_personalization`

Use the base code from Meta Events Manager and replace only the Pixel ID. The
base tag should initialize Pixel and fire `PageView`. `/lp` visits can then be
reported through Meta by filtering `Page URL` or `Page Path` to `/lp`.

### 3.5 Meta Landing CTA Event

Create a Custom HTML tag named `Meta Pixel - LP Demo CTA Click`.

Trigger: `ce.lp_demo_cta_click`

Consent settings: require all ad consent signals listed in section 3.4.

Event call:

```html
<script>
  fbq(
    "trackCustom",
    "LpDemoCtaClick",
    {
      page_path: "{{dlv.page_path}}",
      location: "{{dlv.location}}",
    },
    {
      eventID: "{{dlv.event_id}}",
    },
  );
</script>
```

This is a secondary intent signal only. Do not optimize the campaign against it
unless booking volume is too low to train delivery.

### 3.6 Meta Booking Conversion

Create a Custom HTML tag named `Meta Pixel - Schedule`.

Trigger: `ce.calendly_event_scheduled`

Consent settings: require all ad consent signals listed in section 3.4.

Event call:

```html
<script>
  fbq(
    "track",
    "Schedule",
    {
      page_path: "{{dlv.page_path}}",
      calendly_widget: "{{dlv.calendly_widget}}",
    },
    {
      eventID: "{{dlv.event_id}}",
    },
  );
</script>
```

`Schedule` is the primary v1 conversion for the `/lp` campaign.

## 4. Meta Events Manager Configuration

1. In Events Manager, open the QicTrader data source.
2. Create a custom conversion for `/lp` page visits if needed:
   - Event: `PageView`
   - Rule: URL contains `/lp`
3. Use `Schedule` as the primary conversion event for demo-booking campaigns.
4. Keep `LpDemoCtaClick` available for retargeting/audience diagnostics.
5. Do not enable advanced matching with raw email, phone, wallet addresses, or
   KYC data in v1.

## 5. Verification

Use the browser network panel and Test Events in Meta Events Manager. GTM
Preview mode is optional for GA4 checks, but it is no longer required for Meta
Pixel v1.

### Before Consent

1. Clear `qic_consent` cookie/localStorage.
2. Load `https://www.qictrader.com/lp`.
3. Confirm no `connect.facebook.net` or Meta Pixel requests fire.

### Reject All

1. Click **Reject all** in the consent banner.
2. Reload `/lp`.
3. Confirm no Meta Pixel requests fire.

### Accept All

1. Click **Accept all** in the consent banner.
2. Reload `/lp`.
3. Confirm `PageView` appears in Meta Test Events.
4. Click **Book a Live Demo**.
5. Confirm `LpDemoCtaClick` appears with `eventID`.
6. Complete a Calendly booking.
7. Confirm `Schedule` appears with `eventID`.
8. Confirm the browser console has no CSP violations.

Record the verification date and result in `ops/credentials-meta.md`.

## 6. Later CAPI Follow-up

The frontend now sends `event_id` with the booking event. The server-side CAPI
follow-up must use the same value as Meta `event_id` for deduplication.

Implementation options for v2:

- Use Calendly webhook data to derive the same deterministic booking event id.
- Or post the browser `event_id` to a QicTrader backend endpoint when the
  Calendly booking event fires, then send CAPI from the backend.

Until CAPI ships, do not create or store Meta access tokens in the repo.
