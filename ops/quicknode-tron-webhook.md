# QuickNode Streams — TRC-20 USDT deposit webhook

Runbook for the webhook that makes Tron deposits land in seconds instead of
waiting up to a full `DEPOSIT_MONITOR_INTERVAL_SECS` (default 300s) poller tick.

**Status: built, shipped dark.** `QUICKNODE_WEBHOOK_SECRET` is unset on every
environment, so the endpoint answers 503 and deposit handling is byte-for-byte
what it was before. Enabling is gated on the cost sign-off below.

## The one-line switch

| Action | Command | Effect |
| --- | --- | --- |
| Enable | `heroku config:set QUICKNODE_WEBHOOK_SECRET=… -a <app>` | Webhook accepts deliveries; poller's Tron arm goes detection-only-off |
| Roll back | `heroku config:unset QUICKNODE_WEBHOOK_SECRET -a <app>` | Instant return to poller-only |

Setting the secret requires explicit sign-off (it is a `config:set` on a
money-moving path). Unsetting it does not — rollback should never need a meeting.

## Why enabling also changes the poller

The poller detects Tron deposits by diffing the on-chain balance, so it has no
real transaction id and writes a **synthetic** hash (`auto_deposit_tron_mainnet_…`).
The webhook writes the **real** Tron tx id. Those two never collide, so
`idx_wallet_tx_hash_type_unique` cannot deduplicate them and the same deposit
would be credited **twice**.

So when the secret is set, the poller stops *detecting* Tron deposits
(`jobs.rs`, guarded by `quicknode_webhook::webhook_mode`) but keeps running its
sweep, stamp and credit steps. It stays the backstop that finishes any pending
row a webhook delivery recorded but did not complete, which is what covers
provider downtime and dropped deliveries.

This mirrors exactly what the Solana/Helius path already does.

## Stream setup

1. QuickNode dashboard → Streams → new stream, network **Tron Mainnet**.
2. Destination: webhook, URL `https://<app-host>/webhooks/quicknode`.
3. Security: set a header. Either `Authorization` or `x-quicknode-signature` is
   accepted, and a `Bearer ` prefix is tolerated. The value must equal
   `QUICKNODE_WEBHOOK_SECRET` exactly; comparison is constant-time.
4. Filter: emit **only** inbound TRC-20 transfers to our custodial addresses.
   Filtering by address set at the provider is what keeps the credit cost down —
   see the cost model.

### Required payload shape

The stream's filter function must emit a JSON **array** of objects with exactly
these fields. This contract is pinned by
`a_stream_payload_deserialises_from_the_documented_shape` in
`src/services/quicknode_webhook.rs`; if you change one, change the other or
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

## What the endpoint does and does not reject

Only authentication failures answer non-2xx. QuickNode retries non-2xx, and
retrying a body we can never parse just burns credits and fills the log.

| Situation | Response |
| --- | --- |
| Secret not configured | 503 |
| Missing or wrong secret | 401 |
| Authenticated, unparseable body | 200, logged as an error |
| Non-USDT token, zero/negative/oversized amount, unknown address | 200, skipped |
| Duplicate delivery of a recorded tx | 200, no-op via `ON CONFLICT (tx_hash, tx_type)` |

## Cost model

QuickNode bills **30 API credits per delivered payload**. Build plan is $49/mo
as a baseline.

    monthly credits ≈ TRC-20 deposits/day × 30 × 30 days

Measure the actual rate before enabling:

```sql
SELECT count(*) / 30.0 AS trc20_deposits_per_day
FROM wallet_transactions
WHERE tx_type = 'deposit'
  AND network = 'tron_mainnet'
  AND created_at > now() - interval '30 days';
```

At 10 deposits/day that is ~9,000 credits/month, comfortably inside the Build
plan. The number that actually matters is **noise**: if the stream filter is
broader than our custodial address set, we pay 30 credits for every unrelated
TRC-20 transfer on Tron, which is a different order of magnitude entirely. Keep
the filter address-scoped and re-check the delivery count after the first week.

## Rollout

1. Staging: set the secret, replay a fixture payload, confirm one pending row
   with the real tx hash and exactly one set of ledger entries.
2. Watch `QUICKNODE:` log lines against poller activity for a week.
3. Production: set the secret. Rollback is unsetting it.

## Verifying after enabling

```bash
heroku logs -a <app> -n 500 | grep -i 'QUICKNODE:'
```

Expect `recorded pending TRC-20 USDT deposit` on new deposits and
`duplicate delivery` on redeliveries. A double credit would show as two
`ledger_entries` rows for one deposit — check with:

```sql
SELECT tx_hash, count(*)
FROM wallet_transactions
WHERE tx_type = 'deposit' AND network = 'tron_mainnet'
  AND created_at > now() - interval '1 day'
GROUP BY tx_hash HAVING count(*) > 1;
```

That must return zero rows.
