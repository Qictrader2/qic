#!/bin/bash
# Start all twolebot instances
# Called from Windows startup via VBS script

set -e

# Ensure user binaries are in PATH (WSL startup via VBS doesn't source shell profile)
export PATH="/home/schalk/.local/bin:/home/schalk/.nvm/versions/node/v22.15.0/bin:/home/schalk/.cargo/bin:$PATH"

TWOLEBOT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="$TWOLEBOT_DIR/target/debug/twolebot"

# Optional env file
if [ -f "$TWOLEBOT_DIR/.env" ]; then
    set -a
    source "$TWOLEBOT_DIR/.env"
    set +a
fi

start_instance() {
    local data_dir="$1"
    local port="$2"
    local name="$3"

    # Skip if already running on this port
    if lsof -iTCP:"$port" -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "[$name] Already running on port $port, skipping."
        return 0
    fi

    local log_dir="$data_dir"
    mkdir -p "$log_dir"

    echo "[$name] Starting on port $port (data: $data_dir)..."
    nohup "$BIN" \
        --data-dir "$data_dir" \
        --port "$port" \
        --host 0.0.0.0 \
        >> "$log_dir/twolebot.log" 2>&1 &

    echo "[$name] Started (PID: $!)"
}

echo "=== Starting all twolebot instances ($(date)) ==="

start_instance "$TWOLEBOT_DIR/data"                8080  "main"
start_instance "/home/schalk/git/twolebot-eline"  17701  "eline"
start_instance "/home/schalk/git/twolebot-nita"   17702  "nita"

# Wait a moment then health check
sleep 3

for port in 8080 17701 17702; do
    if curl -s --max-time 5 "http://localhost:$port/api/status" | grep -q '"status":"ok"'; then
        echo "Port $port: OK"
    else
        echo "Port $port: WARNING - not responding yet (may still be starting)"
    fi
done

echo "=== All instances launched ==="
