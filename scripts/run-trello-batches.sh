#!/usr/bin/env bash
# Compatible with macOS default bash 3.2
set -euo pipefail

# ---------------------------------------------------------------------------
# run-trello-batches.sh — Pull tickets from Trello, skip N, take 5, run batch
#
# Usage:
#   ./scripts/run-trello-batches.sh [--skip N] [--take M] [--loops L]
#
# Defaults: skip=14, take=5, loops=infinite
# Each loop: fetch Trello To Do → skip first N → take M → run-batch-auto.sh
# ---------------------------------------------------------------------------

SKIP=14
TAKE=5
MAX_LOOPS=0  # 0 = infinite

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip) SKIP="$2"; shift 2 ;;
        --take) TAKE="$2"; shift 2 ;;
        --loops) MAX_LOOPS="$2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BATCH_RUNNER="$SCRIPT_DIR/run-batch-auto.sh"

TRELLO_KEY="d0f2319aeb29e279616c592d79677692"
TRELLO_TOKEN="ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0"
TODO_LIST_ID="69adb7903d71375329df7382"

export QIC_MACHINE="${QIC_MACHINE:-this-pc}"
export QIC_SLOT_OFFSET_BASE="${QIC_SLOT_OFFSET_BASE:-0}"

LOG_DIR="$SCRIPT_DIR/../logs/${QIC_MACHINE}"
mkdir -p "$LOG_DIR"
LOOP_LOG="$LOG_DIR/trello-loop.log"

log() {
    echo "[$(date '+%H:%M:%S')] $*" | tee -a "$LOOP_LOG"
}

fetch_tickets() {
    local skip="$1"
    local take="$2"

    local raw
    raw=$(curl -s "https://api.trello.com/1/lists/${TODO_LIST_ID}/cards?key=${TRELLO_KEY}&token=${TRELLO_TOKEN}&fields=name")

    if [ -z "$raw" ] || echo "$raw" | grep -q '"error"'; then
        echo "ERROR: Failed to fetch Trello cards" >&2
        return 1
    fi

    # Extract card names, skip N, take M
    echo "$raw" | python3 -c "
import sys, json
cards = json.load(sys.stdin)
names = [c['name'] for c in cards]
selected = names[${skip}:${skip}+${take}]
for n in selected:
    print(n)
"
}

LOOP_COUNT=0

log "============================================"
log "  QIC Trello Batch Loop"
log "============================================"
log "Skip:   $SKIP (start from ticket $((SKIP + 1)))"
log "Take:   $TAKE"
log "Loops:  $([ "$MAX_LOOPS" = "0" ] && echo "infinite" || echo "$MAX_LOOPS")"
log "Machine: $QIC_MACHINE"
log ""

while true; do
    LOOP_COUNT=$((LOOP_COUNT + 1))

    if [ "$MAX_LOOPS" -gt 0 ] && [ "$LOOP_COUNT" -gt "$MAX_LOOPS" ]; then
        log "Reached max loops ($MAX_LOOPS). Stopping."
        break
    fi

    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log "  LOOP $LOOP_COUNT — Fetching Trello To Do list..."
    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    TICKET_NAMES=()
    while IFS= read -r name; do
        [ -n "$name" ] && TICKET_NAMES+=("$name")
    done < <(fetch_tickets "$SKIP" "$TAKE")

    if [ ${#TICKET_NAMES[@]} -eq 0 ]; then
        log "No tickets found at position $((SKIP + 1))+. Done!"
        break
    fi

    log "Picked ${#TICKET_NAMES[@]} tickets:"
    for t in "${TICKET_NAMES[@]}"; do
        log "  - $t"
    done
    log ""

    # Run the batch
    "$BATCH_RUNNER" "${TICKET_NAMES[@]}"
    batch_exit=$?

    if [ $batch_exit -ne 0 ]; then
        log "WARNING: Batch exited with code $batch_exit"
    fi

    log ""
    log "Loop $LOOP_COUNT complete. Cooling down 30s before next batch..."
    sleep 30
done

log ""
log "============================================"
log "  TRELLO BATCH LOOP COMPLETE"
log "============================================"
log "Loops run: $LOOP_COUNT"
