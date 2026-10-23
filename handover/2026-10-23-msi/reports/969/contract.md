# 969 BOIL builder contract (read in full before editing)

Ticket 969, "Customer redesign 906-0b: one dialog style and working notifications". Spec: `/Users/starbird/Workspace/Code/qic/.scratch/969/reports/PLAN.md` (Expected behaviour, Diagnosis, Scope, Implementation, Acceptance). The owner decisions below OVERRIDE the plan where they differ.

## Where

- Frontend worktree: `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root/frontend`, branch `969-dialogs-toasts`, base `origin/redesign/906` f44c50dc. Deps installed. Use absolute paths.
- Never touch `/Users/starbird/Workspace/Code/qic/frontend`. Never switch branches. No stash, hard reset, checkout-to-discard or clean (a hook blocks them). Do not commit unless your brief says so; the overseer commits.
- bun only (`bun run test -- <paths>`, `bun x tsc --noEmit`, `bun run lint`). No new npm dependency. Never symlink node_modules. Never run the full Jest suite.
- Write NO code comments in added or changed lines (no `//`, `/* */`, `{/* */}`). Meaning goes into names and tests.
- Do not edit `src/app/globals.css` unless your brief says so. Card 968 edits it in parallel and owns `--qt-overlay-bottom` and `html[data-bottom-bar]`; never define those.

## Owner decisions (binding)

1. Design source of truth is design pack v5.2 ARTBOARDS. Where an artboard differs from qt-v5.css or the plan, the artboard wins. Board markup (read these, they are short): `/Users/starbird/Workspace/Code/qic/.scratch/customer-app-redesign/research/design-pack/v5.2-src/shared/project/{tb-dialog,tb-confirm,ph-sheet,ph-toast,ph-release}.dc.html`. Stylesheets: same folder `glass-v3.1.css` (v3.2.1: sections 2, 5c, 5d, 5e, 5f) and `../qt-v5.css`. PDFs of boards G4 and G16b: `~/Downloads/906-design-pack-v5.2/906-v5.2-G4-ph-release.pdf`, `906-v5.2-G16b-content-zone-2.pdf`. Never use hosted claude.ai canvases.
2. Board B6's in-dialog toast outlet is REJECTED. No toast is ever drawn on a dialog or sheet.
   - A validation or error message raised while a customer dialog or sheet is open shows INSIDE that dialog: as a v5.2 field error under the field it concerns, or as a form-level error above the footer when no single field applies. `role="alert"`, shown immediately (as today).
   - A success or info toast raised while a customer dialog is open waits until the dialog closes (every open customer dialog), then shows in the page region with its full lifetime.
3. 970 (906-0a) is in the base. Reuse, do not duplicate: the `customer:` Tailwind variant (`html[data-surface=customer]`), v5.2 tokens (`--scrim`, `--radius-sheet`, `--shadow-dialog`, `--shadow-overlay`, `--action-subtle`, `--action-pressed`, `--dur-*`, `--ease-*`), glass classes in `globals.css` (`.gl .gl-dialog .gl-sheet .gl-solid .gl-band .is-top .is-bottom .gl-body .gl-handle .gl-title .gl-band-row .gl-plate .gl-toast`, scroll states `.is-scrolled .has-more`), and components Button, Field/FieldError (`src/components/ui/field.tsx`, FieldError renders role=alert with the v5.2 `.err` look on customer), Input, StatusChip, Amount, ErrorState, `src/lib/customer-classes.ts`. All glass rules are nested under `html[data-surface='customer']`, so `.gl*` class names are inert on staff and public pages.

## Shared API (builder A creates it; B and C consume it exactly)

`src/components/ui/dialog.tsx` keeps every existing export and its default-surface markup and class strings byte for byte, and adds:

