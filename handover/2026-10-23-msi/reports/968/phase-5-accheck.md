# 968 Phase 5 ACCHECK report

Card 968 (Trello xcgVXhJY), "Customer redesign 906-0c: phone bottom bar, one balance and clear overlays". This was an audit only, run by a fresh overseer, 22 Oct 2026, 09:20 to 10:20 Dubai. I audited against the current card text, which was aligned with the v5.2 boards on 22 Oct.

## 1. What I audited

- **Deployed build.** Review app https://qictrader-fe-pr-303.herokuapp.com on the staging backend. When I started it was release v6 (fe023ad2, which is 968). At 09:33 it moved to release v7, `Deploy 366dca2f`, when 969 merged into redesign/906 (the coordinator told me about this mid-run).
- **Evidence build.** Every capture I kept was made on v7, starting 09:39. The captures from the run that straddled the v6 to v7 switch are parked in `superseded-v6-mixed/` and I did not use them.
- **969 does not touch 968's files.** Between fe023ad2 and 366dca2f, 969 changes no file under navigation, routes, globals.css, layout.tsx, mainHeader, ConsentBanner, SupportChatWidget or the bottom-bar hooks.
- **Pins.** Frontend origin/redesign/906 is 366dca2f1097d2e2184cf09ba88038f5af766363. 968 is fe023ad2 and its base is f44c50dc. The backend is the staging backend; 968 is frontend only.
- **Accounts.** All sign-ins went through the qic-qa-signin agent: buyer-906 (user), seller-906 (super_admin) and third-906.
- **Temporary role change.** The third user (starbird) was set to `moderator` for the role check. It is now restored to `user`, read back from the database and through the API. Ledger: `~/.config/qic/qa-sessions/change-ledger/968-20261022T052151Z.json`, marked restored at 20261022T061533Z. The sign-in agent re-signed third-906 so that no stale moderator token is left.
- **Browser.** Puppeteer with a temporary profile created per run under my scratchpad and deleted afterwards. Sessions were planted only through plantSession. No money moved and nothing was submitted.
- **Evidence folder.** `/Users/starbird/.config/qic/qa-artifacts/968/`, containing:
  - the plan: htt-plan.md
  - the card text: card-968.md
  - capture JSON: capture-out-hard-dock-deep.json, capture-large-toast-dialog.json, capture-overlays-roles-history.json, capture-keyboard.json, capture-followup.json, capture-overflow-1440.json
  - about 115 screenshots at 320, 360, 390, 768 and 1440
  - the Jev requests and responses: jev-ac*-request.json, jev-ac*-response.json, jev-ac*-retry-*.json
  - the collage: accheck-968.png

## 2. Verdicts

| AC | Worker | My check | Jev (jev-1.14.0) | Final | Trello |
| --- | --- | --- | --- | --- | --- |
| 1 Dock destinations and the active item on every customer page | MET | MET | met 0.45, retry 0.53 (admissible 0.99, partly wired 0.08) | HELD: Jev below the gate | unticked |
| 2 Bar absent where it must be, covers nothing | MET | MET with limits | met 0.64, retry 0.64 (0.98 / 0.19) | HELD: Jev below the gate | unticked |
| 3 No header balance or Support; staff entries by role | MET | MET | met 0.83, retry 0.89 (0.98 / 0.18) | MET | ticked, read back |
| 4 Back keeps trade IDs and queries | NOT MET, from a capture fault | MET after re-test | met 0.91 (0.99 / 0.14) | MET | ticked, read back |
| 5 Staff, public and excluded pages as before | MET | MET with limits | met 0.45, retry 0.43 (0.98 / 0.16) | HELD: Jev below the gate | unticked |

Gate: verdict met, confidence of at least 0.85, admissible evidence of at least 0.7, partly wired at most 0.5.

On the retry I added the decisive evidence:
- AC1: a 24-row table comparing the card's routes with the deployed active item.
- AC2: the CSS and the toast viewport lines.
- AC3: the CustomerHeader body.
- AC5: the diff stat of the staff, public, auth, layout and landing trees.

Per the brief, I stopped on AC1, AC2 and AC5 instead of deciding them. Under /qic-ship they go to the Jev-fail arbiter (opus55-max).

## 3. Evidence per criterion

- **AC1.**
  - Code: `BOTTOM_BAR_ROUTES` and `getBottomBarActive` (routes.ts:575-640) list Wallet, Market, Trades, Earnings and Account with hrefs /wallet, /marketplace, /trade-history, /earnings and /profile.
  - Tests: an exhaustive test runs over every page.tsx (98 routes).
  - Deployed: at 390 the five items show in order on the 13 dock pages, with exactly one aria-current. At 768 and 1440 the island marks the same section on every dock, task and level-2 page. Of the card's 24 listed routes, none mismatches; for example /dashboard/affiliate lights Earnings and /offer/resell lights Earnings.
  - Each dock tap opens its route.
  - Not in the card: /prices* lights Market.
