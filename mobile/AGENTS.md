# QicTrader Mobile — Agent Rules

You are working on the QicTrader **mobile** repo. This is a separate codebase from the main `Qictrader` monorepo. It contains the React Native (Expo) app for iOS + Android. It consumes the existing Rust backend's HTTP API — **the backend lives elsewhere and is read-only from this repo**.

## Sister repo locations (for read-only reference)

The main monorepo is at **`/Users/jpvanzyl/Workspaces/Qictrader`** on this machine. Treat it as read-only:

- `Qictrader/frontend/` — Next.js web app. You may read source for API contract reference, design patterns, business logic, copy. **You may NOT modify anything in there from this Cursor window.**
- `Qictrader/qictrader-backend-rs/` — Rust backend. Read `src/types/enums.rs`, `src/models/`, and `src/api/*.rs` to know exact field names, request/response shapes, and validation rules.
- `Qictrader/qictrader-backend-rs/docs/intended-entity-state-machines.md` — design intent for trade/escrow/wallet state machines.
- `Qictrader/qictrader-backend-rs/docs/as-built-state-machines.md` — actual implementation reality.
- `Qictrader/DESIGN.md` — single source of truth for visual identity.
- `Qictrader/architecture/SECURITY-CARRYFORWARD-FOR-NEW-APPS.md` — see `.qictrader-context/security-carryforward.md` for the snapshot.
- `Qictrader/ops/trello.md` — Trello credentials + board IDs.

If you discover the web repo has changed since the last sync, **stop and ask JP** whether to update this repo's synced files via `./scripts/sync-from-web.sh`.

## Feature parity is non-negotiable

Every feature this app implements must behave **identically** to the web version. If web behaves a certain way, mobile matches. If mobile diverges, that's a bug — not a "mobile-native improvement" unless explicitly approved by JP in a Trello card.

Before implementing any feature:

1. Read the equivalent web implementation in `/Users/jpvanzyl/Workspaces/Qictrader/frontend/src/`
2. Read the backend handler in `/Users/jpvanzyl/Workspaces/Qictrader/qictrader-backend-rs/src/api/`
3. Match field names, validation rules, error messages, edge cases exactly
4. Match copy / wording (English; defer i18n until QicTrader web ships it)
5. Adapt the rendering layer for RN — same logic, different primitives

## Stack

- **Expo SDK 52+** managed workflow
- **React Native 0.76+** New Architecture enabled
- **TypeScript** strict mode
- **NativeWind v4** (Tailwind for RN, mirrors web)
- **React Navigation v7**
- **Redux Toolkit + Zustand + React Query v5**
- **expo-secure-store** for tokens (NEVER AsyncStorage)
- **expo-local-authentication** for biometric
- **expo-notifications** for push (FCM + APNs)
- **socket.io-client** for realtime
- **bun** package manager
- **Maestro** for E2E
- **EAS Build + EAS Submit** for native builds and store uploads
- **Sentry** for crashes
- **GA4** for analytics

## Engineering principles (carried from main monorepo)

### Types first

- Define types/interfaces before writing logic
- Use enums/unions for domain concepts (status, currency, payment type) — never raw strings
- Make invalid states unrepresentable: if it type-checks, it's valid
- Exhaustive matching — no catch-all `default:` that silently swallows new variants

### Error handling

- Every error path must be explicitly handled — no swallowing errors
- TypeScript: every API call handles both success and error; no empty `catch {}`
- Design errors as typed discriminated unions, not string messages
- No `.then().catch(() => {})` — every catch logs to Sentry AND surfaces to user

### Auth & security

- Every screen that loads user-specific data must verify the user is authorized for THAT resource (not just authenticated)
- Tokens live in **`expo-secure-store` only** — never `AsyncStorage`, never `localStorage` shim
- Biometric re-auth required before: withdraw, change password, disable 2FA, delete account
- See `.cursor/rules/no-asyncstorage-for-secrets.mdc`

### Frontend ↔ backend contract

- Backend uses `#[serde(rename_all = "camelCase")]` — Rust `snake_case` fields become `camelCase` in JSON
- When sending data TO the backend, map your variable names to the backend's expected `camelCase` (e.g. `type` → `offerType`, `type` → `paymentType`)
- When receiving data FROM the backend, map `camelCase` back to your local names
- Always check the backend Rust struct definition (`/Users/jpvanzyl/Workspaces/Qictrader/qictrader-backend-rs/src/models/`) before writing API calls
- Backend list endpoints may return flat arrays; don't assume `{ items: [...], total }` wrappers

### Pure logic, side effects at the edges

- Core domain logic (validators, formatters, fee calculations) should be pure and testable
- Push side effects (API calls, native modules, navigation) to the boundaries
- Immutable by default

### Testing

- Tests make real assertions about specific values — not just "did not crash"
- NEVER `not.toBe(404)` — assert actual expected status: `toBe(200)`, `toBe(201)`
- NEVER `toBeDefined()` alone — assert the expected value: `toBe('USDT')`
- NEVER `if (!res._ok) { console.warn(); return }` — hides real problems behind green checks
- Use `test.skip('reason')` if setup might fail — a silent pass is worse than no test

### Commit discipline

- One logical change per commit
- Commit messages: `MOBILE-XXX-NNN: short summary` matching the Trello ticket
- Build passes before pushing — no "fix build" follow-up chains
- Never include secrets in commits

