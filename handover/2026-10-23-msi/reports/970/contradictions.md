# Ticket 970: v5.2 board contradictions for the owner

22 Oct 2026. These are the places where the v5.2 boards disagree with each other, or where a board's drawing
disagrees with its own spec text or class, so "copy the board" has no single answer. Nothing here is changed in
code beyond what 970 already shipped (merge f44c50dc); each item needs a choice.

Sources: board reference `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/replicate/board/inventory.json`
(values computed in headless Chrome from the v5.2 board sources; the PDFs for G0, G1, G12a, G14 and G17 match the
sources apart from anti-aliasing), comparison `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/replicate/diff/diff.json`.
Crops: `evidence/replicate/board/crops/<rowid>__<board>-<n>.png`. The comparison flagged 25 board rows (41 row x
viewport comparisons). They reduce to items 1 to 13 below (three have sub-cases). Item 14 came from a later scan of every v5.2 PDF.

"Class" means the value the qt-v5.css or glass class gives. "Inline" means the board overrides that class with an
inline style on one drawn instance. "Text" means a written rule on a board.

## Summary

| # | Item | A | B | Build today | Recommended |
|---|---|---|---|---|---|
| 1 | Money figure in summary rows | `.amt`: lining + tabular, -0.01em (G12a, G11, G10) | `.kv dd`: tabular only, normal tracking (G3, G4, G8) | `.amt` everywhere | One style: `.amt` |
| 2 | Non-money values in summary rows | Figtree figures (G4, G8) | Poppins 500 inline (G3) | not built | Poppins for words, Figtree for figures |
| 3 | Reference codes in the figure font | `.amt` on `QT-7F3A21` (G4, G10, G11) | G0 Amount rule is for money | Amount only for money | Figure font, not the Amount component |
| 4 | USDT unit beside or inside the figure | beside `.amt`, normal features (G5) | inside `.amt`, lnum/tnum (G7) | separate unit span | Keep. Same pixels |
| 5 | Tertiary button label colour | Text and class: `--action-pressed` (G0 s9 item 3) | Drawn `--foreground` inline (G4, G7) | `--action-pressed` | Keep `--action-pressed`. Back link goes to 968 |
| 6 | Tertiary shapes drawn inline | class: 0 18px, gap 8px, no underline | G4: 0 12px 0 6px, gap 2px. G7: underline, offset 3px | class values | Back link and band link are 968 patterns |
| 7 | Focus ring on navy | Text: `--focus-on-ink` on navy (G0 s7, G17) | Stylesheet: `--focus` for controls on navy | `--focus-on-ink` inside `.qt-on-ink` | Keep the text rule |
| 8 | "Your turn" chip size | class: 28px, 0 10px, 13px (G0) | inline: 22px, 0 8px, 12px (G4, G6) | 28px only | Owner: add a 22px compact size or not |
| 9 | Chip as a control | static, cursor auto (G0, G4) | tappable, cursor pointer (G6) | static plus an `as="button"` form (in progress) | Interactive only when it does something |
| 10 | Square icon-only buttons | class: secondary lg 0 22px, md 0 18px | inline: 52x52 padding 0 (G3, G6). 44x44 padding 0 trigger (G8) | icon size 44 only | Owner: add 52 px square size or not |
| 11 | Field height in a glass band | class `.input`: 48px | inline: 52px Jump field (G9) | 48px | Keep 48. 52 is the palette's own (968/969) |
| 12 | Steps gap | class `.steps`: gap 0 | every drawn instance: gap 6px inline (G0, G4) | 6px (shipped) | Keep 6px |
| 13 | Checkbox and radio edge | class: 1.5px `--input-border` | painted: 1px in Chrome at DPR 1, 2 and 3 | 1.5px declared, 1px painted | Owner: keep 1.5 or set 1 or 2 |
| 14 | Loading primary button fill | `--action-pressed` #0073AD (C12 frame 28 "Publishing...") | `--action` #00A3F6 (A21a, A21b, E1d frame C, D6a, D6b) | `--action` | Keep `--action` (majority, CTA rule) |

## Items

### 1. Money figure in summary rows: `.amt` against `.kv dd`

- What it is: the same kind of value, a USDT amount in a label and value row, is drawn with two text styles.
- A, `.amt` class: Figtree, lining and tabular figures (`"lnum", "tnum"`), letter spacing -0.01em
  (-0.14px at 14px), host weight. G12a "Edit offer" (rows amount-065, 14/700/20), G11 "Delete this offer?"
  (amount-074, 13/400/18 caption), G10 "My offers" (amount-081, 13/400/18).
- B, `.kv dd` summary value: Figtree, tabular only (`font-feature-settings: normal`), letter spacing normal,
  14/600/20. G3 "Deposit USDT" (amount-066), G4 "Release 120.00 USDT to naledi_m?" (amount-077), G8 "What happened"
  and "Amounts" (amount-088).
- Visible difference: mainly the tracking, 0.14px per character at 14px, about 1px over a seven-character amount.
  The `lnum` feature changes glyphs only if Figtree's default figures are not lining; I did not check that pixel by pixel.
