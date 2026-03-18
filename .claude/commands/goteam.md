---
description: QIC Team Lead - spawn 3 Opus worker teammates and dispatch a batch of ~10 tickets via /quickship.
allowed-tools: Bash, Agent
---

## KNOWN PITFALLS (read first)

1. **Stale submodule gitlinks**: origin/main has orphaned gitlinks (backend, mobile-app, qic, stories, telegram-bot) not in .gitmodules. `git submodule update --init --recursive` will FAIL after reset. You MUST remove them with `git rm --cached` after every hard reset. See the init script in Step 2.
2. **Workers won't auto-start**: Spawned agents report "ready" and wait. Your spawn prompt AND dispatch messages must explicitly tell them to use the **Skill tool** with `skill: "quickship"` and `args: "TICKET-ID"`. Saying "run /quickship" is not enough - they don't know it's a Skill tool invocation.
3. **Do Step 1 (sync) BEFORE Step 2 (spawn)**. Workers are expensive - don't spawn until you know there are tickets.
4. **TeamCreate + worker init can run in parallel** since they're independent.
5. **Dispatch immediately in the spawn prompt** - don't spawn workers idle then send a separate message. Include the first ticket assignment in the spawn prompt itself to avoid the "ready and waiting" loop.
6. **Workers CANNOT be reused for multiple tickets**: After completing 1-2 tickets, workers' context fills up and they replay old completions instead of reading new SendMessage assignments. DO NOT try to reassign via SendMessage - it does not work. Instead: **spawn a fresh worker per ticket** (or per small batch of 2). Shut down and replace workers, don't try to fix stuck ones. This is the single biggest time-waster in the current flow.
8. **DO NOT kill workers during context compaction**: Idle notifications during compaction look identical to "stuck worker" idle spam. If a worker was actively working on a ticket and suddenly goes idle, it is probably compacting - NOT stuck. Wait at least 5 minutes before assuming a worker is stuck. Killing a compacting worker kills in-progress work and wastes the entire ticket run. When in doubt, WAIT.
7. **Orphan gitlinks now fixed**: As of commit 78ba7b9, the orphaned gitlinks (backend, mobile-app, qic, stories, telegram-bot) have been removed from origin/main. The `git rm --cached` cleanup in the init script is no longer needed but is harmless to keep as a safety net.

You are the QIC TEAM LEAD. You coordinate 3 parallel Opus workers to ship a batch of tickets. You do not implement tickets yourself. You assign, track, escalate, and report.

Arguments: `$ARGUMENTS` (optional: batch size and/or column filter, e.g. `/goteam 5`, `/goteam 10 --col "to do"`, `/goteam --col "to do"`)

---

## STEP 1: SYNC + READ BOARD

Pull fresh Trello state and read what is available. Do this before creating any workers.

```bash
# Sync Trello -> ticket-dependencies.json + .tickets-done
node aiteam/sync-trello.js --no-infer

# Show current progress across all waves
node aiteam/next-tickets.js -s

# Show the full unblocked pool with wave info
node aiteam/next-tickets.js -w
```

Read the output. Understand:
- Which wave we are in (pick tickets from the lowest incomplete wave first)
- How many tickets are unblocked and available
- Any that were kicked back (sync will have unmarked them)

If sync fails (network error, bad credentials), stop and report the error to the user. Do not proceed without fresh board state.

---

## STEP 2: CREATE WORKERS

Create 3 teammates now (model: Opus). First init all 3 worktrees in parallel using Bash, then spawn agents.

### Worker init script (run in parallel for a, b, c):

```bash
cd ~/git/qic-worker-X/ \
  && rm -f .current-ticket \
  && git checkout -- . 2>/dev/null \
  && git fetch origin main \
  && git reset --hard origin/main \
  && for sub in backend mobile-app qic stories telegram-bot; do \
       git rm --cached "$sub" 2>/dev/null; rm -rf "$sub"; \
     done \
  && git submodule update --init --recursive \
  && echo "Worker X ready"
```

**IMPORTANT**: The `git rm --cached` loop removes stale gitlinks that exist in origin/main but have no .gitmodules entry. Without this, `git submodule update` will fail with "No url found for submodule path". This must run AFTER `git reset --hard` every time.

### Spawn with first ticket pre-assigned

Do NOT spawn workers idle. Include the first ticket in the spawn prompt so they start immediately:

```
Agent tool params:
  name: "worker-a"
  team_name: "qic-batch"
  model: opus
  mode: bypassPermissions
```

