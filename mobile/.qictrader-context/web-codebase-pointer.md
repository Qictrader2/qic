# Web codebase — pointer + reading map

The web frontend lives at `/Users/jpvanzyl/Workspaces/Qictrader/frontend/`. Treat it as **read-only** from this mobile repo.

## Stack (for context)

- Next.js 16 (App Router)
- React 19
- TypeScript strict
- Tailwind CSS
- Shadcn UI
- Redux Toolkit + Zustand + React Query v5
- Socket.IO client
- Wagmi / Viem (web-only crypto wallet connect)
- bun package manager

## Map: when to read what

### For mobile auth tickets

| Mobile concern | Read |
|---|---|
| Login flow | `frontend/src/app/(auth)/login/page.tsx`, `frontend/src/store/auth-store.ts` |
| Signup flow | `frontend/src/app/(auth)/signup/page.tsx` |
| 2FA verify | `frontend/src/app/(auth)/verify-2fa/page.tsx` |
| Password reset | `frontend/src/app/(auth)/forgot-password/`, `reset-password/` |
| OAuth (Apple/Google) | `frontend/src/lib/auth/oauth.ts` |
| Auth guards | `frontend/src/components/auth/AuthGuard.tsx` |
| API client interceptors | `frontend/src/lib/api-client.ts` |
| Token storage (web uses cookies after SEC-SPRINT) | `frontend/src/lib/auth/cookies.ts` |

### For mobile KYC tickets

| Mobile concern | Read |
|---|---|
| KYC status page | `frontend/src/app/(dashboard)/kyc/page.tsx` |
| Didit flow | `frontend/src/lib/kyc/didit.ts` |
| SumSub flow | `frontend/src/lib/kyc/sumsub.ts` |
| Tier gating | `frontend/src/hooks/use-kyc-tier.ts` |

### For mobile wallet tickets

| Mobile concern | Read |
|---|---|
| Wallet overview | `frontend/src/app/(dashboard)/wallet/page.tsx` |
| Deposit | `frontend/src/app/(dashboard)/wallet/deposit/`, `frontend/src/services/wallet-api.ts` |
| Withdraw | `frontend/src/app/(dashboard)/wallet/withdraw/` |
| Transaction history | `frontend/src/app/(dashboard)/wallet/transactions/` |
| Fee calculation | `frontend/src/lib/fees.ts` |
| Address validation | `frontend/src/lib/wallet/validate.ts` |

### For mobile marketplace tickets

| Mobile concern | Read |
|---|---|
| Marketplace list | `frontend/src/app/(dashboard)/marketplace/page.tsx` |
| Filters | `frontend/src/components/marketplace/Filters.tsx` |
| Offer detail | `frontend/src/app/(dashboard)/marketplace/[offerId]/` |
| Create offer | `frontend/src/app/(dashboard)/marketplace/create/` |
| My offers | `frontend/src/app/(dashboard)/profile/my-offers/` |
| Resell flow | `frontend/src/app/(dashboard)/marketplace/[offerId]/resell/` |

### For mobile trade tickets

| Mobile concern | Read |
|---|---|
| Trades list | `frontend/src/app/(dashboard)/trades/page.tsx` |
| Trade detail | `frontend/src/app/(dashboard)/trades/[tradeId]/` |
| Trade chat (Socket.IO) | `frontend/src/lib/socket.ts`, `frontend/src/components/trade/Chat.tsx` |
| State machine helpers | `frontend/src/lib/state-machines/trade.ts` |
| Mark paid / release | `frontend/src/services/trade-api.ts` |
| Dispute | `frontend/src/components/trade/DisputeModal.tsx` |

### For mobile profile tickets

| Mobile concern | Read |
|---|---|
| Profile page | `frontend/src/app/(dashboard)/profile/page.tsx` |
| Security settings | `frontend/src/app/(dashboard)/profile/security/` |
| Payment methods | `frontend/src/app/(dashboard)/profile/payment-methods/` |
| Sessions | `frontend/src/app/(dashboard)/profile/sessions/` |

### For mobile design

| Mobile concern | Read |
|---|---|
| Design tokens (the SOT) | `Qictrader/DESIGN.md` (synced to this repo) |
| Tailwind config | `frontend/tailwind.config.ts` |
| CSS variables (raw values) | `frontend/src/app/globals.css` |
| Theme implementation | `frontend/src/lib/theme.ts` |
| Card themes (Ocean/Sunset/etc.) | `frontend/src/lib/themes/cards.ts` |

### For mobile analytics

| Mobile concern | Read |
|---|---|
| GA4 setup | `frontend/src/lib/analytics/ga4.ts` |
| Event names | `frontend/src/lib/analytics/events.ts` |
| Consent Mode | `frontend/src/lib/analytics/consent.ts` |

## How to read efficiently

Don't read entire files — open them, find the relevant function or component, read 50-200 lines max. Use `rg` to navigate:

```bash
# Find a hook
rg "export function use" /Users/jpvanzyl/Workspaces/Qictrader/frontend/src/hooks/

# Find an API call
rg "post.*api/v1/wallet/withdraw" /Users/jpvanzyl/Workspaces/Qictrader/frontend/src/

# Find a type
rg "export (interface|type) Trade" /Users/jpvanzyl/Workspaces/Qictrader/frontend/src/types/
```

## Big files that bite

These exist and contain a lot — don't try to read top-to-bottom:

- `frontend/src/store/auth-store.ts` (500+ lines, dense)
- `frontend/src/lib/api-client.ts` (300+ lines, interceptor logic)
- `frontend/src/lib/socket.ts` (Socket.IO setup with reconnect logic)
- `frontend/src/services/*-api.ts` (one per backend resource)

Open with `Read` tool using line offsets.

## What NOT to read

| Path | Why skip |
|---|---|
| `frontend/node_modules/` | Massive, irrelevant |
| `frontend/.next/` | Build cache |
| `frontend/.playwright-mcp/` | Playwright session captures |
| `frontend/coverage/` | Test output |
| `frontend/e2e/` | Web E2E tests; mobile has its own at `.maestro/` |
| `frontend/public/` | Static assets; mobile has its own |
