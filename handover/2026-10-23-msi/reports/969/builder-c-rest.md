# Builder C: offers, direct trade, hand-rolled dialogs, sonner removal (969)

Read `/Users/starbird/Workspace/Code/qic/.scratch/969/briefs/contract.md` in full first (binding). Then PLAN.md "Expected behaviour" 5, 7, 9, "Diagnosis" 2, 6, 10, "Implementation" steps 5, 7, 8 and 9e/9f, and the final dialog.tsx/toast.tsx API builder A produced (read the files; do not edit them).

Worktree `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-root/frontend`. Do not commit. You own only:
- `src/components/features/offers/{EditTradeLimitModal,EditOfferRequirementsModal,EditOfferMarginModal}.tsx`
- `src/components/features/direct-trade/{UserSearchModal,CreateDirectOfferModal}.tsx`
- `src/components/features/escrow/KYCRequirementModal.tsx`, `src/app/(main)/offer/resell/_components/EscrowModal.tsx`
- `src/components/reports/ReportUserModal.tsx`, `src/components/profile/AffiliateDashboard.tsx`, `src/components/features/trade/TradeReviewForm.tsx`
- `package.json`, `bun.lock`, `package-lock.json` (sonner entries only), `e2e/pages/report.page.ts`
- tests for these files and the new suites below.
Builder B owns the wallet dialogs in parallel; do not touch `src/components/features/wallet/*` or `src/components/ui/*`.

Do:
1. Offers and direct trade: replace Modal/ModalFooter with a glass DialogShell/DialogFooter. Sizes from the boards: the three offer edit dialogs `size="lg"` (tb-dialog "Edit offer" is 520 px); others keep their current relative size mapping. Mutations stay in the component bodies. CreateDirectOfferModal keeps reading its open state from its store and stays mounted unconditionally at PublicProfileView.
2. Owner decision on in-dialog messages: client-side validation that names one field becomes a FieldError under that field (input aria-invalid + aria-describedby), same text, immediate, cleared when the value changes. EditTradeLimitModal: "Please enter valid amounts" and "Amounts must be greater than zero" under the field(s) they concern (if both fields apply, form-level via `toast.error` is acceptable; say which you chose), "Minimum must be less than maximum" under the maximum (tb-dialog draws exactly this). EditOfferMarginModal: "Please enter a valid number", "Price must be greater than zero" and the margin range message under their input. Server errors stay `toast.error(...)` (the toast layer renders them as the dialog's form-level error); successes stay `toast.success(...)` (held until close).
3. KYCRequirementModal and /offer/resell EscrowModal: wrap in a glass DialogShell, delete their own overlay, Escape listener and z-index; keep content, props, the setTimeout auto-close and every mutation call unchanged.
4. Sonner: in ReportUserModal, AffiliateDashboard and TradeReviewForm replace `import { toast } from 'sonner'` with `const { toast } = useToast()`; map sonner's `{ description }` to the new positional description argument (`toast.success(msg, undefined, description)`, `toast.error(msg, undefined, description)`). Messages unchanged: 'Report submitted successfully' / 'Our moderation team will review your report.', 'Failed to submit report' with the server message as description, 'Referral link copied!', 'Failed to copy link', 'Please select a rating', 'Thank you for your feedback!', and the review API error. In ReportUserModal, client-side field validation (if any) becomes a FieldError; the submit failure stays toast.error (form-level in the open dialog). Remove `sonner` from package.json and regenerate `bun.lock` with `bun install` and `package-lock.json` with `npm install --package-lock-only --ignore-scripts --no-audit --no-fund` (only for the lockfile); the lockfile diffs must touch sonner entries only. Verify with `git diff --stat` and a grep of the diffs; if anything else moved, hand-edit it back to only remove sonner and report it. In `e2e/pages/report.page.ts` replace sonner selectors with `[data-testid='qt-toast']` and `data-type`.
5. Tests:
   a. KYCRequirementModal.test.tsx, KYCRequirementModal-827-no-limit.test.tsx, resell-modal-math.test.tsx, ticket-u90vdfzd-marketplace-modal-deferral, ticket-647-my-offers-table, public-profile-merchant-tier-607, affiliate suites and any other suite importing your files must pass; switch `container` to `baseElement` only where a portalled dialog requires it, never change an `expect()` otherwise.
   b. New `src/__tests__/regression/ticket-969-no-legacy-overlays.test.ts`: no file in src or e2e imports `ui/modal`, `./modal` or `sonner`; package.json has no sonner; `src/components/ui/modal.tsx` does not exist. (Builder B is migrating the wallet importers in parallel and the overseer deletes modal.tsx last, so this suite will fail until then; that is expected. Report it, do not delete modal.tsx yourself.)
   c. New tests for the three sonner call sites (one suite or three), asserting useToast is called with the exact message, type and description on success and error paths, including the clipboard failure and the missing rating.
   d. Field-error cases for EditTradeLimitModal and EditOfferMarginModal: role=alert text under the field, aria-invalid, no toast.error for that message.
6. Run `bun x tsc --noEmit` (wallet files may be mid-edit by builder B; report errors only from your files), lint on your files, and all suites above. Pre-existing failures must be shown failing on base too (compare with `git show origin/redesign/906:<path>` copies in a scratch dir; never reset the worktree). Logs to `/Users/starbird/Workspace/Code/qic/.scratch/969/evidence/builder-c/`.

Return per the contract's return format, plus the exact lockfile diff stat.

## Notes from the overseer after builder A (commit 49cee000)

- The shared API is committed in 49cee000; read `src/components/ui/dialog.tsx` and `src/components/ui/toast.tsx` as they are. DialogShell also has `headerClassName`, `titleClassName`, `descriptionClassName` (default surface only).
- On the customer surface a caller's `className` on dialog parts keeps only `customer:` tokens and `sr-only`; use `size`, `material` or `customer:` tokens for customer looks. Default-surface classes still apply off customer.
- Field errors: use `FieldError` from `@/components/ui/field` (circle icon); the B-money and C-discovery boards draw the circle icon, which is what you must match.
- Read the contract's 22 Oct addendum: replicate the boards for every element you add (field errors, plates, the error rings on invalid inputs); page content inside these dialogs stays as it is (group cards own it).
