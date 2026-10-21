# QicTrader Design System: Daylight

Source of truth for the customer look of QicTrader. Daylight is the single light theme of design pack v5.2
(qt-v5.css v5.2 and glass v3.2.1, 21 Oct 2026). Every value below is the value shipped in
`frontend/src/app/globals.css`. `frontend/src/__tests__/design/design-md-tokens.test.ts` compares this file
with that CSS in both directions, so change both together.

## 1. Scope

1. Daylight applies only on the customer surface: `html[data-surface='customer']`. The head script in
   `app/layout.tsx` and `ThemeProvider` set that attribute and the class `light` before first paint.
2. Which routes are customer routes is decided by `CUSTOMER_SURFACE_RULES` and `isCustomerSurface()` in
   `frontend/src/config/routes.ts`. That list is the only definition. Do not repeat it elsewhere.
3. On the customer surface there is no dark theme and no theme toggle. A stored dark preference stays in
   `ui-storage` for the pages that honour it.
4. Staff, admin, moderator, public, landing, auth and KYC pages still use the April system (`:root` and `.dark` in
   globals.css, their stored theme and their toggle) until they are redesigned. Nothing here changes them. The
   April document stays on root `main` until the 906 release.
5. Shared components keep their default class strings byte for byte and add `customer:` variant classes. The variant
   is `@custom-variant customer (&:where([data-surface=customer] *))`, with zero added specificity.

## 2. Colour tokens

Use the Tailwind keys (`bg-action`, `text-money-in`, `outline-focus` and so on). No raw hex in components.

| Token | Value | Role |
|-------|-------|------|
| --background | #F8FAFC | Page ground. |
| --card | #FFFFFF | Content cards and opaque panels; glass plates for QR codes, addresses and amounts. |
| --popover | #FFFFFF | Menus and popovers on paper; any layer opened from inside glass. |
| --surface | #F1F5F9 | Wells, tracks, static skeletons and disabled fills; the neutral Cancelled chip. |
| --surface-hover | #E2E8F0 | Row and pill hover on paper. |
| --border | #E2E8F0 | Decorative dividers and card outlines only. Never a control edge. |
| --input-border | #7B8A9C | Edges of inputs, checkboxes, radios, outline buttons and selects (3:1). |
| --ink-surface | #0F172A | Navy navigation layer and active pills. |
| --ink-surface-raised | #1E293B | Hover and pressed on navy. |
| --scrim | rgba(15,23,42,.48) | Backdrop behind dialogs and sheets. |
| --foreground | #0F172A | Headings, body text and figures; links, focus rings and active icons on the glass frost. |
| --muted-foreground | #475569 | Labels, units beside figures and fiat estimates. |
| --text-secondary | #556274 | Captions, timestamps, help text and upcoming step labels. |
| --text-placeholder | #5F6D80 | Input placeholders. |
| --text-on-ink | #FFFFFF | Active labels on navy. |
| --text-on-ink-muted | #A3B1C2 | Inactive labels on navy. |
| --brand-blue | #00A3F6 | Logo and active navigation marker only. Decorative, never text, never the only cue. |
| --brand-blue-light | #38BDF8 | Light cyan brand accent, the same value as --focus-on-ink. |
| --action | #00A3F6 | Primary button fill only, with a white label. Never text. |
| --action-hover | #0284C7 | Primary button hover fill. |
| --action-pressed | #0073AD | Pressed primary; links and outline labels; ticked checkbox, selected radio and current step fills. |
| --action-foreground | #FFFFFF | Label on primary and danger fills; ticks and dots. |
| --action-subtle | #E6F6FF | Selected rows, tertiary hover, action chip and avatars; the glass stain. |
| --success | #047857 | Completed and credited; done steps. |
| --success-bg | #ECFDF5 | Success message and chip background. |
| --warning | #92400E | Time running out and pending checks. |
| --warning-bg | #FFFBEB | Warning message and chip background. |
| --error | #B91C1C | Errors, invalid field edges and danger buttons. |
| --error-hover | #991B1B | Danger button hover. |
| --error-pressed | #7F1D1D | Danger button pressed; destructive rows on the glass frost. |
| --error-bg | #FEF2F2 | Error message and chip background; danger-quiet hover. |
| --destructive | #B91C1C | The --error value under its shadcn name. |
| --info | #0073AD | Neutral facts and network notes; held and Payment sent chips. |
| --info-bg | #E6F6FF | Information message and chip background. |
| --attention | #F59E0B | Your turn chip and count badges only, with navy text. Never text on paper. |
| --attention-foreground | #0F172A | Text and counts on --attention. |
| --money-in | #047857 | Confirmed incoming amounts, always with a plus sign. |
| --money-out | #0F172A | Outgoing amounts, navy with a minus sign. Never red. |
| --usdt | #26A17B | The USDT token mark only. Never text, never status. |
| --focus | #0369A1 | Focus ring and focused field edge on opaque light surfaces, opaque commit steps and plates. |
| --focus-on-ink | #38BDF8 | Focus ring on navy navigation. |