- Build today: Amount always applies the `.amt` style. The builder is adding a `summary` size with the `.kv dd`
  values (14/600/20, tabular, normal tracking) so the G3, G4 and G8 rows can be copied exactly.
- Recommended: one money figure style everywhere, `.amt`, because G0's Amount rule names lining tabular figures and
  this is the only place they differ. If the owner prefers that, the `summary` size takes the `.amt` tracking and
  features.

### 2. Non-money value in a summary row drawn in Poppins

- A: G4 and G8 draw every summary value in Figtree (`.kv dd`).
- B: G3 "Deposit USDT" row 2 (amount-067) overrides one value inline with `font-family: var(--font-sans); font-weight: 500`,
  so it reads 14/500 Poppins.
- Build today: not a 970 component. Summary rows belong to the screens that use them.
- Recommended: Poppins 500 for words (network names, usernames), Figtree for figures. Treat G3's override as that
  rule, not as a contradiction to copy per screen.

### 3. Reference codes in the figure font

- A: G4 (`QT-7F3A21` beside the status chip), G10 and G11 (amount-081, amount-074) wrap trade and offer reference
  codes in `.amt`: Figtree with lining and tabular figures, inside a 13/400 `--text-secondary` caption.
- B: G0's Amount rule is about money: sign, colour by direction, exact digits, a unit.
- Build today: the Amount component is money only and cannot render a reference code.
- Recommended: keep Amount for money. Use the figure font (`font-num` plus tabular) as a plain text style for codes,
  which the `inline` Amount size in progress also covers if the owner wants one helper.

### 4. USDT unit beside or inside the figure

- A: G5 "My offers" (amount-unit-013) places the unit after `.amt` as a sibling: Poppins 13/500 `--muted-foreground`,
  line height normal, no numeric features.
- B: G7 "Affiliate" (amount-unit-014) places the unit inside `.amt`, so it inherits `lining-nums tabular-nums` and a
  26px line height.
- Visible difference: none. The unit has no digits, and the 19px box height is the same.
- Build today: the unit is its own span after the figure, as on G5.
- Recommended: keep it. Recorded only because the computed values differ.

### 5. Tertiary button label colour

- A, text and class: G0 section 9 item 3 says ".lnk, .crumbs a, .btn-tertiary, .chip-action and .av now use
  --action-pressed", and qt-v5.css `.btn-tertiary` is `--action-pressed`.