```ts
export type DialogMaterial = "glass" | "solid"
export type DialogSize = "sm" | "md" | "lg" | "xl" | "2xl"

DialogContent props added (all optional):
  material?: DialogMaterial            default "glass"
  size?: DialogSize                    default "md"
  dismissible?: boolean                default true; false => Escape, outside pointer and interact-outside are prevented and the close button is hidden
  returnFocusRef?: React.RefObject<HTMLElement | null>   focused in onCloseAutoFocus when set (preventDefault the Radix default)
  role?: "dialog" | "alertdialog"      default "dialog"
  hideCloseButton?: boolean            existing

export function DialogShell(props: {
  open: boolean
  onClose: () => void
  title?: React.ReactNode               visually hidden fallback title when absent (requires accessibleTitle)
  accessibleTitle?: string              used when title is absent or not a string
  description?: React.ReactNode
  size?: DialogSize
  material?: DialogMaterial
  dismissible?: boolean
  role?: "dialog" | "alertdialog"
  returnFocusRef?: React.RefObject<HTMLElement | null>
  hideCloseButton?: boolean
  className?: string                    applied to the content panel (default surface: same role as Modal's className)
  children: React.ReactNode
}): JSX.Element

export function DialogFormError(): JSX.Element | null   renders the current form-level error of the enclosing dialog (FieldError look, role="alert"); DialogFooter renders it automatically above its buttons, and the dialog renders it at the end of the body when no DialogFooter is mounted.
```

- `DialogFooter` keeps its API; on the customer surface it is the actions row at the end of the body (boards put actions at the end of `.gl-body`, no bottom band).
- `src/components/ui/dialog-stack.ts` (new, no React import beyond hooks, no import of ui/toast): module-level stack of open dialog entries `{ id, customer: boolean }` with `subscribe/getSnapshot`, `useDialogStackDepth()`, `isCustomerDialogOpen()`, `topCustomerDialogId()`. DialogContent registers while open. A content opened while another is open has depth > 0 and renders solid (never glass on glass).
- Toast routing lives in the store layer (`src/store/ui-store.ts` + `src/components/ui/toast.tsx`): `addNotification` on the customer surface, when a customer dialog is open: `error` and `warning` are tagged with the top dialog id and rendered by that dialog as its form-level error (latest one wins; cleared when that dialog closes or a success/info is raised while it is the top dialog); `success` and `info` are held and released to the page region when no customer dialog is open. Off the customer surface nothing is routed or held.
- Call sites keep calling `useToast().toast.error(message)` for non-field errors. Tests that `jest.mock("@/components/ui/toast")` must keep working: dialog.tsx must not import `ui/toast`.

## Customer dialog look (artboards)

- Panel at >= 640 px: `section.gl.gl-dialog` (role dialog or alertdialog, aria-modal), centred, width by size: sm 440, md 480, lg 520, xl 576, 2xl 672 (tb-confirm is 440, tb-dialog "Edit offer" is 520), max-height calc(100dvh - 48px), the layer scrolls (bands sticky). Header `div.gl-band.is-top` holds `h2.gl-title` and a 44 px `icon-btn` close (aria-label "Close"). Body `div.gl-body` holds content then the actions row.
- Below 640 px: `section.gl.gl-sheet`, fixed to the viewport bottom (force `position: fixed`; `.gl-sheet` sets absolute), full width, 20 px top radius, `max-height: 90dvh`, own scroll, `overscroll-behavior: contain`, header band = `span.gl-handle` + `div.gl-band-row` (title + close).
- Solid: add `gl-solid` (one opaque --action-subtle pane, no line, glass 5f). Solid when `material="solid"` or stack depth > 0.
- Scrim: Radix overlay on customer = `background: var(--scrim)`, no backdrop-filter, fade by `--dur-base`.
- Motion: as in glass section 2 (sheet translateY in 280 ms ease-out, out 180 ms ease-in; dialog opacity+scale .98; solid opacity only), reduced motion opacity only 150 ms linear (already in globals.css via data-state).
- Field errors: FieldError under the field with the input `aria-invalid="true"` and `aria-describedby` pointing at the error id. Form-level error: `DialogFormError` above the actions row.

## Customer toast look (ph-toast, glass 5c and section 2)

