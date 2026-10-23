# 968 Phase 6 re-test 2: menus over the consent banner, focus return

Card 968 (Trello xcgVXhJY), "Customer redesign 906-0c". Run on 23 Oct 2026, 08:13 to 09:05 UTC. Result: **PASS**. The card went QA Failed > QA / Testing > QA Approved / Ready For Deployment (the owner authorized that move in the request).

## 1. Build and accounts

- Review app https://qictrader-fe-pr-303.herokuapp.com on the staging backend. I polled the Heroku API until release v41, "Deploy 3e37fe68", was live. Build e56c88db (source `3e37fe68c32651c3d13e34017621f53c4f8b2d1b`) succeeded at 08:16:45Z and was released at 08:16:31Z.
- 3e37fe68 is the squash of Frontend #343 (968-qafix2) and the tip of `origin/redesign/906`. A deployed chunk (`9f83fa7a824d7acf.js`) contains the fix's `data-qt-focus-landing` marker.
- The qic-qa-signin agent ran `--check` on both sessions. Both returned 200, so it did not sign either one in again and changed no account state.

  | Session | Account | App role | KYC |
  | --- | --- | --- | --- |
  | seller-906 | marcellohaupt@gmail.com | super_admin | approved, level 3 |
  | buyer-906 | marcello@unlimited-beats.com | user | approved, level 2 |

  The agent checked usernames only (marcello_haupt_qic and unlimited_beats). The emails are the ones on record for those accounts.

- Browser: Puppeteer with Chrome for Testing, headless. Each script used its own temporary profile in the scratchpad and deleted it in `finally`. One Chrome ran at a time, and no profile directories or Chrome processes were left afterwards. Sessions came only from plantSession. Before each case I removed the `qic_consent` cookie and localStorage key, and every case confirmed that nothing was stored at its start. Both sessions still answered 200 on `/api/v1/users/me` after the runs.
- Evidence is in `/Users/starbird/.config/qic/qa-artifacts/968-retest2-20261023T081832Z/`:
  - JSON results: s20-menus-390/768/1440, s21-focus-390/1440 (plus -390-B for the rerun), s1-phone, s2-large, s5-staff, s10-consent-390/768/1440, s22-clip.
  - Screenshots for every case, the Jev requests and responses, `trello-comment.txt`, and `trello/` (the attached set).
- Scripts are in `/private/tmp/claude-501/-Users-starbird-Workspace-Code-qic/853e4c8b-e6be-48cd-8930-893197f52192/scratchpad/968rt2/`.

## 2. Last run's finding, re-tested (passed)

Each case followed the same steps: banner showing, layer opened, entries hit-tested, banner checked, 30 Tab presses, layer re-checked, Escape, banner checked again, banner answered, reload.

- Every entry was scrolled into view inside the panel and then hit-tested at its centre.
- "Unreachable" means hit tests on Reject all, Accept all and Customise all missed, focus() failed on all three, and 0 of the 30 Tab stops landed in the banner.

| Layer | Widths | Entries hit-testable | Banner while open | After close | Answer, stored, kept after reload |
| --- | --- | --- | --- | --- | --- |
| Staff menu, super_admin (`profile-menu-panel`, data-qt-overlay="menu", non-modal) | 390x844 | 14 of 14, Sign Out included, 0 on the banner. Panel 56 to 760, dock visible at 772 | covered, inert, aria-hidden, opacity 0, hidden, z 39, unreachable | same rect, all 3 buttons hit-testable and focusable | Reject all |
| Staff menu | 768, 1440 | 14 of 14 | covered, unreachable | back, working | Reject (768), Customise + Save (1440) |
| Trade room more | 390 sheet; 768 and 1440 Radix menu | 5 of 5 (390), 4 of 4 (768, 1440) | covered, unreachable | back; focus on the more button | Reject |
| Marketplace offer row menu | 390, 768, 1440 | 2 of 2 | covered, unreachable | back; focus on the trigger | Customise + Save |
| Notifications popover | 768, 1440 | 8 of 8 | covered, unreachable | back | Accept |
| Notifications at 390 | 390 | n/a: the bell is a link to /notifications | n/a | n/a | n/a |
| Wallet More menu | all | **not run live**: no page mounts it in this build (see 4.1) | n/a | n/a | n/a |

The 390 clip `968-retest2-staff-menu-and-focus-390.webm` shows the whole sequence. On a first visit as super_admin, the menu opens over the banner and scrolls to Sign Out. Escape brings the banner back. Opening Customise from the keyboard and pressing Escape returns focus to Customise, and the last frame shows the focus ring there.

## 3. Focus (passed)

- **Customise closed.** Tested on /wallet, /profile and /marketplace, opened by mouse and by keyboard, closed with Escape and with the X, plus 4 quick open and close cycles on one load. Focus settled on Customise in 16 of 16 tries at 390 and 16 of 16 at 1440, in each case within 1.2 s. The next Tab reaches Reject all. Last run, focus landed on `<body>` in 16 of 16 tries.
- **After a choice.**
  - Where a page element had focus before focus entered the banner, focus goes back to that element. Cases: Accept (keyboard and mouse), Reject (keyboard), Save from Customise (keyboard and mouse).
  - Where nothing had focus, focus lands on `<main>` with `tabindex=-1` and `data-qt-focus-landing`. Cases: Accept (mouse), Reject (mouse and keyboard), Save (mouse).
  - The next Tab moves on, and `<main>` drops both attributes.
  - Result: 9 of 9 at 390 and 9 of 9 at 1440.
  - In the first 390 pass, my script found no focus target on /profile, so 2 cases ran with nothing focused. They landed on main, which is correct for that state. I reran them on /marketplace and both passed.

