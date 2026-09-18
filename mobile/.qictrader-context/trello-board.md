# Trello — Project One board reference

Mobile tickets live on the same Trello board as the main monorepo: **Project One** (<https://trello.com/b/R7WQRSJ9/project-one>). They're distinguished by the `mobile` label.

## Credentials

Live in `~/.qictrader-secrets/trello.env`:

```sh
export TRELLO_API_KEY="…"
export TRELLO_API_TOKEN="…"
```

Source it before any Trello call:

```sh
source ~/.qictrader-secrets/trello.env
```

If the file doesn't exist on your machine, ask JP for the values. Never paste real key/token values into commits, chats, or this file.

## Board + list IDs

| Field | Value |
|---|---|
| Project One Board ID | `69d534f5b391f84aac55d835` |
| Project One Board (short) | `R7WQRSJ9` |
| Project One Board URL | <https://trello.com/b/R7WQRSJ9/project-one> |

### Lists (workflow columns)

| List ID | Column Name |
|---|---|
| `69d5359d3a1bfae53c391af4` | Backlog |
| `69d535a42f27c21b25086118` | Acknowledged |
| `69d535af1db0250e2a011e05` | Development (In Progress) |
| `69d535bce3d4c6f14c6aa113` | Development (Ready) |
| `69d535c8a9b1ec9d314c40b6` | Testing (In Progress) |
| `69d535fe0f2d642891e3ae1d` | QA (Failed) |
| `69d535d2149a252f13a4cb18` | Testing (Ready) |
| `69d535dbae639401ec44fed2` | Deployment |
| `69d535dd08fdd769815caa2f` | Done |

### Labels

| Label ID | Name | Color |
|---|---|---|
| `6a1022c26489d3a09be8088f` | mobile | sky |
| `69d534f5b391f84aac55d850` | frontend | blue |
| `69d534f5b391f84aac55d84b` | feature | green |
| `69d534f5b391f84aac55d84f` | backend | purple |
| `69d534f5b391f84aac55d84d` | infra | orange |
| `69d534f5b391f84aac55d84e` | bug | red |
| `69d534f5b391f84aac55d84c` | cleanup | yellow |

### Members

| Member ID | Username | Full Name |
|---|---|---|
| `69b122ea7b958c6d8b3019dc` | jpvanzyl1 | JP van Zyl |
| `69a28821437be6e8133a1c94` | alfred227 | Alfred |
| `685149e7cc0b8dce6e27736e` | christianshekleton | Christian Shekleton |

All 66 initial mobile tickets are assigned to JP only.

## Workflow

```
Backlog → Acknowledged → Development (In Progress) → Development (Ready)
                            → Testing (In Progress) → Testing (Ready)
                                → Deployment → Done
                                       ↓
                                 QA (Failed) → back to Development
```

## API examples

Base URL: `https://api.trello.com/1`

```bash
# List all cards in Acknowledged (mobile work)
curl -s "https://api.trello.com/1/lists/69d535a42f27c21b25086118/cards?key=${TRELLO_API_KEY}&token=${TRELLO_API_TOKEN}&fields=name,labels" \
  | python3 -c "
import json, sys
mobile_label = '6a1022c26489d3a09be8088f'
for c in json.load(sys.stdin):
    if mobile_label in [l['id'] for l in c.get('labels', [])]:
        print(c['name'])
"

# Move a card to a different list
curl -X PUT "https://api.trello.com/1/cards/{cardId}?idList={targetListId}&key=${TRELLO_API_KEY}&token=${TRELLO_API_TOKEN}"

# Add a comment to a card
curl -X POST "https://api.trello.com/1/cards/{cardId}/actions/comments?text=...&key=${TRELLO_API_KEY}&token=${TRELLO_API_TOKEN}"

# Add a checklist item
curl -X POST "https://api.trello.com/1/checklists/{checklistId}/checkItems?name=...&key=${TRELLO_API_KEY}&token=${TRELLO_API_TOKEN}"
```

## Mobile ticket conventions

- **ID prefix:** `MOBILE-{PHASE}-{NUM}` where PHASE ∈ {INIT, AUTH, KYC, WALLET, MARKET, TRADE, NOTIF, PROFILE, AFF, SUPPORT, SEC, PERF, TEST, LAUNCH}
- **Title format:** `MOBILE-XXX-NNN: short description`
- **Labels:** always `mobile` + one of `feature`/`infra`/`cleanup`/`bug` (UI tickets also get `frontend`)
- **Assignee:** JP for all initial 66 tickets
- **Description:** scope, out-of-scope, acceptance criteria, dependencies, tech notes

## Per-ticket workflow

1. Start ticket → move card to **Development (In Progress)**
2. Implement, test, commit
3. Push → move to **Testing (In Progress)** when build is available on TestFlight / Play internal
4. JP tests on device → moves to **Testing (Ready)** if passes, **QA (Failed)** if not
5. JP releases → moves to **Done**

Every state transition by an agent should be done via Trello API (see examples above), with a comment explaining what's in the build.

## Where ticket specs live (cached)

`.qictrader-context/tickets-export.json` has all 66 mobile ticket bodies cached for offline reference. Refresh via the Trello API if descriptions are edited (rare).
