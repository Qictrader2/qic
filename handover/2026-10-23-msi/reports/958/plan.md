# 958 plan: Customer redesign 906-B, wallet, deposits, withdrawals and earnings

Planning only. Written 22 Oct 2026 from:

- Trello 958 (pRSsADn7): description, the 5-item Acceptance Criteria checklist, the "Mandatory QA coverage" block. The card has no comments.
- Design pack v5.2, group B: `~/Downloads/906-design-pack-v5.2/CHANGES.md` and the B PDFs; sources in `.scratch/customer-app-redesign/research/design-pack/v5.2-src/B-money/` (design-notes.md including "Change log: 21 Oct v5.2", B19 `decisions.dc.html`, B0 `Main.dc.html`, register.json with 275 frames, tests.md N1 to N20, every `project/*.dc.html`).
- Code: read-only `redesign/906` at `.scratch/906-redesign/base-ro/frontend` (HEAD 9bbd3d88: 970, 968, 969 merged). The 970 follow-up branch `origin/970-v52-replicate` (adds `ui/amount-input.tsx`, the held `Amount`, the new `status-chip.tsx`, the spinner ring and checkbox/radio fixes) is assumed merged.
- Backend, read only from `qictrader-backend-rs` `origin/main`: `docs/intended-entity-state-machines.md` §3.1 to §3.3 and §5.3, `docs/design/patterns/HARC.md`, `src/api/wallet.rs`, `src/api/affiliate.rs`, `src/api/platform_config.rs`. No per-entity design doc exists for wallet, withdrawal or deposit (`docs/design/` holds only dormant-fee, fee-sweep, merchant-tier, money-recovery-operation, poa-attempt, treasury-sol-swap).

Owner rules applied: the boards are the spec, and only impossible or board-contradicted items are recorded (section 7); frontend only; USDT only plus TRX and SOL named as gas; merge into `redesign/906`, then QA on the review app https://qictrader-fe-pr-303.herokuapp.com (staging backend); USDT is canonical and fiat is display only.

---

## 1. Boards in scope and routes

v5.2 changed 39 of the 44 B boards. B1 to B4, B13c and B17b carry forward from v5 unchanged and are still in scope.

| Board(s) | Route / surface | What it specifies |
|---|---|---|
| B0 | group | Route status, the 308 redirect table, where /dashboard's contents go, pack rules (fee 1.00% rounded up to micro-USDT, reseller share 25%, tier names qualified, outcome unknown, HARC words, Rand rule), phone and navigation checks |
| B1, B2, B3 | `/wallet` at 390, 768, 1440 | Balance hero: "Available to trade or withdraw", ≈ R line, "Held in trades and offers", "Withdrawing", Total (1440 only); Deposit, Withdraw, More; "Your turn" card; Activity (phone rows, desktop table with search, status and type filters, Export CSV, pagination); Network gas note |
| B4 | `/wallet` states | Loading (static skeletons), new account, balance error, Rand missing plus Activity failing on its own. Withdraw and More wait for the balance and say why |
| B5a, B5b | `/wallet` Activity, held funds, merged `/wallet/manage/[currency]` | Activity, no match, transaction detail; held breakdown sheet/dialog with Unlock as an in-place opaque step; Lock funds for an offer; element-by-element map of the old manage page |
| B5c (new) | `/wallet` lock and unlock outcomes | Done, couldn't confirm, Check status found and not found at 390; 768 and 1440 reuse the send result dialog |
| B6a to B6c | Deposit sheet/dialog | Network as inline radio rows (Solana only with `featureFlags.solana`), address and QR on an opaque plate, minimum from `/config/deposit-minimums`, seen (Tron only) and credited from socket events, loading, error |
| B7a to B7c | Withdraw | Where (network rows, address, wrong-network hint, username pointer to Send), how much (Max = available minus fee, "Held USDT isn't included"), review (opaque, total = amount plus fee, quote held to a clock time), "Withdrawal security check" (authenticator or emailed code in the pilot), refused code, offline with details kept, submitted |
| B8a to B8d | Withdrawal outcomes | Sent, held ("Taking longer than usual"), cancelled ("Cancelled, balance restored"), unconfirmed with request reference, Check status found, not found, offline; reference and "Get help with this withdrawal" on every held, cancelled and unknown card; B8d status map (failed vs pending vs unconfirmed) |
| B9a to B9d | More menu, Send to a QicTrader user, network gas | More as action sheet (phone) or glass menu; send who, how much, review, "Send security check", done, errors, couldn't confirm, Check status; gas view (Tron chip from `/gas/system-status`, no Solana chip) |
| B9e, B9f (new) | Security check at 768 and 1440 | Emailed code, authenticator, refused codes, too many tries, too many codes; nothing locks |
| B10 | Prototype | Reference behaviour for deposit, withdrawal, Check status, send |
| B11a, B11b | `/earnings` (new route) | Overview variant A: October commission from `/affiliate/earnings` with month dates, Pay out to wallet, three lines (Reseller all time, Affiliate, Loyalty), section pills, per-card loading and error, 320 and 200% |
| B12, B13a to B13c | `/reseller` | Commission all time, markup sheet/dialog driven by `markupPolicies.buy`, pause a resell offer, trade rows with the v5.2 status labels and commission line, all states |
| B14, B15a to B15d | `/dashboard/affiliate` | Payout figure first, payout (opaque from first step, 10 USDT minimum shown up front), couldn't confirm and Check status, tabs for referrals, commission, payouts, QR code, campaign link, full export, FAQ fix, all states |
| B16a to B16c | `/dashboard/affiliate/tiers` | Progress once, "All affiliate tiers" table, worked example on the 120.00 sale using the live fee, "fee unavailable" instead of the 0.7% fallback, every state |
| B17a to B17c | `/loyalty-programme` signed in | Escrow fee hero, "Merchant tiers" ladder, current month edge, override, paused, error, 320 and 200% |
| B18 | a11y | Contrast roles, focus rules, 44 px targets via `.hit44`, reduced motion, 320 px and 200%, tables become rows below 768 |
| B19 | decisions | Confirmed product rules, build dependencies, tests N1 to N20 |

