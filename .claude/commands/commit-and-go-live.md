---
description: QIC go-live — commit, push, deploy (parallel), and move Trello ticket to Dev Complete.
allowed-tools: Agent, Bash, Read, Glob, Grep, WebFetch
---

You are shipping QIC Trader to production. This skill commits all changes, pushes, deploys frontend + backend **in parallel**, and moves the Trello ticket to Dev Complete.

**Trello credentials:**
- API Key: `d0f2319aeb29e279616c592d79677692`
- Token: `ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0`
- Dev Complete list ID: `69adb791e90fb428655d9ad3`
- Qictrader Dev board ID: `69a5bb4b56b71b138fb3f2be`

Arguments: `$ARGUMENTS`

The repo layout:
- `/home/marcello/git/qic/` — root monorepo (tracks submodule refs)
- `/home/marcello/git/qic/frontend/` — Next.js frontend submodule
- `/home/marcello/git/qic/qictrader-backend-rs/` — Rust backend submodule

---

# THE GO-LIVE FLOW

```
┌──────────────────────────────────────────────────────────────────┐
│  1. INSPECT       →  Status + diff + log across all repos        │
│  2. COMMIT        →  Autonomous commit across submodules + root  │
│  3. PUSH          →  Push all repos to origin                    │
│  4. DEPLOY        →  Frontend + Backend IN PARALLEL              │
│  5. FIND TICKET   →  Resolve Trello card ID                      │
│  6. MOVE TICKET   →  Move Trello card to Dev Complete            │
│  7. CLEANUP       →  Remove breadcrumbs + plan file              │
│  8. REPORT        →  Summary of what shipped                     │
└──────────────────────────────────────────────────────────────────┘
```

---

## STEP 1: INSPECT

Read the active ticket and inspect all changes. Run these **in parallel**:

```bash
cat /home/marcello/git/qic/.current-ticket 2>/dev/null
```

```bash
cd /home/marcello/git/qic && git status && git log -5 --oneline
```

```bash
cd /home/marcello/git/qic/frontend && git status && git diff HEAD && git diff --staged && git log -3 --oneline
```

```bash
cd /home/marcello/git/qic/qictrader-backend-rs && git status && git diff HEAD && git diff --staged && git log -3 --oneline
```

**Active ticket (`.current-ticket`):**
- **Line 1:** Trello hex card ID (e.g. `69a5bb4b56b71b138fb3f2be`) — REQUIRED
- **Line 2:** Ticket label (e.g. `ES-001`) — OPTIONAL

If the file exists and line 1 is non-empty:
- Store the card ID — it will be embedded as a `Ticket-Id:` git trailer in every commit
- If line 2 exists, prefix commit messages with the ticket label: `ES-001: description`

If the file does not exist or is empty — no active ticket. Use emoji-prefix commit format.

Based on the inspection results, determine which repos have changes:
- **HAS_BACKEND_CHANGES**: true if backend has uncommitted changes, untracked files, or staged changes
- **HAS_FRONTEND_CHANGES**: true if frontend has uncommitted changes, untracked files, or staged changes

These flags drive Steps 2–4: **skip commit, push, and deploy entirely for repos with no changes.** Do not inspect, diff, push, or deploy a clean repo — it wastes time.

If everything is already clean (nothing to commit in any repo) and $ARGUMENTS has no special flags, confirm with the user whether to deploy current HEAD or stop.

---

## STEP 2: COMMIT

For **each** of the three repos (backend, frontend, root), classify independently:

| Category | When to use |
|----------|-------------|
| **NO CHANGES** | Nothing to commit — skip it |
| **SAVE POINT** | WIP / debugging / incomplete feature mid-flight |
| **SCRATCHPAD** | Temporary scripts in `scripts/scratch-pad/` |
| **NORMAL** | Coherent, shippable set of changes |
| **SPLIT** | Truly unrelated changes mixed together |

### Handle SPLIT autonomously (no asking)

If a repo has unrelated changes mixed together, **decide yourself**:

- Group by domain: auth changes together, payment changes together, etc.
- Group by layer: migrations separate from API handlers if unrelated
- If in doubt: one commit per logical feature is fine

**Never ask the user if you should split. Just do it.**

### Commit order — submodules first, root last

1. `qictrader-backend-rs/` — backend first
2. `frontend/` — frontend second
3. `/` (root) — root last (tracks updated submodule refs)

**CRITICAL: Every commit MUST include the `Ticket-Id:` git trailer if `.current-ticket` exists.**

```bash
cd /home/marcello/git/qic/qictrader-backend-rs && git add -A && git commit -m "$(cat <<'EOF'
ES-001: implement atomic escrow lock

- Add single-transaction escrow locking in repo layer
- Guard state transitions with can_transition_to()

Ticket-Id: 69a5bb4b56b71b138fb3f2be
EOF
)"
```

If no active ticket, commit without the trailer:
```bash
cd /home/marcello/git/qic/frontend && git add -A && git commit -m "$(cat <<'EOF'
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

**Root repo commit** (when only submodule refs changed):
```
🔗 Update submodule refs (frontend + backend)

