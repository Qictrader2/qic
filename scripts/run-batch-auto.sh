#!/usr/bin/env bash
# Compatible with macOS default bash 3.2 (no associative arrays).
set -euo pipefail

# ---------------------------------------------------------------------------
# run-batch-auto.sh -- Fully autonomous batch runner (v3 -- scrollback counting)
#
# Usage:
#   ./scripts/run-batch-auto.sh "TICKET-1 name" "TICKET-2 name" ...
#
# Detection method: count occurrences of phase-specific completion strings
# in tmux scrollback. A NEW occurrence = the command finished. No pipe-pane.
# ---------------------------------------------------------------------------

MACHINE="${QIC_MACHINE:-default}"
BASE_DIR="${QIC_PARALLEL_DIR:-$HOME/Workspaces/Qictrader/parallel-runs/$MACHINE}"
SESSION="qic-${MACHINE}"
STARTUP_WAIT="${CLAUDE_STARTUP_WAIT:-15}"
POLL_INTERVAL=20

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SOURCE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

REMOTE_ROOT="https://github.com/Qictrader2/qic.git"
REMOTE_FRONTEND="https://github.com/Qictrader2/Frontend.git"
REMOTE_BACKEND="https://github.com/Qictrader2/qictrader-backend-rs.git"
REMOTE_HEROKU="https://git.heroku.com/qictrader-backend-rs.git"

LOG_DIR="$SOURCE_DIR/logs/$MACHINE"
PROGRESS_LOG="$LOG_DIR/progress.log"
QUESTIONS_FILE="$SOURCE_DIR/ticket_questions_${MACHINE}.md"
SUMMARY_FILE="$SOURCE_DIR/ticket_summary_${MACHINE}.md"

TICKETS=("$@")

if [ ${#TICKETS[@]} -eq 0 ]; then
    echo "Usage: $0 'TICKET-1 name' 'TICKET-2 name' ..."
    exit 1
fi

mkdir -p "$LOG_DIR"

log() {
    echo "[$(date '+%H:%M:%S')] $*" | tee -a "$PROGRESS_LOG"
}

# ---------------------------------------------------------------------------
# Strip ANSI escape codes from terminal output
# ---------------------------------------------------------------------------
strip_ansi() {
    perl -pe 's/\e\[[0-9;]*[mGKHJsu]//g; s/\e\[\?[0-9;]*[hl]//g; s/\e\([AB]//g; s/\e\][^\a]*\a//g; s/\r//g' 2>/dev/null || \
    sed 's/\x1b\[[0-9;]*[a-zA-Z]//g; s/\x1b\[?[0-9;]*[a-zA-Z]//g'
}

# ---------------------------------------------------------------------------
# CORE DETECTION: Count occurrences of a pattern in tmux scrollback
# ---------------------------------------------------------------------------
count_in_scrollback() {
    local window="$1"
    local pattern="$2"
    local result
    result=$(tmux capture-pane -t "$SESSION:$window" -p -S -1000 2>/dev/null | \
        strip_ansi | \
        grep -ci "$pattern" 2>/dev/null) || result=0
    echo "$result"
}

# ---------------------------------------------------------------------------
# Idle detection: Claude is idle when "bypass permissions on" visible
# and "esc to interrupt" is NOT visible
# ---------------------------------------------------------------------------
is_claude_idle() {
    local window="$1"
    local last_lines
    last_lines=$(tmux capture-pane -t "$SESSION:$window" -p -S -8 2>/dev/null)
    if ! echo "$last_lines" | grep -q "bypass permissions on"; then
        return 1
    fi
    if echo "$last_lines" | grep -q "esc to interrupt"; then
        return 1
    fi
    return 0
}

# ---------------------------------------------------------------------------
# Check for actual code changes in a slot directory
# ---------------------------------------------------------------------------
has_code_changes() {
    local slot=$1
    local slot_dir="$BASE_DIR/slot-$slot"
    local fe_diff be_diff fe_new be_new
    fe_diff=$(cd "$slot_dir/Frontend" 2>/dev/null && git diff --stat HEAD 2>/dev/null || true)
    be_diff=$(cd "$slot_dir/qictrader-backend-rs" 2>/dev/null && git diff --stat HEAD 2>/dev/null || true)
    fe_new=$(cd "$slot_dir/Frontend" 2>/dev/null && git ls-files --others --exclude-standard 2>/dev/null | head -1 || true)
    be_new=$(cd "$slot_dir/qictrader-backend-rs" 2>/dev/null && git ls-files --others --exclude-standard 2>/dev/null | head -1 || true)
    [ -n "$fe_diff" ] || [ -n "$be_diff" ] || [ -n "$fe_new" ] || [ -n "$be_new" ]
}

send_command() {
    local window="$1"
    local cmd="$2"
    tmux send-keys -t "$SESSION:$window" "$cmd" Enter
}

get_pane_content() {
    local window="$1"
    tmux capture-pane -t "$SESSION:$window" -p -S -200 2>/dev/null | strip_ansi
}

# Minimum wait (seconds) before even checking for completion
min_wait_for_phase() {
    case "$1" in
        ticket)  echo 180 ;;
        temper)  echo 120 ;;
        commit)  echo 60  ;;
        golive)  echo 60  ;;
        *)       echo 60  ;;
    esac
}