Routes this card owns: `/wallet` (with `?action=deposit|withdraw|transfer`), `/wallet/manage`, `/wallet/manage/[currency]`, `/wallet/deposit/[currency]`, `/wallet/withdraw/[currency]`, `/dashboard`, `/dashboard/resell`, `/earnings`, `/reseller`, `/dashboard/affiliate`, `/dashboard/affiliate/tiers`, signed-in `/loyalty-programme`, plus the signed-in branch of `/affiliate` and the post-sign-in landing.

---

## 2. Current code map (`base-ro/frontend/src`, line counts in brackets)

Routes

- `app/(dashboard)/wallet/page.tsx` [267]: `WithdrawalMaintenanceBanner`, `CardThemeSelector`, `WalletCard` grid, `WalletActions`, `TransactionHistory`, `PortfolioDistribution`, `SecurityTip`, `PriceCards`, and the Deposit, Withdraw, Transfer modals. Reads `?currency=` and `?action=transfer` only.
- `app/(dashboard)/wallet/manage/[currency]/page.tsx` [614]: balance (adds on-chain USDT at the deposit address to Available, which B5b and CH4 remove), locked funds with Unlock (L360-420), `grid-cols-4` actions (L462), collapsible history, `LockFundsModal`. `?action=deposit|withdraw` opens a modal and is never cleared. `/wallet/manage` itself has no page and 404s.
- `app/(dashboard)/wallet/deposit/[currency]/page.tsx` [17] and `withdraw/[currency]/page.tsx` [17]: server `redirect` to `/wallet/manage/{c}?action=…`; other query params dropped. Helper `lib/wallet-route-redirect.ts` [27].
- `app/(dashboard)/dashboard/page.tsx` [138] plus `features/dashboard/*` (StatsGrid, MonthlyTradingVolume, ProfitTrend, RecentActivity, EscrowOverview, WalletOverview [81], AffiliateSummary, MerchantTierPanel, MyOffersTable [315], ActiveTradesTable [231], QuickActions, PriceChart, escrow.tsx).
- `app/(dashboard)/dashboard/resell/page.tsx` [17]: client `router.replace("/reseller")`, query lost, blank render.
- `app/(dashboard)/dashboard/affiliate/page.tsx` [46] and `tiers/page.tsx` [46] over `features/affiliate/*` (ReferralSection 352, AffiliateStats 154, ReferralActivityTable 410, EarningsChart 284, EarningsHistory 505, TieredRewards 360 with the 0.7 fallback at L17, RequestPayout 220, PayoutHistory 354, FAQ 53, TierBadge 51) and `useAffiliateSocket`.
- `app/(main)/reseller/page.tsx` [39] over `features/reseller/ResellerDashboard.tsx` [640].
- `app/(main)/affiliate/page.tsx` [543]: public; signed in → `router.replace("/dashboard/affiliate")` at L169-173, `?ref` dropped.
- `app/(main)/loyalty-programme/page.tsx` [264]: signed in renders `features/merchant-tier/LoyaltyProgrammeSignedIn.tsx` [591].
- `app/(dashboard)/earnings/page.tsx` [24]: temporary server redirect to `/dashboard/affiliate` (968), tested by `__tests__/navigation/earnings-redirect.test.ts`.
- `next.config.ts`: one redirect (`/terms-and-condition`). `proxy.ts` [62]: staging gate only.
- Post-sign-in landing: `lib/auth/login-redirect.ts` [74] `DEFAULT_POST_LOGIN_PATH = "/dashboard"` (L3); `safePostLoginTarget` used by `app/(auth)/login/page.tsx` L65-66 and `components/providers/auth-provider.tsx`.

Wallet components (`components/features/wallet/`)

