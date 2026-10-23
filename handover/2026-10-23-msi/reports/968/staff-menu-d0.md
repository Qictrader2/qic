**968 follow-up: staff keep their own menu (board D0)**

PR: https://github.com/Qictrader2/Frontend/pull/310 into redesign/906 (not merged)
Branch: 968-staff-menu-d0, head b79ee946 (commits ec182c7d, b311df3a, then merge of redesign/906 at dd59fb9e)

**What the boards draw**
- D0, Jump section: "Replaces the avatar menu's eleven links for customers; staff keep their menu." D0 Leaves: "The header avatar menu and hamburger for customers."
- D8b: "staff keep UserProfileDropdown"; MainHeader on public, auth and staff pages keeps NotificationsDropdown and UserProfileDropdown for staff.
- No board draws staff entries on the Account page or in Jump. Checked D0, D1a to D1c and G9 (lg-palette). No board draws where the staff menu sits inside the customer chrome either.
- Placement decided and then approved by the owner as built (22 Oct): right of the capsule at 768 and 1440; end of the phone header at 390, just before the bell; island shifted 52px for staff only.

**What changed**
- Staff menu on customer pages: StaffMenu.tsx (new), CustomerHeader, Capsule, Island, PhoneHeader, navigation/index. It renders today's UserProfileDropdown only when the user is signed in and visibleStaffEntries is non-empty.
- Staff group removed from AccountHub; staff rows removed from Jump (JumpPalette, jump-search); STAFF_ENTRIES trimmed to id, label and href (routes.ts).
- Role gating unchanged: moderator from useModeratorAuth, admin from useAdminAuth, War Room when isInvestor && !isAdmin.
- Panel fit on customer pages: UserProfileDropdown takes panelFit "viewport", which caps the height with --qt-overlay-bottom and scrolls. The default "content" keeps today's MainHeader classes. UserProfileDropdown.tsx is not in 968's ownership list in BUILD-SCHEDULE, and no other slice owns it.
- One visibleJumpTrigger (the strict one) together with JUMP_TRIGGER_SELECTOR now lives in support/return-focus.ts. JumpPalette imports it, and the inert retry loop retries while it returns null.
- "/earnings" added to CUSTOMER_SURFACE_RULES.paths (coordinator request for 958-S9). Customer-surface, bottom-bar-routes and phone-header tests updated.

**Card 968 (Trello xcgVXhJY)**
- Backup: .scratch/906-redesign/card-backups/968-20261022T115901.json
- Description: the staff sentence now follows D0 and D8b (own staff menu, no staff entries on the Account page or in Jump). QA coverage AC3 is updated to match. Added the line "Aligned with v5.2 boards on 22 Oct 2026 (owner decision, follow D0): D0, D8b".
- Checklist AC3 renamed to "...and staff keep their own staff menu, the only place staff entries appear; a plain customer never sees one." It is still complete.

**Tests**
- Navigation 17 suites with 613 tests; design 15 suites with 527 tests; navigation, design and 959-s8 after the merge: 33 suites with 1148 tests, all passed.
- Support chat and sign-out: 5 suites with 56 tests. Dialog-shell and 969 initial focus: 2 suites with 68 tests. All passed.
- New: staff-menu-role-matrix (50 tests: user, moderator, admin, super_admin, investor, investor plus admin, signed out, across the staff menu, Account hub and Jump at the band and phone surfaces), jump-trigger-helper (13 tests), staff-menu-panel-fit (3 tests).

**Gate**
GATE PASS (typecheck, lint-no-new-errors, build). Receipts: 968c-root/.scratch/gate/20261022T140840, after the merge of dd59fb9e; also 20261022T140145.

**Browser check**
Local production build with roles seeded through request interception; temp profile deleted and server stopped.
- Panel bottom above the dock: 583 vs 595 at 390x667, 760 vs 772 at 390x844.
- Sign Out reachable after scrolling.
- Admin sees Moderator Dashboard and Admin Panel; investor sees War Room only.
- A plain user sees no staff menu or staff text at 390, 768 and 1440, and Jump finds nothing for "admin" or "war room".
- /earnings shows the customer shell.
- Evidence: .scratch/968/evidence/staff-menu/ (panel-fit/results.json, staff-menu-matrix.json, side-by-side PNGs).

**Review**
opus55-high: APPROVE, with three LOW notes left open:
1. Jump passes a null opener to the support chat, and focus still returns correctly.
2. Escape in the dropdown does not refocus the avatar (today's behaviour).
3. useStaffMenuVisible is called twice per render.

**Environment fix**
The backend symlink suggested in SLICE-BRIEF made gate.sh run the whole backend section: 6131 "changed files" and two env-dependent cargo test failures. I removed the symlink in 968c-root and added a note to SLICE-BRIEF "Machine limits" to remove it before gating.

**Backend-blocked items**
None. Frontend only.