Ticket-Id: 69a5bb4b56b71b138fb3f2be
```

---

## STEP 3: PUSH

**Only push repos that had commits in Step 2.** Skip clean repos entirely.

Push changed submodules **in parallel**:

If backend was committed:
```bash
cd /home/marcello/git/qic/qictrader-backend-rs && git push
```

If frontend was committed:
```bash
cd /home/marcello/git/qic/frontend && git push
```

After submodule pushes complete, push root (if it was committed):
```bash
cd /home/marcello/git/qic && git push
```

If a push is rejected (remote ahead), run `git pull --rebase` then push again. Never force-push.

---

## STEP 4: DEPLOY — CONDITIONAL + PARALLEL

**Only deploy repos that had new commits pushed in Step 3.** If a repo had NO CHANGES in Step 1 (nothing to commit, already up to date with remote), skip its deploy entirely — do not touch it.

If both repos need deploying, run them **in parallel**. If only one needs deploying, run just that one.

### Backend → Heroku (only if HAS_BACKEND_CHANGES)

Fast deploy via cross-compile + Slug API (default):

```bash
REPO_ROOT="$(git -C /home/marcello/git/qic rev-parse --show-toplevel 2>/dev/null || echo /home/marcello/git/qic)"
"$REPO_ROOT/scripts/fast-deploy-backend.sh" 2>&1
```

### Frontend → Vercel (only if HAS_FRONTEND_CHANGES)

```bash
cd /home/marcello/git/qic/frontend && vercel --prod --scope qictraders-projects --yes 2>&1
```

**All triggered deploys MUST succeed.** If any fails:
- STOP — do not move the Trello ticket
- Report which deploy failed and the error message
- The user must fix and re-run `/golive`

---

## STEP 5: FIND TICKET

Resolve the Trello card ID using this priority order. Stop at the first match.

**Priority 1: `Ticket-Id:` git trailer in recent commits**

```bash
git -C /home/marcello/git/qic log -20 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
```

If that returns a non-empty hex string (24 chars), use it.

If the root repo has no trailer, also check submodules:
```bash
git -C /home/marcello/git/qic/qictrader-backend-rs log -5 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
git -C /home/marcello/git/qic/frontend log -5 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
```

**Priority 2: `.current-ticket` breadcrumb file**

```bash
head -1 /home/marcello/git/qic/.current-ticket 2>/dev/null
```

**Priority 3: From $ARGUMENTS**

If $ARGUMENTS contains a 24-char hex string, use it directly.

If $ARGUMENTS contains a ticket label like `ES-001`, search Trello:
```
https://api.trello.com/1/search?query={TICKET_LABEL}&key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&modelTypes=cards&idBoards=69a5bb4b56b71b138fb3f2be&cards_limit=10
```

**If none of the above yields a card ID:**
> "No Trello card ID found in git trailers, .current-ticket, or arguments — card not moved. Run `/golive <card-id>` to move it manually."

---

## STEP 6: MOVE TICKET

Fetch the card to confirm it exists and get its name:
```
GET https://api.trello.com/1/cards/{cardId}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&fields=name,idList
```

If the card is already in Dev Complete or a later column, skip the move and note it.

Otherwise move it:
```
PUT https://api.trello.com/1/cards/{cardId}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&idList=69adb791e90fb428655d9ad3
```

---

## STEP 7: CLEANUP

After a successful go-live (both deploys succeeded AND ticket moved), read then delete the breadcrumb and plan file:

```bash
# Read BEFORE deleting (need values for rm)
CARD_ID=$(head -1 /home/marcello/git/qic/.current-ticket 2>/dev/null)
TICKET_LABEL=$(sed -n '2p' /home/marcello/git/qic/.current-ticket 2>/dev/null)

rm -f /home/marcello/git/qic/.current-ticket
rm -f "/home/marcello/git/qic/ticket-plans/${TICKET_LABEL}.md"
rm -f "/home/marcello/git/qic/ticket-plans/${CARD_ID}.md"
```

---

## STEP 8: REPORT

```
## Go-Live Complete

**Commit:** [sha] [message]
**Deployed:** frontend (Vercel) ✅ + backend (Heroku) ✅
**Ticket moved:** "[card name]" → Dev Complete
**Card ID source:** git trailer / .current-ticket / argument
```

Or if ticket could not be moved:
```
## Go-Live Complete

**Commit:** [sha] [message]
**Deployed:** frontend (Vercel) ✅ + backend (Heroku) ✅
**Ticket:** not moved — [reason]
```

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

**NORMAL (active ticket):**
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

---

## RULES

- **Never ask about splitting** — decide autonomously
- **Never ask for approval** on normal commits — just commit
- **Only pause** if about to commit a file that looks like secrets (`.env`, credentials)
- **Never add Co-Authored-By or AI attribution** footers
- **Commit message in imperative mood** — "Add feature" not "Added feature"
- **Root commits last** — always after submodules so the refs are up to date
- Use `git add -A` within each submodule directory — never `git add` from root with submodule paths
- **Ticket-Id trailer is mandatory** when `.current-ticket` exists — every commit, no exceptions
- **Deploy only repos with changes** — skip clean repos entirely (no inspect, no push, no deploy)
- **Deploy changed repos in parallel** when both have changes
- **Never move the ticket unless ALL triggered deploys succeed**
- If a deploy fails, report all outcomes and do NOT move the ticket
- If the ticket is already in Dev Complete or a later column, skip the move and note it
- The `Ticket-Id:` git trailer is the primary source of truth — `.current-ticket` is only a fallback
- Always report which source the card ID came from (trailer / file / argument)

$ARGUMENTS
