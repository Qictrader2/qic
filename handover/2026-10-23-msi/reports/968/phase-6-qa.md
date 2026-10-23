# 968 Phase 6: hands-on QA on the review app

Card 968 (Trello xcgVXhJY), "Customer redesign 906-0c: phone bottom bar, one balance and clear overlays". Run 22 to 23 Oct 2026 (17:37 to 03:34 UTC), resumed once after a session restart. Result: **FAIL, one finding**. Card moved to QA Failed.

## 1. Build and accounts

- Review app https://qictrader-fe-pr-303.herokuapp.com on the staging backend (health 200). Releases from the Heroku API: v30 `ce1f24bc90e776a8758fb2c40f6fff991e663a69`, then v31 `bab070d9` (19:00Z), then v32 `1679d151` (21:01Z, still current at 03:30Z). Runs are mapped to releases in `release-receipt.txt`.
- All three 968 merges are in every tested release: #305 fe023ad2, #307 9bbd3d88, #310 d3cd77a0. `git diff ce1f24bc 1679d151` over navigation, support, routes.ts, globals.css, layout.tsx, use-bottom-bar-state.ts, ui/toast.tsx and consent is empty.
- The qic-qa-signin agent handed over the sessions and changed no account state. All three accounts have 2FA off.

  | Account | Role |
  | --- | --- |
  | marcello@unlimited-beats.com | plain user |
  | starbirdbeats@gmail.com | plain user |
  | marcellohaupt@gmail.com | super_admin |

- Browser: Puppeteer with Chrome for Testing. Each run used a temporary profile in the scratchpad, deleted afterwards, and only one Chrome ran at a time. Sessions came only from plantSession, and the qa-curl.sh API reads were GET only.
- Evidence: `/Users/starbird/.config/qic/qa-artifacts/968-20261022T174013Z/`. It holds about 70 screenshots, the result JSON for each run (s1 to s8), three clips, `release-receipt.txt`, the Jev requests and responses, and `trello-comment.txt`.

## 2. Finding (fails the card)

1. **AC2, overlays.** On a phone (390x844), on a first visit before the cookie choice is made, Account > Chat with support opens the chat sheet. The consent banner then draws on top of it, because the banner is `ConsentBanner.tsx:82` `fixed z-50` and the sheet is `SupportChatPanel.tsx:45` `z-[41]`.
   - The banner hides 3 of the 6 chat options.
   - Its Reject all, Accept all and Customise buttons cannot be used while the chat is open. The modal sheet makes everything outside it inert, so a hit test at their centres returns `chatbot-log`, the banner stays and no consent cookie is set.
   - Closing the chat, answering the banner and reopening shows all 6 options.
   - At 1440 the banner (z-50) and the non-modal panel (z-50) both stay usable, and Reject all works. The panel does slightly overlap the banner's Accept all button.
   - Board D1a frame 3 draws the phone sheet with nothing on top of it. Proof files: 968-AC2-1-page.png, -2-working.png, -3-evidence.png, -4-source.png. Reproduced twice on v32.
   - Suggested fix (for the builder to decide): put the banner under the sheet's scrim while the sheet is open, or keep the banner usable above the sheet.

## 3. Cases that passed