# ---------------------------------------------------------------------------
# Phase-specific completion patterns
# /ticket  -> "Implementation complete"
# /temper  -> "VERDICT:"
# /git-commit -> "pushed" (after git push)
# /golive  -> "Go-Live Complete"
# ---------------------------------------------------------------------------
TICKET_DONE_PAT="Implementation complete"
TICKET_QUESTION_PAT="[Qq]uestion"
TEMPER_DONE_PAT="VERDICT:"
COMMIT_DONE_PAT="pushed"
GOLIVE_DONE_PAT="Go-Live Complete"

# ---------------------------------------------------------------------------
# Step 1: Create slot directories
# ---------------------------------------------------------------------------

log "============================================"
log "  QIC Autonomous Batch Runner (v3)"
log "============================================"
log "Tickets: ${#TICKETS[@]}"
log ""

mkdir -p "$BASE_DIR"

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    SLOT_DIR="$BASE_DIR/slot-$SLOT"
    TICKET="${TICKETS[$i]}"

    log "--- Slot $SLOT: $TICKET ---"

    if [ -d "$SLOT_DIR/.git" ]; then
        log "  Resetting to origin/main..."
        (cd "$SLOT_DIR/Frontend" && git remote set-url origin "$REMOTE_FRONTEND" && git fetch origin --quiet && git reset --hard origin/main --quiet) 2>/dev/null || true
        (cd "$SLOT_DIR/qictrader-backend-rs" && git remote set-url origin "$REMOTE_BACKEND" && git fetch origin --quiet && git reset --hard origin/main --quiet) 2>/dev/null || true
        (cd "$SLOT_DIR" && git remote set-url origin "$REMOTE_ROOT" && git fetch origin --quiet && git reset --hard origin/main --quiet) 2>/dev/null || true
    else
        rm -rf "$SLOT_DIR"
        log "  Cloning (local + reset to remote)..."
        git clone --quiet "$SOURCE_DIR" "$SLOT_DIR"
        rm -rf "$SLOT_DIR/Frontend"
        git clone --quiet "$SOURCE_DIR/Frontend" "$SLOT_DIR/Frontend"
        rm -rf "$SLOT_DIR/qictrader-backend-rs"
        git clone --quiet "$SOURCE_DIR/qictrader-backend-rs" "$SLOT_DIR/qictrader-backend-rs"

        (cd "$SLOT_DIR" && git remote set-url origin "$REMOTE_ROOT")
        (cd "$SLOT_DIR/Frontend" && git remote set-url origin "$REMOTE_FRONTEND")
        (cd "$SLOT_DIR/qictrader-backend-rs" && git remote set-url origin "$REMOTE_BACKEND")
        (cd "$SLOT_DIR/qictrader-backend-rs" && git remote add heroku "$REMOTE_HEROKU" 2>/dev/null || true)
        (cd "$SLOT_DIR" && ln -sf Frontend frontend 2>/dev/null || true)

        (cd "$SLOT_DIR" && git fetch origin --quiet && git reset --hard origin/main --quiet) 2>/dev/null || true
        (cd "$SLOT_DIR/Frontend" && git fetch origin --quiet && git reset --hard origin/main --quiet) 2>/dev/null || true
        (cd "$SLOT_DIR/qictrader-backend-rs" && git fetch origin --quiet && git reset --hard origin/main --quiet) 2>/dev/null || true
    fi

    # Copy commands if needed
    if [ -f "$HOME/.claude/commands/git-commit.md" ] && [ ! -f "$SLOT_DIR/.claude/commands/git-commit.md" ]; then
        mkdir -p "$SLOT_DIR/.claude/commands"
        cp "$HOME/.claude/commands/git-commit.md" "$SLOT_DIR/.claude/commands/git-commit.md"
    fi

    # Copy design docs
    for doc in intended-entity-state-machines.md as-built-state-machines.md; do
        SRC="$SOURCE_DIR/qictrader-backend-rs/docs/$doc"
        DST="$SLOT_DIR/qictrader-backend-rs/docs/$doc"
        if [ -f "$SRC" ] && [ ! -f "$DST" ]; then
            mkdir -p "$(dirname "$DST")"
            cp "$SRC" "$DST"
        fi
    done

    rm -f "$SLOT_DIR/.current-ticket"
    rm -rf "$SLOT_DIR/ticket-plans"
