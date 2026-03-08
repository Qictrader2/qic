#!/bin/bash
set -e

echo "=== twolebot build ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED=0

# 1. Rust backend
echo -e "${YELLOW}[1/4] Building Rust backend...${NC}"
cd "$(dirname "$0")"
if cargo build 2>&1; then
    echo -e "${GREEN}  ✓ Rust build passed${NC}"
else
    echo -e "${RED}  ✗ Rust build failed${NC}"
    FAILED=1
fi

# 2. Rust tests
echo ""
echo -e "${YELLOW}[2/4] Running Rust tests...${NC}"
if cargo test 2>&1; then
    echo -e "${GREEN}  ✓ Rust tests passed${NC}"
else
    echo -e "${RED}  ✗ Rust tests failed${NC}"
    FAILED=1
fi

# 3. Elm frontend (if exists)
if [ -d "frontend" ] && [ -f "frontend/elm.json" ]; then
    echo ""
    echo -e "${YELLOW}[3/4] Building Elm frontend...${NC}"
    cd frontend
    if lamdera make src/Main.elm --output=dist/elm.js 2>&1; then
        echo -e "${GREEN}  ✓ Elm build passed${NC}"
        # Copy to local data/ for development
        mkdir -p ../data/frontend/dist
        cp dist/elm.js ../data/frontend/dist/
        cp index.html ../data/frontend/dist/
        # Copy elm-pkg-js modules
        if [ -d "elm-pkg-js" ]; then
            mkdir -p ../data/frontend/dist/elm-pkg-js
            cp elm-pkg-js/*.js ../data/frontend/dist/elm-pkg-js/
        fi

        # Also copy to XDG data dir for installed usage
        if [ "$(uname)" = "Darwin" ]; then
            XDG_DATA="${HOME}/Library/Application Support/twolebot"
        else
            XDG_DATA="${XDG_DATA_HOME:-${HOME}/.local/share}/twolebot"
        fi
        mkdir -p "${XDG_DATA}/frontend/dist"
        cp dist/elm.js "${XDG_DATA}/frontend/dist/"
        cp index.html "${XDG_DATA}/frontend/dist/"
        if [ -d "elm-pkg-js" ]; then
            mkdir -p "${XDG_DATA}/frontend/dist/elm-pkg-js"
            cp elm-pkg-js/*.js "${XDG_DATA}/frontend/dist/elm-pkg-js/"
        fi
    else
        echo -e "${RED}  ✗ Elm build failed${NC}"
        FAILED=1
    fi
    cd ..
else
    echo ""
    echo -e "${YELLOW}[3/4] Elm frontend...${NC}"
    echo -e "  (skipped - frontend/ not set up yet)"
fi

# 4. Elm tests (if elm-test available and tests directory exists)
if [ -d "frontend/tests" ] && command -v elm-test &> /dev/null; then
    echo ""
    echo -e "${YELLOW}[4/4] Running Elm tests...${NC}"
    cd frontend
    if elm-test 2>&1; then
        echo -e "${GREEN}  ✓ Elm tests passed${NC}"
    else
        echo -e "${RED}  ✗ Elm tests failed${NC}"
        FAILED=1
    fi
    cd ..
else
    echo ""
    echo -e "${YELLOW}[4/4] Elm tests...${NC}"
    echo -e "  (skipped - no tests directory or elm-test not available)"
fi

# Summary
echo ""
echo "=== Build Summary ==="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}Some checks failed${NC}"
    exit 1
fi