| Case | Result |
| --- | --- |
| Dock at 390 | Wallet, Market, Trades, Earnings, Account in that order. Each tap opens its route with exactly one aria-current and the matching phone title. |
| /earnings | A real page now: 200, stays on /earnings?ref=qa, title "Earnings \| QicTrader", Earnings lit. |
| Island at 768 and 1440 | The same five items, no dock. Correct active item on /wallet, /marketplace, /trade-history, /earnings, /profile, /settings, /offer/resell, /dashboard/affiliate, /trade/<id> and /notifications. |
| No dock at 390 | /offer/create, /offer/<id>, /offer/resell, /profile/<userId>, /trade/<id>, /settings, /notifications, /report, /my-tickets and /dashboard/affiliate/tiers. Each shows a back header and has 0 bottom padding. |
| Hard load of /wallet at 390 | Padding is 88 px from the first sample (34 ms) and never changes. The dock mounts at 491 ms. One layout shift of 0.334 at 1.3 s comes from the page footer when the wallet content loads, not from the dock. At the end of the page the content stops at 756, above the dock at 772. |
| Daily notice sheet | The daily server notice ("Transaction verification starts 30 October") opens on the first load of the day. The dock goes covered and inert while it is open and returns after Escape. |
| Back | The back control on a hard-loaded /settings, /notifications, /trade/<id> or /offer/<id>, with an off-site history entry before it, stays on the review app. Browser Back returns exactly to /trade-history?status=active and to /trade/<id>. |
| Jump palette (768 and 1440) | Opens by click, Ctrl K and /. "sup" shows the G9 rows in G9 order. Escape returns focus to the Jump button. "admin", "moderator" and "war room" find nothing for any account. "settings" + Enter opens Account (/profile), which is its first result by design. |
| Support chat | At 390, a modal glass sheet (top 112, scrim, focus to the title, dock covered and inert). At 768 and 1440, a non-modal 400x600 glass popover 24 px in from the right and bottom, with no scrim and a live island. Escape returns focus to the Account row, or to the Jump button when opened from Jump. A chatbot option got a reply. Pixel luminance: the sheet is 233 of 255 (pale, not the old dark box). |
| Toast and consent banner at 390 | Both sit 12 px above the dock (bottom 760, dock 772). |
| Tab focus at 700x560 | Across 5 dock pages and about 90 Tab stops each, no focused element landed behind the dock. scroll-padding-bottom is 84px. |
| Labels at 320 and 360 | No label is clipped. |
| Header | Home, the island, Jump and the bell only: 0 balance controls and no Support button. The Account hub shows no staff entries for anyone. |
| Staff menu (super_admin) | Right of the capsule at 1440 (capsule ends at 1356, menu at 1364) and at 768 (692 vs 700). The island is shifted 52 px. At 390 and 390x667 it sits in the phone header before the bell. It lists Moderator Dashboard, Admin Panel, the existing links and Sign Out. At 390x667 the panel scrolls (bottom 583, dock 595) and Sign Out is in view and above the dock. Moderator Dashboard opens /moderator. /admin and /moderator keep today's header and have no dock or island. |
| Plain users | No staff menu and no staff text at 390, 768 or 1440. |
| Real submits through the dock | Personal details bio save: PUT /users/profile 200, "Profile updated successfully", persisted after a reload, then restored with a second save (PUT 200). Report a problem (Bug Report): POST /support/ticket 200, "Report Submitted" plus a toast. |

## 4. Not run, not judged, and state changes

- **2FA and step-up.** No 968 surface led to a 2FA or step-up step for these accounts (2FA off). Security and sign-in opens /settings?tab=security with the 2FA section. Its panels are excluded per D0 (card 904).
- **Money.** No money moved. My safety rules do not allow fund movements on this host.
- **Real devices.** No real iOS or Android keyboard was used.
- **Recorded, not judged (958 and 956).** /wallet page content is 381 px wide at 320 and 360. /profile is 357 px at 320.
- **Not in 968's scope.** The 1440 /profile left column still shows the old wallet and trade cards (956).
- **State changes.**
  - The buyer's bio was set and then cleared. The API now returns `''`; before the test it was `null`, and both look the same in the app.
  - One staging support ticket ("QA 968 review app check, please ignore") was created by marcello@unlimited-beats.com.
  - No role, KYC or 2FA changes were made, so nothing needs restoring.

## 5. Jev gate

| Call | Verdict | Confidence | Reproducible | Other |
| --- | --- | --- | --- | --- |
| First | pass | 0.72 | 0.6 | Below the gate. The screenshots had not been inspected by eye then, because the Read hook was failing. |
| Retry | fail | 0.91 | 0.85 | Severity 0.95, product decision 0.22. Run after inspecting the screenshots and adding the finding. |

The retry permits the failing flow because my own evidence identifies the failing case. Files: jev-request.json, jev-response.json, jev-request-retry.json and jev-response-retry.json in the evidence folder.

## 6. Trello writes (each one read back)

- Moved Development Complete > QA / Testing at the start, then > QA Failed (idList 69d535fe0f2d642891e3ae1d confirmed).
- Seven attachments, each byte count confirmed by GET: 968-AC2-1-page.png, 968-AC2-2-working.png, 968-AC2-3-evidence.png, 968-AC2-4-source.png and three webm clips.
- One comment, 6adad5ab49504d2f95e0695f.
- The checklist is untouched, so AC2 is still ticked from the acceptance check.

## 7. Environment

- For part of the run, Read and Write PreToolUse hooks timed out ("host client may be unreachable"). I worked through Bash and checked by DOM and pixel measurement until Read came back, then inspected every image I attached.
- The proof-shot runner always pre-sets consent, so it cannot show this bug. The page proofs were annotated in-page by my own Puppeteer script; the evidence and source cards come from the runner.
- qic_record.ts accepts only staging.qictrader.com, so the clips are Puppeteer screencasts. Each decoded with no errors, and frame sampling shows the playback advancing.
- SendUserFile was not available to me.