Spawn prompt template (replace TICKET-ID and worker dir):

> You are a QIC full-stack engineer working in ~/git/qic-worker-a/. You handle both frontend (Next.js/TypeScript) and backend (Rust/Axum) work.
>
> YOUR FIRST TICKET: TICKET-ID
> Start immediately. Use the Skill tool: `skill: "quickship", args: "TICKET-ID"` to implement it. Do NOT wait for further instructions.
>
> For subsequent tickets, the team lead will send you a new ticket ID. Each time, use the Skill tool the same way: `skill: "quickship", args: "NEW-TICKET-ID"`.
>
> MERGE PROTOCOL - before committing in Phase 4 of /quickship:
> ```bash
> cd ~/git/qic-worker-a/
> git fetch origin main
> git log --oneline HEAD..origin/main
> git diff HEAD...origin/main --stat
> git rebase origin/main
> cargo check 2>&1 | tail -5
> cd frontend && bun run typecheck 2>&1 | tail -5
> ```
> If rebase fails with submodule pointer conflicts:
> ```bash
> git checkout --theirs -- frontend qictrader-backend-rs
> git add frontend qictrader-backend-rs
> git rebase --continue
> ```
> Push to main only. No worker branches. If push rejected: `git pull --rebase && git push`.

Once agents are spawned, set up tmux tiling with auto-retile hooks so the layout survives pane creation/destruction:

```bash
# Set tiled layout
tmux select-layout -t qic:0 tiled

# Auto-retile when panes are added or removed (CRITICAL - keeps layout stable)
tmux set-hook -t qic after-split-window 'select-layout tiled'
tmux set-hook -t qic pane-exited 'select-layout tiled'

# Pane labels and borders
tmux set-option -t qic pane-border-status top
tmux set-option -t qic pane-border-format "#{?pane_active,#[bold#,fg=colour255],#[fg=colour245]} #{pane_title} "
tmux set-option -t qic pane-border-style        "fg=colour238"
tmux set-option -t qic pane-active-border-style "fg=colour255,bold"

# Color each pane (adjust indices as workers spawn/die)
tmux select-pane -t qic:0.0 -T "  LEAD"    -P "bg=colour17,fg=colour255"
tmux select-pane -t qic:0.1 -T "  Agent A" -P "bg=colour22,fg=colour255"
tmux select-pane -t qic:0.2 -T "  Agent B" -P "bg=colour54,fg=colour255"
tmux select-pane -t qic:0.3 -T "  Agent C" -P "bg=colour23,fg=colour255"
```

**Re-run `tmux select-layout -t qic:0 tiled` after every worker spawn or shutdown** as a safety net. The hooks should handle it automatically, but belt-and-suspenders.

### Pane naming (MUST DO after every spawn/replacement)

Tmux panes default to "Claude Code" which is useless. After spawning or replacing workers, rename all panes to match agent names:

```bash
# After initial spawn (pane 0 = lead, 1-3 = workers)
tmux select-pane -t qic:0.0 -T "LEAD"
tmux select-pane -t qic:0.1 -T "Worker A"
tmux select-pane -t qic:0.2 -T "Worker B"
tmux select-pane -t qic:0.3 -T "Worker C"

# When replacing a worker (e.g. worker-a died, spawned worker-a2):
# Find which pane the new agent landed in and rename it
tmux list-panes -t qic:0 -F '#{pane_index} #{pane_title}'
tmux select-pane -t qic:0.N -T "Worker A2"
```

**Naming convention**: worker-a, worker-a2, worker-a3 etc. The letter maps to the worktree dir (a = qic-worker-a/). The number increments each time that slot is respawned. This makes it obvious which agent is which and which worktree it uses.

Colors: Lead=blue, Agent A=green, Agent B=purple, Agent C=teal. All white text on dark bg - high contrast.

---

## STEP 3: PICK BATCH

Parse `$ARGUMENTS` for a number (batch size, default 10) and an optional `--col "COLUMN"` flag.

```bash
# Example: BATCH_SIZE=10, COL_FILTER="to do"
# Adapt based on what the user passed in $ARGUMENTS
node aiteam/next-tickets.js -n "$BATCH_SIZE" -w ${COL_FILTER:+-c "$COL_FILTER"}
```

Select up to the batch size from the output. Prefer tickets in the same wave to minimise submodule pointer conflicts. Note which tickets you are targeting for this batch.

---

## STEP 4: DISPATCH LOOP

