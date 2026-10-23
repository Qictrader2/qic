# 970 builder contract (shared by every BOIL worker)

Ticket 970, "Customer redesign 906-0a: Daylight colours, motion and shared controls". You are one of several builders
working in parallel on file-disjoint slices of the same worktree. Read this whole file before you edit anything.

## Where

- Frontend worktree (all frontend code): `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root/frontend`
  (branch `970-daylight-tokens-controls`, base `origin/redesign/906` = 04fed798). Dependencies are installed.
- Root worktree (only DESIGN.md): `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/970-root`.
- NEVER touch `/Users/starbird/Workspace/Code/qic/frontend` (someone else's uncommitted work).
- Git: read-only commands only (status, diff, log, show). No commit, no branch switch, no stash, no reset, no
  checkout/restore of files, no clean. The orchestrator commits.
- Only edit the files your slice owns. If you believe a file outside your slice must change, do not edit it; say so
  in your final report with the exact change needed.

## Specs, in priority order

1. Design pack v5.2 (wins over everything below where they differ; report each difference):
   `/Users/starbird/Workspace/Code/qic/.scratch/customer-app-redesign/research/design-pack/v5.2-src/shared/qt-v5.css`
   (tokens and control styles, copy values word for word), `.../shared/glass-v3.1.css` (glass v3.2.1),
   `.../shared/project/glass.css` (defines --glass-frost-stain and --glass-frost-lead, needed by glass v3.2.1),
   `.../shared/design-notes.md`. Plain-text extracts of the v5.2 boards: `/Users/starbird/Workspace/Code/qic/.scratch/970/sources/`
   (`tokens-v5.txt` = G0 token sheet, `Main.txt` = G1 overview, `apply.txt` = G14 apply incl. section 5, `status-v5.txt` = G17).
   G14 section 5 rule: never paint a ticked checkbox, selected radio, current step, progress bar, focused field edge
   or focus ring in --action or --brand-blue.
2. The pinned plan: `/Users/starbird/Workspace/Code/qic/.scratch/970/reports/PLAN.md` (read sections "Expected
   behaviour", "Implementation" step(s) for your slice, "Risks", "Acceptance and evidence"). Ignore any "QA on a local
   build" wording.
3. The owner-approved design package v1 (21 Oct 2026):
   `/Users/starbird/Workspace/Code/qic/.scratch/ticket-preparation/runtime/cards/6ad887d109cbe36bd6fee854/8f39f58463a31d4f6d5f/`
   `design-notes.md` (section "Design decisions for the builder" is binding where the card leaves a detail open),
   `boards/b*.html` (copy and layout per board), `exports/*.png` (renders), `src/base.css`.

## Hard rules

- Write NO code comments of any kind in lines you add: no `//`, `/* */`, `{/* */}`, `/** */`, `<!-- -->`, and no CSS
  comments. Meaning goes into names, types and test names. Do not delete comments on lines you are not otherwise changing.
- Staff, public and excluded pages (admin, moderator, landing / and /lp, help, FAQ, legal, contact-us, bug-report,
  /escrow, /whatsapp, /verify-email*, OAuth callback, 2FA dialogs, KYC pages, auth pages) must not change look or
  behaviour. Every v5.2 look applies ONLY under the customer surface. For shared components that means: keep today's
  class strings byte for byte and append `customer:` variant classes; never edit or reorder today's classes.
- No new npm dependency. bun only (never npm, npx, yarn, pnpm). Never symlink node_modules. Do not run `next build`
  or `bun run build` or start a server (the orchestrator does that once). Never run the full Jest suite.
- Run your targeted tests with `cd <frontend worktree> && bun run test -- <paths>`. Run `bun run typecheck` at the end
  and fix every error in files you own (errors in other slices' files may be in-flight; list them, do not fix them).
  Run `bunx eslint <your files>` and fix new errors in your files.
- Tests: assert exact values (no `toBeDefined()` alone, no `not.toBe(...)` where a positive assertion is possible),
  never skip or weaken an existing assertion, test names state the invariant they guard.
- Money display: never round, never parseFloat/Number() a server amount for display, never invent 0 or 0.00.
- Write no report or summary files. Your final message is your report.

## Shared names every slice relies on (do not rename)

- Surface marker: `document.documentElement` gets `data-surface="customer"` and class `light` (no `dark`) on customer
  routes. Classifier: `isCustomerSurface(pathname: string, opts?: { signedIn?: boolean }): boolean` exported from
  `src/config/routes.ts`, plus a serialisable rule list `CUSTOMER_SURFACE_RULES` used by the head script.
- Tailwind variant (defined in globals.css): `@custom-variant customer (&:where([data-surface=customer] *));`
  Use it as `customer:<utility>`, also stacked: `customer:hover:bg-action-hover`, `customer:focus-visible:outline-focus`,
  `customer:disabled:bg-surface`, `customer:aria-invalid:border-error`.
- Theme colour keys available to Tailwind (so `bg-*`, `text-*`, `border-*`, `outline-*`, `ring-*`, `fill-*` work):
  action, action-hover, action-pressed, action-foreground, action-subtle, ink-surface, ink-surface-raised, text-on-ink,
  text-on-ink-muted, money-in, money-out, focus, focus-on-ink, attention, attention-foreground, error, error-hover,
  error-pressed, error-bg, destructive, success, success-bg, warning, warning-bg, info, info-bg, surface, surface-hover,
  border, input-border, card, popover, background, foreground, muted-foreground, text-secondary, text-placeholder, usdt.
- Radius utilities: `rounded-control` (12px), `rounded-card` (16px), `rounded-sheet` (20px).
- Font utility: `font-num` (Figtree via next/font variable `--font-figtree`, then Poppins). Use it only inside Amount figures.
- Shadow utilities: `shadow-dock`, `shadow-overlay`, `shadow-dialog`.
- Motion utilities: `duration-press` (100ms), `duration-quick` (150ms), `duration-base` (200ms), `duration-sheet`
  (280ms), `duration-exit` (180ms); easings `ease-qt-out` cubic-bezier(.2,0,0,1) and `ease-qt-in` cubic-bezier(.4,0,1,1).
- Global helper classes (globals.css): `qt-motion-only` (display:none under prefers-reduced-motion: reduce),
  `qt-reduced-motion-only` (display:none normally, shown under reduced motion), `hit44` (customer-scoped 44x44 hit area
  via ::after, as glass v3.2.1 section 4), existing `no-min-touch` (opts out of the global 44px min-height on
  button/a/[role=button]), `qt-theme-swap` (on html: disables transitions while the theme flips).
- Glass classes ported from glass v3.2.1 and scoped to the customer surface: `.gl`, `.gl-band`, `.gl-body`,
  `.gl-plate`, `.gl-solid`, `.gl-sheet`, `.gl-dialog`, `.gl-popover`, `.gl-menu`, `.gl-mi`, `.gl-toast`, `.hit44`.
- data-testids (set as a default BEFORE `{...props}` so a caller's own data-testid still wins): `qt-button`,
  `qt-input`, `qt-field-error`, `qt-checkbox`, `qt-radio`, `qt-step`, `qt-spinner`, `qt-amount` (+ `data-direction`),
  `qt-status-chip` (+ `data-tone`), `qt-error`, `qt-empty`, `qt-skeleton`.
- Specificity: `customer:` utilities have the specificity of one class (`:where` adds zero). A base `hover:` or
  `focus-visible:` utility therefore beats a bare `customer:` utility in that state; always pair them with
  `customer:hover:...`, `customer:focus-visible:...`, `customer:disabled:...` as needed.
