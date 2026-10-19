#!/usr/bin/env bash
# Store a long-lived Claude token for ticket preparation without echoing it.
# Run `claude setup-token` first, then run this and paste the token when asked.
# Works on macOS and in Git Bash on Windows.
set -euo pipefail

dest="${QIC_CLAUDE_TOKEN_FILE:-$HOME/.config/qic/claude-token.env}"
mkdir -p "$(dirname "$dest")"
umask 077

printf 'Paste the token from `claude setup-token` (input is hidden): '
IFS= read -rs token
printf '\n'
token="${token//[$'\r\n\t ']/}"

if [ -z "$token" ]; then
  echo "No token entered; nothing was written." >&2
  exit 1
fi
if ! [[ "$token" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "That does not look like a token (unexpected characters); nothing was written." >&2
  exit 1
fi
case "$token" in
  sk-ant-oat*) ;;
  *) echo "Warning: tokens from \`claude setup-token\` usually start with sk-ant-oat. Storing it anyway." >&2 ;;
esac

tmp="$dest.tmp.$$"
trap 'rm -f "$tmp"' EXIT
{
  printf "CLAUDE_CODE_OAUTH_TOKEN='%s'\n" "$token"
  printf 'CLAUDE_CODE_OAUTH_TOKEN_CREATED=%s\n' "$(date +%Y-%m-%d)"
} > "$tmp"
chmod 600 "$tmp"
mv -f "$tmp" "$dest"
trap - EXIT

case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    icacls "$(cygpath -w "$dest")" /inheritance:r /grant:r "${USERNAME}:F" > /dev/null
    echo "Restricted $(cygpath -w "$dest") to ${USERNAME} only."
    ;;
  *)
    echo "Stored $dest (owner read/write only)."
    ;;
esac

if command -v claude > /dev/null 2>&1; then
  echo "Checking that the Claude CLI accepts the token..."
  if reply="$(CLAUDE_CODE_OAUTH_TOKEN="$token" claude -p "Reply with OK only" 2>&1)" && [[ "$reply" == *OK* ]]; then
    echo "OK: the token works."
  else
    echo "The CLI did not accept the token. Its reply was:" >&2
    printf '%s\n' "$reply" | head -5 >&2
    unset token
    exit 1
  fi
else
  echo "The claude CLI is not on PATH, so the token was stored but not tested."
fi
unset token
