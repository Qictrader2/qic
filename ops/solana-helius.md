# Solana and Helius runbook: USDT-SPL deposits, rent, sponsorship and excess-rent inventory

Date: 2026-10-20. Ticket: 812 (Solana USDT gas: cache live rent and reclaim eligible excess SOL).

Operator runbook for Solana USDT-SPL deposits, Helius as the RPC and webhook
provider, the 812 rent and sponsorship changes, and the read-only excess-rent
inventory and dry run. Modelled on `ops/quicknode-tron-webhook.md`.

**Status:** the code behind this runbook is the 812 build. No operator access to
the Helius dashboard or to production was granted during that build, so every
live fact below is labelled UNVERIFIED. Do not treat an UNVERIFIED row as true
until an authorized operator has read it and filled in the date and source.

## Purpose and scope

- A custodial Solana wallet receives USDT-SPL. The deposit is detected by the
  Helius webhook (`POST /webhooks/helius`) and by the deposit monitor poller.
  The sweep moves the USDT to the treasury and needs the wallet to hold enough
  SOL, which the treasury tops up (a gas sponsorship).
- 812 changes how that top-up is sized (live rent per tick, conservative
  fallback), how it is recorded (a durable sponsorship identity) and adds a
  read-only inventory of USDT token accounts that hold SOL above their rent
  floor.
- Out of scope: any signing, broadcast or claim of excess rent. `apply` refuses.

## Facts table

"Verified on" is a date plus the evidence source, or the literal label
UNVERIFIED with the reason. Reason for every UNVERIFIED row: not verified in
build 812; needs an authorized operator read.

| Fact | Value | Verified on | Evidence source |
| --- | --- | --- | --- |
| Helius owner account | not recorded here | UNVERIFIED (not verified in build 812; needs an authorized operator read) | Helius dashboard, account page |
| Helius plan | not recorded here | UNVERIFIED (not verified in build 812; needs an authorized operator read) | Helius dashboard, billing |
| Capacity and credit limits (RPC requests per second, monthly credits, webhook count) | not recorded here | UNVERIFIED (not verified in build 812; needs an authorized operator read) | Helius dashboard, usage |
| RPC provider and endpoint identity | Helius, configured through `SOL_RPC_URL` | UNVERIFIED (not verified in build 812; needs an authorized operator read) | `heroku config` names only; never print the value |
| Configured cluster and genesis hash | not recorded here | UNVERIFIED (not verified in build 812; needs an authorized operator read) | `getGenesisHash` against `SOL_RPC_URL`, see the procedure |
| USDT mint | `USDT_SPL_MINT`, default is the constant `USDT_SPL_MINT` in `src/services/blockchain/solana.rs` | UNVERIFIED (not verified in build 812; needs an authorized operator read) | `heroku config` for an override; code for the default |
| Webhook ID(s) | not recorded here | UNVERIFIED (not verified in build 812; needs an authorized operator read) | Helius dashboard, Webhooks |
| Webhook URL path | `<APP_PUBLIC_URL>/webhooks/helius` | Route read in code on 2026-10-20 | `src/api/webhooks.rs` (route `/helius`), `src/services/helius_management.rs` |
| Watch-list size and maintenance | The backend syncs every `custodial_wallets` row with `network = 'solana_mainnet'` (the marker also covers devnet on staging) into the webhook's `accountAddresses` | Mechanism read in code on 2026-10-20; the live size is UNVERIFIED (not verified in build 812; needs an authorized operator read) | `src/services/helius_management.rs` |
| Treasury SOL address | `TREASURY_SOL_ADDRESS` | UNVERIFIED (not verified in build 812; needs an authorized operator read) | `heroku config` for the app, compared with the signer's public address |

Historical claim found in a code comment, not a verified fact: the module
comment in `src/services/deposit_credit.rs` (marked "updated 13Ycw50L,
2026-07-03") says `HELIUS_WEBHOOK_SECRET` was set on staging (release v221,
2026-06-26) and production (v830, 2026-06-27), that the Helius webhooks were
registered and active, and that 12 addresses were watched on staging. Re-verify
before relying on any of it.

