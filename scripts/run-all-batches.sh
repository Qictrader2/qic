#!/usr/bin/env bash
# Compatible with macOS default bash 3.2
set -euo pipefail

# ---------------------------------------------------------------------------
# run-all-batches.sh — Runs ALL batches for a machine, one after another
#
# Usage:
#   ./scripts/run-all-batches.sh THIS-PC-TICKETS.md
#   ./scripts/run-all-batches.sh PC2-TICKETS.md
#
# Derives machine name from the ticket filename (e.g. THIS-PC-TICKETS.md → this-pc).
# Logs, summaries, and questions are all per-machine and persistent.
# ---------------------------------------------------------------------------

TICKET_FILE="${1:-}"

if [ -z "$TICKET_FILE" ] || [ ! -f "$TICKET_FILE" ]; then
    echo "Usage: $0 <TICKETS-FILE.md>"
    echo ""
    echo "  Reads all batches from the ticket file and runs them sequentially."
    echo "  Each batch runs up to 5 tickets in parallel via run-batch-auto.sh."
    echo ""
    echo "Example:"
    echo "  $0 THIS-PC-TICKETS.md"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SOURCE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BATCH_RUNNER="$SCRIPT_DIR/run-batch-auto.sh"

# Derive machine name from filename: THIS-PC-TICKETS.md → this-pc
BASENAME=$(basename "$TICKET_FILE" .md)
MACHINE=$(echo "$BASENAME" | sed 's/-TICKETS$//' | tr '[:upper:]' '[:lower:]')
export QIC_MACHINE="$MACHINE"

# Migration offset base: this-pc slots get 1-5, pc2 slots get 6-10
case "$MACHINE" in
    this-pc) export QIC_SLOT_OFFSET_BASE=0 ;;
    pc2)     export QIC_SLOT_OFFSET_BASE=5 ;;
    *)       export QIC_SLOT_OFFSET_BASE=0 ;;
esac

LOG_DIR="$SOURCE_DIR/logs/$MACHINE"
mkdir -p "$LOG_DIR"
ALL_LOG="$LOG_DIR/all-batches.log"

log() {
    echo "[$(date '+%H:%M:%S')] $*" | tee -a "$ALL_LOG"
}

# ---------------------------------------------------------------------------
# Parse the ticket file: extract batch numbers and ticket names
# ---------------------------------------------------------------------------

BATCH_COUNT=0
PARSED_FILE=$(mktemp)
trap 'rm -f "$PARSED_FILE"' EXIT

current_batch=""
in_code_block=false

while IFS= read -r line; do
    if echo "$line" | grep -qE '^## Batch [0-9]+'; then
        current_batch=$(echo "$line" | grep -oE '[0-9]+')
        in_code_block=false
        continue
    fi

    if [ -n "$current_batch" ] && echo "$line" | grep -qE '^\s*```bash'; then
        in_code_block=true
        continue
    fi

    if [ -n "$current_batch" ] && $in_code_block && echo "$line" | grep -qE '^\s*```\s*$'; then
        in_code_block=false
        current_batch=""
        continue
    fi

    if $in_code_block && [ -n "$current_batch" ]; then
        ticket=$(echo "$line" | sed 's/.*"\(.*\)".*/\1/' | sed 's/\\$//')
        ticket=$(echo "$ticket" | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
        if [ -n "$ticket" ] && ! echo "$ticket" | grep -qE '^\./scripts/|^cd |^$'; then
            echo "${current_batch}|${ticket}" >> "$PARSED_FILE"
        fi
    fi
done < "$TICKET_FILE"

if [ ! -s "$PARSED_FILE" ]; then
    echo "ERROR: No batches found in $TICKET_FILE"
    exit 1
fi

TOTAL_BATCHES=$(cut -d'|' -f1 "$PARSED_FILE" | sort -un | tail -1)
TOTAL_TICKETS=$(wc -l < "$PARSED_FILE" | tr -d ' ')

log "============================================"
log "  QIC All-Batches Runner"
log "============================================"
log "Machine:  $MACHINE"
log "Tickets:  $TOTAL_TICKETS"
log "Batches:  $TOTAL_BATCHES"
log "Logs:     $LOG_DIR/"
log "Summary:  $SOURCE_DIR/ticket_summary_${MACHINE}.md"
log "Questions: $SOURCE_DIR/ticket_questions_${MACHINE}.md"
log ""

for batch_num in $(cut -d'|' -f1 "$PARSED_FILE" | sort -un); do
    BATCH_COUNT=$((BATCH_COUNT + 1))

    tickets=()
    while IFS='|' read -r bnum ticket; do
        if [ "$bnum" = "$batch_num" ]; then
            tickets+=("$ticket")
        fi
    done < "$PARSED_FILE"

    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log "  BATCH $batch_num of $TOTAL_BATCHES  (${#tickets[@]} tickets)"
    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    for t in "${tickets[@]}"; do
        log "    - $t"
    done
    log ""

    "$BATCH_RUNNER" "${tickets[@]}"
    batch_exit=$?

    if [ $batch_exit -ne 0 ]; then
        log "WARNING: Batch $batch_num exited with code $batch_exit"
    fi

    log ""
    log "Batch $batch_num complete. Cooling down 30s before next batch..."
    log ""

    if [ "$BATCH_COUNT" -lt "$TOTAL_BATCHES" ]; then
        sleep 30
    fi
done

log ""
log "============================================"
log "  ALL BATCHES COMPLETE"
log "============================================"
log "Machine: $MACHINE"
log "Batches run: $BATCH_COUNT"
log "Check ticket_summary_${MACHINE}.md for per-batch results"
log "Check ticket_questions_${MACHINE}.md for any parked tickets"