Legacy shadcn and April aliases (`--primary`, `--ring`, `--input`, `--muted`, `--secondary`, `--brand-blue-bg` and
the rest) are remapped to these roles inside the customer block so older components pick up Daylight. New customer
code uses the tokens above, never the aliases.

## 3. Type

Poppins is the face for all text: headings, body, labels, buttons and the units beside figures. Figtree is used only
for the figures inside the shared Amount component (`ui/amount.tsx`, utility `font-num`), with tabular lining
numerals (`tabular-nums lining-nums`). The QicTrader name and logo are unchanged.

| Token | Value | Use |
|-------|-------|-----|
| --font-sans | var(--font-poppins), sans-serif | All text. |
| --font-num | var(--font-figtree), var(--font-poppins), sans-serif | Figures inside the Amount component only. |

qt-v5.css names the faces directly: `'Poppins',sans-serif`, and `'Figtree','Poppins',sans-serif` for Amount figures.
The app loads both through next/font, so the shipped values point at the `--font-poppins` and `--font-figtree`
variables of the same faces in the same order, and Figtree stays limited to Amount figures.

| Class | Size / line | Weight | Use |
|-------|-------------|--------|-----|
| t-display | 32/40, tracking -.02em | 600 | Page hero, desktop |
| t-h1d | 28/36, tracking -.015em | 600 | Page title, desktop |
| t-h1 | 24/30, tracking -.01em | 600 | Page title, phone |
| t-h2 | 20/28 | 600 | Section heading |
| t-h3 | 16/24 | 600 | Card and state titles |
| t-body | 16/24 | 400 | Body text; inputs are 16 px |
| t-body-sm | 14/20 | 400 | Supporting text |
| t-label | 14/20 | 500 | Field labels |
| t-caption | 13/18, --text-secondary | 400 | Help text and step labels |
| t-over | 12/16, uppercase, tracking .06em, --text-secondary | 600 | Overlines and table headers |

| Amount size | Figures | Weight | Unit |
|-------------|---------|--------|------|
| amt-xl | 40/44, tracking -.02em | 600 | 18 px |
| amt-lg | 28/34, tracking -.015em | 600 | 15 px |
| amt-md | 20/26 | 600 | 13 px |
| amt-sm | 16/22 | 600 | 12 px |
| amt-xs | 14/20 | 500 | 12 px |

The unit is Poppins 500 in --muted-foreground.

## 4. Radius and elevation

| Token | Value | Use |
|-------|-------|-----|
| --radius-control | 12px | Buttons, inputs, selects and plates (`rounded-control`). |
| --radius-card | 16px | Cards, panels, popovers, menus and toasts (`rounded-card`). |
| --radius-sheet | 20px | Dialogs and phone sheets (`rounded-sheet`). |
| --radius-pill | 9999px | Chips, pills, the dock and the island (`rounded-full`). |
| --radius | 0.625rem | Legacy 10 px base behind rounded-sm to rounded-xl and glass menu rows. |

Component-internal radii: checkbox 6 px, skeleton blocks 8 px.