## Credentials

Names only. Never put a value, or a URL carrying an API key, in a ticket, chat,
log, export or this file.

| Name | What it is |
| --- | --- |
| `SOL_RPC_URL` | Solana RPC endpoint. For Helius it carries the API key in its query string, so treat the whole value as a secret |
| `HELIUS_API_KEY` | Helius platform API key used to manage the webhook |
| `HELIUS_WEBHOOK_SECRET` | Authenticates inbound webhook deliveries; unset means the endpoint refuses |
| `TREASURY_SOL_ADDRESS` | Treasury SOL address that funds top-ups |
| `USDT_SPL_MINT` | Configuration (not a secret): the USDT mint the backend treats as deposits |

If a placeholder is ever needed, write it redacted:
`https://<helius-rpc-host>/?api-key=<HELIUS_API_KEY>`.

Apps: `qictrader-backend-staging` (staging) and `qictrader-backend-rs`
(production). Production changes need the owner's explicit go-ahead.

## Read-only verification procedure

Nothing here creates, edits or deletes a webhook, account or config var.

1. **Owner, plan, capacity.** Read them from the Helius dashboard (account,
   billing, usage). Record the date and who read it.
2. **Cluster.** Run `getGenesisHash` against the configured endpoint, passing the
   endpoint from the environment, never typed or pasted into a shared place:
   `curl -sS -X POST "$SOL_RPC_URL" -H 'content-type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"getGenesisHash"}'`
   Compare the result with the genesis hash the app expects for that
   environment (mainnet for production, devnet for staging). Take the expected
   hashes from the Solana documentation and record the source. A mismatch is a
   stop.
3. **Webhook watch list.** Read the webhook BY ID:
   `GET /v0/webhooks/{webhookID}` (the Helius API key goes in the request as
   Helius documents, not in this runbook). The LIST endpoint
   (`GET /v0/webhooks`) omits `accountAddresses`, so a list response looks like
   an empty watch list when it is not. Count the addresses in the by-ID
   response.
4. **Compare with the wallet count.** Read-only SQL:
   `SELECT count(*) FROM custodial_wallets WHERE network = 'solana_mainnet';`
   The webhook watch-list count should equal it. A shortfall means new wallets
   are only found by the poller until the list is synced.
5. Record each result in the facts table with the date and source.

## Rent and sponsorship behaviour (812)

### Live rent per tick

Each monitor tick builds one rent scope. For each (RPC endpoint fingerprint,
cluster, account data length) it reads `getMinimumBalanceForRentExemption` once
and shares the result between concurrent wallets. The endpoint fingerprint is a
hash of the complete configured endpoint string, so the same host with a
different query identity is a different key, and the URL is never logged. A
reading is live only inside its own tick; the next tick reads again, so a lower
rent is picked up and a higher one is too.

### Sweep row metadata

Read these from the sweep receipt's metadata:

| Key | Meaning |
| --- | --- |
| `sweep_gas_basis` | `measured` or `fallback` for the gas need |
| `sweep_gas_needed_lamports` | The need the top-up was sized to |
| `sweep_rent_basis` | `live` or `fallback` for the rent legs |
| `sweep_rent_min_balance_lamports` | Wallet minimum balance (0-byte account) used |
| `sweep_rent_token_account_lamports` | Token-account (165-byte) rent used, when the treasury token account was missing |
| `sweep_rent_observed_at` | When the rent was observed |
| `sweep_rent_fallback_reason` | Why the fallback applied: timeout, malformed, zero reading, RPC error and similar |
| `sweep_rent_cluster` | `genesis:<hash>` from the endpoint's `getGenesisHash`, or `unverified` when that read failed. Never assumed mainnet |

### Outage and fallback bound

If any rent read fails, the need is the largest of:

