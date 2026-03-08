#!/usr/bin/env bash
# Source this to set prod credentials for scripts. Usage: source ./scripts/prod-env.sh
# Requires: Heroku CLI logged in.
# We test only against production; this is the single source for prod URLs and DB.

HEROKU_APP="${HEROKU_APP:-qictrader-backend-rs}"
export HEROKU_APP
export PROD_API_BASE_URL="https://qictrader-backend-rs-13eab0516d9a.herokuapp.com"
export API_BASE_URL="${API_BASE_URL:-$PROD_API_BASE_URL}"
if [ -z "$DATABASE_URL" ]; then
  DATABASE_URL=$(heroku config:get DATABASE_URL -a "$HEROKU_APP" 2>/dev/null) || true
  export DATABASE_URL
fi
echo "PROD env: HEROKU_APP=$HEROKU_APP API_BASE_URL=$API_BASE_URL DATABASE_URL=${DATABASE_URL:+<set>}"
if [ -z "$DATABASE_URL" ]; then
  echo "WARN: DATABASE_URL not set. Run: export DATABASE_URL=\$(heroku config:get DATABASE_URL -a $HEROKU_APP)"
fi
