# 956 implementation plan: Customer redesign 906-D (profile, settings, notifications, support)

Planning only. Nothing here was built, committed or written to Trello.

- Card: 956 https://trello.com/c/BGcCmK7g. I read its description and the five-item Acceptance Criteria checklist through the REST API on 22 Oct 2026. The card has no comments.
- Base: `redesign/906` at `origin/redesign/906` 9bbd3d88. That commit includes 970 (#304), 968 (#305), 969 (#306) and 968's support chat panel (#307). I also read the diff of `origin/970-v52-replicate` and assume it is merged.
- Design: v5.2 group D sources in `.scratch/customer-app-redesign/research/design-pack/v5.2-src/D-account/` (design-notes.md with "Change log: 21 Oct v5.2", register.json, project/*.dc.html, d.css) and the PDFs in `~/Downloads/906-design-pack-v5.2/`. The boards are the spec. Each v5.2 board is cited by its id (D0 to D8b).
- Backend facts were checked read-only at qictrader-backend-rs 4e3957e.

---

## (a) Boards and routes

v5.2 changed 16 group D boards, and each has a PDF in `~/Downloads/906-design-pack-v5.2/`. Seven boards are unchanged since v5 and carry forward: D1a, D1b, D2a, D2b, D2c, D5b and D5c. They have no v5.2 PDF, so build them from their `.dc.html` source, which is byte-identical to v5.

| Board | Source file | Route and state | v5.2 | Owner in this plan |
|---|---|---|---|---|
| D0 | Main.dc.html | Overview: navigation levels (dock, then rows or pills, then breadcrumb), journey states table (B3), "Leaves / Arrives / Renamed", 2FA as built | changed | all slices (reference) |
| D1a / D1b / D1c | profile-entry-390/768/1440 | /profile Account destination: top, end, chat sheet or panel, Jump, loading | D1c shared only | **built by 968** (AccountHub, chat panel, Jump). 956 adds only the pills (D1b, D1c) and the body above the hub |
| D2a / D2b / D2c | profile-390/768/1440 | /profile whole page, new account, reviews failed, couldn't load, signing out | unchanged | S3 |
| D3a / D3b / D3c | settings-390/768/1440 | /settings: one section per page on a phone, a 208 px index at 768 and above; Personal details, Preferences and alerts, Security and sign-in, Payment methods, Verification; save bar; saved, save failed, validation, offline; today's 2FA panels drawn as "unchanged, excluded" | changed | S4a, S4b, S5a, S5b |
| D4a / D4b / D4c | notif-390/768/1440 | /notifications and the bell popover: All/Unread, type sheet (390), select (768) or chips (1440), item options, deleted with Undo, caught up, none yet, loading, couldn't load, offline | changed | S6 |
| D5a / D5b / D5c | tickets-390/768/1440 | /my-tickets and /my-tickets/[id]: list, empty, loading, couldn't load, new ticket sheet or dialog (validation, sending, couldn't confirm); thread, reply file error, sending, resolved, closed, not found, keyboard, offline | D5a shared only | S7a, S7b |
| D6a / D6b / D6c | report-390/768/1440 | /report: form, validation, sending, sent, couldn't confirm, signed out, ?user=, ?trade=, offline | changed (radio and focus roles) | S8 |
| D7a / D7b | a11y-1/2 | WCAG 2.2 AA proof, 320 and 360 px, 200%, keyboard, reduced motion, glass checks | changed | QA for every slice |
| D8a / D8b | tests-1/2 | Decisions, statuses, corrected links (D8b section 1), backend items (section 2), source findings (section 3), tests per route (section 4), v5.2 tests (4b) | changed | QA for every slice |

Every route is "Complete at 390" (M19, decided by the owner on 14 Oct). At 768 and 1440 the reused states take the phone card's content in the large layout, as register.json lists.

Route inventory (code at 9bbd3d88):

| Route | File | Lines |
|---|---|---|
| /profile | `src/app/(main)/profile/page.tsx` | 152 |
| /settings | `src/app/(dashboard)/settings/page.tsx` | 317 |
| /notifications | `src/app/(dashboard)/notifications/page.tsx` | 787 |
| /my-tickets | `src/app/(main)/my-tickets/page.tsx` | 275 |
| /my-tickets/[id] | `src/app/(main)/my-tickets/[id]/page.tsx` | 425 |
| /report | `src/app/(main)/report/page.tsx` | 349 |

The card's `settings/notifications` route does not exist. See (g).

---

## (b) Current code map

**Already built on redesign/906 (do not redo)**
- `components/navigation/AccountHub.tsx` (346 lines, 968) holds the Settings, Activity and Help groups, a Staff group gated by `useModeratorAuth` and `useAdminAuth` (also investor), Sign out, legal links and version. Its config lives in `config/routes.ts` (`ACCOUNT_HUB_SECTIONS` at line 816, `JUMP` index at line 976).
- `components/navigation/PhoneHeader.tsx` plus `PHONE_SUB_HEADERS` in routes.ts provide level-2 phone headers for /settings (title from `SETTINGS_TAB_TITLES`), /notifications (Back), /my-tickets, /my-tickets/[id] and /report.
- Support chat: `components/support/SupportChatPanel.tsx`, `support-chat-state.ts` (968, #307).
- Bell popover: `components/navigation/NotificationsGlass.tsx` and `NotificationsDropdown.tsx` (968).
- Shared kit (970, 969): `ui/dialog.tsx` (glass dialog and sheet, `material="solid"`), `ui/toast.tsx` (one toast, no action button), `ui/status-chip.tsx`, `ui/error-state.tsx`, `ui/empty-state.tsx`, `ui/skeleton.tsx`, `ui/radio-group.tsx`, `ui/switch.tsx`, `ui/loading-button.tsx`, `ui/breadcrumb.tsx`, `ui/password-input.tsx`.

**/profile**
- The page still renders `ProfileHeader`, `WalletOverview`, `ActiveTrades`, `ProfileReviews`, `AffiliateDashboard`, `TransactionHistory` and `TrustSafety`, all in `components/profile/`. D2 removes every money section.
- Data comes from `hooks/api/use-profile-page.ts` (539 lines), which fetches the wallet, affiliate, transactions and trust-and-safety data.
- `?tab=resells` renders `FlipDeals`. `offer/resell/_components/ActiveResellsSidebar.tsx:174` links to it.
- The six profile components above are used only by /profile. The same-named files under `features/` are different components.
- `ProfileReviews` is shared with `PublicProfileView` (group C, 957).
- `profile/[userId]/page.tsx:21-27` redirects your own id back to /profile.

**/settings**
- One page with four `Tabs` (general, security, payment-methods, verification). The URL comes from `?tab=` and tab changes call `router.push`.
- An unknown `?tab=` value renders an empty page, because no `TabsContent` matches.
- The general tab mixes Personal details (`ProfileSection.tsx`, 467 lines) with the display currency select and the email toggles, which are inline in page.tsx.
- Security: `SecuritySettings.tsx` (430), `security/ChangePasswordSection.tsx` (299, strength meter, 8 characters), `TwoFactorSection.tsx` and `TwoFactorEnabledState.tsx` (excluded, 904), `ActiveSessionsSection.tsx`, `VerificationStatusSection.tsx` (to be removed), `BlockedUsersSection.tsx`. The "View trade" link points to `/trades/{id}` (SecuritySettings.tsx:165).
- Payment methods: `PaymentMethodsSection.tsx` (595).
- Verification: `KYCVerificationContent.tsx` (excluded except the retry link at 174-184, which points to `?tab=profile`), `TradingLimitsSection.tsx` (formats limits as dollars).
- Sign out sits at the bottom of the page. D moves it to /profile, where 968 already placed it.
- Hooks: `use-settings-page.ts`, `use-users.ts` (email prefs), `use-display-fiat.ts`, `use-payment-methods.ts`, `use-kyc.ts`, `use-usdt-to-fiat.ts`.
- `lib/constants.ts:106` `PASSWORD_MIN_LENGTH = 8` has no consumers.

**/notifications**
- One 787-line page with stat cards, eight chips, numbered pages and a second "settings" tab that holds the in-app preferences (`useNotificationPreferences`).
- On error the page shows "You're all caught up!" (around lines 547 to 556).
- The type-to-route switch lives inline at lines 220 to 290. Offers go to `/offers/{id}` and `/offers`, which have no pages.
- The bell has its own switch (`NotificationsDropdown.tsx:55-71`) that sends support rows to `/moderator/support-tickets` for everyone. Both follow `actionUrl` first through `lib/notification-route.ts`.
- `NotificationIcon[type]` has no fallback, so an unknown type renders `undefined` and crashes with React #130 (pre-existing).
- The bell popover has no loading or error state, and its "Alert settings" button points to `?tab=general`.

**/my-tickets**
- The list has no create action. "New ticket" sends people to /contact-us. Status labels are raw, it shows a Priority column, and errors print the raw `{error}` string.
- `[id]` prints the server's error for someone else's ticket ("you do not have access to this ticket"), which confirms the ticket exists. It renders an empty page for a missing id.
- `services/support-api.ts` `getTicket` throws `new Error(message)` and drops the HTTP status.
- Attachments render as `<img src={API_CONFIG.baseUrl + attachmentUrl}>` and `<a href>`. The backend builds `attachmentUrl` as `/api/v1/support/tickets/{id}/attachments/{file}/download` (support.rs:744) and serves it only to the owner or staff (support.rs:783-797, AuthUser, bearer token or `qic_access` cookie). The base URL already ends in `/api/v1`, so the path probably doubles. `lib/api/authenticated-blob.ts` strips that prefix and sends the bearer token.
- Hooks: `use-support.ts` (`useTickets`, `useTicket`, `useCreateTicket`, `useAddMessage`, `useUploadTicketAttachment`, `useCloseTicket`).
- `createTicket` returns the id, which the service maps from `id` to `ticketId`.

**/report**
- Five type buttons with white-on-grey titles. "QIC Escrow Hub" copy. A priority is computed but never sent.
- The page reads no query parameters.
- Signed out, it shows the form with a disabled Send button.
- It creates a `[REPORT] ` ticket through `useCreateTicket`. A report is a ticket, so it has the same owner-only access.

**No shared component exists yet** for any of the following:
- section pills (`.pill` in qt.css)
- an online/offline hook or banner
- a leave guard for unsaved edits
- a toast action (Undo)
- a password requirements list
- `interactive-widget=resizes-content` in the viewport (`lib/seo/metadata.ts` `siteViewport`)

---

## (c) Gap per board

**D1b, D1c (/profile entry at 768 and 1440).** Missing:
- the Account pills (Profile, Settings, Notifications, Support), with the active pill on `--ink-surface`. Support also covers /report. There is no Overview pill.
- the two-column body at 1440: the left column holds you, reputation, reviews, and Verification and security; the hub is on the right at 360 px (968 already places it).

The hub's "Preferences and alerts" row and the Jump entry point to `?tab=general`. D1a note 5 says `?tab=preferences`. Fixed in S4a.

**D2a to D2c (/profile body).** Everything above the hub is a gap:
- Identity card: avatar, name, "Online now · joined", ID verified, "Standard merchant tier", bio, Edit or "Edit details" (opens `/settings?tab=general`), and "See how others see you" (opens `/profile/{id}?view=public`).
- One reputation block (Trades, Completion rate, Typical reply, Rating). A new account shows honest words: "No trades yet", "No replies yet", "No rating yet".
- Reviews: average, count, "Excellent · 92% positive", the 2 latest (3 on larger screens), "See all N reviews" expanding in place to 50, an empty state with "Find an offer", and the section-level "We couldn't load your reviews".
- Verification and security grid: Identity opens `?tab=verification` (not `/settings/kyc`), Email, Phone ("Add" opens Personal details), and 2FA ("Turn on" opens `?tab=security`).
- Whole-page "We couldn't load your profile", with the shortcuts still working.
- Loading skeletons after 300 ms with labels rendered at once.
- Removed: the wallet, active trades, affiliate dashboard, transaction history, the four money buttons, Trust score, Disputes won, Affiliate rank and "Escrow Verified".
- The known 320 px sideways scroll.

**D3a to D3c (/settings).**
- Layout: on a phone each section is its own level-2 page. At 768 and 1440 a 208 px index card sits beside the section (912 px section at 1440 with two field columns).
- New `?tab=preferences`. The page title follows the tab.
- Save bar: hidden until an edit. Paper, never glass. Discard. "Discard your changes?" when leaving. Validation summary that focuses the first field and names the count. Saving locks the fields. Saved toast for 4 s. Save failed block with `role=alert`, focus on its heading, and "Try again". Offline banner with the edits kept.
- Personal details: "Change photo" (JPG or PNG up to 5 MB, errors under the photo), "Name on trades", locked username with its reason, the bio rule (no links, 500 characters, counter).
- Preferences: currency autosave with "Saving…" and "Saved" for 3 s, and the previous value restored on failure. Email alerts as autosave switch rows that undo on failure. In-app alerts moved here from /notifications, with System "Always on" as a plain row.
- Security:
  - 2FA card with the Off/On chip and two separate lines ("When you sign in…", "Withdrawals and transfers…"). The second sentence shows only when `/users/me` reports `step_up_2fa`. Today's 2FA panel sits inside the card unchanged.
  - Password row with "Change", which opens the form in place with "Password requirements" and the count.
  - "Devices signed in" and "Blocked users": today's data, restyled.
  - "Close your account" keeps both confirmations and fixes "View trade" to `/trade/{id}`.
  - Remove the empty "Security Options" card and "Verification Status".
- Payment methods: cards below 768 and a table at 1440. Masked last four in Figtree. Default and "Set as default". Remove confirmation as a solid (opaque) sheet on a phone and a 440 px solid dialog at 768 and above, with focus on "Keep it". "Capitec savings removed" toast. Empty state copy.
- Verification: today's view unchanged (excluded), including Proof of residence. Two changes only:
  1. Trading limits read in USDT with a fiat estimate rounded down (CH6, for example "250 USDT a day (≈ R 4,480)").
  2. The retry card's "Edit profile" opens `?tab=general`.

**D4a to D4c (/notifications and bell).**
- Remove the stat cards, eight chips, numbered pages and the settings tab.
- Add:
  - An All/Unread segmented control showing the unread count.
  - A type filter: glass action sheet with radios on a phone, select at 768, single-select chips at 1440. "Support" is a new type. Escrow folds into Trades.
  - Day groups: Today, Yesterday, Earlier.
  - Unread rows on `--action-subtle` with a dot and "Unread" for screen readers.
  - A "Your turn" chip.
  - A 44 px ⋯ options sheet with "Mark as unread" and "Delete" (last row, `--error-pressed`).
  - Delete with a 6 s Undo toast.
  - "Show older", 10 at a time, with a polite "10 more loaded".
  - Empty states per filter in words; "You're all caught up" only for Unread after a successful load.
  - "We couldn't load your notifications", which never shows the empty state and keeps the last known count.
  - Offline: the list stays as loaded and actions wait.
  - An "Alert settings" link to `?tab=preferences`.
  - Corrected targets from the D4a table.
  - A fallback for unknown types (the #130 crash).
- Bell popover: add the loading and error states, Alert settings to `?tab=preferences`, routing through the shared resolver, and no "My Profile" footer (already gone).

**D5a to D5c (tickets).**
- List:
  - Status words: Awaiting your reply (yellow, sorts first), In progress, Open, Resolved, Closed.
  - A "Report" tag instead of `[REPORT] `, plus the department.
  - "Show older" instead of pages. A table at 1440 (subject, department, status, updated). No Priority or Created column.
  - A side column with report, the help centre, and where help lives in a trade.
  - Empty, loading and couldn't load states.
- New ticket in place: glass sheet on a phone, 480 px glass dialog at 768 and above. Fields: department (General, Technical support, Fraud), subject, message (10 or more characters). Sending locks the form. "Ticket sent. You'll get the reply here and as a notification." After a lost response: "We couldn't confirm it was sent. Check Support tickets before sending again." Escape asks before discarding.
- Thread:
  - Breadcrumb at 768 and above: Account › Support › subject.
  - Your messages on the right on `--action-subtle`; support on the left with the agent's name; day headings.
  - 52 px attachment tiles: images open in a new tab, PDFs download.
  - Reply at the end (not sticky). File errors appear on pick (JPEG, PNG, WebP, GIF or PDF, up to 10 MB).
  - Sending. "Reply sent" with a polite status and the box cleared. "Couldn't send. Try again" keeps the text. A lost response gets "We couldn't confirm it was sent. Check the conversation before sending again".
  - Offline keeps the draft, which also survives leaving and returning in the same session.
  - Resolved notice: "If not, reply below and it reopens". Close confirmation in place. Closed marker. One "Ticket not found" for missing, malformed and not yours. "We couldn't load this ticket" with Try again for network failures only.
  - Keyboard rule: the composer stays 8 px above the keyboard.

**D6a to D6c (/report).**
- Real radio cards, 64 px or taller, in one radio group. The chosen card has a 2 px `--action-pressed` edge and a filled radio.
- "Something else" is the default. Trade ID shows for Trade problem and Suspicious user.
- `?trade=` chooses Trade problem, fills the Trade ID, focuses Subject, and Back returns to the trade. `?user=` chooses Suspicious user and fills the person, which goes into the subject as `[REPORT] kabelo_x: …`.
- Counter towards 5,000. Validation summary. Sending lock.
- The sent page names and links the ticket, with one safety line for trade and user reports.
- After a lost response: "We couldn't confirm it was sent", with "Check Support tickets" opening in a new tab. A server error, where nothing was created, reads "We couldn't send your report".
- Signed out replaces the form with "Sign in to send a report…", which opens `/login?redirect=/report`, keeping `?trade` or `?user`, and links /contact-us.
- Offline keeps the report. Leaving with text asks "Leave without sending? Your report will be lost."
- Delete the unused priority code. "QIC Escrow Hub" becomes QicTrader. At 1440, "What happens next" replaces the confidentiality card.

**D7a, D7b, D8a, D8b.** These are QA gates (see (e)). D8b section 4 also lists e2e and unit tests to rewrite, which I folded into each slice.

---

## (d) Slice plan

Each slice is a reviewable PR into `redesign/906`. Slices do not share files: each file is owned by one slice.

Line counts are product lines added or changed, without tests or pure deletions. Tests are extra, roughly 0.7x product.

| # | Slice | Owns (files) | Product lines | Depends on |
|---|---|---|---|---|
| S1 | **Password requirements** (separable, shared with 955) | new `lib/password-rules.ts` (five rules; the symbol class equals sign-up's `signup/page.tsx:265-269`; `metCount`), new `components/ui/password-requirements.tsx` ("Password requirements", "N of 5 requirements met" in a polite live region, icon plus text, is-met and is-miss states, E's `.a-req-n.is-all` success colour, 2-column grid), `lib/constants.ts` (`PASSWORD_MIN_LENGTH` 12) | ~170 | none. **If 955 lands it first, drop S1 and import 955's.** |
| S2 | **Account kit** (shared building blocks) | new `config/account-nav.ts` (pills and the settings index list), new `components/navigation/SectionPills.tsx` (generic qt `.pill`, `aria-current`, `.hit44`), new `components/account/AccountPageHeader.tsx` (pills at md and up, breadcrumb at level 3), new `hooks/use-online-status.ts`, new `components/ui/offline-banner.tsx` ("Back online" polite status, 150 ms fade, instant under reduced motion), new `hooks/use-leave-guard.ts` (beforeunload plus in-page link intercept), `components/ui/toast.tsx` and `store/ui-store.ts` (optional `action: {label, onClick}` for Undo, 6 s), `lib/seo/metadata.ts` (`interactiveWidget: "resizes-content"`) | ~550 | none |
| S3 | **/profile body** (D2, plus the pills on D1b and D1c) | `app/(main)/profile/page.tsx`, new `components/profile/IdentityCard.tsx`, `ReputationBlock.tsx`, `ReviewsCard.tsx`, `VerificationSecurityCard.tsx`, `hooks/api/use-profile-page.ts` (slimmed to the profile, public profile for your own id, reviews and KYC status), `app/(main)/profile/[userId]/page.tsx` (`?view=public` skips the redirect for your own id), `components/profile/index.ts`. **Deletes** `ProfileHeader`, `WalletOverview`, `ActiveTrades`, `AffiliateDashboard`, `TransactionHistory` and `TrustSafety` (profile versions) and rewrites their tests (556, 401, profile-tx-refresh, 818-ac5-profile-history, the 969 references, `ui-fixes` 195-198). `?tab=resells` and `FlipDeals` stay as today. Diagnoses and fixes the 320 px scroll. | ~800 (about 1,600 deleted) | S2 |
| S4a | **Settings shell and Personal details** | `app/(dashboard)/settings/page.tsx` (sections, `?tab` normalisation with unknown values and `profile` falling back to `general`, preferences added, phone section pages, index at md and up, Sign out removed), new `features/settings/SettingsIndex.tsx`, new `components/account/SaveBar.tsx`, new `components/account/SaveFailedBlock.tsx`, `features/settings/ProfileSection.tsx` (rebuild as Personal details), new `features/settings/PreferencesSection.tsx` (today's inline currency and email markup moved as-is), `config/routes.ts` (`SETTINGS_TAB_TITLES.preferences`, hub row and Jump "Preferences and alerts" to `?tab=preferences`) | ~1,050 | S2 |
| S4b | **Preferences and alerts** | `features/settings/PreferencesSection.tsx` (rebuild), new `components/account/AutosaveSwitchRow.tsx`, new `components/account/AutosaveStatus.tsx` | ~420 | S4a |
| S5a | **Security and sign-in** | `features/settings/SecuritySettings.tsx`, new `features/settings/security/TwoFactorCard.tsx` (wrapper only; `TwoFactorSection` and below untouched), `security/ChangePasswordSection.tsx` (uses S1), `security/ActiveSessionsSection.tsx`, `BlockedUsersSection.tsx`, `security/types.ts`; deletes `security/VerificationStatusSection.tsx` | ~950 | S1, S4a |
| S5b | **Payment methods and verification limits** | `features/settings/PaymentMethodsSection.tsx`, `features/settings/TradingLimitsSection.tsx` (CH6 through `use-usdt-to-fiat`, rounded down, USDT only when no rate), `features/settings/KYCVerificationContent.tsx` (the retry link only) | ~600 | S4a |
| S6 | **Notifications page and bell** | `app/(dashboard)/notifications/page.tsx` (rewrite, settings tab removed), new `components/notifications/NotificationRow.tsx`, `NotificationTypeFilter.tsx`, `NotificationOptionsSheet.tsx`, `lib/notification-route.ts` (one resolver for page and bell: D4a table, role-aware support route, unknown-type safe), new `lib/notification-groups.ts`, `components/navigation/NotificationsGlass.tsx` (loading, error, Alert settings), `components/navigation/NotificationsDropdown.tsx` (use the resolver) | ~1,150 | S2, S4b (so the in-app alerts exist in Settings before the tab goes) |
| S7a | **Support list and new ticket** | `app/(main)/my-tickets/page.tsx`, new `components/support/TicketRow.tsx`, `NewTicketDialog.tsx`, new `lib/ticket-status.ts` | ~750 | S2 |
| S7b | **Ticket thread** | `app/(main)/my-tickets/[id]/page.tsx`, new `components/support/TicketConversation.tsx`, `TicketAttachmentTile.tsx` (`fetchAuthenticatedBlob`, revoke on unmount), `ReplyComposer.tsx` (session draft), `services/support-api.ts` (keep `ApiError.status` on `getTicket`, `addMessage`, `uploadAttachment`, `createTicket`), new `lib/ticket-errors.ts` (403, 404, 400 or 422 and a malformed id all map to not found; network errors map to couldn't load) | ~1,050 | S2, S7a (`lib/ticket-status.ts`) |
| S8 | **/report** | `app/(main)/report/page.tsx`, new `components/support/ReportTypeCards.tsx`, new `lib/report-query.ts` | ~640 | S2, S7b (`support-api` error status) |

**Order.** These lanes can run in parallel; every slice merges into `redesign/906`, then QA runs on the review app (owner rule 5).

| Lane | Slices in order |
|---|---|
| 1 | S1, then S5a |
| 2 | S2, then S3 |
| 3 | S4a, then S4b, then S6 |
| 4 | S5b (needs S4a) |
| 5 | S7a, then S7b, then S8 |

S2 must land before S3, S4a, S6, S7a and S8. Merge in this order: S1, S2, S4a, S3, S4b, S5a, S5b, S6, S7a, S7b, S8.

- Each slice runs `bun run typecheck`, `bun run lint`, `bun test` (jest) for its files and `bun run build`. The frontend repo has no CI, so these local gates are the merge proof.
- Each slice adds a small jsdom test for 320 px overflow and asserts the v5.2 control roles from D8b 4b where its board names them: radio and edge `--action-pressed` on S6 and S8, focus `--focus` on S4a and S7b, limit bar `--action-pressed` on S5b.

---

## (e) Acceptance mapping

Trello checklist item, then the card's Mandatory QA AC, then slices, then blockers.

| # | Checklist item / QA AC | Slices | Backend card the full board outcome needs |
|---|---|---|---|
| 1 | Manage profile, settings, payment methods, notifications and tickets on the v5.2 screens on phone and desktop. QA AC1: edit, read and clear, create, reply, attach, close, report; states at 390 x 844 and 1440 x 900. | S3 to S8 | Frontend outcomes are not blocked. Specific board states need backend work: **919** (held-withdrawal "Taking longer than usual" row; the server sends no held state), **915** (support-reply notification `actionUrl` is `/support/tickets/{id}`, so it does not open the ticket; see the frontend alias option in (g)), **923** (the "reply below and it reopens" promise on resolved tickets), **924** (create idempotent and returning the id; until then "Send again" may duplicate, and the interim copy ships as drawn), **925** (Unread lists every unread item; until then it filters the loaded page). D-B4 notification copy with amounts has no card (owner build note); rows show server text as sent. |
| 2 | The Account area links to earnings, security, verification, notifications and support; moderator and admin entries only for those roles. QA AC2. | 968 (hub, staff group), S2 (pills), S3 (Verification and security), S4a (the preferences target) | None. Earnings is not on Account per the board; see (g). |
| 3 | Change password shows "Password requirements" with a met count and the same 12-character rules. QA AC3: 11 characters refused, a compliant 12 accepted, wording equal to sign-up and reset. | S1, S5a | **Blocked: 917 and 964. Stays unticked.** The frontend refuses 11 characters on the client, but the server still accepts 8 on change (`users.rs:144`) and checks none of the five rules. D8b 4b requires "the server answers 400 too". |
| 4 | Existing settings tab links keep working; tickets, attachments and reports stay visible to their owner only. QA AC4. | S4a (tab normalisation, all inbound `?tab=` links, `#proof-of-residence`), S7b, S8 | Not blocked: the server already refuses non-owners (support.rs `get_ticket` and `download_ticket_attachment`). **914** is hardening (403 instead of 404 reveals that an id exists); the frontend hides it behind one "Ticket not found". |
| 5 | Current, proposed and implemented captures for every changed page and state; 2FA and KYC unchanged. QA AC5: WCAG 2.2 AA, 320 px, 200% zoom, reduced motion. | all slices plus QA | None. Captures follow 965's register format. 2FA (904) and KYC panels are rendered unchanged; the 904 test file is left as it is. |

Other backend cards named in the owner rules:
- 920 is the pattern for 924. No D screen moves money.
- 918 is the fee default. The 1.20 fee appears only in server notification text.
- 911 and 910 concern offer fees and trade creation. No D screen uses them.
- 963, 953 and 877 cover dispute outcomes and the trade menu (D0 note). Group A owns that screen.
- No group D acceptance criterion depends on any of these.
- 913 (report priority) has no UI: the page promises no priority.
- 926 (blocked flag) is shown on group C's public profile. D's Security list uses today's data.

---

## (f) Risks

1. **Ticket ownership leak in the UI today.** `[id]` prints the server's 403 text, which confirms someone else's ticket exists. S7b must keep the HTTP status: `getTicket` currently throws a plain `Error`. Map 403, 404 and malformed-id 400 or 422 to the same not-found view, never echo server text, and offer no retry. Test both 403 and 404 (D8b section 4). Only 914 closes the gap at the API.
2. **Attachments may not render today.**
   - `baseUrl + "/api/v1/…"` probably doubles the prefix.
   - A cross-site review app may not send the `qic_access` cookie.
   - S7b switches to `fetchAuthenticatedBlob`, which sends the bearer token and leaves authorization unchanged.
   - QA must prove three things on the review app: the owner sees the image and PDF, a second customer gets the not-found view, and a direct download URL answers 403 to the second customer.
3. **Roles.**
   - The staff entries come from 968's `visibleStaffEntries` (moderator, admin, investor). A 968 D0 staff-menu follow-up is still coming and will touch `AccountHub.tsx` and `config/routes.ts`. S4a edits routes.ts, so whichever lands second rebases.
   - The resolver must keep `/moderator/support-tickets` for staff and use `/my-tickets/{id}` for customers (D8b section 1). It needs the role as an input.
4. **Settings tab URLs.**
   - The valid set becomes general, preferences, security, payment-methods and verification.
   - `?tab=general` loses the currency and email alerts. Every link that means "alerts" must move to `?tab=preferences`: the hub row, Jump and the bell's Alert settings (S4a and S6). The e2e tests `ticket-405` and `ticket-nav-affiliate-active` need updating.
   - Unknown values and the legacy `?tab=profile` must fall back to general (today they render empty).
   - `#proof-of-residence` must still scroll, because KYC content is untouched.
   - The phone and desktop layouts share one URL per section.
5. **Undo after delete.** The API deletes at once. S6 hides the row and sends DELETE after 6 s, or on unmount or navigation. A closed tab inside 6 s means no delete, which fails safe. Mark-all-read is unaffected.
6. **System alerts "Always on".** Today System has a switch, so some accounts may hold `system=false` on the server, and the row would claim what the server does not hold. The coordinator decides between re-enabling it with one PUT on load (a client write to the user's data) or a backend default. I recommend asking rather than a silent write.
7. **Global viewport change.** `interactive-widget=resizes-content` (S2) changes keyboard behaviour on every page, including 959, 958 and 957 screens. It needs a regression look on the trade room composer.
8. **The 320 px scroll on /profile has no known cause yet.** Candidates are the identity block, the page's `lg:px-24` grid, or 968's legal nav. If the cause is in `AccountHub.tsx`, S3 needs a small edit to a 968 file.
9. **Excluded flows.** `TwoFactorSection` and below, the setup wizard and `KYCVerificationContent` (except one link) must be byte-unchanged. The reviewer diffs them.
10. **Password rules.** The client rule is stricter than the server's, so a change accepted by the server could have been blocked earlier by the UI. That is acceptable. S1 must use sign-up's symbol class so 955 and 956 agree.
11. **Merge conflicts.** `970-v52-replicate` edits `notifications/page.tsx`, and S6 rewrites it, so rebase after 970 merges. Deleting the profile components in S3 touches tests written by 969 and 818.
12. **FlipDeals.** `/profile?tab=resells` must keep working for 958's resell sidebar link.
13. **CH6 fiat estimate.** It needs the USDT marketplace rate through `use-usdt-to-fiat`, under the USDT-canonical rule. When the rate fails, show USDT only and never a fabricated number.
14. **Signed-out /report.** The round trip relies on 935's `?redirect` handling and 955's sign-in. The `trade` and `user` parameters must survive.

---

## (g) Card-versus-board drift for the coordinator

The board wins in every case below.

1. **Earnings on Account.** The card says the hub covers "earnings (Reseller, Affiliate, Loyalty)". D1a notes 2 and 9 say Account shows no money or affiliate figures; Earnings is a dock destination (group B). Build per the board.
2. **`settings/notifications` route.** The card lists it, but it does not exist. The board moves in-app alerts to `/settings?tab=preferences` and removes the notifications page's settings tab.
3. **Settings scope.** The card says "general, payment and ordinary tabs". The board adds a fifth section (preferences) and redesigns the Security card around the unchanged 2FA panel: password entry, devices, blocked users, close account, and two removed cards.
4. **Attachments on new tickets.** The card says "attach files to … support tickets". The board attaches files on replies only; the new-ticket sheet has three fields.
5. **AC3 server enforcement.** The board (D8b 4b) also requires the server to answer 400, which needs 917 and 964, so the item stays unticked.
6. **Hosted canvases are stale.** The card's Related section says the hosted canvas links were republished in place and show v5.2. CHANGES.md says they were NOT republished and still show v5. Build only from the v5.2 sources and PDFs.
7. **Hub preferences target (968 drift).** 968's hub row and Jump point "Preferences and alerts" at `?tab=general`; D1a note 5 says `?tab=preferences`. Fixed in S4a.
8. **Board-versus-board, for Marcello.** D's copy of the password component (d.css `.a-req-n`) lacks E's success colour and icon (`.is-all`) and the `.is-wide` row, which auth.css has. D notes call it "group E's component", so S1 builds E's version. Confirm.
9. **Possible board limitation.** D8b says the support-reply link "stays wrong … whatever the frontend does" until 915. A frontend alias from `/support/tickets/{id}` to `/my-tickets/{id}` in `lib/notification-route.ts`, the same way 936 remapped `/account/kyc`, would fix it now without a backend change. The coordinator decides; the default is to build per the board, without the alias.
10. **"Read and clear notifications."** There is no clear-all on the board, only "Mark all read" and per-item Delete with Undo.
11. **Copy and links the server must change.** These are recorded as impossible for the frontend: the D-B4 notification wording, the held row (919) and the reopen promise (923).

---

## (h) Files other page-group cards also touch

**Shared password component (955).** S1 owns these files:
- `lib/password-rules.ts`
- `components/ui/password-requirements.tsx`
- `lib/constants.ts`

Whichever of 955 and 956 lands first builds them, and the other imports them. 955 then wires signup and reset. E's `e2e/tests/system/eml003-password-requirements.test.ts` still expects "At least 11 characters" and belongs to 955.

**Other overlaps**

| Card | Overlap |
|---|---|
| 957 (C) | `app/(main)/profile/[userId]/page.tsx` (`?view=public`, edited by S3); `PublicProfileView.tsx` and `ProfileReviews.tsx` (left to 957, which should adopt S3's `ReputationBlock` and `ReviewsCard` for C9's "one reputation block"); `SectionPills` (C uses pills); the report-user modal stays in C and links `/report?user=`. |
| 958 (B) | `SectionPills`, the offline banner and the toast Undo from S2; `/profile?tab=resells` (FlipDeals); the Security card's withdrawal wording must match B's `StepUpPrompt`; the withdrawal "Get help with this withdrawal" opens the support chat. |
| 959 (A) | `/report?trade=<id>` entry and the "Get help" chat from the room menu; the notification resolver's `/trade/{id}`; the global viewport change; the offline banner. |
| 955 (E) | Password component; sign out lands on /login with "You have been signed out"; the `/login?redirect=/report` round trip. |
| 968 | `AccountHub.tsx` (S3 only if the 320 px fix lands there), `config/routes.ts` (S4a), `NotificationsGlass.tsx` and `NotificationsDropdown.tsx` (S6); the D0 staff-menu follow-up is pending. |
| 969 | `ui/toast.tsx` and `store/ui-store.ts` (S2 adds an action); `ui/dialog.tsx` used as is (solid material for removing a payment method). |
| 970 | `notifications/page.tsx` (its follow-up restyle conflicts with S6); status-chip tones and the radio group are used, not edited. |
| 966 | Runs after 956 and must skip every file S3 to S8 rebuild. S3 deletes six profile components that sit on 966's list. |
| 965 | Screenshot register for the AC5 captures. |

---

## (i) Honest size estimate

- **XL.** Eleven PRs and about 8,100 product lines added or changed, the largest about 1,150 lines, all under the 1,500 cap. On top of that come about 1,700 deleted lines (profile sections and the slimmed hook) and about 5,500 to 6,000 test lines.
- **Rough effort:**
  - Serial: 14 to 18 builder-days.
  - With three to four parallel lanes after S1 and S2: about 6 working days.
  - QA and AC5 captures: 2 to 3 more days (about 70 states × 390 and 1440, plus 320 px, 200% and reduced-motion checks).
- **Biggest uncertainties:**
  - the 320 px root cause
  - whether attachments work through the review app's cross-site setup
  - the System-alerts decision
  - whether 955 lands S1 first
  - conflicts from the pending 968 and 970 follow-ups
