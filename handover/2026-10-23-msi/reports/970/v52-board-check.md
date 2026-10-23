# Ticket 970: pre-merge v5.2 board check

PR: https://github.com/Qictrader2/Frontend/pull/304 into `redesign/906`. Frontend worktree
`.scratch/906-redesign/970-root/frontend` at `419b721b`. Root DESIGN.md at `.scratch/906-redesign/970-root/DESIGN.md`
(root commit `93f3493`). Checked 21 Oct 2026.

## Verdict

**GAPS BLOCK MERGE.** One blocking gap (a focus rule inside glass), three minor gaps.

| Result | Count |
|---|---|
| MATCH | 42 |
| GAP, blocks merge | 1 |
| GAP, minor | 3 |
| OUT-OF-SCOPE | 9 |

Blocking gap:

1. Glass focus ring on a field is removed when the field is the first child of a content zone.
   `frontend/src/app/globals.css:1009` ports the pack rule
   `.gl:is(.gl-sheet,.gl-dialog,.gl-popover,.gl-fade) > .gl-body > :first-child:not(.gl-plate,.gl-mi,.input,.btn,.card,.chip){box-shadow:none !important}`
   word for word. The pack exempts its own `.input` class, but the React `Input` (`qt-input`) does not carry `.input`.
   A focused `Input` that opens a glass sheet, dialog or popover body, or an opaque commit step (`.gl-solid`), therefore
   loses its 1 px focus ring (and its 1 px error ring when invalid). Only the 1 px edge remains. v5.2 (qt-v5.css
   `.input.is-focus`, glass 5i, G1 item n) draws the field focus as a 1 px edge plus a 1 px ring: `--foreground` in glass,
   `--focus` in commit steps and on plates. Measured: first child `box-shadow: none`; the same input moved to second
   child gets `0 0 0 1px rgb(15,23,42)` (glass) or `0 0 0 1px rgb(3,105,161)` (commit step). Evidence:
   `evidence/v52/h-glass-first-child-probe.json`, `h-glass-1440-input-focus.png`.
   Fix: add the shared controls to the exemption, for example
   `:not(.gl-plate,.gl-mi,.input,.btn,.card,.chip,[data-testid=qt-input],[data-testid=qt-button],[data-testid=qt-status-chip])`,
   or exempt `input, textarea, select`. Add a Jest assertion that a first-child `qt-input` in `.gl-body` keeps its ring.
   No screen in this card has a glass layer yet, but the 969 dialogs and sheets will inherit this rule, and a field
   as the first row of a sheet is the common case (amount, address, code).

## Method

1. Sources. The v5.2 PDFs in `~/Downloads/906-design-pack-v5.2/` (G0, G1, G14, G17 vector; G4 vector; G16b 2x raster),
   rendered with `sips`/`qlmanage` and text-extracted with PDFKit (`evidence/v52/boards/`). `CHANGES.md` rows G1, G0,
   G17, G14 (960 part 970), G4 and G16b (960 part 969) and the A-group "shared only" rows (A11a, A11c, A20a, A20c, A9a,
   A19a, A22a, A22b). Text sources: `qt-v5.css` (sha256 3b761a9b...c47, matches G0 and G14), `glass-v3.1.css` v3.2.1
   (sha256 f9d992f4...39d, matches), the G0 board source `shared/project/tokens-v5.dc.html` and `g31.css`. The hosted
   canvases were not used.
2. Build. `rm -rf .next && bun run build` with a temporary harness route `/offer/qt-harness-v52` (exit 0,
   `evidence/v52/build.log`), served by `PORT=3000 bun run start`. The harness rendered the G0 section 7 "Selected
   controls" composition (ticked checkbox, selected radio, Funded/Paid/Released steps, focused Amount to sell field),
   every Button variant and size, Input default/error/disabled with FieldError, Amount and StatusChip rows from G0, a
   paper card, a `.qt-on-ink` navy panel, a `.gl.gl-dialog` with band, content zone and plate, a `.gl.gl-dialog.gl-solid`
   commit step, and EmptyState, ErrorState, Skeleton and Spinner.
3. Capture. Puppeteer (Bun) on Chrome for Testing with a temporary profile, deleted afterwards; 390x844 at 2x and
   1440x900. Every request to a host other than localhost:3000 was aborted (only connect.facebook.net was tried) and
   every `/api/*` and `/socket.io` request was answered from local fixtures. No sign-in, no cookies, no passwords; the
   customer pages used a seeded `auth-storage` fixture user (fixture-970@example.test) as in the earlier evidence.
   Keyboard focus came from real Tab presses; hover and press from real mouse events.
