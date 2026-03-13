#!/usr/bin/env bash
set -euo pipefail

SESSION="qic"
REPO="$HOME/git/qic"
WA="$HOME/git/qic-worker-a"
WB="$HOME/git/qic-worker-b"
WC="$HOME/git/qic-worker-c"

# Fall back to main repo if worktrees haven't been created yet
[[ -d "$WA" ]] || WA="$REPO"
[[ -d "$WB" ]] || WB="$REPO"
[[ -d "$WC" ]] || WC="$REPO"

# Kill stale session
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Killing existing tmux session '$SESSION'..."
  tmux kill-session -t "$SESSION"
fi

echo "Launching QIC AI Team..."

# --- Build 2x2 grid ---
# After step 1: pane 0 = full window
tmux new-session -d -s "$SESSION" -c "$REPO" \
  -e "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1" \
  -e "TRELLO_API_KEY=d0f2319aeb29e279616c592d79677692" \
  -e "TRELLO_TOKEN=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0"

# After step 2: 0=left-half, 1=right-half
tmux split-window -h -t "$SESSION:0.0" -c "$WA"

# After step 3: 0=top-left, 2=bot-left, 1=right-half
tmux split-window -v -t "$SESSION:0.0" -c "$WB"

# After step 4: 0=top-left, 2=bot-left, 1=top-right, 3=bot-right
tmux split-window -v -t "$SESSION:0.1" -c "$WC"

# --- Pane titles ---
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format \
  "#{?pane_active,#[bold#,fg=colour255],#[fg=colour245]} #{pane_title} "

tmux select-pane -t "$SESSION:0.0" -T "  LEAD"
tmux select-pane -t "$SESSION:0.1" -T "  Agent A"
tmux select-pane -t "$SESSION:0.2" -T "  Agent B"
tmux select-pane -t "$SESSION:0.3" -T "  Agent C"

# --- Pane colors ---
# Lead:     deep blue
# Agent A: deep green
# Agent B: deep purple
# Agent C: deep teal
tmux select-pane -t "$SESSION:0.0" -P "bg=colour17,fg=colour255"
tmux select-pane -t "$SESSION:0.1" -P "bg=colour22,fg=colour255"
tmux select-pane -t "$SESSION:0.2" -P "bg=colour54,fg=colour255"
tmux select-pane -t "$SESSION:0.3" -P "bg=colour23,fg=colour255"

# --- Border style ---
tmux set-option -t "$SESSION" pane-border-style        "fg=colour238"
tmux set-option -t "$SESSION" pane-active-border-style "fg=colour33,bold"

# --- Launch claude in lead pane only ---
tmux send-keys -t "$SESSION:0.0" \
  "claude --dangerously-skip-permissions" Enter

# Focus lead and attach
tmux select-pane -t "$SESSION:0.0"
tmux attach -t "$SESSION"
