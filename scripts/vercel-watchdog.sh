#!/usr/bin/env bash
# vercel-watchdog.sh — Monitors Vercel deployments every 2 minutes via REST API.
# If the most recent deploy failed, redeploys via CLI as logged-in user (jp-6647).
#
# Usage:
#   ./scripts/vercel-watchdog.sh                # foreground
#   nohup ./scripts/vercel-watchdog.sh &        # background
#
# Logs to stdout. Stop: kill $(cat /tmp/vercel-watchdog.pid)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/Frontend"
[ -d "$FRONTEND_DIR" ] || FRONTEND_DIR="$ROOT_DIR/frontend"

SCOPE="qictraders-projects"
TEAM_ID="team_oT8YESScxj17i1VS7OXQlLz7"
PROJECT_ID="prj_5ELXaaIflGEmCkpeGyQ4DHJitPnQ"
CHECK_INTERVAL=120
LAST_FIXED_ID=""

# Read Vercel auth token from CLI config
AUTH_FILE="$HOME/Library/Application Support/com.vercel.cli/auth.json"
if [ -f "$AUTH_FILE" ]; then
  VERCEL_TOKEN=$(python3 -c "import json; print(json.load(open('$AUTH_FILE'))['token'])" 2>/dev/null) || true
fi
if [ -z "${VERCEL_TOKEN:-}" ]; then
  echo "ERROR: Cannot read Vercel token from $AUTH_FILE"
  exit 1
fi

log() { echo "[$(date '+%H:%M:%S')] [vercel-wd] $*"; }

echo $$ > /tmp/vercel-watchdog.pid
log "Started (PID $$, every ${CHECK_INTERVAL}s)"
log "Frontend: $FRONTEND_DIR"
log "Using REST API (team: $TEAM_ID, project: $PROJECT_ID)"
echo ""

while true; do
  # Fetch latest 3 production deployments via REST API
  RESPONSE=$(curl -sf -H "Authorization: Bearer $VERCEL_TOKEN" \
    "https://api.vercel.com/v6/deployments?teamId=$TEAM_ID&projectId=$PROJECT_ID&target=production&limit=3" 2>/dev/null) || {
    log "API call failed, retrying..."
    sleep "$CHECK_INTERVAL"
    continue
  }

  # Parse the most recent deployment
  LATEST_STATE=$(echo "$RESPONSE" | python3 -c "
import sys, json
d = json.load(sys.stdin)
deps = d.get('deployments', [])
if deps:
    latest = deps[0]
    print(f\"{latest.get('state','UNKNOWN')}|{latest.get('uid','')}|{latest.get('url','')}\")
else:
    print('NONE||')
" 2>/dev/null) || { log "Parse failed, retrying..."; sleep "$CHECK_INTERVAL"; continue; }

  STATE=$(echo "$LATEST_STATE" | cut -d'|' -f1)
  UID_VAL=$(echo "$LATEST_STATE" | cut -d'|' -f2)
  URL=$(echo "$LATEST_STATE" | cut -d'|' -f3)

  case "$STATE" in
    READY)
      log "OK — latest deploy READY ($URL)"
      ;;
    ERROR)
      if [ "$UID_VAL" = "$LAST_FIXED_ID" ]; then
        log "Waiting — already redeployed for $UID_VAL"
      else
        log "FAILED deploy detected: $URL (uid: $UID_VAL)"
        log "Redeploying from $FRONTEND_DIR ..."

        cd "$FRONTEND_DIR"
        DEPLOY_OUT=$(vercel --prod --yes --scope "$SCOPE" 2>&1) || true

        if echo "$DEPLOY_OUT" | grep -qi 'aliased\|production:'; then
          ALIAS=$(echo "$DEPLOY_OUT" | grep -iE 'Aliased:|Production:' | tail -1)
          log "SUCCESS — $ALIAS"
          LAST_FIXED_ID="$UID_VAL"
        else
          log "DEPLOY FAILED — will retry next cycle"
          echo "$DEPLOY_OUT" | tail -3
        fi
      fi
      ;;
    BUILDING|QUEUED|INITIALIZING)
      log "OK — deploy in progress ($STATE): $URL"
      ;;
    CANCELED)
      log "OK — latest deploy was canceled: $URL"
      ;;
    NONE)
      log "No deployments found"
      ;;
    *)
      log "Unknown state: $STATE ($URL)"
      ;;
  esac

  sleep "$CHECK_INTERVAL"
done
