---
description: QIC go-live — deploy frontend + backend and move the Trello ticket to Dev Complete.
allowed-tools: Bash, Read
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
│  4. MOVE TICKET   →  Move Trello card to Dev Complete            │
│  5. REPORT        →  Confirm what was deployed and moved         │
└──────────────────────────────────────────────────────────────────┘
```

---

## STEP 1: CHECK

```bash
git status
cd frontend && git status
cd qictrader-backend-rs && git status
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

## STEP 4: MOVE TICKET

Find the ticket ID using this priority order:

**1. From $ARGUMENTS** — scan for a pattern matching `[A-Z]+-\d+` (e.g. `ES-001`, `AUTH-004`).

**2. From git log** — commits made via `/ticket` are stamped with the ticket ID as the message prefix (`ES-001: description`). Extract it with:
```bash
git log --oneline -20 | grep -oP '^[a-f0-9]+ \K[A-Z]+-\d+(?=:)' | head -1
```
Also check submodule logs:
```bash
cd frontend && git log --oneline -20 | grep -oP '^[a-f0-9]+ \K[A-Z]+-\d+(?=:)' | head -1
cd qictrader-backend-rs && git log --oneline -20 | grep -oP '^[a-f0-9]+ \K[A-Z]+-\d+(?=:)' | head -1
```
Use the first match found across any of the three repos.

**If no ticket ID found in either place** → skip the move and tell the user:
> "Could not identify a ticket ID — card not moved. Run `/golive TICKET-ID` to move it manually."

**Once ticket ID is known**, fetch the card from the Qictrader Dev board:
```
https://api.trello.com/1/search?query={TICKET_ID}&key=d0f2319aeb29e279616c592d79677692&token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0&modelTypes=cards&idBoards=69a5bb4b56b71b138fb3f2be&cards_limit=10
```

From the results, pick the card whose `name` starts with the ticket ID. If multiple match, prefer the one NOT already in Dev Complete, Human Reviewed, or QIC Reviewed lists.

Move it to Dev Complete:
```
PUT https://api.trello.com/1/cards/{cardId}
  key=d0f2319aeb29e279616c592d79677692
  token=ATTA36ac291783275f0d046d254f4d9810898716023569970be9464b6c6a363385fd0CAB02F0
  idList=69adb791e90fb428655d9ad3
```

---

## STEP 5: REPORT

Print a clean summary:

```
## Go-Live Complete

**Deployed:** frontend (Vercel) + backend (Heroku)
**Commit:** [sha] [message]
**Ticket moved:** [TICKET-ID] "[card name]" → Dev Complete
```

Or if ticket could not be moved:
```
## Go-Live Complete

**Deployed:** frontend (Vercel) + backend (Heroku)
**Commit:** [sha] [message]
**Ticket:** not moved — [reason]
```

---

## RULES

- Never move the ticket unless the deploy succeeds
- If deploy partially fails (e.g. Heroku succeeds but Vercel fails), report both outcomes and do NOT move the ticket
- If the ticket is already in Dev Complete or a later column, skip the move and note it

$ARGUMENTS