- the 2,500,000-lamport fill,
- 3,139,588 lamports when the treasury token account is missing or unknown
  (890,880 wallet minimum plus 110% of 5,000 signature fee and 2,039,280
  token-account rent),
- the highest accepted live reading or durable safety floor.

A fallback value, and any durable safety floor, is never labelled live. The
durable floors live in `solana_rent_safety_floors` (endpoint fingerprint,
cluster, data length, highest floor, updated time). A floor is only ever raised.
If the table cannot be read or written, the static bound alone applies and the
failure is logged once, without the URL.

### Sponsorship identity

Each Solana top-up is a `treasury_gas_sponsorships` row written before the
transfer is broadcast. Columns: `sent_lamports` (immutable sent cost),
`amount` (remaining parked lamports), `prepared_signature`,
`prepared_blockhash`, `broadcast_state`, `allocation_state`,
`consuming_sweep_tx_hash`, `ledger_posted_at`.

| `broadcast_state` | Meaning |
| --- | --- |
| `legacy` | Written before 812; never carried, never charged |
| `prepared` | Signed and recorded; outcome not yet known |
| `unresolved` | A send or confirmation error left the outcome unknown |
| `landed` | Confirmed on chain; the treasury debit is posted once |
| `not_landed` | Proven reverted or proven absent; `amount` is 0; nothing charged |

| `allocation_state` | Meaning |
| --- | --- |
| `legacy` | Written before 812 |
| `unallocated` | Landed funding not yet attributed to a sweep |
| `reserved` | Attributed to one in-flight sweep (`consuming_sweep_tx_hash`) |
| `finalized` | The consuming sweep settled |

### Uncertain send

While a sponsorship is `prepared` or `unresolved` the wallet gets no second
top-up and no sweep. It is proven absent only when all four hold: no signature
status, the blockhash is expired, `getTransaction` has no record, and the row is
older than the sweep reconcile window. Any read error leaves it unresolved. A
revert marks it `not_landed`.

### One-time fee attribution

Landed, unallocated funding on a wallet is reserved by the next sweep's intent
and added into that sweep's `sweep_topup_lamports`, so carried funding from a
failed earlier attempt is charged once. Settlement finalizes the reservation;
only a proven-unlandable sweep releases it.

### Read-only SQL

```sql
SELECT address, broadcast_state, allocation_state, sent_lamports, amount,
       prepared_signature, consuming_sweep_tx_hash, ledger_posted_at, created_at
FROM treasury_gas_sponsorships
WHERE broadcast_state IN ('prepared','unresolved')
ORDER BY created_at;

SELECT broadcast_state, allocation_state, count(*), sum(sent_lamports) AS sent, sum(amount) AS parked
FROM treasury_gas_sponsorships
GROUP BY 1, 2 ORDER BY 1, 2;

SELECT cluster, data_len, highest_floor_lamports, updated_at
FROM solana_rent_safety_floors ORDER BY cluster, data_len;
```

Rows older than the reconcile window that stay `prepared` or `unresolved`
deserve a human look.

## Excess-rent inventory and dry run

The `solana-excess-rent` binary lists USDT token accounts holding SOL above
their rent floor and simulates what a recovery would yield. It never signs,
broadcasts or claims, and it writes no database table.

Commands:

```bash
solana-excess-rent inventory --out-dir <dir>
solana-excess-rent dry-run   --out-dir <dir>
```

On Heroku, the way other operator binaries run:

```bash
heroku run -a <app> -- ./target/release/solana-excess-rent inventory --out-dir /tmp/excess-rent
heroku run -a <app> -- ./target/release/solana-excess-rent dry-run --out-dir /tmp/excess-rent
```

`/tmp` on a one-off dyno is lost when it exits; copy the output out inside the
same run or run it locally with the environment loaded, as the other operator
binaries are. Required environment (names only): `DATABASE_URL`, `SOL_RPC_URL`,
`TREASURY_SOL_ADDRESS`, `USDT_SPL_MINT` (defaulted), and the wallet-key
configuration the backend already uses to confirm custodial ownership.

