#!/usr/bin/env bash
# commit-all.sh — Commit across the frontend + backend submodules, then update
# the monorepo root pointer. COMMIT/PUSH ONLY — this script never deploys.
#
# Deploys are automatic and PR-driven (see DEPLOYMENT.md):
#   open a PR  -> Heroku review app is created (seeded from staging)
#   merge to main -> staging auto-deploys
#   promote in the Heroku pipeline -> production
#
# Usage:
#   ./commit-all.sh "your commit message"                 # commit only
#   ./commit-all.sh "your commit message" --push          # commit + push current branch
#   ./commit-all.sh "your commit message" --frontend-only # frontend submodule only
#   ./commit-all.sh "your commit message" --backend-only  # backend submodule only
#   ./commit-all.sh "your commit message" --dry-run       # preview, make no changes
#
# Reminder: work on a feature branch and open a PR. Never push to `main`
# directly, and never force-push.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
FRONTEND="$ROOT/frontend"
BACKEND="$ROOT/qictrader-backend-rs"

MESSAGE=""
PUSH=false
DRY_RUN=false
FRONTEND_ONLY=false
BACKEND_ONLY=false

for arg in "$@"; do
  case "$arg" in
    --push)          PUSH=true ;;
    --dry-run)       DRY_RUN=true ;;
    --frontend-only) FRONTEND_ONLY=true ;;
    --backend-only)  BACKEND_ONLY=true ;;
    --deploy|--prod|--fast-deploy|--buildpack)
      echo "ERROR: '$arg' is no longer supported. Deploys are automatic and PR-driven." >&2
      echo "       See DEPLOYMENT.md: open a PR -> merge to main (staging) -> promote (prod)." >&2
      exit 1
      ;;
    *)               MESSAGE="$arg" ;;
  esac
done

if [[ -z "$MESSAGE" ]]; then
  echo "Usage: ./commit-all.sh \"commit message\" [--push] [--frontend-only] [--backend-only] [--dry-run]"
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

echo ""
echo "Commit message: \"$MESSAGE\""
$PUSH    && echo "Mode: commit + push"
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

echo ""
echo "Done. Deploys are automatic — open a PR; see DEPLOYMENT.md."
