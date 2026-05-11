#!/usr/bin/env bash
# fast-deploy-backend.sh — Cross-compile Rust backend locally and deploy via Heroku Slug API.
#
# On macOS (ARM): cross-compiles to x86_64-unknown-linux-gnu using cargo-zigbuild
# On Linux (x86_64): builds natively with cargo build
#
# Usage:
#   ./scripts/fast-deploy-backend.sh              # build + deploy to STAGING (default)
#   ./scripts/fast-deploy-backend.sh --staging    # explicit staging (same as default)
#   ./scripts/fast-deploy-backend.sh --prod       # build + deploy to PRODUCTION
#   ./scripts/fast-deploy-backend.sh --build-only # just cross-compile, don't deploy
#   ./scripts/fast-deploy-backend.sh --dry-run    # show what would happen
#
# Prerequisites (macOS):
#   brew install zig
#   cargo install cargo-zigbuild
#   rustup target add x86_64-unknown-linux-gnu
#
# Prerequisites (Linux):
#   rustup (stable toolchain)
#
# Both:
#   heroku CLI logged in
#   Optional: RUSTC_WRAPPER=sccache for faster cached builds

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/qictrader-backend-rs"
HEROKU_APP_STAGING="qictrader-backend-staging"
HEROKU_APP_PROD="qictrader-backend-rs"
TARGET="x86_64-unknown-linux-gnu"
GLIBC_VERSION="2.39"  # heroku-24 = Ubuntu 24.04

BUILD_ONLY=false
DRY_RUN=false
DEPLOY_ENV="staging"  # default to staging

for arg in "$@"; do
  case "$arg" in
    --staging)    DEPLOY_ENV="staging" ;;
    --prod)       DEPLOY_ENV="production" ;;
    --build-only) BUILD_ONLY=true ;;
    --dry-run)    DRY_RUN=true ;;
    *)            echo "Unknown arg: $arg"; exit 1 ;;
  esac
done

if [[ "$DEPLOY_ENV" == "production" ]]; then
  HEROKU_APP="$HEROKU_APP_PROD"
else
  HEROKU_APP="$HEROKU_APP_STAGING"
fi

# ---------------------------------------------------------------------------
# Production deploy guard rails (added 2026-05-11 after a Cursor-agent-
# triggered unauthorised v796 deploy that crashed prod for 8 hours).
#
# Bypass is intentional — each guard exists to make the "I am about to deploy
# untested code to a real-money custodial backend" step deliberate, not
# accidental. Bypassing should require ≥2 explicit signals.
#
# Guard order (cheapest first):
#   1. CONFIRM_PROD_DEPLOY=yes env var          — blocks autonomous agents
#   2. HEAD is on a tagged commit               — enforces release discipline
#   3. Staging /health = 200                    — enforces staging-first
#   4. Staging's deployed commit = HEAD         — enforces "test the exact bits"
#   5. Interactive "type PRODUCTION" if TTY     — extra friction for humans
#
# All guards are skipped when --build-only or --dry-run (no actual deploy).
# ---------------------------------------------------------------------------
prod_audit_log() {
  local log_dir="$ROOT_DIR/ops"
  local log_file="$log_dir/prod-deploys.log"
  [[ -d "$log_dir" ]] || mkdir -p "$log_dir"
  local actor
  actor="$(heroku auth:whoami 2>/dev/null || echo unknown)"
  local head_commit
  head_commit="$(cd "$BACKEND_DIR" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"
  local head_tag
  head_tag="$(cd "$BACKEND_DIR" && git tag --points-at HEAD 2>/dev/null | head -1)"
  printf '%s  actor=%s  commit=%s  tag=%s  reason=%s\n' \
    "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
    "$actor" \
    "$head_commit" \
    "${head_tag:-none}" \
    "$1" \
    >> "$log_file"
}

if [[ "$DEPLOY_ENV" == "production" && "$BUILD_ONLY" != "true" && "$DRY_RUN" != "true" ]]; then
  echo "==> Production deploy requested — running guard rails"
  echo ""

  # Guard 1: explicit confirmation env var
  if [[ "${CONFIRM_PROD_DEPLOY:-}" != "yes" ]]; then
    prod_audit_log "blocked-no-confirm"
    cat >&2 <<'EOF'
