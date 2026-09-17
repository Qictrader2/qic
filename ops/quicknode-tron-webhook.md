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

## Three discovery paths, and why all three stay on

| Path | Latency | Cost scales with | Turn off? |
| --- | --- | --- | --- |
| QuickNode webhook | seconds | chain activity | Only to stop paying QuickNode |
| Finalized-block scan (`TRON_BLOCK_SCAN_ENABLED`) | ~1 min | chain activity | Only if TronGrid cannot keep up |
| Address-rotation scan (deposit monitor) | ~125 min | number of custodial wallets | **Never** |

All three record under the real transaction id and land on the same unique
index, so they converge on one credit and none of them needs the others to be off.

The address-rotation scan is the reason a deposit is never permanently lost. It
reads a balance rather than a feed, so it is the only path that can find money
that arrived while every feed was broken. It is also the slowest, because its
cost grows with the number of wallets: 32 wallets per 300-second tick over 792
wallets is a ~125 minute lap, so a given Tron address is read about every two
hours. That is the number the block scan exists to fix, and it is a backstop, not
a fallback to be switched off once something faster is live.

### The finalized-block scan

`TRON_BLOCK_SCAN_ENABLED=true` starts a job on its own interval
(`TRON_BLOCK_SCAN_INTERVAL_SECS`, default 60) that reads the TRON blocks
finalized since its last pass and picks out TRC-20 USDT transfers to our
custodial addresses.

Why it is fast where the address scan is not: following blocks costs the same
whether we have 200 custodial addresses or 200,000, because the work is
proportional to chain activity. Tron produces a block roughly every 3 seconds, so
a 60-second pass reads about 20 blocks.

What makes it safe:

- **Finality is free.** Blocks come from the `walletsolidity` node, which only
  serves irreversible blocks, so a transfer read here cannot be orphaned by a fork.
- **It cannot skip.** The watermark lives in `chain_scan_cursors`, is monotonic
  (`GREATEST` on write, so a second dyno mid-deploy cannot drag it back), and
  advances one block at a time *after* that block's transfers are recorded. A
  crash mid-range resumes at the first block that was not fully written.
- **A failed read holds the line.** If a block cannot be read, or a deposit
  cannot be written, the pass stops and the watermark stays put. Skipping ahead
  would leave a permanent hole that only the address scan could ever fill.
- **A cold start takes the head, not genesis.** First run begins at the current
  finalized block. Catching up from genesis would rate-limit itself into never
  reaching the present, which is the only part users are watching, and history is
  already the address scan's job.
- **Transfers are read from event logs**, not from `TriggerSmartContract`
  calldata, so a transfer routed through another contract is still seen. Only
  `receipt.result = SUCCESS` counts: a reverted call still emits a log but moved
  no funds.

Operational notes:

- `TRON_BLOCK_SCAN_MAX_BLOCKS_PER_TICK` (default 60) chunks a backlog after an
  outage instead of attempting it in one burst.
- It adds roughly one `gettransactioninfobyblocknum` call per block, so about 20
  calls a minute at the default interval. Watch the TronGrid rate limit if you
  shorten the interval.
- The tick logs as `tron_block_scan_tick` and stays quiet when a pass finds
  nothing, so a silent log on a quiet chain is expected.
- `chain_scan_cursors` is machine state, not operator config. It is deliberately
  NOT in `platform_config` (which is admin-editable) because editing the
  watermark by hand skips deposits.

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
