# QuickNode Webhooks — TRC-20 USDT deposit trigger

Runbook for the webhook that makes Tron deposits land in seconds instead of
waiting for the deposit scanner to come round to the address again.

**Status (2026-09-26): live on staging (Tron Nile), not enabled on production.**
Staging has its own webhook, KV list and config. Production still answers 503
on `POST /webhooks/quicknode` until JP approves the staging evidence and the
production webhook is created with its own key and token.

> This runbook was rewritten for the **Webhooks** product (not Streams). The
> two differ in ways that matter here: authentication, the address-list
> mechanism, and what a REST update can silently break. Ticket: xBiZvcC3.

## The one-line switch

| Action | Command | Effect |
| --- | --- | --- |
| Enable | `heroku config:set QUICKNODE_WEBHOOK_SECRET=… -a <app>` | Endpoint starts accepting deliveries |
| Roll back | `heroku config:unset QUICKNODE_WEBHOOK_SECRET -a <app>` | Endpoint returns to 503 |

Setting the secret requires explicit sign-off (it is a `config:set` on a
money-moving path). Unsetting it does not — rollback should never need a meeting.

The value must equal the **destination's security token** in the QuickNode
dashboard, character for character. It is the HMAC key, not a password.

## Credentials and config checklist

One set per environment. Never reuse the staging values in production. Store
the values in Enpass under the `developers@qictrader.com` QuickNode account,
never in Trello, source control, chat, or logs.

| Config var | Where it comes from | Required? | Effect when unset |
| --- | --- | --- | --- |
| `QUICKNODE_WEBHOOK_SECRET` | Webhook destination **security token** (dashboard, Webhooks, destination) | Yes, it is the on/off switch | Endpoint answers 503, nothing else changes |
| `QUICKNODE_API_KEY` | QuickNode **platform API key** (dashboard, API keys). It must include the **Key-Value Store** and **Webhooks** applications with the **Admin** role. Permissions cannot be edited after a key is created, so a key missing Key-Value Store has to be replaced (the KV calls answer 403). Can modify account resources, treat as the more dangerous credential | Strongly recommended | Watched-address list is not synced; new custodial addresses are invisible to the webhook until added by hand (the scanners still find them) |
| `QUICKNODE_TRON_ADDRESS_LIST` | Name of the Key-Value Store list the backend syncs and the webhook reads | Yes on staging (`qic_tron_custodial_addresses_staging`); production uses the default | Defaults to `qic_tron_custodial_addresses` |
| `TRON_BLOCK_SCAN_ENABLED` | Our own switch, no credential | Independent of QuickNode | Finalized-block scan is off; the webhook and the wallet-rotation scan still run |

Order: set `QUICKNODE_API_KEY` and `QUICKNODE_TRON_ADDRESS_LIST` (boot creates
the KV list and syncs every custodial address into it), create the webhook
pointing at the list, then set `QUICKNODE_WEBHOOK_SECRET`. The setup script
below does all of it in that order.

Current state (2026-09-26):

| Environment | Webhook | Network | KV list | Config |
| --- | --- | --- | --- | --- |
| Staging (`qictrader-backend-staging`) | `qictrader-staging-tron-nile`, active | `tron-nile` | `qic_tron_custodial_addresses_staging` | all three QuickNode vars set; `TRON_BLOCK_SCAN_ENABLED=true` |
| Production (`qictrader-backend-rs`) | not created | `tron-mainnet` (planned) | `qic_tron_custodial_addresses` (planned) | no QuickNode vars; `TRON_BLOCK_SCAN_ENABLED` unset (off) |

Production needs a new API key with Key-Value Store + Webhooks (Admin). The
existing `qictrader-production-webhooks` key has Webhooks only.

## The scanner keeps running

Enabling the webhook no longer changes how deposits are discovered. Both the
webhook and the scanner record a deposit under its **real Tron transaction id**,
so `idx_wallet_tx_hash_type_unique` deduplicates whichever arrives second and
one transfer produces one credit either way.

