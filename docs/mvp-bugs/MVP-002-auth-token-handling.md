# MVP-002: Auth token / login flow issues

**Severity:** HIGH — blocks user login from production frontend  
**Status:** Linked to MVP-001; resolved when frontend uses correct backend (code fallback in env.ts)  
**Date:** 2026-03-08

## Problem

Users cannot log in or sign up from the production frontend at `qictrader.com`. The login page sends credentials to the old Node.js backend which either:
- Returns incorrect 401 errors
- Returns 500 errors interpreted as auth failures
- Returns stale/incompatible token formats

## Symptoms Observed

1. **"Invalid email or password"** — login fails on production frontend even with correct credentials
2. **"Username is already taken"** — signup blocked (username check hits wrong backend)
3. **Auth token cleared repeatedly** — console shows token being cleared in a loop
4. **401 "Unauthorized - No token provided"** — authenticated API calls fail after login

### Console Evidence

```
{"level":"info","message":"Auth token cleared"}
{"level":"info","message":"Auth token cleared"}
{"level":"error","message":"[users-api] Get profile failed","data":{"error":"Unauthorized - No token provided","status":401,"code":"NO_TOKEN"}}
```

## Root Cause

Same as MVP-001: the frontend sends auth requests to the old Node.js backend (`qic-trader-backend-f44367dc781f`) instead of the Rust backend (`qictrader-backend-rs-13eab0516d9a`).

Tokens issued by one backend are invalid on the other, so even if login "succeeds" on the old backend, subsequent API calls to the Rust backend (or vice versa) fail with 401.

## Fix

Resolving MVP-001 (correct `NEXT_PUBLIC_API_URL` on Vercel) will fix this issue.

## Workaround

Log in via curl directly to the Rust backend:
```bash
curl -s -X POST https://qictrader-backend-rs-13eab0516d9a.herokuapp.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"..."}' | jq .
```

Then use the returned token for authenticated requests.