done

# ---------------------------------------------------------------------------
# Step 2: Create tmux session and launch clauded
# ---------------------------------------------------------------------------

tmux kill-session -t "$SESSION" 2>/dev/null || true
tmux new-session -d -s "$SESSION"
tmux set-option -t "$SESSION" history-limit 50000

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    SLOT_DIR="$BASE_DIR/slot-$SLOT"
    if [ "$i" -eq 0 ]; then
        tmux rename-window -t "$SESSION:0" "slot-$SLOT"
    else
        tmux new-window -t "$SESSION" -n "slot-$SLOT"
    fi
    tmux send-keys -t "$SESSION:slot-$SLOT" "cd '$SLOT_DIR'" Enter
done

log "Launching clauded in all slots..."

SLOT_OFFSET_BASE="${QIC_SLOT_OFFSET_BASE:-0}"

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    OFFSET=$((SLOT_OFFSET_BASE + SLOT))
    tmux send-keys -t "$SESSION:slot-$SLOT" \
        "export QIC_SLOT_OFFSET=$OFFSET && clauded" Enter
done

log "Waiting ${STARTUP_WAIT}s for Claude Code to start..."
sleep "$STARTUP_WAIT"

# Press Enter in each slot to dismiss any startup prompt
for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    tmux send-keys -t "$SESSION:slot-$SLOT" Enter
done
sleep 8

# ---------------------------------------------------------------------------
# Step 3: Record baselines BEFORE sending /ticket
# ---------------------------------------------------------------------------

# Baseline counts (indexed arrays, index 0 unused)
BL_TICKET_DONE=("" )
BL_TICKET_Q=("" )
BL_TEMPER_DONE=("" )
BL_COMMIT_DONE=("" )
BL_GOLIVE_DONE=("" )

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    W="slot-$SLOT"
    BL_TICKET_DONE[$SLOT]=$(count_in_scrollback "$W" "$TICKET_DONE_PAT")
    BL_TICKET_Q[$SLOT]=$(count_in_scrollback "$W" "$TICKET_QUESTION_PAT")
    BL_TEMPER_DONE[$SLOT]=$(count_in_scrollback "$W" "$TEMPER_DONE_PAT")
    BL_COMMIT_DONE[$SLOT]=$(count_in_scrollback "$W" "$COMMIT_DONE_PAT")
    BL_GOLIVE_DONE[$SLOT]=$(count_in_scrollback "$W" "$GOLIVE_DONE_PAT")
done

# ---------------------------------------------------------------------------
# Step 4: Send /ticket to all slots
# ---------------------------------------------------------------------------

log "Sending /ticket commands..."

SLOT_STATE=("" )
SLOT_TICKET=("" )
SLOT_CMD_TIME=("" )
SLOT_NUDGES=("" )

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    TICKET="${TICKETS[$i]}"

    send_command "slot-$SLOT" "/ticket $TICKET"
    SLOT_STATE[$SLOT]="ticket"
    SLOT_TICKET[$SLOT]="$TICKET"
    SLOT_CMD_TIME[$SLOT]=$(date +%s)
    SLOT_NUDGES[$SLOT]=0

    log "  Slot $SLOT: /ticket $TICKET (baseline done=${BL_TICKET_DONE[$SLOT]} q=${BL_TICKET_Q[$SLOT]})"
    sleep 2
done

# ---------------------------------------------------------------------------
# Step 5: Monitor and auto-chain commands
# ---------------------------------------------------------------------------

log ""
log "Monitoring all slots (poll every ${POLL_INTERVAL}s)..."
log "Attach to watch: tmux attach -t $SESSION"
log ""

all_done() {
    for i in "${!TICKETS[@]}"; do
        SLOT=$((i + 1))
        state="${SLOT_STATE[SLOT]}"
        if [ "$state" != "done" ] && [ "$state" != "parked" ] && [ "$state" != "failed" ] && [ "$state" != "nowork" ]; then
            return 1
        fi
    done
    return 0
}

TOTAL_WAIT=0
MAX_TOTAL_WAIT=14400

