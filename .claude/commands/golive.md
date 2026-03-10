---
description: QIC go-live — deploy frontend + backend and move the Trello ticket to Dev Complete.
allowed-tools: Bash, Read, WebFetch
---

You are deploying QIC Trader to production. This commits everything, pushes to Vercel + Heroku, and moves the ticket to Dev Complete in Trello.

**Trello credentials:**
- API Key: `d0f2319aeb29e279616c592d79677692`
- Token: `ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0`
- Dev Complete list ID: `69adb791e90fb428655d9ad3`
- Qictrader Dev board ID: `69a5bb4b56b71b138fb3f2be`

Arguments: `$ARGUMENTS`

---

# THE GO-LIVE FLOW

```
┌──────────────────────────────────────────────────────────────────┐
│  1. CHECK         →  Confirm there is something to deploy        │
│  2. COMMIT        →  Commit all submodules + root                │
│  3. DEPLOY        →  Push to Vercel (frontend) + Heroku (backend)│
│  4. FIND TICKET   →  Resolve Trello card ID with 100% accuracy  │
│  5. MOVE TICKET   →  Move Trello card to Dev Complete            │
│  6. REPORT        →  Confirm what was deployed and moved         │
└──────────────────────────────────────────────────────────────────┘
```

---

## STEP 1: CHECK

```bash
git -C /home/schalk/git/qic status
git -C /home/schalk/git/qic/frontend status
git -C /home/schalk/git/qic/qictrader-backend-rs status
```

If everything is already clean (nothing to commit) and there are no deploy-only flags in $ARGUMENTS, confirm with the user whether to deploy the current HEAD anyway or stop.

---

## STEP 2: COMMIT

If there are uncommitted changes, commit them using `commit-all.sh` with an appropriate message.

Extract the commit message from $ARGUMENTS if provided (e.g. `/golive ES-001 feat: escrow lock`).
Otherwise derive a message from the staged changes (look at git diff --staged).

```bash
./commit-all.sh "<message>" --push
```

If there is nothing to commit (working tree already clean), skip this step — just deploy HEAD.

---

## STEP 3: DEPLOY

```bash
./commit-all.sh "" --deploy
```

Wait for the deploy to complete. If the deploy script fails, STOP and report the error to the user. Do not move the ticket.

Note: `commit-all.sh --deploy` triggers:
- Frontend → Vercel deploy hook
- Backend → `git push heroku main`

---

## STEP 4: FIND TICKET — exact card ID resolution

Resolve the Trello card ID using this priority order. Stop at the first match.

**Priority 1: `Ticket-Id:` git trailer in recent commits**

This is the most reliable source — the card ID was embedded at commit time by `/get-commit`.

```bash
git -C /home/schalk/git/qic log -20 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
```

If that returns a non-empty hex string (24 chars), use it. This is the Trello card ID — no search needed.

If the root repo has no trailer (e.g., only submodule ref updates), also check submodules:
```bash
git -C /home/schalk/git/qic/qictrader-backend-rs log -5 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
git -C /home/schalk/git/qic/frontend log -5 --format='%(trailers:key=Ticket-Id,valueonly)' | head -1
```

**Priority 2: `.current-ticket` breadcrumb file**

Fallback if commits were made without `/get-commit` (e.g., manual commit):

```bash
head -1 /home/schalk/git/qic/.current-ticket 2>/dev/null
```

If that returns a non-empty hex string, use it.

**Priority 3: From $ARGUMENTS**

If $ARGUMENTS contains what looks like a Trello card ID (24-char hex string), use it directly.

If $ARGUMENTS contains a ticket label like `ES-001`, search for it:
```
https://api.trello.com/1/search?query={TICKET_LABEL}&key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&modelTypes=cards&idBoards=69a5bb4b56b71b138fb3f2be&cards_limit=10
```

**If none of the above yields a card ID** — skip the move and tell the user:
> "No Trello card ID found in git trailers, .current-ticket, or arguments — card not moved. Run `/golive <card-id>` to move it manually."

---

## STEP 5: MOVE TICKET

Once the Trello card ID is known, move it directly — no search needed:

```
PUT https://api.trello.com/1/cards/{cardId}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&idList=69adb791e90fb428655d9ad3
```

Before moving, optionally fetch the card to confirm it exists and get its name for the report:
```
GET https://api.trello.com/1/cards/{cardId}?key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&fields=name,idList
```

If the card is already in Dev Complete or a later column, skip the move and note it.

---

## STEP 6: REPORT

Print a clean summary:

```
## Go-Live Complete

**Deployed:** frontend (Vercel) + backend (Heroku)
**Commit:** [sha] [message]
**Ticket moved:** "[card name]" → Dev Complete
**Card ID source:** git trailer / .current-ticket / argument
```

Or if ticket could not be moved:
```
## Go-Live Complete

**Deployed:** frontend (Vercel) + backend (Heroku)
**Commit:** [sha] [message]
**Ticket:** not moved — [reason]
```

---

## CLEANUP

After a successful go-live (deploy succeeded AND ticket moved), delete the breadcrumb:

```bash
rm -f /home/schalk/git/qic/.current-ticket
```

This prevents stale ticket IDs from leaking into the next `/ticket` cycle. The git trailer in the commit history is the permanent record.

---

## RULES

- Never move the ticket unless the deploy succeeds
- If deploy partially fails (e.g. Heroku succeeds but Vercel fails), report both outcomes and do NOT move the ticket
- If the ticket is already in Dev Complete or a later column, skip the move and note it
- The `Ticket-Id:` git trailer is the primary source of truth — `.current-ticket` is only a fallback
- Always report which source the card ID came from (trailer / file / argument) so the user can verify

$ARGUMENTS
