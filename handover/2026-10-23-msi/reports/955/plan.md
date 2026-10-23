# Card 955 (906-E) implementation plan: sign in, sign up and password reset

Planning only. Nothing was edited, committed or written to Trello.

Sources read on 22 Oct 2026:
- Trello 955 (3YMhHROU): description, the 5 checklist items, no comments.
- Boards: `~/Downloads/906-design-pack-v5.2/` (CHANGES.md and the nine changed E PDFs) and the editable sources in `.scratch/customer-app-redesign/research/design-pack/v5.2-src/E-auth/` (design-notes.md including "Change log: 21 Oct v5.2", register.json, tests.md, all 23 `project/*.dc.html` boards as text, auth.css).
- Code: read-only checkout `.scratch/906-redesign/base-ro/frontend` at 366dca2f (redesign/906 with 970, 968 and 969 merged), root DESIGN.md beside it, `.scratch/906-redesign/HANDOVER.md`, and the backend submodule at origin/main f043a14 (read only).
- Dependency cards 64, 755, 904, 916, 917, 935, 964 and the sibling cards 956 to 959 read from Trello.

## (a) Boards and routes

| Board | Route | What it fixes | v5.2 changed |
|---|---|---|---|
| E0 Main | all four | Layout table (390, 720, 768, 1440), state coverage matrix, excluded entry points, what changes from today | yes |
| E1a, E1e, E1b, E1c | /login | Default at 390, 720 (1440 at 200 %), 768, 1440 | no |
| E1d login-states | /login | A field errors, B wrong email or password (and 5xx), C loading, D rate limited, E session ended, F deactivated or banned, G email not verified (entry point), H two-factor (entry point), I show or hide, J signed in on /wallet with the toast, K already signed in, L copy deck, M emailed code (entry point), N offline, O deep links, B1 second-step panel | yes |
| E2a, E2e, E2b, E2c | /signup | Default at 390 (tall), 720, 768, 1440 | no |
| E2d signup-states | /signup | A username (checking, taken with chips, free, reserved), B email (typo, malformed, 409), C password requirements, D suggested password and confirm, E currency (loading, failed, chosen), F phone currency sheet (glass, content zone B), G referral, H and H2 submit with problems (summary), I creating, rate limited, failed, J hand-off to /verify-email-pending, K copy deck, L offline, M lost answer, N query parameters, currency listbox at 768 and 1440 | yes |
| E3a, E3b, E3c | /forgot-password | Default at 390, 768, 1440 | no |
| E3d forgot-states | /forgot-password | A field error, B sending, C check your email (and 1440 crop), D rate limited, E 503, F something went wrong, G copy deck and BE5 (card 916), H offline, kept-input table | yes |
| E4a, E4b, E4c | /reset-password | Default at 390, 768, 1440 | no |
| E4d reset-states | /reset-password | A dead link (and 1440 crop), B rules not met and no match, C all met, D saving, E password changed, F page loading, G save failed, H copy deck, I offline | yes |
| E5 a11y | all four | Contrast (the CTA 2.77:1 limitation kept as approved), control corrections, glass proof, focus order, names and announcements, 44 px targets, motion, B5 checklist | yes |
| E6 decisions | all four | Status per item, product rules PR-E1 to PR-E4, decisions D-E1 to D-E10 and E-Q1, build notes, backend items, proposed DESIGN.md changes, tests per route | yes |
| E7 large | /login, /signup, /reset-password | D at 1440, H at 1440, B at 768, the glass listbox at 768 | yes |
| E8 reflow | all four | 320 px, keyboard frames K1 to K3 (320 and 360), 200 % text | yes |

Routes built: `/login`, `/signup`, `/forgot-password`, `/reset-password`, plus the shared `(auth)` layout and `authHeader`. `/verify-email` and `/verify-email-pending` share that layout, so under D-E3 they get the compact header and the Daylight tokens but keep their bodies (excluded from the redesign; regression screenshots only). Not touched: `TwoFactorLoginModal`, `EmailCodeLoginModal` (D-E7, D-E10), `SessionConflictGate`, the legal re-acceptance modal, `/auth/callback/google`, `use-oauth.ts`.

Landing (owner rule 7, PR-E3): after sign-in the customer lands on `/wallet` unless a valid same-site return link applies (`?redirect`, a protected page opened while signed out, a forced sign-out), including after the authenticator or emailed-code step. Open redirects stay refused exactly as `safePostLoginTarget` refuses them today.

