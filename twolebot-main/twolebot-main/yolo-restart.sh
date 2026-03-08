#!/bin/bash
# Restart twolebot - kills the process on a specific port and starts new one
# Usage: ./yolo-restart.sh <port>
#
# This script detaches itself so it survives if the calling process dies.

# Port is required - check before detaching so the user sees the error
if [ -z "$1" ]; then
    echo "Usage: ./yolo-restart.sh <port>"
    echo "Port is required to identify which instance to restart."
    exit 1
fi

# Immediately fork to background and detach from parent process tree
if [ -z "$YOLO_DETACHED" ]; then
    if command -v setsid >/dev/null 2>&1; then
        YOLO_DETACHED=1 nohup setsid "$0" "$@" > /dev/null 2>&1 &
    else
        # macOS typically does not ship setsid; nohup background is sufficient here.
        YOLO_DETACHED=1 nohup "$0" "$@" > /dev/null 2>&1 &
    fi
    echo "Restart initiated (detached PID: $!)"
    exit 0
fi

set -e

# Strip Claude Code env vars so spawned claude CLI processes work
unset CLAUDECODE CLAUDE_CODE_ENTRYPOINT

TWOLEBOT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="$TWOLEBOT_DIR/data"
PORT="$1"
HOST="${HOST:-0.0.0.0}"
BIN="$TWOLEBOT_DIR/target/debug/twolebot"

# Optional env file (legacy/local overrides)
if [ -f "$TWOLEBOT_DIR/.env" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$TWOLEBOT_DIR/.env"
    set +a
fi

mkdir -p "$DATA_DIR"

echo "=== Restarting twolebot ===" >> "$DATA_DIR/twolebot.log"

# Kill the process listening on this port
OLD_PIDS=$(lsof -tiTCP:"$PORT" -sTCP:LISTEN 2>/dev/null | tr '\n' ' ' || true)

if [ -n "$OLD_PIDS" ]; then
    echo "Killing process(es) on port $PORT: $OLD_PIDS" >> "$DATA_DIR/twolebot.log"
    # shellcheck disable=SC2086
    kill $OLD_PIDS 2>/dev/null || true
else
    echo "No process found on port $PORT" >> "$DATA_DIR/twolebot.log"
fi

# Wait for port to be free (max 30 seconds)
echo "Waiting for port $PORT to be free..." >> "$DATA_DIR/twolebot.log"
for i in {1..60}; do
    if ! lsof -iTCP:"$PORT" -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "Port $PORT is free." >> "$DATA_DIR/twolebot.log"
        break
    fi
    sleep 0.5
done

# Final check - if port still busy, force kill anything on it
if lsof -iTCP:"$PORT" -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "Port still busy, force killing..." >> "$DATA_DIR/twolebot.log"
    lsof -tiTCP:"$PORT" -sTCP:LISTEN | xargs kill 2>/dev/null || true
    sleep 1
fi

# Start new process
echo "Starting new twolebot..." >> "$DATA_DIR/twolebot.log"
cd "$TWOLEBOT_DIR"
nohup "$BIN" \
    --data-dir "$DATA_DIR" \
    --port "$PORT" \
    --host "$HOST" \
    >> "$DATA_DIR/twolebot.log" 2>&1 &

NEW_PID=$!
echo "New process started (PID: $NEW_PID)" >> "$DATA_DIR/twolebot.log"

# Wait and verify
sleep 2

if curl -s --max-time 5 http://localhost:$PORT/api/status | grep -q '"status":"ok"'; then
    echo "Server is UP and healthy!" >> "$DATA_DIR/twolebot.log"
else
    echo "WARNING: Server may not be responding" >> "$DATA_DIR/twolebot.log"
fi

echo "=== Restart complete ===" >> "$DATA_DIR/twolebot.log"