- `WithdrawModal.tsx` [1819]. Key facts:
  - Idempotency key lives in `useWithdraw` (`hooks/api/use-wallet.ts` L218-271) as `pendingKeyRef`: reused after a timeout or transport error, cleared on success or a typed API error. It is not bound to the payload, and the modal's close-reset (L379-401) does not clear it, so an edited amount or address can be sent under the old key (section 6, risk 3).
  - Fee quote: `useWithdrawalGas` (L491-510), quote session id from `newQuoteSessionId()` on open (L380), `holdQuoteAndContinue` (L1107-1125) holds it 2 minutes, submitted as `feeQuoteId` (L743, L773); cleared on address or network change; expiry and change errors refetch (L822-834); `feeStatus` blocks submit (L519-528).
  - Step-up: `continueWithdrawal` (L1059-1100) reads `GET /users/2fa/status`, shows TOTP or requests an emailed code via `POST /wallet/step-up/request-code`; prompts at L1660-1695.
  - Network picker: `NetworkDropdown.tsx` [219] (custom listbox). Its Escape handler (L181-186) does not stop propagation, Radix's capture-phase listener closes the whole dialog. Its test renders outside a dialog.
  - Lost response: no lookup and no UI; a transport error toasts (L850).
  - Max: `handleSetMax` (L601-615) → `maxWithdrawableAmount` (`utils/pricing.ts` L601), BigInt minor units, recomputed when the fee arrives.
  - Minimums hard-coded per row (L103-139: 1 USDT Solana, 3 USDT Tron, plus dead BTC/ETH rows).
  - Username in the address field turns into an internal transfer (`handleUserTransfer`, L958-1057).
- `TransferModal.tsx` [669]: `useTransfer` has an in-flight guard only, no request key (backend has none). Step-up at L296-321.
- `DepositModal.tsx` [637]: `useCustodialWalletPage` → `GET /custodial-wallet/deposit-address`, `NetworkDropdown` (L469).
- `LockFundsModal.tsx` [302], `TransactionHistory.tsx` [596], `TransactionStatusModal.tsx` [324] (explorer link correct, history rows pass `tx.currency` so Solana gets no link), `transaction-display.ts` [186], `WalletCard.tsx` [367], `PortfolioDistribution.tsx` [231], `GasFeeDisplay.tsx` [215] (no importers), `StepUpPrompt.tsx` [123], `ManageWalletModal.tsx` [129], `WalletActions.tsx` [49].

Hooks, libs, endpoints

- `hooks/api/use-wallet.ts` [340] and `services/wallet-api.ts` [563]: `GET /wallet`, `GET /wallet/balance` (15 s poll; returns `balance`, `lockedBalance`, `pendingBalance`, `availableBalance` per currency), `GET /wallet/transactions`, `GET /wallet/pending`, `POST /wallet/transfer`, `POST /wallet/withdraw`, `GET /wallet/withdraw/check`, `GET /wallet/locks`, `POST /wallet/locks/lock|unlock`, `requestStepUpCode`.
- Endpoints that exist on the backend and that Check status needs: `GET /wallet/transfers` (wallet.rs:3684), `GET /wallet/locks/{offer_id}` (:3697), `GET /affiliate/payouts` (affiliate.rs:741). None is called by the frontend today.
- `TransactionResponse.customerOutcome` (`cancelled`, wallet.rs:238) and the refund line already exist (818). No held flag exists (919).
- Gas: `services/gas-api.ts` [709], `hooks/api/use-gas-fees.ts` [596]; `/gas/withdrawal-fee/network`, `…/quote/{id}/continue`, `/gas/withdrawal-fees`, `/gas/system-status`.
- Balance and money display: `hooks/use-balance-state.ts` [106] (`useBalanceState`, `useHeaderBalanceView`), `lib/wallet-balance-status.ts` [93], `lib/balance-valuation.ts` [80] (`RATE_UNAVAILABLE_LABEL`, `valuationText`), `hooks/api/use-balance-fiat-rates.ts` (checks the error before cached data, card 908), `lib/fiat-display.ts` [140], `lib/currency-utils.ts` [125], `lib/wallet-manage-transactions.ts` [57], `lib/reporting-money.ts` [26], `lib/exact-amount.ts` [53], `lib/effective-max.ts` [36] (70 bps, card 911, group C).
- Earnings: `services/affiliate-api.ts` [463] + `use-affiliate.ts` [172]; `services/reseller-api.ts` [476] + `use-reseller.ts` [143]; `services/merchant-tier-api.ts` [102] + `use-merchant-tier.ts` [127]; `features/merchant-tier/*` (MerchantTierPanel 252, MerchantTierBadge 135, EscrowFeeBlock 81, EscrowFeeDisclosure 100, merchant-tier-config.ts 131).

906-0 pieces to reuse (970, 969, 968, plus the 970 follow-up)