4. Cleanup. Harness deleted, server stopped, `.next` rebuilt without the harness (`build-clean.log`, exit 0, no harness
   route), `git status --short` empty in the frontend worktree.
5. Also run: `contrast.test.ts` and `design-md-tokens.test.ts` on 419b721b, 72 of 72 passed
   (`evidence/v52/jest-contrast-designmd.txt`).

Evidence paths below are relative to `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/v52/`. Scripts:
`.scratch/970/tools/v52-check.ts` and `v52-probe.ts`.

## Board items

Colours are written as the token, with the measured computed value where it was measured.

### G0: token sheet v5.2

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 1 | Token values | qt-v5.css v5.2 `:root`, unchanged from v5 values | All 58 pack tokens read from `html[data-surface=customer]` on the harness equal qt-v5.css at both widths; only serialisation differs (`#fff`, hex alpha such as `#0f172a7a` = rgba(15,23,42,.48), `.1s` = 100ms, font names resolved to the next/font faces) (`h-harness.json` runs[].surface, `_summary-h.json`) | MATCH |
| 2 | Section 1, primary CTA | `--action` #00A3F6 white label; hover `--action-hover` #0284C7; pressed `--action-pressed` #0073AD, scale .98, 100 ms | rest rgb(0,163,246)/white; hover rgb(2,132,199); pressed rgb(0,115,173), matrix(.98); transition transform 0.1s cubic-bezier(.2,0,0,1) (`h-harness.json` interact, motionOn) | MATCH |
| 3 | Section 7, outline button | "Outline button: `--action-pressed` label (5.18:1), `--input-border` edge, `--card` fill" (drawn "View trade" uses g31.css `.btn-outline`) | `variant="outline"` and `"secondary"` both render qt-v5.css `.btn-secondary`: card fill, #7B8A9C edge, `--foreground` label rgb(15,23,42) (`h-harness-1440.png`, `#b-outline`) | GAP, minor |
| 4 | Section 7, text links and tertiary | `--action-pressed` label; tertiary hover `--action-subtle` | ghost and link: rgb(0,115,173); hover and pressed rgb(230,246,255) | MATCH |
| 5 | Section 7, danger | `--error` fill, white label, hover `--error-hover`, pressed `--error-pressed` | rgb(185,28,28) / rgb(153,27,27) / rgb(127,29,29), scale .98; danger-quiet `--error` label, `--error-bg` hover | MATCH |
| 6 | Section 7, control edges | `--input-border` on every control edge; `--border` decorative only | Input, Checkbox, Radio, outline button edges rgb(123,138,156); disabled controls use `--border` per qt-v5.css `.is-disabled` | MATCH |
| 7 | Section 7, money in and out | `+120.00 USDT` `--money-in`; `-120.00 USDT` navy `--money-out`, never red; held 121.20 navy, no sign | `+120.00` rgb(4,120,87); `−120.00` rgb(15,23,42); `121.20` rgb(15,23,42) no sign; Figtree tabular lining figures; unit Poppins 500 `--muted-foreground`; `1,234.567891`; `Unavailable` (`h-harness.json` text.amounts) | MATCH |
| 8 | Section 7, status badges | info, success, warning, Your turn (`--attention` with navy), error, Cancelled neutral (`--muted-foreground` on `--surface`), each with a word | All six tones plus action computed exactly to the qt-v5.css `.chip-*` pairs, 28 px, 13/16 600, label required | MATCH |
| 9 | Section 7, USDT mark | `--usdt` mark only, never text or status | Amount and StatusChip never use `--usdt`; token shipped for the mark | MATCH |
| 10 | Section 7, focus on paper and navy | `--focus` on opaque light surfaces; `--focus-on-ink` on navy | Paper button: 2px solid rgb(3,105,161), offset 2px; `.qt-on-ink` button: 2px solid rgb(56,189,248) (`h-focus-*-f-paper.png`, `h-focus-*-f-ink.png`) | MATCH |
| 11 | Section 7, ticked checkbox | `--action-pressed` fill and edge, white tick, 22 px, 6 px radius, 1.5 px `--input-border` when off | rgb(0,115,173) fill and edge, tick rgb(255,255,255), 22x22, radius 6, 44x44 `::before` hit area. The 1.5 px edge computes to 1 px, as does a reference `border:1.5px` div at DPR 1 (Chrome border snapping, same as the pack would render) (`h-g0-controls-*.png`, `h-glass-first-child-probe.json` border) | MATCH |
| 12 | Section 7, selected radio | `--action-pressed` fill, 8 px white dot | rgb(0,115,173), `::after` 8x8 white; label row 44 px high | MATCH |
| 13 | Section 7, current step colours | done `--success`, current `--action-pressed` with 600 label, upcoming `--border` and `--text-secondary` 500 label, 4 px bar, 13/18 | rgb(4,120,87) / rgb(0,115,173) / rgb(226,232,240); labels 500 / 600 / 500 in foreground, foreground, text-secondary; `aria-current="step"` on Paid | MATCH |
| 14 | Section 7 and G4, step layout | Bars separated by 6 px (G0 `gap: 6px`, G4 trade room `gap: 6px`, every group board 6 to 8 px); labels are text only | `Steps` renders bars touching (measured gap 0, qt-v5.css `.steps{gap:0}` default) and adds a check icon before done labels (`h-harness.json` text.stepGap = 0, hasIcon true; `h-g0-controls-1440-fieldfocus.png`) | GAP, minor |
| 15 | Section 7, focused field on paper | Edge `--focus` plus 1 px ring (qt-v5.css `.input.is-focus`) | border rgb(3,105,161) plus `0 0 0 1px rgb(3,105,161)`; error edge and ring rgb(185,28,28); 48 px, 12 px radius | MATCH |
| 16 | Sections 8 and 9, contrast table | Pairs and ratios as listed; CTA 2.77/4.10/5.18 the only failing row | Shipped tokens equal the pack, so every ratio is the pack's; `contrast.test.ts` passes on 419b721b | MATCH |
| 17 | Section 10, file hashes | qt-v5.css 3b761a9b..., glass v3.2.1 f9d992f4... | Local sources hash to both; build values equal qt-v5.css; 179 of 217 glass rule lines are verbatim in globals.css, the other 38 are board-only demo classes (`.fb-*`, `.is-window`, `.is-narrow`, `:not(.v3-geometry)`) or dock geometry adapted without the board class (`glass-port-compare.txt`) | MATCH |
| 18 | Section 1, emailed sign-in code dialog | CTA instance with the same 2.77:1, owner card 904 | Not in this card | OUT-OF-SCOPE (904) |

