#!/usr/bin/env bash
# commit-all.sh — Commit to frontend and backend submodules, then update the monorepo root.
#
# Usage:
#   ./commit-all.sh "your commit message"
#   ./commit-all.sh "your commit message" --push
#   ./commit-all.sh "your commit message" --deploy        # push + deploy to STAGING (default)
#   ./commit-all.sh "your commit message" --prod          # push + deploy to PRODUCTION (restricted)
#   ./commit-all.sh "your commit message" --buildpack     # push + deploy via git push (slow, staging only)
#   ./commit-all.sh "your commit message" --frontend-only
#   ./commit-all.sh "your commit message" --backend-only
#   ./commit-all.sh "your commit message" --dry-run
#
# Deploy details:
#   Staging frontend:    vercel --yes (preview deploy — no --prod)
#   Production frontend: vercel --prod --yes (only with --prod flag)
#   Staging backend:     cross-compile + Heroku Slug API → qictrader-backend-staging
#   Production backend:  cross-compile + Heroku Slug API → qictrader-backend-rs
#   Buildpack backend:   git push heroku-staging main (staging) or heroku main (prod)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
FRONTEND="$ROOT/frontend"
BACKEND="$ROOT/qictrader-backend-rs"
HEROKU_APP_STAGING="qictrader-backend-staging"
HEROKU_APP_PROD="qictrader-backend-rs"

MESSAGE=""
PUSH=false
DEPLOY=false
USE_BUILDPACK=false
DRY_RUN=false
FRONTEND_ONLY=false
BACKEND_ONLY=false
DEPLOY_ENV="staging"  # default to staging

for arg in "$@"; do
  case "$arg" in
    --push)          PUSH=true ;;
    --deploy)        DEPLOY=true; PUSH=true ;;
    --fast-deploy)   DEPLOY=true; PUSH=true ;;  # alias, same as --deploy now
    --prod)          DEPLOY=true; PUSH=true; DEPLOY_ENV="production" ;;
    --buildpack)     DEPLOY=true; PUSH=true; USE_BUILDPACK=true ;;
    --dry-run)       DRY_RUN=true ;;
    --frontend-only) FRONTEND_ONLY=true ;;
    --backend-only)  BACKEND_ONLY=true ;;
    *)               MESSAGE="$arg" ;;
  esac
done

if [[ "$DEPLOY_ENV" == "production" ]]; then
  HEROKU_APP="$HEROKU_APP_PROD"
else
  HEROKU_APP="$HEROKU_APP_STAGING"
fi

if [[ -z "$MESSAGE" ]]; then
  echo "Usage: ./commit-all.sh \"commit message\" [--push] [--deploy] [--prod] [--buildpack] [--frontend-only] [--backend-only] [--dry-run]"
  exit 1
fi

commit_submodule() {
  local dir="$1"
  local name="$2"

  if [[ ! -d "$dir" ]]; then
    echo "[$name] Directory not found, skipping: $dir"
    return 1
  fi

  local status
  status=$(git -C "$dir" status --porcelain 2>/dev/null)

  if [[ -z "$status" ]]; then
    echo "[$name] Nothing to commit, skipping."
    return 1
  fi

  if $DRY_RUN; then
    echo "[dry-run] [$name] git add -A"
    echo "[dry-run] [$name] git commit -m \"$MESSAGE\""
  else
    git -C "$dir" add -A
    git -C "$dir" commit -m "$MESSAGE"
    echo "[$name] ✅ Committed"
  fi

  if $PUSH; then
    if $DRY_RUN; then
      echo "[dry-run] [$name] git push"
    else
      git -C "$dir" push
      echo "[$name] ✅ Pushed"
    fi
  fi

  return 0
}

# --- Production deploy guards (added 2026-05-11) -----------------------------
# See scripts/fast-deploy-backend.sh for the parallel backend guard. The
# frontend guard is lighter because Vercel keeps every preview and `vercel
# rollback` is one click — but it still requires the explicit confirmation
# env var so autonomous tooling cannot ship to www.qictrader.com by mistake.
assert_frontend_prod_allowed() {
  if [[ "${CONFIRM_PROD_DEPLOY:-}" != "yes" ]]; then
    cat >&2 <<'EOF'
ERROR: frontend production deploy refused — CONFIRM_PROD_DEPLOY env var not set.

Why: this guard exists so autonomous tools (Cursor agents, cron jobs,
     misclicked shell history) cannot ship to www.qictrader.com without a
     second deliberate signal.

Fix: re-run the same command with the env var inline, e.g.

  CONFIRM_PROD_DEPLOY=yes ./commit-all.sh "..." --prod

If you didn't mean to deploy to production, drop the --prod flag — the
default --deploy targets staging only.
EOF
    return 1
  fi

  if [[ -t 0 ]]; then
    cat <<EOF
================================================================
  FRONTEND PRODUCTION DEPLOY — www.qictrader.com
================================================================
EOF
    read -p "Type 'PRODUCTION' to confirm (anything else aborts): " confirm
    if [[ "$confirm" != "PRODUCTION" ]]; then
      echo "Aborted." >&2
      return 1
    fi
  fi
  return 0
}

