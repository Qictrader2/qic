# 970 PHASE 6 QA (hands-on /qic-qa)

Card: https://trello.com/c/pY8CFeEI. Run 23 Oct 2026, 04:09Z to 05:15Z. Verdict: **PASS**.

## Environment and pins

- Review app https://qictrader-fe-pr-303.herokuapp.com on the staging backend (staging API /health 200).
- Heroku builds API: v34 = 28645d0c (built 03:40Z) at the start; v35 = ca8fbe25 (built 04:27Z, released 04:30Z) for the later runs. ca8fbe25 adds only 968's consent-banner change (#337: ConsentBanner.tsx, use-bottom-bar-state.ts, a consent block at the end of globals.css). No 970 shared control, token, route or theme file changed between the two.
- Spec: v5.2 boards (G0 and shared control boards), contradictions.md decisions, HANDOVER coordinator rulings (field error circle; button icons 18/20 px with the #327 per-control list).
- Sessions from the qic-qa-signin agent, used only through qa-curl.sh and plantSession: buyer-906 (marcello@unlimited-beats.com, user, KYC level 2, fresh sign-in 04:10Z), seller-906 (marcellohaupt@gmail.com, super_admin), staging seller (marcellohaupt@gmail.com) for the staff comparison. No account state changed, so nothing to restore.
- Browser: Chrome for Testing 143, headless, one temporary profile per run under the scratchpad, deleted at the end. The script waited whenever another agent's Chrome was running. A crash handler closes every tab first. All sessions still valid at the end.
- Evidence root: `/Users/starbird/.config/qic/qa-artifacts/970-20261023T0410Z/` (`data/*.json`, `shots/`, `970-markup-save-390.webm`, `frames/strip.png`, `jev/`). Scripts: `/private/tmp/claude-501/-Users-starbird-Workspace-Code-qic/853e4c8b-e6be-48cd-8930-893197f52192/scratchpad/qa970/`.

## Results by criterion

| AC | Result | Evidence |
|---|---|---|
| 1 Daylight, staff and public unchanged | PASS | `data/daylight.json`, `data/staff-public.json`, `data/staff-toggle.json` |
| 2 Controls and focus rings | PASS | `data/controls*.json`, `data/focus*.json`, `data/submits.json` |
| 3 Amounts, chips, states | PASS | `data/controls2.json`, `data/motion-states.json` |
| 4 Reduced motion | PASS | `data/motion-states.json`, `data/submits.json`, recording |
| 5 DESIGN.md matches shipped CSS | PASS (live re-check) | `data/ac5-served-vs-design.json` |

### AC1
1. buyer-906 with dark stored: 18 signed-in customer pages at 390, 768 and 1440 (54 loads). Every load gave html `light`, `data-surface=customer`, body rgb(248,250,252), and all 16 card tokens equal Christian's table. 0 page errors, no sideways scroll.
2. First paint, CPU 4x, full loads of /wallet, /settings, /offer/{id} and /trade-history at all three widths: 0 dark animation frames and 0 dark screencast frames after navigation.
3. Staff: /admin, /moderator/disputes and /admin/treasury as the admin account at three widths, review app against staging. Both are dark with no customer marker, and screenshots differ by at most 0.01% of pixels. The theme toggle in the staff drawer switches to Light and back to Dark on the review app and on staging.
4. Public, signed out: /, /help, /faqs, /contact-us, /login and /terms at 390 and 1440 are dark with stored dark and differ from staging by 0%. The /help drawer toggle switches Light and Dark.

### AC2
1. AmountInput on /offer/{id}: 48 px, radius 12, edge rgb(123,138,156), figure Figtree 18/600 tabular. Focus gives a rgb(3,105,161) edge with a 1 px ring. Typing 1.00 under the 600.00 minimum shows the shared field error: rgb(185,28,28), 13/18 500, 16 px circle icon, role alert, aria-invalid and aria-describedby set. Same at all three widths.
2. Checkbox on /offer/create: 22 px with a 44 px hit area, 1 px painted rgb(123,138,156) edge (1.5 px declared, owner item 13). Click ticks it with a rgb(0,115,173) fill and a 16 px white stroked tick (stroke 2.6). Space unticks it.
3. Inputs 48 px, radius 12, focus ring rgb(3,105,161). The /settings Select trigger is 48 px, radius 12, edge rgb(123,138,156). Enter, arrow and Enter changes the value on /offer/create at 768 and 1440. A tap opens it at 390.
4. Buttons on 12 customer pages at three widths: heights 36, 44 and 52; every 36 px button has a 44 px hit area. Primary is rgb(0,163,246) with a white label and the outline label is rgb(0,115,173). Icons are 18 and 20 px; the outliers are the #327 list (Create new ticket 20, Sign out 20) and 958's own `[&_svg]:size-4` on the reseller Edit button.
5. Trade-history filter chips (StatusChip as button) toggle aria-pressed, have a 44 px hit area, a pressed rgb(230,246,255) fill with a rgb(0,115,173) label, and a 2 px rgb(15,23,42) focus ring in the glass filter layer.
6. Keyboard focus: Tab sweep of 14 customer pages at three widths, 45 stops each. Shared controls always show a ring: rgb(3,105,161) on paper, rgb(15,23,42) on glass, and trade rows draw theirs on `::after`. Rings and edges measured against DESIGN.md: focus 5.67:1 on the page, 8.33:1 on navy, input edge 3.52:1, ticked fill 5.18:1.
7. Radios and progress Steps: no reachable live customer page renders them yet (radio rows are 958's, Steps sit in an active trade room). The AC2 re-check already proved them on the deployed CSS (pitch 34 px, tick 0 differing pixels). Not re-tested here.

### AC3
1. /dashboard/affiliate shows `+0.24 USDT` with figure and unit in rgb(4,120,87), `−121.20 USDT` (U+2212) in rgb(15,23,42), and held `121.20 USDT` navy with no sign. /loyalty-programme keeps `56.460674 USDT` at six decimals. Figures are set in Figtree with tnum and lnum.
2. Trade-history chips are all labelled (0 empty): Completed success, Cancelled and Dispute resolved neutral.
3. Error and Try again at 768 (the acceptance check covered 390 and 1440): /my-offers, /notifications and /offer/{id} each show the shared error with no 0.00. One click on Try again sends exactly one GET and the data comes back. Skeletons are static on rgb(241,245,249) at all widths.

### AC4
1. With reduced motion on, at three widths: 0 running infinite animations; skeletons have no animation; the button press transition is 0.00001 s; the phone sheet opens with opacity 150 ms linear only.
2. Real save: reseller default markup 5.0 to 5.5 at 390 with the request slowed by 3.5 s. The LoadingButton shows a still 18 px ring (animation none) with its words "Save markup" visible (`shots/submit-m-markup-loading.png`).
3. With motion on at 1440: the ring spins at 0.8 s linear infinite; press is 100 ms cubic-bezier(0.2,0,0,1), select 150 ms, dialog overlay 200 ms, phone sheet 280 ms. Recording `970-markup-save-390.webm` (428 frames, decodes clean, inspected as a frame strip) shows both saves with a turning ring and the toast.

### AC5
All 48 colour and motion rows of root DESIGN.md (f8de298) equal the served customer values. 42 match exactly and 6 match after unit normalisation (.1s = 100ms; #0f172a7a = rgba(15,23,42,.48)). The contrast exception table (2.77, 4.10, 5.18) matches the served values.

## Real submits (no money moved)

1. Settings: Name Display changed from Hidden to Initial + Surname with the shared Select and saved at 1440. The button read "Saving...", a "Profile updated successfully" toast appeared, and the API returned `initial_surname`. Restored to Hidden at 390 and the API returned `hidden`, so the baseline is back.
2. Reseller default markup: 5.0 to 5.5 and back, twice (once probing, once recorded). The API is back at 5.0.
3. Report a problem (/report, Bug Report) at 768: one POST /support/ticket and a success toast. I closed my test ticket afterwards ("[REPORT] QA 970 check, please ignore", now closed). 968's earlier QA ticket is still open and is not mine.
4. Marketplace sort filter changed (URL `?sort=most-trades`); a trade-history status chip was pressed and released.

## Jev gate

The first packet came back pass at confidence 0.37, reproducible 0.56. I re-examined it and added a decisive live probe (AC5 served tokens against DESIGN.md) plus the scope rulings. The retry (jev-1.14.0) returned pass at confidence 0.82 (p 0.88), severity 0.12, product decision 0.16, reproducible 0.70. Both answers are saved in `jev/`. Gate passed.

## Observations for other cards (not counted against 970)

1. /report (956-S8): the four report-type titles are hard-coded white on the Daylight surface (1.1:1), so they are unreadable until 956-S8 rebuilds the page.
2. /profile (956): caption text rgb(148,163,184) on rgb(241,245,249) is 2.34:1.
3. /dashboard/affiliate legacy FAQ band (958 affiliate): the white "Contact Support" button on the #00A3F6 band gets the paper ring rgb(3,105,161), which is 2.14:1 against the band.
4. Old /offer/create (957) and the wallet transaction search (958) keep their legacy field styles. On create, page classes override the shared Select edge and the outline price chips.
5. Wallet maintenance banner (958, pre-existing on main): a decorative 6 s spin with motion on. It stops under reduced motion.
6. Deposit dialog at 1440 (958/969): Escape did not close it while focus sat on its heading (focus was not inside the dialog element). Checked once, not investigated further.
7. Switches are page-owned (D3 and C boards), not on the shared boards. They toggle by click and Space, and show the old green on fill.
8. Settings (956): the disabled "Save Changes" fill matches its card, so the button reads as plain text; the bio link error is legacy text with no icon.

## Trello

Card moved Development Complete to QA / Testing at the start (read back). At the end: attachments, one comment, then a move to QA Approved / Ready For Deployment (user-authorized on PASS). Read-backs are in the final section below.

## Trello read-backs (fresh GETs, 05:11Z)

1. List: QA Approved / Ready For Deployment (69d535d2149a252f13a4cb18). The user authorized this move on PASS.
2. Comment 6adaec7d13a61b50b95b0100, first person, no em dashes.
3. Attachments 970-QA-1 to 970-QA-9 (eight PNGs and the webm clip), byte sizes equal to the local files in `upload/`.
4. Cleanup: Chrome for Testing profiles deleted, no tab left open. Test ticket closed, profile Name Display back to hidden, reseller markup back to 5.0.
