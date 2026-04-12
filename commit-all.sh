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

deploy_frontend() {
  if [[ "$DEPLOY_ENV" == "production" ]]; then
    local VERCEL_FLAGS="--prod --yes --scope qictraders-projects"
    local LABEL="production"
  else
    local VERCEL_FLAGS="--yes --scope qictraders-projects"
    local LABEL="staging (preview)"
  fi

  if $DRY_RUN; then
    echo "[dry-run] [frontend] vercel $VERCEL_FLAGS"
  else
    echo "[frontend] Deploying to Vercel [$LABEL]..."
    (cd "$FRONTEND" && vercel $VERCEL_FLAGS 2>&1) | sed 's/^/[frontend] /'
    echo "[frontend] ✅ Vercel deploy complete [$LABEL]"
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
