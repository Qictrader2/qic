# Scripts

**We test only against production.** For prod credentials, DB, and API URLs see **`docs/PROD_TESTING.md`**. Quick prod env: `source ./scripts/prod-env.sh`.

## Deployment

We host everything on **Heroku** and deploys are **PR-driven and automatic** —
there are no deploy scripts here, and no Vercel. See the repo-root
**[`DEPLOYMENT.md`](../DEPLOYMENT.md)** for the full process:

- Open a PR → a Heroku review app is created (seeded from staging).
- Merge to `main` → staging auto-deploys.
- Promote the verified staging release in the Heroku pipeline → production.

`./commit-all.sh` (repo root) is a **commit/push helper only** — it commits across
both submodules and updates the root pointer. It never deploys.
