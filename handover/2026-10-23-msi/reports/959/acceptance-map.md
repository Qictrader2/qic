# 959 acceptance map (906-A: trade room, trade history and money steps)

Card: https://trello.com/c/SPVPnRAe (Development (In Progress) on 23 Oct 2026). Written 23 Oct 2026 from the plan, the slice reports S1 to S8, the S9 facts the overseer verified, the code in `.scratch/906-redesign/959-S9-root/frontend` (redesign/906 d60684b3 plus S9's uncommitted changes), the screenshots under `.scratch/959/reports/` and read-only Trello GETs. No code, commit, Trello write or sign-in was made.

Paths below:
- Unit tests are under `frontend/src/__tests__/`.
- e2e specs are under `frontend/e2e/tests/trade-room/` unless another folder is named.
- Images are under `/Users/starbird/Workspace/Code/qic/.scratch/959/reports/`.

The v5.2 boards are the spec. Where the card text and a board differ, the board wins (section 5).

## How to read the status column

| Status | Meaning |
|---|---|
| Met | The evidence passes now: unit tests and tier 1 e2e run locally. |
| Met pending QA | Built, unit tested and (where a spec exists) tier 1 tested. The live proof with QA accounts runs in review-app QA on https://qictrader-fe-pr-303.herokuapp.com. |
| Unticked: blocked by backend card N | The frontend part is built. The case cannot be proven until the named card ships. |
| Unticked: waits on 957 | Not a backend block. The screen lives on 957's `/offer/[id]`. |
| Open: frontend bug | A frontend defect fails the case today. None of the three statuses above fits it. |

## What the e2e tiers are

- **Tier 1** runs locally against `bunx next dev -p 3191`. Every `/api/v1` call is answered by fixtures (`e2e/helpers/trade-room-fixtures.ts`, `trade-room-harness.ts`), with a fake signed-in user and no real sign-in. Result: 110 of 112 passed. The two failures are known product bugs and are now marked `it.failing` (section 1, AC5).
- **Tier 2** is `trade-room-live.test.ts`, gated by `E2E_TRADE_ROOM_LIVE=1`. It has not been run. It runs in review-app QA with the QA accounts.
- **Legacy specs.** S9 rewrote `e2e/pages/trade.page.ts` to the v5.2 room and adjusted 21 specs and runners (listed in section 4). They need signed-in QA accounts on a live app, so they also run in review-app QA.

## What the S9 build captures are

`s9-shots/build-<width>-<role>-<status>.png` were taken by `trade-room-evidence-shots.test.ts` (gated by `E2E_TRADE_ROOM_SHOTS=1`) on the final code, driven by mocked API fixtures with the socket blocked. Every room capture therefore shows the "Live updates paused" bar in the chat. They are not live-socket captures.

---

## 1. Acceptance criteria

### AC1. Buyers and sellers can start, pay, release, cancel and dispute trades on the approved v5.2 trade screens on phone and desktop.

