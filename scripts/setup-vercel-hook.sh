#!/usr/bin/env bash
# One-time setup: save your Vercel Deploy Hook URL so deploy-both.sh can use it.
# Run: ./scripts/setup-vercel-hook.sh
# Or:  ./scripts/setup-vercel-hook.sh "https://api.vercel.com/v1/integrations/deploy/..."
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK_FILE="$SCRIPT_DIR/.vercel-deploy-hook"
URL="$1"

if [[ -z "$URL" ]]; then
  echo "Vercel Deploy Hook setup"
  echo ""
  echo "1. Open: https://vercel.com/dashboard → your Frontend project"
  echo "2. Settings → Git → Deploy Hooks"
  echo "3. Create a hook (e.g. name: 'Deploy from Heroku', branch: main)"
  echo "4. Copy the hook URL"
  echo ""
  read -p "Paste the Deploy Hook URL here: " URL
  if [[ -z "$URL" ]]; then
    echo "No URL entered. Exiting."
    exit 1
  fi
fi

echo "$URL" > "$HOOK_FILE"
echo "Saved to $HOOK_FILE"
echo "You can now run: ./scripts/deploy-both.sh"
echo "(Deploy hook file is gitignored and will not be committed.)"