- `ui/button.tsx`, `ui/amount.tsx` (`Amount {value, status, unit, direction, size}`, held variant from the follow-up), `ui/amount-input.tsx` (follow-up), `ui/status-chip.tsx`, `ui/dialog.tsx` [705] `DialogShell {material: glass|solid, dismissible, role, returnFocusRef, size}` (bottom sheet below 768), `ui/dialog-stack.ts`, `ui/toast.tsx`, `ui/field.tsx`, `ui/radio-group.tsx`, `ui/progress.tsx` (`Steps`), `ui/spinner.tsx`, `error-state.tsx`, `empty-state.tsx`, `loading-button.tsx`, `confirm-dialog.tsx`, `lib/customer-field-classes.ts`, `hooks/use-customer-surface.ts`.
- Navigation: `navigation/BottomBar.tsx` (dock, Earnings → `/earnings`), `Island.tsx`, `Capsule.tsx`, `CustomerHeader.tsx`, `PhoneHeader.tsx`, `AccountHub.tsx`, `JumpPalette.tsx`; route data in `config/routes.ts` [1029] (dock L564-617, `CUSTOMER_SURFACE_RULES` L353-358). The customer header shows no balance, so the "balance repeated in the header" problem is already gone on money pages.

Width at 360 and 320 (known 381 px content): likeliest causes are the `TransactionHistory` pagination row (L552-585, no wrap, about 355 px inside `p-6`), the two pill groups (L254, L271, no wrap), the fixed `w-48` search (L249), `PortfolioDistribution`'s fixed donuts, `CardThemeSelector`'s no-wrap row and `PriceCards`' `grid-cols-2`. All of them are removed or rebuilt in slices 6 and 7.

Tests already present (Jest; `e2e/` is Jest plus Puppeteer): `ticket-969-withdraw-state-survival`, `ticket-969-withdraw-fee-refresh`, `ticket-864-withdraw-quote-expiry`, `ticket-769-withdraw-*`, `ticket-776`, `ticket-839`, `ticket-zoqyJwfY-*-amount-precision`, `ticket-drcwuijn-transfers-2fa`, `ticket-9dl9xx7a-step-up-2fa`, `ticket-645-wallet-manage-*`, `ticket-414-wallet-completed-filter`, `components/NetworkDropdown`, `WithdrawModal.networks`, `balance-surfaces-861`, `header-fiat-864`, `loyalty-programme-*`, `merchant-tier-*`, `affiliate-*`, `lib/balance-valuation`, `lib/fiat-display`, `lib/reporting-money`, `lib/wallet-manage-transactions`, `lib/login-redirect`, `navigation/earnings-redirect`. tests.md section 1 to 10 gives a verdict for each matching file (9 delete, 60 adjust, 22 keep, 20 new: N1 to N20).

---

## 3. Gap per board

| Board | Today | Gap to build |
|---|---|---|
| B1 to B3 | Card grid, theme picker, portfolio donut, price cards, security tip, BTC/ETH remnants | New page: one hero (Available from ledger `availableBalance`, Held = `lockedBalance`, Withdrawing = `pendingBalance`, Total at 1440 = sum), ≈ R from the existing rate, Deposit/Withdraw/More, Your turn card (trades needing the viewer, group A labels), Activity, gas note. Remove everything in the B9a "Removed" list |
| B4 | Single loading state, "Rate unavailable", R 0.00 possible | Per-region loading (static `--surface` skeletons, `aria-busy`), per-region inline errors with Try again (`role="alert"` once), Withdraw disabled while balance unknown or zero and More while unknown, with a visible reason; "Rand estimate unavailable"; no Rand line on zero |
| B5a, B5b | Separate manage page; Available adds on-chain USDT; per-offer Unlock in a list | Held breakdown sheet/dialog from the hero (open trades rows "120.00 + 1.20 fee", offer locks), Unlock as an opaque in-place step, Lock funds in More, Activity as its own phone view and the table at 768/1440, detail sheet, escrow lock rows unsigned (N14), Solana explorer link (FE2), "Failed" filter renamed "Cancelled" |
| B5c | No outcome handling | Lock and unlock done, couldn't confirm, Check status via `GET /wallet/locks/{offer_id}`; Start again blocked on 920 |
| B6 | Modal with `NetworkDropdown` (Escape bug) | Inline network radio rows, address and QR on opaque plate, minimum inline from config, "couldn't check the minimum" fallback, seen/credited driven by `wallet:deposit_detected` and `wallet:balance_updated`, focus rules |
| B7 | One 1819-line modal, dropdown, username-to-transfer path, hard-coded minimums, total computed but never shown | Where → how much → review → security check → submitted; network rows; wrong-network hint; username pointer to Send; Max copy; review shows total and the held-quote clock time; "Withdrawal security check" heading; refused, expired, used, too many tries, too many codes messages; offline keeps everything (N11); code boxes flex to fit 288 px |
| B8a to B8d | Toast on transport error, no lookup, "Failed" wording in history | Opaque, non-dismissible "We couldn't confirm your withdrawal" with request reference; Check status by reading `GET /wallet/transactions` (newest withdrawal of that exact amount, address and network since the confirm time); not found only after the request timeout; offline; Start again with same key and a new code; HARC card with reference and help link; status map N3 |
| B9a to B9d | Transfer modal in the action grid | More menu (sheet/menu, arrow keys, Esc returns focus), Send flow with user search (2 chars, 350 ms, 8 results), "Send security check", done, errors, couldn't confirm and Check status via `GET /wallet/transfers`; gas view with Tron chip only |
| B9e, B9f | Step-up prompt exists | Same SecurityCheck component drawn at 768 and 1440 with all refusal states; no lock screen |
| B11 | Redirect to affiliate | New Overview page with four independent reads and section pills; robots Disallow, no sitemap entry (N7) |
| B12, B13 | ResellerDashboard with raw lowercase statuses, failed markup save only logs, "View Details" does nothing | Commission hero, markup sheet from `markupPolicies.buy` with validation and visible save failure, pause dialog, trade status labels and commission line, incomplete note, every state |
| B14, B15 | Chart summed in browser, export of visible page only, monthly-payout FAQ, dead Contact support | Payout hero, opaque payout flow, minimum up front, couldn't confirm and Check status via `GET /affiliate/payouts`, tabs, QR on plate, campaign link, whole-list export (FE3), FAQ copy fix, help link to Account › Support |
| B16 | Progress shown twice, 0.7% fallback displayable | Progress once, "All affiliate tiers" with Current chip, worked example from `usePlatformFees` with "fee unavailable" (N5) |
| B17 | "LIVE" pulse, "Earned in September", unqualified tier names | Fee hero, "Merchant tiers", "September volume", override, paused, error, no motion on figures |
| B18 | Controls partly retoned by 970 | Apply the v5.2 roles on every B control (radio, selected network edge, step line, progress fill, Loyalty month edge in `--action-pressed`; focus in `--focus` / `--foreground` on glass), `.hit44`, no sideways scroll at 320, 200% phone shell |

