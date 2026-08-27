# QicTrader Mobile

React Native (Expo) mobile app for iOS + Android. Consumes the existing `qictrader-backend-rs` HTTP API. Feature parity with `frontend/` is **non-negotiable** — every feature on web must behave identically on mobile.

## Quick start

```bash
git clone git@github.com:Qictrader2/qictrader-mobile.git
cd qictrader-mobile
bun install
bun expo start
```

iOS simulator: `bun expo run:ios`
Android emulator: `bun expo run:android`

Full setup steps live in **MOBILE-INIT-001** on Trello (see `.qictrader-context/tickets-export.json`).

## Repository purpose

This is a **separate repo from the main `Qictrader` monorepo** for two reasons:
1. Mobile tooling (Xcode, Android Studio, EAS, Maestro) is heavy and shouldn't be installed by web/backend devs.
2. Mobile release cadence (App Store + Play Store reviews) is fundamentally different from web's hot-deploy flow.

The web frontend and Rust backend live elsewhere:
- **Web/backend monorepo:** `/Users/jpvanzyl/Workspaces/Qictrader` (read-only reference; do NOT modify from this repo's Cursor window)
- **GitHub:** main repo is private under the `Qictrader2` org

## How we avoid duplication

Some things are shared between web and mobile via a sync script (`scripts/sync-from-web.sh`), not via a monorepo or npm packages:

| Shared via file copy | What | Source of truth |
|---|---|---|
| API types | TypeScript types matching backend Rust structs | Generated from backend OpenAPI; committed to both repos |
| Design tokens | Colors, spacing, radii, shadows, typography | `design-tokens.json` in web repo |
| Constants / enums | Currency codes, network names, payment method types, statuses | Web repo `frontend/src/constants/` |
| Validators | Zod schemas matching backend validation | Web repo `frontend/src/schemas/` |
| Formatters | Currency, date, address-truncation, fee-calc helpers | Web repo `frontend/src/lib/format.ts` |

**Run `bun run sync` (or `./scripts/sync-from-web.sh`) after any change to shared web logic.** TypeScript will fail your build if API contracts drift.

## What is NOT shared (necessarily different)

- React Native components (RN primitives ≠ DOM)
- Navigation (React Navigation ≠ Next App Router)
- Forms (same `react-hook-form`, different JSX rendering)
- Platform integrations (biometric, push, camera, deep links — mobile-only)
- Web-only features (SEO, RSC streaming, Wagmi wallet connect)

## The 66-ticket roadmap

All 66 mobile tickets live on the **Project One** Trello board, in the **Acknowledged** column, tagged with the `mobile` label, assigned to JP. Full ticket bodies are also cached in `.qictrader-context/tickets-export.json` for offline reference.

Phases:

| Phase | Tickets | Block |
|---|---|---|
| INIT | 5 | Project scaffolding, design system, navigation, EAS CI/CD, branding |
| AUTH | 6 | Login, signup, 2FA, biometric, OAuth (Apple + Google) |
| KYC | 4 | Status, Didit, SumSub, document capture |
| WALLET | 8 | Overview, deposit, withdraw, history, transfer, fiat switcher, prices, confirm sheet |
| MARKET | 6 | List, filters, offer detail, create offer, my offers, resell |
| TRADE | 8 | List, detail, chat, POP upload, mark paid + release, dispute, history, cancel |
| NOTIF | 3 | Push registration, in-app center, deep links + badge |
| PROFILE | 5 | Profile, edit, security, payment methods, preferences |
| AFF | 2 | Dashboard + share, commission history |
| SUPPORT | 2 | Help center, submit ticket |
| SEC | 4 | TLS pinning, jailbreak/screen-cap, SecureStore audit, session timeout |
| PERF | 3 | FlashList + images, offline cache, skeletons + optimistic |
| TEST | 3 | Jest+RNTL, Maestro E2E, visual regression |
| LAUNCH | 7 | App Store, Play Console, listings, TestFlight, Play beta, Sentry+GA4, runbook |

## Project conventions

Read these before writing any code:

- **`AGENTS.md`** + **`CLAUDE.md`** — agent rules (same content, two filenames for tool discovery)
- **`.cursor/rules/`** — workspace rules, all `alwaysApply: true` except as noted
- **`.qictrader-context/`** — snapshots from web/backend (security carryforward, state machines, API endpoints, Trello info)

Engineering principles that bite hardest in practice:

- **Types first.** Define interfaces before logic. Enums over strings.
- **Make invalid states unrepresentable.** No catch-all `default:` on domain unions.
- **No swallowed errors.** Every `catch` does something explicit. No `let _ = …` on Promise rejections.
- **Auth checks specific resources, not just authentication.** Every screen that loads user data verifies ownership.
- **SecureStore for tokens, never AsyncStorage.** See `.cursor/rules/no-asyncstorage-for-secrets.mdc`.
- **Feature parity > novelty.** If web behaves a certain way, mobile must match. Diverging requires a Trello card and explicit JP approval.

## Stack

- **Expo SDK 52+** managed workflow
- **React Native 0.76+** with New Architecture
- **TypeScript** strict mode (`strict: true`, `noUncheckedIndexedAccess: true`)
- **NativeWind v4** (Tailwind for RN) — mirrors web's Tailwind setup
- **React Navigation v7** (native stack + bottom tabs + drawer)
- **Redux Toolkit + Zustand + React Query (TanStack v5)** — same patterns as web
- **expo-secure-store** for tokens (NOT AsyncStorage)
- **expo-local-authentication** for biometric
- **expo-notifications** for FCM + APNs
- **Socket.IO client** for realtime trade chat
- **bun** as package manager
- **Maestro** for E2E tests
- **EAS Build** for cloud iOS + Android builds
- **EAS Submit** for store uploads
- **Sentry** for crash reporting
- **GA4 (Firebase Analytics)** for product analytics

## Backend endpoints

API base URLs:

| Env | URL |
|---|---|
| Production | `https://qictrader-backend-rs-13eab0516d9a.herokuapp.com` (will move to `api.qictrader.com`) |
| Staging | (see `.qictrader-context/api-endpoints.md`) |
| Local dev | `http://localhost:8080` |

The backend uses `#[serde(rename_all = "camelCase")]` — Rust `snake_case` fields become `camelCase` in JSON. Always verify field names against the Rust source.

## Deploy / release

- **Dev builds:** `eas build --profile development --platform all`
- **Internal beta:** `eas build --profile preview --platform all` → distributes via TestFlight + Play internal track
- **Production:** tag `mobile-v{semver}` → CI triggers `eas build --profile production` → manual store submission via `eas submit`

Never deploy to production without:
1. Passing Maestro E2E suite locally
2. TestFlight internal review pass
3. Explicit JP confirmation

Full release runbook: **MOBILE-LAUNCH-007** ticket.

## Trello + GitHub coordination

- **Board:** Project One (<https://trello.com/b/R7WQRSJ9/project-one>)
- **Label filter:** `mobile`
- **Ticket prefix:** `MOBILE-{PHASE}-{NUM}`
- **GitHub:** <https://github.com/Qictrader2/qictrader-mobile>

Card workflow: Acknowledged → Development (In Progress) → Testing (In Progress) → Testing (Ready) → Deployment → Done.

Commit messages follow: `MOBILE-XXX-NNN: short summary` (no emoji prefixes).

## License

Proprietary. © Qic Trade Systems Limited.