ERROR: production deploy refused — CONFIRM_PROD_DEPLOY env var not set.

Why: this guard exists so autonomous tools (Cursor agents, cron jobs,
     misclicked shell history) cannot ship to production without a second
     deliberate signal.

Fix: re-run the same command with the env var inline, e.g.

  CONFIRM_PROD_DEPLOY=yes ./scripts/fast-deploy-backend.sh --prod

If you didn't mean to deploy to production, drop the --prod flag.
EOF
    exit 2
  fi

  # Guard 2: HEAD must be on a tagged commit
  HEAD_TAG="$(cd "$BACKEND_DIR" && git tag --points-at HEAD 2>/dev/null | head -1)"
  if [[ -z "$HEAD_TAG" ]]; then
    prod_audit_log "blocked-untagged"
    HEAD_COMMIT_SHORT="$(cd "$BACKEND_DIR" && git rev-parse --short HEAD)"
    cat >&2 <<EOF
ERROR: production deploy refused — HEAD ($HEAD_COMMIT_SHORT) is not at a tagged commit.

Why: production deploys must come from explicitly-tagged releases so we have
     a single name we can rollback to, a single thing we can talk about in
     post-incident reviews, and a single commit we can ask "did staging test
     this exact code?".

Fix:
  cd $BACKEND_DIR
  git tag -a v$(date +%Y.%m.%d) -m 'release notes'
  git push origin v$(date +%Y.%m.%d)
  cd -
  # then re-run this command
EOF
    exit 2
  fi

  # Guard 3 + 4: staging must be healthy AND on the same commit
  STAGING_HOST="qictrader-backend-staging-2290a9290b6b.herokuapp.com"
  STAGING_HEALTH="$(curl -sS -m 10 -o /dev/null -w '%{http_code}' "https://$STAGING_HOST/health" 2>/dev/null || echo 000)"
  if [[ "$STAGING_HEALTH" != "200" ]]; then
    prod_audit_log "blocked-staging-unhealthy-$STAGING_HEALTH"
    cat >&2 <<EOF
ERROR: production deploy refused — staging /health returned $STAGING_HEALTH (expected 200).

Why: prod can only ship what staging has proven healthy. If staging itself
     is broken, prod must not become the next thing to break.

Fix: deploy this commit to staging first, confirm /health=200, then re-run --prod.
EOF
    exit 2
  fi

  HEAD_COMMIT_SHORT="$(cd "$BACKEND_DIR" && git rev-parse --short HEAD)"
  STAGING_DEPLOY_LINE="$(heroku releases --app "$HEROKU_APP_STAGING" --num 1 2>/dev/null | grep -E '^\s*v[0-9]+' | head -1)"
  STAGING_DEPLOY_COMMIT="$(echo "$STAGING_DEPLOY_LINE" | awk '{print $3}')"
  # Strip trailing 'd' from short commit refs ("Deploy 35b1312d" sometimes appears).
  STAGING_DEPLOY_COMMIT="${STAGING_DEPLOY_COMMIT%d}"

  if [[ -z "$STAGING_DEPLOY_COMMIT" ]] || [[ "${HEAD_COMMIT_SHORT}" != "${STAGING_DEPLOY_COMMIT:0:${#HEAD_COMMIT_SHORT}}" ]]; then
    prod_audit_log "blocked-staging-mismatch-staging=${STAGING_DEPLOY_COMMIT}-head=${HEAD_COMMIT_SHORT}"
    cat >&2 <<EOF
ERROR: production deploy refused — staging is not running this commit.

  Local HEAD:      $HEAD_COMMIT_SHORT  (tag: $HEAD_TAG)
  Staging release: $STAGING_DEPLOY_COMMIT
  Staging health:  200 OK

Why: prod must ship exactly the commit staging has been running. Drift here
     is how silent regressions ship — staging tests on one commit, prod runs
     a different one, and you don't notice until users do.

Fix: deploy this commit to staging first (without --prod), smoke-test it,
     then re-run with --prod.
EOF
    exit 2
  fi

  # Guard 5: interactive confirmation
  if [[ -t 0 ]]; then
    cat <<EOF