## 4. Recorded, not judged

1. **Wallet More menu.**
   - At 3e37fe68, a source search finds no importer of `components/features/wallet/MoreMenu.tsx` apart from its tests. /wallet and /wallet/manage/usdt at 390 and 1440 render no `qt-wallet-more`, so no user can open it yet.
   - Instead, I ran the fix's suites at 3e37fe68 in a temporary worktree (since removed): `consent-menu-layers.test.tsx` and `consent-customise-focus-customer.test.tsx`. 24 of 24 pass, including "wallet More menu at 768+ ... covers the banner while open, leaves the dock, and gives the banner back on Escape" and the phone sheet case. Log: `968rt2/jest-menus.log`.
   - Once a 958 slice mounts it, the live check is worth repeating.
2. **Known, not 968's.** The /my-offers row menu (MyOfferCard, owned by 957-S10) at 390 opens with the banner not covered: opacity 1 and not inert. On the first card the menu sits above the banner. At 768 and 1440 the actions are inline buttons, with no menu.
3. The marketplace offer row menu at 390 opens flush with the left edge of the screen (screenshot -5). That is OfferRowMenu placement, not 968 code.
4. The staff menu and the notifications popover stay open while Tab moves focus out of them, so focus after Escape is wherever Tab left it. Opened and closed directly, focus returns to "User menu" (s5).

## 5. Regression on v41 (passed, same scripts and values as the last run)

- **Chat layer.** The sheet at 390 and the popover at 1440 cover the banner (3 passes each, answered in all three ways). All 6 options are in view and clear, the bot replies, and focus returns to the Chat with support row. At 768, chat opened from Jump behaves the same.
- **Jump.** At 768 and 1440 it covers the banner and focus returns to Jump. Click, Ctrl K and / all open it, staff words return nothing for a plain user, and "settings" + Enter opens /profile.
- **Dock at 390.** Order is Wallet, Market, Trades, Earnings, Account, with one aria-current per tab. /earnings returns 200 with the title "Earnings | QicTrader". None of the 10 deeper pages shows the dock, and each has a back header. Back stays on the site, and browser Back keeps `?status=active` and the trade id.
- **Island at 768 and 1440.** Five items with the correct active item on 10 routes, 0 header balances, no floating support button and no horizontal overflow.
- **Gaps at 390.** Banner bottom 760 and dock top 772 (12 px) on /wallet, /marketplace and /earnings. The toast "Referral link copied" also has its bottom at 760, 12 px above the dock.
- **Staff menu.** Right of the capsule at 1440 (1364 vs 1356) and 768 (700 vs 692), in the phone header at 390 and 390x667. Sign Out is hit-testable at all four sizes and above the dock at 390 (759 vs 772) and 390x667 (582 vs 595). /moderator and /admin have no dock.
- **/wallet hard load at 390.** Padding is 88 px from the first sample (11 ms), the dock mounts at 275 ms, content ends at 756 above the dock at 772, and the only layout shift is 0.334, from the footer as before.
- Other surfaces: the Cookie preferences sheet, Export trades and Filters still cover the banner as before. At 1440, Filters is inline, so it is not a layer.

## 6. Money and state

- No money moved and none was needed.
- No role, KYC or 2FA changes, so nothing needs restoring.
- Consent choices lived only in the deleted temporary profiles.
- `qic_record.ts` accepts only staging.qictrader.com, so the clip is a Puppeteer screencast. I checked its frames by eye.
- No real iOS or Android device was used.

## 7. Jev gate

- **First call:** verdict fail, confidence 0.97, severity 1.1, product decision 0.14, reproducible 0.73. This disagreed with my evidence, which has no failing case. My packet listed the wallet More menu as "could not be exercised live" without saying that no page mounts it, and it had no unit-test run.
- **The fix:** I ran the unit tests and corrected the packet.
- **Retry:** pass, confidence 0.92 (pass 0.95), severity 1.1, product decision 0.10, reproducible 0.88. Model jev-1.14.0.
- That permits the passing flow, and my own evidence covers every criterion.
- **Files:** jev-request-1.json, jev-response-1.json, jev-request.json, jev-response.json.
- **Key loading:** the retry first returned HTTP 403 because my shell missed the skill's last line for loading the key. The corrected call is the one recorded above.

## 8. Trello writes (each one read back)

- Moved to QA / Testing at the start, then to QA Approved / Ready For Deployment. A GET confirms idList 69d535d2149a252f13a4cb18.
- Nine attachments. Each byte count matches the local file on GET:

  | File | Bytes |
  | --- | --- |
  | 968-retest2-1-staff-menu-390-open.png | 238684 |
  | 968-retest2-2-staff-menu-390-closed.png | 213281 |
  | 968-retest2-3-staff-menu-1440-open.png | 256512 |
  | 968-retest2-4-room-more-390.png | 383731 |
  | 968-retest2-5-offer-menu-390.png | 264309 |
  | 968-retest2-6-notifications-1440.png | 325941 |
  | 968-retest2-7-customise-focus-390.png (last clip frame) | 198188 |
  | 968-retest2-8-known-my-offers-390.png | 253187 |
  | 968-retest2-staff-menu-and-focus-390.webm | 888738 |

- One comment, 6adb2276f0a5f753578e8525, 2829 characters. It is written in the first person with no em dashes, and its text is in trello-comment.txt.
- The checklist is untouched.
- `acceptance-map.md` has a new "Phase 6 re-test 2" section.
