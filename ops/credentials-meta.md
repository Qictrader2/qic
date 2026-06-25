# Meta Credentials and Asset IDs

This file records non-secret Meta business identifiers for QicTrader marketing
measurement. It must never contain access tokens, app secrets, API tokens,
downloaded event payloads, or user data.

## Meta Pixel / Dataset - `/lp` Demo Campaign

| Field                    | Value                                                                                                                                                        |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Business portfolio       | `qictrader`                                                                                                                                                  |
| Data source name         | `QicTrader Dataset`                                                                                                                                          |
| Public Pixel/Dataset ID  | `1598536338721831`                                                                                                                                           |
| Website domain           | `qictrader.com`                                                                                                                                              |
| Primary landing page     | `https://www.qictrader.com/lp`                                                                                                                               |
| Primary conversion event | `Schedule`                                                                                                                                                   |
| Secondary event          | `LpDemoCtaClick`                                                                                                                                             |
| GTM container            | `GTM-5X3M5QCS`                                                                                                                                               |
| Browser Pixel status     | `Direct frontend install deployed to Vercel deployment dpl_J8PFVYBXq3rGixLTj4ZdAB61b9CJ; www.qictrader.com alias update blocked by Vercel domain permission` |
| CAPI status              | `Not implemented - v2 follow-up`                                                                                                                             |
| Test Events verification | `Pending www.qictrader.com alias update and Meta Test Events check`                                                                                          |

## GTM Tag Inventory

| Tag                              | Trigger                                   | Consent Required                            | Status                                                                      |
| -------------------------------- | ----------------------------------------- | ------------------------------------------- | --------------------------------------------------------------------------- |
| `Meta Pixel - Base`              | Frontend `MetaPixel` component            | `ads`, `ad_user_data`, `ad_personalization` | `Deployed to dpl_J8PFVYBXq3rGixLTj4ZdAB61b9CJ; custom-domain alias blocked` |
| `Meta Pixel - LP Demo CTA Click` | `lp_demo_cta_click` frontend event        | `ads`, `ad_user_data`, `ad_personalization` | `Deployed to dpl_J8PFVYBXq3rGixLTj4ZdAB61b9CJ; custom-domain alias blocked` |
| `Meta Pixel - Schedule`          | `calendly_event_scheduled` frontend event | `ads`, `ad_user_data`, `ad_personalization` | `Deployed to dpl_J8PFVYBXq3rGixLTj4ZdAB61b9CJ; custom-domain alias blocked` |

## Data Layer Contract

| Frontend Event             | GTM Trigger                   | Meta Event                    | Primary Fields                             |
| -------------------------- | ----------------------------- | ----------------------------- | ------------------------------------------ |
| `lp_demo_cta_click`        | `ce.lp_demo_cta_click`        | `LpDemoCtaClick` custom event | `event_id`, `page_path`, `location`        |
| `calendly_event_scheduled` | `ce.calendly_event_scheduled` | `Schedule` standard event     | `event_id`, `page_path`, `calendly_widget` |

## Ownership

| Role                | Owner   |
| ------------------- | ------- |
| Meta Business admin | `<TBD>` |
| GTM publisher       | `<TBD>` |
| Verification owner  | `<TBD>` |

See `ops/meta-pixel-setup.md` for setup and verification steps.
