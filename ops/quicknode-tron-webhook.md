# QuickNode Webhooks — TRC-20 USDT deposit trigger

Runbook for the webhook that makes Tron deposits land in seconds instead of
waiting for the deposit scanner to come round to the address again.

**Status: built, shipped dark.** `QUICKNODE_WEBHOOK_SECRET` is unset on every
environment, so `POST /webhooks/quicknode` answers 503. Deposit handling is
exactly what it was before, on every environment.

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
| `QUICKNODE_API_KEY` | QuickNode **platform API key** (dashboard, API keys). Can modify account resources, treat as the more dangerous credential | Strongly recommended | Watched-address list is not synced; new custodial addresses are invisible to the webhook until added by hand (the scanners still find them) |
| `QUICKNODE_TRON_ADDRESS_LIST` | Name of the Key-Value Store list the webhook template filters on | Only if the list is not named `qic_tron_custodial_addresses` | Defaults to `qic_tron_custodial_addresses` |
| `TRON_BLOCK_SCAN_ENABLED` | Our own switch, no credential | Independent of QuickNode | Finalized-block scan is off; the webhook and the wallet-rotation scan still run |

Order: create the KV list, create the webhook pointing at it, set
`QUICKNODE_API_KEY` (boot syncs every custodial address into the list), then
set `QUICKNODE_WEBHOOK_SECRET`.

Current state (2026-09-25): none of the three QuickNode vars is set on staging
or production. `TRON_BLOCK_SCAN_ENABLED=true` on staging, unset on production.

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
| QuickNode webhook | `QUICKNODE_WEBHOOK_SECRET` | seconds | chain activity matching the KV list |
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

1. QuickNode dashboard → **Webhooks** → Create Webhook.
2. Network: **Tron Mainnet**. See the Nile note below before planning a staging
   test.
3. Template: the Tron **Wallet Activity Monitor**.
4. Destination URL: `https://<app-host>/webhooks/quicknode`.
5. Security token: generate a high-entropy value, set it here **and** as
   `QUICKNODE_WEBHOOK_SECRET`. Do not let QuickNode auto-generate it unless you
   are going to copy it straight into Heroku config.
6. Compression: `none` is preferred. `gzip` also works (we inflate before
   verifying), but `none` keeps the failure modes simpler.
7. Watched addresses: point the template at a **Key-Value Store list** named to
   match `QUICKNODE_TRON_ADDRESS_LIST` (default
   `qic_tron_custodial_addresses`). Do not paste addresses inline — see below.

### Nile testnet is not available on Webhooks

QuickNode's Webhooks supported-network table lists Tron with **no testnet**.
Nile is available for RPC only. Every other chain that has a testnet lists it
(Solana: Devnet, Testnet; Stellar: Testnet), so the omission is real rather than
a documentation gap.

Consequences for the rollout plan:

- A Nile webhook for staging **cannot be created**. The plan of "test on Nile,
  then promote" does not work as written.
- Staging's `TRON_RPC_URL` points at Nile, so a mainnet webhook's deliveries
  would reference transactions staging's RPC cannot confirm — and the receipt
  check would (correctly) refuse them.
- What can be verified without mainnet money: everything except a real
  delivery. The signature contract, the payload contract, deduplication,
  ordering, restart behaviour and forgery rejection are all covered by the
  automated tests, which run against real Postgres.
- What genuinely needs a live delivery: point a webhook at **production** with
  a small real deposit, or ask QuickNode support to enable a Tron testnet (the
  docs invite template and network requests).

Flag this to whoever owns the QuickNode account before the free trial is spent.

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
{ "addItems": ["T…", "T…"] }
```

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
curl -sS https://api.quicknode.com/kv/rest/v1/lists/qic_tron_custodial_addresses \
  -H "x-api-key: $QUICKNODE_API_KEY" | jq 'length'
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

So before recording, the handler reads the recipient's confirmed TRC-20
transfers from TronGrid and requires the claimed transaction id, contract,
recipient and **amount** to match. Amounts are summed across transfers sharing a
transaction id. A claim that does not match is logged and dropped; if it turns
out to be real, the scanner records it later.

Log line to look for: `QuickNode claim does not match a confirmed TRC-20
transfer`. A burst of these means the webhook's filter and our contract have
diverged — check the payload shape first.

### Required payload shape

The template must emit a JSON **array** of objects with these fields. Pinned by
`a_stream_payload_deserialises_from_the_documented_shape` in
`src/services/quicknode_webhook.rs`; change one, change the other, or deposits
stop silently.

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
| Claim with no matching confirmed transfer | 200, skipped and warned |
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

### Staging proof without a live webhook

QuickNode cannot deliver Nile transactions, so staging is proven by replaying
a correctly signed delivery with `ops/scripts/quicknode_tron_replay.py`. It
signs exactly as QuickNode does and never prints the token.

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
4. Watch for a week alongside the scanner. Deposits found by both paths must
   still show one row and one credit.
5. Production is a separate webhook with its own security token and API key.
   Never reuse the staging pair.

## Verifying after enabling

```bash
heroku logs -a <app> -n 500 | grep -iE 'QUICKNODE'
```

Expect `recorded pending TRC-20 USDT deposit` on new deposits, `duplicate
delivery` on redeliveries, and `QUICKNODE_MGMT` lines at boot. A double credit
would show as two rows for one transaction id:

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
