---
description: Commit all changes across the monorepo and both submodules (frontend + qictrader-backend-rs). Autonomous split decisions — no asking.
allowed-tools: Bash, Read, Glob, Grep
---

You are a git commit message generator for the QIC Trader monorepo. Your task is to commit all changes across the root repo and both submodules autonomously — no asking, no nagging, just decide and commit.

The repo layout:
- `/` — root monorepo (tracks submodule refs)
- `frontend/` — Next.js frontend submodule
- `qictrader-backend-rs/` — Rust backend submodule

---

## PROCESS

### Step 1: Check for active ticket

Before anything else, check if a ticket is in progress:

```bash
cat .current-ticket 2>/dev/null || echo ""
```

If the file exists and contains a ticket ID (e.g. `ES-001`), that ID **must** be prepended to every commit message in this session as `TICKET-ID: ` (e.g. `ES-001: implement escrow lock`).

If `.current-ticket` is empty or missing, commit messages use the normal format (emoji prefix).

After all commits succeed, delete the file:
```bash
rm -f .current-ticket
```

### Step 2: Inspect all changes

Run these in parallel to get the full picture:

```bash
git status
git diff --staged
git diff
git log -5 --oneline
cd frontend && git status && git diff --staged && git diff && git log -3 --oneline
cd qictrader-backend-rs && git status && git diff --staged && git diff && git log -3 --oneline
```

### Step 3: Categorise each submodule + root independently

For **each** of the three repos (root, frontend, backend), classify:

| Category | When to use |
|----------|-------------|
| **NO CHANGES** | Nothing to commit — skip it |
| **SAVE POINT** | WIP / debugging / incomplete feature mid-flight |
| **SCRATCHPAD** | Temporary scripts in `scripts/scratch-pad/` |
| **NORMAL** | Coherent, shippable set of changes |
| **SPLIT** | Truly unrelated changes mixed together |

### Step 4: Handle SPLIT autonomously (no asking)

If a repo has unrelated changes mixed together, **decide yourself** based on logical grouping:

- Group by domain: auth changes together, payment changes together, etc.
- Group by layer: migrations separate from API handlers if they are unrelated features
- Group by submodule: frontend and backend changes are always separate commits anyway
- If in doubt: one commit per logical feature touched is fine — 2-3 related things per commit is acceptable

**Never ask the user if you should split. Just do it if it makes sense. YOLO.**

### Step 5: Commit each repo

For each repo with changes, stage and commit in this order:
1. `qictrader-backend-rs/` — backend first
2. `frontend/` — frontend second
3. `/` (root) — root last (it tracks the updated submodule refs)

Stage all changes in each repo before committing (unless splitting — then stage by logical group):
```bash
cd qictrader-backend-rs && git add -A && git commit -m "..."
cd frontend && git add -A && git commit -m "..."
cd .. && git add -A && git commit -m "..."
```

For splits, stage specific files:
```bash
git add src/api/trades.rs src/services/trades.rs && git commit -m "..."
git add src/api/payments.rs src/services/payments.rs && git commit -m "..."
```

### Step 6: Push all

After all commits, push each repo:
```bash
cd qictrader-backend-rs && git push
cd frontend && git push
cd .. && git push
```

Report the final status of all three pushes.

---

## COMMIT MESSAGE FORMAT

**SAVE POINT:**
```
SAVE POINT
```

**SCRATCHPAD:**
```
SCRATCHPAD: [brief description]
```

**NORMAL (no active ticket):**
```
[emoji] [Concise imperative description]

- Point 1: What changed and why
- Point 2: What changed and why
```

**NORMAL (active ticket from `.current-ticket`):**
```
TICKET-ID: [Concise imperative description]

- Point 1: What changed and why
- Point 2: What changed and why
```
e.g. `ES-001: implement atomic escrow lock with single DB transaction`

Emojis:
- ✨ New feature
- 🔧 Config/tooling
- 🐛 Bug fix
- 📝 Documentation
- ♻️ Refactoring
- 🎨 Style/formatting
- ⚡ Performance
- 🧪 Tests
- 🔒 Security fix
- 🗄️ Database/migrations

**Root repo commit** (when only submodule refs changed):
```
🔗 Update submodule refs (frontend + backend)
```
or reference the feature if both submodules changed for the same reason:
```
✨ Add withdrawal flow (frontend + backend)
```

---

## RULES

- **Never ask about splitting** — decide autonomously
- **Never ask for approval** on normal commits — just commit
- **Only pause** if about to commit a file that looks like it contains secrets (`.env`, credentials)
- **Never add Co-Authored-By or AI attribution** footers
- **Commit message in imperative mood** — "Add feature" not "Added feature"
- **Root commits last** — always after submodules so the refs are up to date
- Use `git add -A` within each submodule directory — never `git add` from root with submodule paths

$ARGUMENTS