## NO SUPPRESSION — ZERO TOLERANCE

This is a custodial fintech app. Silent failures mean money moves but the user can't see it, or worse, a user thinks their transfer succeeded when it didn't.

### `let _ = fallible_call()` is FORBIDDEN

Every `let _` on a Promise/Result is a bug. Fix it:

```ts
// WRONG — silent failure
fetch('/api/v1/wallet/withdraw', { ... });

// WRONG — silent failure
const _ = submitWithdraw(payload);

// RIGHT — handle both outcomes
try {
  const result = await submitWithdraw(payload);
  // success path
} catch (e) {
  Sentry.captureException(e);
  showToast('Withdrawal failed — please try again');
}
```

### Empty catches are FORBIDDEN

```ts
// WRONG
try { await foo() } catch {}

// WRONG  
foo().catch(() => {});

// RIGHT
try { await foo() } catch (e) {
  Sentry.captureException(e);
  // and: surface to UI, log structured event, or rethrow
}
```

### Other forbidden suppressions

- No `// @ts-expect-error` without a comment explaining why and a tracking ticket
- No `// @ts-ignore` ever — delete the line or fix the type
- No `as any` — narrow the type or change the type definition
- No `_ =>` catch-all on domain enums in `switch` statements
- If the linter warns, fix the cause — don't disable the rule

### Pre-push scan (mandatory)

Before every push, run:

```bash
grep -rn 'let _ = ' src/ | grep -v -E '(\.test\.|__tests__)' && \
  echo "FOUND let _ = in production code — reject push" && exit 1

grep -rn 'as any' src/ | grep -v -E '(\.test\.|__tests__)' && \
  echo "FOUND as any in production code — reject push" && exit 1

grep -rn 'catch\s*({.*}|()\s*=>\s*{}' src/ && \
  echo "FOUND empty catch — reject push" && exit 1
```

Wire as a pre-commit hook in `.husky/pre-commit`.

## Sync from web

When the web frontend changes shared logic (types, validators, constants, design tokens, formatters):

1. `cd /Users/jpvanzyl/Workspaces/qictrader-mobile`
2. `./scripts/sync-from-web.sh`
3. Review `git diff src/` to see what changed
4. Fix any TypeScript errors caused by the sync (these are the contract changes mobile needs to handle)
5. Run affected tests
6. Commit the synced files with message `chore: sync from web @ {web-commit-sha}`

See `.cursor/rules/sync-from-web.mdc` for details.

## Trello workflow

Trello is the project tracker. Read `.qictrader-context/trello-board.md` for board IDs, list IDs, API examples.

Mobile tickets:
- **Board:** Project One (`R7WQRSJ9`)
- **Label filter:** `mobile`
- **Ticket prefix:** `MOBILE-{PHASE}-{NUM}`
- All starting tickets are in **Acknowledged** column, assigned to JP

Workflow:
```
Acknowledged → Development (In Progress) → Testing (In Progress) →
Testing (Ready) → Deployment → Done
              ↓
        QA (Failed) → back to Development
```

On every ticket:
1. Read the card description (use Trello API or `.qictrader-context/tickets-export.json`)
2. Read relevant web implementation for parity reference
3. Read relevant backend struct/handler for contract
4. Move card to "Development (In Progress)" when starting
5. After commit + push + (where applicable) staging build, move to "Testing (In Progress)"

## Trello credentials

Same `~/.qictrader-secrets/trello.env` as the main monorepo. Source it before any Trello call:

```bash
source ~/.qictrader-secrets/trello.env
```

If you don't have it, ask JP. Never paste the actual values into chat, commits, or this file.

## Deploy

- **iOS dev build:** `eas build --profile development --platform ios`
- **Android dev build:** `eas build --profile development --platform android`
- **Internal beta:** `eas build --profile preview --platform all`
- **Production:** Tag `mobile-v{semver}`, CI runs `eas build --profile production`, then `eas submit -p ios` / `eas submit -p android`

See `.cursor/rules/eas-deploy-only.mdc`.

## Read more

- `.cursor/rules/engineering-principles.mdc` — full engineering rules
- `.cursor/rules/credential-hygiene.mdc` — secrets handling
- `.cursor/rules/feature-parity-with-web.mdc` — parity enforcement
- `.cursor/rules/sync-from-web.mdc` — file-level sync mechanism
- `.cursor/rules/no-asyncstorage-for-secrets.mdc` — token storage rules
- `.cursor/rules/web-codebase-reference.mdc` — how to use the web repo as reference
- `.cursor/rules/react-native-stack.mdc` — Expo + RN conventions
- `.cursor/rules/eas-deploy-only.mdc` — build + release rules
- `.cursor/rules/trello-mobile-tickets.mdc` — mobile board specifics
- `.cursor/rules/tests-before-store-submit.mdc` — testing gate before store submission
- `.cursor/rules/use-workers-for-split-tasks.mdc` — when to spawn subagents
- `.qictrader-context/security-carryforward.md` — security patterns from the main app
- `.qictrader-context/api-endpoints.md` — backend endpoint summary
- `.qictrader-context/tickets-export.json` — all 66 ticket specs

When in doubt about a security or money-handling decision, read `.qictrader-context/security-carryforward.md`. It distills the lessons from the QicTrader pen-test and 6 months of production hardening.
