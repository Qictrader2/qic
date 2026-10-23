# Card 919 design notes: held and cancelled withdrawals in the current app (proposed v2)

Status: **proposed**. No human has approved this package. Generated 2026-10-22 against root `0cdc6464`, frontend `a73d2708`, backend `8fda17fa`.

This is an annotated version of the existing layout, not a redesign. v2 replaces the earlier v1 package for this card. On 22 Oct 2026 the owner decided that the approved v5.2 boards (B8a to B8d, plus the BE1 note and item 8 on B19, wording confirmed by Christian on 21 Oct 2026) are the design gate and the exact wording. Where the v1 refinements D1 to D5 went beyond the boards, the boards win. So v2:

- drops v1's invented helper sentence and uses the B8a held sentence;
- uses the board's warning tone for the held chip instead of info blue;
- adds the withdrawal reference and "Get help with this withdrawal" (BE1, B19 item 8);
- renames the "failed" filter chip to "Cancelled" (BE1, B8d);
- shows "Cancelled, balance restored" only when the refund is confirmed;
- adds the `/report` prefill;
- drops v1's focus rings on Copy and explorer and the spinner reduced-motion change (D4, D5), which no board asks for.

## Boards

| File | Viewport | Content |
| --- | --- | --- |
| `design/919-held-withdrawal-desktop.html` (`.png`, `.pdf`) | 1440 × 1100, full page | A: `/wallet` table, light. B: the same, dark, with keyboard focus and hover. C: one 200 USDT + 2.50 fee withdrawal across fetches (held, then resumed or cancelled). D: profile Transaction History, light and dark. E: `/report?withdrawal=…` prefill, light and dark. F: label rules, reference format, CSV, motion and accessibility. |
| `design/919-held-withdrawal-mobile.html` (`.png`, `.pdf`) | 390 × 844, full page | A: `/wallet` card view, light. B: the same, dark, with keyboard focus. C: profile Status column, light and dark. D: `/report` prefill. |

`/wallet/manage/[currency]` renders the same `features/wallet/TransactionHistory` component, so the `/wallet` boards cover it. The builder must confirm that `mergeManageTransactions` passes `customerOutcome` through.

## Decisions

1. **Wording, from the boards only.** Labels: "Taking longer than usual" and "Cancelled, balance restored". Helper for held rows, from B8a: "Your {amount} {currency} stays held and you don't need to do anything. If it can't be sent, we'll cancel it and restore your balance." Reference label: "Withdrawal reference", from B8a. Link: "Get help with this withdrawal", from BE1. Filter chip: "Cancelled". There is no other new copy. The shipped 818 refund line stays under cancelled rows.
2. **Held amount.** `{amount}` is `amount + fee`, the figure that left Available. That is what B8a shows: 202.50 for a 200.00 withdrawal with a 2.50 fee. Format it with 2 to 6 decimals, so USDT never loses micro-units and never reads "202.5".
3. **When each label shows.** Held: `type = withdrawal`, `status = pending`, `customerOutcome = "held"`. "Cancelled, balance restored": a failed withdrawal, or one with outcome `"cancelled"`, where `refundedAmount` is present. Any other failed withdrawal keeps "Cancelled", so a refund the backend has not confirmed is never claimed. Ordinary pending, confirmed, deposits and transfers are unchanged.
4. **Reference and link placement.** They show on held rows and on every cancelled withdrawal row, including one without refund evidence, because B19 says every held, cancelled and unconfirmed withdrawal shows them. Desktop: in the Status cell under the helper. Mobile: the reference is a detail row after Amount and Note and before TxID, and the link closes the card with a 44px hit area. Profile: under the pill and helper.
5. **Reference format.** `"W-" + first 4 + "-" + last 4` hex characters of the wallet transaction id, hyphens removed, upper-cased. Example: `7c41…9e2a` becomes `W-7C41-9E2A`, matching the B8a sample. Help href: `/report?withdrawal=` plus `encodeURIComponent(ref)`. If card 920 fixes a different derivation first, use 920's; one withdrawal must never show two references. That item is a plan preparation task, not a design choice.
6. **Held chip.** Warning tone, as on B8a and B8d: `bg-warning-bg text-amber-800 dark:text-warning`, about 5.8:1 in light. A static 6px dot replaces the spinner: `h-1.5 w-1.5 rounded-full bg-current mr-1.5`, `aria-hidden`. `text-warning` alone is about 2:1 on the light panel, so the light text is darkened, the same way the shipped info chip uses `text-blue-700 dark:text-info`. Amber with a dot reads apart from the orange spinner chip of ordinary pending.
7. **Case.** The badge's `capitalize` class would print "Taking Longer Than Usual". Both sentence labels get `normal-case` so the board wording shows as written. In the profile, `titleCase` only raises the first letter, so it needs no change.
8. **Cancelled chip contrast.** Withdrawal cancelled chips change from `text-gray-400` to `text-gray-600 dark:text-gray-400`, in the `getStatusColor` branch this card already edits. Gray-400 on the light panel is about 2:1. The board's neutral chip uses dark text.
9. **Link style.** `text-xs` on desktop and profile, `text-sm` on mobile cards. All use `font-medium underline underline-offset-2 text-sky-700 dark:text-brand-blue-light`: 5.2:1 on the light panel and 7.7:1 on the dark one. `#00A3F6` text would be about 2.5:1. Focus: `rounded-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-blue focus-visible:ring-offset-2 focus-visible:ring-offset-background-light`. Same tab, as a Next `Link`.
10. **Mobile header.** The two sentence chips wrap to two lines: `min-w-0 whitespace-normal items-start text-left`, dot `mt-1.5`. The header row gets `gap-2`, the ID and date block `shrink-0`, and the date `whitespace-nowrap`. The first render showed the date breaking without these; it is fixed and re-inspected. Desktop chips stay on one line. The desktop helper width grows from 220 to 260px for the held sentence.
11. **Profile.** Add `statusColors` keys for "Taking longer than usual" (warning style, as on `/wallet`) and "Cancelled, balance restored" (reuses Cancelled). Without them both fall back to the orange Pending pill. `ProfileTransactionRow` gains `reference` and `helpHref` from the same `transaction-display.ts` helpers. `helperText` takes the held sentence, then the sweep-hold line, then the refund line.
12. **/report prefill.** Only the Subject field is prefilled, and only with a value matching `^W-[0-9A-F]{4}-[0-9A-F]{4}$` (case-insensitive, then upper-cased). Report type stays Other Issue, and Description stays empty with its 20-character rule. Invalid values are ignored. The prefill runs once on mount, inside a `Suspense` boundary for `useSearchParams`. Signed out, the existing Login link carries `redirect=` plus the encoded `/report?withdrawal=…`, so the reference survives sign-in. No new text.
13. **CSV.** The Status cell comes from `statusLabel`, so it carries both new labels. `escapeCsvField` already quotes the comma label; add a test that pins it. The helper, reference and link are not exported, so the column set is unchanged.

