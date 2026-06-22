---
description: QIC go-live - STAGING deploy of frontend + backend, alias staging.qictrader.com to the exact frontend preview, and move the Trello ticket. Production is out of scope.
allowed-tools: Agent, Bash, Read, Glob, Grep, WebFetch
---

You are deploying QIC Trader to **staging**. This command first integrates the ticket's work onto `main` (rebase → ff-merge → push, submodules then superproject — a no-op if `/qic-ship` already merged it), then deploys `main`: backend to Heroku staging, a frontend Vercel preview built from `main`, aliases that exact preview to `staging.qictrader.com`, and moves the Trello ticket when it can be resolved exactly. **Staging is always a deploy of `main` — never a stray branch.** Every run **ends with the finalization step** that leaves every local `main` synced to `origin/main` and the superproject pointing at the right submodule commits.

**STAGING-STRICT - NEVER DEPLOY TO PRODUCTION FROM THIS COMMAND.**

- If `$ARGUMENTS` includes `--prod`, `production`, or asks for production, stop and say `/golive` is staging-only.
- Vercel deploys must omit `--prod`. Heroku deploys must use the staging app.
- Do not cherry-pick fixes onto stale `frontend/production` or backend `production` branches. Production releases are separate explicit releases from tested `main`.

Arguments: `$ARGUMENTS`

## Deployment Model

- The Vercel `qictraders-projects/frontend` project is CLI/API deployed and is not currently Git-linked.
- Pushing `main` to GitHub does not auto-deploy Vercel production for this project.
- `main` is the code source of truth. Staging and production are deployments of specific commits.
- `staging.qictrader.com` is an alias to one exact Vercel preview deployment. It is not a live branch pointer in the current no-Git-link setup.
- **Staging always deploys `main`.** `staging.qictrader.com` is only ever aliased to a preview built from `main`, with root submodule pointers matching each submodule's `main` HEAD.
- If the ticket's work is still on a feature branch, `/golive` **integrates it to `main` first** (rebase → ff-merge → push, submodules then superproject — the same linear flow as `/qic-ship` PHASE 4), then deploys `main`. This is what stops a deploy from stranding half a ticket off `main` (e.g. backend merged, frontend left on a branch).
- A throwaway branch preview is allowed only when the user explicitly asks for one; it is built from that branch, is NOT aliased to `staging.qictrader.com`, and is NOT a release candidate.

## Trello Credentials

Source local QIC/Codex env before any Trello API call:

```bash
set -a
source "$HOME/.config/qic/.env"
set +a
: "${TRELLO_API_KEY:?Missing TRELLO_API_KEY in $HOME/.config/qic/.env}"
: "${TRELLO_TOKEN:?Missing TRELLO_TOKEN in $HOME/.config/qic/.env}"
: "${TRELLO_DEV_COMPLETE_LIST_ID:?Missing TRELLO_DEV_COMPLETE_LIST_ID in $HOME/.config/qic/.env}"
: "${TRELLO_BOARD_ID:?Missing TRELLO_BOARD_ID in $HOME/.config/qic/.env}"
```

Never paste Trello tokens, Vercel tokens, Heroku tokens, or other secrets into this command file or into the user-facing report.

## Flow

1. Inspect status:
   ```bash
   git status --short --branch
   git -C frontend status --short --branch
   git -C qictrader-backend-rs status --short --branch
   git submodule status --recursive
   ```