### Report columns

| Column | Meaning |
| --- | --- |
| address | The token account |
| ATA classification | `Ata` or `NonAta` (derived from authority, token program and mint) |
| token program | The account's owning program |
| authority | The token-account authority (distinct from the program) |
| data length | Account data length in bytes |
| lamports | Current SOL balance |
| exact USDT | Exact token amount in minor units |
| rent floor | Live minimum balance for that data length |
| gross | Lamports above the floor |
| predicted fee | Fee for the candidate message |
| net | Gross minus predicted fee |
| eligibility and reason | `Eligible`, `Excluded(reason)` or `Unknown(reason)` |

Report header fields: generated time, cluster genesis hash, configured mint,
treasury address, wallets scanned, wallets with no USDT account, accounts found,
scan status (`complete` or `partial`), totals for eligible rows, excluded and
unknown counts with unknown totals kept separate, `simulation_only: true`, and
signer policy status. Signer policy is always reported unverified: the deployed
policy is not checked and no signature is requested.

An Unknown input never counts as zero cost or as eligible.

Instruction support. The candidate is `WithdrawExcessLamports` (data `[38]`,
accounts source, destination, authority), with the treasury as fee payer and
destination. `inventory` never simulates, so rows that would otherwise qualify
show `Unknown(instruction_support_unverified)`. `dry-run` simulates each
candidate unsigned (`sigVerify=false`). A row is eligible only when the
simulation succeeds, the account stays open at its rent floor with the same
USDT, and the treasury gains gross minus fee. A rejected instruction is
`Excluded(instruction_unsupported_on_cluster)`. The report records the token
program id and genesis hash for which support was shown. A simulation is dry-run
evidence only, never a recovery.

## Apply refusal

`solana-excess-rent apply` always exits nonzero (code 2) on every environment,
with any flags, before it reads configuration, opens a database or RPC
connection, or builds a key or signer. No flag or environment variable changes
that. It stays that way until:

1. card 812 records the materiality decision and the accounting attribution
   decision, and
2. a later change enables apply, together with specific execution
   authorization.

Decision record template (fill in on card 812, never here with secrets):

| Item | Entry |
| --- | --- |
| Materiality threshold decision (minimum net lamports per account and in total) | |
| Approved destination | `TREASURY_SOL_ADDRESS` (confirm it matches the signer's address) |
| Accounting and customer-liability attribution of recovered rent | |
| Execution authorization (scope, environment, account list or report hash) | |
| Decided by, and date | |

Mandatory later live-recovery QA, before any recovery is trusted:

- the transaction carries the correct signatures for the authorities involved,
- the rent floor and the USDT amount are preserved on the source account,
- one audit record per recovery,
- partial and uncertain-send resumption works without a second recovery,
- source, destination, fee and receipt reconcile.

## Post-run reconciliation

Read-only checks, taken before and after an inventory or dry run. Each pair must
match:

```sql
SELECT count(*) FROM wallet_transactions;
SELECT count(*) FROM ledger_entries;
SELECT count(*) FROM treasury_gas_sponsorships;
```

Also compare SOL and USDT balances for the custodial wallets and the treasury
(`getBalance`, token balance) before and after. Any change during a run is an
incident, not noise. Recovered rent, if it is ever executed, keeps its
attribution as recovered rent; it is never called profit or a refund.

## Secret hygiene

Run over every export, log excerpt and evidence file before it leaves the
machine; any hit is a stop:

```bash
grep -nEi 'api-key=|api_key=' <files>
grep -nE '\b[0-9a-fA-F]{64,}\b' <files>
grep -nE '\b[1-9A-HJ-NP-Za-km-z]{87,88}\b' <files>
```

Expected: no output. (The second pattern also matches long hex hashes; read
the hit before deciding.)
