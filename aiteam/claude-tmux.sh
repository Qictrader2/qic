#!/usr/bin/env bash
set -euo pipefail

SESSION="qic"
REPO="$HOME/git/qic"

# Kill stale session if it exists
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Killing existing tmux session '$SESSION'..."
  tmux kill-session -t "$SESSION"
fi

echo "Launching QIC AI Team..."
tmux new-session -s "$SESSION" -c "$REPO" "claude"
