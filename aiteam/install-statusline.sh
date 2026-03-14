#!/usr/bin/env bash
set -euo pipefail

# Installs the Claude Code status line showing folder, context usage bar, and model.
# Run once per machine: bash install-statusline.sh

CLAUDE_DIR="$HOME/.claude"
SCRIPT="$CLAUDE_DIR/statusline-command.sh"
SETTINGS="$CLAUDE_DIR/settings.json"

mkdir -p "$CLAUDE_DIR"

# --- Write the statusline script ---
cat > "$SCRIPT" <<'STATUSLINE'
#!/bin/bash
# Claude Code status line - shows folder, context usage bar, and model
# Usage:
#   Render status line (called by Claude Code, reads JSON from stdin):
#     bash statusline-command.sh
#   Install for current user (safe - won't clobber existing settings):
#     bash statusline-command.sh --install

set -euo pipefail

SCRIPT_NAME="statusline-command.sh"

if [ "${1:-}" = "--install" ]; then
  CLAUDE_DIR="$HOME/.claude"
  SETTINGS="$CLAUDE_DIR/settings.json"
  DEST="$CLAUDE_DIR/$SCRIPT_NAME"

  mkdir -p "$CLAUDE_DIR"

  # Copy script to ~/.claude/
  SELF="$(readlink -f "$0")"
  if [ "$SELF" != "$DEST" ]; then
    cp "$SELF" "$DEST"
    chmod +x "$DEST"
    echo "Copied $SCRIPT_NAME to $DEST"
  fi

  # Merge statusLine into settings.json without clobbering other keys
  STATUSLINE_CMD="bash $DEST"
  if [ -f "$SETTINGS" ]; then
    UPDATED=$(jq --arg cmd "$STATUSLINE_CMD" '.statusLine = {"type": "command", "command": $cmd}' "$SETTINGS")
    echo "$UPDATED" > "$SETTINGS"
    echo "Updated statusLine in $SETTINGS"
  else
    jq -n --arg cmd "$STATUSLINE_CMD" '{"statusLine": {"type": "command", "command": $cmd}}' > "$SETTINGS"
    echo "Created $SETTINGS with statusLine config"
  fi

  echo "Done. Restart Claude Code to see the new status line."
  exit 0
fi

# --- Render status line (reads JSON from stdin) ---
input=$(cat)
cwd=$(echo "$input" | jq -r '.cwd')
model=$(echo "$input" | jq -r '.model.display_name // "unknown"')
pct=$(echo "$input" | jq -r '.context_window.used_percentage // 0' | cut -d. -f1)

# Progress bar
BAR_WIDTH=20
FILLED=$((pct * BAR_WIDTH / 100))

# Compaction happens around 80-95% context usage
COMPACT_POS=$((80 * BAR_WIDTH / 100))  # position 16 of 20

bar=""
for ((i=0; i<BAR_WIDTH; i++)); do
  if [ "$i" -eq "$COMPACT_POS" ]; then
    # Compaction marker (magenta pipe)
    bar="${bar}\033[01;35m|\033[00m"
  elif [ "$i" -lt "$FILLED" ]; then
    # Filled portion - color based on usage
    if [ "$pct" -lt 70 ]; then
      bar="${bar}\033[01;32m▓\033[00m"
    elif [ "$pct" -lt 90 ]; then
      bar="${bar}\033[01;33m▓\033[00m"
    else
      bar="${bar}\033[01;31m▓\033[00m"
    fi
  else
    bar="${bar}\033[0;37m░\033[00m"
  fi
done

# Single line: folder | context bar | model
printf '\033[01;34m%s\033[00m [%b] %d%% \033[0;36m%s\033[00m' "$cwd" "$bar" "$pct" "$model"
STATUSLINE

chmod +x "$SCRIPT"
echo "  statusline script -> $SCRIPT"

# --- Merge statusLine into settings.json ---
STATUSLINE_CMD="bash $SCRIPT"
if [ -f "$SETTINGS" ]; then
  UPDATED=$(jq --arg cmd "$STATUSLINE_CMD" '.statusLine = {"type": "command", "command": $cmd}' "$SETTINGS")
  echo "$UPDATED" > "$SETTINGS"
  echo "  merged statusLine into existing $SETTINGS"
else
  jq -n --arg cmd "$STATUSLINE_CMD" '{"statusLine": {"type": "command", "command": $cmd}}' > "$SETTINGS"
  echo "  created $SETTINGS"
fi

# --- Check jq is available ---
if ! command -v jq &>/dev/null; then
  echo ""
  echo "WARNING: jq is not installed. The statusline needs it."
  echo "  Ubuntu/Debian:  sudo apt install jq"
  echo "  macOS:          brew install jq"
fi

echo ""
echo "Done. Restart Claude Code to see the statusline."