---

## 4. Slice plan

Each file belongs to exactly one slice. Later slices render earlier slices' components only through props fixed in slice 1 (`WalletActionProps`: `{open, onClose, initial?}`), so the flow slices can rewrite `DepositModal.tsx`, `WithdrawModal.tsx`, `TransferModal.tsx` and `LockFundsModal.tsx` in place and the old pages keep working until slice 8 retires them. Sizes are product lines added or changed, excluding tests and deletions. Every slice merges into `redesign/906` and is QA'd on the pr-303 review app.

| # | Slice | Boards | Main files (all under `src/`) | Product lines | Depends on |
|---|---|---|---|---|---|
| 1 | Money primitives | B7a, B8a to B8d, B9a, B9e, B9f, B18 | New `lib/money-request-intent.ts` (one request key per intent fingerprint: kind, network, address or recipient, exact minor amount; kept in `sessionStorage` plus a `storage` event so a second tab sees it; cleared only on a server answer), `lib/withdrawal-customer-status.ts` (B8d map, HARC labels, reference short form W/S/P/L-XXXX-XXXX, `/report?withdrawal=` builder), `lib/check-status-match.ts` (pure matchers over transactions, transfers, payouts, locks), `components/money/OutcomeUnknown.tsx`, `CheckStatusResult.tsx`, `RequestReference.tsx`, `HarcStatusCard.tsx`, `SecurityCheckStep.tsx` + `CodeBoxes.tsx`, `NetworkRadioRows.tsx`, `wallet-action-props.ts` | about 1,200 | 970 follow-up |
| 2 | Held funds, lock and unlock | B5a, B5b (held), B5c, B9a (Lock funds) | New `features/wallet/HeldFundsSheet.tsx`, rewrite `LockFundsModal.tsx` in place, lock/unlock outcome wiring using slice 1 | about 900 | 1 |
| 3 | Deposit | B6a to B6c | Rewrite `DepositModal.tsx` in place; new `features/wallet/deposit/*` | about 700 | 1 |
| 4 | Withdraw and outcomes | B7a to B7c, B8a to B8d, B9e, B9f (withdraw half) | Rewrite `WithdrawModal.tsx` in place as a step orchestrator over a pure reducer; new `features/wallet/withdraw/*` steps; delete `NetworkDropdown.tsx` and `StepUpPrompt.tsx` if no importers remain; bind the key to the intent in `hooks/api/use-wallet.ts` `useWithdraw` (fingerprint from slice 1, everything else unchanged) | about 1,450 | 1 |
| 5 | Send and network gas | B9a to B9d, B9e, B9f (send half) | Rewrite `TransferModal.tsx` in place; new `features/wallet/send/*`, `features/wallet/NetworkGasSheet.tsx`, `features/wallet/MoreMenu.tsx` | about 1,100 | 1 |
| 6 | Wallet page | B1 to B4, B9a (More entry) | Rewrite `app/(dashboard)/wallet/page.tsx`; new `features/wallet/WalletOverview.tsx` (the one overview), `BalanceHero.tsx`, `YourTurnCard.tsx`, `WalletActionsHost.tsx` (reads and clears `?action=deposit|withdraw|transfer` and `?network=`); delete WalletCard, CardThemeSelector, PortfolioDistribution, PriceCards, SecurityTip, WalletActions, ManageWalletModal, GasFeeDisplay where unimported | about 900 | 1 to 5 (mounts them) |
| 7 | Activity and detail | B5a, B5b (Activity), B1 to B3 table | Rewrite `TransactionHistory.tsx` (phone rows below 768, table above, wrapping filters and pagination, search, status and type filters, whole-history CSV), `transaction-display.ts` (uses slice 1 status map), `TransactionStatusModal.tsx` → detail sheet with the right explorer per network | about 1,100 | 1, 6 |
| 8 | Redirects and landing | B0, B13, B5b map | `next.config.ts` 308s (`/dashboard`, `/wallet/manage`, `/wallet/manage/:currency` → `/wallet`; `/wallet/deposit/:c` → `/wallet?action=deposit` with `usdt-spl`/`usdt_spl` → `&network=solana`; same for withdraw; `/dashboard/resell` → `/reseller`; queries kept); delete the five old route pages and the manage page; `lib/auth/login-redirect.ts` default to `/wallet`; `(main)/affiliate/page.tsx` signed-in replace keeps the query; delete `features/dashboard/*` files with no remaining importer; `lib/wallet-route-redirect.ts` removed | about 250 | 6 |
| 9 | Earnings overview | B11a, B11b | Replace `app/(dashboard)/earnings/page.tsx`; new `features/earnings/EarningsOverview.tsx`, `EarningsSectionPills.tsx`; `app/robots.ts` Disallow; sitemap and public-routes tests | about 550 | none |
| 10 | Reseller | B12, B13a to B13c | Rewrite `features/reseller/ResellerDashboard.tsx` into `features/reseller/*` (hero, markup sheet, pause dialog, trade rows, states); `app/(main)/reseller/page.tsx` | about 1,100 | 9 |
| 11 | Affiliate and payout | B14, B15a to B15d | Rewrite `app/(dashboard)/dashboard/affiliate/page.tsx` and `features/affiliate/*` except TieredRewards/TierBadge/TierProgressBar (payout flow on slice 1's outcome pieces, tabs, QR, campaign link, export, FAQ) | about 1,450 | 1, 9 |
| 12 | Tiers and loyalty | B16a to B16c, B17a to B17c | Rewrite `dashboard/affiliate/tiers/page.tsx`, `features/affiliate/TieredRewards.tsx`, `TierBadge.tsx`, tier progress; `features/merchant-tier/LoyaltyProgrammeSignedIn.tsx` and the signed-in branch of `app/(main)/loyalty-programme/page.tsx` | about 1,200 | 9 |

Order: slice 1 first. Then two tracks in parallel. Wallet track: 2, 3, 4, 5 in parallel, then 6, then 7 and 8. Earnings track: 9, then 10, 11 (after 1), 12 in parallel. Slice 4 is the riskiest and should get the strictest review (two sign-offs); slices 1, 2, 5 and 11 also move or display money and should get the same.

Tests per slice follow tests.md: delete the 9 obsolete tests in the slice that removes their component, adjust the 60 in the slice that changes their target, and add N1 to N20 where they land (N1 slice 6; N2, N9, N12 slices 1 and 4; N3, N20 slices 1 and 7; N4 slice 8; N5 slice 12; N6 already 968; N7 slice 9; N8, N13 slices 5 and 11; N10, N11, N17, N18 slices 1, 4, 5; N14 slice 7; N15 slice 10; N16 every slice; N19 slice 2). Add a dialog-level test for Escape inside the network rows (the known 969 regression) in slice 4. Captures for AC5 (current, proposed, implemented at 390, 768, 1440, plus 320 and 200% where drawn) are produced per slice on the review app and handed to card 965's register.

---

## 5. Acceptance mapping

AC numbers follow the card's checklist (1 to 5); the QA lines are the card's "Mandatory QA coverage" bullets.

| AC | QA line | Slices | Status |
|---|---|---|---|
| 1 | Deposit Tron and Solana USDT, withdraw on both, internal transfer, read history on staging with QA accounts | 3, 4, 5, 7 | Buildable today |
| 1 | Loading, empty, populated, validation, error, pending, held, completed render the v5.2 screens at 390 and 1440 (boards also draw 768) | all | Held state blocked by **919** (no held flag on `GET /wallet/transactions`); the rest buildable |
| 2 | Available, held, pending, fees and totals equal the API to six decimals; held trade funds not double counted | 1, 4, 6, 7 | Buildable (ledger `availableBalance`, `lockedBalance`, `pendingBalance`; 854 shipped). The fee figure is whatever the server quotes, so it shows 1.00% wherever the environment's config does; the 0.7% code default is **918** and does not block display |
| 2 | Fiat uses the existing rate rule and shows the unavailable state | 6, 7, 9, 11, 12 | Buildable. The wrong "unavailable" after one failed refetch is card 908 (frontend, Backlog): decide whether slice 6 absorbs the one-hook fix in `use-balance-fiat-rates.ts` |
| 3 | Same security check as today; unflagged account without authenticator sees no emailed code; wrong or expired code refused | 1, 4, 5 | Buildable (682 and 904 shipped; `STEP_UP_2FA_SCOPE=flagged`). QA needs one staging account with the `step_up_2fa` flag row, set and restored by the sign-in agent |
| 3 | Dropped withdrawal: confirmation stays open, Check status only reads, found/not found/offline, exactly one debit | 1, 4 | Buildable with the history lookup; exact lookup by key and a server reference in Activity are **920** (optional for withdrawal, the key already replays) |
| 3 | Dropped transfer: same, exactly one debit; double submit and two tabs never send twice | 1, 5 | **Blocked by 920.** `POST /wallet/transfer` takes no request key, so Start again could debit twice; the UI ships Check status (read only) and holds Start again behind the request timeout, as B9a draws, but "exactly one debit" cannot be proven |
| 3 | Two tabs never send twice (withdrawal) | 1, 4 | Partly: the key store shares one key per intent across tabs in the same browser session; a guarantee across devices needs **920** |
| 3 | Held withdrawal and HARC cancellation wording only after server confirmation; held, failed, unknown distinct; reference and support link | 1, 4, 7 | "Taking longer than usual" **blocked by 919**. "Cancelled, balance restored" buildable from the existing `customerOutcome` and refund line (818). Reference and link buildable; the link's landing (`/report?withdrawal=`) needs card 956 to read the parameter |
| 3 | Max = available minus fees and existing holds; uncredited deposit never available | 4, 5, 6 | Buildable (`availableBalance` is ledger only; slice 6 drops the manage page's on-chain addition) |
| 4 | Reseller, affiliate, affiliate tier and loyalty figures equal the server; merchant and affiliate tiers named apart | 9 to 12 | Buildable. Dispute labels "Released to the buyer" and "Returned to the seller" (N15) depend on **963** (and 953) giving a distinguishable outcome in `/reseller/trades`; until then the row reads "Commission waits for the dispute" or the settled line |
| 5 | Old deposit, withdraw, resell, affiliate links land with query kept | 8 | Buildable |
| 5 | Captures for every changed money page and state; WCAG 2.2 AA, 320 px, 200%, reduced motion | all | Buildable except the CTA contrast limitation (kept as approved by Christian, B18) and the held state frames (919) |

Payout and lock/unlock "Start again keeps the request identity" (B15, B5c, N19) are also **blocked by 920**. Cards 910, 911, 917 and 964 block no 958 AC: 910 has no B screen (held funds only), 911 is a group C frontend card (`effective-max.ts`), and 917 and 964 are password rules drawn by groups D and E.

---

## 6. Risks

1. **Money figures.** Every amount goes through `Amount` and `lib/exact-amount.ts` from minor units; nothing is computed in floating point or rounded for display beyond the board's rule (2 decimals, up to 6 when present, never rounded away). Total on the 1440 hero is a sum of server fields, never a client recalculation of fees.
2. **Fees and Max.** The fee always comes from the server quote; Review stays off until it has loaded; the held quote's clock time comes from `quoteHoldUntil`; an expired quote is re-read, never assumed. Max keeps `maxWithdrawableAmount` with the server fee. Send has no fee.
3. **Idempotency key not bound to intent (existing defect).** Today a key kept after a transport error is reused for a different amount or address, and the backend would replay the first withdrawal (`find_by_idempotency_key`, wallet.rs:2249). Slice 4 binds the key to the intent fingerprint and keeps it across Close (B8a) so a later Withdraw to the same address and amount opens Check status first. Everything else in `useWithdraw`, the fee quote handling and the step-up flow stays as built.
4. **Never resend after an unknown result.** Step-up codes are single use and checked before the replay (wallet.rs:2159, 2201; step_up.rs:316). The client never re-POSTs on its own; Start again asks for a new code with the same key.
5. **Check status false match.** Matching by exact minor amount, address, network and creation time after the confirm can still pick the wrong row if the customer made two identical withdrawals in the window. Show the reference and time; the exact answer needs 920's lookup by key.
6. **Transfers, payouts, locks without keys** (920): a late-arriving request plus Start again can move money twice. The UI must not offer Start again before the request timeout and must say "Don't send it again yet".
7. **HARC wording.** Never infer "held" from elapsed time; without 919 a pending withdrawal stays "Sending". "Cancelled, balance restored" only when `customerOutcome = cancelled` and the refund line exist. The internal status stays `Failed`; the customer never sees "Failed" or "Rejected".
8. **Dismissal during commit.** Opaque commit and outcome layers use `DialogShell dismissible=false` and `role="alertdialog"`; Escape inside network rows must not close a glass step (the known 969 issue disappears with inline radio rows, and slice 4 adds the test).
9. **Authorization.** Frontend only: every read is the session user's own endpoint; no user id goes into a request or URL; the help link carries only the reference. Unlock of a trade lock is refused by the backend (962) and must show that refusal, not a success.
10. **Redirect regressions.** Check on the review app how the `next.config.ts` 308s interact with the staging gate in `proxy.ts` (gated visitors must still land on the right page after unlocking). Backend notifications still link `/dashboard/resell` (escrow_release.rs:1191), so that 308 must stay.
11. **Deleting `/dashboard`.** `features/dashboard/MyOffersTable` and `ActiveTradesTable` may be imported by group C or A pages; delete only files with no importer at merge time.

---

## 7. Card vs board drift (for the coordinator)

| # | Card says | Board says (wins) |
|---|---|---|
| 1 | Fix sketch: rebuild `wallet/manage/[currency]` and `dashboard` | Both become 308s to `/wallet` (B0, B13; Confirmed by Christian 21 Oct). Nothing is rebuilt there |
| 2 | "Keep the redirect pages" | Deposit and withdraw entry points become one 308 hop each to `/wallet?action=…` in config (B0) |
| 3 | Example totals at 6 decimals ("10.500000 USDT") | USDT shown to 2 decimals, up to 6 when the amount has them, never rounded (B7a Total note). QA compares to 6 decimals numerically |
| 4 | Fiat "Rate unavailable" fix; fiat "using the existing marketplace-rate rule" | Words are "≈ R" and "Rand estimate unavailable", never "R 0.00" or "Rate unavailable" alone; NGN and KES follow the same rule (B4, M10) |
| 5 | QA states at 390 x 844 and 1440 x 900 | Every B route is drawn at 390, 768 and 1440 (M19), so 768 is in scope too |
| 6 | "backup-code access where supported" | Backup codes work at sign-in only; "Can't use your authenticator?" opens account recovery as built (B7a) |
| 7 | "the emailed code for flagged accounts" | Emailed code only for accounts without an authenticator in the step-up pilot; authenticator accounts always use it (v5.2 scope correction) |
| 8 | Signed-in `/affiliate` lands with query kept | B0 says "kept and unchanged" and today drops `?ref`; keeping the query is a one-line change with no board contradiction, planned in slice 8 |
| 9 | "Double submit and two tabs never send twice" for transfers | Not achievable before 920 (section 5) |

Impossible or contradicted board items, recorded:

1. **FE1, withdrawal minimums from config (B7a, B19).** No customer endpoint exposes the enforced per-network minimums (`withdrawal_minimums_by_network` is read only inside `POST /wallet/withdraw`; `/config/cryptos` returns a hard-coded "5" for USDT). Keep 3 USDT Tron and 1 USDT Solana in one constant and let the server's refusal show. Needs a backend card; none exists among the named dependencies.
2. **Held state "Taking longer than usual"** cannot be shown until 919 ships (board itself says so).
3. **Request reference surviving in Activity after Close** (B0 phone checks): the transaction response carries no key; the client can keep it for the browser session only until 920.
4. **Reseller dispute outcome labels** need a distinguishable outcome from the backend (963).
5. **Solana gas chip** stays absent until 928 (board draws it that way).

---

## 8. Files other page-group cards also touch

| File or area | Other card | Note |
|---|---|---|
| `next.config.ts` redirects | 959 (`/trade-history/[id]` 308), 957 (`/prices` retirement) | Append-only rule blocks; agree order of merge |
| `lib/auth/login-redirect.ts`, `auth-provider.tsx`, `app/(auth)/login/page.tsx` | 955 | 958 changes only the default constant to `/wallet`; tests are group E's |
| `/report` page (`app/(main)/report/page.tsx`) | 956 | Must read `?withdrawal=` to prefill the report; 958 only links |
| Account `/profile` price alerts link (M9), reputation figures from /dashboard | 956 | 958 removes them from /dashboard; 956 adds them |
| `components/money/OutcomeUnknown` and friends (FE11 shared with group A) | 959 | Built in 958 slice 1 unless 959 lands first; one component |
| `lib/state-machine/status-display.ts`, `use-trades` (Your turn card, Activity trade rows) | 959 | 958 reads, does not edit |
| `features/dashboard/MyOffersTable.tsx`, `ActiveTradesTable.tsx`, `lib/effective-max.ts`, `features/marketplace/fiat-reference-display.ts`, resell flow | 957 | 958 deletes only unimported dashboard files; 911 stays with 957 |
| `features/wallet/*`, `TransactionHistory.tsx`, `WalletOverview.tsx`, `features/affiliate/*`, `features/dashboard/*`, loyalty and dashboard pages | 966 | 966 skips files 958 rebuilt; land 958 first or narrow 966 |
| Screenshot register | 965 | 958 supplies implemented captures |
| `config/routes.ts`, `__tests__/navigation/earnings-redirect.test.ts`, `app/robots.ts`, sitemap and public-routes tests | 968 (done), 957/959 for their own routes | 958 edits only the /earnings entries |

---

## 9. Size estimate

- Product code: about 11,900 lines added or changed across 12 PRs (largest 1,450). Tests: about 4,000 to 5,000 lines (N1 to N20 plus about 60 adjusted files). Deletions: about 5,000 lines (old wallet cards, manage page, dashboard page and widgets, NetworkDropdown, chart).
- This is XL. 4 of the 12 slices move or display money and need strict review. Realistic effort is 9 to 12 build days for one agent pipeline, about 6 to 8 with the two tracks in parallel, plus review-app QA per slice. That excludes waiting on 919 and 920, which keep part of AC1, AC3 and the payout and lock outcomes open whatever the frontend does.