- B, drawn: G4 top bar back link "Trades" (button-589) and G7 band link "Notifications" (button-618) are
  `.btn-tertiary` with an inline `color: var(--foreground)` (#0F172A).
- Build today: the tertiary role (code variant `ghost`) is `--action-pressed` (#0073AD), hover and pressed
  `--action-subtle`.
- Recommended: keep `--action-pressed` for the shared tertiary button, since both the text and the class say so.
  The navy back link and band link are header and glass-band patterns owned by card 968. If the owner wants them
  navy, that card adds a back-link look.

### 6. Tertiary shapes drawn inline (same two instances)

- A, class: `.btn-tertiary` md is 44px high, padding 0 18px, gap 8px, no underline. sm is 36px, padding 0 14px.
- B, inline: G4 "Trades" uses padding 0 12px 0 6px and gap 2px (a chevron plus a word). G7 "Notifications" adds
  `text-decoration: underline; text-underline-offset: 3px`.
- Build today: class values.
- Recommended: same as item 5. These are the back link and band link shapes for 968, not tertiary button sizes.

### 7. Focus ring colour on navy

- A, text: G0 section 7 "Navy navigation: --focus-on-ink (8.33 ...)"; G17 control corrections, Confirmed:
  "--focus-on-ink #38BDF8 on navy navigation"; G17 accessibility row: "--focus-on-ink on navy".
- B, stylesheet: qt-v5.css and glass give buttons, checkboxes, radios and inputs on navy the general `--focus`
  #0369A1 ring (rows button-071/173/284/308/345/381/493/602, checkbox-009/010, radio-009/010, input-011). Only the
  dock and island classes use `--focus-on-ink`.
- No board draws a 970 control on navy. This shows only in the probe pages that put each control on a navy panel.
- Build today: anything inside `.qt-on-ink` gets `--focus-on-ink` (`globals.css`, `.qt-on-ink :focus-visible`).
- Recommended: keep the build, because it follows the Confirmed text. `--focus` on navy is 3.01:1, just at the 3:1 a
  focus ring needs, while `--focus-on-ink` is 8.33:1.

### 8. "Your turn" chip size

- A, class (G0 "Status badges", chip-096): 28px high, padding 0 10px, 13/16 600.
- B, inline (G4 "120.00 USDT to naledi_m", chip-097; G6 "Wallet" x2, chip-098): `height: 22px; padding: 0 8px; font-size: 12px`.
- Build today: StatusChip has one size, 28px.
- Recommended: owner choice. Either add a compact 22px size (12px text, 0 8px) for chips beside a label or inside a
  row, as G4 and G6 draw, or keep one size and treat the inline values as board drift. G4 and G6 draw it in a
  "next step" card and in a list row, so a compact size seems intended.

### 9. Chip as a control

- A: G0 and G4 chips are static labels (cursor auto).
- B: G6 draws the "Your turn" chip in a tappable row with cursor pointer (chip-098). The derived probe also has a
  `.chip.is-focus` state.
- Build today: f44c50dc ships a static chip. The builder is adding an `as="button"` form with cursor pointer, a
  focus ring and a 44px hit area.
- Recommended: a chip is interactive only when it does something itself. A chip inside a tappable row stays static,
  and the row takes the pointer and focus.

### 10. Square icon-only buttons drawn by inline size

- A, class: secondary lg is 52px high with padding 0 22px. secondary md is 44px with padding 0 18px.
- B, inline: G3 and G6 "Wallet" (button-442) draw a 52 x 52 square secondary button with `width: 52px; padding: 0`.
  G8 (button-463) draws a 44 x 44 open menu trigger with `padding: 0`, in the open state
  (`.gl-trigger[aria-expanded=true]`: 1.5px inset `--action-pressed` edge).
- Build today: Button `size="icon"` is 44 x 44 only. The builder is adding the icon-button look and the open-trigger
  state.
- Recommended: owner choice on a 52px square size (`icon-lg`). The 44px square already exists.

### 11. Field height inside a glass band

- A, class: `.input` is 48px high (also G0 and G10).
- B, inline: G9 Jump palette field (input-042) is `height: 52px`.
- Build today: 48px.
- Recommended: keep 48px for the shared field. The palette is card 968/969's surface, and it can size its own field
  if 52 is wanted.
- The 2px `--foreground` outline G9 also draws on that focused field is not a contradiction. It comes from glass
  v3.2.1's band focus rule, and it is being replicated.

### 12. Steps gap

- A, class: qt-v5.css `.steps` has no gap (0).
- B, drawn: every Steps instance on the boards adds `gap: 6px` inline (G0 section 7, G4), and so does every group
  board.
- Build today: 6px (fixed in f44c50dc).
- Recommended: keep 6px, and add `gap: 6px` to `.steps` in the pack stylesheet so the class and the drawings agree.

### 13. Checkbox and radio edge width

- A, class: qt-v5.css draws the off checkbox and radio with a 1.5px `--input-border` edge, and DESIGN.md says 1.5px.
- B, painted: Chrome renders that 1.5px edge as 1px at DPR 1, 2 and 3, on the boards and in the build. A plain
  `border: 1.5px` div does the same, so this is a browser limit, not a code choice.
- Build today: 1.5px declared, 1px painted, the same pixels as the board.
- Recommended: owner choice. Keep 1.5px and accept 1px on screen, or set 2px if a heavier edge is wanted. Either
  way the board and the build look the same.

### 14. Loading primary button fill

- What it is: a primary button in its loading state (ring plus the button's loading words) is drawn in two fills.
- A: C12 frame 28, "Publishing..." on the create-offer phone board, fills the loading primary with `--action-pressed`
  #0073AD (sampled from the PDF render).
- B: A21a "Starting trade..." and "Checking...", A21b "Releasing...", E1d frame C "Signing in...", D6a and D6b
  "Sending report..." fill it with `--action` #00A3F6.
- Found by text extraction of all 150 v5.2 PDFs plus OCR, then fill sampling. Crops:
  `/Users/starbird/Workspace/Code/qic/.scratch/970/evidence/replicate/fix/boards/loading-scan/`.
- Build today: `--action` (the button keeps its rest fill while `aria-busy`).
- Recommended and built (coordinator, 22 Oct): keep `--action`. It is the majority across groups A, D and E, and
  the shared rule keeps the CTA at #00A3F6. C12 frame 28 is the outlier.

## Checked and not counted

- Outline button label: G0 draws `--action-pressed` through the g31.css helper `.btn-outline`, and qt-v5.css has only
  `.btn-secondary` with a navy label. Settled in f44c50dc: `outline` uses `--action-pressed`, `secondary` stays navy.
- Hover at 390 and 768: the build draws no hover on touch widths (Tailwind v4 `hover:` sits inside
  `@media (hover: hover)`). Pressed still applies. The boards draw hover as a class, so they say nothing about touch.

## Defaults built until the owner answers (coordinator, 22 Oct)

1. Item 1: the `summary` Amount size uses the `.amt` features and tracking (lining and tabular, -0.01em) at
   14/600/20, so there is one money style everywhere.
2. Item 5: the tertiary label stays `--action-pressed`.
3. Item 8: owner decision (Marcello, 22 Oct), build both. StatusChip `size="compact"` (22px, 0 8px, 12/16) where
   G4, G6 and G7 draw it; 28px stays the default.
4. Item 10: owner decision (Marcello, 22 Oct), build both. Button `size="icon-lg"` (52 x 52, padding 0) as G3 and
   G6 draw it; 44px stays the default square.
5. Item 13: the checkbox and radio edge stays 1.5px as declared.
6. Items 2, 3, 4, 6, 7, 9, 11 and 12 follow the recommendation above.
7. Items 1, 5 and 13: owner decision (Marcello, 22 Oct), stay as built.
8. Item 14: keep `--action` (coordinator, 22 Oct).
