#!/usr/bin/env bash
set -euo pipefail

# Installs the Claude Code status line showing model, context usage bar, and cwd.
# Run once per machine: bash aiteam/install-statusline.sh

CLAUDE_DIR="$HOME/.claude"
SCRIPT="$CLAUDE_DIR/statusline-command.sh"
SETTINGS="$CLAUDE_DIR/settings.json"

mkdir -p "$CLAUDE_DIR"

# --- Write the statusline script ---
cat > "$SCRIPT" <<'EOF'
#!/bin/bash
input=$(cat)
cwd=$(echo "$input" | jq -r '.cwd')
model=$(echo "$input" | jq -r '.model.display_name // "unknown"')
pct=$(echo "$input" | jq -r '.context_window.used_percentage // 0' | cut -d. -f1)

BAR_WIDTH=20
FILLED=$((pct * BAR_WIDTH / 100))
COMPACT_POS=$((80 * BAR_WIDTH / 100))

bar=""
for ((i=0; i<BAR_WIDTH; i++)); do
  if [ "$i" -eq "$COMPACT_POS" ]; then
    bar="${bar}\033[01;35m|\033[00m"
  elif [ "$i" -lt "$FILLED" ]; then
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

printf '\033[0;36m%s\033[00m [%b] %d%% \033[01;34m%s\033[00m' "$model" "$bar" "$pct" "$cwd"
EOF

chmod +x "$SCRIPT"
echo "  statusline script -> $SCRIPT"

# --- Merge statusLine into settings.json ---
if [ -f "$SETTINGS" ]; then
  # File exists - merge in the statusLine key, preserving everything else
  tmp=$(mktemp)
  jq '. + {"statusLine": {"type": "command", "command": "bash '"$SCRIPT"'"}}' "$SETTINGS" > "$tmp"
  mv "$tmp" "$SETTINGS"
  echo "  merged statusLine into existing $SETTINGS"
else
  # No settings file yet - create it
  cat > "$SETTINGS" <<ENDJSON
{
  "statusLine": {
    "type": "command",
    "command": "bash $SCRIPT"
  }
}
ENDJSON
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
