#!/usr/bin/env bash
# Deploy backend to Heroku and trigger frontend deploy on Vercel.
# First-time: run ./scripts/setup-vercel-hook.sh and paste your Vercel Deploy Hook URL.
# Then run: ./scripts/deploy-both.sh
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/qictrader-backend-rs"
HOOK_FILE="$SCRIPT_DIR/.vercel-deploy-hook"

# Hook URL: from env, then first argument, then saved file
HOOK_URL="${VERCEL_DEPLOY_HOOK_URL:-$1}"
if [[ -z "$HOOK_URL" && -f "$HOOK_FILE" ]]; then
  HOOK_URL="$(cat "$HOOK_FILE" | tr -d '\n\r')"
fi

echo "==> Deploying backend to Heroku..."
if [[ ! -d "$BACKEND_DIR" ]]; then
  echo "Error: Backend not found at $BACKEND_DIR"
  exit 1
fi
cd "$BACKEND_DIR"
git push heroku main
echo "==> Backend deployed to Heroku."

if [[ -n "$HOOK_URL" ]]; then
  echo "==> Triggering Vercel frontend deploy..."
  curl -sS -X POST "$HOOK_URL" || true
  echo ""
  echo "==> Vercel deploy triggered (check Vercel dashboard for status)."
else
  echo "==> Skipping Vercel (run ./scripts/setup-vercel-hook.sh once to save your Deploy Hook URL)."
fi
echo "Done."
