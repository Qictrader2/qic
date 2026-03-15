#!/usr/bin/env bash
# Deploy backend to Heroku and trigger frontend deploy on Vercel.
# Deploys backend to Heroku and frontend to Vercel (CLI).
# Requires: vercel CLI logged in with qictraders-projects team access.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/qictrader-backend-rs"
FRONTEND_DIR="$ROOT_DIR/Frontend"
if [[ ! -d "$FRONTEND_DIR" ]]; then
  FRONTEND_DIR="$ROOT_DIR/frontend"
fi

echo "==> Deploying backend to Heroku..."
if [[ ! -d "$BACKEND_DIR" ]]; then
  echo "Error: Backend not found at $BACKEND_DIR"
  exit 1
fi
cd "$BACKEND_DIR"
git push heroku main
echo "==> Backend deployed to Heroku."

echo "==> Deploying frontend to Vercel via CLI..."
if [[ ! -d "$FRONTEND_DIR" ]]; then
  echo "Error: Frontend not found at $FRONTEND_DIR"
  exit 1
fi
cd "$FRONTEND_DIR"
vercel --prod --yes --scope qictraders-projects 2>&1
echo "==> Vercel deploy complete."
echo "Done."
