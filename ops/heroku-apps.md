# Heroku App Inventory (PENTEST-015)

Single source of truth for every Heroku app the QIC Trader org / `jp.vanzyl@icloud.com` account owns. Pen-test Addendum H-6 found `qictrader-api` running 5 weeks stale with live secrets in `heroku config` and no documented owner. To stop that recurring, every app is listed here with **owner**, **purpose**, **status**, and a link to the source repo (or "ad-hoc" if there isn't one).

> **Rule:** any Heroku app under the org that is **not** in this file gets `apps:destroy`'d in the next sprint, after a final secrets-rotation pass at the issuer.

## QIC Trader apps

| App | Status | Purpose | Owner | Repo / source |
|-----|--------|---------|-------|---------------|
| `qictrader-backend-rs` | **Production** | Rust/Axum API serving production traffic | JP | `Qictrader2/qictrader-backend-rs` |
| `qictrader-backend-staging` | **Staging** | Pre-prod validation; PENTEST tickets ship here first | JP | `Qictrader2/qictrader-backend-rs` (`heroku-staging` remote) |
| `qictrader-api` | **DECOMMISSION (PENTEST-015)** | Old Rails sister API. Last release v12, 2026-03-08. Holds `RAILS_MASTER_KEY`, `DATABASE_URL`, `ANTHROPIC_API_KEY`, `TRON_API_KEY` in config. Codebase audit confirms zero references from `qictrader-backend-rs` or `frontend`. Pending: rotate keys at issuers, then `heroku apps:destroy -a qictrader-api`. | JP | none — orphan |
| `qictrader-dev-dashboard` | **Active (internal)** | Internal dev metrics dashboard (last release v10, 2026-03-27) | JP | `qictrader-dev-deployments/` (this monorepo) |
| `qictrader-nextjs` | **DECOMMISSION** | Stale Next.js demo from 2026-02-17 (v7). Predates current Vercel-hosted frontend. Pending: confirm no traffic for 7 days, then destroy. | JP | none |

## Operator apps (out of QIC scope — listed for completeness)

| App | Owner |
|-----|-------|
| `devhouse-site` | JP — personal devhouse landing |
| `pen-testing` | JP — pen-test platform engagement scratch |
| `y-qa-platform`, `y-qa-platform-staging` | JP — separate Y product |
| `yellow-mantis-api`, `yellow-mantis-app` | JP — separate Yellow Mantis product |

## Inventory check (runbook)

Run quarterly (or before/after any pen-test):

```bash
# 1. Diff the live Heroku org against this file.
heroku apps --json | jq -r '.[].name' | sort > /tmp/heroku-live.txt
grep -oE '`qictrader[a-z0-9-]+`' ops/heroku-apps.md | tr -d '`' | sort -u > /tmp/heroku-documented.txt
diff /tmp/heroku-live.txt /tmp/heroku-documented.txt
```

- Any line only in `/tmp/heroku-live.txt` is an undocumented app — must be added here or destroyed.
- Any line only in `/tmp/heroku-documented.txt` is a stale entry — remove it from this file.

## Decommission procedure (for `qictrader-api`)

Before destroying any app, **rotate every secret in its config at the issuer first** — Heroku app destroy does not revoke API keys upstream.

```bash
APP=qictrader-api

# 1. Capture (do NOT print) names only — confirm what we have to rotate.
heroku config -a "$APP" --json | jq 'keys'

# 2. Rotate each key at its issuer (manual; see notes below).
#    - ANTHROPIC_API_KEY  → console.anthropic.com → keys → revoke + create new
#    - TRON_API_KEY       → trongrid.io          → API keys → revoke
#    - RAILS_MASTER_KEY   → re-encrypt config/credentials.yml.enc, commit, redeploy any consumer (none — orphan)
#    - DATABASE_URL       → heroku pg:credentials:rotate DATABASE -a "$APP"
#    - SECRET_KEY_BASE    → not needed once app is destroyed (Rails session signing only)

# 3. Confirm the OLD value is dead by hitting the issuer with it once and expecting 401/403.
#    (Manual; do not paste old key into shell history.)

# 4. Destroy the app.
heroku apps:destroy -a "$APP" --confirm "$APP"
```

After destruction, update this file: change the row's status from "DECOMMISSION" to "DESTROYED <date>" and remove from the active inventory in the next monthly cleanup.

## Cross-references

- `.cursor/rules/credential-hygiene.mdc` — what a rotation actually means and which keys are in scope.
- `QIC_TRADER_PENTEST_REPORT.md` §5 — original observation that triggered this inventory.
- Trello ticket: PENTEST-015 [HIGH].
