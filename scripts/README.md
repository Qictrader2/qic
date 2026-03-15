# Scripts

**We test only against production.** For prod credentials, DB, and API URLs see **`docs/PROD_TESTING.md`**. Quick prod env: `source ./scripts/prod-env.sh`.

## Vercel: CLI Deploy Only

**We deploy the frontend via `vercel --prod --yes --scope qictraders-projects`** from the `frontend/` directory. Git-push deploys are disabled (Vercel Ignored Build Step = `exit 0`). Any team member with access to the `qictraders-projects` Vercel team can deploy.

### Prerequisites

- Vercel CLI installed: `npm i -g vercel`
- Logged in: `vercel login` (use any account that is a member of the `qictraders-projects` team)
- Project linked: `cd Frontend && vercel link --scope qictraders-projects --project qictrader-frontend --yes`

### Deploy frontend only

From the **workspace root** (Qictrader):

```bash
./scripts/trigger-vercel-deploy.sh
```

Or directly from the frontend dir:

```bash
cd Frontend && vercel --prod --yes
```

### Deploy backend + frontend

From the **workspace root** (Qictrader):

```bash
./scripts/deploy-both.sh
```

This will:

1. Deploy the backend: `git push heroku main` from `qictrader-backend-rs`.
2. Deploy the frontend: `vercel --prod --yes` from `Frontend/`.

### Via commit-all.sh

```bash
./commit-all.sh "message" --deploy
```

This commits, pushes, and deploys both backend (Heroku) and frontend (Vercel CLI).
