# Production-Only Testing

**We test and check only against production.** No local backend/DB for validation.

## Canonical production credentials

| Purpose | Value |
|--------|--------|
| **Heroku app** | `qictrader-backend-rs` |
| **Backend API (prod)** | `https://qictrader-backend-rs-13eab0516d9a.herokuapp.com` |
| **API base URL** | `https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1` |
| **WebSocket (prod)** | `wss://qictrader-backend-rs-13eab0516d9a.herokuapp.com` |
| **Database** | From Heroku: `heroku config:get DATABASE_URL -a qictrader-backend-rs` |
| **Frontend (prod)** | Vercel; ensure `NEXT_PUBLIC_API_URL` and `NEXT_PUBLIC_WS_URL` point to the backend URLs above (or to `https://api.qictrader.com` if that custom domain is set). |

## Quick prod setup

```bash
# From repo root – use prod DB and prod API for scripts
export HEROKU_APP=qictrader-backend-rs
export DATABASE_URL=$(heroku config:get DATABASE_URL -a $HEROKU_APP)
export API_BASE_URL=https://qictrader-backend-rs-13eab0516d9a.herokuapp.com
```

Or run the helper script (from repo root):

```bash
source ./scripts/prod-env.sh
```

## Seed alpha account (prod)

```bash
cd qictrader-backend-rs
export DATABASE_URL=$(heroku config:get DATABASE_URL -a qictrader-backend-rs)
export API_BASE_URL=https://qictrader-backend-rs-13eab0516d9a.herokuapp.com
./scripts/seed_alpha_account.sh
```

Or with the prod helper:

```bash
source scripts/prod-env.sh
cd qictrader-backend-rs && ./scripts/seed_alpha_account.sh
```

**Prod test user:** `alpha@qictest.com` / `TestPass123!`

## Query prod DB

```bash
heroku pg:psql -a qictrader-backend-rs -c "SELECT ..."
```

## E2E against prod

Set env so E2E hits prod backend and frontend:

```bash
export BACKEND_URL=https://qictrader-backend-rs-13eab0516d9a.herokuapp.com
export FRONTEND_URL=https://qictrader.com
# or your Vercel prod URL
```

Then run E2E; helpers use `BACKEND_URL` / `FRONTEND_URL` when set.

## Vercel production env

In Vercel Dashboard (production), set:

- `NEXT_PUBLIC_API_URL` = `https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1`
- `NEXT_PUBLIC_WS_URL` = `wss://qictrader-backend-rs-13eab0516d9a.herokuapp.com`

If you use a custom domain (e.g. `https://api.qictrader.com`) for the same Heroku app, use that instead and keep API and WS consistent.