while ! all_done && [ $TOTAL_WAIT -lt $MAX_TOTAL_WAIT ]; do
    sleep "$POLL_INTERVAL"
    TOTAL_WAIT=$((TOTAL_WAIT + POLL_INTERVAL))

    for i in "${!TICKETS[@]}"; do
        SLOT=$((i + 1))
        state="${SLOT_STATE[SLOT]}"
        ticket="${SLOT_TICKET[SLOT]}"
        window="slot-$SLOT"

        # Skip finished slots
        if [ "$state" = "done" ] || [ "$state" = "parked" ] || [ "$state" = "failed" ] || [ "$state" = "nowork" ]; then
            continue
        fi

        # Enforce minimum wait
        now=$(date +%s)
        cmd_time="${SLOT_CMD_TIME[SLOT]}"
        elapsed=$((now - cmd_time))
        min_wait=$(min_wait_for_phase "$state")

        if [ "$elapsed" -lt "$min_wait" ]; then
            continue
        fi

        # Must be idle (not actively processing)
        if ! is_claude_idle "$window"; then
            continue
        fi

        # Double-check idle after 10 seconds
        sleep 10
        if ! is_claude_idle "$window"; then
            continue
        fi

        # --- Claude is idle. Check scrollback for completion. ---

        case "$state" in
            ticket)
                cur_done=$(count_in_scrollback "$window" "$TICKET_DONE_PAT")
                cur_q=$(count_in_scrollback "$window" "$TICKET_QUESTION_PAT")
                bl_done="${BL_TICKET_DONE[SLOT]}"
                bl_q="${BL_TICKET_Q[SLOT]}"

                if [ "$cur_done" -gt "$bl_done" ]; then
                    log "Slot $SLOT: /ticket COMPLETE ($ticket) [count: $bl_done->$cur_done]"

                    if has_code_changes "$SLOT"; then
                        log "Slot $SLOT: Code changes detected -> sending /temper"
                        BL_TEMPER_DONE[$SLOT]=$(count_in_scrollback "$window" "$TEMPER_DONE_PAT")
                        send_command "$window" "/temper"
                        SLOT_STATE[SLOT]="temper"
                        SLOT_CMD_TIME[SLOT]=$(date +%s)
                    else
                        log "NO-WORK: Slot $SLOT -- $ticket (no code changes)"
                        SLOT_STATE[SLOT]="nowork"
                    fi

                elif [ "$cur_q" -gt "$bl_q" ]; then
                    nudges="${SLOT_NUDGES[SLOT]}"
                    if [ "$nudges" -lt 1 ]; then
                        log "Slot $SLOT: questions detected -- auto-replying 'proceed' (nudge $((nudges+1))) [count: $bl_q->$cur_q]"
                        send_command "$window" "Proceed with your best judgment. Pick the simplest, safest approach for any ambiguities. Follow the intent doc. Do not ask more questions - just implement."
                        SLOT_NUDGES[SLOT]=$((nudges + 1))
                        BL_TICKET_Q[SLOT]=$cur_q
                        SLOT_CMD_TIME[SLOT]=$(date +%s)
                    else
                        log "PARKED: Slot $SLOT -- $ticket (genuine question after nudge) [count: $bl_q->$cur_q]"
                        SLOT_STATE[SLOT]="parked"
                        {
                            echo ""
                            echo "---"
                            echo "## $ticket"
                            echo "**Machine:** $(hostname) | **Slot:** slot-$SLOT"
                            echo "**Date:** $(date '+%Y-%m-%d %H:%M')"
                            echo "**Status:** AWAITING BUSINESS ANSWER"
                            echo ""
                            echo "### Questions"
                            get_pane_content "$window" | \
                                sed -n '/[Qq]uestion/,/^[^[:space:]]/p' | \
                                grep -v -E 'Fetch\(|https://|key=|token=|Received|^$' | \
                                tail -20
                            echo ""
                        } >> "$QUESTIONS_FILE"
                    fi

                else
                    log "  Slot $SLOT: /ticket -- idle but no completion marker yet (${elapsed}s, done=$cur_done q=$cur_q)"
                fi
                ;;

            temper)
                cur_done=$(count_in_scrollback "$window" "$TEMPER_DONE_PAT")
                bl_done="${BL_TEMPER_DONE[SLOT]}"

                if [ "$cur_done" -gt "$bl_done" ]; then
                    # Check if BLOCKED
                    blocked=$(count_in_scrollback "$window" "VERDICT:.*BLOCKED")
                    if [ "$blocked" -gt 0 ]; then
                        log "BLOCKED: Slot $SLOT -- $ticket (/temper BLOCKED)"
                        SLOT_STATE[SLOT]="failed"
                    else
                        log "Slot $SLOT: /temper COMPLETE -> sending /git-commit [count: $bl_done->$cur_done]"
                        BL_COMMIT_DONE[$SLOT]=$(count_in_scrollback "$window" "$COMMIT_DONE_PAT")
                        send_command "$window" "/git-commit"
                        SLOT_STATE[SLOT]="commit"
                        SLOT_CMD_TIME[SLOT]=$(date +%s)
                    fi
                else
                    log "  Slot $SLOT: /temper -- idle but no VERDICT yet (${elapsed}s, count=$cur_done)"
                fi
                ;;

            commit)
                cur_done=$(count_in_scrollback "$window" "$COMMIT_DONE_PAT")
                bl_done="${BL_COMMIT_DONE[SLOT]}"

                if [ "$cur_done" -gt "$bl_done" ]; then
                    log "Slot $SLOT: /git-commit COMPLETE -> sending /golive [count: $bl_done->$cur_done]"
                    BL_GOLIVE_DONE[$SLOT]=$(count_in_scrollback "$window" "$GOLIVE_DONE_PAT")
                    send_command "$window" "/golive"
                    SLOT_STATE[SLOT]="golive"
                    SLOT_CMD_TIME[SLOT]=$(date +%s)
                else
                    log "  Slot $SLOT: /git-commit -- waiting (${elapsed}s, count=$cur_done)"
                fi
                ;;

            golive)
                cur_done=$(count_in_scrollback "$window" "$GOLIVE_DONE_PAT")
                bl_done="${BL_GOLIVE_DONE[SLOT]}"

                if [ "$cur_done" -gt "$bl_done" ]; then
                    log "DEPLOYED: Slot $SLOT -- $ticket [count: $bl_done->$cur_done]"
                    SLOT_STATE[SLOT]="done"
                else
                    log "  Slot $SLOT: /golive -- waiting (${elapsed}s, count=$cur_done)"
                fi
                ;;
        esac
    done