- `.gl-toast` pane (opaque --action-subtle, 1 px white edge, --shadow-overlay, 16 px radius, --foreground text, status icon in its status colour, description in `.muted`, 44 px dismiss `icon-btn` aria-label "Dismiss"), no backdrop-filter, no progress bar on customer unless the board draws one (it does not).
- Container: `bottom: var(--qt-overlay-bottom, 24px)` (24 px equals today's offset), left/right 12 px on customer, max-width 440 centred. One toast at a time on the customer surface (5c): queue the rest, oldest first.
- Roles: two persistent regions mounted even when empty: polite (`role="status"` items for success, info, warning) and assertive (`role="alert"` items for error). Remove the contradictory role=alert + aria-live=polite pair.
- Timer: default 6000 ms; explicit durations kept. Pause on mouseenter and focusin, resume on mouseleave and focusout with the remaining time.
- Motion: in 200 ms ease-out translateY(8px)+opacity; out 180 ms ease-in opacity; reduced motion opacity only 150 ms linear, no moving bar.
- Staff and public: today's dark look (class strings kept), but the new timing, roles, pause and motion.
- `data-testid="qt-toast"` and `data-type="<type>"` on each toast item.

## Money-dialog invariants (non-negotiable)

- Every `useWithdraw`, `useTransfer`, `useLockFunds`, quote state (heldTronQuote, quoteSessionId, quoteHoldAttemptRef), step-up state (show2faPrompt, showEmailPrompt, codes, pendingAction), step state and ref stays in the component body ABOVE `DialogShell`. Never move a hook or state into a component rendered inside DialogContent.
- Same requests, same fields, same messages. A double click still sends one request.
- ticket-645-wallet-manage-modals source shapes stay exactly: `<NetworkDropdown`, `onChange={(symbol) => { setChosenRowSymbol(symbol)`, `getVisibleWithdrawalCurrencies(solanaEnabled)`, the USDT rail objects, the manage page's setActiveTransactionModal calls.
- WithdrawModal solid from its first frame. TransferModal glass on 'form', solid from 'confirm' through 'verify' and 'success' (latch, reset only when isOpen turns false, never back to glass on Back). LockFundsModal glass on 'select', solid from 'confirm' (same latch). TransactionStatusModal and the 819 notice solid with role alertdialog. Deposit glass with the address and QR on `.gl-plate`. While a money request is in flight, `dismissible={false}`.

## Tests

- Suites that read `container` from a dialog must switch to `baseElement` with no `expect()` change.
- Where the owner decision turns a toast into an inline field error, update the assertion to find the same message string as an inline `role="alert"` in the dialog; never weaken or delete an assertion; list every such change in your return.
- Targeted runs only: `bun run test -- <paths>`.

## Return format (at most 25 lines)

Files changed with +/- counts, commands run with pass/fail counts and log paths, every test assertion you changed (file:line, before, after, why), deviations from this contract, open risks.

## Addendum, 22 Oct 2026: replicate the boards (owner rule, tightens everything above)

The build must REPLICATE the v5.2 boards, not interpret them. Every value a board shows is the spec: colours, sizes, spacing, radius, type (family, size, weight, line height), borders, shadows, focus and error rings, every state, and copy of the shell (title row, close button, handle, scrim, actions row, field and form errors, toast pane, dismiss button). Boards for 969: G16b (content-zone-2), G4 (ph-release), tb-dialog, tb-confirm, ph-sheet, ph-toast, plus the shell of any dialog, sheet or toast drawn on other boards. Editable sources: `/Users/starbird/Workspace/Code/qic/.scratch/customer-app-redesign/research/design-pack/v5.2-src/shared/project/*.dc.html` (they load `qt.css`, `glass-v3.1.css` and `g31.css` from that folder); PDFs: `~/Downloads/906-design-pack-v5.2/`. When a board's markup uses a class (`.icon-btn`, `.btn`, `.err`, `.field`, `.label`, `.input`, `.kv`, `.well`, `.note`, `.gl-title`), the implemented element must compute the same styles as that class does in the board, not an approximation. Page content inside wallet, offer and trade dialogs belongs to group cards 956 to 959; 969 replicates the shell and the elements this card adds (field and form errors, plates, toasts). 970 (f44c50dc) already fixed the glass first-child field focus ring and the outline-button label. The overseer renders board and implementation at the same size and state and compares side by side and by computed style; any difference is fixed. Only a difference impossible in code, or contradicted between boards, is recorded for the owner with board ids and values.
