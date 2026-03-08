# Scripts

**We test only against production.** For prod credentials, DB, and API URLs see **`docs/PROD_TESTING.md`**. Quick prod env: `source ./scripts/prod-env.sh`.

## Vercel: Deploy Hook only

**We always deploy the frontend via the Deploy Hook.** Builds are triggered by the hook—either automatically on every push to `main` (GitHub Action) or manually with the scripts below.

### One-time setup

1. **Create a Vercel Deploy Hook**
   - Open [Vercel Dashboard](https://vercel.com/dashboard) and select your **Frontend** project.
   - Go to **Settings** → **Git** → **Deploy Hooks**.
   - Click **Create Hook** (e.g. name: `Deploy from script`, branch: `main`).
   - Copy the generated URL.

2. **Disable Vercel’s built-in deploy on push** (so only the hook triggers builds):
   - In the same project: **Settings** → **Git** → **Ignored Build Step**.
   - Set the command to: `exit 0`
   - Result: the GitHub Action (or manual script) calls the hook after each push; Vercel does not start a build from the push itself.

3. **Add the hook URL as a GitHub secret** (so the Action can trigger on push):
   - In the **Frontend** repo on GitHub: **Settings** → **Secrets and variables** → **Actions**.
   - **New repository secret**: name `VERCEL_DEPLOY_HOOK_URL`, value = the full deploy hook URL from step 1.

4. **Save the hook URL locally** (optional, for manual runs from the workspace root):

   ```bash
   chmod +x scripts/setup-vercel-hook.sh
   ./scripts/setup-vercel-hook.sh
   ```
   When prompted, paste the Deploy Hook URL. It is saved to `scripts/.vercel-deploy-hook` (gitignored). This is only needed for running `./scripts/trigger-vercel-deploy.sh` or `./scripts/deploy-both.sh` locally; the GitHub Action uses the `VERCEL_DEPLOY_HOOK_URL` secret.

### Deploy frontend only

From the **workspace root** (Qictrader):

```bash
./scripts/trigger-vercel-deploy.sh
```

### Deploy backend + frontend

From the **workspace root** (Qictrader):

```bash
./scripts/deploy-both.sh
```

This will:

1. Deploy the backend: `git push heroku main` from `qictrader-backend-rs`.
2. Trigger a Vercel production deploy for the frontend using your saved hook.

Override the hook URL for one run if needed:

```bash
VERCEL_DEPLOY_HOOK_URL="https://api.vercel.com/..." ./scripts/deploy-both.sh
# or
./scripts/deploy-both.sh "https://api.vercel.com/..."
```