done

# ---------------------------------------------------------------------------
# Step 6: Summary
# ---------------------------------------------------------------------------

log ""
log "============================================"
log "  Batch Complete"
log "============================================"

DEPLOYED=0; PARKED=0; FAILED=0; NOWORK=0

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    state="${SLOT_STATE[SLOT]}"
    ticket="${SLOT_TICKET[SLOT]}"

    # Save slot scrollback to log file
    get_pane_content "slot-$SLOT" > "$LOG_DIR/slot-${SLOT}.log" 2>/dev/null || true

    case "$state" in
        done)   log "  OK  $SLOT: DEPLOYED -- $ticket"; DEPLOYED=$((DEPLOYED + 1)) ;;
        parked) log "  ??  $SLOT: PARKED  -- $ticket";  PARKED=$((PARKED + 1)) ;;
        nowork) log "  --  $SLOT: NO-WORK -- $ticket";  NOWORK=$((NOWORK + 1)) ;;
        failed) log "  XX  $SLOT: FAILED  -- $ticket";  FAILED=$((FAILED + 1)) ;;
        *)      log "  ~~  $SLOT: TIMEOUT -- $ticket ($state)"; FAILED=$((FAILED + 1)) ;;
    esac
done

log ""
log "Deployed: $DEPLOYED | Parked: $PARKED | No-work: $NOWORK | Failed: $FAILED"

# Append to summary (never overwrite)
{
    echo ""
    echo "---"
    echo ""
    echo "## Batch Run -- $(hostname) -- $(date '+%Y-%m-%d %H:%M')"
    echo "**Tickets:** ${#TICKETS[@]}"
    echo ""
    echo "| Slot | Ticket | Result |"
    echo "|------|--------|--------|"
    for i in "${!TICKETS[@]}"; do
        SLOT=$((i + 1))
        state="${SLOT_STATE[SLOT]}"
        ticket="${SLOT_TICKET[SLOT]}"
        case "$state" in
            done)   result="DEPLOYED" ;;
            parked) result="PARKED (questions)" ;;
            nowork) result="NO-WORK" ;;
            failed) result="FAILED" ;;
            *)      result="TIMEOUT ($state)" ;;
        esac
        echo "| $SLOT | $ticket | $result |"
    done
    echo ""
    echo "**Deployed:** $DEPLOYED | **Parked:** $PARKED | **No-work:** $NOWORK | **Failed:** $FAILED"
} >> "$SUMMARY_FILE"

log ""
log "Summary: $SUMMARY_FILE"
log "Questions: $QUESTIONS_FILE"
log "Batch finished. tmux session '$SESSION' kept alive."
log "Kill with: tmux kill-session -t $SESSION"