| Token | Value | Use |
|-------|-------|-----|
| --shadow-dock | 0 10px 28px rgba(15,23,42,.22),0 2px 6px rgba(15,23,42,.14) | Dock and island (`shadow-dock`). |
| --shadow-overlay | 0 12px 32px rgba(15,23,42,.16),0 2px 8px rgba(15,23,42,.08) | Popovers, menus and toasts (`shadow-overlay`). |
| --shadow-dialog | 0 24px 64px rgba(15,23,42,.24),0 4px 12px rgba(15,23,42,.08) | Dialogs and sheets (`shadow-dialog`). |

Shadows are for floating layers only. Cards on paper use a --border outline, not a shadow.

## 5. Spacing

A 4 px base, the Tailwind spacing unit (globals.css does not override `--spacing`). The Tailwind column is the
utility step (`p-4`, `gap-3`, `px-3.5`).

| Value | Tailwind | Kind | Use |
|-------|----------|------|-----|
| 4 | 1 | scale | Icon to text, tight stacks |
| 8 | 2 | scale | Button gap, chip rows |
| 12 | 3 | scale | Row gap, list items |
| 16 | 4 | scale | Phone gutter, card stacks |
| 20 | 5 | scale | Phone card padding |
| 24 | 6 | scale | Desktop card padding, phone section gap |
| 32 | 8 | scale | Desktop gutter and section gap |
| 40 | 10 | scale | Large separations |
| 48 | 12 | scale | Page top and bottom |
| 6 | 1.5 | internal | Chip gap, field label gap, menu padding |
| 10 | 2.5 | internal | Input icon gap, group label padding |
| 14 | 3.5 | internal | Input side padding, sheet body gap |

| Layout | Phone | Desktop |
|--------|-------|---------|
| Page gutter | 16 | 32 |
| Card padding | 20 | 24 |
| Section gap | 24 | 32 |

Internal steps stay inside components. Layout spacing uses the scale only.

## 6. Hit areas

Every pointer target is at least 44 x 44 px. The global rule in globals.css gives `button`, `a` and
`[role=button]` a 44 px minimum; a control that is smaller on purpose opts out (`no-min-touch`, or
`customer:min-h-0 customer:min-w-0` inside a component) and extends its hit area around its centre instead
(`.hit44`, or the same `::after` or `::before` extender in the component).

| Target | Visual | Hit area | How |
|--------|--------|----------|-----|
| Button, default | 44 | 44 | Its own height |
| Button, large | 52 | 52 | Its own height |
| Button, small | 36 | 44 | Centred ::after extender |
| Icon button | 44 | 44 | Its own size |
| Checkbox | 22 | 44 | customer:min-h-0 and customer:min-w-0, and a centred ::before extender |
| Radio | 22 | 44 | The wrapping label row |
| Glass menu row | 44 | 44 | min-height on .gl-mi |
| Section pill | 40 | 44 | .hit44 |
| Status chip | 28 | 44 | .hit44 when it is interactive |

Keep 44 px between the centres of two extended targets, and never extend inside an `overflow:hidden` box narrower
than 44 px.

## 7. Controls

Every shared control keeps its public API, adds a stable `data-testid` before `{...props}` so a caller's own value
wins, and takes the v5.2 look only on the customer surface.

| Variant | Button prop | Default | Hover | Pressed |
|---------|-------------|---------|-------|---------|
| Primary | default | --action, white label | --action-hover | --action-pressed, scale .98 |
| Secondary | outline | --card, --input-border edge, --action-pressed label | --surface | --surface, scale .98 |
| Secondary | secondary | --card, --input-border edge, --foreground label | --surface | --surface-hover, scale .98 |
| Tertiary | ghost, link | transparent, --action-pressed label | --action-subtle | --action-subtle, scale .98 |
| Danger | destructive | --error, white label | --error-hover | --error-pressed, scale .98 |
| Danger quiet | danger-quiet | transparent, --error label | --error-bg | --error-bg |