================================================================
  PRODUCTION DEPLOY — $HEROKU_APP_PROD
  Commit:  $HEAD_COMMIT_SHORT
  Tag:     $HEAD_TAG
  Staging: HEALTHY at $STAGING_DEPLOY_COMMIT (match)
================================================================
EOF
    read -p "Type 'PRODUCTION' to confirm (anything else aborts): " CONFIRM
    if [[ "$CONFIRM" != "PRODUCTION" ]]; then
      prod_audit_log "blocked-tty-abort"
      echo "Aborted." >&2
      exit 2
    fi
  fi

  prod_audit_log "approved"
  echo "==> All guard rails passed. Proceeding with production deploy."
  echo ""
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

echo "==> Fast deploy: $OS/$ARCH -> Heroku ($HEROKU_APP) [${DEPLOY_ENV}]"
echo ""

# --- Step 1: Build ---

cd "$BACKEND_DIR"
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

build_start=$(date +%s)

if [[ "$OS" == "Darwin" ]]; then
  # macOS: cross-compile with cargo-zigbuild
  if ! command -v cargo-zigbuild &>/dev/null; then
    echo "ERROR: cargo-zigbuild not found. Install with: cargo install cargo-zigbuild"
    exit 1
  fi
  if ! command -v zig &>/dev/null; then
    echo "ERROR: zig not found. Install with: brew install zig"
    exit 1
  fi

  echo "==> Cross-compiling for $TARGET (glibc $GLIBC_VERSION)..."
  BINARY_PATH="target/$TARGET/release/qictrader-backend-rs"

  if $DRY_RUN; then
    echo "[dry-run] cargo zigbuild --release --target $TARGET.$GLIBC_VERSION"
  else
    cargo zigbuild --release --target "$TARGET.$GLIBC_VERSION" 2>&1
  fi

elif [[ "$OS" == "Linux" && "$ARCH" == "x86_64" ]]; then
  # Linux x86_64: build natively
  echo "==> Building natively for $TARGET..."
  BINARY_PATH="target/release/qictrader-backend-rs"

  if $DRY_RUN; then
    echo "[dry-run] cargo build --release"
  else
    cargo build --release 2>&1
  fi

elif [[ "$OS" == "Linux" ]]; then
  # Linux non-x86_64 (e.g. ARM): cross-compile
  echo "==> Cross-compiling on Linux $ARCH for $TARGET..."
  BINARY_PATH="target/$TARGET/release/qictrader-backend-rs"

  if command -v cargo-zigbuild &>/dev/null; then
    if $DRY_RUN; then
      echo "[dry-run] cargo zigbuild --release --target $TARGET.$GLIBC_VERSION"
    else
      cargo zigbuild --release --target "$TARGET.$GLIBC_VERSION" 2>&1
    fi
  else
    # Fallback: try native cross if linker is configured
    if $DRY_RUN; then
      echo "[dry-run] cargo build --release --target $TARGET"
    else
      cargo build --release --target "$TARGET" 2>&1
    fi
  fi
else
  echo "ERROR: Unsupported platform: $OS/$ARCH"
  exit 1
fi

build_end=$(date +%s)
build_secs=$((build_end - build_start))

if ! $DRY_RUN; then
  BINARY_SIZE=$(wc -c < "$BINARY_PATH" 2>/dev/null | tr -d ' ')
  echo ""
  echo "==> Build complete in ${build_secs}s"
  echo "    Binary: $BINARY_PATH ($(( BINARY_SIZE / 1024 / 1024 ))MB)"
fi

if $BUILD_ONLY; then
  echo "==> Build only mode, skipping deploy."
  exit 0
fi

# --- Step 2: Package slug tarball ---

echo ""
echo "==> Packaging slug..."

SLUG_DIR=$(mktemp -d)
SLUG_TAR=$(mktemp).tgz
trap "rm -rf '$SLUG_DIR' '$SLUG_TAR'" EXIT

mkdir -p "$SLUG_DIR/app/target/release"
if $DRY_RUN; then
  echo "[dry-run] cp $BINARY_PATH -> slug tarball"