## (b) Current code map (base-ro 366dca2f)

| File | Lines | Today | Change in 955 |
|---|---|---|---|
| `src/app/(auth)/layout.tsx` | 39 | Renders `authHeader` and children; metadata says "trade BTC, USDT, ETH and SOL" | USDT-only description (E6 build note); page ground; no other change |
| `src/components/common/authHeader.tsx` | 68 | Logo, `nav[aria-label="Task bar"]` with Sign Up and Support, `MobileMenuButton` opening `MobileNav` | Rewrite to logo plus one link (D-E3) |
| `src/app/(auth)/login/page.tsx` | 695 | Form, error and notice banners, AUTH-015 panel, 2FA and emailed-code handlers, Google block behind `isGoogleConfigured`, toasts for every error | Rebuild the markup; keep every handler and challenge path |
| `src/app/(auth)/signup/page.tsx` | 1,454 | Hero image and testimonial, eight fields, username checks and suggestions, typo map, password generator, 5-rule checklist (labels "At least 1 ..."), Radix Select currency, terms box, disabled submit, toasts, Google block | Rebuild; extract pure helpers first |
| `src/app/(auth)/forgot-password/page.tsx` | 302 | Hero image, gradient button, "Email Sent Successfully", raw error text inline and as a toast | Rebuild (states A to H) |
| `src/app/(auth)/reset-password/page.tsx` | 223 | Hero image, 6-character client check, 3-second redirect, spinner fallback | Rebuild (states A to I), 12 characters and five rules on the client |
| `src/components/ui/password-input.tsx` | 147 | Eye toggle without `aria-pressed`, 16 px icon, unused `showStrength` meter (no caller sets it) | Customer look: 44 px toggle, `aria-pressed`, name switch, reset to hidden on submit (shared with 956) |
| `src/lib/auth/login-redirect.ts` | 74 | `DEFAULT_POST_LOGIN_PATH = "/dashboard"`, `safePostLoginTarget`, `loginUrlFor` (card 935) | Default `/wallet`; add a helper that builds switch links carrying a safe `redirect` and `ref` |
| `src/components/providers/auth-provider.tsx` | 120 | Signed-out on a non-public path goes to `loginUrlFor(...)`; signed-in on /login goes to the safe `redirect`, on /signup always to the default | /signup honours `redirect` too (E1d K, E2d N) |
| `src/lib/auth/public-routes.ts` | 44 | Omits /verify-email, /verify-email-pending, /auth/callback/google | No change (owner rule; own card) |
| `src/lib/api/http-client.ts` | 609 | 429 becomes "Too many requests. Please wait a moment." with `data` from the JSON body and the `Retry-After` header dropped; forced sign-out goes to bare `/login?reason=session_expired` (L89) | Keep message; add `retryAfterSeconds`; forced sign-out adds `&redirect=` |
| `src/lib/auth/session-idle.ts` | 196 | `redirectToLogin` sends bare `/login?reason=...` (L113) | Add `&redirect=` (E1d E, O) |
| `src/services/auth-api.ts` | 923 | `login`, `signup`, `resetPassword` throw `new Error(message)`, losing `status` and `code` | Throw `ApiResponseError` with the same message (see drift item 5) |
| `src/hooks/auth/use-auth-actions.ts` | 145 | `forgotPassword` throws `new Error(message)` | Same as above |
| `src/config/routes.ts` | ~700 | `CUSTOMER_SURFACE_RULES` excludes every auth route, so auth pages still render the April theme (dark by default) | Add the six (auth) routes |
| `src/lib/constants.ts` L106 | 1 | `PASSWORD_MIN_LENGTH = 8` (no importer) | 12 (E6 build note) |
| `src/hooks/auth/*`, `src/store/auth-store.ts`, `TwoFactorLoginModal`, `EmailCodeLoginModal`, `TwoFactorCodeInput`, `use-oauth.ts`, `/auth/callback/google` | | Challenge handling and OAuth | No change |