1. Buttons (`qt-button`): 44 px default, 52 px large, 36 px small with a 44 px hit area, 44 px icon. 600 weight,
   --radius-control, press scale over `duration-press`. Disabled is --surface with --text-secondary and no opacity
   fade. Loading keeps the button width and shows a spinner.
2. Inputs (`qt-input`): 48 px, --input-border edge, --card fill, 16 px text, --text-placeholder. Focus is a 1 px edge
   plus a 1 px ring in --focus. Error is --error with a message and icon (`qt-field-error`, role alert). Disabled is
   --surface with a --border edge.
3. Checkbox (`qt-checkbox`) and radio (`qt-radio`): 22 px, 1.5 px --input-border edge on --card. On is an
   --action-pressed fill with a white tick or an 8 px white dot. Disabled and selected is a --border fill with a
   --text-secondary mark.
4. Stepper (`qt-step`): done steps --success, the current step --action-pressed with `aria-current="step"` and a 600
   label, upcoming steps --border. Segments sit 6 px apart. Labels are text only, 13/18, with no tick. State is also
   given by `aria-current` and a hidden "completed" suffix on done steps.
5. Selected fills and the current step always use --action-pressed.
6. Focus is a 2 px ring with a 2 px offset. Its colour depends on the surface:

| Surface | Focus colour |
|---------|--------------|
| Opaque light surfaces, opaque commit steps (.gl-solid), plates | --focus |
| Glass layers (.gl), bands and content zones | --foreground |
| Navy navigation (--ink-surface, .qt-on-ink) | --focus-on-ink |

7. Never paint a ticked checkbox, selected radio, current step, progress bar, focused field edge or focus ring in
   --action or --brand-blue (G14 section 5). --action is the primary fill only.

## 8. Glass

Glass v3.2.1 classes (`.gl`, `.gl-band`, `.gl-body`, `.gl-plate`, `.gl-solid`, `.gl-sheet`, `.gl-dialog`,
`.gl-popover`, `.gl-menu`, `.gl-mi`, `.gl-toast`, `.hit44`) live under the customer surface only.

1. Glass is for navigation and floating layers only: dock, island, sheets, dialogs, popovers and menus. Pages,
   cards and forms are opaque paper.
2. One material: blur 16 px (never more), saturate 1.5, an --action-subtle stain, a 1 px white edge and a lead line.
3. Text on the frost (`.gl-band`) is --foreground only, with links underlined. Forms, amounts and status sit in the
   content zone (`.gl-body`).