The first ticket per worker is assigned in the spawn prompt (Step 2). For subsequent tickets, send via SendMessage:

```
SendMessage to worker-X:
  "Your next ticket is TICKET-ID. Use the Skill tool: skill: "quickship", args: "TICKET-ID" to implement it now."
```

**CRITICAL**: Workers don't understand "run /quickship". You must tell them to use the **Skill tool** with the exact params. This is the #1 cause of workers sitting idle.

**When a worker returns its Phase 6 report:**

### On "Ship Complete"

**Verify before trusting.** Do not mark done based on self-reported status alone.

```bash
# 1. Confirm the commit actually landed on main in both submodules
cd ~/git/qic-worker-X/qictrader-backend-rs && git log --oneline -1 origin/main
cd ~/git/qic-worker-X/frontend && git log --oneline -1 origin/main

# 2. Confirm the Trello card was moved to Dev Complete
# (the lead fetches card status - if not moved, move it now)
curl -s "https://api.trello.com/1/cards/CARD_ID?key=$TRELLO_KEY&token=$TRELLO_TOKEN&fields=idList" \
  | node -e "const d=JSON.parse(require('fs').readFileSync('/dev/stdin','utf8')); \
     if(d.idList!=='69adb791e90fb428655d9ad3') console.log('NOT MOVED - fixing...'); \
     else console.log('OK');"

# 3. Mark done locally
node aiteam/next-tickets.js -m TICKET-ID
```

If the commit is missing from origin/main or the card wasn't moved, fix it before marking done.
Then immediately assign the next ticket from the batch to that agent.

### On "BLOCKED" or "BUILD FAILED" or "TESTS FAILED"
- Do NOT mark the ticket done
- Log: `SKIPPED: TICKET-ID - <one-line reason>`
- Assign the next ticket from the batch to that agent

### On "NEEDS_CLARIFICATION"
- Relay the contradiction to the user and pause that agent
- Assign other workers their next tickets while waiting for the human answer
- Once resolved, re-assign the clarified ticket to the freed worker

**Rules:**
- ONE ticket per agent at a time. Never two.
- Mark done ONLY after a confirmed "Ship Complete" in the Phase 6 report.
- Skipped tickets must be reported at batch end - never silently dropped.

---

## MERGE PROTOCOL (include in every worker assignment)

Remind each agent to do this BEFORE Phase 4 commit inside /quickship:

```bash
# In the worker's worktree (e.g. ~/git/qic-worker-a/)
# All pushes go to main - no worker branches.
git fetch origin main
git log --oneline HEAD..origin/main       # what landed since you started
git diff HEAD...origin/main --stat        # which files changed
# Read any overlapping files before rebasing
git rebase origin/main
# Resolve conflicts if any — you already read the incoming changes
cargo check 2>&1 | tail -5
cd frontend && bun run typecheck 2>&1 | tail -5
```

If the rebase fails with submodule pointer conflicts:
```bash
git checkout --theirs -- frontend qictrader-backend-rs
git add frontend qictrader-backend-rs
git rebase --continue
```
Then re-verify both builds before proceeding.

**Push to main only.** Do not create or push to worker branches (worker-a, worker-b, etc.).
Use `git push origin main` or `./commit-all.sh "message" --push`.
If the push is rejected because another worker pushed first, do `git pull --rebase && git push`.

---

## STEP 5: BATCH COMPLETE

When all batch tickets are shipped or skipped, STOP. Do not start a new batch.

Run the batch-end sync automatically:

```bash
# 1. Re-sync Trello to pick up any state changes
node aiteam/sync-trello.js --no-infer

# 2. Show progress
node aiteam/next-tickets.js -s

# 3. Verify all shipped tickets are actually on Dev Complete in Trello
#    For each shipped ticket, confirm the card is on the Dev Complete list.
#    If any are not, move them now and report the discrepancy.
```

Report to the user:
```
Batch complete.

Shipped (X):
  - TICKET-ID: name
  ...

Skipped (Y):
  - TICKET-ID: <reason>
  ...

Trello verification:
  - All cards confirmed on Dev Complete / N cards required manual move

[paste summary output]
```

---

## RECOVERY (if lead session dies mid-batch)

If this session dies, the state is preserved in `.tickets-done`. To recover:
1. Check which tickets actually landed: `git log --oneline -20` in each submodule
2. Manually mark any that shipped: `node aiteam/next-tickets.js -m TICKET-ID`
3. Start a new `/goteam` session - workers will see the corrected state
