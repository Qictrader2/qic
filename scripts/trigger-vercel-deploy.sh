#!/usr/bin/env bash
# Deploy frontend to Vercel production via CLI.
# Any team member logged into vercel CLI with qictraders-projects access can deploy.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/../Frontend"

if [[ ! -d "$FRONTEND_DIR" ]]; then
  FRONTEND_DIR="$SCRIPT_DIR/../frontend"
fi

if [[ ! -d "$FRONTEND_DIR" ]]; then
  echo "Error: Frontend directory not found"
  exit 1
fi

echo "Deploying frontend to Vercel (production)..."
cd "$FRONTEND_DIR"
vercel --prod --yes --scope qictraders-projects 2>&1
echo ""
echo "✅ Vercel deploy complete."
