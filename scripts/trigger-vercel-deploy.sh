#!/usr/bin/env bash
# Trigger Vercel frontend deploy via deploy hook and show how to verify.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK_FILE="$SCRIPT_DIR/.vercel-deploy-hook"

HOOK_URL="${VERCEL_DEPLOY_HOOK_URL:-$1}"
[[ -z "$HOOK_URL" && -f "$HOOK_FILE" ]] && HOOK_URL="$(cat "$HOOK_FILE" | tr -d '\n\r')"

if [[ -z "$HOOK_URL" ]]; then
  echo "No deploy hook. Run: ./scripts/setup-vercel-hook.sh"
  exit 1
fi

echo "Triggering Vercel deploy..."
RESP=$(curl -sS -w "\n%{http_code}" -X POST "$HOOK_URL")
HTTP_CODE=$(echo "$RESP" | tail -n1)
BODY=$(echo "$RESP" | sed '$d')

echo "Response: $BODY"
echo "HTTP status: $HTTP_CODE"

if [[ "$HTTP_CODE" == "201" ]]; then
  echo ""
  echo "Deploy accepted. To verify:"
  echo "  1. Open https://vercel.com/dashboard"
  echo "  2. Open the project this hook is for (Settings → Git → Deploy Hooks shows which project)"
  echo "  3. Go to the 'Deployments' tab — the new deployment should appear within a few seconds"
  echo "  4. If you don't see it, the hook may be for a different project; create a new hook in the correct project and run ./scripts/setup-vercel-hook.sh"
else
  echo "Unexpected status. Check the hook URL in $HOOK_FILE"
  exit 1
fi