| # | Case (plan section 5) | Unit tests | e2e | Board vs build (390 / 768 / 1440) | Status |
|---|---|---|---|---|---|
| 1.1 | In-room journeys on QA accounts: proof upload, I've paid, release, early release from Funded, expiry, cancel before payment, dispute with evidence, both document rejections before a dispute | `components/trade-steps/mark-paid-step.test.tsx`: "ready: uploads through the given path, shows the file and enables I've paid R 2,184.00", "upload failed: keeps the file name, says what to do, and Try again uploads the same file", "the expiry job coming first reads Time ran out, with Get help and Message thabo_k". `release-step.test.tsx`: "ReleaseStep: early release from Funded" / "warns first, with its own title and the expiry line". `cancel-step.test.tsx`: "an opaque alertdialog: the money first, the expiry time, then Keep the trade and Cancel trade". `dispute-step.test.tsx`, `evidence-sheet.test.tsx`: "uploads each file once with the trimmed description, then closes, clears and reports how many went in". `components/trade-docs/trade-kyc-panels-959-s6.test.tsx`: "rejects attempt 1 in a glass dialog that needs a reason and keeps errors inside it", "rejects the last attempt in an opaque alertdialog that names the 121.20 USDT consequence". `components/trade-room/room-page.test.tsx`: "commit steps open from their openers and send nothing until confirmed (959-S5 wiring)" | Tier 1 `trade-room-action-matrix.test.ts`: "confirm steps at $width px" / "$role at $status, $kind: the consequence is above the confirm button". Legacy specs adjusted by S9 and pending QA: `dispute-flow.test.ts`, `complete-trade-lifecycle.test.ts`, `two-browser-trade-flow.test.ts`, `single-browser-trade-flow.test.ts`, `trade-flow-with-escrow.test.ts`, `journey/phase2-trade-lifecycle.ts`, `reviews-ratings.test.ts` | Step frames: `s5-shots/boards/A8-*` with `s5-shots/390-*`, `s5-shots/boards/A19a-*` with `s5-shots/768-*`, `s5-shots/boards/A9a-*` / `A9b-*` with `s5-shots/1440-*`. Documents: `s6-shots/b-board-docs-*` with `s6-shots/b-ours-docs-*` | Met pending QA |
| 1.2 | Every trade status for buyer and seller renders its v5.2 screen at 390 x 844, 768 and 1440 x 900 (A2 to A7) | `components/trade-room/room-page.test.tsx`: "every status for both roles at 390 (A2, A3)" / "%s, %s: title, chip, step caption and the pinned bar", "768 and 1440 (A4 to A7)". `components/trade-room-status/status-cards.test.tsx`, `seller-outcome-cards.test.tsx`. `lib/state-machine/status-display-959.test.ts`: "the seven customer labels" / "are exactly the A1a set, in order, each unique" | Tier 1 `trade-room-action-matrix.test.ts`: "actions matrix at $width px" / "$role at $status: chip, commit openers, ⋯ menu and copy" (buyer and seller x 8 statuses x 390/768/1440). Captures: `trade-room-evidence-shots.test.ts` | All 48 pairs in section 2. Example: `s3-shots/board_room-buyer-390_0_funded.png` with `s9-shots/build-390-buyer-funded.png` | Met pending QA. Visible gaps are backend-blocked (975 reference, 973 moderator note, 981 buyer "Where it went", 987 legal name, 974 timeline lines) or fixture-driven (section 2) |
| 1.3 | In dispute, no Cancel or Release in the room, menu or sheets for either role, and the API refuses them | `lib/trade/actions.test.ts`: "prototype rules (proto_test.js)" / "in dispute neither side can release, release early or cancel". `regression/ticket-953-disputed-trade-no-cancel.test.tsx`: "#953 disputed trade has no cancel for either party". `components/trade-room/room-more-menu.test.tsx`: "adds S5's reason line in dispute, for either party only". `status-cards.test.tsx`: "offers no Cancel or Release in dispute at either placement" | Tier 1 matrix: the in-dispute rows assert no Cancel or Release in the ⋯ menu and the D1 note. API refusal: tier 2 and legacy `dispute-flow.test.ts`, pending QA (953 is Deployed (Monitoring)) | `s3-shots/b1/board_a8_menu_dispute.png` with `s3-shots/b1/build_a8_menu_disputed.png`; `s3-shots/b1/board_a9_menu_dispute_768.png` with `build_menu_768_disputed.png`; `board_a19_menu_dispute_1440.png` with `build_menu_1440_disputed.png` | Met pending QA |
| 1.4 | A moderator's decision: returned to the seller gives 121.20 back with no fee row; released to the buyer matches a normal completion in the ledger | `lib/trade/outcome.test.ts`: "has exactly two outcomes, no split", "a split settlement (released with a split: hash) has no outcome name". `lib/trade/money.test.ts`: "Dispute resolved, returned to the seller: 0.00 and no fee", "Dispute resolved, released to the buyer: −121.20 and the fee charged" | None run. The ledger before and after needs a moderator on the review app | `s3-shots/board_room-seller-390_6_dispute-resolved-returned-to-the-seller.png` with `s9-shots/build-390-seller-resolved-returned.png` (matches, minus the moderator note) | Unticked: blocked by backend card 963. Until it ships a moderator can still split, and the room then shows ledger amounts with no outcome name |
| 1.5 | Funded start | `lib/trade-start-errors.test.ts`, `components/money-commit-outcomes.test.tsx` (S2's StartOutcomeBody for 957 to mount) | None. The start screen is 957's `/offer/[id]` | `s2-shots/390-start-*` and `1440-start-*` (S2's component frames, not the page) | Unticked: blocked by backend card 910 (non-custodial starts can leave a trade unfunded; in Development). The screen also waits on 957 |

### AC2. Each trade step shows the trade amount, fee, held USDT and balance change exactly as the server reports them.

| # | Case | Unit tests | e2e | Board vs build | Status |
|---|---|---|---|---|---|
| 2.1 | Each money step's four figures equal `GET /trades/:id` (`cryptoAmount`, `sellerEscrowFee.effectiveEscrowFeeAmount`, `escrowLockedAmount`, `escrowBuyerAmount`) and, once settled, the trade ledger, to six decimals. Fiat is display only. 120.00, 1.20 and 121.20 on the seller's room and Release step | `lib/trade/money.test.ts`: "the board example: seller (A3, A7)" / "Funded: 120.00, fee 1.20, held 121.20, −121.20 once released"; "the board example: buyer (A2, A6)" / "never shows the fee as the buyer's, in any status, even if a quote were present"; "usdtToMicro returns null for missing or unusable values, never 0"; "microToDecimalString prints 2 to 6 decimals without a float". `release-step.test.tsx`: "shows Unavailable (never 0.00) when the fee quote is missing". `components/trade-room/room-proof-and-figures.test.tsx`: "the pay figure and the summary's fiat rows go through Amount" | Tier 1 captures show the fixture's server figures (120.00 / 1.20 / 121.20) on every status. The QA table of API value against screen value runs in review-app QA | `s3-shots/board_room-seller-768_1_payment-sent.png` with `s9-shots/build-768-seller-payment-sent.png`: summary rows match line for line | Met pending QA. 918 (Backlog) is still open: the code default is 70 bps, staging and production use 100 bps. The room reads the server's quote, so 918 changes the source value, not the display. Known: a buyer's cancel outcome reads "Unavailable" for the fee because only the seller gets the quote (981). Trades-list seller rows read "Unavailable" because `GET /trades` sends no `sellerEscrowFee` (no card yet, S7) |
| 2.2 | Insufficient balance and funding failure block the start with the specified message on every start route; required balance and Max include the fee after discount | `lib/trade-start-errors.test.ts`: "reads not enough USDT with needed, available and short", "reads a refused lock as did not start". `hooks/use-trade-commit.test.tsx`: "a start answer reads not enough USDT with its figures" | None. The routes are 957's | `s2-shots/1440-start-seller-notenough.png`, `390-start-*-didnotstart.png` (component frames) | Unticked: blocked by backend cards 910 (funding failure on non-custodial starts) and 911 (Max assumes 0.7%; Backlog). The screens are 957's |

### AC3. Before any cancel, dispute or release, the customer sees what happens to the USDT. Once a dispute opens nobody can cancel or release, and the only outcomes shown are cancel to the seller or release to the buyer.

| # | Case | Unit tests | e2e | Board vs build | Status |
|---|---|---|---|---|---|
| 3.1 | The actions matrix per role and status matches A12 and the handlers; sellers never get Cancel; reseller and staff viewers get nothing | `lib/trade/actions.test.ts`: "availableTradeActions: 4 roles x 8 statuses", "the seller never gets Cancel or the cancel refusal, in any status", "Cancel is buyer-only and Funded-only; Payment sent offers the refusal instead". `room-more-menu.test.tsx`: "never gives the seller a Cancel row (%s)", "gives a reseller or a staff viewer no profile and no Cancel" | Tier 1 `trade-room-action-matrix.test.ts` (84 tests): "actions matrix at $width px" for buyer and seller x 8 statuses at 390/768/1440 plus reseller and staff viewer; "buyer at Payment sent: Cancel trade opens the refusal, which offers a dispute and cannot cancel" | `s3-shots/b1/board_a8_menu_funded.png` with `build_a8_menu_funded.png` | Met |
| 3.2 | A consequence block renders before every confirm | `mark-paid-step.test.tsx`: "puts the consequence before the confirm and describes the alertdialog with it". `release-step.test.tsx`: "not ticked: the consequence and the money first, then the tick, and Release disabled" | Tier 1 matrix: "confirm steps at $width px" / "$role at $status, $kind: the consequence is above the confirm button"; "$kind stays disabled until the box is ticked" | `s5-shots/boards/A8-release-not-ticked.png` with `s5-shots/390-release-not-ticked.png`; `A19a-release-not-ticked.png` with `768-release-not-ticked.png`; `A9b-release-not-ticked.png` with `1440-release-not-ticked.png` | Met |
| 3.3 | No screen, menu, status label or prototype names a split outcome | `lib/trade/outcome.test.ts`: "has exactly two outcomes, no split". `evidence-sheet.test.tsx`: "never shows HARC or split wording" | Tier 1 matrix: every room, menu and step text passes the no-split and no "Taking longer than usual" check (`noForbiddenCopy`) | All resolved pairs in section 2 show only "Released to the buyer" or "Returned to the seller" | Met. The backend can still record a split until 963; the room then shows the amounts with no outcome name |
| 3.4 | A dispute past its deadline shows the same In dispute state with the funds held (board rule, drift item 4) | `status-cards.test.tsx`: "NextStepCard: buyer In dispute" / "quotes the reason, has the warning edge and no your-turn edge". `money.test.ts`: "In dispute: −121.20 if released, 0.00 if returned" | Tier 1 matrix in-dispute rows. A real escalated trade (953 job) is checked in QA | `s3-shots/board_room-seller-390_4_in-dispute.png` with `s9-shots/build-390-seller-in-dispute.png` | Met pending QA |
| 3.5 | HARC wording only after a confirmed refund. The boards removed HARC from trades (drift item 3), so no trade screen shows it | `lib/trade-commit-check.test.ts`: "no trade copy uses HARC words" | Tier 1 matrix: no "Taking longer than usual" copy on any status or step | Cancelled pairs: `s3-shots/board_room-seller-390_7_cancelled.png` with `s9-shots/build-390-seller-cancelled.png` ("121.20 USDT back in Available") | Met |

### AC4. When a trade action's response is lost, the customer can check the result without repeating it, and no action runs twice or reports false success.

| # | Case | Unit tests | e2e | Board vs build | Status |
|---|---|---|---|---|---|
| 4.1 | Drop the response to I've paid, Release, Cancel and Open a dispute: the step stays open, Check status sends only GETs, found / not found / offline render, one action only | `hooks/use-trade-commit.test.tsx`: "Check status only reads: found, unconfirmed before the timeout, not found after it, offline on a failed GET", "never calls a request lost while the first send is still running (H1)". `lib/trade-commit-classifier.test.ts`: "classifyCommitError (Step 0 shapes)". `lib/trade-commit-check.test.ts`: "matchCheck: every action x every trade status". Per step: `cancel-step.test.tsx` "CancelStep: a lost response (A21b, A22)", `release-step.test.tsx` "ReleaseStep: a lost response (A21b, A22)", `mark-paid-step.test.tsx` "MarkPaidStep: a lost response (A21a, A22)" | Tier 1 `trade-room-lost-response.test.ts` (16 tests): "lost answer to $kind ($role at $from)" / "keeps the step open on \"We couldn't confirm\", offers no resend, and Check status finds it with one GET and no second POST"; "reads \"hasn't reached us\" only once the request timeout (30 s) has passed, and never resends" (Playwright clock); "renders the offline result when Check status cannot reach QicTrader, and sends nothing". It also asserts the request reference is unchanged. Tier 2 `trade-room-live.test.ts`: "I've paid with a lost answer: the step stays open, Check status reads it as sent, one POST", "Release with a lost answer: Check status finds it Completed, and the ledger shows one release" (real `route.fetch()` then `route.abort()`, ledger before and after) | `s5-shots/boards/A21a-i-ve-paid-couldn-t-confirm.png` with `s5-shots/390-paid-unconfirmed.png`; `A22a-release-couldn-t-confirm-768.png` with `768-release-unconfirmed.png`; `A22b-release-it-went-through-1440.png` with `1440-release-found.png`; cancel and dispute: `s5-shots/phase-b/*-cancel-*`, `*-dispute-*` | Met pending QA (tier 2 ledger proof) |
| 4.2 | Drop the response to a start | `lib/trade-start-lookup.test.ts`: "parseStartLookup: the lookup's body, read strictly". `hooks/trade-start-request-key.test.tsx`: "a 5xx, Check status by the same key, then Start again posts the SAME body under the SAME key; the replayed first trade lands as succeeded". `use-trade-commit.test.tsx`: "a start's Check status reads its request key, never the trades list, and never posts" | None. No start screen in this card | `s2-shots/*-start-*` (component frames from before S2w) | Unticked: waits on 957. 920 is live (Deployed (Monitoring)), and S2w wired the start key and `GET /money-requests/{key}`. What stays open is 957 mounting `StartOutcomeBody` on `/offer/[id]` |
| 4.3 | Double click, two tabs and a concurrent counterparty action never give a second action or a false success | `use-trade-commit.test.tsx`: "a double confirm while pending sends once", "a manual cache write (not a GET) never closes the step", "a GET showing a blocking status moves 'hasn't reached us' to failed, as a read (M4, L4)", "a remount during an unconfirmed step shows the same step and reference and can never confirm again (item 1)". `cancel-step.test.tsx`: "an early release came first: failed, +120.00 USDT, You still owe, ...". `dispute-step.test.tsx`: "a release came first: failed, ..." | Tier 1 `trade-room-lost-response.test.ts`: "sends one POST for a double click on the confirm button" (4 actions). Tier 2: "a double click on Cancel trade sends one POST and returns the lock once". No e2e spec covers two tabs or a live counterparty action yet | Not a visual case | Met pending QA. Gap: two tabs and the concurrent counterparty action rest on unit tests and the backend state guards; add them to the review-app QA script |
| 4.4 | A non-participant cannot open or act on another customer's trade | `room-page.test.tsx`: "shows one Can't open for 403 and 404 alike, never naming which" | Tier 1 `trade-room-cant-open.test.ts`: "403 and 404 read the same one message, with no trade data, at %i px". Tier 2: "a non-participant is refused on every read and every action, and the room shows one message" (third QA account, GETs 403/404 and refused POSTs) | `s3-shots/board_room-states-390_2_can-t-open.png` with `s9-shots/build-390-buyer-cant-open.png`; `board_room-states-768-1_2_can-t-open.png` with `build-768-buyer-cant-open.png`; `board_room-states-lg-1_2_can-t-open.png` with `build-1440-buyer-cant-open.png` | Met pending QA (tier 2 IDOR) |

Request keys on the in-room actions: 920 keys only the trade start. I've paid, Release, Cancel and Open a dispute send no key, so Check status reads the trade and the reference is display-only (S2w "920 follow-up", no card yet). "One action only" for those four rests on the backend state guards, which tier 2 checks against the ledger.

### AC5. Existing trade links, history filters, chat and proof uploads keep working, with current, proposed and implemented captures for every changed trade page and state.

| # | Case | Unit tests | e2e | Board vs build | Status |
|---|---|---|---|---|---|
| 5.1 | Direct `/trade/[id]`; `/trade-history/[id]?x=1` answers 308 to `/trade/[id]?x=1` | `regression/ticket-959-s8-trade-history-redirect.test.ts`: "redirects() serves exactly one /trade-history/:id -> /trade/:id 308 rule with no has/missing/statusCode", "has no legacy detail page and no route-table row for /trade-history/[id]". `lib/trade-room-shell-model.test.ts`: "back whitelist (no open redirect)" | Tier 1 `trade-room-cant-open.test.ts`: "the old trade detail route" / "answers 308 to /trade/<id>, keeping the query", "answers 308 for the trailing-slash form too". S8 also checked it live with `next start` and curl | A16 (`~/Downloads/906-design-pack-v5.2/906-v5.2-A16-history-id-redirect.pdf`) is a rule board; no frame to pair | Met |
| 5.2 | `/trade-history` with the new query parameters round-trips and survives Back | `trade-history/query.test.ts`: "serializeTradesQuery" / "writes keys in a fixed order and round-trips", "parseTradesQuery" / "ignores unknown keys"; "tradeRoomHref" / "opens the room directly with the encoded Trades view as back". `trade-history/trades-page.test.tsx`: "links each row straight to the room with the Trades view as back", "shows the Filtered frame: tags, the match count and the matching rows" | No S9 spec for the list. Walk-through in QA | `s7-shots/boards/history-390.png` with `s7-shots/build-390-history.png`; `history-768.png` with `build-768-history.png`; `history-1440.png` with `build-1440-history.png` | Met pending QA |
| 5.3 | Chat reconnects after a dropped socket | `regression/ticket-SNtRWcvP-trade-chat-rest-send-socket-down.test.tsx`: "reconciles the REST-confirmed message by server id with no duplicate, then survives reconnect". `components/trade-chat/trade-chat-layouts-959-s6.test.tsx`: "live updates paused keeps its testid and adds ticket 733's specific reason". `regression/ticket-731-join-trade-ack.test.ts` | Legacy e2e `e2e/tests/regression/ticket-SNtRWcvP-chat-socket-blocked.test.ts`, `e2e/tests/trade-chat.test.ts` (selectors updated by S6), pending QA. Every S9 capture shows the paused bar because the socket is blocked | `s3-shots/board_room-states-lg-1_7_live-updates-paused.png` with `s6-shots/col-1440-buyer-paused.png` | Met pending QA |
| 5.4 | Proof upload and viewing keep working | Upload: `mark-paid-step.test.tsx` (above). Viewing: `components/trade-docs/proof-and-reseller-959-s6.test.tsx`: "shows the plain failure, never the 401, on the customer surface and retries". `room-proof-and-figures.test.tsx`: "A10 Proof can't load: the buyer's 401 (909) is the plain failure with Retry, in the card" | Legacy specs pending QA. The S9 fixtures have no proof attachment message, so no S9 capture shows a proof row | `s6-shots/b-board-proof-390.png` with `s6-shots/b-ours-proof-390.png` (and 768, 1440) | Upload: Met pending QA. Viewing: Unticked: blocked by backend card 909 (the buyer gets 401 on their own proof; Backlog) |
| 5.5 | Current, proposed and implemented captures for every changed trade page and state | Not unit tested | `trade-room-evidence-shots.test.ts` (53 captures) | Proposed: board renders in `s3-shots/board_*`, `s5-shots/boards/`, `s6-shots/b-board-*`, `s7-shots/boards/`. Implemented: `s9-shots/`, slice captures in `s2-shots` to `s7-shots`. Current (before): only `s3-shots/b2/before_*` (10 frames) | Met pending QA. Still needed: the 965 register run with live-socket captures on the review app, and "current" captures for the states not in `b2/` |
| 5.6 | WCAG 2.2 AA, 320 px, 200% zoom, reduced motion; the sticky bar and keyboard never cover the focused control | `room-more-menu.test.tsx`: "puts Help (20 px icon) and ⋯ (24 px icon, 44 px target) in the phone header, not in the page". `components/money-commit-outcomes.test.tsx`: "Live regions (A17b, round 2 item 4)". `dispute-step.test.tsx`: "OpenDisputeStep: what the counter announces (A8 filled, A17a)" | Tier 1 `trade-room-reflow-motion.test.ts` (8): passing "320 px: a 24-character username and 12,480.00 USDT reflow with no sideways scroll, and amounts never break", "200% zoom (640 x 400 CSS px at device scale 2): no sideways scroll in the room or its Release step", "the Check status spinner is still and the button reads \"Checking…\" under prefers-reduced-motion", "never covers the focused control while tabbing through the room ($name)". `it.failing`: "320 px: nothing in the room runs past the right edge (the room clips, so this reads the boxes)" and "the phone bar's Release label groups 12,480.00 like the Release step". No axe run yet | `s3-shots/board_a11y-2_3_320-px-long-name-large-amount.png` with `s9-shots/build-320-seller-payment-sent-long-name.png`; `s3-shots/board_a11y-2_1_200-zoom-first-screen.png` with `s9-shots/build-640-seller-payment-sent-zoom200.png` | Open: frontend bug. Known: `TradeSummary.tsx` L41 and `status-cards-model.ts` L115. New from the 320 capture: `HelpLine.tsx` L32 (section 2). Axe (WCAG 2.2 AA) still has to run; the white-on-action CTA at 2.77:1 is the documented exception |

### Totals

22 cases:
- **Met: 5** (3.1, 3.2, 3.3, 3.5, 5.1).
- **Met pending QA: 11** (1.1, 1.2, 1.3, 2.1, 3.4, 4.1, 4.3, 4.4, 5.2, 5.3, 5.5).
- **Unticked, blocked by a backend card: 4** (1.4 by 963, 1.5 by 910, 2.2 by 910 and 911, 5.4 viewing by 909).
- **Unticked, waits on 957: 1** (4.2).
- **Open, frontend bug: 1** (5.6).

5.4 counts once, as Unticked: its upload half is Met pending QA.

Per AC:
- AC1 and AC2 cannot be ticked until 963, 910 and 911 ship.
- AC3 can be ticked after review-app QA confirms 3.4.
- AC4 can be ticked once 957 mounts the start outcome and tier 2 passes.
- AC5 needs 909 and the three reflow fixes.

---

## 2. Board vs build pairing

Board renders come from S3's `s3-shots/board_room-*` (A2 to A7), `board_room-states-*` (A10, A11a, A20a) and `board_a11y-2_*` (A17b). Every S9 frame has a board render, so no PDF fallback was needed. The 1440 board renders are viewport-only (1440 x 900), so for content below the fold use `906-v5.2-A6a/A6b/A7a/A7b-*.pdf`.

I put each pair side by side, board on the left (above at 1440), and looked at 45 of the 53. I did not open these 8, all 1440: buyer usdt-released, in-dispute, resolved-released, cancelled; seller funded, usdt-released, completed, resolved-returned. Their 768 counterparts, which use the same components, were all checked.

### Differences that are not bugs

The fixtures or a backend card cause these. "As F-notes" in the table means that pair shows only these:

| Code | Difference | Cause |
|---|---|---|
| F1 | The 36-character trade id (wraps) where the board prints QT-7F3A21, in the title, Reference rows, breadcrumb and copy | Backend 975 |
| F2 | No proof row in the Payment sent card and no "Proof of payment" card in dispute or resolved | Fixture: `messagesJson` holds only the payment-details system message, no proof attachment message |
| F3 | No latest-message card at 390 and no chat bubbles in the column; the chat column shows the paused bar | Fixture (no chat messages) and the blocked socket |
| F4 | No "Download receipt" on Completed | Fixture: `GET /users/me` sends no `featureFlags`, so the `trade_receipt` flag is off (`DownloadReceiptButton.tsx`) |
| F5 | Seller Completed shows "Rate naledi_m" where the board shows "Your rating" sent | Fixture has no rating (backend 991 for the read-back) |
| F6 | No "Moderator's note" block | Backend 973 |
| F7 | Buyer's "Where the 121.20 USDT went" missing (390 card below the fold; 768/1440 rows inside the decision card) | Backend 981 |
| F8 | "naledi_m's name" where the seller's board says "N Mokoena's name" | Backend 987 |
| F9 | No "Dispute resolved ..." timeline line; usdt-released shows an extra "Completed" line | Backend 974; the fixture's usdt-released events include `trade_completed` |
| F10 | Times, countdown and counterparty stats differ (09:23 vs 14:17, 98% · 412 for both roles) | Fixture clock and profile |

### Verdict per pair

Format: board image, build image, verdict.

**390** (board `s3-shots/board_room-<role>-390_<n>_<status>.png`, build `s9-shots/build-390-<role>-<status>.png`)

| Role / status | Verdict |
|---|---|
| buyer funded | Matches; F1, F3, F10 |
| buyer payment-sent | Matches layout and copy; F1, F2 (no proof row), F3 |
| buyer usdt-released | Matches; F1, F3, F9 |
| buyer completed | Matches; F1, F3, F4 |
| buyer in-dispute | Matches; F1, F2, F3; the reason quote omits "at 14:31" because the fixture reason has no time |
| buyer resolved-released | Matches card and bar; F6, F7 (no Where it went card), F3 |
| buyer resolved-returned | Matches including "Think this is wrong? Get help" and the Get help bar; F6, F7, F3 |
| buyer cancelled | Matches; F1, F3 |
| seller funded | Matches, including "Shown to naledi_m", Release early and the help line; F1, F3 |
| seller payment-sent | Matches; F1, F2, F3, F8 |
| seller usdt-released | Matches; F1, F3, F9 |
| seller completed | Matches; F1, F3, F4, F5 |
| seller in-dispute | Matches; F1, F2, F3 |
| seller resolved-released | Matches, including Where it went inside the card; F6, F3 |
| seller resolved-returned | Matches; F6, F3 |
| seller cancelled | Matches; F1 |

**768** (board `s3-shots/board_room-<role>-768[-2]_<n>_<status>.png`, build `s9-shots/build-768-<role>-<status>.png`)

All 16 match in layout (title row with chips, labelled stepper, card and chat column, summary, timeline), apart from the F-notes:

| Role / status | F-notes |
|---|---|
| buyer funded | F1, F3 |
| buyer payment-sent | F2 |
| buyer usdt-released | F9 |
| buyer completed | F4 |
| buyer in-dispute | F2 |
| buyer resolved-released | F6, F7, F2 |
| buyer resolved-returned | F6, F7, F2 |
| buyer cancelled | F1 |
| seller funded | F1 |
| seller payment-sent | F2, F8 |
| seller usdt-released | F9 |
| seller completed | F4, F5 |
| seller in-dispute | F2 |
| seller resolved-released | F6, F2 |
| seller resolved-returned | F6, F2 |
| seller cancelled | F1 |

Every 768 build also shows the item 2 chat card listed below.

**1440** (board `s3-shots/board_room-<role>-1440[-2]_<n>_<status>.png`, build `s9-shots/build-1440-<role>-<status>.png`)

| Role / status | Verdict |
|---|---|
| buyer funded | Matches (two columns, chat as a full-height column); F1, F3 |
| buyer payment-sent | Matches; F2, F3 |
| buyer completed | Matches; F4 (View in wallet goes full width with no receipt beside it) |
| buyer resolved-returned | Matches card; F6, F7, F2 |
| seller payment-sent | Matches; F2, F8 |
| seller in-dispute | Matches; F2 |
| seller resolved-released | Matches, including Where it went rows in the card; F6, F2 |
| seller cancelled | Matches |
| 8 others | Not opened (see above) |

**States and accessibility**

| Pair | Verdict |
|---|---|
| `board_room-states-390_2_can-t-open.png` / `build-390-buyer-cant-open.png` | Matches (one message, Go to Trades, Get help) |
| `board_room-states-768-1_2_can-t-open.png` / `build-768-buyer-cant-open.png` | Matches; the breadcrumb shows the UUID (F1) |
| `board_room-states-lg-1_2_can-t-open.png` / `build-1440-buyer-cant-open.png` | Matches; F1 |
| `board_a11y-2_3_320-px-long-name-large-amount.png` / `build-320-seller-payment-sent-long-name.png` | The board frame is the buyer's Funded room, the build is the seller's Payment sent room, so only the reflow rules compare. Known 1: "naledi_mokoena_trader_24 pays you" pushes "R 227,136.00" past the edge (TradeSummary L41). Known 2: the bar reads "Release 12480.00 USDT" ungrouped (status-cards-model L115). **New:** in the help line at the foot, the long name runs into "Get help" and the button is pushed past the 320 px edge (item 1 below) |
| `board_a11y-2_1_200-zoom-first-screen.png` / `build-640-seller-payment-sent-zoom200.png` | Matches the one-column phone layout with Help, ⋯ and the pinned bar at 640 CSS px; known 2 again ("Release 12480.00 USDT"); F2 |

### Differences that look like real issues

These are separate from the two known bugs.

1. **`src/components/features/trade/room/status/HelpLine.tsx` L32.** The help text is `<p className={`${T_BODY_SM} flex-1`}>` with no `min-w-0` and no `overflow-wrap`. At 320 px with a 24-character username, the text overlaps "Get help" and pushes the button past the right edge (`s9-shots/build-320-seller-payment-sent-long-name.png`). The room clips sideways overflow, so the passing "no sideways scroll" test does not catch it. The `it.failing` "nothing in the room runs past the right edge" test probably flags it too, but its comment names only TradeSummary L41, so fixing L41 alone will not turn that test green.
2. **Payment-details card in the chat column at 768 and 1440, in every status.** It is drawn by `TradeChat.tsx` L672 / `chat/ChatMessages.tsx` L227 `PaymentDetailsCard` from the start's `payment_details` system message. It shows as "FNB · bank" with "Account Holder", "Account Number" and "Branch Code" (title case, from `lib/trade/payment-details.ts` `paymentDetailLabel`) and a Copy all. No room board draws this card in the chat (A6, A7, A4, A5 columns start with "Trade started, funds in escrow"), and the pay plate in the card already carries the same details in sentence case ("Account holder"). The card also shows on Cancelled and Dispute resolved trades. In real chats it sits at the top of the history rather than beside the composer, so it is less prominent than in the fixture captures, but it is still a board difference and a casing mismatch. It needs a design call: drop it in the room's chat, or restyle it.

---

## 3. Backend-blocked parts

I checked state with read-only Trello GETs on 23 Oct 2026.

| Card | Trello state | What it blocks in 959 |
|---|---|---|
| 920 | Deployed (Monitoring) | **Live.** S2w uses its request key on the start and reads `GET /money-requests/{key}` for Check status. For AC4 this unblocks a lost start being found, not found or refused by its key, and Start again resending the same body under the same key. Still waiting: 957 mounting `StartOutcomeBody` on `/offer/[id]`. The in-room actions take no key (920 follow-up, no card), so their reference is display-only and "one action" rests on the state guards. A key held only in the tab's query cache is lost on reload or in a new tab |
| 963 | Backlog | Two moderator outcomes only. Until it ships a split can be recorded; the room and list show its amounts with no outcome name (AC1 1.4) |
| 910 | Development (In Progress) | Funding failure on non-custodial starts; an unfunded start on every route (AC1 1.5, AC2 2.2) |
| 911 | Backlog | Required balance and Max include the 1.00% fee (code assumes 0.7%) on 957's offer page (AC2 2.2) |
| 918 | Backlog | Code default escrow fee 70 bps; staging and production charge 100 bps. The room shows the server's quote, so this affects the source value and the worked example, not the display (AC2 2.1) |
| 877 | Backlog, **closed (archived)**, last activity 22 Oct | Seller take-back on a paid or disputed trade. S5 found 962 made the refund routes staff-only and `cancel_trade` refuses the seller. With the card closed, nothing is left for 959 |
| 909 | Backlog | The buyer gets 401 viewing their own proof; the room shows "Proof can't load" with Retry (AC5 5.4) |
| 921 | Backlog | The dispute reason is 1,000 characters in the room; the backend still accepts 2,000 |
| 972 | Backlog | `GET /trades/:id/ledger` returns the counterparty's rows and `balance_after`; the room uses only the viewer's rows |
| 973 | Backlog | The moderator's decision note is not exposed; the note block is omitted and decision cards are about 76 px shorter |
| 974 | Backlog | No events for dispute resolved, balance restored, document review or counterparty viewed; timeline and chat show only lines that have events |
| 975 | Backlog | No short reference (QT-7F3A21); the trade id stands in and wraps |
| 981 | Backlog | The buyer gets no held amount or fee: no buyer "Where the 121.20 USDT went", and the buyer's cancel outcome reads "Unavailable" for the fee |
| 986 | Backlog | No reseller earnings; reseller rows and summary read "Unavailable" |
| 987 | Backlog | No verified legal name; the username replaces "N Mokoena" in the seller's check-your-bank copy |
| 988 | Backlog | No escrow status after a cancel; "back in Available" is assumed |
| 990 | Backlog | No read receipts; no "· read" marks |
| 991 | Backlog | The rating a trader already gave is read from the counterparty's 50 newest ratings |
| 992 | Backlog | No document file names; captions read "JPG · 1.2 MB" |
| No card yet | n/a | `GET /trades` sends no `sellerEscrowFee`, so seller list rows read "Unavailable" (S7). The in-room actions carry no request key (S2w). The resell reseller share is not in the seller's response (S4) |

---

## 4. What remains for review-app QA

1. **Tier 2 live spec.** Run `E2E_TRADE_ROOM_LIVE=1` `trade-room-live.test.ts` with the QA accounts on the review app:
   - lost answer on I've paid and Release with `route.fetch()` then `route.abort()`;
   - ledger before and after shows one action;
   - double click on Cancel sends one POST;
   - IDOR from a third account: 403/404 on GETs and refused POSTs.
2. **Concurrency cases with no spec yet.** Two tabs on the same step, and a counterparty action (early release, dispute) landing while a step is open: the step ends failed or found, never with a second POST or a false success.
3. **Legacy specs.** These need signed-in QA accounts and funded QA sellers:
   - page object: `e2e/pages/trade.page.ts`;
   - runners: `e2e/run-escrow-e2e.ts`, `e2e/run-escrow-balance-e2e.ts`;
   - specs: `buy-sell-offer-flow`, `complete-trade-lifecycle`, `continue-trade-kyle`, `dispute-flow`, `escrow-confirm-test`, `escrow-micro-amounts/usdt-custodial-escrow`, `full-trade-flow-port5000`, `journey/phase1-create-offer`, `journey/phase2-trade-lifecycle`, `regression/ticket-atf78hg5-trade-receipt-pdf`, `reviews-ratings`, `single-browser-trade-flow`, `system/escrow-balance`, `trade-chat`, `trade-flow-with-escrow`, `two-browser-trade-flow`, `usdt-escrow-kyle`, `usdt-escrow-real`, `usdt-escrow`.
4. **AC1 journeys end to end.** Proof upload, I've paid, release, early release, expiry, cancel before payment, dispute with evidence, both document rejections. Then a moderator's two decisions with the ledger (121.20 back with no fee row; release equal to a normal completion), and an escalated dispute past its deadline.
5. **AC2 table.** API value against screen value, to six decimals, before and after each step.
6. **Captures with a live socket.** Capture all 8 statuses x 2 roles x 390/768/1440 plus the states, with the receipt flag on, real chat and proof messages, and the 965 register entries (current, proposed, implemented).
7. **Accessibility.** An axe run (WCAG 2.2 AA) on every trade screen and the 768 iPad Safari check of the history row link (S7). Recheck 320 px after the TradeSummary L41, status-cards-model L115 and HelpLine L32 fixes.
8. **Chat reconnect.** Drop the socket on the review app and confirm the paused bar, the REST fallback and the rejoin.
9. **Footer.** Confirm the (main) site footer no longer renders under `/trade/[id]`. S9 hid it, with unit tests `navigation/site-footer.test.ts` ("shouldShowSiteFooter: the trade room") and `navigation/main-site-footer.test.tsx` ("renders no footer on the trade room, signed in"); six other signed-in pages whose boards omit it are also covered.

## 5. Card vs board: drift that changes how an AC reads (boards win)

From plan section 7:

1. **AC1 verbs.** "I have paid" is "I've paid" (confirm "I've paid R 2,184.00"). "Release USDT" is "Release 120.00 USDT".
2. **AC1 screen sizes.** "390 x 844 and 1440 x 900" also includes 768 for every status, sheet, state and outcome (A4, A5, A19, A20, A22).
3. **AC1 moderator outcome names.** A moderator's "cancel" reads "Returned to the seller", and the other outcome "Released to the buyer", under the "Dispute resolved" chip.
4. **AC3 HARC.** "HARC wording appears only after a confirmed refund" becomes no HARC wording on any trade screen. A cancel restores 121.20 in the same step ("121.20 USDT back in Available"). HARC is for held withdrawals (B).
5. **AC3 escalation.** "Shows the escalated state" becomes the same In dispute state, with no deadline in customer copy.
6. **AC4 start.** The start step is a "layer for C" on 957's `/offer/[id]`. 959 builds the outcome component only.
7. **AC4 references.** Every pending, failed, unconfirmed and Check status step shows "Request reference X-XXXX-XXXX · sent hh:mm:ss" (not in the card text).
8. **AC5 legacy link.** "Keeping both URLs" becomes `/trade-history/[id]` answering 308 to `/trade/[id]` with the query kept. The URL works, the page is gone.
9. **AC5 history filters.** "Existing query and filter parameters": none existed. The board adds tab and filter state to the URL, replaces the crypto filter with "Paid in" and uses the seven status labels.
10. **AC5 WCAG.** The CTA's white on --action (2.77:1) is a documented exception kept by Christian.
11. **Dispute reason.** The limit is 1,000 characters on the board (backend 921 still allows 2,000).
