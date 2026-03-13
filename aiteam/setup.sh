#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKERS=(qic-worker-a qic-worker-b qic-worker-c)
BRANCHES=(worker-a worker-b worker-c)
PARENT_DIR="$(dirname "$REPO_ROOT")"

CLEAN=false
for arg in "$@"; do
  case "$arg" in
    --clean) CLEAN=true ;;
    -h|--help)
      echo "Usage: $0 [--clean]"
      echo ""
      echo "Sets up Claude Agent Team worktrees and settings for QIC."
      echo ""
      echo "  --clean   Tear down existing worktrees and recreate from scratch"
      exit 0
      ;;
  esac
done

echo "QIC Team Setup"
echo "=============="
echo "Repo: $REPO_ROOT"
echo ""

# --- Step 1: .claude/settings.json ---

SETTINGS_FILE="$REPO_ROOT/.claude/settings.json"

if [ -f "$SETTINGS_FILE" ] && [ "$CLEAN" = false ]; then
  echo "[1/3] .claude/settings.json already exists - skipping"
else
  echo "[1/3] Writing .claude/settings.json ..."
  cat > "$SETTINGS_FILE" <<'EOF'
{
  "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" },
  "teammateMode": "tmux"
}
EOF
  echo "      done"
fi

# --- Step 2: Worktrees ---

echo "[2/3] Setting up git worktrees ..."

if [ "$CLEAN" = true ]; then
  for i in "${!WORKERS[@]}"; do
    WDIR="$PARENT_DIR/${WORKERS[$i]}"
    BRANCH="${BRANCHES[$i]}"
    if [ -d "$WDIR" ]; then
      echo "      removing $WDIR ..."
      git -C "$REPO_ROOT" worktree remove --force "$WDIR" 2>/dev/null || true
      rm -rf "$WDIR"
    fi
    # Delete the branch if it exists so we can recreate
    git -C "$REPO_ROOT" branch -D "$BRANCH" 2>/dev/null || true
  done
  git -C "$REPO_ROOT" worktree prune
fi

for i in "${!WORKERS[@]}"; do
  WDIR="$PARENT_DIR/${WORKERS[$i]}"
  BRANCH="${BRANCHES[$i]}"

  if [ -d "$WDIR" ]; then
    echo "      $WDIR already exists - skipping (use --clean to recreate)"
  else
    echo "      creating $WDIR (branch: $BRANCH) ..."
    git -C "$REPO_ROOT" worktree add "$WDIR" -b "$BRANCH" main
  fi
done

# --- Step 3: Submodules in each worktree ---

echo "[3/3] Initialising submodules in each worktree ..."

for i in "${!WORKERS[@]}"; do
  WDIR="$PARENT_DIR/${WORKERS[$i]}"
  echo "      ${WORKERS[$i]} ..."
  git -C "$WDIR" submodule update --init --recursive --quiet
done

# --- Done ---

echo ""
echo "All done. Workers are ready:"
for i in "${!WORKERS[@]}"; do
  echo "  ~/git/${WORKERS[$i]}   (branch: ${BRANCHES[$i]})"
done
echo ""
echo "Next steps:"
echo "  1. Create ticket-dependencies.json:  node aiteam/sync-trello.js --dry-run"
echo "  2. Launch the team:                  tmux new-session -s qic; cd ~/git/qic && claude, then /goteam"
