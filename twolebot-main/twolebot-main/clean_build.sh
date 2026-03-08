#!/usr/bin/env bash
# Clean Rust build artifacts while preserving the final binaries.
# Removes: incremental compilation cache, deps, build scripts, fingerprints,
#          doc output, test artifacts, and tmp.
# Keeps:  the compiled binaries (twolebot, twolebot-mcp) and their .d files.
set -euo pipefail

TARGET="target"

if [ ! -d "$TARGET" ]; then
    echo "No target/ directory found. Nothing to clean."
    exit 0
fi

echo "=== Before ==="
du -sh "$TARGET" 2>/dev/null || true

freed=0

clean_profile() {
    local profile="$1"
    local dir="$TARGET/$profile"
    [ -d "$dir" ] || return 0

    echo ""
    echo "--- $profile ---"
    du -sh "$dir" 2>/dev/null || true

    for subdir in incremental deps build .fingerprint examples; do
        if [ -d "$dir/$subdir" ]; then
            size=$(du -sb "$dir/$subdir" 2>/dev/null | cut -f1)
            freed=$((freed + size))
            echo "  Removing $profile/$subdir ($(du -sh "$dir/$subdir" | cut -f1))"
            rm -rf "$dir/$subdir"
        fi
    done
}

clean_profile "debug"
clean_profile "release"

# Remove docs, test artifacts, tmp
for subdir in doc tests tmp flycheck0; do
    if [ -d "$TARGET/$subdir" ]; then
        size=$(du -sb "$TARGET/$subdir" 2>/dev/null | cut -f1)
        freed=$((freed + size))
        echo "Removing $subdir/ ($(du -sh "$TARGET/$subdir" | cut -f1))"
        rm -rf "$TARGET/$subdir"
    fi
done

echo ""
echo "=== After ==="
du -sh "$TARGET" 2>/dev/null || true

freed_mb=$((freed / 1024 / 1024))
freed_gb=$((freed_mb / 1024))
if [ "$freed_gb" -gt 0 ]; then
    echo "Freed ~${freed_gb}GB"
else
    echo "Freed ~${freed_mb}MB"
fi
echo ""
echo "Binaries preserved:"
ls -lh "$TARGET"/debug/twolebot* "$TARGET"/release/twolebot* 2>/dev/null || echo "  (none)"
echo ""
echo "Next 'cargo build' will recompile deps (~2-3 min)."
