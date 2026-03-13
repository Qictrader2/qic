# MVP-001: Vercel frontend hitting old Node.js backend

**Severity:** CRITICAL — blocks all new signups + affiliate dashboard  
**Status:** Env var fixed; code fallback added so wrong URL is rejected (no deploy yet)  
**Date:** 2026-03-08

## Problem

The Vercel production frontend (`qictrader.com`) has `NEXT_PUBLIC_API_URL` baked in pointing to the **old, deprecated Node.js backend** (`https://qic-trader-backend-f44367dc781f.herokuapp.com/api/v1`) instead of the current Rust backend (`https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1`).

### Impact

- **Username check** → old backend returns 500 → frontend shows "Username is already taken"
- **Signup** → old backend returns 409 → blocks all new account creation
- **Affiliate dashboard** → API calls fail → page is blank, no referral link shown
- **All API calls** → may hit wrong backend silently

## Console Evidence

```
HttpClient initialized {"baseUrl":"https://qic-trader-backend-f44367dc781f.herokuapp.com/api/v1"}
GET https://qic-trader-backend-f44367dc781f.herokuapp.com/api/v1/users/check-username/jpsarfat 500
```

Direct curl to the **correct** Rust backend confirms username is available:
```
$ curl https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1/users/check-username/jpsarfat
{"available":true,"message":null}
```

## Root Cause

The `qictrader-frontend` Vercel project had a stale `NEXT_PUBLIC_API_URL` environment variable pointing to the old Node.js backend. This overrode the correct value in `.env.production` and the hardcoded fallback in `Frontend/src/lib/env.ts`.

The Vercel CLI `env ls` inconsistently showed this variable (sometimes "No Environment Variables found" even though it existed), making diagnosis difficult.

## Fix Applied

1. **Vercel (manual):** `vercel env rm NEXT_PUBLIC_API_URL production --yes` then re-add correct URL; trigger deploy.
2. **Code (this repo):** `Frontend/src/lib/env.ts` — if `NEXT_PUBLIC_API_URL` or `NEXT_PUBLIC_WS_URL` equals or contains the deprecated Node backend host, `getApiUrl()` / `getWsUrl()` now return the Rust backend URL so a stale env cannot break signup/login.

## Verification

After deploy goes live, confirm browser console shows:
```
HttpClient initialized {"baseUrl":"https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1"}
```

## Workaround

Create accounts directly via curl to the Rust backend:
```bash
curl -s -X POST https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"...","username":"...","referralCode":"..."}' | jq .
```

## Files

- `Frontend/src/lib/env.ts` (lines 53-66) — `getApiUrl()` reads `NEXT_PUBLIC_API_URL`
- `Frontend/.env.production` (line 27) — has correct URL but overridden by Vercel env var