Shared pieces already on the branch: `ui/button` (customer variants, 52 px `lg`), `ui/field`, `lib/customer-field-classes` (focus edge `--focus`), `ui/checkbox` (`--action-pressed`), `ui/radio-group`, `ui/spinner`, `ui/skeleton`, `ui/dialog` (opens as a `gl-sheet` below 768 on the customer surface, 969), `ui/toast` (one at a time, 6 s, above the dock, 969), `navigation/Logo` (QicIcon below md, QicLockup from md), glass classes in `globals.css`. Missing: a four-tone banner, the password requirements component, suggestion chips (970's replication follow-up adds tappable chips and the Select customer look but is not merged into base-ro yet), an error summary.

## (c) Gap per board

1. E0 and E8 layout. Today: full-bleed split with hero art at 768 and up, `max-w-md` centred column, no card. Needed: phone column (viewport minus 32, max 400, centred, top 135 on phone and 88 at 720), 480 px card with 40 px padding and a fixed top at 136 from 768, title 24/30 then 28/36, 48 px fields, 52 px button, 20 px field gap, "Need help? Contact support" at the foot (phone) or 24 px under the card. 320 px, keyboard (scroll margin, `enterkeyhint`) and 200 % text without horizontal scroll.
2. Header (D-E3, all frames). Today: Task bar nav, Sign Up, Support, menu, `MobileNav`. Needed: shield (below 768) or lockup, one link ("Create account" on /login, "Sign in" elsewhere) on `--action-pressed`, 56 or 72 px, no menu.
3. Customer surface. Today the auth routes are outside `CUSTOMER_SURFACE_RULES`, so none of the Daylight tokens or the `customer:` control looks apply and the stored theme (default dark) wins. Needed: Daylight on all six (auth) routes, no theme toggle.
4. E1a to E1e, E1d. Copy deck L throughout. A: field messages on submit, then on blur, focus the first invalid field, `aria-describedby`, no toast. B: one banner for 401 ("We couldn't sign you in" / "The email or password is incorrect. Check both and try again, or reset your password."), email kept, password cleared, focus on password; 5xx uses the same banner with "Something went wrong on our side. Try again in a moment.". C: "Signing in...", fields read-only, `aria-busy`. D: "Too many sign-in attempts" with a clock time from Retry-After, button disabled until then and re-enabled on its own. E: session-ended info banner (role=status), hidden when an error shows. F: deactivated (warning) and banned (error) copy, no reactivation. G: AUTH-015 panel kept in place (restyle only through tokens). H, M: dialogs untouched. I: 44 px toggle. J: "Welcome back" / "You're signed in." toast on the destination. K: "You're already signed in" status, never the form, button after 1 s. N: offline banner and "You're back online" status. O: return links through every step; switch links carry `redirect` and `ref`. Remove every error toast.
5. E2a to E2e, E2d. Hero art, testimonial and header box go; heading and D-E6 subtitle in the card. A: username chips in the flow (`username-suggestions`, `username-suggestion-N`), hint "3 to 30 characters: letters, numbers, _ or -." replaces the info bubble. B: typo suggestion with "Use it", 409 copy (D-E1). C: "Password requirements" with "n of 5 requirements met" shown before typing, the five rules in group D's words, fold at 5 of 5. D: "Suggest a password" fills both fields and shows the value, "Copy password" then "Copied" for 2 s, no toast; match line. E and F and listbox: currency from the live catalog, phone glass sheet (content zone B, radio rows), from 768 a `.gl-menu` listbox with rows on the frost, `aria-activedescendant`, type-ahead; keep `signup-currency-*` testids. G: referral line instead of the "removed" toast. H and H2: button never disabled, error summary (role=alert, takes focus, links to fields, terms box item). I: "Creating account...", 429 clock time, server error, nothing cleared. J: no "Account created!" toast. L, M: offline and lost answer (repeat meets 409). N: switch link carries `redirect`.
6. E3a to E3c, E3d. All states new except the field and the basic send. C replaces the form, focus to heading, "Send another link" locked until request time plus 10 minutes with the time shown, "Use a different email". D clock time and same words for both limits. E (503) and F (other 5xx or no answer) with "Try again". H offline. No toasts.
7. E4a to E4c, E4d. A replaces "Invalid Reset Link" with "This reset link doesn't work" and a primary "Get a new link" (on load without token and on a 400 after save). B, C: the requirements component and 12 characters on the client. D: "Saving...". E: "Password changed", no redirect (today 3 s). F: skeleton instead of the spinner. G: save failed copy. I: offline.
8. E5. Focus order equals DOM order with banners and summaries before the first field; labels with `for`, `autocomplete` (username, current-password, email, new-password, name); `aria-invalid`; one announcement per message; 44 px targets (Forgot password?, Suggest, Use it, chips via the hit-area extender); reduced motion for every motion. The CTA label stays at 2.77:1 (documented limitation, not a defect).
9. E6. Build notes listed in (d); proposed DESIGN.md entries; tests per route.
10. E7. Proves the large-screen states reuse the phone column in the card; the listbox at 768.

## (d) Slice plan

Each slice is one PR into `redesign/906` (Qictrader2/Frontend), reviewable on its own, at most about 1,500 product lines (tests not counted). Slices are file-disjoint: no product file appears in two slices. Line figures are added plus changed product lines.

| # | Slice | Product files | Size | Depends on |
|---|---|---|---|---|
| S1 | Shared password requirements (separable, shared with 956) | new `src/lib/auth/password-rules.ts` (the five rules as a typed list, `evaluatePassword`, met count, symbol set from signup L264-269, `PASSWORD_MIN_LENGTH` 12); new `src/components/ui/password-requirements.tsx` ("Password requirements", polite count debounced 1 s, icon plus words per rule, hidden "met" or "not met yet", cross after a submit, fold at 5 of 5 in 200 ms, reduced motion instant, `data-testid="password-requirements"`); `src/components/ui/password-input.tsx` customer look (44 px toggle, `aria-pressed`, name switch, a way for the form to hide it again on submit; default classes byte for byte, `customer:` additions only); `src/lib/constants.ts` L106 | ~350 | none (whichever of 955 or 956 lands first builds it; the other imports it) |
| S2 | Auth plumbing, no UI | `src/lib/auth/login-redirect.ts` (default `/wallet`; `authSwitchHref(path, search)` that keeps only a `redirect` passing `safePostLoginTarget` and a sanitised `ref`); `src/components/providers/auth-provider.tsx` (/signup honours `redirect`); `src/lib/api/http-client.ts` (429 keeps its message and adds `retryAfterSeconds` from `Retry-After`; forced sign-out adds `&redirect=` via `loginReturnTarget`); `src/lib/auth/session-idle.ts` (same); `src/services/auth-api.ts` and `src/hooks/auth/use-auth-actions.ts` (throw `ApiResponseError` with the unchanged message); new `src/lib/auth/auth-errors.ts` (pure classifier: invalid credentials, rate limited with retry time, offline or no answer, server, 503 mail unavailable, 409 email taken, 400 reset token, banned, deactivated, unverified; 24-hour clock formatter in the user's time zone); new `src/hooks/use-online-status.ts` | ~400 | none |
| S3 | Auth shell and header | `src/config/routes.ts` (six auth routes in `CUSTOMER_SURFACE_RULES.paths`); `src/components/common/authHeader.tsx`; `src/app/(auth)/layout.tsx`; new `src/components/features/auth/AuthPage.tsx` (column, card, title, subtitle, foot link); new `src/components/ui/banner.tsx` (info, success, warning, error; icon, bold first line, optional link; `role` prop; 150 ms opacity fade); new `src/components/features/auth/AlreadySignedIn.tsx` (E1d K) | ~500 | S2 (switch link helper) |
| S4 | /login | `src/app/(auth)/login/page.tsx`; new `src/lib/auth/login-form-state.ts` (pure reducer for which banner and field state shows) | ~650 | S2, S3 |
| S5 | /forgot-password and /reset-password | `src/app/(auth)/forgot-password/page.tsx`; `src/app/(auth)/reset-password/page.tsx`; new `src/lib/auth/reset-request-lock.ts` (pure: "send another link" unlock time from request time plus 10 minutes) | ~650 | S1, S2, S3 |
| S6a | Sign-up helpers moved out of the page, no behaviour change | new `src/lib/auth/signup-rules.ts` (profanity and impersonation lists, `normalizeLeetspeak`, username format check, typo map, suggestion generators, `generateStrongPassword`) moved verbatim; `signup/page.tsx` imports them | ~350 moved, net ~0 | none |
| S6b | Sign-up components | new `src/components/features/auth/CurrencyPicker.tsx` (phone: `Dialog` sheet with `RadioGroup` rows, content zone B; from 768: `.gl-menu` listbox; reuse 970's Select customer look if merged by then); new `UsernameSuggestionChips.tsx`; new `src/components/ui/error-summary.tsx` (role=alert, focus, links move focus) | ~550 | S3; reads S6a |
| S6c | /signup page | `src/app/(auth)/signup/page.tsx` | ~850 | S1, S2, S3, S6a, S6b |
| S7 | Tests and evidence | e2e and unit test updates only (below), the capture register | 0 product | S4 to S6c |

Order: S1 and S2 in parallel, then S3, then S4, S5 and S6a in parallel, then S6b, then S6c, then S7. S3 alone changes the header and theme on pages whose bodies are still old, so merge S3 with S4 and S5 close together on `redesign/906` and run review-app QA only after S6c (the branch never reaches staging or production before Christian's final approval).

Kept on purpose in S4 and S6c: ids `#email`, `#password`, `#fullName`, `#username`, `#confirmPassword`, `#affiliateCode`, `#acceptTerms`, `#preferredCurrency`, `#new-password`, `#confirm-password` and every testid E6 lists; removed `registration-header-box` and `username-info-bubble`. The `isGoogleConfigured` branches stay in both pages unchanged and unstyled (drift item 10). Every challenge handler in /login (2FA, emailed code, resend, close, legal flag, session-conflict count, token setting) moves without edits; only the JSX and the error mapping change.

Test changes (S7 and inside each slice): `login-redirect.test.ts` and `ticket-935-login-redirect.test.tsx` expect `/wallet`; `http-client-session-expired` and `use-idle-timeout-591` expect `&redirect=`; `ui-fixes.test.ts` and `ticket-tbTLFRQa-username-focus.test.ts` read the new helper path (S6a); e2e `auth-004` L76 `/wallet`, `auth-008` L35/L37 "Check your email", `signup-acceptance` L54-59 summary item, delete `ticket-782` L127-151, rewrite `usr002`, `registration-taskbar` delete REG-001/002 and adjust L27, L81, L93, `eml003` L48/L59 shared words, `auth-001` L104-109 exact 409 copy, `affiliate-referral-flow` L392-393 add `/wallet` (one owner across B, C, D, E); new specs from tests.md (header, submit with problems, requirements count and no Weak/Fair/Strong, reset at 11 and 12 characters, landing and return links after each second step, the three second-step cases, no Google at 390/720/768/1440, control colours, offline on all four).

## (e) Acceptance mapping

| Card AC | Covered by | Status |
|---|---|---|
| AC1 sign up, sign in, request a reset, set a new password through the emailed link; every state at 390 x 844 and 1440 x 900 | S3 to S6c; boards also require 768, 720 and E8 | Buildable. Server half of "set a new password" with 12 characters and five rules needs 917 and 964 (see AC2). Sign-up hands off to `/verify-email-pending`, which is not in `PUBLIC_ROUTES`: if a signed-out new customer is bounced to /login there, AC1's sign-up outcome fails for a reason outside 955; needs the verification-routes card (no card exists yet). 755 has an empty description; nothing to build against (coordinator: close as covered by 955 or link) |
| AC2 "Password requirements" with a live count; 11 refused, compliant 12 accepted; wording matches change password in 906-D | S1, S5, S6c | Client side buildable. Blocked for the server refusal on reset: card 917 (reset accepts 6) and card 964 (no server character rules on any endpoint). Sign-up already refuses 11 on the server (length only). "Matches 906-D" depends on 956 importing S1 |
| AC3 unverified reaches verification; 2FA account reaches its challenge (authenticator, emailed code, backup code); new device without authenticator gets the emailed code; no Google option; lands on /wallet; a protected link opened signed out returns there | S2, S4 | Buildable (handlers unchanged; landing and return links in S2). Verification after sign-up shares AC1's blocker (no card). No Google option holds only while `NEXT_PUBLIC_GOOGLE_CLIENT_ID` is unset (checked: no Google client id in the review-app bundle) |
| AC4 wrong password, locked or rate limited (64), expired reset link, network failure: specified message, fields kept except passwords; no new account-existence signal (916 owns the existing one) | S2, S4, S5, S6c | Buildable. The clock time needs a readable `Retry-After`: card 64 (done) sends it, the backend CORS layer exposes only `content-disposition`, and the frontend's same-origin `/api/v1` proxy forwards it. If the review app calls the backend cross-origin, the clock time cannot show there without a backend change (no card; card 64's area). The existing forgot-password signal stays until card 916 |
| AC5 old links (redirect targets, reset tokens) keep working; current, proposed and implemented captures; WCAG 2.2 AA, labels, announcements, 320 px, 200 %, keyboard-only | S2 to S7 | Buildable. WCAG 2.2 AA passes except the documented CTA label limitation (2.77:1, kept as approved by Christian, E5 and DESIGN.md section 12) |

Checklist items: 1 = AC1 (+917, 964 for reset); 2 = AC2 (blocked server-side by 917, 964); 3 = AC3 (verification-routes card needed); 4 = AC4 (Retry-After visibility; 916 for the existing signal); 5 = AC5.

Backend cards named by the owner that no E AC depends on: 920, 918, 911, 963, 910, 919 (trades, fees, disputes, held withdrawals). 904 owns the emailed-code dialog, which stays as it is.

## (f) Risks

1. Auth flows. The /login rebuild touches the file that holds the 2FA, emailed-code, unverified, banned and deactivated paths. Move handlers verbatim, change only JSX and error mapping, and keep `ticket-IrQ37kKt-new-device-email-code`, `ticket-9dl9xx7a-backup-code-login` and the staging emailed-code e2e green. No QA account has 2FA; the qic-qa-signin agent can turn it on for a staging test user and must restore it. The emailed code needs a fresh browser profile and the sign-in agent's mailbox access.
2. Moving auth routes onto the customer surface changes the excluded dialogs and the verify-email pages through shared tokens (both dialogs use `ui/button`, which has `customer:` variants). Regression screenshots of both dialogs and both verify-email pages before and after S3.
3. Return links. Keep `safePostLoginTarget` as the only gate (card 935 rules: one leading slash, no `//`, no backslash, no control characters, same origin, no /login or /signup loop, KYC callback query stripped). Switch links must pass `redirect` through the same check before writing it. The forced sign-out redirect must never carry a reset token (reset is public, so no forced redirect there) and uses `loginReturnTarget`. AuthProvider and the page both call `router.replace` after sign-in; point both at the same target so they do not race (E6 build note).
4. No new account-existence signal. One message for any 401; deactivated and banned only after a correct password (backend checks the password first); one rate-limit message for both limits; forgot-password always says "If an account uses ..."; the "send another link" time is computed from the request time, identical for every email; the 503 is checked before the lookup. Sign-up keeps its 409 message (D-E1). The existing forgot-password difference after the lookup stays until 916.
5. Rate limits. The page can only show a clock time when `Retry-After` reaches the browser (risk in (e) AC4). In-memory buckets reset on a dyno restart, so QA timing can vary. Verify which API origin the review app calls (network tab) before testing D.
6. Error classification. Without S2's `ApiResponseError`, pages would match message strings; S2 keeps every message identical so `use-email-auth`, `auth-api` and `auth-store` tests that assert "Invalid credentials" still pass.
7. Sign-up never disabled. A pending username or email check, an unrecognised referral and the terms box must all block sending and appear in the summary; nothing may be sent while a problem remains.
8. Merge order with siblings: HANDOVER order is 957, 956, 955. If 956 lands first with S1, 955 drops S1 and imports it; if the 970 replication follow-up (chips, Select customer look) is not merged when S6b starts, S6b builds against base and swaps later.
9. The interim branch state after S3 shows the new header over old bodies; acceptable on `redesign/906` only.

## (g) Card text versus boards (for the coordinator; boards win)

1. Card: sign-up and reset "show a Weak/Fair/Strong meter". Code: no auth page sets `showStrength`; sign-up shows a five-rule checklist when the field is focused, reset shows nothing. The outcome is unchanged.
2. Card AC1 names 390 x 844 and 1440 x 900 only. Boards also require 768, 720 (E1e, E2e) and E8's 320, 360 keyboard and 200 % text.
3. Card AC4 "locked". No board draws a separate lockout; D uses one message for both limits.
4. Card: "Keep `use-auth`, `auth-api` ... unchanged". E6 build notes require Retry-After passthrough in `http-client.ts` and `&redirect=` in `http-client.ts` and `session-idle.ts`; S2 also changes the thrown error class in `auth-api.ts` and `use-auth-actions.ts` with messages unchanged. If the coordinator refuses the class change, S2 falls back to message matching in `auth-errors.ts`.
5. Card scope lists four routes; D-E3 also changes the header on `/verify-email` and `/verify-email-pending`.
6. Board E1d J: toast 4 s. Owner decision of 22 Oct: 6 s everywhere including sign-in (already shipped by 969). Owner wins; no code change.
7. Board E6 build note and owner rule 7: default landing `/wallet`. Card 935 AC3 (done) says an evil redirect lands on the dashboard; its test changes to `/wallet`.
8. Board E1d K and E2d N: /signup with a live session goes to `?redirect` else `/wallet`; today /signup ignores `redirect`.
9. 755 ("Reset password page is terrible") has no description; the owner lists it as a backend dependency, but nothing in it is backend.
10. Google: boards draw no button; PR-E1's build note says leave `NEXT_PUBLIC_GOOGLE_CLIENT_ID` unset. Plan keeps the dormant `isGoogleConfigured` branches unchanged (also read by `ui-fixes.test.ts` L78-84) rather than deleting them, so no auth policy changes. Coordinator may choose deletion instead; that needs 761's owner.
11. Board D-E9: large-screen states reuse the phone column (E7 proves four); QA captures at 768 and 1440 for the default frames only, as the board allows.

Impossible item to record if it occurs: the D clock time (login, sign-up and forgot-password) when `Retry-After` cannot be read. No board draws a banner without a time, so ask the coordinator for the fallback wording before building it rather than inventing copy.

## (h) Files other page-group cards also touch

| File | 955 change | Other cards |
|---|---|---|
| `src/lib/auth/password-rules.ts`, `src/components/ui/password-requirements.tsx` (new) | S1 | 956 (change password, D3a to D3c): whichever lands first builds it |
| `src/components/ui/password-input.tsx` | S1 | 956 (`ChangePasswordSection.tsx` uses it; 956 removes its own Weak/Fair/Strong meter) |
| `src/lib/constants.ts` | S1 | possibly 956 |
| `src/components/ui/banner.tsx` (new) | S3 | 958 (HARC held banners, B19), 956, 957 may need the same four tones |
| `src/components/ui/error-summary.tsx` (new) | S6b | 958 forms if they adopt a summary |
| `src/config/routes.ts` | S3 (`CUSTOMER_SURFACE_RULES`) | 968 owner; 958 (/earnings), 956, 957 may add paths |
| `src/lib/auth/login-redirect.ts`, `auth-provider.tsx` | S2 | 958 (Wallet default landing on B0; the /dashboard to /wallet redirect) |
| `src/lib/api/http-client.ts` | S2 | 958 (step-up limits B9e, B9f would use `retryAfterSeconds`), 959 |
| `src/lib/auth/session-idle.ts`, sign-out tests | S2 | 956 (sign-out entry points) |
| `src/components/common/mobile-nav.tsx` | none (authHeader stops importing it) | 968, 956 |
| `e2e/tests/affiliate-referral-flow.test.ts`, `e2e/pages/login.page.ts`, `src/__tests__/ui-fixes.test.ts` | S7 | 958, 957, 956 (cross-pack 4.3: one owner per file) |
| Root `DESIGN.md` (root repo PR) | scope item 4 drops "auth"; auth task page, banner, password field, chips, short pick list, anti-patterns from E6 | every 906 card |

## (i) Honest size

- Product code: about 4,300 lines added or changed across 9 PRs (S1 350, S2 400, S3 500, S4 650, S5 650, S6a 350 moved, S6b 550, S6c 850), replacing about 2,700 lines of the four old pages; net about +1,600.
- Tests: about 1,800 lines (unit for the pure modules and components, page tests, 10 new e2e specs, about 12 adjusted e2e specs).
- Effort: XL. Roughly 4 to 6 agent build days including reviews, plus 1 to 2 days of review-app QA (2FA setup, emailed-code device, offline, 320 and 200 %, captures for every state). The register's `build_effort_h: 4` is not realistic.
- Blocked and left unticked at the end: the server half of AC2 and checklist item 2 (917, 964); possibly AC1/AC3 sign-up verification hand-off (no card yet); possibly the AC4 clock time on the review app (Retry-After visibility).