- **AC2.**
  - No dock in any of these cases:
    - signed out
    - at 768 and 1440 on all 28 customer routes
    - on 15 task and level-2 pages at 390
    - on /admin, /moderator, /escrow, /whatsapp, /verify-email, /verify-email-pending, /, /help, /faqs, /terms, /privacy, /contact-us, /about and /how-escrow-works
  - Labels are not clipped at 320 or 360.
  - Toasts (969) end 12 px above the dock at 390, 360 and 320. The consent banner also ends 12 px above it at all three widths.
  - With a text field focused (coarse pointer and focus emulation), the dock goes to state keyboard: inert, aria-hidden, opacity 0, dropped 16 px. It comes back on blur.
  - With a dialog (wallet info) or the chat sheet open, the dock goes to state covered and is inert. It is visible again after Escape.
  - There is no support FAB on any customer page at any width.
  - Limits:
    - not tried on a real iOS or Android keyboard
    - the safe-area inset was 0 in the capture, so the safe-area term is proven by the CSS only
- **AC3.**
  - On every customer route at 768 and 1440: 0 `Portfolio balance` controls and no header Support control. The balance shows on the /wallet card (81.99 USDT for the QA buyer).
  - Staff entries:

    | Account | Account hub at 390 | Jump palette at 1440 | Desktop account menu on /help at 1440 |
    | --- | --- | --- | --- |
    | buyer-906 (user) | none | none | none |
    | third-906 (moderator) | Moderator Dashboard only | Moderator Dashboard only | Moderator Dashboard only |
    | seller-906 (super_admin) | Moderator Dashboard and Admin Panel | Moderator Dashboard and Admin Panel | Moderator Dashboard and Admin Panel |

  - Public pages keep today's header, with its balance and Support link, by design.
- **AC4.**
  - These cases return the start URL exactly after Back:
    - 768: /trade/978a39b8-... through island Wallet
    - 390: /trade-history?status=active through dock Wallet
    - 768: /wallet/manage/usdt?action=withdraw through island Market (capture-followup.json)
    - 390: /wallet/manage/usdt?action=withdraw through its back link
  - The first withdraw attempt failed only because the page opens a Withdraw dialog, which makes the island inert. The re-test closes it with Escape first.
  - Hard-loaded /notifications, /settings, /trade/<id> and the withdraw page: the back controls stay on the review app.
  - /earnings?ref=qa goes to /dashboard/affiliate?ref=qa. Signed out, /earnings goes to /login?redirect=%2Fearnings.
- **AC5.**
  - Diff f44c50dc..fe023ad2 under src/app, middleware, proxy, auth, admin, moderator, layout and landing changes only:
    - the new /earnings page
    - the Account hub on /profile
    - the globals.css block (customer or data-bottom-bar selectors only)
    - two mounts in the root layout
    - a robots /earnings entry

    src/app/api is unchanged.
  - Every staff, public and excluded page loads (200, /terms 304) with today's header and FAB and no dock. That includes the seller (super_admin) on /admin, /moderator, /admin/users and /moderator/disputes.
  - A signed-in /login goes to /dashboard and /auth/callback/google goes to /dashboard; both were redirects before this card. /account/kyc goes to /settings?tab=verification.
  - Route handlers give the same status as staging main:

    | Status | Handlers |
    | --- | --- |
    | 200 | runtime-config, social/youtube, v1 proxy, runtime-env.js, robots, sitemap, manifest |
    | 405 | staging-access, waitlist/virtual-card, bug-report/discord |

  - Not run: real form submits (read-only audit) and the 2FA step.

## 4. QA watch items from the review

1. **Hard load of /wallet at 390, signed in.**
   - The body bottom padding is 88 px from the first frame (13 ms, before the dock mounts at 263 ms). The theme script sets data-bottom-bar early, so the padding never shifts.
   - One layout shift of 0.328 at about 1.2 s comes from the footer moving when the wallet content loads. It is not the dock padding.
2. **Hydration.** No hydration error in the console on signed-in hard loads. The only errors are the expected harness lines: the blocked refresh call and the "Cannot connect" messages that follow it.
3. **Back from a level-2 page stays inside QicTrader.** Yes, including hard loads with no referrer.
4. **/earnings redirects to /dashboard/affiliate.** Yes, keeping the query.

## 5. Seen, outside the checklist (not judged against 968)

1. Page content scrolls sideways at 360 and 320 on /wallet (381 px) and at 320 on /profile (357 px). The builder recorded this for 320 only, and it also happens at 360 on /wallet. The cause is page components that 968 did not change (cards 958 and 956). The dock itself stays inside the viewport.
2. The support chat opens from the Account hub and the Jump palette. However, it is still the old dark ChatBot box: 374 x 500 on phones and 380 x 500 at the bottom right, 24 px in, at 1440. The card's Expected behaviour asks for a glass sheet on phones and a 400 x 600 glass panel. No checklist item or Mandatory QA line covers it, so the owner should decide whether 969 or a later card owns it.
3. Signed-in public pages at 1440 scroll sideways (1634 px on the review app). Staging main also does (1466 px). It comes from today's default header nav, which 968 leaves unchanged.
4. Mandatory QA AC2 says "long names and amounts do not overflow". I read that as the bar, whose labels fit at 320 and 360. Point 1 above is page content.

## 6. Trello writes

- Ticked AC3 (6ad887c985aedc66abb7aa31) and AC4 (6ad887cab9b582bddf5e79c8). A fresh checklist GET shows items 3 and 4 complete and items 1, 2 and 5 incomplete.
- One summary comment, id 6ad9ab0503e7d5fa6b4d3dd3, read back.
- I did not move the card (its idList is unchanged) and attached nothing.

## 7. Collage

`/Users/starbird/.config/qic/qa-artifacts/968/accheck-968.png`. It has one panel per criterion plus a summary card, and I read it back. It shows no secrets; the QA account names and balances in it are staging test data.