2. **Ensure the work is on `main` (integrate if needed) — submodules first, superproject last.**
   Staging deploys `main`, so the ticket's commits MUST be on `main` before deploying. Pick the path that matches the current state:
   - **Already integrated** (e.g. `/qic-ship` ran): just sync. In the superproject and each submodule:
     ```bash
     git checkout main && git pull --ff-only origin main
     ```
   - **Still on a `<ticket-name>` feature branch:** integrate now with the **same linear flow as `/qic-ship` PHASE 4** — for each **touched** submodule (backend, then frontend if changed):
     ```bash
     cd qictrader-backend-rs
     git add -A && git commit -m "<ticket>: <summary>" -m "Ticket-Id: <CARD_ID>"  # only if still uncommitted
     git fetch origin && git checkout <ticket-name> && git rebase origin/main      # replant on newest main
     git checkout main && git merge --ff-only <ticket-name>                        # linear — no merge commit
     git push origin main                                                          # rejected → re-fetch, rebase, retry
     cd ..
     ```
     Then bump the superproject pointer(s) the same way (only the touched submodules):
     ```bash
     git -C qictrader-backend-rs checkout main        # and: git -C frontend checkout main, if frontend changed
     git checkout <ticket-name>
     git add qictrader-backend-rs frontend
     git commit -m "<ticket>: bump submodule pointer(s)" -m "Ticket-Id: <CARD_ID>"
     git fetch origin && git rebase origin/main
     git checkout main && git merge --ff-only <ticket-name>
     git push origin main
     ```
   - **Shared-checkout safety:** if the superproject or a submodule `main` checkout is dirty or on another session's branch, do the integrate from a clean `git worktree add <tmp> origin/main` — never reset or clobber it.

3. **Gate: verify the deploy candidate IS `main` and fully current.** This is the check that prevents shipping a stranded half-ticket — do not proceed past it on a mismatch:
   ```bash
   git rev-parse --abbrev-ref HEAD                                # superproject must be on main
   git fetch origin --recurse-submodules
   git rev-parse main origin/main                                 # root: must be equal
   git -C frontend rev-parse main origin/main                     # frontend: must be equal
   git -C qictrader-backend-rs rev-parse main origin/main         # backend:  must be equal
   git ls-tree HEAD frontend qictrader-backend-rs                 # pointers must equal each submodule's main HEAD
   git -C frontend rev-parse main; git -C qictrader-backend-rs rev-parse main
   ```
   If any pair differs, STOP — the work is not fully on `main`. Finish step 2 (integrate) before deploying. Never deploy a branch to `staging.qictrader.com`.

4. Deploy backend to Heroku staging (buildpack — Heroku compiles server-side, ~2-3 min).
   The old `scripts/fast-deploy-backend.sh` cross-compile / Slug-API path was REMOVED after
   the 2026-06-15 prod outage. Deploys now go through the Heroku buildpack via git push:
   ```bash
   cd qictrader-backend-rs
   git checkout main && git pull --ff-only origin main   # deploy MAIN — guaranteed current by steps 2-3
   git push heroku-staging main                          # buildpack compiles + releases to qictrader-backend-staging
   cd ..
   ```
   Wait for `Released vNNN` and `Verifying deploy... done`. Staging app: `qictrader-backend-staging`.
   (If the `heroku-staging` remote lacks credentials, the token-bearing `heroku` remote in the
   backend submodule also points at the staging app — `git push heroku main` works the same.)

5. Deploy frontend to Vercel preview **from `main`** (not the current checkout). Do not use `--prod`.
   ```bash
   git -C frontend checkout main && git -C frontend pull --ff-only origin main   # deploy MAIN
   cd frontend && vercel deploy --target preview --yes --scope qictraders-projects
   cd ..
   ```
   Capture the immutable preview URL from stdout. (The preview is built from `main`; this is the deployment that gets aliased to `staging.qictrader.com`.)

6. Alias the exact preview deployment to staging:
   ```bash
   DEPLOY_URL="<captured preview URL>"
   set -a
   source "$HOME/.config/qic/.env"
   set +a
   : "${VERCEL_TOKEN:?Missing VERCEL_TOKEN in $HOME/.config/qic/.env}"
   curl -s -X POST \
     "https://api.vercel.com/v2/deployments/${DEPLOY_URL#https://}/aliases?teamId=team_oT8YESScxj17i1VS7OXQlLz7" \
     -H "Authorization: Bearer $VERCEL_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"alias":"staging.qictrader.com"}'
   ```
   Verify the response contains `"alias":"staging.qictrader.com"`. If aliasing fails, report it and do not move the Trello ticket.

7. Verify staging:
   ```bash
   curl -fsS https://qictrader-backend-staging-2290a9290b6b.herokuapp.com/health
   curl -fsSI https://staging.qictrader.com
   ```

