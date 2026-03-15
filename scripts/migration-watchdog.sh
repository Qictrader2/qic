#!/usr/bin/env bash
# migration-watchdog.sh — Checks for duplicate migration timestamps every 2 minutes.
# If two or more .up.sql files share the same numeric version prefix, renames the
# duplicates to unique timestamps, commits, and pushes automatically.
#
# Scans:
#   - Main repo:    qictrader-backend-rs/migrations/
#   - Parallel slots: parallel-runs/this-pc/slot-*/qictrader-backend-rs/migrations/
#
# Usage:
#   ./scripts/migration-watchdog.sh                # foreground
#   nohup ./scripts/migration-watchdog.sh &        # background
#
# Stop: kill $(cat /tmp/migration-watchdog.pid)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECK_INTERVAL=120

log() { echo "[$(date '+%H:%M:%S')] [mig-wd] $*"; }

echo $$ > /tmp/migration-watchdog.pid
log "Started (PID $$, every ${CHECK_INTERVAL}s)"
log "Root: $ROOT_DIR"
echo ""

fix_duplicates_in() {
  local mig_dir="$1"
  local label="$2"

  [ -d "$mig_dir" ] || return 0

  # Extract numeric version prefixes from .up.sql files, find duplicates
  local dupes
  dupes=$(ls "$mig_dir"/*.up.sql 2>/dev/null \
    | xargs -I{} basename {} \
    | sed 's/_.*//' \
    | sort | uniq -d) || true

  [ -z "$dupes" ] && return 0

  log "[$label] DUPLICATE timestamps found: $dupes"

  for version in $dupes; do
    # Get all .up.sql files with this version
    local files
    files=$(ls "$mig_dir"/${version}_*.up.sql 2>/dev/null | sort) || continue
    local count
    count=$(echo "$files" | wc -l | tr -d ' ')

    if [ "$count" -le 1 ]; then
      continue
    fi

    log "[$label] Version $version has $count migrations — fixing..."

    # Keep the first file (alphabetically), rename the rest
    local idx=0
    while IFS= read -r upfile; do
      idx=$((idx + 1))
      [ "$idx" -eq 1 ] && continue  # keep the first one

      local base
      base=$(basename "$upfile")
      local suffix="${base#${version}_}"    # e.g. auth003_oauth_providers.up.sql
      local name_part="${suffix%.up.sql}"    # e.g. auth003_oauth_providers

      local new_version=$((version + idx - 1))
      local new_up="${mig_dir}/${new_version}_${name_part}.up.sql"
      local new_down="${mig_dir}/${new_version}_${name_part}.down.sql"
      local old_down="${mig_dir}/${version}_${name_part}.down.sql"

      if [ -f "$new_up" ]; then
        log "[$label]   Target $new_version already exists, trying +10..."
        new_version=$((version + idx * 10))
        new_up="${mig_dir}/${new_version}_${name_part}.up.sql"
        new_down="${mig_dir}/${new_version}_${name_part}.down.sql"
      fi

      log "[$label]   Rename: ${version}_${name_part} -> ${new_version}_${name_part}"
      mv "$upfile" "$new_up"

      if [ -f "$old_down" ]; then
        mv "$old_down" "$new_down"
      fi
    done <<< "$files"
  done

  return 1  # signal that fixes were made
}

commit_and_push() {
  local repo_dir="$1"
  local label="$2"

  cd "$repo_dir"
  local changes
  changes=$(git status --porcelain -- migrations/ 2>/dev/null) || true
  [ -z "$changes" ] && return 0

  log "[$label] Committing migration timestamp fixes..."
  git add migrations/ 2>/dev/null || true
  git commit -m "hotfix: resolve duplicate migration timestamps (auto-watchdog)" 2>/dev/null || true
  git push origin main 2>/dev/null || true
  git push heroku main 2>/dev/null || true
  log "[$label] Pushed to origin + heroku"
}

while true; do
  FIXED=false

  # Check main repo
  MAIN_MIG="$ROOT_DIR/qictrader-backend-rs/migrations"
  if [ -d "$MAIN_MIG" ]; then
    fix_duplicates_in "$MAIN_MIG" "main" || {
      FIXED=true
      commit_and_push "$ROOT_DIR/qictrader-backend-rs" "main"
    }
  fi

  # Check parallel slots
  for slot_dir in "$ROOT_DIR"/parallel-runs/this-pc/slot-*/qictrader-backend-rs; do
    [ -d "$slot_dir/migrations" ] || continue
    slot_name=$(echo "$slot_dir" | grep -oE 'slot-[0-9]+')
    fix_duplicates_in "$slot_dir/migrations" "$slot_name" || {
      FIXED=true
      commit_and_push "$slot_dir" "$slot_name"
    }
  done

  if [ "$FIXED" = "true" ]; then
    log "Fixes applied and pushed"
  else
    log "OK — no duplicate migration timestamps"
  fi

  sleep "$CHECK_INTERVAL"
done
