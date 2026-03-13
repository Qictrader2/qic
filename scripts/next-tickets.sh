#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPS_JSON="$SCRIPT_DIR/../ticket-dependencies.json"
DONE_FILE="$SCRIPT_DIR/../.tickets-done"

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Show the next batch of tickets that can be worked in parallel.

Options:
  -n NUM    Max tickets to show (default: all available)
  -d DOMAIN Filter by domain (e.g. Auth, Trade, Escrow, Wallet)
  -w        Show which wave each ticket belongs to
  -l        List all done tickets
  -m ID     Mark ticket(s) as done (comma-separated)
  -u ID     Unmark ticket(s) (comma-separated)
  -r        Reset — clear all done tickets
  -s        Summary: show progress per wave
  -h        This help

Examples:
  $(basename "$0")              # show all unblocked tickets
  $(basename "$0") -n 5         # show top 5
  $(basename "$0") -d Escrow    # only escrow tickets
  $(basename "$0") -m ES-006    # mark ES-006 done
  $(basename "$0") -m ES-006,AUTH-001,WS-001  # mark several done
  $(basename "$0") -s           # progress summary
EOF
  exit 0
}

touch "$DONE_FILE"

done_tickets() {
  grep -v '^#' "$DONE_FILE" 2>/dev/null | grep -v '^\s*$' | sort -u
}

is_done() {
  grep -qx "$1" "$DONE_FILE" 2>/dev/null
}

mark_done() {
  local ids
  IFS=',' read -ra ids <<< "$1"
  for id in "${ids[@]}"; do
    id="$(echo "$id" | tr -d ' ')"
    if is_done "$id"; then
      echo "  already done: $id"
    else
      echo "$id" >> "$DONE_FILE"
      echo "  ✓ $id"
    fi
  done
}

unmark() {
  local ids
  IFS=',' read -ra ids <<< "$1"
  for id in "${ids[@]}"; do
    id="$(echo "$id" | tr -d ' ')"
    local tmp
    tmp=$(mktemp)
    grep -vx "$id" "$DONE_FILE" > "$tmp" || true
    mv "$tmp" "$DONE_FILE"
    echo "  ✗ $id unmarked"
  done
}

reset_all() {
  > "$DONE_FILE"
  echo "All tickets reset."
  exit 0
}

list_done() {
  local count
  count=$(done_tickets | wc -l)
  if [ "$count" -eq 0 ]; then
    echo "No tickets marked done yet."
  else
    echo "Done ($count):"
    done_tickets | while read -r t; do
      local name domain wave
      name=$(jq -r ".tickets[\"$t\"].name // \"?\"" "$DEPS_JSON")
      domain=$(jq -r ".tickets[\"$t\"].domain // \"?\"" "$DEPS_JSON")
      wave=$(jq -r ".tickets[\"$t\"].wave // \"?\"" "$DEPS_JSON")
      printf "  %-14s  W%s  %-12s  %s\n" "$t" "$wave" "[$domain]" "$name"
    done
  fi
  exit 0
}

summary() {
  local total_done
  total_done=$(done_tickets | wc -l)
  local total
  total=$(jq '.tickets | length' "$DEPS_JSON")
  echo "Progress: $total_done / $total tickets done"
  echo ""
  printf "  %-8s %-28s %6s %6s %6s\n" "Wave" "Name" "Done" "Total" "%"
  printf "  %-8s %-28s %6s %6s %6s\n" "----" "----" "----" "-----" "--"
  for w in 0 1 2 3 4 5 6 7 8; do
    local wname wtickets wtotal wdone pct
    wname=$(jq -r ".waves[\"$w\"].name" "$DEPS_JSON")
    wtotal=$(jq -r ".waves[\"$w\"].ticketCount" "$DEPS_JSON")
    wdone=0
    while IFS= read -r tid; do
      if is_done "$tid"; then
        wdone=$((wdone + 1))
      fi
    done < <(jq -r ".waves[\"$w\"].tickets[]" "$DEPS_JSON")
    if [ "$wtotal" -gt 0 ]; then
      pct=$((wdone * 100 / wtotal))
    else
      pct=0
    fi
    printf "  W%-7s %-28s %6d %6d  %3d%%\n" "$w" "$wname" "$wdone" "$wtotal" "$pct"
  done
  echo ""
  exit 0
}

MAX=""
DOMAIN=""
SHOW_WAVE=false

while getopts "n:d:m:u:wlrsh" opt; do
  case $opt in
    n) MAX="$OPTARG" ;;
    d) DOMAIN="$OPTARG" ;;
    m) mark_done "$OPTARG"; exit 0 ;;
    u) unmark "$OPTARG"; exit 0 ;;
    w) SHOW_WAVE=true ;;
    l) list_done ;;
    r) reset_all ;;
    s) summary ;;
    h) usage ;;
    *) usage ;;
  esac
done

# Build the set of done ticket IDs as a jq-friendly array
DONE_ARR="[]"
if [ -s "$DONE_FILE" ]; then
  DONE_ARR=$(done_tickets | jq -R . | jq -s .)
fi

# Find all tickets whose dependencies are fully satisfied
AVAILABLE=$(jq -r --argjson done "$DONE_ARR" '
  .tickets | to_entries[]
  | select(.key as $k | ($done | index($k)) == null)
  | select(.value.dependsOn | length == 0 or all(. as $dep | $done | index($dep) != null))
  | [.key, .value.name, .value.domain, (.value.wave | tostring)]
  | @tsv
' "$DEPS_JSON" | sort -t$'\t' -k4,4n -k3,3 -k1,1)

if [ -n "$DOMAIN" ]; then
  AVAILABLE=$(echo "$AVAILABLE" | awk -F'\t' -v d="$DOMAIN" 'tolower($3) == tolower(d)')
fi

COUNT=$(echo "$AVAILABLE" | grep -c . || true)

if [ "$COUNT" -eq 0 ]; then
  if [ -n "$DOMAIN" ]; then
    echo "No unblocked tickets in domain '$DOMAIN'."
  else
    echo "No unblocked tickets (all done or graph is empty)."
  fi
  exit 0
fi

if [ -n "$MAX" ]; then
  AVAILABLE=$(echo "$AVAILABLE" | head -n "$MAX")
  echo "Next $MAX unblocked tickets (of $COUNT available):"
else
  echo "All $COUNT unblocked tickets:"
fi
echo ""

if $SHOW_WAVE; then
  printf "  %-14s  %-4s  %-12s  %s\n" "TICKET" "WAVE" "DOMAIN" "NAME"
  printf "  %-14s  %-4s  %-12s  %s\n" "------" "----" "------" "----"
  echo "$AVAILABLE" | while IFS=$'\t' read -r id name domain wave; do
    printf "  %-14s  W%-3s  %-12s  %s\n" "$id" "$wave" "[$domain]" "$name"
  done
else
  printf "  %-14s  %-12s  %s\n" "TICKET" "DOMAIN" "NAME"
  printf "  %-14s  %-12s  %s\n" "------" "------" "----"
  echo "$AVAILABLE" | while IFS=$'\t' read -r id name domain wave; do
    printf "  %-14s  %-12s  %s\n" "$id" "[$domain]" "$name"
  done
fi
