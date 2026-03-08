#!/bin/bash
# Watch Elm files and rebuild on change
# Requires: inotifywait (sudo apt install inotify-tools)

cd "$(dirname "$0")/frontend"

OUTPUT="../data/frontend/dist/elm.js"

echo "Watching Elm files... (Ctrl+C to stop)"
echo "Output: $OUTPUT"
echo ""

# Initial build
echo "Building..."
if lamdera make src/Main.elm --output="$OUTPUT" 2>&1; then
    echo "✓ Build successful"
else
    echo "✗ Build failed"
fi
echo ""

# Watch for changes
while true; do
    inotifywait -q -r -e modify,create,delete --include '\.elm$' src/
    echo ""
    echo "Change detected, rebuilding..."
    if lamdera make src/Main.elm --output="$OUTPUT" 2>&1; then
        echo "✓ Build successful ($(date +%H:%M:%S))"
    else
        echo "✗ Build failed"
    fi
done