8. Resolve Trello card ID exactly, in this order:
   ```bash
   git log -20 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
   git -C qictrader-backend-rs log -5 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
   git -C frontend log -5 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
   head -1 .current-ticket 2>/dev/null
   ```
   If `$ARGUMENTS` contains a 24-character card ID, that is also valid. If no exact card ID exists, skip the move and report it.

9. Move the Trello card only after backend deploy, frontend deploy, aliasing, and health checks all succeed:
   ```bash
   set -a
   source "$HOME/.config/qic/.env"
   set +a
   curl -sS "https://api.trello.com/1/cards/${CARD_ID}?key=${TRELLO_API_KEY}&token=${TRELLO_TOKEN}&fields=name,idList"
   curl -sS -X PUT "https://api.trello.com/1/cards/${CARD_ID}?key=${TRELLO_API_KEY}&token=${TRELLO_TOKEN}&idList=${TRELLO_DEV_COMPLETE_LIST_ID}&pos=top"
   ```

10. Clean up `.current-ticket` and matching ticket plan only after deploy and ticket move both succeed.

## End-of-run finalization — MANDATORY, runs at the END of every `/golive`

After the deploy + Trello steps, always leave the working tree coherent: every local `main`
equal to its `origin/main`, and the superproject pointing at the right submodule commits.
Order is **frontend → backend → superproject** (submodules first, superproject last).

**Safety first — never clobber another session's work.** Before touching any checkout, re-read
`git -C <repo> status --short --branch`. If a checkout is dirty with changes that are NOT this
ticket's, do the integrate from a clean `git worktree add <tmp> origin/main` (or a labelled
`git stash`), never `reset --hard` / force-checkout. If you cannot finalize a repo without
clobbering foreign work, STOP for that repo and report it as "needs manual sync" with the reason.

### 1. Frontend
If `frontend` is checked out on a branch (not `main`):
```bash
cd frontend
git fetch origin
# a) ensure the branch is in sync with origin/main (replant it on newest main; resolve conflicts)
git checkout <frontend-branch> && git rebase origin/main
# b) merge the branch into local main (linear; if not ff, the rebase above wasn't clean — redo it)
git checkout main && git pull --ff-only origin main
git merge --ff-only <frontend-branch>
# c) sync local main with origin/main
git push origin main                       # rejected → re-fetch, rebase the branch, retry
cd ..
```
If `frontend` is already on `main`: `git -C frontend checkout main && git -C frontend pull --ff-only origin main`.

### 2. Backend
Ensure backend local `main` equals `origin/main` (integrate any ticket branch the same way as the frontend if one is still ahead):
```bash
cd qictrader-backend-rs
git fetch origin
git checkout main && git pull --ff-only origin main
cd ..
```

### 3. Superproject pointers
ONLY after BOTH submodule local `main`s equal their `origin/main`, point the superproject at them:
```bash
git fetch origin
git checkout main && git pull --ff-only origin main
git -C frontend checkout main
git -C qictrader-backend-rs checkout main
git add frontend qictrader-backend-rs
git commit -m "<ticket>: sync submodule pointers to main" -m "Ticket-Id: <CARD_ID>"   # only if pointers changed
git push origin main
```

### 4. Verify the finalization (report this)
```bash
for r in . frontend qictrader-backend-rs; do
  echo "$r  local=$(git -C "$r" rev-parse --short main)  origin=$(git -C "$r" rev-parse --short origin/main)"
done
git ls-tree HEAD frontend qictrader-backend-rs   # superproject pointers must equal each submodule's main HEAD
```
Every local `main` must equal its `origin/main`, and the superproject's submodule pointers must
equal the submodule `main` HEADs. Report any repo still out of sync and exactly why.

## Report

Report:

- root/frontend/backend branch names
- root/frontend/backend commit SHAs deployed
- backend staging health result
- immutable Vercel preview URL
- `staging.qictrader.com` alias result
- Trello card move result, if any
- **finalization sync table** — for root/frontend/backend: local `main` vs `origin/main` (equal or not), and superproject pointers vs submodule `main` HEADs; flag anything left "needs manual sync"

$ARGUMENTS
