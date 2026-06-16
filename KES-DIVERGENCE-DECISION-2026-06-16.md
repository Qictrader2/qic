# KES Fiat Support — Divergence Decision

**Trello:** #425 (Divergence: prod has KES fiat support that main intentionally removed — decide)
**Status:** DECIDED
**Date:** 2026-06-16

---

## Decision

**KES is NOT a supported fiat platform-wide. `main` is correct. Production must drop KES.**

The alignment (removing KES from prod) happens during the **Thursday production deploy**
via the standard cherry-pick/cutover — it is **not** a code change to `main` (main already
excludes KES).

---

## Background

During prod→main reconciliation we found production runs KES fiat support that `main`
deliberately removed:

- **Prod-only (frontend):** commits `bc2e7ad5` + `ca096109`
  (`HOTFIX-NGN-PERCENT-001`, KES-inclusive `SupportedFiat`).
- **main (correct):** `prices-api.ts` documents *"KES was removed because the backend has
  no variant for it and any KES offer would 422 on create"*; regression tests **#195** and
  **#418** assert KES is unsupported.

## Why main is correct (verified 2026-06-16)

The backend `FiatCurrency` enum (`qictrader-backend-rs/src/types/enums.rs`) has exactly:

```
Zar, Usd, Eur, Gbp, Ngn
```

There is **no `Kes` variant**. Therefore:

- A KES offer would **422 on create** (no backend variant to deserialize into).
- Showing KES in the frontend is a dead end that produces failed offer creation.
- Porting the two prod FE commits to main would **break the build** (the
  `SupportedFiat satisfies` constraint) and **regress tests #195 / #418**.

## What was NOT done, and why

The two prod commits (`bc2e7ad5`, `ca096109`) were **intentionally not ported** to main.
Porting them would reintroduce a broken fiat option and break the type constraint + tests.

## Action items

- [x] Decision recorded: **KES dropped (prod diverged); main is the source of truth.**
- [ ] **Thursday prod deploy:** when prod is brought in line with `main`, KES disappears
      from prod (the prod-only KES FE commits are simply not carried forward). No KES offers
      exist that need migrating (any that were attempted would have 422'd).
- [ ] If KES is ever wanted as a *real* product feature: it must start in the **backend**
      (add a `Kes` `FiatCurrency` variant + price feed), then the frontend re-adds it and
      tests #195/#418 are updated. That is a new feature ticket, not this divergence fix.
