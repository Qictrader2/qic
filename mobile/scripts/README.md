# scripts/

Helper scripts for mobile development.

| Script | Purpose |
|---|---|
| `sync-from-web.sh` | Pull shared files (types, tokens, constants, validators, formatters, design docs) from the main `Qictrader` monorepo at `/Users/jpvanzyl/Workspaces/Qictrader/`. Run before every push, or on demand when web frontend changes shared logic. `--check` for dry-run drift detection (used in CI). |

## Adding new scripts

Keep scripts:

- POSIX bash (`#!/usr/bin/env bash` + `set -euo pipefail`)
- Self-contained (no implicit env-var dependencies; document required env at the top)
- Idempotent where possible
- Logged with clear `[ N/M ] step name` prefixes
- Exit 0 on success, non-zero on failure (CI relies on this)

Document each new script here. Make executable: `chmod +x scripts/your-script.sh`.
