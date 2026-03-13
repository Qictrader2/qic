#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# run-tickets.sh — Run N tickets in parallel Claude Code instances via tmux
#
# Usage:
#   ./scripts/run-tickets.sh "TICKET-1 name" "TICKET-2 name" ...
#
# Each ticket gets its own isolated clone directory and tmux window.
# ---------------------------------------------------------------------------

BASE_DIR="${QIC_PARALLEL_DIR:-/tmp/qic-parallel}"
SESSION="qic-tickets"
STARTUP_WAIT="${CLAUDE_STARTUP_WAIT:-12}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SOURCE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

REMOTE_ROOT="https://github.com/Qictrader2/qic.git"
REMOTE_FRONTEND="https://github.com/Qictrader2/Frontend.git"
REMOTE_BACKEND="https://github.com/Qictrader2/qictrader-backend-rs.git"
REMOTE_HEROKU="https://git.heroku.com/qictrader-backend-rs.git"

TICKETS=("$@")

if [ ${#TICKETS[@]} -eq 0 ]; then
    echo "Usage: $0 'TICKET-1 name' 'TICKET-2 name' ..."
    echo ""
    echo "Example:"
    echo "  $0 'ES-004: Dispute — Escrow Freeze' 'ES-005: Moderator Resolution Dispatch'"
    echo ""
    echo "Environment variables:"
    echo "  QIC_PARALLEL_DIR    Base directory for clones (default: /tmp/qic-parallel)"
    echo "  CLAUDE_STARTUP_WAIT Seconds to wait for Claude REPL to start (default: 12)"
    exit 1
fi

echo "============================================"
echo "  QIC Parallel Ticket Runner"
echo "============================================"
echo "Source:   $SOURCE_DIR"
echo "Clones:   $BASE_DIR"
echo "Tickets:  ${#TICKETS[@]}"
echo "Session:  $SESSION"
echo ""

# ---------------------------------------------------------------------------
# Step 1: Create clone directories
# ---------------------------------------------------------------------------

mkdir -p "$BASE_DIR"

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    SLOT_DIR="$BASE_DIR/slot-$SLOT"
    TICKET="${TICKETS[$i]}"

    echo "--- Slot $SLOT: $TICKET ---"

    if [ -d "$SLOT_DIR/.git" ]; then
        echo "  Clone exists, pulling latest..."
        (cd "$SLOT_DIR" && git pull --rebase 2>/dev/null || true)
        (cd "$SLOT_DIR/Frontend" && git pull --rebase 2>/dev/null || true)
        (cd "$SLOT_DIR/qictrader-backend-rs" && git pull --rebase 2>/dev/null || true)
    else
        echo "  Creating fresh clone..."
        rm -rf "$SLOT_DIR"

        git clone --quiet "$SOURCE_DIR" "$SLOT_DIR"

        rm -rf "$SLOT_DIR/Frontend"
        git clone --quiet "$SOURCE_DIR/Frontend" "$SLOT_DIR/Frontend"

        rm -rf "$SLOT_DIR/qictrader-backend-rs"
        git clone --quiet "$SOURCE_DIR/qictrader-backend-rs" "$SLOT_DIR/qictrader-backend-rs"

        (cd "$SLOT_DIR" && git remote set-url origin "$REMOTE_ROOT")
        (cd "$SLOT_DIR/Frontend" && git remote set-url origin "$REMOTE_FRONTEND")
        (cd "$SLOT_DIR/qictrader-backend-rs" && git remote set-url origin "$REMOTE_BACKEND")
        (cd "$SLOT_DIR/qictrader-backend-rs" && git remote add heroku "$REMOTE_HEROKU" 2>/dev/null || true)

        # Symlink for slash command compatibility
        (cd "$SLOT_DIR" && ln -sf Frontend frontend 2>/dev/null || true)
    fi

    # Ensure git-commit.md is available in project commands
    if [ -f "$HOME/.claude/commands/git-commit.md" ] && [ ! -f "$SLOT_DIR/.claude/commands/git-commit.md" ]; then
        mkdir -p "$SLOT_DIR/.claude/commands"
        cp "$HOME/.claude/commands/git-commit.md" "$SLOT_DIR/.claude/commands/git-commit.md"
        echo "  Copied git-commit.md to project commands"
    fi

    # Copy design intent docs so CLARIFY step can self-answer
    INTENT_DOC="$SLOT_DIR/qictrader-backend-rs/docs/intended-entity-state-machines.md"
    if [ ! -f "$INTENT_DOC" ]; then
        SRC_INTENT="$SOURCE_DIR/qictrader-backend-rs/docs/intended-entity-state-machines.md"
        if [ -f "$SRC_INTENT" ]; then
            mkdir -p "$SLOT_DIR/qictrader-backend-rs/docs"
            cp "$SRC_INTENT" "$INTENT_DOC"
        fi
        SRC_ASBUILT="$SOURCE_DIR/qictrader-backend-rs/docs/as-built-state-machines.md"
        if [ -f "$SRC_ASBUILT" ]; then
            cp "$SRC_ASBUILT" "$SLOT_DIR/qictrader-backend-rs/docs/as-built-state-machines.md"
        fi
        echo "  Copied design intent docs for self-answering"
    fi

    # Clean any leftover ticket artifacts from a previous run
    rm -f "$SLOT_DIR/.current-ticket"
    rm -rf "$SLOT_DIR/ticket-plans"

    echo "  Ready: $SLOT_DIR"
done

echo ""

# ---------------------------------------------------------------------------
# Step 2: Create tmux session
# ---------------------------------------------------------------------------

tmux kill-session -t "$SESSION" 2>/dev/null || true
tmux new-session -d -s "$SESSION"

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    SLOT_DIR="$BASE_DIR/slot-$SLOT"
    TICKET="${TICKETS[$i]}"

    if [ "$i" -eq 0 ]; then
        tmux rename-window -t "$SESSION:0" "slot-$SLOT"
    else
        tmux new-window -t "$SESSION" -n "slot-$SLOT"
    fi

    tmux send-keys -t "$SESSION:slot-$SLOT" "cd '$SLOT_DIR'" Enter
done

echo "Tmux session '$SESSION' created with ${#TICKETS[@]} windows."
echo ""

# ---------------------------------------------------------------------------
# Step 3: Pull latest + launch clauded in all windows
# ---------------------------------------------------------------------------

echo "Pulling latest and launching Claude Code in all slots..."

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    SLOT_DIR="$BASE_DIR/slot-$SLOT"

    # Set QIC_SLOT_OFFSET so migrations get unique version numbers per slot
    tmux send-keys -t "$SESSION:slot-$SLOT" \
        "export QIC_SLOT_OFFSET=$SLOT && git pull --rebase 2>/dev/null; cd Frontend && git pull --rebase 2>/dev/null; cd ../qictrader-backend-rs && git pull --rebase 2>/dev/null; cd .. && clauded" Enter
done

echo "Waiting ${STARTUP_WAIT}s for Claude Code to start in all slots..."
sleep "$STARTUP_WAIT"

# Accept the workspace trust prompt ("Yes, I trust this folder") if present
echo "Accepting workspace trust prompts (if any)..."
for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    tmux send-keys -t "$SESSION:slot-$SLOT" Enter
done
sleep 8

# ---------------------------------------------------------------------------
# Step 4: Send /ticket commands
# ---------------------------------------------------------------------------

echo "Sending /ticket commands..."
echo ""

for i in "${!TICKETS[@]}"; do
    SLOT=$((i + 1))
    TICKET="${TICKETS[$i]}"

    tmux send-keys -t "$SESSION:slot-$SLOT" "/ticket $TICKET" Enter
    echo "  Slot $SLOT: /ticket $TICKET"
done

echo ""
echo "============================================"
echo "  All ${#TICKETS[@]} tickets launched!"
echo "============================================"
echo ""
echo "Attach to the session:"
echo "  tmux attach -t $SESSION"
echo ""
echo "Navigate windows:"
echo "  Ctrl+B then n     — next window"
echo "  Ctrl+B then p     — previous window"
echo "  Ctrl+B then 0-4   — jump to window"
echo "  Ctrl+B then w     — list windows"
echo "  Ctrl+B then d     — detach (keeps running)"
echo ""
echo "Questions file (append-only, never cleared):"
echo "  $SOURCE_DIR/ticket_questions.md"
echo ""
echo "Migration collision prevention:"
echo "  Each slot has QIC_SLOT_OFFSET=N set (1-5)."
echo "  Migrations use offset in the seconds field to avoid collisions."
echo ""
echo "When /ticket completes in a window:"
echo ""
echo "  QUESTIONS?  The CLARIFY step now auto-checks design docs first:"
echo "              - intended-entity-state-machines.md"
echo "              - as-built-state-machines.md"
echo "              If the doc answers the question → proceeds automatically."
echo "              If still unanswered → append to ticket_questions.md"
echo "              (ticket #, context, questions, doc sections checked)."
echo "              Ctrl+C to kill instance. DO NOT answer — needs business input."
echo ""
echo "  NO QUESTIONS?  Continue the pipeline without any confirmations:"
echo "              /temper → /git-commit → /golive"
echo "              Each step runs fully autonomous. Just type the next command"
echo "              when the previous one finishes."
echo ""