### G1: shared glass v3.2.1

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 19 | Item n, focus ring in glass | 2 px `--foreground` in bands (offset -2) and content zones | Band link 2px solid rgb(15,23,42) offset -2px; content-zone button and checkbox 2px solid rgb(15,23,42) offset 2px | MATCH |
| 20 | Item n and 5i, focused field edge in glass and commit step | Edge and 1 px ring in `--foreground` in glass, `--focus` in `.gl-solid` and on plates | Edge colour right in both; the 1 px ring is removed when the field is the first child of `.gl-body` (`globals.css:1009`) | GAP, blocks merge |
| 21 | 5i, focus inside commit step and on a plate | `--focus` | `.gl-solid` button and checkbox, plate button: 2px solid rgb(3,105,161) | MATCH |
| 22 | Item o, ticked checkbox in content zone and commit step | `--action-pressed`, white tick | rgb(0,115,173) in both contexts (`h-glass-*.png`, `h-solid-*-cb-focus.png`) | MATCH |
| 23 | Sections 2 and 3, material | Blur 16 px, saturate 1.5, 84 to 78% stain, brand 10%, content zone 96%, 28 px fade, plates on `--card`, 1 px white edge, lead line | Variables and classes ported verbatim (`globals.css:416-430`, `:827-905`; `glass-port-compare.txt`) | MATCH |
| 24 | Section 5 and v3.1.1 5f, opaque commit step | One opaque `--action-subtle` pane, no line under the header, same-hue body over 28 px | `.gl-solid` computes rgb(230,246,255), body gradient, no band rule (`globals.css:996-1005`; `h-harness.json` #ctx-solid) | MATCH |
| 25 | Section 6, motion | Enter and exit per layer on `--dur-*` and `--ease-*`; reduced motion opacity only, 150 ms linear | Layer transitions and the reduced-motion override ported (`globals.css:894-908`) | MATCH |
| 26 | Fallbacks | No backdrop-filter, reduced transparency, more contrast, forced colours, `data-glass="lite"` | All five present, scoped to the customer surface (`globals.css:495`, `:921-951`). Read from source, not rendered under emulation | MATCH |
| 27 | Layer budget, dock, island, toast placement, sheet and dialog behaviour | Section 4, 8 and 10 B5 rules | Not in this card | OUT-OF-SCOPE (968 phone bar and overlays, 969 dialogs and toasts) |
| 28 | 44 px hit areas (.hit44, Q6) | Keep the visual size, extend the hit area | `.hit44` ported; checkbox `::before` 44x44; radio label 44 high; small button 36 high with 44 high `::after` | MATCH |
| 29 | Dispute line (no cancel or release in dispute) | Confirmed rule | Not in this card | OUT-OF-SCOPE (953, 963, group A 959) |

### G14: what each group copies, section 5 (v5.2)

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 30 | Copy qt-v5.css v5.2 unchanged | `.check.is-on`, `.radio.is-on` `--action-pressed`; `.step.is-current` `--action-pressed`; `.input.is-focus` `--focus`; `.is-focus` outlines `--focus` | All reproduced on the customer variant (rows 11 to 15) | MATCH |
| 31 | Copy glass v3.2.1, section 5i | Focus `--foreground` in `.gl`, `--focus` in `.gl-solid` and on `.gl-plate` | Ported (rows 19, 21); the first-child ring defect is counted once, at row 20 | MATCH |
| 32 | Remove overrides painting indicators or focus in `--action` or `--brand-blue` | Ticked, selected, current step, progress bar, focused field edge, focus ring | Customer variants use `--action-pressed` and `--focus`; Progress fill rgb(0,115,173) on `--surface`; the April `outline-brand-blue` base rule is overridden under the customer surface (`globals.css:762-766`) | MATCH |
| 33 | CTA kept, own row | `--action` with white text; 2.77/4.10/5.18 on a separate row "Kept as approved by Christian (21 Oct 2026); limitation documented" | Button default is `--action` white; DESIGN.md section 12 carries the row with that wording | MATCH |
| 34 | Sections 1 to 4, group items | Docks, per-group layers, stylesheet moves | Not in this card | OUT-OF-SCOPE (group cards 955 to 959, 968, 969) |

### G17: decisions and status

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 35 | Control corrections, Confirmed | Fills and current step `--action-pressed`; focus by surface | As rows 11 to 15, 19, 21 | MATCH |
| 36 | CTA label, Kept as approved | 2.77/4.10/5.18, palette not called AA compliant, separate row | DESIGN.md section 12; no line calls the palette compliant (`design-md-tokens.test.ts` asserts this) | MATCH |
| 37 | B5 shared rules | Visible focus per surface, 44 px targets, reduced motion opacity only, colour never alone | Rows 10, 19, 21, 28; chips require a label; reduced-motion run below | MATCH |
| 38 | B6 real-phone test | Pending (QA, at build) | Not in this card | OUT-OF-SCOPE (QA at build; 965) |
| 39 | Disputes item 1, A5 password rules | Confirmed product rules | Not in this card | OUT-OF-SCOPE (953, 963, 917, groups A, D, E) |

### G4 and G16b (960 part 969, shared controls only)

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 40 | G4 current step bar and ticked checkbox | `--action-pressed` through qt-v5.css v5.2 | Same rendering as rows 11 and 13 (step spacing counted at row 14) | MATCH |
| 41 | G16b ticked checkbox on the opaque commit step | `--action-pressed` (4.88 on the fill), commit step one opaque `--action-subtle` pane | `.gl-solid` checkbox rgb(0,115,173) on rgb(230,246,255) pane (`h-solid-*-cb-focus.png`) | MATCH |

### A group, shared-only rows (960 part 967)

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 42 | qt.css v5.2 ticked check, current step, focused field | As G0 section 7 | Rows 11, 13, 15 | MATCH |
| 43 | trading.css items | Current step bar on trading screens, timeline dot, your-turn edge, caret | Not in this card | OUT-OF-SCOPE (967) |

### Card-scope items checked against the same v5.2 rules

| # | Item | v5.2 specifies | Build shows (evidence) | Result |
|---|---|---|---|---|
| 44 | Daylight forcing on the three sample pages | Single light theme, customer surface | `/my-offers`, `/notifications`, `/offer/{id}` all `html.light[data-surface=customer]` with stored dark (`s-states.json` before.ds, cls) | MATCH |
| 45 | Reduced motion | Decorative animation and shimmer stop; spinner becomes text; status text stays | `getAnimations()` running: none; LoadingButton reads "Loading...", spinner icon `display:none`, spinner label stays (`h-harness.json` reducedMotion; `h-harness-*-rm.png`) | MATCH |
| 46 | Motion on | Spinner the only loop; press 100 ms `--ease-out` | Only two `spin` animations (800 ms); button transition transform 0.1s cubic-bezier(.2,0,0,1) | MATCH |
| 47 | Skeleton | Static `--surface` blocks, 8 px radius (qt-v5.css `.sk`) | Loading state on all three pages: rgb(241,245,249), radius 8 (avatars round), animation none (`s-*-loading.png`) | MATCH |
| 48 | Error and empty states on the sample pages | Error words in `--error` on `--error-bg`, text label, no 0.00 | ErrorState icon well rgb(254,242,242)/rgb(185,28,28), title 16/24 600, text 14/20 `--muted-foreground`, outline "Try again"; EmptyState `--surface` well; no 0.00 in the page body; Try again sent exactly one request and rendered data in 5 of 6 runs; the sixth (`/notifications` at 390) clicked the cookie banner that covers the button in this test browser, not the button (`s-states.json`, `s-notifications-390-retry.png`) | MATCH |
| 49 | Amount rule edge cases | Exact digits, never rounded; Unavailable, never 0.00; loading skeleton | `1,234.567891`, `Unavailable`, static skeleton (row 7 evidence) | MATCH |
| 50 | Inputs: disabled and error | Disabled `--surface` fill, `--border` edge, `--text-secondary`; error edge and ring `--error` with message and icon | Disabled rgb(241,245,249)/rgb(226,232,240)/rgb(85,98,116); error rgb(185,28,28) edge and ring; FieldError 13/18 500 rgb(185,28,28) with icon, role alert | MATCH |
| 51 | DESIGN.md tokens against G0 | Same names, values and roles | All 41 colour tokens equal G0 and qt-v5.css; type, radius, shadows, motion equal qt-v5.css; `design-md-tokens.test.ts` passes | MATCH |
| 52 | DESIGN.md CTA exception against G0 sections 1 and 9 and G17 | 2.77 default (fails 4.5 and 3), 4.10 hover (fails 4.5), 5.18 pressed (passes); "Kept as approved by Christian (21 Oct 2026); limitation documented"; palette not AA compliant as a whole | Section 12 states all of it in those words and names #00A3F6 as the only exception | MATCH |
| 53 | DESIGN.md internal consistency on outline buttons | One role for the outline label | Token table gives `--action-pressed` the role "links and outline labels" (as G0), but section 7 maps `outline` to a `--foreground` label (as qt-v5.css `.btn-secondary`) | GAP, minor |
| 54 | Page-level items seen on the sample pages | Header balance shows Unavailable or a skeleton, never 0 | Header balance strip reads "0 USDT" when the balance request returns nothing (`s-my-offers-1440-error.png`) | OUT-OF-SCOPE (968, header and BalanceStrip) |
| 55 | Other page-level colours on the sample pages | `--action` is never text | Cookie banner "Cookie Policy" link in #00A3F6 text; brand-blue filled "All" tab on /my-offers and "Notifications" tab | OUT-OF-SCOPE (colour sweeps 966 and 967; /notifications page look group D 956) |

## Minor gaps, with suggested fixes

1. Row 3 and row 53, outline button label. The pack disagrees with itself: G0 section 7 (and its `--action-pressed`
   role text) draws an outline button with an `--action-pressed` label through `g31.css .btn-outline`, while qt-v5.css,
   the stylesheet every screen board uses, has only `.btn-secondary` with a `--foreground` label. The build follows
   qt-v5.css. Owner decision: either map `variant="outline"` to an `--action-pressed` label
   (`frontend/src/components/ui/button.tsx`, the `outline` entry's `customer:text-foreground` and
   `customer:hover:text-foreground`) and keep `secondary` navy, or keep the build and change DESIGN.md's
   `--action-pressed` role to "links and tertiary labels". Either way, make DESIGN.md section 2 and section 7 agree.
2. Row 14, Steps layout. `frontend/src/components/ui/progress.tsx`: change the `Steps` list from `gap-0` to `gap-1.5`
   (6 px, as G0 and G4 draw it) and drop the check icon in done labels, or document it as an intentional addition. The
   colours are right; only spacing and the extra icon differ from the boards.

## Not verified

1. Glass fallbacks (row 26) were read from source, not rendered under `prefers-reduced-transparency`,
   `prefers-contrast: more` or `forced-colors` emulation.
2. Nothing was checked on the review app https://qictrader-fe-pr-303.herokuapp.com or with a signed-in account; the
   customer pages ran on a local production build with local fixtures.
3. `/offer/{id}` with a 404 shows the generic error state ("We couldn't load this offer"); the offer not-found screen is
   drawn on group C board C6 (card 957) and was not judged here.
