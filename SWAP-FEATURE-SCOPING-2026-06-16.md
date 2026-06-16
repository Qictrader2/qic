# Token / Coin Swap — Feasibility & Scoping Assessment

**Trello:** #367 (Scope Token / Coin Swap Feature for QicTrader)
**Status:** Scoping / feasibility (no implementation in this ticket)
**Author:** Engineering
**Date:** 2026-06-16

---

## 1. Executive summary

A swap feature lets a user convert one asset they hold into another (e.g. USDT → BTC)
without leaving QicTrader. This document assesses feasibility against the platform
**as it is actually built today** and recommends an MVP that does not require us to
take on market-making or cross-chain bridging risk before we are ready.

**Headline recommendation:** ship an MVP as a **quote-based internal swap funded from
the platform treasury**, limited initially to the assets where we already hold pooled,
liquid, spendable funds — i.e. **USDT (Tron/Ethereum)**. Do **not** launch BTC/ETH/SOL
swaps until the custody/reserves problem below is resolved, because the same on-chain
backing gap that breaks BTC withdrawals (#421/#423) would break BTC swaps.

---

## 2. Hard constraint: our custody model is not swap-ready for non-USDT

This is the single most important finding and it gates everything else.

QicTrader uses **two different custody models** (confirmed in code):

| Asset | Custody model | Spendable pooled liquidity? |
| ----- | ------------- | --------------------------- |
| USDT  | Swept to a treasury / hot wallet after deposit (`deposit_sweep::sweep_tron_usdt`, `sweep_solana_usdt_spl`) | **Yes** — treasury holds pooled funds |
| BTC, ETH, SOL | **Per-user custodial wallets**, never swept | **No** — funds sit in 84+ individual user addresses |

For BTC/ETH/SOL the **internal ledger balance ≠ on-chain balance**. Internal P2P
transfers and escrow settlements are ledger-only; the platform-wide on-chain BTC backing
is currently ~9% of the ledger liability (see #423 forensics). A swap engine that has to
*deliver* BTC to a user is subject to the exact same "no spendable UTXOs" failure as a
BTC withdrawal.

**Implication:** any swap that *outputs* BTC/ETH/SOL needs the same treasury-funded
reserve model we are introducing for withdrawals (#421/#423). Until a funded BTC/ETH/SOL
treasury exists, only **USDT-out** swaps can be settled reliably.

---

## 3. Architecture options

### A. Internal matching engine (user-to-user)
Match a user wanting USDT→BTC against another wanting BTC→USDT, settle on the ledger.

- **Pro:** no external liquidity cost; reuses our existing offer/escrow primitives.
- **Con:** swaps need *instant* fills; P2P matching is not instant and liquidity is thin.
  This is effectively what the marketplace already does (slowly). Not a good swap UX.
- **Verdict:** not suitable for MVP. Revisit only if volume justifies an internal book.

### B. Treasury-funded internal swap (RECOMMENDED for MVP)
Platform quotes a rate, debits the user's source-asset ledger balance, credits the
destination-asset ledger balance from treasury inventory. No external API on the hot path.

- **Pro:** instant, simple, fully under our control, reuses the ledger + treasury we
  already operate for USDT. Pricing reuses our existing CoinGecko/Binance feeds
  (`api::prices`, `api::gas` already consume these).
- **Con:** platform carries inventory risk and must rebalance treasury. Only safe for
  assets we hold liquid (USDT today).
- **Verdict:** **MVP path**, scoped to USDT pairs first.

### C. Third-party liquidity integration (Changelly / SimpleSwap / 1inch / THORChain / Jupiter)
Route the swap to an external provider; we take a spread.

- **Pro:** offloads liquidity + inventory risk; supports many pairs/chains.
- **Con:** adds a custody round-trip (we send their deposit address, they send to ours),
  KYC/AML surface, settlement latency (minutes, not instant), provider counterparty risk,
  and per-swap fees that compress margin. Cross-chain (THORChain) adds bridge risk.
- **Verdict:** **Phase 2.** Best as a *fallback* router behind a hybrid model (D) once MVP
  proves demand.

### D. Hybrid (internal first, external fallback)
Treasury-funded internal swap when we hold inventory; route to an external provider
otherwise.

- **Verdict:** the **target end-state**, but only after B ships and C is integrated.

---

## 4. Pricing & rate engine

We already pull live prices for ZAR/USD pairs (CoinGecko + Binance) in `api::prices`.
Swap pricing reuses this with added swap-specific logic:

- **Mid-market rate** from existing feeds.
- **Spread** (platform revenue) — config-driven, per-pair, admin-editable via
  `platform_config` (same pattern as `withdrawal_minimums`).
- **Quote expiry window** (e.g. 30–60s) — quote is a signed, time-boxed object; user must
  confirm before expiry or re-quote. Protects the platform from price drift.
- **Slippage** is **zero for internal swaps** (we are the counterparty at the quoted rate);
  slippage handling only matters for the Phase-2 external router.

---

## 5. Supported assets — phased

| Phase | From → To | Settlement | Gating dependency |
| ----- | --------- | ---------- | ----------------- |
| MVP   | USDT ↔ (display only of others) | treasury ledger | none — treasury already funded |
| MVP+  | any → USDT | treasury ledger | source asset must be debited from a *backed* balance |
| P2    | USDT → BTC/ETH/SOL | treasury-funded on-chain reserve | **funded BTC/ETH/SOL treasury** (blocked by #421/#423 reserves work) |
| P3    | arbitrary cross-asset / cross-chain | external router (THORChain/Changelly) | external integration + AML review |

NGN/ZAR fiat legs are **out of scope** for crypto-swap MVP — fiat↔crypto is the existing
P2P marketplace flow, not a swap.

---

## 6. Revenue model

- **Spread-based** (primary): bake margin into the quoted rate. Transparent "you receive X"
  figure; spread is invisible to the user but config-controlled.
- **Optional fixed swap fee** on top (like withdrawal flat fees) if spread alone is thin.
- **Network fee pass-through** for any swap that touches chain (Phase 2+).

---

## 7. Compliance, risk & operations

- **AML:** internal treasury swaps stay on-platform (lower risk than external routing).
  External routing (P2/P3) must pass provider KYC + our travel-rule posture — legal review
  required before C.
- **Treasury exposure:** B makes the platform a principal. Need a **reserve floor** and
  **rebalancing runbook** per asset (mirror the USDT treasury reserve-floor work in
  `TREAS-100`). Do not let swaps drain treasury below the withdrawal reserve floor.
- **Failed-swap recovery:** swaps must be atomic on the ledger (debit + credit in one DB
  tx) exactly like the withdrawal pending→confirmed pattern. A crash mid-swap must leave an
  auditable, reconcilable record — never a half-applied swap.
- **Reconciliation:** every swap posts paired ledger entries; treasury inventory per asset
  must reconcile against on-chain holdings on a schedule.

---

## 8. Recommended MVP scope

1. **USDT-centric, treasury-funded, quote-based internal swap.**
2. Quote endpoint (mid + spread + expiry) reusing existing price feeds.
3. Atomic ledger swap (debit source / credit destination in one DB transaction), modelled
   on the existing withdrawal pending→confirmed accounting.
4. Admin-config spread + per-asset reserve floor in `platform_config`.
5. Hard block on swaps that would output an asset without funded treasury backing
   (reuse the `select_btc_source`-style reserve guard from #421/#423).
6. Frontend: from/to selector, live quote with countdown, "you receive" transparency,
   confirm, status.

**Explicitly deferred:** external/cross-chain routing (Phase 2+), BTC/ETH/SOL output
(blocked on reserves), fiat legs.

---

## 9. Complexity & rough timeline

| Workstream | Est. |
| ---------- | ---- |
| Quote engine + spread config | ~1 week |
| Atomic ledger swap + accounting + tests | ~1.5 weeks |
| Reserve guard + treasury reserve floor reuse | ~0.5 week |
| Frontend swap flow | ~1.5 weeks |
| Admin config + reconciliation report | ~1 week |
| **MVP total (USDT internal)** | **~5–6 weeks** |
| Phase 2 external router (per provider) | +3–4 weeks |

---

## 10. Decision asks for Product / Management

1. Approve **MVP = treasury-funded internal USDT swap** (option B), deferring external
   routing and non-USDT output?
2. Confirm the platform is willing to **carry treasury inventory risk** for swaps (this is
   market-making in miniature).
3. Set initial **spread policy** and **per-asset reserve floors**.
4. Acknowledge that **BTC/ETH/SOL swap output is blocked** until the custody/reserves gap
   (#421/#423) is funded and resolved.
