# QIC Trader — Deployment Runbook

**This is the single source of truth for how we ship code. Read it fully before
deploying anything. If another doc contradicts this file, this file wins.**

We host **everything on Heroku**. There is **no Vercel**. You **never** run a
deploy command by hand — deploys happen automatically through Heroku pipelines,
driven by GitHub pull requests.

---

## 1. The mental model (read this first)

```
            you open a PR                 you merge the PR              you click "Promote"
  branch ───────────────▶ Review App ───────────────────▶ Staging ───────────────────────▶ Production
  (your work)            (temporary,                      (auto-deploy                     (manual, gated,
                          seeded test data)                on merge to main)                 one click)
```

Three environments, in order:

1. **Review App** — a throwaway, full-stack copy created automatically when you
   open a PR. It has its own URL and its own database seeded from a copy of
   staging. Use it to demo and review your change in isolation.
2. **Staging** — `staging.qictrader.com`. Updated **automatically** every time a
   PR is merged to `main`. This is the shared pre-production environment.
3. **Production** — `www.qictrader.com`. Updated **only** when a human clicks
   **"Promote to production"** in the Heroku pipeline. Never automatic.

**Golden rules**

- All changes go through a **PR**. Never push directly to `main`. Never force-push.
- `git pull --rebase origin main` **before** you start and **before** you push.
- Production is **promoted from a verified staging release** — never deployed
  directly, never from your laptop.

---

## 2. What's where (apps, repos, URLs)

The app is split into two GitHub repos (git submodules of this monorepo). Each
repo has its **own** Heroku pipeline with a staging app and a production app.

| Piece | GitHub repo | Heroku pipeline | Staging app → URL | Production app → URL |
|---|---|---|---|---|
| **Frontend** (Next.js) | `Qictrader2/Frontend` | `qictrader-frontend` | `qictrader-frontend-staging` → `staging.qictrader.com` | `qictrader-frontend` → `www.qictrader.com` |
| **Backend** (Rust) | `Qictrader2/qictrader-backend-rs` | `qictrader-backend` | `qictrader-backend-staging` | `qictrader-backend-rs` |

- All apps live under the **`qictrader` Heroku team**.
- Apex `qictrader.com` permanently redirects (301) to `https://www.qictrader.com`.
- The frontend talks to the backend through a **same-origin proxy** (`/api/v1`),
  configured by the `BACKEND_URL` and `NEXT_PUBLIC_*` Heroku Config Vars on each
  frontend app. WebSockets connect directly to the backend's `NEXT_PUBLIC_WS_URL`.

**A change that only touches the frontend → one PR in `Qictrader2/Frontend`.**
**A change that only touches the backend → one PR in `Qictrader2/qictrader-backend-rs`.**
A change that touches both → one PR per repo (they deploy independently).

---

## 3. The end-to-end process (do exactly this)

### Step 1 — Start from the latest `main`

From inside the submodule you're changing (`frontend/` or `qictrader-backend-rs/`):

```bash
git checkout main
git pull --rebase origin main
```

### Step 2 — Create a branch and do your work

```bash
git checkout -b your-name/short-description     # e.g. jp/fix-withdrawal-banner
# ...make your changes...
```

### Step 3 — Verify locally before you push

