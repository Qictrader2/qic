#!/usr/bin/env bash
# commit-all.sh — Commit to frontend and backend submodules, then update the monorepo root.
#
# Usage:
#   ./commit-all.sh "your commit message"
#   ./commit-all.sh "your commit message" --push
#   ./commit-all.sh "your commit message" --deploy        # push + deploy both
#   ./commit-all.sh "your commit message" --frontend-only
#   ./commit-all.sh "your commit message" --backend-only
#   ./commit-all.sh "your commit message" --dry-run
#
# Deploy details:
#   Frontend: Vercel deploy hook (URL read from $ROOT/.vercel-deploy-hook or $VERCEL_DEPLOY_HOOK_URL)
#   Backend:  git push heroku main (Heroku app: qictrader-backend-rs)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
FRONTEND="$ROOT/frontend"
BACKEND="$ROOT/qictrader-backend-rs"
HEROKU_APP="qictrader-backend-rs"
HOOK_FILE="$ROOT/.vercel-deploy-hook"

MESSAGE=""
PUSH=false
DEPLOY=false
DRY_RUN=false
FRONTEND_ONLY=false
BACKEND_ONLY=false

for arg in "$@"; do
  case "$arg" in
    --push)          PUSH=true ;;
    --deploy)        DEPLOY=true; PUSH=true ;;
    --dry-run)       DRY_RUN=true ;;
    --frontend-only) FRONTEND_ONLY=true ;;
    --backend-only)  BACKEND_ONLY=true ;;
    *)               MESSAGE="$arg" ;;
  esac
done

if [[ -z "$MESSAGE" ]]; then
  echo "Usage: ./commit-all.sh \"commit message\" [--push] [--deploy] [--frontend-only] [--backend-only] [--dry-run]"
  exit 1
fi

run() {
  local label="$1"; shift
  if $DRY_RUN; then
    echo "[dry-run] [$label] $*"
  else
    echo "[$label] running: $*"
    git -C "$1" "${@:2}" 2>&1 | sed "s/^/[$label] /"
  fi
}

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
  local hook_url="${VERCEL_DEPLOY_HOOK_URL:-}"
  if [[ -z "$hook_url" && -f "$HOOK_FILE" ]]; then
    hook_url=$(cat "$HOOK_FILE" | tr -d '[:space:]')
  fi
  if [[ -z "$hook_url" ]]; then
    echo "[frontend] ⚠️  No Vercel deploy hook found. Set VERCEL_DEPLOY_HOOK_URL or create $HOOK_FILE"
    return 1
  fi
  if $DRY_RUN; then
    echo "[dry-run] [frontend] curl -X POST <vercel-deploy-hook>"
  else
    echo "[frontend] Triggering Vercel deploy..."
    curl -s -X POST "$hook_url" | grep -o '"job":{[^}]*}' || true
    echo ""
    echo "[frontend] ✅ Vercel deploy triggered"
  fi
}

deploy_backend() {
  if $DRY_RUN; then
    echo "[dry-run] [backend] git push heroku main (app: $HEROKU_APP)"
  else
    echo "[backend] Deploying to Heroku ($HEROKU_APP)..."
    git -C "$BACKEND" push heroku main
    echo "[backend] ✅ Heroku deploy pushed"
  fi
}

echo ""
echo "Commit message: \"$MESSAGE\""
$PUSH    && echo "Mode: commit + push"
$DEPLOY  && echo "Mode: commit + push + deploy"
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