4. QR codes, deposit addresses and amounts to pay sit on an opaque `.gl-plate`.
5. Irreversible steps (release, I've paid, cancel, dispute, withdrawal review, send, start a trade, delete) are
   opaque `.gl-solid` from the commit step until the layer closes. Toasts are opaque too.
6. Never glass on glass. A layer opened from a glass layer is paper. One glass layer on a phone at rest (the dock).
7. Fallbacks, all to one opaque look (--action-subtle pane, --input-border edge, --card content):
   no `backdrop-filter` support, `prefers-reduced-transparency: reduce`, `prefers-contrast: more` (adds 1 px
   --input-border edges), `forced-colors: active` (Canvas and CanvasText, Highlight for the active row), and
   `data-glass="lite"` on html for slow phones (blur 8 px, stain 6 points stronger).
8. Blur, saturate and stain never animate.

## 9. Motion

| Token | Value | Utility | Use |
|-------|-------|---------|-----|
| --dur-press | 100ms | duration-press | Press scale, tick and dot in |
| --dur-quick | 150ms | duration-quick | Menus, popovers, selects; any opacity-only fade |
| --dur-base | 200ms | duration-base | Dialog in, progress fill, toast in |
| --dur-sheet | 280ms | duration-sheet | Phone sheet up |
| --dur-exit | 180ms | duration-exit | Dialog, sheet and toast out |
| --ease-out | cubic-bezier(.2,0,0,1) | ease-qt-out | Everything that enters or responds |
| --ease-in | cubic-bezier(.4,0,1,1) | ease-qt-in | Everything that leaves |

1. Only opacity and transform animate. Colour, border, shadow and focus rings change in the same frame.
2. Amounts, balances and status chips never animate; they replace in place.
3. The spinner (800 ms per turn, linear) is the only loop on the customer surface. Customer skeletons are static.
4. The theme change never animates (`qt-theme-swap` and the customer block turn the html transition off).

With `prefers-reduced-motion: reduce`:

1. Decorative animation, pulse, ping and skeleton shimmer stop (`animate-pulse`, `animate-ping`, `animate-spin`,
   `animate-shimmer` get `animation: none`; everything else ends at once in 0.01 ms with its end state kept).
2. Spinners become static text: the icon is hidden (`qt-motion-only`) and the label or "Loading..." shows
   (`qt-reduced-motion-only`). Status and progress text stay visible.
3. Layers fade only: opacity cross-fades are capped at 150 ms, with no slide and no scale.
4. Entry animations (`animate-in`, `fade-in`, `slide-in-from-*`, the tick and dot) jump to their end state, so
   nothing is left invisible.

## 10. Money and status

One rule for every amount, through the shared `Amount` component (`qt-amount`, `data-direction`).

1. A confirmed 120.00 USDT deposit shows `+120.00 USDT` in --money-in green.
2. A 121.20 USDT trade lock shows `−121.20 USDT` in --money-out navy, never red.
3. Held funds are a fact, not a direction: navy, no sign.
4. A balance of 1234.567891 USDT shows `1,234.567891 USDT`. Amount never rounds and never drops a decimal; it pads to
   at least two decimals. It never runs a server amount through parseFloat or Number for display.
5. A value that is loading shows a skeleton. A value that is unavailable shows `Unavailable`, never 0.00 or 0.
6. A cancelled trade shows a neutral `Cancelled` chip (--muted-foreground on --surface), never red.
7. Every status chip (`qt-status-chip`, `data-tone`) has a text label, with an optional icon or dot, so colour never
   carries meaning alone. Tones: success, warning, error, info, neutral, action and turn (--attention).

## 11. Anti-patterns

1. No raw hex or rgb in components. Use the tokens and their Tailwind keys.
2. No fonts other than Poppins, and Figtree only inside Amount.
3. No --action or --brand-blue as text, as an indicator or as a focus ring.
4. No red for outgoing money or for a cancelled outcome.
5. No 0.00 standing in for a missing or failed value.
6. No radius outside the table in section 4.
7. No shadow on paper cards, and no coloured shadows.
8. No glass on pages, cards or forms, and never glass on glass.
9. No animated colour, blur or filter, and no infinite animation other than the spinner.
10. No v5.2 class outside the `customer:` variant on a shared component.

## 12. Contrast exception

The CTA blue #00A3F6 with a white label is the only contrast exception in the palette. Kept as approved by Christian
(21 Oct 2026); limitation documented.

| State | Fill | Label | Ratio | Result |
|-------|------|-------|-------|--------|
| Default | --action #00A3F6 | white #FFFFFF | 2.77:1 | Fails 4.5:1 for text and 3:1 for large text |
| Hover | --action-hover #0284C7 | white #FFFFFF | 4.10:1 | Fails 4.5:1 for text |
| Pressed | --action-pressed #0073AD | white #FFFFFF | 5.18:1 | Passes 4.5:1 |

While this stands, the palette is not AA compliant as a whole. Every other text pair passes 4.5:1, and every control
edge, indicator and focus ring passes 3:1 (`contrast.test.ts`).

## 13. Files

| File | Contents |
|------|----------|
| `frontend/src/app/globals.css` | Customer block, Tailwind theme keys, motion utilities, glass, reduced motion |
| `frontend/src/config/routes.ts` | `CUSTOMER_SURFACE_RULES` and `isCustomerSurface()` |
| `frontend/src/app/layout.tsx` | Poppins, Figtree for Amount figures, the before-paint surface script |
| `frontend/src/components/ui/` | Button, Input, Field, Checkbox, RadioGroup, Progress and Steps, Spinner, Amount, StatusChip, EmptyState, ErrorState, Skeleton |
| `frontend/src/__tests__/design/` | Jest checks for this document, contrast, reduced motion and the controls |