- **Frontend:** `bun run build` must succeed.
- **Backend:** `cargo build` must succeed and `cargo clippy` must be clean.
- Run the relevant tests (see each repo's test commands).

### Step 4 — Commit and push the branch

```bash
git add -A
git commit -m "TICKET-ID: short description of what and why"
git pull --rebase origin main      # pull again in case others merged
git push -u origin your-name/short-description
```

> Working across both submodules at once? From the monorepo root you can use
> `./commit-all.sh "TICKET-ID: message" --push` to commit + push both submodule
> branches and bump the root pointer. It **only commits/pushes** — it never deploys.

### Step 5 — Open the PR

```bash
gh pr create --fill        # or open it in the GitHub UI
```

Opening the PR **automatically creates a Heroku Review App**:

- It gets its own URL (posted by Heroku as a check / on the PR, and visible in
  the Heroku pipeline under "Review Apps").
- Its database is a **copy of the staging database**, plus seeded test accounts
  and balances (see §5). The frontend review app points at the **staging backend**.
- It is destroyed automatically when the PR is closed/merged (or after the
  configured idle timeout).

### Step 6 — Review on the Review App

Demo the change on the review-app URL. Get approval. Iterate by pushing more
commits to the same branch — the review app redeploys automatically.

### Step 7 — Merge to `main` → auto-deploy to staging

When the PR is approved and green, **merge it**. Merging to `main` triggers an
**automatic deploy to the staging app**. Verify your change on:

- Frontend: `https://staging.qictrader.com`
- Backend: `https://qictrader-backend-staging-2290a9290b6b.herokuapp.com/health`

### Step 8 — Promote to production (manual, gated)

Once staging is verified and you have sign-off:

- **Heroku Dashboard:** open the pipeline (`qictrader-frontend` or
  `qictrader-backend`) → find the current **staging** release → click
  **"Promote to production"**.
- **Or CLI:**

```bash
# Frontend
heroku pipelines:promote -a qictrader-frontend-staging

# Backend
heroku pipelines:promote -a qictrader-backend-staging
```

Promotion ships the **exact slug that was verified on staging** — no rebuild, no
laptop involved. Production = `www.qictrader.com`.

> **Production is gated.** Only promote with explicit sign-off (JP for prod).
> This is real-money financial software.

### Step 9 — Verify production

```bash
# Frontend
curl -sS -o /dev/null -w '%{http_code}\n' https://www.qictrader.com/      # expect 200

# Backend
curl -sS -o /dev/null -w '%{http_code}\n' https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/health   # expect 200
heroku releases -a qictrader-backend-rs -n 1
heroku logs -a qictrader-backend-rs -n 100 | grep -iE 'migrat|panic|Error:|crashed'
```

---

## 4. Configuration & secrets (Heroku Config Vars)

There is **no Vercel dashboard**. All environment variables live as **Heroku
Config Vars** on each app.

```bash
# View (single key — preferred, avoids dumping every secret to your terminal)
heroku config:get NAME -a qictrader-frontend

# Set
heroku config:set NAME=value -a qictrader-frontend          # production frontend
heroku config:set NAME=value -a qictrader-frontend-staging  # staging frontend
```

- When you add a new env var, set it on **both** the staging and production app
  for that piece, and document it (frontend: `.env.example`; backend: its config docs).
- **Never** print full config dumps into a saved terminal, screenshot, or chat.
  See the credential-hygiene rule. Reference variable **names**, not values.

---

## 5. Review-app seed data

Review apps must be usable immediately, so they are seeded automatically via each
repo's `app.json` `postdeploy` hook:

- **Backend** review app: copies the **staging database** into the review app's
  fresh Postgres (`pg_dump … | psql …`), then runs the `seed-review-app` binary
  to create fixed test accounts (SuperAdmin / Moderator / User) and fund test
  balances. The seed binary **refuses to run if `APP_ENV=production`**.
- **Frontend** review app: inherits its Config Vars from the staging frontend app
  and points at the **staging backend**, so it behaves like staging out of the box.

The staging database URL is provided to backend review apps via the
`STAGING_DATABASE_URL` config var (set on the backend staging app and inherited
by review apps through `app.json`).

---

## 6. Database migrations (backend) — boot-critical

SQLx migrations run **on backend boot**. A bad migration crash-loops the dyno and
takes the service down. Hard rules learned from the 2026-06-15 outage:

1. **`ledger_entries` is append-only** (enforced by a DB trigger). A migration
   that `UPDATE`s historical ledger rows is rejected and crash-loops boot. Never
   mutate history in a migration — post a compensating entry or filter in the reader.
2. **Fix-forward, don't roll back.** Once a migration is recorded in
   `_sqlx_migrations`, an older slug that lacks that file refuses to boot. If a
   migration is bad, make it a safe no-op in a new migration and redeploy.
3. **Avoid timestamp collisions.** Before creating a migration: `git pull`, check
   `ls migrations/ | sort | tail`, and pick a timestamp strictly greater than the max.

Because production is **promoted** (same slug as staging), any migration has
already run on staging before it reaches production — so verify staging boots
cleanly before promoting.

---

## 7. Rollback

- **Promote a known-good older release:** in the pipeline, promote the previous
  good slug, or:

```bash
heroku releases -a <app>                 # find the good version, e.g. v123
heroku rollback v123 -a <app>            # roll that app back
```

- **Frontend** rollback is safe and instant.
- **Backend**: rolling back is fine for code, but **not** for an already-applied
  migration (see §6 — fix-forward instead).

---

## 8. What NOT to do

- ❌ No Vercel. Do not install the Vercel CLI, create `vercel.json`, or look for a
  Vercel dashboard. It's gone.
- ❌ Do not push directly to `main`. Open a PR.
- ❌ Do not force-push `main`.
- ❌ Do not deploy production from your laptop. Production is **promoted** from a
  verified staging release, with sign-off.
- ❌ Do not run local cross-compiles or the Heroku Slug API for the backend (the
  old `fast-deploy-backend.sh` path caused a prod outage and was removed).
- ❌ Do not deploy without `git pull --rebase origin main` first.
- ❌ Do not dump full `heroku config` into a captured terminal.

---

## 9. Quick reference

```bash
# See the pipelines and their stages
heroku pipelines:info qictrader-frontend
heroku pipelines:info qictrader-backend

# Promote verified staging -> production
heroku pipelines:promote -a qictrader-frontend-staging
heroku pipelines:promote -a qictrader-backend-staging

# Health checks
curl -sS -o /dev/null -w '%{http_code}\n' https://www.qictrader.com/
curl -sS -o /dev/null -w '%{http_code}\n' https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/health

# Logs
heroku logs -a qictrader-frontend -n 100
heroku logs -a qictrader-backend-rs -n 100
```