That matters for a specific failure mode: a provider that stops delivering can
no longer hide a deposit, because the scanner never stood down.

Previously the scanner detected by balance-diff and had no transaction id, so it
invented one (`auto_deposit_tron_mainnet_…`). A synthetic hash can never collide
with a real one, the index could not deduplicate them, and running both paths
would have credited the same deposit twice — which is why enabling the webhook
used to require turning the scanner's Tron detection off. That coupling is gone.

Proven by `tests/quicknode_tron_webhook_xbizvcc3.rs`:
`the_scanner_and_the_webhook_converge_on_a_single_deposit` and
`the_order_the_two_paths_arrive_in_does_not_change_the_outcome`.

## Three discovery paths, all live

| Path | Switch | Latency | Cost scales with |
| --- | --- | --- | --- |
| QuickNode webhook | `QUICKNODE_WEBHOOK_SECRET` | delivered in seconds, recorded once the block is solid (about a minute) | chain activity matching the KV list |
| Finalized-block scan (#821) | `TRON_BLOCK_SCAN_ENABLED` | about a minute | blocks produced, not wallet count |
| Wallet-rotation scan (deposit monitor) | always on | one lap of every Tron wallet per tick | number of wallets |

All three record under the real transaction id and converge on one credit. The
wallet-rotation scan is the permanent backstop: it reads a balance rather than a
feed, so it is the only path that finds money that arrived while every feed was
broken. It is not a fallback to retire once something faster ships.

The block scan reads from the `walletsolidity` node, so every block it sees is
already final. Its watermark (`chain_scan_cursors`) is durable and advances one
block at a time after that block's transfers are recorded, so a crash resumes
rather than skips, and a failed read holds the watermark instead of leaving a
hole. Transfers are read from event logs, so contract-routed transfers are seen,
and only `SUCCESS` receipts count. Rollback is `heroku config:unset
TRON_BLOCK_SCAN_ENABLED`; the watermark is kept and resumes on re-enable.

## Webhook setup

Use `ops/scripts/quicknode_tron_webhook_setup.py`. It is idempotent: run it
again to repair drift, and it only changes what differs.

```bash
source ~/.qictrader-secrets/load-all.sh
# The API key is read from a 0600 file holding just the key, never from argv:
#   ~/.qictrader-secrets/quicknode_api_key_staging
#   ~/.qictrader-secrets/quicknode_api_key_production
python3 ops/scripts/quicknode_tron_webhook_setup.py --env staging status
python3 ops/scripts/quicknode_tron_webhook_setup.py --env staging apply
```

`apply` does this, in order, and never prints a secret:

1. Reads the custodial Tron addresses from the app's database and adds them to
   the environment's KV list (creating it if needed).
2. Sets `QUICKNODE_API_KEY` and `QUICKNODE_TRON_ADDRESS_LIST` on the app if
   they differ, then waits for `/health` after the restart. Boot re-syncs the
   list.
3. Reuses the app's `QUICKNODE_WEBHOOK_SECRET` as the security token, so a
   rerun never rotates it. Otherwise it generates one, keeps a copy in
   `~/.qictrader-secrets/quicknode_webhook_secret_<env>` (0600) and sets it.
   Then it requires an unsigned probe to answer `401`, which proves the
   endpoint is armed.
4. Creates the webhook (or updates the existing one's template) with template
   `evmWalletFilter` and `{"walletsListName": "<list>"}`, activates it with
   `startFrom: latest`, and verifies it is active and its token matches the
   app's.

| Environment | App | Webhook name | Network id | KV list | Destination |
| --- | --- | --- | --- | --- | --- |
| `staging` | `qictrader-backend-staging` | `qictrader-staging-tron-nile` | `tron-nile` | `qic_tron_custodial_addresses_staging` | `https://staging-api.qictrader.com/webhooks/quicknode` |
| `production` | `qictrader-backend-rs` | `qictrader-production-tron-mainnet` | `tron-mainnet` | `qic_tron_custodial_addresses` | `https://api.qictrader.com/webhooks/quicknode` |

The script refuses to proceed when more than one webhook matches the name, or
when the existing webhook is on a different network (a template update cannot
move networks). `delete-webhook --id <id>` removes a stray one.

Setting config on production needs JP's explicit go-ahead, and production must
never be configured in the same step as staging.

Notes from setting it up:

- **Nile is available on Webhooks** as network id `tron-nile`, even though the
  public supported-network table lists Tron without a testnet. The REST API's
  network list (returned in the 400 for an invalid network) includes it.
- The template delivers blocks with receipts. The backend keeps successful
  receipts (`status` `0x1`) and USDT `Transfer` logs, and converts the 20- or
  32-byte hex addresses to base58 itself. See "Required payload shape".
- Cloudflare in front of `api.quicknode.com` rejects Python urllib's default
  User-Agent (error 1010). The script sends its own.
- The custom filter in `ops/quicknode/tron-usdt-deposit-filter.js` is not used
  by the current webhooks. It stays as a fallback if the template is ever
  withdrawn.

## Watched addresses sync themselves

A webhook only fires for addresses it was told to watch, so every custodial
address created after setup would otherwise be invisible to it — silently, with
the deposit just falling back to the slow path.

`src/services/quicknode_management.rs` keeps the list in step. It reconciles at
boot and again whenever a custodial Tron wallet is created, and it needs
`QUICKNODE_API_KEY` set (the platform API key, `x-api-key` — a different, more
dangerous credential than the security token).

It writes to the **KV Store list**, never to the webhook:

```
PATCH https://api.quicknode.com/kv/rest/v1/lists/<QUICKNODE_TRON_ADDRESS_LIST>
{ "addItems": ["0x0c31bf48…", "0x…"] }
```

Items are the **20-byte EVM form** of each Tron address: lowercase `0x` plus
40 hex characters, with the `41` prefix dropped. That is what the
`evmWalletFilter` template matches against. The backend converts from base58
(`TB5ghrgm…` becomes `0x0c31bf48…`), and re-adding an existing item is a no-op.

Two reasons it works this way:

1. Template-based webhooks cannot have their filter rewritten over REST.
   QuickNode's documented answer to a changing address set is the KV list
   indirection.
2. **Updating a webhook can rotate the security token.** Any REST call that
   sends `destination_attributes` without `security_token` makes QuickNode
   generate a new one. That instantly invalidates every signature we verify and
   stops Tron deposits dead, with nothing in our logs but 401s we caused
   ourselves. Editing a list cannot do that. If you ever do PATCH the webhook by
   hand, always include `security_token`.

The sync is **additive**. It never removes an address, because a custodial
address stays deposit-capable for as long as the user exists: dropping one turns
a real future deposit into a missed one, while a stale extra entry costs a
filtered block. Addresses upstream that we do not own are logged and left alone.

Check what is watched:

```bash
python3 ops/scripts/quicknode_tron_webhook_setup.py --env staging status
```

or by hand (the response is `{"code", "msg", "data": {"items": [...]}, "cursor"}`):

```bash
curl -sS https://api.quicknode.com/kv/rest/v1/lists/qic_tron_custodial_addresses_staging \
  -H "x-api-key: $QUICKNODE_API_KEY" | jq '.data.items | length'
```

Compare against the database:

```sql
SELECT count(*) FROM custodial_wallets WHERE network = 'tron_mainnet';
```

The list count should be greater than or equal to the row count. Less means the
sync is not running — check for `QUICKNODE_MGMT:` lines at boot.

## How a delivery is authenticated

HMAC-SHA256 over the UTF-8 concatenation `nonce + timestamp + payload`, keyed
with the destination's security token, hex encoded, compared in constant time.

| Header | Purpose |
| --- | --- |
| `x-qn-nonce` | Unique per delivery |
| `x-qn-timestamp` | When QuickNode signed it |
| `x-qn-signature` | The digest to match |

Two things worth knowing:

- **The payload is signed, not just the secret.** The previous implementation
  compared a static shared secret, which proved the caller knew a token but said
  nothing about the body. Since the amount and recipient in that body decide who
  gets credited, the body has to be covered.
- **The timestamp is checked** against a 300s window
  (`MAX_TIMESTAMP_SKEW_SECS`). Without it a captured delivery stays replayable
  forever and the nonce means nothing. If deliveries start failing with
  `StaleTimestamp`, suspect dyno clock drift before suspecting an attack.
- If compression is on, the digest is over the **uncompressed** JSON. We inflate
  first; verifying the gzip octets would reject every genuine delivery.

## The chain is checked before anything is recorded

A valid signature proves QuickNode sent the bytes unaltered. It does not prove
the chain agrees — a provider bug, a bad filter, or a stale address list can all
deliver a claim with no confirmed transfer behind it.

So before recording, the handler reads the recipient's TRC-20 transfers from
TronGrid **with `only_confirmed=true`** and requires the claimed transaction
id, contract, recipient and **amount** to match. Amounts are summed across
transfers sharing a transaction id. Webhook rows are tagged
`sighted_unconfirmed: false` and skip the 821 solidity hold, so only a solid
block may satisfy the check. TronGrid's default listing also returns transfers
from blocks that are not solid yet, which is why the flag is explicit.

QuickNode delivers at block inclusion, but TronGrid lists the transfer as
confirmed only once the block is solid, about 19 blocks (roughly a minute)
later. The check at delivery time therefore normally misses. The handler
answers 200 at once and re-checks in the background at 15, 30, 45, 60, 80, 110,
170 and 290 seconds (`CLAIM_RECHECK_DELAYS`, one TronGrid call per step). Once
it matches, the deposit is recorded, swept and credited as usual. If the
scanner records it first, the webhook stands down. After the last step, or if
the dyno restarts mid-schedule, the scanners are the backstop.

Transfers to an address we do not custody (every sweep out of a custodial
wallet goes to the hot wallet, and the wallet filter fires for those too) are
settled from the database without calling TronGrid.

Log lines:

| Line | Meaning |
| --- | --- |
| `QuickNode claim is not confirmed on chain yet — re-checking in the background` | Normal for every new deposit |
| `QuickNode claim confirmed on chain and recorded` | The webhook recorded it; sweep and credit follow |
| `QuickNode claim confirmed on chain; already recorded by another path` | The block scan or poller got there first |
| `QuickNode claim never matched a confirmed TRC-20 transfer` | Nothing confirmed within about five minutes. A burst means the payload and the chain disagree; check the payload shape first |

### Required payload shape

The backend accepts two shapes:

1. **The `evmWalletFilter` template's delivery** (what both webhooks send):
   blocks with transaction receipts. Only receipts with `status` `0x1` count,
   only logs whose first topic is the ERC-20 `Transfer` signature, and the
   recipient and contract are converted from hex to base58. Pinned by
   `the_wallet_template_payload_yields_the_real_nile_deposit` (a captured Nile
   delivery) and `a_reverted_receipt_produces_no_transfer` in
   `src/services/quicknode_webhook.rs`.
2. **A flat JSON array**, which is what the custom filter emits:

The flat shape is an array of objects with these fields. Pinned by
`a_stream_payload_deserialises_from_the_documented_shape` and
`the_flat_transfer_list_still_parses`; change one, change the other, or
deposits stop silently.

```json
[
  {
    "transaction_id": "d1f2e3…",
    "token_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to": "TRecipientAddress…",
    "from": "TSenderAddress…",
    "value": "2500000"
  }
]
```

`value` is a decimal **string** in token minor units, because Tron amounts are
not bounded by `i64` in the general case. Anything that will not parse into
`i64` is skipped rather than guessed at.

## What the endpoint rejects

Only transport and authentication failures answer non-2xx. QuickNode retries
non-2xx, and retrying a body we can never parse just burns credits.

| Situation | Response |
| --- | --- |
| Security token not configured | 503 |
| Missing signing headers, bad digest, stale timestamp | 401 |
| Declared `gzip` but not inflatable | 400 |
| Authenticated, unparseable body | 200, logged as an error |
| Non-USDT token, zero/negative/oversized amount, unknown address | 200, skipped |
| Claim not confirmed on chain yet | 200, re-checked in the background for about five minutes |
| Transfer to an address we do not custody (for example a sweep) | 200, settled without calling TronGrid |
| Duplicate delivery of a recorded tx | 200, no-op via `ON CONFLICT (tx_hash, tx_type)` |

## Cost model

Free for 30 days, then **$49/month**. Keep the account on the free tier during
development and upgrade before production, per Christian.

Billing is per matching block, so the number that matters is **filter noise**,
not our deposit rate. A filter broader than our custodial address set bills for
unrelated Tron activity, which is a different order of magnitude. Keep the KV
list as the only address source and re-check delivery volume after week one.

Current deposit rate:

```sql
SELECT count(*) / 30.0 AS trc20_deposits_per_day
FROM wallet_transactions
WHERE tx_type = 'deposit'
  AND network = 'tron_mainnet'
  AND created_at > now() - interval '30 days';
```

## Rollout

1. Confirm the automated suite passes: `cargo test --lib quicknode` and
   `cargo test --test quicknode_tron_webhook_xbizvcc3` (the latter needs
   `DATABASE_URL`).
2. Create the webhook and the KV list. Set `QUICKNODE_API_KEY` first so the
   address sync populates the list at boot.
3. Set `QUICKNODE_WEBHOOK_SECRET`. Confirm a delivery is accepted, not 401'd.

### Live staging proof (2026-09-26)

Real Nile USDT transfers from `testusera` (sent through the staging withdraw
API) to `testuserb`'s custodial address `TB5ghrgmttWbWZkh72mx6uQmFVdk1kCjNs`.
The signup sync had added that address to the watch list 74 ms after the wallet
was created.

| Run | Setup | Transfer | What happened | Result |
| --- | --- | --- | --- | --- |
| 1 | Before #249, block scan on | `8c2d51f5…`, 5 USDT | QuickNode delivered within a second and the signature and payload were fine, but the single chain check ran before TronGrid had indexed the block and dropped the claim. The block scan recorded it 4 s later. Held below the 10 USDT minimum | Found the bug fixed in backend #249 |
| 2 | After #249, `TRON_BLOCK_SCAN_ENABLED=false` (as on production) | `eebe2897…`, 10 USDT | Delivered 08:21:18 UTC, not confirmed yet, re-check started. Confirmed and recorded by the webhook at 08:22:19 (61 s). 15 USDT swept at 08:22:32 (`8dd442bf…`), both deposits credited net of the sweep fee (3.314565 + 6.629130 USDT) | 1 row each; this one tagged `quicknode_webhook` with `sighted_unconfirmed=false`; 3 ledger rows each |
| 3 | After #249, block scan on | `7be1caf8…`, 10 USDT | Block scan recorded it at head in 6 s. The webhook re-check confirmed it at 61 s and stood down ("already recorded by another path"). Swept and credited at 08:24:58 | 1 row, one credit |

`testuserb`'s USDT balance went 2013.611111, then 2023.554806, then
2033.554806. With the block scan off, a deposit went from broadcast to credited
in about 75 seconds through the webhook alone.

### Replaying a signed delivery

The live staging webhook is the main proof (see "Live staging proof" below).
Replaying a correctly signed delivery with `ops/scripts/quicknode_tron_replay.py`
still covers the cases a live delivery cannot stage on demand: a forged
signature, a redelivery and a wrong amount. It signs exactly as QuickNode does
and never prints the token.

1. Set a staging-only security token (sign-off required):
   `heroku config:set QUICKNODE_WEBHOOK_SECRET=<random 32+ chars> -a qictrader-backend-staging`.
2. Load it into the shell without echoing:
   `export QUICKNODE_WEBHOOK_SECRET="$(heroku config:get QUICKNODE_WEBHOOK_SECRET -a qictrader-backend-staging)"`.
3. Pick a real Nile USDT deposit already recorded on staging. Staging stores
   Nile deposits under the `tron_mainnet` enum value; `amount` is in minor
   units and goes straight into `--value`:
   ```sql
   SELECT wt.tx_hash, cw.address, wt.amount
   FROM wallet_transactions wt
   JOIN custodial_wallets cw ON cw.user_id = wt.user_id AND cw.network = wt.network
   WHERE wt.tx_type = 'deposit' AND wt.network = 'tron_mainnet'
     AND wt.tx_hash NOT LIKE 'auto_deposit_%'
   ORDER BY wt.created_at DESC LIMIT 1;
   ```
   Record the ledger row count before replaying:
   `SELECT count(*) FROM ledger_entries WHERE reference = '<tx>';`
4. Replay it with a **wrong** signature: expect `401`.
5. Replay it correctly signed: expect `200` and the log line `duplicate
   delivery`. Then confirm the transaction still has exactly one row and the
   ledger count is unchanged from step 3:
   ```sql
   SELECT count(*) FROM wallet_transactions WHERE tx_hash = '<tx>' AND tx_type = 'deposit';
   SELECT count(*) FROM ledger_entries WHERE reference = '<tx>';
   ```
6. Replay a claim whose amount differs from the chain: expect `200` and the
   warning `QuickNode claim does not match a confirmed TRC-20 transfer`, with no
   new row.
7. Unset the staging secret afterwards unless the week-long watch is running.

A fresh Nile deposit to a staging custodial address, replayed before the
scanners reach it, additionally proves the first-arrival path end to end.

Last run 2026-09-25 against tx `40e74e47…` (11 USDT, 1 row, 3 ledger rows):
wrong signature 401; signed 200 and chain check passed; same delivery again
200; wrong amount 200 with the mismatch warning. Afterwards still 1 row and 3
ledger rows. The staging secret was left set so the dashboard test delivery in
step 8 of the setup can be checked.
4. Watch for a week alongside the scanner. Deposits found by both paths must
   still show one row and one credit.
5. Production is a separate webhook with its own security token and API key.
   Never reuse the staging pair.

## Verifying after enabling

```bash
heroku logs -a <app> -n 500 | grep -iE 'QUICKNODE'
```

Expect, for each new deposit, `processing QuickNode TRC-20 transfers`, then
`not confirmed on chain yet — re-checking in the background`, then about a
minute later either `confirmed on chain and recorded` (followed by the sweep
and `credited deferred USDT deposit`) or `already recorded by another path`.
Expect `duplicate delivery` on redeliveries and `QUICKNODE_MGMT` lines at boot
and when a custodial Tron wallet is created. A double credit would show as two
rows for one transaction id:

```sql
SELECT tx_hash, count(*)
FROM wallet_transactions
WHERE tx_type = 'deposit' AND network = 'tron_mainnet'
  AND created_at > now() - interval '1 day'
GROUP BY tx_hash HAVING count(*) > 1;
```

That must return zero rows.

Synthetic hashes should stop appearing for newly detected deposits. Existing
ones are historical and are left alone:

```sql
SELECT count(*) FROM wallet_transactions
WHERE tx_type = 'deposit' AND network = 'tron_mainnet'
  AND tx_hash LIKE 'auto_deposit_tron_mainnet_%'
  AND created_at > now() - interval '1 day';
```