else
  cp "$BINARY_PATH" "$SLUG_DIR/app/target/release/qictrader-backend-rs"
  chmod +x "$SLUG_DIR/app/target/release/qictrader-backend-rs"

  # Include migrations if they exist (needed for runtime migration)
  if [[ -d "$BACKEND_DIR/migrations" ]]; then
    cp -r "$BACKEND_DIR/migrations" "$SLUG_DIR/app/migrations"
  fi

  (cd "$SLUG_DIR" && tar czf "$SLUG_TAR" ./app)
  SLUG_SIZE=$(wc -c < "$SLUG_TAR" | tr -d ' ')
  echo "    Slug: $(( SLUG_SIZE / 1024 / 1024 ))MB compressed"
fi

# --- Step 3: Deploy via Heroku Slug API ---

echo ""
echo "==> Deploying to Heroku ($HEROKU_APP) via Slug API..."

deploy_start=$(date +%s)

TOKEN=$(heroku auth:token 2>/dev/null | tail -1)
if [[ -z "$TOKEN" ]]; then
  echo "ERROR: Could not get Heroku auth token. Run: heroku login"
  exit 1
fi

if $DRY_RUN; then
  echo "[dry-run] POST /apps/$HEROKU_APP/slugs (create slug)"
  echo "[dry-run] PUT blob URL (upload tarball)"
  echo "[dry-run] POST /apps/$HEROKU_APP/releases (release slug)"
  echo ""
  echo "==> Dry run complete."
  exit 0
fi

# Create slug
SLUG_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/vnd.heroku+json; version=3" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"process_types\": {\"web\": \"./target/release/qictrader-backend-rs\"},
    \"commit\": \"$COMMIT\",
    \"stack\": \"heroku-24\"
  }" \
  "https://api.heroku.com/apps/$HEROKU_APP/slugs")

SLUG_HTTP=$(echo "$SLUG_RESPONSE" | grep "HTTP_STATUS:" | cut -d: -f2)
SLUG_RESPONSE=$(echo "$SLUG_RESPONSE" | grep -v "HTTP_STATUS:")

if [[ "$SLUG_HTTP" -lt 200 || "$SLUG_HTTP" -ge 300 || -z "$SLUG_RESPONSE" ]]; then
  echo "ERROR: Failed to create slug (HTTP $SLUG_HTTP)."
  echo "$SLUG_RESPONSE"
  exit 1
fi

SLUG_ID=$(echo "$SLUG_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
UPLOAD_URL=$(echo "$SLUG_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['blob']['url'])" 2>/dev/null)

if [[ -z "$SLUG_ID" || -z "$UPLOAD_URL" ]]; then
  echo "ERROR: Could not parse slug response."
  echo "$SLUG_RESPONSE"
  exit 1
fi

echo "    Slug ID: $SLUG_ID"

# Upload tarball
echo "    Uploading slug..."
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X PUT \
  -H "Content-Type:" \
  --data-binary @"$SLUG_TAR" \
  "$UPLOAD_URL")

if [[ "$HTTP_STATUS" -lt 200 || "$HTTP_STATUS" -ge 300 ]]; then
  echo "ERROR: Slug upload failed with HTTP $HTTP_STATUS"
  exit 1
fi

# Release slug
RELEASE_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST \
  -H "Accept: application/vnd.heroku+json; version=3" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"slug\": \"$SLUG_ID\"}" \
  "https://api.heroku.com/apps/$HEROKU_APP/releases")

RELEASE_HTTP=$(echo "$RELEASE_RESPONSE" | grep "HTTP_STATUS:" | cut -d: -f2)
RELEASE_RESPONSE=$(echo "$RELEASE_RESPONSE" | grep -v "HTTP_STATUS:")

if [[ "$RELEASE_HTTP" -lt 200 || "$RELEASE_HTTP" -ge 300 || -z "$RELEASE_RESPONSE" ]]; then
  echo "ERROR: Failed to release slug (HTTP $RELEASE_HTTP)."
  echo "$RELEASE_RESPONSE"
  exit 1
fi

RELEASE_VERSION=$(echo "$RELEASE_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('version','?'))" 2>/dev/null)

deploy_end=$(date +%s)
deploy_secs=$((deploy_end - deploy_start))

echo ""
echo "==> Deployed to ${DEPLOY_ENV}! Release v${RELEASE_VERSION} (commit $COMMIT)"
echo "    App: $HEROKU_APP"
echo "    Build: ${build_secs}s | Deploy: ${deploy_secs}s | Total: $(( build_secs + deploy_secs ))s"
