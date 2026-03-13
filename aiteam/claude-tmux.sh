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
  "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 TRELLO_API_KEY=d0f2319aeb29e279616c592d79677692 TRELLO_TOKEN=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0 claude --dangerously-skip-permissions"