## States covered

Held, with the keyboard focus on the help link in light and dark. Resumed (confirmed). Cancelled with confirmed refund. Cancelled without refund evidence. Ordinary pending (control). Dark hover. The renamed filter chip next to an active type filter. Profile light and dark. `/report` prefilled, light and dark. CSV. Loading (shimmer), empty and error are unchanged: a held row only appears after data loads, so there is no new loading or error variant.

## Motion

- Held chip, dot and helper: **static**. No spinner, pulse or colour cycling while money is held.
- Held to confirmed or cancelled: **instant** on the next history fetch. No fade, slide or count-up on the chip, helper, refund line or balance. No polling and no new socket event.
- Help link hover: `transition-colors` 150ms `cubic-bezier(0.4,0,0.2,1)`, colour only, retargets on pointer leave. Under reduced motion it is the same, because nothing moves.
- Focus ring: instant.
- The existing ordinary Pending spinner and row hover are unchanged.

## Privacy (AC3)

The customer surfaces show only the board wording, the reference, the link and the shipped refund line. They never show the hold reason, `failure_reason`, treasury figures, "auto-cancelled", admin wording or a countdown. The `/report` prefill carries only the reference.

## Approval

The owner decision of 22 Oct 2026 makes the approved v5.2 boards the design gate and says no separate design approval is needed. This package only maps that approved wording onto the current components using existing tokens, so it adds no implementation gate. The boards are still labelled proposed because a generated package is never called approved.

## Observed, out of scope

- A confirmed wallet row in the profile reads "Confirmed" and falls back to the orange Pending pill, because no `statusColors` key exists for it. This is existing behaviour, and a resumed withdrawal will show it.
- Other sentence labels on `/wallet` ("On hold", "Received, not yet credited") are still title-cased by `capitalize`. This card does not change them.

## Verification

Both boards were rendered with `qic_design_render.ts` and checked at full resolution in slices: desktop 1440 in five 1000px slices, mobile 390 in five 900px slices. Defects found and fixed before the final render:

- the CSV excerpt now leaves the withdrawal network-fee cell blank, as `transactionToCsvRow` does;
- focus rings no longer sit inside the dashed annotation boxes;
- the light cancelled chip contrast is fixed;
- the tab-order statement now matches the DOM order;
- the mobile date no longer wraps;
- the profile annotation outline is no longer clipped.

The PDFs come from the same Chromium pass with backgrounds printed. They were not opened page by page.
