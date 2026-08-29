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

## Ticket time tracker (`ticket-time.ts`)

Read-only, Clockify-free time-per-ticket report built from two signals we
already generate: Trello column-move timestamps (wall-clock dwell per workflow
column) and `TICKET-ID:`-prefixed git commits (clustered into work sessions for
an *estimated active effort* number). No new services, no new secrets.

```bash
source ~/.qictrader-secrets/trello.env          # TRELLO_API_KEY / TRELLO_API_TOKEN
bun scripts/ticket-time.ts --since 2026-08-01    # per-card dwell, cycle, est. active, commits
bun scripts/ticket-time.ts --since 2026-08-01 --until 2026-08-31 --gap-hours 2
```

The git session number is labelled **estimated active** on purpose — agent-heavy
work compresses commit timestamps, so it undercounts thinking/review time; it is
never authoritative logged time. Commits whose ticket id matches no card land in
an explicit **unattributed** bucket rather than being dropped.

Tests (pure column-dwell maths + session clustering, exact durations):

```bash
bun test scripts/ticket-time.test.ts
```
