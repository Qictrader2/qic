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
SLUG_RESPONSE=$(curl -sf -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/vnd.heroku+json; version=3" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"process_types\": {\"web\": \"./target/release/qictrader-backend-rs\"},
    \"commit\": \"$COMMIT\",
    \"stack\": \"heroku-24\"
  }" \
  "https://api.heroku.com/apps/$HEROKU_APP/slugs")

if [[ -z "$SLUG_RESPONSE" ]]; then
  echo "ERROR: Failed to create slug."
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
RELEASE_RESPONSE=$(curl -sf -X POST \
  -H "Accept: application/vnd.heroku+json; version=3" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"slug\": \"$SLUG_ID\"}" \
  "https://api.heroku.com/apps/$HEROKU_APP/releases")

RELEASE_VERSION=$(echo "$RELEASE_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('version','?'))" 2>/dev/null)

deploy_end=$(date +%s)
deploy_secs=$((deploy_end - deploy_start))

echo ""
echo "==> Deployed to ${DEPLOY_ENV}! Release v${RELEASE_VERSION} (commit $COMMIT)"
echo "    App: $HEROKU_APP"
echo "    Build: ${build_secs}s | Deploy: ${deploy_secs}s | Total: $(( build_secs + deploy_secs ))s"
