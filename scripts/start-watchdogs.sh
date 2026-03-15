#!/usr/bin/env bash
# start-watchdogs.sh — Launch both watchdogs in the background.
#
# Usage:
#   ./scripts/start-watchdogs.sh          # start both
#   ./scripts/start-watchdogs.sh stop     # stop both
#   ./scripts/start-watchdogs.sh status   # check if running
#
# Logs:
#   /tmp/vercel-watchdog.log
#   /tmp/migration-watchdog.log

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

VERCEL_PID_FILE="/tmp/vercel-watchdog.pid"
MIG_PID_FILE="/tmp/migration-watchdog.pid"
VERCEL_LOG="/tmp/vercel-watchdog.log"
MIG_LOG="/tmp/migration-watchdog.log"

stop_watchdogs() {
  for pidfile in "$VERCEL_PID_FILE" "$MIG_PID_FILE"; do
    if [ -f "$pidfile" ]; then
      pid=$(cat "$pidfile")
      if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null
        echo "Stopped PID $pid ($(basename "$pidfile" .pid))"
      fi
      rm -f "$pidfile"
    fi
  done
}

status_watchdogs() {
  for name in vercel-watchdog migration-watchdog; do
    pidfile="/tmp/${name}.pid"
    if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
      echo "$name: RUNNING (PID $(cat "$pidfile"))"
      tail -1 "/tmp/${name}.log" 2>/dev/null | sed 's/^/  last: /'
    else
      echo "$name: STOPPED"
    fi
  done
}

case "${1:-start}" in
  stop)
    stop_watchdogs
    ;;
  status)
    status_watchdogs
    ;;
  start)
    stop_watchdogs

    echo "Starting watchdogs..."
    nohup "$SCRIPT_DIR/vercel-watchdog.sh" > "$VERCEL_LOG" 2>&1 &
    echo "  vercel-watchdog:    PID $! -> $VERCEL_LOG"

    nohup "$SCRIPT_DIR/migration-watchdog.sh" > "$MIG_LOG" 2>&1 &
    echo "  migration-watchdog: PID $! -> $MIG_LOG"

    sleep 3
    echo ""
    status_watchdogs
    ;;
  *)
    echo "Usage: $0 [start|stop|status]"
    exit 1
    ;;
esac
