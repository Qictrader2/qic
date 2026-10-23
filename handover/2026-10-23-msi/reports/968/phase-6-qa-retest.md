# 968 Phase 6 re-test: consent banner under open layers

Card 968 (Trello xcgVXhJY), "Customer redesign 906-0c". Run on 23 Oct 2026, 05:13 to 05:53 UTC. Result: **FAIL, one finding**. The original finding is fixed, but one 968 layer, the phone staff menu, still opens under the banner. The card went QA Failed > QA / Testing > QA Failed.

## 1. Build and accounts

- Review app https://qictrader-fe-pr-303.herokuapp.com on the staging backend. The Heroku API shows release v35, "Deploy ca8fbe25" (build `ca8fbe251a9c253385a4d1c9ac88511ff91e576b` succeeded 04:27Z, released 04:30Z). It was still current when testing started.
- ca8fbe25 is the squash of Frontend #337 ("968: Consent banner steps aside under every open layer") and sits at the tip of `origin/redesign/906`. Since the last QA build (1679d151) three commits landed: #335 (958), #336 (970) and #337. Of those, only #337 touches consent or the bottom-bar hook.
- The qic-qa-signin agent ran `--check` on all three sessions. buyer-906 failed its check with HTTP 500, so the agent signed it in again (200, no email code). No account state was changed.

  | Session | Account | App role |
  | --- | --- | --- |
  | buyer-906 | marcello@unlimited-beats.com | user |
  | seller-906 | marcellohaupt@gmail.com | super_admin |
  | third-906 | starbirdbeats@gmail.com | user (not needed) |

- Browser: Puppeteer with Chrome for Testing, headless. Every run used a new temporary profile in the scratchpad, which was deleted in `finally`. One Chrome ran at a time. Sessions came only from plantSession, and its block on refresh and logout explains the expected "Token refresh failed" console lines. Before each layer case I removed the `qic_consent` cookie and localStorage key, and each case confirmed that nothing was stored at its start. After the runs, both sessions still returned 200 through qa-curl.sh.
- Evidence: `/Users/starbird/.config/qic/qa-artifacts/968-retest-20261023T051541Z/`. Result JSON: s10-consent-390/768/1440(-bell-only), s11-focus-390/1440, s12-baseline, s13-trade-pad, s14-staff-layer, s15-proof, s1-phone, s2-large, s5-staff, s16/s17 clips. Also there: screenshots, the Jev request and response, and trello-comment.txt.

## 2. Finding (fails the card)

1. **AC2, staff menu under the banner at 390.** Setup: 390x844, first visit, cookie choice not made, signed in as marcellohaupt@gmail.com (super_admin). Tapping the avatar opens the staff menu panel `profile-menu-panel` (fixed, top 56 to bottom 760, z-50).
   - The consent banner is not covered (no `data-covered`, opacity 1, z-50) and draws on top of the panel from 495 to 760.
   - Hit tests on 6 of the 14 entries (Security & 2FA, Payment Methods, Reseller Profile, My Tickets, Help & Support, Sign Out) land on the banner. Scrolling the panel to its end still leaves 5 of them, Sign Out included, under the banner (clip frames).
   - The banner itself stays usable. Once it is answered, the menu works (s5 regression).
   - At 768 and 1440 the panel does not overlap the banner, which stays usable there.
   - Cause: `use-bottom-bar-state.ts:6` `COVERING_OVERLAY_SELECTOR` matches only `[role="dialog"]`, `[role="alertdialog"]`, `[aria-modal="true"]` and `[data-qt-overlay]`. The panel at `UserProfileDropdown.tsx:172` has none of them. The phone placement of this menu is 968's own work (d3cd77a0, staff-menu-d0).
   - Expected: the banner steps aside under this menu as it does under every other layer, and all 14 entries are reachable while the menu is open.
   - Proof: 968-AC2-5-page.png, 968-AC2-6-working.png, 968-AC2-7-evidence.png, 968-AC2-8-source.png, 968-AC2-retest-staff-menu-390.webm.
   - Severity: low. It affects only staff accounts, only on a first visit, and only at phone width, and the banner can still be answered.

## 3. Re-test of the original finding (passed)

Each case followed the same steps: banner showing, layer opened, banner checked, layer used, layer closed, banner checked again, banner answered, reload. For "Banner while open", "not reachable" means hit tests at all three button centres missed the banner, focus() on each button failed, and 30 Tab presses never reached it.

| Layer | Widths | Banner while open | Layer usable | After close | Answer stored and kept after reload |
| --- | --- | --- | --- | --- | --- |
| Chat with support (Account) | 390, 1440 (3 passes each) | covered, inert, aria-hidden, opacity 0, hidden, z 39, not reachable | all 6 options fully in view and hit-testable, "Something Else" replied | same rect, buttons hit-testable, focus on Chat with support | Reject all (deny all), Accept all (grant all), Customise (Analytics on, Save) |
| Jump | 768, 1440 | covered, not reachable | "sup" gives G9 results | focus on the Jump button | Reject (768), Accept (1440) |
| Chat from Jump | 768 | covered, inert (aria-hidden missing once, inert still applies) | 6 options, reply | focus on the Jump button | Customise |
| Cookie preferences (Account) | 390, 768, 1440 | covered | dialog usable | focus on the Cookie preferences row | Reject |
| Notifications popover | 768, 1440 | covered | usable | back in place | Accept |
| Export trades dialog | 390, 768, 1440 | covered | usable | focus on the Export button | Customise |
| Trade Filters sheet | 390, 768 (not at 1440, inline there) | covered | usable | focus on the Filters button | Accept |
| Daily notice (first load) | 390, 768, 1440 | covered | n/a | n/a | n/a |
| Staff menu (super_admin) | 390 | **not covered (finding 1)** | 6 entries hidden | n/a | Reject |
| Staff menu (super_admin) | 1440 | not covered, no overlap | usable, Sign Out hit-testable | n/a | Customise |

