# 959 plan: trade room, trade history and money steps (906-A)

Card: https://trello.com/c/SPVPnRAe (no comments on the card as of 22 Oct 2026). Planning only: no code, commits or Trello writes were made.

Sources read:
- Boards: `~/Downloads/906-design-pack-v5.2/` (CHANGES.md, A PDFs).
- Board sources: `.scratch/customer-app-redesign/research/design-pack/v5.2-src/A-trading/` (design-notes.md including "Change log: 21 Oct v5.2", register.json with 255 entries, tests.md, project/*.dc.html, trading.css, canvas.json).
- Code: the read-only `redesign/906` checkout at `.scratch/906-redesign/base-ro/frontend` (tip 9bbd3d88), plus the `970-v52-replicate` diff.
- Rules: `.scratch/906-redesign/HANDOVER.md`, root `CLAUDE.md` and `DESIGN.md`.
- Intent docs: backend `origin/main` (f043a14), `docs/intended-entity-state-machines.md` sections 1.1 Trade, 1.3 Escrow, 1.4 TradeKyc and 4.2 Dispute. `docs/design/` has no trade, escrow or dispute entity doc yet.
- Backend handlers on `origin/main`: `api/trades.rs` (TradeResponse, get_trade, get_trade_ledger) and `api/moderation.rs` (resolve_dispute).
- Dependency cards: 920, 918, 910, 911, 963, 953, 877, 909, 612 and 921, plus the sibling cards 955 to 958, 965 and 967.

Rule applied throughout: the v5.2 boards are the spec. Where the card or the shared test notes disagree, this plan follows the board and records the difference in section 7.

---

## 1. (a) Boards in scope and routes

### 1.1 Routes
| Route | Today | After 959 |
|---|---|---|
| `/trade/[id]` | Live trade room (`app/(main)/trade/[id]/page.tsx`) | Rebuilt as the v5.2 room for buyer, seller and reseller at 390, 768 and 1440, plus 320 px and 200% |
| `/trade-history` | Table page (`app/(dashboard)/trade-history/page.tsx`) | Trades page with Active and History sections, filters, export and states |
| `/trade-history/[id]` | Older duplicate room | 308 redirect to `/trade/[id]` that keeps the id and the query (A16) |

### 1.2 Boards and frames (249 build frames)
| Board | File | Frames |
|---|---|---|
| A2 | room-buyer-390 | 8: Funded, Payment sent, USDT released, Completed, In dispute, Dispute resolved · released to the buyer, Dispute resolved · returned to the seller, Cancelled |
| A3 | room-seller-390 | The same 8 statuses, seller view |
| A4a / A4b | room-buyer-768 / -2 | 4 + 4: the same statuses, buyer, tablet |
| A5a / A5b | room-seller-768 / -2 | 4 + 4: seller, tablet |
| A6a / A6b | room-buyer-1440 / -2 | 4 + 4: buyer, desktop |
| A7a / A7b | room-seller-1440 / -2 | 4 + 4: seller, desktop |
| A8 | room-sheets-390 | 17 phone sheets: More menu (open trade, closed trade, in dispute); I've paid (empty, upload failed, ready); Release (not ticked); Early release; Cancel (before paying, after I've paid); Open a dispute (error, filled, over the limit); Add evidence (plain, typing and scrolled); Chat (buyer, seller) |
| A9a / A9b / A9c | room-dialogs-lg-1..3 | 15 desktop dialogs: the A8 set plus "Release · ticked" (the chats are a column on large screens) |
| A19a / A19b | room-dialogs-768-1..2 | 15 tablet dialogs, the same set |
| A10 | room-states-390 | 19 edge states: Loading; Couldn't load; Can't open; Under 5 minutes; Time ran out (buyer, seller); Proof can't load; Live updates paused; Slow down; Reseller watching; Documents (buyer, seller review, reject attempt 1, send again, reject again); Cancelled · documents (buyer, seller); Rating (picked, sent) |
| A11a / A11b / A11c | room-states-lg-1..3 | 19, the same at 1440 |
| A20a / A20b / A20c | room-states-768-1..3 | 19, the same at 768 |
| A21a | room-unknown-390 | 14: Start (starting, not enough USDT, did not start, couldn't confirm, checking, found it, hasn't reached us, still can't reach us), drawn as a "layer for C"; I've paid (confirming, didn't go through, couldn't confirm, it went through, hasn't reached us, still can't reach us) |
| A21b | room-unknown-390-2 | 18: Release, Cancel and Open a dispute, each with pending, failed, couldn't confirm, it went through, hasn't reached us and still can't reach us |
| A22a / A22b | room-unknown-lg-1..2 | 7 + 7: unconfirmed, found and not found at 768 and 1440 |
| A22c / A22d | room-unknown-lg-3..4 | 5 + 5: pending, failed, found and offline at 768 and 1440 |
| A12 | proto-trade | Click-through prototype. It is the behaviour spec; `tools/proto_test.js` holds the role and state rules |
| A13 | history-390 | 14: Active, History, Filters, Filtered, Export, No trades yet, No matches, Loading, Couldn't load; 320 px (History, Active, Filtered); 200% (first screen, scrolled) |
| A14 | history-768 | 9 |
| A15 | history-1440 | 8 |
| A16 | history-id-redirect | 3 (the completed trade reached from History at 390, 768 and 1440), plus the redirect rule |
| A17a / A17b | a11y-1 / a11y-2 | Dock at 2x (390, 360, 320); 320 reflow; 200%; 320 px with a long name and a large amount; contrast and keyboard tables |
| A1a, A1b, A18a, A18b | overview and decision boards | Rules, the status model, motion and layer tables, and the tests to adjust. These are not built as screens; they are read as the spec |

The Start frames on A21a sit on C's `/offer/[id]` (C6 to C8, C22a, C22b). The page belongs to 957. 959 builds the shared outcome step that the page mounts (section 4, S2).

---

## 2. (b) Current code map

All paths are under `frontend/src/`, at tip 9bbd3d88.

### 2.1 `/trade/[id]`
- **Page:** `app/(main)/trade/[id]/page.tsx` (405 lines) and `layout.tsx` (13). The `(main)` group has **no `RequireAuth`**. There is no `loading.tsx`, `error.tsx` or `not-found.tsx`.
- **Hook:** `hooks/api/use-trade-detail-page.ts` (1,002). It owns data, polling, escrow, socket and every action. Per the card, it stays unchanged.
- **Queries and mutations:** `hooks/api/use-trades.ts` (564), keys in `tradeKeys`:
  - `['trades','detail',id]`, staleTime 10 s
  - `['trades','messages',id,{limit:100}]`
  - `['trades','events',id]`
  - `['trades','kyc',id]`
  - The global default is `retry: 0`.
- **Socket:** `hooks/use-trade-socket.ts` (154), `hooks/use-socket-connected.ts`, `lib/socket-client.ts` (661).
  - Events: `join:trade`, `trade:update` (invalidates the detail), `trade:message`, `trade:typing`, `trade:send_message` (with `clientMessageId`).
  - Reconnection: 15 attempts at 1 to 5 s. Room re-join runs on reconnect, and a failed join is retried up to 3 times.
- **Polling:**
  - Trade every 5 s while open.
  - Messages every 5 s, or every 3 s without the socket.
  - Escrow and balance every 10 s.
  - KYC every 15 s.
- **Page-load side effects (kept, but see Risks):** a seller who opens a pending trade can auto-create a custodial escrow (`POST /escrow/custodial/create`) and auto-confirm a deposit (`POST /escrow/trade/:id/confirm-deposit`).
- **Components** (`components/features/trade/`, lines in brackets):
  - Room: `TradeHeader` (198), `TradeStatusBanner` (158), `TradeSidebar` (973: summary, fee disclosure, reseller breakdown, every action button and dialog), `release-confirm-copy` (38).
  - Chat: `TradeChat` (1,014), `ChatAttachment` (373), `UploadProgressBar` (186), `message-reconciliation.ts` (135).
  - Disputes and proof: `DisputeEvidenceUpload` (198), `ProofOfPaymentDisplay` (333).
  - Completion: `DownloadReceiptButton` (75), `TradeReviewForm` (166), `TradeEventTimeline` (163).
  - Documents: `TradeKycPanel` (341), `TradeKycReviewPanel` (255), `TradeKycSubmittedFiles` (137).
  - From outside the folder: `features/merchant-tier/EscrowFeeDisclosure` (100), `ui/confirm-dialog`, `ui/breadcrumb`, `ui/toast`.
- **API calls** (`services/trades-api.ts`, 1,426; kept unchanged):

  | Call | Endpoint |
  |---|---|
  | `getTrade` | GET `/trades/:id` |
  | `markPaid` | POST `/trades/:id/mark-paid {proofUrl, notes}` |
  | `completeTrade` | POST `/trades/:id/complete` |
  | `cancelTrade` | POST `/trades/:id/cancel` |
  | `openDispute` | POST `/trades/:id/dispute` (409, or "from status disputed", counts as success: `isAlreadyDisputedError`) |
  | `uploadDisputeEvidence` | POST `/trades/:id/dispute/evidence` |
  | `listDisputeEvidence` | GET `/trades/:id/dispute/evidence` (no UI calls it) |
  | `uploadAttachment` | POST `/trades/:id/attachments` |
  | `submitRating` | POST `/trades/:id/rating` |
  | `downloadReceipt` | GET `/trades/:id/receipt.pdf` |
  | KYC calls | GET `/trades/:id/kyc`, POST submissions, POST review, GET files |
  | `getTradeEvents` | GET `/trades/:id/events` |
  | Messages | `/trades/:id/messages` |

  The backend also has **GET `/trades/:id/ledger`** (participant-only), which the frontend never calls.
- **Guards today:**
  - None of the trade actions sends a request key or idempotency key.
  - Mark paid ignores repeats only while `isPending`. **A failed mark-paid is silent:** `handleMarkPaidConfirm` does not await, so there is no toast.
  - Complete has a retry banner.
  - Cancel and dispute rely only on a disabled button.
- **Errors:** `getTrade` throws `new Error(message)`, so the page classifies by text and loses the HTTP status. It shows three different screens: "Trade Not Found", "Access Denied" and "Something Went Wrong".
- **Money shown:**
  - Shown: crypto and fiat amounts, `EscrowFeeDisclosure` (seller only, from `sellerEscrowFee`), the reseller breakdown (`resellerFeeBps` from the server) and the rate at creation.
  - **Not shown:** the held amount and the balance change. `escrowLockedAmount` and `escrowBuyerAmount` are in the response, but only the moderator page reads them.
- **Payment details** (bank account, reference): parsed from a `__type: "payment_details"` chat message inside `TradeChat.tsx` (`tryParsePaymentDetails`, L120).

### 2.2 `/trade-history`
- **Files:** `app/(dashboard)/trade-history/page.tsx` (505, inside `RequireAuth`) and `hooks/api/use-trade-history.ts` (326). Helpers: `lib/trade-pricing.ts` (`cryptoAmountForUser`), `lib/active-trade-counterparty.ts`, `hooks/api/use-backend-dashboard.ts` (`useTradingSummary`), `lib/reporting-money.ts`.
- **Data:** `GET /trades?limit=500`. The 500 cap is silent and there is no pagination.
- **Filters:** all client-side: type, status (eight raw labels), crypto (BTC/ETH/USDT/SOL), dates and search. **No URL query parameters are read or written.**
- **Status cell:** the raw enum, title-cased ("Escrow_funded").
- **Export:** `exportToCSV` and `exportTaxReport`, browser blobs. Neither escapes quotes inside cells.
- **Stats:** completed / total, computed client-side.
- Each row has a receipt button and a Chat button. There are no sockets and no polling.

### 2.3 `/trade-history/[id]`
- **Page:** `app/(dashboard)/trade-history/[id]/page.tsx` (381), which calls the mutations directly:
  - Mark paid: no confirm, and proof is optional.
  - Dispute: the reason is hard-coded to "Payment dispute".
  - Cancel: no reason is passed.
  - Chat: sends without a client id.
- **Components it alone uses:** `TradeDetailHeader` (66), `TradeStatusTimeline` (55), `TradeActions` (331), `TradeDetailsCard` (88; always "Unknown Seller/Buyer"), `ParticipantsCard` (55), `CounterpartyTradeHistory` (265; the only link into this route), `PaymentProofCard` (243).
- No redirect exists. `next.config.ts` redirects only the terms page, and there is no `middleware.ts` (`src/proxy.ts` is the staging gate only).
- `config/routes.ts` L734 to L735 gives both routes the phone header "Trades → /trade-history".

### 2.4 Status naming today (one status, many labels)
| Source | created | escrow_funded | paid | released | disputed | resolved | cancelled |
|---|---|---|---|---|---|---|---|
| `lib/state-machine/status-display.ts` | Created | Escrow Funded | Paid | Released | Disputed | Resolved | Cancelled |
| `TradeHeader` | Created | Awaiting Payment | Payment Sent | Completed | Disputed | Resolved | Cancelled |
| `TradeDetailHeader` | Awaiting Payment | Escrow Funded | Payment Sent | Completed | Disputed | Resolved | Cancelled |
| `TradeStatusBanner` | none | none | Payment Sent | Completed | Disputed | Resolved | Cancelled |
| `/trade-history` | Created | "Escrow_funded" | "Paid" | "Released" | … | … | … |

Notes:
- `status-display.ts` is used only by `marketplace/OffersList.tsx`, which belongs to 957.
- `TradeEventTimeline` labels `created`, `completed` and `disputed`, but the backend writes `trade_created`, `trade_completed` and `dispute_opened`, so those events render as raw words.
- `utils/format.ts` has **no** status map. The card's claim that it does is wrong.
- `ui/escrow-badge.tsx` is an *offer* escrow badge used only by `marketplace/offer-card/CreatorSection.tsx` (957). It is not a trade status source.
- Status types: `TradeState` is created, escrow_funded, paid, released, disputed, resolved or cancelled. Legacy values are pending, active and completed. `normalizeTradeStatus` maps unknown values to created.

### 2.5 Shared 906 pieces to reuse (do not rebuild)
- `ui/amount.tsx` (`Amount`; directions `in`, `out`, `held`, `neutral`; sizes `xs` to `xl`, `summary`, `inline`). It never parses with a float.
- `ui/status-chip.tsx` (tones success, warning, error, info, neutral, action and turn; sizes `md` and `compact` 22 px from 970-v52).
- `ui/empty-state.tsx`, `ui/error-state.tsx`, `ui/skeleton.tsx`.
- `ui/spinner.tsx` (`SpinnerRing`), `ui/loading-button.tsx`.
- `ui/progress.tsx` (`Steps`, `Step`).
- `ui/dialog.tsx` (`DialogContent` with `material: "glass" | "solid"` and `role: "dialog" | "alertdialog"`; a sheet below 768; `DialogShell`; `DialogFormError`), `ui/dialog-stack.ts`, `ui/confirm-dialog.tsx`, `ui/toast.tsx`.
- `ui/field.tsx`, `ui/checkbox.tsx` (`CheckboxRow`, 970-v52), `ui/amount-input.tsx` (970-v52), `ui/copy-button.tsx`, `ui/breadcrumb.tsx`, `ui/textarea.tsx`.
- Navigation: `navigation/PhoneHeader.tsx` (has a `trading` sub-header style with no right-hand actions), `BottomBar`, `Island`, `Capsule`, `JumpPalette`.
- `hooks/use-trades-badge.ts` (`isTradeWaitingOnUser`: the buyer when Funded, the seller when Paid). This is the board's "Your turn" rule.
- `hooks/use-bottom-bar-state.ts` (`useKeyboardOpen`).

### 2.6 Users of trade code outside these routes
- **Moderator page:** `app/(dashboard)/moderator/trades/[id]/page.tsx` uses `TradeEventTimeline`, `ProofOfPaymentDisplay` and `useTrade*`. Any restyle must stay under the `customer:` variant.
- **Navigation:** `BottomBar`, `Island` (`useTradesBadge`).
- **Socket provider:** `socket-provider.tsx` (invalidation).
- **Marketplace:** `marketplace/OffersList.tsx` (`getTradeStatusDisplay`, `/trade/:id` links).
- **Dashboard and profile:** `use-dashboard.ts`, `dashboard/ActiveTradesTable.tsx` (its own colour map), `use-profile-page.ts`, `profile/ActiveTrades.tsx` (its own label map).
- **Reseller:** `reseller/ResellerDashboard.tsx`.
- **Notifications:** `notifications/page.tsx`, `NotificationsDropdown.tsx` (they link to `/trade/:id`, not the legacy route).
- **Trade start:** the only path is `useCreateTrade`, from `hooks/api/use-offer-detail-page.ts` on `app/(offers)/offer/[id]/page.tsx`.
  - Insufficient balance is handled only before submit, as "Offer Unavailable".
  - Server errors go to `toast.error`.
  - Max uses `lib/effective-max.ts` at **70 bps**.

### 2.7 Existing tests that pin this code
- Component tests: `TradeHeader.test`, `TradeStatusBanner.test`, `buy-resell-display.test` (403), `participants-card-607`, `download-receipt-button-291`, `escrow-deposit-card-864`, `escrow-fee-disclosure-608`.
- Regression tests:
  - Disputes and cancels: `ticket-393-394-disputed-trade-actions`, `ticket-953-disputed-trade-no-cancel`, `ticket-689-dispute-cache`.
  - Room layout and dialogs: `ticket-800-trade-sidebar-fx-currency`, `ticket-trade-overflow`, `ticket-969-confirm-dialog-geometry`, `ticket-969-silent-toasts`.
  - Chat and proof: `ticket-SNtRWcvP-*`, `ticket-ev63zn6f-*`, `ticket-766-*`, `ticket-731-*`, `ticket-717-promote-proof`.
- History: `use-trade-history-volume-864`, `ticket-408-*`, `ticket-556-*`.
- E2E: `e2e/pages/trade.page.ts` (1,081) plus the specs listed in A-trading `tests.md` sections 1 to 3.
- **No test covers** `use-trade-detail-page.ts`, the mark-paid and complete guards, the history filters or the CSV export.

---

## 3. (c) Gap per board

| Board | What exists | Gap to close |
|---|---|---|
| A2, A3 (390 rooms) | Old header, banner, sidebar and chat stacked in one column; the same layout for both roles | **Phone header:** back "Trades", Help and ⋯; the room has no dock. **Title block:** "Buying 120.00 USDT from thabo_k", the id and one status chip. **Step bar:** "Step 2 of 4 · You pay" plus Next. **Next-step card per role and status:** `.ns` with the your-turn edge in --action-pressed. **Pay plate:** kv rows, Copy and Copy all, countdown plate with the under-5-minute look. **Locked note.** **Pinned action bar:** Chat with an unread badge, plus the primary action. **Below the fold:** latest message, trade summary (trade amount, price, you pay, pay by, escrow fee, held, balance change, started, reference), timeline, help line. **Outcomes:** two decision frames with "Where the 121.20 USDT went". **Cancelled card.** **Seller only:** Release early, the "Before you release" list (no checkbox) and the Open a dispute link |
| A4 to A7 (768, 1440) | Same single layout | **Island and capsule** (from 968). **Breadcrumb:** "Trades › Active or History › id" from the arrival query. **Title row:** chips and Get help. **Steps:** a labelled stepper with times. **1440:** two columns, with the chat as a full-height column. **768:** part 1 and part 2 layouts as drawn |
| A8, A9a to A9c, A19a, A19b (sheets and dialogs) | `ConfirmDialog` per action inside `TradeSidebar`; dispute reason has no limit; release has no tick; proof is required (kept) | **⋯ menu:** an action sheet on the phone, a glass menu on larger screens. **Rows:** Get help, Copy trade ID, View profile with stats, Report a problem (`/report?trade=<id>`), and Cancel trade (buyer, Funded only). **In dispute:** no Cancel, with the reason line. **Cancel after I've paid:** a glass refusal that offers a dispute. **Opaque commit steps** (`material="solid"`, `role="alertdialog"`): I've paid with To, Amount and Reference, proof required, upload-failed recovery, a 500-character note and the consequence; Release with a required tick and a −121.20 balance change; Early release; Cancel before paying (consequence plus the expiry time); Open a dispute with a 1,000-character counter, over-limit state, minimum 5 and the two-outcome copy. **Add evidence** as glass. **Content zone** and the 150 ms opacity-only motion for commit steps |
| A10, A11a to A11c, A20a to A20c (edge states) | Separate "Not Found" and "Access Denied" screens; the raw 401 proof error; the paused banner exists; KYC panels are old style; rating shows five stars when already reviewed | **One "We can't open this trade" message** for 403 and 404 (CH1). **Couldn't load** with Retry. **Static skeleton.** **Under 5 minutes** in m:ss. **"Cancelled: time ran out"** for both roles. **Proof can't load** as a plain failure with Retry. **Live updates paused** keeps its testid. **"Slow down a bit…"** copy. **Reseller watching** (read-only, commission after the 25% share). **Documents:** five frames; attempt 1 is a glass dialog with a reason, attempt 2 is an opaque dialog with its consequence "while no dispute is open". **Two "Cancelled · documents" cards.** **Rating:** stars first, "Send rating", and the real rating once sent |
| A21a, A21b, A22a to A22d (money outcomes) | Nothing: errors become toasts or an inline banner; no unknown state; no Check status | **Every outcome for I've paid, Release, Cancel and Open a dispute:** pending (button keeps its width, spinner, other controls locked), failed (what happened to the money, then why, then the next step; entered text kept), couldn't confirm, and Check status (reading, found, not found, offline). **Request reference** in the P-XXXX-XXXX format with the send time. **Start frames** as a component for 957 to mount |
| A12 (prototype) | none | The role and state rules feed the actions matrix and its tests (section 4, S1). Mirror `tools/proto_test.js` in the unit tests: two decisions only; no Release or Cancel for either side in dispute; a stale release step cannot release; Check status never calls the mutation |
| A13 to A15 (/trade-history) | Table with the crypto filter, raw status, receipt and chat per row, client stats, no query parameters | **Header:** Export and notifications. **Pills:** "Active (n open)" and "History". **Active list:** "Your turn · n" and "Waiting on others · n" groups, with action rows like "Pay thabo_k R 2,184.00". **History:** grouped by month; the right-hand figure is the **balance change** (open sale "80.80 USDT held", open buy "None yet", cancelled "None"). **Search** "Name or ID". **Filters:** glass sheet on the phone, popover at 768, inline bar at 1440: Type, Status (the 7 labels), Paid in (fiat), From and To, Clear all, "Show n trades". **Applied filter tags.** **Export sheet** with two CSVs. **Summary:** Trades, Completed, Completion (the backend rate, FE12), and Volume with its "Incomplete" note. **Table at 1440** with a receipt icon button on completed rows. **States:** no trades yet, no matches, loading, couldn't load. **320 px and 200% reflow.** **State in the URL** so Back from a trade restores the tab and filters |
| A16 (redirect) | A separate page with weaker checks | **308** from `/trade-history/:id` to `/trade/:id`, keeping the query. Delete the legacy page and its seven legacy-only components. A History row opens `/trade/[id]` directly with its arrival context |
| A17a, A17b (accessibility) | partial | **Focus colour by surface:** --focus, --foreground or --focus-on-ink. **Targets:** 44 px. **Reflow:** 320 px with a 24-character name and 12,480.00 USDT and no horizontal scroll; `.kv dd` wraps at max 58%; trade ids and amounts never break. **200%.** **Announcements** of money outcomes and of the two decisions. **Reduced motion:** the spinner turns still and Check status shows "Checking…". **Keyboard and sticky bar:** the focused control is never covered (scroll-padding against the pinned bar) |

Every row also takes the v5.2 control colours from qt.css and glass v3.2.1: current step, timeline dot and your-turn edge in --action-pressed; focused field and caret in --focus; the CTA stays --action. These come from the 970 tokens; nothing new is defined in 959.

---

## 4. (d) Slice plan

**Rules for every slice:**
- Each slice is one PR into `redesign/906`.
- Slices do not edit each other's files. Only S8 edits the barrel `components/features/trade/index.ts`; everyone else imports by path.
- `services/trades-api.ts`, `hooks/api/use-trades.ts`, `use-trade-detail-page.ts`, the socket code and proof fetching stay unchanged, as the card requires. The one new read (the ledger) goes in its own file.
- Every colour, size and radius comes from the 970 tokens and classes. There are no new tokens.
- Components that the staff moderator page shares take their new look only under `customer:`.
- To check boards before S3 lands, a builder uses a temporary local preview route with a seeded query cache, never committed (method in memory `qic-visual-verify`).

| Slice | Owns (file-disjoint) | Product lines | Test lines | Depends on |
|---|---|---|---|---|
| **S1 Trade model** (pure TS) | New `lib/trade/room-model.ts`, `money.ts`, `actions.ts`, `outcome.ts`, `timeline.ts` and `payment-details.ts`; rewrite of `lib/state-machine/status-display.ts` (the only label source, keeping the `getTradeStatusDisplay` signature that 957's `OffersList` uses); new `services/trade-ledger-api.ts` and `hooks/api/use-trade-ledger.ts` | ~650 | ~800 | none |
| **S2 Money commit core** | New `lib/trade/commit.ts`, `lib/trade/start-errors.ts`, `hooks/use-trade-commit.ts` and `components/features/trade/commit/*` (MoneyOutcomeBody, RequestReference, ConsequenceList, CommitFooter, StartOutcomeBody) | ~850 | ~750 | none (uses existing types) |
| **S8 Legacy merge and redirect** | `next.config.ts` (the 308); delete the `trade-history/[id]/page.tsx` page and TradeActions, ParticipantsCard, CounterpartyTradeHistory, TradeDetailHeader, TradeDetailsCard, TradeStatusTimeline, PaymentProofCard and EscrowDepositCard (each confirmed unused first), with their tests; the `features/trade/index.ts` barrel | ~60 added, ~2,100 deleted | ~150 | none |
| **S4 Status cards and summary** | New `components/features/trade/room/status/*`: NextStepCard and its per-status cards, PayPlate, Countdown, TradeSummary, WhereItWentCard, RoomTimeline (a new customer timeline; `TradeEventTimeline` stays for staff), LatestMessageCard, ProofRow | ~1,500 | ~800 | S1 |
| **S5 Commit steps** | New `components/features/trade/steps/*`: MarkPaidStep, ReleaseStep (with the early variant), CancelStep, CancelRefusal, OpenDisputeStep, AddEvidenceSheet, and the in-dispute menu note | ~1,500 | ~900 | S1, S2 |
| **S6 Chat, documents, rating and edge cards** (restyle in place) | `TradeChat.tsx`, `ChatAttachment.tsx`, `TradeKycPanel.tsx`, `TradeKycReviewPanel.tsx`, `TradeKycSubmittedFiles.tsx`, `TradeReviewForm.tsx`, `ProofOfPaymentDisplay.tsx`, `DisputeEvidenceUpload.tsx`; new `components/features/trade/room/reseller/*` | ~1,300 | ~600 | S1 (payment-details parser) |
| **S7 Trades list** | `app/(dashboard)/trade-history/page.tsx`; new `components/features/trade-history/*`; new `lib/trade-history/query.ts`; `hooks/api/use-trade-history.ts` | ~1,400 | ~700 | S1 |
| **S3 Room shell and composition** | `app/(main)/trade/[id]/*`; new `components/features/trade/room/{RoomPage,RoomTitle,RoomStepBar,RoomActionBar,RoomMoreMenu,RoomLayout,RoomStates}.tsx`; `components/navigation/PhoneHeader.tsx` (right-hand actions slot); `config/routes.ts` (the `/trade/[id]` back link built from a whitelisted `back` query, and removal of the dead `/trade-history/[id]` entry); deletion of TradeHeader, TradeStatusBanner, TradeSidebar and release-confirm-copy, with their tests moved or retargeted | ~1,300 added, ~1,400 deleted | ~600 | S1, S2, S4, S5, S6 |
| **S9 E2E and evidence** | `e2e/pages/trade.page.ts` and the specs in tests.md sections 1 to 3 (adjust, or delete the BTC, SOL and no-escrow specs: USDT only); new Playwright specs for lost responses, the role and state action matrix, IDOR, the 308, and 320, 200% and reduced motion | 0 | ~1,300 | S3, S7, S8 |

### 4.1 What each slice does

**S1 Trade model**
- **Role:** buyer, seller, reseller watching, or a non-participant staff viewer (read-only).
- **Labels:** the seven customer labels. Funded covers escrow_funded, and created maps to Funded because a trade is funded at creation. Paid shows as Payment sent, released as USDT released, legacy completed as Completed, disputed as In dispute, resolved as Dispute resolved, and cancelled as Cancelled. The match is exhaustive, and an unknown status shows no actions.
- **Per status:** chip tone, step bar (done, current, warn, stopped) and step caption.
- **Turn:** re-uses `isTradeWaitingOnUser` from 968's hook; it does not edit it.
- **Money:**
  - The four lines (trade amount, fee, held, balance change) are computed in integer micro-USDT from the server's `cryptoAmount`, `sellerEscrowFee.effectiveEscrowFeeAmount` (minor units), `escrowLockedAmount` and `escrowBuyerAmount`.
  - Settled trades read the trade ledger.
  - A missing value returns `null` ("Unavailable"), never 0.00.
- **Actions matrix:**
  - Mark paid: buyer, Funded.
  - Release: seller, Funded (early) or Payment sent.
  - Cancel: buyer, Funded only. The cancel refusal (pointing to a dispute) shows for the buyer at Payment sent.
  - Dispute: either party, Payment sent.
  - Nothing in dispute, after resolution, after completion or after cancellation. Nothing for the reseller or for staff viewers.
- **Dispute outcome:** escrow `released` reads "Released to the buyer"; `refunded` reads "Returned to the seller". Any other recorded settlement (a split made before 963 lands) shows its ledger amounts under "Dispute resolved" with no outcome name.
- **Timeline:** maps the backend event names `trade_created`, `trade_completed`, `dispute_opened`, `expired`, `cancelled`, `attachment_uploaded`, `counterparty_viewed`, `escrow_released` and `escrow_refunded`.
- **Payment details:** the parser is moved out of `TradeChat` into `lib/trade/payment-details.ts`.
- **Tests:** table tests over every role and status pair; property tests that held = amount + fee and that signs are right; a uniqueness test for the seven labels.

**S2 Money commit core**
- **Steps:** a pure reducer for idle, pending, failed(reason), unconfirmed, checking, found, notFound and offline.
- **Error classifier:**
  - No response, a timeout, or 502, 503 or 504 means **unconfirmed**.
  - A definite 4xx means **failed**, with a reason: a dispute came first, a release came first, time ran out, already done, or not enough USDT.
  - `isAlreadyDisputedError` means **found**.
  - Anything not understood means **unconfirmed**. The step never claims success or failure it cannot prove.
  - First build task: record the exact error shapes the five mutations throw today. `getTrade` already loses the status.
- **Request reference:** `T|M|R|X|D-XXXX-XXXX` plus the send time, kept from the first send to the last check.
- **One in-flight lock per step:** stops double clicks.
- **Check status:**
  - It is `queryClient.fetchQuery(tradeKeys.detail(id), {staleTime: 0})`: a GET only.
  - **found:** the status reached the action's target set.
  - **notFound:** only after the request's own timeout has passed since the send.
  - **offline:** the GET failed.
- **Retry:** offered only for the in-room actions whose backend state guard makes a repeat harmless, and only after notFound. It is never offered for Start (section 6).
- **Start errors:** `start-errors.ts` reads `insufficient_lock_balance_message` ("Required: X, available: Y") as Not enough USDT, with needed, available and short. A refused lock reads as The trade did not start. `StartOutcomeBody` draws the A21a frames for 957 to mount on `/offer/[id]`.

**S4 Status cards and summary:** one card per board frame for buyer and seller in every status, including:
- the pay plate, with the account details from the payment-details message
- the waiting and checking-bank cards
- released, with a slot for the rating
- completed, with the receipt
- in dispute, with the reason quote
- the two decision cards and Where-it-went
- the three Cancelled variants (you cancelled, time ran out, documents)

The summary lines follow the role copy exactly ("Escrow fee: Paid by the seller. 0.00 for you"; "Your balance change +120.00 USDT once released"). The `rate-at-creation` testid moves to the "Price, fixed at start" line.

**S5 Commit steps:** every A8, A9 and A19 frame, driven through `useTradeCommit` and `MoneyOutcomeBody`.
- **Mark paid:** proof upload calls `tradesApi.uploadAttachment` through the existing path; the file name survives a failed upload, with Try again and Choose another.
- **Text survives a recoverable failure** (dispute reason, note, evidence description).
- **Dispute reason:** 1,000 characters on the frontend. The backend still accepts 2,000 (card 921).
- **Release tick:** the confirm stays disabled until the tick is set.
- **Consequence first:** each consequence block renders before the confirm button.

**S6 Chat, documents, rating and edge cards**
- **Chat layout:** a full-height column at 1440 and a full-height view on the phone.
- **Chat details:** composer "Message {name}", the safety line, moderator bubbles, the paused livebar (keeps `chat-live-updates-paused`) and the "Slow down a bit…" copy.
- **Chat logic, reconciliation and rate limits** stay unchanged.
- **Documents:** the five frames; the rejection consequence is hidden when the trade is disputed.
- **Rating:** the A10 rating frames.
- **Proof can't load:** a plain failure with Retry.
- **Reseller:** the read-only reseller card.

**S7 Trades list:** the whole A13 to A15 set.
- **Query:** `lib/trade-history/query.ts` parses and writes `tab`, `type`, `status`, `paidIn`, `from`, `to` and `q`, and ignores unknown keys.
- **Exports:** the CSV exporters fix the unescaped-quote bug.
- **Completion:** from the backend completion rate (FE12) through the existing profile and stats hook.
- **Paid in:** filters on the `fiatCurrency` the rows already carry.

**S3 Room shell and composition**
- **Page:** keeps `useTradeDetailPage` and composes S4, S5 and S6.
- **Layouts:** 390, 768 and 1440, with the pinned phone action bar and scroll-padding so the keyboard and bar never cover the focused field.
- **States:** loading, couldn't load, and can't open (one message for 403 and 404).
- **Navigation:** the breadcrumb and back link from the arrival query.
- **Phone header:** Help and ⋯ go into the 968 phone header through a small actions slot.

**S8 Legacy merge and redirect**
- **Redirect:** `{source: '/trade-history/:id', destination: '/trade/:id', permanent: true}`. Next answers 308 and keeps the query.
- **Deletions:** the legacy page and its components.
- **Test rules carried over:** `ticket-393-394` is deleted; its D1 rule (no Cancel for anyone in dispute) is already covered by `ticket-953-disputed-trade-no-cancel` and S1's matrix tests.

### 4.2 Order and parallelism
1. **Wave A:** S1, S2 and S8 run in parallel. All three are independent.
2. **Wave B:** after S1 and S2 merge, S4, S5, S6 and S7 run in parallel.
3. **Wave C:** S3 (composition).
4. **Wave D:** S9, then merge to `redesign/906` and QA on the review app (owner rule 6).

S4 and S5 sit at the 1,500 cap. If either grows, split it on pre-agreed lines: S4a (buyer cards and summary) and S4b (seller cards and outcome cards); S5a (I've paid and Release) and S5b (Cancel, Dispute and Evidence).

---

## 5. (e) Acceptance mapping

"Blocked" means the case cannot be fully met by a frontend change; the backend card is named. Everything else is buildable now against today's API.

| AC | Cases | Evidence | Blocked by |
|---|---|---|---|
| **AC1** Start, pay, release, cancel and dispute on v5.2 screens | (1) Buyer and seller journeys with QA accounts on the review app: proof upload, I've paid, release, early release from Funded, expiry, cancel before payment, dispute with evidence, both document rejections before a dispute. (2) Every status for both roles at 390 x 844, 768 and 1440 x 900 against A2 to A7. (3) In dispute: no Cancel or Release in the room, menu or sheets for either role; the API refuses them | Playwright captures paired with board frames; network log; `GET /trades/:id/ledger` before and after | Funded **start** lives on 957's `/offer/[id]`; non-custodial starts: **910**. Refusal of cancel in dispute: 953 (merged, on staging). Seller take-back on a paid or disputed trade: **877** (962 already made the refund routes staff-only; confirm 877 can close). Moderator outcomes limited to two: **963** |
| **AC2** Amount, fee, held and balance change as the server reports them | For each step, the four figures equal `GET /trades/:id` (`cryptoAmount`, `sellerEscrowFee.effectiveEscrowFeeAmount`, `escrowLockedAmount`, `escrowBuyerAmount`) and, once settled, the trade ledger, to 6 decimals. Fiat is display only. The 120.00, 1.20 and 121.20 example is on the seller's room and Release step | Unit property tests; a QA table of API value against the value on screen | **918** (code default 70 bps; staging and production use 100 bps, so the review app shows 1.00%). The second bullet (insufficient balance or funding failure on every start route): **910**, and the screens are 957's. Required balance and Max include the fee after discount: **911** (on 957's offer page) |
| **AC3** Consequence first; no cancel or release in dispute; two outcomes | (1) The actions matrix per role and status matches A12 and the backend handlers; sellers never get Cancel. (2) A consequence block renders before every confirm. (3) No customer string, menu or label names a split (grep test). (4) An escalated dispute shows the same In dispute state (board rule). (5) No HARC or "Taking longer than usual" copy anywhere in the room (board rule; see drift) | Jest matrix and grep tests; review-app walk-through | **963**: until it ships a moderator can still split, and the room then shows the recorded ledger amounts under "Dispute resolved" with no outcome name. 953 (done) |
| **AC4** Lost response; no double action or false success | (1) Drop the response to I've paid, Release, Cancel and Dispute (Playwright `route.fetch()` then abort): the step stays open; Check status sends only GETs; found, not found and offline render; the ledger shows one action. (2) Double click, two tabs and a concurrent counterparty action end in found or failed, never a second action or a false success. (3) A non-participant gets "We can't open this trade" and every action call is refused (IDOR spec) | Request log in the spec (one POST per action); ledger rows | **920**: a lost **start** (no lookup by key), a retry "reusing the original request identity", and starts from two tabs. Per the card, AC4 stays unticked until 920 ships. The start screens are 957's |
| **AC5** Links, filters, chat, proofs, captures and accessibility | (1) Direct `/trade/[id]`. (2) `/trade-history` with the new query parameters (none exist today) round-trips and survives Back. (3) `/trade-history/[id]?x=1` answers 308 to `/trade/[id]?x=1`. (4) Chat recovers after a dropped socket (paused bar, rejoin). (5) Proof upload works. (6) Current, proposed and implemented captures for every changed page and state (965's register tooling). (7) WCAG 2.2 AA except the documented CTA contrast; 320 px; 200%; reduced motion; the sticky bar and the keyboard never cover the focused control | Captures; axe report; reflow and keyboard checks | **909**: the buyer gets 401 when downloading their own proof, so "proofs keep working" fails on viewing |

**Backend gaps with no card yet** (for the coordinator to ticket; drawn on the boards as BE17):
1. The moderator's note and decision text on a resolved trade. `dispute.resolution` is not exposed to participants, and resolve sends notifications only.
2. Timeline lines for "Dispute resolved", "balance restored" and document review (no trade event is written).
3. The short trade and payment reference "QT-7F3A21". The model has `payment_reference` ("QT-NAME-XXXXXX", used on receipts), but `TradeResponse` does not return it.

Until these exist, the room shows the existing id, omits the note row, and shows only the timeline lines that have events.

---

## 6. (f) Risks

**Money display**
1. `cryptoAmount` and `feeAmount` arrive as JSON floats. All arithmetic runs in integer micro-USDT. Minor-unit fields are preferred wherever the API has them (`sellerEscrowFee.*Amount`, ledger `amount`). Display goes through `Amount`, which never parses with a float.
2. A missing figure shows "Unavailable", never 0.00.
3. Fiat (R 2,184.00) is the server's `fiatAmount`, exact and without "≈".
4. The buyer never sees the fee as theirs.
5. The fee label uses `effectiveEscrowFeeBps` and names the merchant tier only when `merchantTier` is present.

**Wrong role, wrong actions**
1. The role comes from the signed-in user against `buyerId`, `sellerId`, `vendorId` and `resellerId`. Buy and sell offers swap maker and taker.
2. A reseller or a staff viewer with moderator access (GET is allowed) must get no actions.
3. The backend stays the authority; S1's matrix only hides controls.

**Authorization**
1. `(main)` has no `RequireAuth`.
2. Use one message for 403 and 404 (CH1) so the page never confirms that a trade id exists.
3. The new `back` query on the room must accept only `/trade-history` paths (no open redirect).
4. The unused ledger endpoint returns the counterparty's rows (`balance_after`). The UI shows only the viewer's rows. The exposure is a backend issue to raise; it is not introduced here.

**State display**
1. Legacy statuses (pending, active, completed) and unknown values are matched exhaustively.
2. `USDT released` is nearly transient: complete sets Completed in the same call.
3. A dispute escalated past its deadline looks the same as In dispute.
4. A split resolved before 963 shows its recorded amounts without an outcome name.
5. The expiry countdown uses `expiresAt` and can drift with the client clock. Show "pay by 14:47" beside it, as drawn.

**Lost responses without 920**
1. "Not found" is a judgement made after a timeout. A late request can still land, so the next socket update must win.
2. In-room retries rely on the backend state guards: release, mark paid and cancel are refused once the state has moved, and a dispute answers "already open".
3. A start retry is never offered (920).
4. Today's silent mark-paid failure is fixed by awaiting inside the commit hook. The mutation itself is unchanged.

**Real-time sockets**
1. `trade:update` only invalidates the cache, so a commit step must never close or show success because of a counterparty update.
2. "An early release came first" and "a dispute came first" must render as failed, with the reason.
3. Polling pauses during a commit, as today.
4. The chat logic (`message-reconciliation`, the REST fallback, rate limits) is restyle-only. The REST fallback drops `clientMessageId` today; this is recorded, not changed.

**Page-load side effects**
- A seller who opens a pending trade can make the page create an escrow or confirm a deposit (`use-trade-detail-page.ts` L154 to L286, L453 to L465).
- This stays unchanged as the card requires. For custodial trades it is unreachable, because they are funded at creation; it relates to 910.

**Staff pages**
- `TradeEventTimeline`, `ProofOfPaymentDisplay` and `TradeChat` are also used on `/moderator/trades/[id]`.
- Changes go under `customer:`, and S3 and S6 check the moderator page for regressions.

**History**
1. The 500-row cap is silent and unchanged; the board's stats assume all trades.
2. FE12 uses the backend completion rate, so History's figure can differ from what the page computes today.

---

## 7. (g) Card versus board drift (the boards win)

| # | Card text | v5.2 board | Boards |
|---|---|---|---|
| 1 | Buyer verb "I have paid" | "I've paid"; the confirm reads "I've paid R 2,184.00" | A2, A8, A12 |
| 2 | Seller verb "Release USDT" | "Release 120.00 USDT" (button with the amount); "Release" in the status model | A3, A8, A1a |
| 3 | "A HARC cancellation reads 'Cancelled, balance restored'…" and AC3 "HARC wording appears only after a confirmed refund" | No trade flow shows a held state or HARC copy. A cancel restores 121.20 in the same step ("121.20 USDT back in Available"). HARC is for held withdrawals only (B). The HARC frames were removed on 13 Oct | A1a, A1b, A10, A21a |
| 4 | "A dispute that passes its deadline is escalated … and the trade room says so"; AC3 "shows the escalated state" | The customer sees the same In dispute state; customer copy names no deadline; there is no escalated frame | A1a, A18a, design-notes gaps |
| 5 | AC1 "renders its v5.2 screen at 390 x 844 and 1440 x 900" | Boards also draw 768 for every status, sheet, edge state and outcome | A4, A5, A19, A20, A22 |
| 6 | AC1 "a moderator's **cancel** returns 121.20" | The outcome is named "Returned to the seller" (and "Released to the buyer"); the chip reads "Dispute resolved" | A2, A3, A1a |
| 7 | Fix sketch "keeping both URLs" | `/trade-history/[id]` becomes a 308 to `/trade/[id]`; the URL still works but the page is gone | A16 |
| 8 | AC5 "history filters … with existing query and filter parameters" | No query parameters exist today. The board adds tab and filter state to the URL and replaces the crypto filter with "Paid in" (M3) and the seven status labels | A13 to A15, A1a |
| 9 | Problem: status names differ in "`ui/escrow-badge` and `utils/format.ts`" | `utils/format.ts` has no status map. `escrow-badge` is an offer badge on marketplace cards (957). The trade label sources are the room components and `status-display.ts` | code |
| 10 | "Required balance and Max include the fee after any merchant discount"; "unfunded start blocked on every trade-start route" | The start step is drawn as a "layer for C" on `/offer/[id]`, and C owns its 768 and 1440 frames. The only start route in code is 957's offer page | A21a, A1b, C8, C22a, C22b |
| 11 | Expected behaviour omits it | Every pending, failed, unconfirmed and Check status step shows "Request reference X-XXXX-XXXX · sent hh:mm:ss" | A21a, A21b, A22 |
| 12 | Card does not mention them | Report a problem opens `/report?trade=<id>` (FE10; `/report` is 956's page); the rating restyle; Reseller watching; the 1,000-character dispute reason (M4; backend 921) | A8, A10, A18a |
| 13 | AC5 "WCAG 2.2 AA" | The CTA's white on --action (2.77:1) is a documented exception kept by Christian | A17b, DESIGN.md section 12 |

**Board versus board and board versus other sources**
1. A2's Cancelled frame labels ⋯ "…, cancel", but A8's "More menu · closed trade" has no Cancel. Follow A8 and drop the word from the label.
2. tests.md section 1 (14 Oct) still has "Cancel trade while In dispute … cancels". The 19 and 21 Oct rules (D1) supersede it.
3. A1b's toast lifetime is 4 s; the owner decided 6 s (22 Oct). Use 6 s.
4. The 969 release confirm was built to G16b at 520 px. Measure A9a and A19a at build time; if the widths differ, record it for Marcello instead of changing 969.
5. The backend intent doc (`intended-entity-state-machines.md` 1.1 design note and 4.2 "Split resolution", ticket 954) still describes a split. That is drift from the confirmed two-outcome rule; card 963 must update it. Flag it to the coordinator; 959 does not edit backend docs.

---

## 8. (h) Files other page-group cards also touch

| File | 959 slice | Other card | Coordination |
|---|---|---|---|
| `app/(offers)/offer/[id]/page.tsx`, `hooks/api/use-offer-detail-page.ts` | none (S2 provides `StartOutcomeBody` and `start-errors`) | **957** | 957 mounts the start outcomes there. Land S2 before 957 builds its start step, or 957 waits |
| `lib/effective-max.ts`, `__tests__/hooks/buy-max-fee-headroom.test.ts` | none | **957** (and 911) | The Max fee is not 959's |
| `lib/state-machine/status-display.ts` | S1 | **957** (`marketplace/OffersList.tsx` consumes it) | S1 keeps the `getTradeStatusDisplay` signature; 957 gets the new labels |
| `components/ui/escrow-badge.tsx`, `marketplace/offer-card/CreatorSection.tsx` | none | **957** | Leave to 957 |
| `next.config.ts` | S8 | **957** (`/prices*` 308s, C17) | Both append redirects. Merge in sequence, or have one PR add both entries |
| `config/routes.ts` | S3 | **968** (header config), **957** (named in its fix sketch) | S3 changes only the `/trade/[id]` and `/trade-history/[id]` entries |
| `components/navigation/PhoneHeader.tsx` | S3 (actions slot) | **968** (owner; follow-up branches 968-staff-menu-d0 and 968-support-chat-panel) | Land after those 968 branches merge |
| `hooks/use-trades-badge.ts` | read only | **968** | No edit; S1 re-uses the rule |
| Support chat panel (968) | S3 "Get help" | **968**, **956** | D1c note 11 (Get help opens support chat with the trade attached) was handed to 959. It needs the panel to accept a trade context, which is a 968 file. Coordinate one small PR |
| `app/(dashboard)/notifications/page.tsx`, `navigation/NotificationsDropdown.tsx` | none | **956** | They link to `/trade/:id` already; no change |
| `app/(main)/report/**` (`?trade=` prefill, FE10) | none (S3 only links) | **956** | 956 reads the query |
| `components/profile/ActiveTrades.tsx`, `hooks/api/use-profile-page.ts` | none | **956** | Should adopt S1 labels; 956 or 967 |
| `components/features/dashboard/ActiveTradesTable.tsx`, `hooks/api/use-dashboard.ts`, `reseller/ResellerDashboard.tsx` | none | **958** | Should adopt S1 labels; 958 or 966 |
| `app/(main)/trade/[id]/page.tsx`, `app/(dashboard)/trade-history/page.tsx`, `components/features/trade/*` | S3, S4, S5, S6, S7 | **967** (raw hex to tokens on the same files) | Run 967 after 959 merges and shrink its trade scope; 959 introduces no raw hex |
| `e2e/pages/offer-detail.page.ts`, `marketplace.page.ts` | none | **957** | C's page objects land before S9's `trade.page.ts` (tests.md order) |
| `e2e/visual/*` register | S9 adds the trade routes | **965** | 959 supplies the captures; 965 owns the tooling |
| **955** | none | **955** | No shared files. The only behaviour link is 955's AC3 (return to `/trade/[id]` after sign-in); `(main)` has no auth gate today |

---

## 9. (i) Size estimate

| Slice | Product lines (added or changed) | Deleted | Test lines |
|---|---|---|---|
| S1 | 650 | 0 | 800 |
| S2 | 850 | 0 | 750 |
| S3 | 1,300 | ~1,400 | 600 |
| S4 | 1,500 | 0 | 800 |
| S5 | 1,500 | 0 | 900 |
| S6 | 1,300 | ~300 | 600 |
| S7 | 1,400 | ~450 | 700 |
| S8 | 60 | ~2,100 | 150 |
| S9 | 0 | 0 | 1,300 (E2E) |
| **Total** | **~8,600** | **~4,200** | **~6,600** |

This is the largest 906 page card. The room has 8 statuses x 2 roles x 3 sizes, 47 sheet and dialog frames, 57 edge-state frames and 56 money-outcome frames, and the code it replaces is about 8,000 lines.

Expect about ±20% on S4 to S6 once each board is measured row by row.

Each slice needs the same replicate pass that 970 ran (board rows matched, fixed or recorded with a reason). That review time is not counted in these line figures.
