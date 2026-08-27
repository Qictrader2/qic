# .qictrader-context

This folder holds **snapshots** of context from the main `Qictrader` monorepo. It's not the source of truth — the main monorepo at `/Users/jpvanzyl/Workspaces/Qictrader` is. These snapshots exist so:

1. Cursor agents in this repo have enough context without needing to read the entire main monorepo every session
2. The repo is self-contained if someone clones it on a machine that doesn't have the main monorepo
3. CI can reference these files without network access to the main repo

## Files

| File | Source | Refresh via |
|---|---|---|
| `tickets-export.json` | Trello board (66 cards in Acknowledged) | `scripts/export-tickets.sh` (TODO) |
| `trello-board.md` | `Qictrader/ops/trello.md` | manual port; rarely changes |
| `api-endpoints.md` | hand-written summary | manual maintenance |
| `intended-state-machines.md` | `Qictrader/qictrader-backend-rs/docs/intended-entity-state-machines.md` | `./scripts/sync-from-web.sh` |
| `as-built-state-machines.md` | `Qictrader/qictrader-backend-rs/docs/as-built-state-machines.md` | `./scripts/sync-from-web.sh` |
| `database-schema.md` | `Qictrader/qictrader-backend-rs/docs/database-schema.md` | `./scripts/sync-from-web.sh` |
| `security-carryforward.md` | `Qictrader/architecture/SECURITY-CARRYFORWARD-FOR-NEW-APPS.md` | `./scripts/sync-from-web.sh` |
| `web-codebase-pointer.md` | manual | maintenance as web evolves |
| `backend-codebase-pointer.md` | manual | maintenance as backend evolves |

## Rule

Do not edit these files directly to "fix" something you found in them. If the source has changed, run `./scripts/sync-from-web.sh`. If you spot a real bug or outdated info, fix it in the source repo and resync.
