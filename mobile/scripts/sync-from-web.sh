#!/usr/bin/env bash
# Sync shared files from the main Qictrader monorepo into this mobile repo.
# Run from the qictrader-mobile root. Pass --check to dry-run for CI.

set -euo pipefail

MOBILE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_ROOT="/Users/jpvanzyl/Workspaces/Qictrader"
WEB_FRONTEND="$WEB_ROOT/frontend"
WEB_BACKEND="$WEB_ROOT/qictrader-backend-rs"

CHECK_MODE=false
if [[ "${1:-}" == "--check" ]]; then
  CHECK_MODE=true
fi

if [[ ! -d "$WEB_ROOT" ]]; then
  echo "ERROR: web monorepo not found at $WEB_ROOT" >&2
  echo "       update WEB_ROOT in this script if the path is different on your machine" >&2
  exit 1
fi

if [[ ! -d "$WEB_FRONTEND" ]]; then
  echo "ERROR: web frontend not found at $WEB_FRONTEND" >&2
  exit 1
fi

cd "$MOBILE_ROOT"

# Track whether anything changed (for --check mode)
CHANGES=0

# Helper: copy file if it exists and differs; record change
sync_file() {
  local src="$1"
  local dest="$2"
  local label="$3"

  if [[ ! -f "$src" ]]; then
    echo "  SKIP $label (source missing: $src)"
    return
  fi

  mkdir -p "$(dirname "$dest")"

  if [[ -f "$dest" ]] && diff -q "$src" "$dest" >/dev/null 2>&1; then
    echo "  ===  $label (unchanged)"
    return
  fi

  if $CHECK_MODE; then
    echo "  DRIFT $label"
    CHANGES=$((CHANGES + 1))
  else
    cp "$src" "$dest"
    echo "  SYNC $label"
    CHANGES=$((CHANGES + 1))
  fi
}

# Helper: copy directory contents (top level, not recursive)
sync_dir_flat() {
  local src_dir="$1"
  local dest_dir="$2"
  local label="$3"

  if [[ ! -d "$src_dir" ]]; then
    echo "  SKIP $label (source dir missing: $src_dir)"
    return
  fi

  mkdir -p "$dest_dir"
  shopt -s nullglob
  local files=( "$src_dir"/*.ts "$src_dir"/*.tsx )
  shopt -u nullglob

  if [[ ${#files[@]} -eq 0 ]]; then
    echo "  SKIP $label (no .ts/.tsx files in $src_dir)"
    return
  fi

  for f in "${files[@]}"; do
    [[ -f "$f" ]] || continue
    local name
    name="$(basename "$f")"
    sync_file "$f" "$dest_dir/$name" "$label/$name"
  done
}

echo ""
echo "=========================================="
if $CHECK_MODE; then
  echo " DRY RUN — checking for drift only"
else
  echo " Syncing from web @ $(cd "$WEB_FRONTEND" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"
fi
echo "=========================================="
echo ""

# --- Shared types ---
echo "[ 1/8 ] API + domain types"
sync_file "$WEB_FRONTEND/src/types/api.ts"     "src/types/api.ts"     "src/types/api.ts"
sync_file "$WEB_FRONTEND/src/types/domain.ts"  "src/types/domain.ts"  "src/types/domain.ts"
sync_file "$WEB_FRONTEND/src/types/index.ts"   "src/types/index.ts"   "src/types/index.ts"

# --- Constants ---
echo ""
echo "[ 2/8 ] Constants (currencies, networks, payment methods)"
sync_dir_flat "$WEB_FRONTEND/src/constants" "src/constants" "constants"

# --- Zod schemas (validators) ---
echo ""
echo "[ 3/8 ] Zod validators"
sync_dir_flat "$WEB_FRONTEND/src/schemas" "src/schemas" "schemas"

# --- Pure utility functions ---
echo ""
echo "[ 4/8 ] Formatters + fee calc"
sync_file "$WEB_FRONTEND/src/lib/format.ts"    "src/lib/format.ts"    "lib/format.ts"
sync_file "$WEB_FRONTEND/src/lib/fees.ts"      "src/lib/fees.ts"      "lib/fees.ts"
sync_file "$WEB_FRONTEND/src/lib/dates.ts"     "src/lib/dates.ts"     "lib/dates.ts"

# --- State machine helpers ---
echo ""
echo "[ 5/8 ] State machine helpers"
sync_dir_flat "$WEB_FRONTEND/src/lib/state-machines" "src/lib/state-machines" "state-machines"

# --- Design tokens ---
echo ""
echo "[ 6/8 ] Design tokens + DESIGN.md"
sync_file "$WEB_FRONTEND/src/design-tokens.json" "src/design-tokens.json" "design-tokens.json"
sync_file "$WEB_ROOT/DESIGN.md"                  "DESIGN.md"              "DESIGN.md"

# --- Design intent + as-built state machines (backend docs) ---
echo ""
echo "[ 7/8 ] Backend design intent docs (snapshots into .qictrader-context/)"
sync_file "$WEB_BACKEND/docs/intended-entity-state-machines.md" \
          ".qictrader-context/intended-state-machines.md" \
          "intended-state-machines.md"
sync_file "$WEB_BACKEND/docs/as-built-state-machines.md" \
          ".qictrader-context/as-built-state-machines.md" \
          "as-built-state-machines.md"
sync_file "$WEB_BACKEND/docs/database-schema.md" \
          ".qictrader-context/database-schema.md" \
          "database-schema.md"

# --- Security carryforward ---
echo ""
echo "[ 8/8 ] Security carryforward (architecture snapshot)"
sync_file "$WEB_ROOT/architecture/SECURITY-CARRYFORWARD-FOR-NEW-APPS.md" \
          ".qictrader-context/security-carryforward.md" \
          "security-carryforward.md"

# --- Summary ---
echo ""
echo "=========================================="
if $CHECK_MODE; then
  if [[ $CHANGES -eq 0 ]]; then
    echo " OK — all synced files match web"
    exit 0
  else
    echo " DRIFT detected — $CHANGES file(s) out of sync"
    echo " Run: ./scripts/sync-from-web.sh (without --check) to update"
    exit 1
  fi
else
  echo " Done — $CHANGES file(s) updated"
  echo ""
  echo " Next steps:"
  echo "   1. git status                        # see what changed"
  echo "   2. git diff src/ .qictrader-context/ # review changes"
  echo "   3. bun run typecheck                 # catch any contract drift"
  echo "   4. bun test                          # run affected tests"
  echo "   5. git add . && git commit -m 'chore: sync from web @ $(cd "$WEB_FRONTEND" && git rev-parse --short HEAD 2>/dev/null || echo unknown)'"
fi
echo "=========================================="