deploy_frontend() {
  if [[ "$DEPLOY_ENV" == "production" ]]; then
    if ! $DRY_RUN; then
      assert_frontend_prod_allowed || exit 2
    fi
    local VERCEL_FLAGS="--prod --yes --scope qictraders-projects"
    local LABEL="production"
  else
    local VERCEL_FLAGS="--yes --scope qictraders-projects"
    local LABEL="staging (preview)"
  fi

  if $DRY_RUN; then
    echo "[dry-run] [frontend] vercel $VERCEL_FLAGS"
    if [[ "$DEPLOY_ENV" != "production" ]]; then
      echo "[dry-run] [frontend] alias deployment to staging.qictrader.com"
    fi
  else
    echo "[frontend] Deploying to Vercel [$LABEL]..."
    local VERCEL_OUT
    VERCEL_OUT=$(cd "$FRONTEND" && vercel $VERCEL_FLAGS 2>&1)
    echo "$VERCEL_OUT" | sed 's/^/[frontend] /'
    echo "[frontend] ✅ Vercel deploy complete [$LABEL]"

    if [[ "$DEPLOY_ENV" != "production" ]]; then
      local DEPLOY_URL
      DEPLOY_URL=$(echo "$VERCEL_OUT" | grep -Eo 'https://frontend-[a-z0-9-]+\.vercel\.app' | tail -1)
      if [[ -z "$DEPLOY_URL" ]]; then
        echo "[frontend] ⚠️  Could not parse deployment URL — skipping staging.qictrader.com alias" >&2
        return 0
      fi
      local VERCEL_TOKEN
      VERCEL_TOKEN=$(python3 -c "import json; print(json.load(open('$HOME/.local/share/com.vercel.cli/auth.json'))['token'])" 2>/dev/null)
      if [[ -z "$VERCEL_TOKEN" ]]; then
        echo "[frontend] ⚠️  No Vercel auth token found — skipping staging.qictrader.com alias" >&2
        return 0
      fi
      echo "[frontend] Aliasing $DEPLOY_URL → staging.qictrader.com..."
      local ALIAS_RESP
      ALIAS_RESP=$(curl -s -X POST \
        "https://api.vercel.com/v2/deployments/${DEPLOY_URL#https://}/aliases?teamId=team_oT8YESScxj17i1VS7OXQlLz7" \
        -H "Authorization: Bearer $VERCEL_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"alias":"staging.qictrader.com"}')
      if echo "$ALIAS_RESP" | grep -q '"alias":"staging.qictrader.com"'; then
        echo "[frontend] ✅ staging.qictrader.com now serves this deployment"
      else
        echo "[frontend] ⚠️  Alias API call failed: $ALIAS_RESP" >&2
      fi
    fi
  fi
}

deploy_backend() {
  local HEROKU_REMOTE
  if [[ "$DEPLOY_ENV" == "production" ]]; then
    HEROKU_REMOTE="heroku"
  else
    HEROKU_REMOTE="heroku-staging"
  fi

  if $USE_BUILDPACK; then
    if $DRY_RUN; then
      echo "[dry-run] [backend] git push $HEROKU_REMOTE main (app: $HEROKU_APP) [$DEPLOY_ENV]"
    else
      echo "[backend] Deploying to Heroku via buildpack ($HEROKU_APP) [$DEPLOY_ENV]..."
      git -C "$BACKEND" push "$HEROKU_REMOTE" main
      echo "[backend] ✅ Heroku buildpack deploy pushed [$DEPLOY_ENV]"
    fi
  else
    local FAST_DEPLOY_FLAG
    if [[ "$DEPLOY_ENV" == "production" ]]; then
      FAST_DEPLOY_FLAG="--prod"
    else
      FAST_DEPLOY_FLAG="--staging"
    fi

    if $DRY_RUN; then
      echo "[dry-run] [backend] fast deploy: cross-compile + Slug API (app: $HEROKU_APP) [$DEPLOY_ENV]"
    else
      echo "[backend] Fast deploying to Heroku ($HEROKU_APP) [$DEPLOY_ENV]..."
      "$ROOT/scripts/fast-deploy-backend.sh" "$FAST_DEPLOY_FLAG"
      echo "[backend] ✅ Fast deploy complete [$DEPLOY_ENV]"
    fi
  fi
}

echo ""
echo "Commit message: \"$MESSAGE\""
$PUSH    && echo "Mode: commit + push"
$DEPLOY  && echo "Mode: commit + push + deploy [$DEPLOY_ENV]"
$DRY_RUN && echo "Mode: DRY RUN — no changes will be made"
echo ""

FRONTEND_CHANGED=false
BACKEND_CHANGED=false

if ! $BACKEND_ONLY; then
  commit_submodule "$FRONTEND" "frontend" && FRONTEND_CHANGED=true || true
fi

if ! $FRONTEND_ONLY; then
  commit_submodule "$BACKEND" "backend" && BACKEND_CHANGED=true || true
fi

if $FRONTEND_CHANGED || $BACKEND_CHANGED; then
  ROOT_STATUS=$(git -C "$ROOT" status --porcelain 2>/dev/null)
  if [[ -n "$ROOT_STATUS" ]]; then
    if $DRY_RUN; then
      echo "[dry-run] [monorepo] git add frontend qictrader-backend-rs"
      echo "[dry-run] [monorepo] git commit -m \"$MESSAGE\""
    else
      git -C "$ROOT" add frontend qictrader-backend-rs
      git -C "$ROOT" commit -m "$MESSAGE"
      echo "[monorepo] ✅ Submodule pointers updated"
    fi

    if $PUSH; then
      if $DRY_RUN; then
        echo "[dry-run] [monorepo] git push"
      else
        git -C "$ROOT" push
        echo "[monorepo] ✅ Pushed"
      fi
    fi
  else
    echo "[monorepo] No pointer changes to commit."
  fi
else
  echo "Nothing was committed."
fi

if $DEPLOY; then
  echo ""
  if ! $BACKEND_ONLY; then
    deploy_frontend || true
  fi
  if ! $FRONTEND_ONLY; then
    deploy_backend || true
  fi
fi

echo ""
echo "Done."
