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

Read the Trello card ID and ticket label from `.current-ticket` in the monorepo root:

```bash
cat .current-ticket 2>/dev/null
```

The file has up to two lines:
- **Line 1:** Trello hex card ID (e.g. `69a5bb4b56b71b138fb3f2be`) — REQUIRED
- **Line 2:** Ticket label (e.g. `ES-001`) — OPTIONAL

If the file exists and line 1 is non-empty:
- Store the card ID — it will be embedded as a `Ticket-Id:` git trailer in every commit
- If line 2 exists, prefix commit messages with the ticket label: `ES-001: description`

If the file does not exist or is empty — no active ticket. Use normal emoji-prefix commit format.

### Step 2: Inspect all changes

Run these in parallel to get the full picture:

```bash
git status && git diff --staged && git diff && git log -5 --oneline
```

```bash
cd frontend && git status && git diff --staged && git diff && git log -3 --oneline
```

```bash
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

### Step 5: Commit each repo with Ticket-Id trailer

For each repo with changes, stage and commit in this order:
1. `qictrader-backend-rs/` — backend first
2. `frontend/` — frontend second
3. `/` (root) — root last (it tracks the updated submodule refs)

**CRITICAL: Every commit MUST include the `Ticket-Id:` git trailer if `.current-ticket` exists.**

The trailer goes at the end of the commit message body, separated by a blank line:

```bash
cd qictrader-backend-rs && git add -A && git commit -m "$(cat <<'EOF'
ES-001: implement atomic escrow lock

- Add single-transaction escrow locking in repo layer
- Guard state transitions with can_transition_to()

Ticket-Id: 69a5bb4b56b71b138fb3f2be
EOF
)"
```

If no active ticket, commit without the trailer:
```bash
cd frontend && git add -A && git commit -m "$(cat <<'EOF'
✨ Add withdrawal confirmation dialog

- New ConfirmWithdrawal component with amount validation
- Wired to existing withdrawal API endpoint
EOF
)"
```

For splits, stage specific files:
```bash
git add src/api/trades.rs src/services/trades.rs && git commit -m "..."
git add src/api/payments.rs src/services/payments.rs && git commit -m "..."
```

**Every split commit still gets the same `Ticket-Id:` trailer.**

### Step 6: Push all

After all commits, push each repo:
```bash
cd qictrader-backend-rs && git push
cd frontend && git push
git push
```

Report the final status of all three pushes.

---

## COMMIT MESSAGE FORMAT

**SAVE POINT:**
```
SAVE POINT

Ticket-Id: 69a5bb4b56b71b138fb3f2be
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

Ticket-Id: 69a5bb4b56b71b138fb3f2be
```
e.g. `ES-001: implement atomic escrow lock with single DB transaction`

Emojis (only used when no active ticket):
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

Ticket-Id: 69a5bb4b56b71b138fb3f2be
```
or reference the feature if both submodules changed for the same reason:
```
ES-001: Add withdrawal flow (frontend + backend)

Ticket-Id: 69a5bb4b56b71b138fb3f2be
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
- **Ticket-Id trailer is mandatory** when `.current-ticket` exists — every commit, no exceptions
- **Do NOT delete `.current-ticket`** — it persists until the next `/ticket` overwrites it or the user manually removes it

$ARGUMENTS
