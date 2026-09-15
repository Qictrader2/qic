#!/usr/bin/env bash
set -euo pipefail

SESSION="qic"
REPO="$HOME/git/qic"

if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Killing existing tmux session '$SESSION'..."
  tmux kill-session -t "$SESSION"
fi

echo "Launching QIC AI Team..."

tmux new-session -s "$SESSION" -c "$REPO" \
  "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 claude --dangerously-skip-permissions"
