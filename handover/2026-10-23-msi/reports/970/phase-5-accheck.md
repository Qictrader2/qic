# 970 PHASE 5 ACCHECK (audit-only)

Card: https://trello.com/c/pY8CFeEI (list id 69d535bce3d4c6f14c6aa113, not moved). Run 21 to 22 Oct 2026.

## What was audited

- Round 1 (21 Oct) pinned f44c50dc (review app v5). Only AC5 finished before the pause; it was ticked then.
- Round 2 (22 Oct, coordinator resume): the review app https://qictrader-fe-pr-303.herokuapp.com kept redeploying as redesign/906 moved. Heroku builds API:
  - e0f7b827 (v14, built 10:44Z): AC1 browser run, AC2 live run, AC5 served-CSS check.
  - c04cac53 (built 14:43Z) and 3120af5d (16:09Z): AC3 browser run (16:06Z to 16:11Z).
  - c04cac53 to 4c1e806c (16:28Z to 16:58Z): AC4 browser run.
  - Between e0f7b827 and 674eb6d3 nothing touched the theme head script, routes.ts, theme-provider, globals.css, field.tsx, checkbox.tsx or radio-group.tsx. button.tsx changed only for icon sizes (3120af5d, 970 polish #327). /offer/[id] was rewritten by 957-S6.
- Local pins: frontend e0f7b827 (detached audit checkout `.scratch/970/accheck-root/frontend`, clean), root DESIGN.md 4344ec4 (root #302).
- Sessions: buyer-906, seller-906, third-906 from the qic-qa-signin agent; used only through qa-curl.sh and plantSession, one temp Chrome at a time. No writes, no money, no settings changed. Workers clicked "Reject all" on the consent banner once (privacy default).
- Decision rule applied: boards win over card text; contradictions.md decisions followed.

## Verdicts

| AC | Verdict | Jev (jev-1.14.0) | Trello | Evidence |
|---|---|---|---|---|
| 1 Daylight on signed-in customer pages, staff/public unchanged | MET | met 0.97, admissible 0.97, partly 0.11 | ticked | `~/.config/qic/qa-artifacts/970/r2/ac1/` |
| 2 Controls per v5.2 | MET on re-check 23 Oct (28645d0c, after #336); first check NOT MET | first check not_met 1.00 | ticked 23 Oct | `r2/ac2/recheck-28645d0c/` (first: `r2/ac2/live/`, `r2/ac2/pixel/`) |
| 3 Shared Amount, chips, states | MET (opus55-max arbiter 22 Oct; worker MET, Jev below bar) | met 0.74 (p 0.83); retry with corrected contiguous excerpts met 0.66 (p 0.77) | ticked by the arbiter 22 Oct, held for the arbiter | `r2/ac3/`, `jev/ac3-*` |
| 4 Reduced motion | MET | met 0.92, admissible 0.99, partly 0.10 | ticked | `r2/ac4/` |
| 5 DESIGN.md matches shipped CSS | MET (re-checked on 4344ec4 + e0f7b827) | round 1: met 0.88, admissible 0.94, partly 0.11 | ticked (21 Oct) | `ac5/`, `ac5/jest-e0f7b827.log`, `ac5/compare-css-e0f7.log` |

Evidence root: `/Users/starbird/.config/qic/qa-artifacts/970/`. Jev packets and answers: `jev/`.

## AC1 (MET)

buyer-906 with stored dark: 9 customer pages at 390 and 1440, html `light`, `data-surface=customer`, body rgb(248,250,252), 58/58 v5.2 tokens equal the fixture and all 15 card values. Radix select and dialog portals 58/58. First paint at 4x CPU on /dashboard, /settings, /offer/{id}: 0 dark frames (the #121212 frame appears only in the data: URL control). Six real-click round trips /help to customer page and back: 0 wrong-theme frames, /help keeps dark and its toggle. seller-906 /admin and /moderator/disputes keep the stored theme, no data-surface, toggle in the 390 drawer. Signed-out /, /help, /login, /contact-us, /faqs, /loyalty-programme: 24/24 equal staging; signed-in /loyalty-programme is Daylight. Jest 162/162.

## AC2 (MET on re-check 23 Oct at 28645d0c; first check NOT MET)

Matches: checkbox on rgb(0,115,173) with white tick; inputs 48 px, edge rgb(123,138,156), focus ring rgb(3,105,161), error rgb(185,28,28); focus rgb(3,105,161) on paper, rgb(15,23,42) in glass and gl-band (2 px at -2 px), rgb(56,189,248) on navy (contradictions item 7); 80 indicator and focus pairs at least 3:1; Steps and radio fills rgb(0,115,173) with white marks. Board comparison of 1,246 rows on a local build of e0f7b827: computed styles 404 MATCH, 722 FIXED-equal, 68 MEASUREMENT, 52 RECORDED, 0 REAL; served customer CSS equals the local build.

Carried reviewer notes, both pass on the live app:
1. Outline "Try again": label rgb(0,115,173) at rest, hover and pressed; hover fill rgb(241,245,249); pressed fill rgb(241,245,249), transform matrix(0.98,0,0,0.98,0,0).
2. Glass first-child field: cloned live qt-input keeps a 1 px ring, rgb(15,23,42) in .gl-dialog, .gl-sheet, .gl-popover and rgb(3,105,161) in .gl-solid; aria-invalid keeps rgb(185,28,28); a plain div still gets box-shadow none; a first-child qt-button keeps its outline.

Failures (board differences in 970 code, unchanged up to 674eb6d3):
1. FieldError draws a circle alert icon; the boards draw a triangle alert (rows field-error-001 to 007). `frontend/src/components/ui/field.tsx:247` `<CircleAlert`. Sheet `r2/ac2/pixel/side-px/field-error-007__1440d2.png`.
2. Checkbox tick is the Radix `CheckIcon`, smaller and thinner than the board's 16 px stroked tick, on every ticked row. `frontend/src/components/ui/checkbox.tsx:33`. Sheet `r2/ac2/pixel/side-px/checkbox-012__1440d2.png`.
3. RadioGroup default `grid gap-1` (`radio-group.tsx:82`) stacks rows at 26 px; G0 draws stacked rows at 34 px (22 px plus a 12 px gap). Each row's 44 px hit extender is cut to 26 px by its neighbour (`r2/ac2/pixel/hit-4a.json`). No live page uses a radio yet.

Not counted against 970 (page-level overrides, excluded by PLAN Scope exclusion 2, owned by the page-group cards): /offer/create Preview and Publish buttons (46 px, radius 4/10, blue edge), the -1% button, borderless title, price, min and max inputs and ZAR select, the markup input, its 12/400 error text; /settings Sign Out red label (page className at `settings/page.tsx:304-307`); /offer/create first checkbox's 31.5 px effective hit height from the page's 32 px row pitch. Listed for 957 and 956.

### AC2 re-check after Frontend #336 (23 Oct 2026): MET

Build: review app v34 deployed 28645d0c (Heroku build succeeded 03:44Z). Sessions from the qic-qa-signin agent; one headless Chrome for Testing on a temp profile, closed and deleted afterwards. Customer probes ran as third-906 (buyer-906 went 401 mid-run because two crashed probe scripts left app tabs open, and their refresh retry was no longer blocked; I closed them and made `lib.ts` close every tab on a crash). Staff probes as seller-906. My only non-GET requests were /g/collect analytics and app-boot refresh-token calls, which were blocked. The checkbox ticks were local form state and I put them back.

1. Checkbox tick (failure 2, fixed): both /offer/create checkboxes at 390 (DPR 2), 1440 and 1440 DPR 2 draw `data-slot=checkbox-tick`, 16 x 16 at 3/3 in a 22 px box, viewBox 0 0 24 24, path `M20 6 9 17l-5-5`, stroke 2.6 px rgb(255,255,255), round caps, on rgb(0,115,173). The Radix CheckIcon is `display:none`. Focus is 2 px solid rgb(3,105,161), offset 2. Pixel compare (DPR 2, 76 x 76, delta > 24, 1 px edge tolerance): against G0 `checkbox-024__G0-20` 0 after tolerance (bad 0 or 1); against `checkbox-012__probe-paper` 1 (bad 3 or 4). The e0f7b827 crop had 103 against the same probe crop. Sheets: `recheck-28645d0c/pixel/side-px/checkbox-*`.
2. Radio stack (failure 3, fixed): no live customer page renders a RadioGroup (NetworkRadioRows has no caller at 28645d0c), so I rendered the 28645d0c component (`logs/radio-dump.html`) and placed it on live /settings with the deployed CSS. Group class `grid gap-1 customer:gap-3` gives row-gap 12 px, rows 22 px, pitch 34 px at 390 and 1440 (was 26). Radio 22 px rgb(0,115,173), white 8 px dot, label 14/500/20. The effective hit is 34 px per inner row (rows share the gap) and 44.5 px on the last row; the 30 px last-row figure in the raw file is my probe's layout, where the next probe group sits 8 px below. A caller's own gap still wins (`gap-2.5`, 10 px). The G0 stack (rows at y 5200 and 5234 of `renders/G0.png`) against the live 1440 stack: 150 x 72, bad 232, 0 after tolerance (anti-aliased text edges). Sheet `pixel/side-px/radio-stack__G0__live-1440.png`.
3. Amount units: live /dashboard/affiliate (third-906) `+0.24 USDT` has its unit in rgb(4,120,87), the same as the figure, where the 22 Oct run showed navy. `−121.20 USDT` is rgb(15,23,42) on both. Matrix render on live /settings: in/out units take the figure tone at xs, sm, md, xl, summary and inline; lg stays rgb(71,85,105) (B8 and lg-popover); held and neutral units stay muted; `unitTone` overrides work both ways. Other customer pages (/loyalty-programme, /trade-history, /earnings, /wallet, /dashboard, /reseller, /marketplace) show no directional Amount for this account.
4. Carried notes still pass: Try again (outline md 44 px) on /my-offers and /notifications at 390 and 1440 has label rgb(0,115,173) at rest, hover and pressed, hover and pressed fill rgb(241,245,249), pressed transform matrix(0.98,0,0,0.98,0,0) and focus 2 px rgb(3,105,161). Glass first child: 1 px ring rgb(15,23,42) in gl-dialog, gl-sheet and gl-popover, rgb(3,105,161) in gl-solid, aria-invalid rgb(185,28,28), plain div box-shadow none. All 36 probes and all Try again states give the same computed values as the 22 Oct run; one background sample differs by 1 colour unit.
5. Staff keep the old look: on seller-906 /admin/banned-words (html `dark`, no data-surface) the real checkboxes are 16 x 16 with a 2 px rgb(0,163,246) edge, radius 4, and the Radix tick (12 px) shown, with the board tick hidden. The injected RadioGroup there has a 4 px gap, 44 px rows and native 16 px radios. Amount units stay muted rgb(148,163,184).
6. Failure 1 (circle field-error icon): not a defect, per the coordinator ruling. Not re-tested.

Trello: AC2 item 6ad887d452bd446e7d78bc1e ticked and read back complete; comment 6adadd600a039be536071ffd. The card was not moved. All five AC items now read complete.
Evidence: `/Users/starbird/.config/qic/qa-artifacts/970/r2/ac2/recheck-28645d0c/` (live/checkbox-raw.json, radio-amount-raw.json, amount-live-raw.json, B-raw.json, CD-raw.json, live/shots/, live/scripts/, pixel/pxcompare-*.json, pixel/side-px/). Render harness source: `live/logs/radio-dump.html`, `live/logs/amount-dump.html`.

## AC3 (held for the arbiter)

Worker verdict MET. Live: /affiliate shows `+0.24 USDT` rgb(4,120,87) and `−121.20 USDT` rgb(15,23,42) in Figtree, held 121.20 with no sign; chips neutral rgb(71,85,105) on rgb(241,245,249), compact 22 px. /my-offers, /notifications, /offer/{id} at 390 and 1440: qt-skeleton static on rgb(241,245,249), qt-error with "Try again" and no 0.00, exactly 1 request per retry and data back in 6 of 6 runs, qt-empty on the two list pages. Jest 135 tests in 7 suites at e0f7b827. Held, Unavailable, loading skeleton and six-decimal cases rest on Jest bodies only (no live page shows them). Jev stayed below 0.85 on both tries (0.74, then 0.66 after replacing stitched excerpts with exact contiguous lines). Per the run rule I stopped on AC3 and did not decide it; the coordinator escalates it. Side note: /trade/{id} still draws its own grey Cancelled pill and plain amounts (959 adoption, outside 970).

## AC4 (MET)

Reduced motion on: getAnimations() 0 on /my-offers, /notifications (requests held), /marketplace, /wallet, /trade-history at both sizes; skeletons static; html transition none through a public/customer navigation; select, popover and modal open with no animation; loading text at opacity 1. Spinner still ring and LoadingButton words render-proved (live markup injection plus 23 Jest tests); no reachable live spinner without a write. Motion on: press 0.1 s cubic-bezier(0.2,0,0,1), progress 0.2 s, glass modal 0.28/0.2 s, palette 0.15 s. / and /lp animations equal staging. Jest reduced-motion 61 passed. Outside 970: Radix select/popover content 150 ms `ease`, legacy buttons, consent banner.

## AC5 (MET, re-checked)

DESIGN.md at 4344ec4 with frontend e0f7b827: design-md-tokens 17/17, including the in-memory negatives (--focus #0284C7 named; CTA exception row removal named). Served review-app CSS: 58 documented rows, 0 differences. Stays ticked.

## Trello writes

1. AC5 ticked 21 Oct (read back complete).
2. AC1 ticked 22 Oct (read back complete).
3. AC4 ticked 22 Oct (read back complete).
4. One summary comment posted (comment id 6ada448f5ebf76c0c90aa6bf).
No move, no attachment. Collage not built: attachments were not authorized for this run.

## Fix scope for a new PR into redesign/906 (AC2)

1. FieldError icon to the board's triangle alert.
2. Checkbox tick to the board's 16 px stroked tick.
3. RadioGroup default stack gap to 12 px (board pitch 34). Whether stacked hit areas must reach 44 px when the board pitch is 34 is a board-vs-rule question; the board pitch wins under the decision rule.

Arbiter decision, opus55-max, 22 Oct 2026, after Jev 0.74 and 0.66: MET.

1. Money: live /affiliate `+0.24 USDT` rgb(4, 120, 87) and `−121.20 USDT` rgb(15, 23, 42); /loyalty-programme `56.460674 USDT`. `frontend/src/components/ui/amount.tsx:56-92` gives held `text-money-out` with no sign and turns null, non-decimal or wrong-signed values into Unavailable (`:139-155`); `src/lib/exact-amount.ts` pads and never rounds. Follows G0 section 7. U+2212 is what 342 money figures on 48 v5.2 boards use; only the G0 sample row uses a hyphen.
2. Chips: `status-chip.tsx:65-67` throws on an empty label; live neutral 28 px, success and compact 22 px 12/600 chips resolve to the G0 tones.
3. States: (main) /my-offers, (dashboard) /notifications and (offers) /offer/{id} are the three customer route groups. All three use the shared Skeleton and ErrorState at ce1f24bc, the two list pages also EmptyState. Skeletons are static on --surface with radius 8, as c.css and C18 draw them. 1 request per Try again in 6 of 6 runs.
4. Jest re-run at ce1f24bc: 7 suites, 135 of 135 pass (`r2/ac3/arbiter-jest-ce1f24bc.log`).
5. Jev's gap: held, Unavailable and Cancelled were proven by Jest classes only. They are the same classes the live reads resolve (text-money-out rgb(15, 23, 42); neutral rgb(71, 85, 105) on rgb(241, 245, 249)), so no live render of them is needed.
6. Not counted, page cards: Amount colours the unit only in `summary`, while the boards colour USDT with a directional figure (B16 `+0.24 USDT` all green), so the /affiliate inline example shows a navy USDT (958). /trade/{id} keeps its own Cancelled pill and plain amounts (959). The /notifications empty title reads "No notifications" where D4 draws "No notifications yet" (956).

Trello, for the overseer: tick AC3 item 6ad887d5608fd2d0d575aa3b, post the arbiter comment, do not move the card. Update the Verdicts row and this heading to MET.
