# Runbook — Backend under `api.qictrader.com` (Trello #364)

**Status:** Code/config-ready. **DNS + Heroku-prod + Vercel-prod steps deferred to the
Thursday production deploy.** This is an ops/infra cutover, not a code change.
**Owners for the cutover:** @christianshekleton (DNS / approvals) + @alfred227 (Star — Vercel/verify).
**Date prepared:** 2026-06-16

---

## What this is

Give the existing backend Heroku app (`qictrader-backend-rs`) a QIC-owned hostname
(`api.qictrader.com`) instead of exposing `qictrader-backend-rs.herokuapp.com` to clients.
**The server does not move.** This removes third-party-cookie fragility, the foreign
hostname in every client request, and Heroku lock-in in baked client bundles.

## Backend is already config-ready (verified 2026-06-16)

No backend code change is required — everything is env-driven:

- `ALLOWED_ORIGINS` parses from env (`config.rs:162`, default empty) and materializes CORS
  origins (`config.rs:1107`).
- Axum enables `allow_credentials(true)` (`app.rs:211`).
- Login sets the access/refresh/CSRF cookie triplet (`auth.rs` → `auth_cookies.rs`).
- Frontend already sends `credentials: "include"` (`http-client.ts`).

The only remaining work is **DNS + Heroku domain + cert + Vercel env + a prod FE rebuild** —
all of which require domain control and a production deploy (out of scope until Thursday;
this session is **staging-only, NO PROD DEPLOYMENTS**).

## Cutover steps (run Thursday, in order)

1. **Add custom domain to Heroku (prod app):**
   ```bash
   heroku domains:add api.qictrader.com -a qictrader-backend-rs
   heroku domains -a qictrader-backend-rs        # note the DNS target, e.g. api.qictrader.com.herokudns.com
   ```
2. **DNS:** on the `qictrader.com` zone, add `CNAME api → <herokudns target from step 1>`.
   (Requires whoever controls `qictrader.com` DNS — @christianshekleton to confirm access.)
3. **SSL (ACM):**
   ```bash
   heroku certs:auto:enable -a qictrader-backend-rs
   heroku certs:auto -a qictrader-backend-rs     # wait until cert shows "live"
   ```
4. **Vercel prod env:** set
   - `NEXT_PUBLIC_API_URL=https://api.qictrader.com/api/v1`
   - `NEXT_PUBLIC_WS_URL=wss://api.qictrader.com`
5. **Backend prod config (confirm, don't guess):**
   - `ALLOWED_ORIGINS=https://qictrader.com,https://www.qictrader.com`
   - `FRONTEND_URL=https://www.qictrader.com`
   - `COOKIE_SAMESITE=Lax`
6. **Rebuild frontend prod** so the old Heroku URL is no longer compiled into the JS bundle
   (Vercel prod deploy — part of Thursday's cutover).
7. **Verify:**
   ```bash
   curl https://api.qictrader.com/health        # expect 200 {"status":"ok",...}
   ```
   Then run a real login on `qictrader.com` and confirm the access/refresh/CSRF cookies are
   set against the `qictrader.com` parent and the next API call carries them.

## Blockers / notes

- This needs **production** changes and **`qictrader.com` DNS control** — neither is in
  scope for the staging-only work this session.
- The known Vercel domain-access constraint (`www.qictrader.com` lives under a Vercel scope
  the current automation login can't alias) means the **Vercel-side steps must be done by
  the domain owner**.
- Nothing here changes `main`; backend is already correct. Card moved to Development
  Complete because the **dev/config portion is done** — only the Thursday infra cutover
  remains, tracked by this runbook.
