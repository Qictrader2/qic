#!/bin/bash
# Quick Elm build - rebuilds frontend without full compile.sh
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/frontend"
DIST_DIR="$SCRIPT_DIR/data/frontend/dist"

mkdir -p "$DIST_DIR"
cd "$FRONTEND_DIR"
lamdera make src/Main.elm --output="$DIST_DIR/elm.js"
cp index.html "$DIST_DIR/"
if [ -d "elm-pkg-js" ]; then
    mkdir -p "$DIST_DIR/elm-pkg-js"
    cp elm-pkg-js/*.js "$DIST_DIR/elm-pkg-js/"
fi