- The Customise dialog also covers the banner while it is open, and the banner returns after it closes.
- At 390 the banner bottom is 760 and the dock top is 772 (12 px) on /wallet, /marketplace and /earnings. The toast "Referral link copied" also has its bottom at 760, 12 px above the dock. With the banner showing, the toast briefly sits over the banner's Accept all button. The previous QA accepted this shared position.

## 4. Regression on v35 (passed, same scripts as the last run)

- **Dock at 390.** Order is Wallet, Market, Trades, Earnings, Account, with exactly one aria-current per tab. /earnings returns 200 with the title "Earnings | QicTrader". None of the 10 deeper pages shows the dock, and each has a back header. Back stays on the site. Browser Back keeps `?status=active` and the trade id.
- **Hard load of /wallet at 390.** Padding is 88 px from the first sample (15 ms). The dock mounts at 275 ms. The only layout shift is 0.334, from the footer when wallet content loads, as before. The content ends at 756, above the dock at 772.
- **Island at 768 and 1440.** Five items with the correct active item on 10 routes, 0 header balances, no support bubble and no horizontal overflow.
- **Jump.** Click, Ctrl K and / all open it. Staff words return nothing for a plain user. "settings" + Enter opens /profile.
- **Chat focus return.**
  - Sheet at 390: modal, top 112, scrim, dock covered and inert. Escape returns focus to the Account row.
  - Popover at 768 and 1440: 400x600, non-modal. The bot replies. Escape returns focus to the row, or to Jump when the chat was opened from Jump.
- **Staff menu (super_admin).**
  - Placement: right of the capsule at 1440 (1364 vs 1356) and at 768 (700 vs 692), and in the phone header at 390 and 390x667.
  - Entries: Moderator Dashboard and Admin Panel are listed.
  - Sign Out: in view and above the dock at 390x667 (582 vs 595) with the banner answered.
  - Staff pages: /moderator and /admin have no dock.

## 5. Recorded, not judged

- **Focus after Customise.** On a signed-in customer page, closing the banner's own Customise dialog (Escape or X, opened by mouse or keyboard) leaves focus on `<body>` in 16 of 16 tries. The next Tab goes to the top of the page. Two comparisons:
  - Logged out on the review app, focus does return to Customise.
  - Logged out on staging main (pre-968), focus is also lost.

  So this is not a 968 regression. A likely cause is that the banner is inert at the moment Radix restores focus. The fix report's reviewer left this unverified in Chrome on the customer surface. A small follow-up is worth considering.
- **/trade/<id> padding at 390.** The page now has 76 px of bottom padding, versus 0 px in the last run. The padding is inline style on body, set alongside the trade room's own `room-action-bar`, not by the dock, and it is absent once that bar unmounts. This is not 968 code; the trade room belongs to the 958/959 cards.
- On a first visit, the banner covers the trade room's action bar on no-dock pages. This is unchanged and outside this re-test.

## 6. Not run, money and state

- No money moved, and none was needed.
- No role, KYC or 2FA changes, so nothing needs restoring.
- Consent choices were stored only inside the temporary browser profiles, which are now deleted.
- No real iOS or Android device was used.
- `qic_record.ts` accepts only staging.qictrader.com, so the clips are Puppeteer screencasts. Both decode cleanly, and I checked the frame samples by eye.
- A first staff-menu clip ran in a profile where the buyer had already answered the banner, so it did not show the bug. I deleted it and recorded the clip again in its own profile (s17).

## 7. Jev gate

One call, made after inspecting the screenshots. Result: verdict fail, confidence 0.98; severity 0.89; product decision 0.24; reproducible 0.86. Model jev-1.14.0. That permits the failing flow, and my own evidence identifies the failing case. Files: jev-request.json and jev-response.json.

## 8. Trello writes (each one read back)

- Moved QA Failed > QA / Testing at the start, then > QA Failed. idList 69d535fe0f2d642891e3ae1d confirmed by GET.
- Six attachments. Each byte count matches the local file on GET:

  | File | Bytes |
  | --- | --- |
  | 968-AC2-5-page.png | 281182 |
  | 968-AC2-6-working.png | 241662 |
  | 968-AC2-7-evidence.png | 147453 |
  | 968-AC2-8-source.png | 97426 |
  | 968-AC2-retest-staff-menu-390.webm | 285607 |
  | 968-AC2-retest-chat-sheet-390.webm | 944700 |

- One comment, 6adaf60be7162b2769c5def1, 3237 characters. It is written in the first person with no em dashes, and its text is in trello-comment.txt.
- The checklist is untouched.
- `acceptance-map.md` has a new "Phase 6 re-test" section.
- SendUserFile was not available to me, so the files are only on the card and in the evidence folder.
