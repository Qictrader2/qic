# 959-S2: trade mutation error shapes (backend to thrown JS Error)

Read-only research. Backend source: `git show origin/main:<path>` at f043a14. `trades.rs` below means `src/api/trades.rs` (the scratchpad copy `s2/trades.rs` was diffed against origin/main and is identical, so line numbers match). Frontend source: `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/959-S2-root/frontend` at 179f342f (`HttpClient` = `src/lib/api/http-client.ts`, `TA` = `src/services/trades-api.ts`).

Notation: the backend strings below are verbatim. Three of them (rows 4g, 5c and 5s) contain one real em dash character (U+2014) copied from the Rust source; those rows are the only places this file uses one. All apostrophes in the strings are plain ASCII `'`.

## 1. How a backend error becomes the thrown Error

### 1.1 Backend wire shape

`impl IntoResponse for AppError` (`src/error.rs:831-1047`). Every `AppError` without extra data uses the default arm (`error.rs:1033-1038`):

```json
{ "message": "<text>", "error": "<same text>", "status": 400, "code": "VALIDATION_ERROR" }
```

`message` and `error` always carry the same string (`user_message()`, `error.rs:719-828`). The `code` and HTTP status come from `error_code()` (`error.rs:659-717`) and `status_code()` (`error.rs:599-655`):

| AppError constructor used by the trade handlers | HTTP | code |
|---|---|---|
| `validation(..)` | 400 | `VALIDATION_ERROR` (`error.rs:607-609`, `667-669`) |
| `forbidden(..)` | 403 | `FORBIDDEN` (`error.rs:603`, `663`) |
| `KycRequired` | 403 | `KYC_REQUIRED`, message "KYC verification required to perform this action", plus `data.currentStatus` (`error.rs:604`, `738-740`, `880-885`) |
| `not_found(..)` | 404 | `NOT_FOUND` |
| `conflict(..)` | 409 | `CONFLICT` (`error.rs:612`, `672`) |
| `unprocessable(..)` | 422 | `UNPROCESSABLE_ENTITY` (`error.rs:610`, `670`) |
| `rate_limited(..)` | 429 | `RATE_LIMITED` (`error.rs:615`), `retry-after` header when set (`error.rs:1041-1045`) |
| `internal(..)` | 500 | `INTERNAL_ERROR` (message passes through as written) |
| `Database(sqlx::Error::RowNotFound)` (any `find_by_id(..)?`) | 404 | `DATABASE_ERROR`, message "resource not found" (`error.rs:617-620`, `677`, `765-768`) |
| `Database(other)` | 500 | `DATABASE_ERROR`, message "a database error occurred" (`error.rs:619`, `767`) |

Other non-AppError bodies a trade mutation can meet:
- CSRF middleware: 403, `{"error":"csrf token mismatch","code":"CSRF_TOKEN_MISMATCH"}` (no `message` key), `src/middleware/csrf.rs:152-160`. The frontend falls back to `data.error`, so the message is `csrf token mismatch`.
- Axum's own extractor rejections (bad UUID in the path, bad JSON on `mark_paid`, which uses plain `Json<MarkPaidRequest>`, `trades.rs:4452`) are plain text, not JSON. They go down the non-JSON path in section 3.
- `ValidatedJson` (used by `create_trade` and `open_dispute`) maps bad JSON to `validation("invalid request body")` (`src/extractors/validated.rs:30-37`) and field-rule failures to `validation("<field>: validation failed (<rule>)")` joined by `"; "` (`validated.rs:47-60`). Example: a dispute `reason` shorter than 5 characters gives 400 `reason: validation failed (length)` (rule declared at `trades.rs:4643`).

### 1.2 Frontend transformation (the part that decides what a matcher sees)

1. `HttpClient.request` returns `{ error: ApiError }` with `ApiError = { message, status, code?, details?, data? }` (`src/lib/api/types.ts:6-22`).
2. For a non-2xx JSON response (`http-client.ts:449-497`): `message = data.message || data.error || "Request failed (<status>)"` (`:468-469`), `code = data.code` (`:470`), `status = response.status` (`:480`). If `data.message` is an object, `message = data.message.message || JSON.stringify(data.message)` (`:463-466`). If the result is empty or the string `"undefined"`, `message = "Server error (<status> <statusText or Unknown>)"` (`:474-476`). No prefix and no rewrite of the backend text.
3. 401 on a non-auth endpoint is handled earlier (`:292-428`), not relevant to the cases below.
4. The `tradesApi` method then throws:
   - `createTrade` (`TA:302-340`), `cancelTrade` (`TA:513-523`), `completeTrade` (`TA:552-596`), `markPaid` (`TA:1007-1017`), `getTrade` (`TA:465-474`), `getTrades` (`TA:345-355`), `updateTradeStatus` (`TA:488-498`): `throw new Error(response.error.message)`. The thrown object is a plain `Error`. `status` and `code` are dropped (`.status` is `undefined`, `.code` is `undefined`).
   - `createTrade` has one rewrite (`TA:322-327`): if the message includes `escrow wallet`, the message becomes `Failed to setup escrow for this trade. The offer may have invalid network settings. Please contact support or try a different offer.` None of the backend messages in this report contain `escrow wallet`.
   - `openDispute` (`TA:710-728`): `throw new DisputeRequestError(response.error.message, response.error.status, response.error.code)`. `DisputeRequestError` (`TA:237-247`) extends `Error` and keeps `.status` and `.code`. This is the only trade mutation that keeps the status.
5. Success path that can still throw: `extractTrade` throws `Error("Invalid trade data received from server")` (`TA:281`) when the 2xx body has no usable trade id. `cancelTrade` swallows that and returns `{ id, status: 'cancelled' }` (`TA:529-545`).
6. React Query hooks (`src/hooks/api/use-trades.ts:199-366`) pass the thrown error through unchanged to `onError`.

## 2. Case table

Columns: case, HTTP status, exact backend message, backend file:line, message the frontend Error carries, status on thrown error, frontend file:line.

### 2.1 Mark paid (`POST /trades/{id}/mark-paid`, handler `trades.rs:4448`)

Check order inside the handler: `find_by_id` (4455) then buyer-only check (4457) then KYC-document check (4463) then crypto-quote branch (4470) then status check (4474). So a trade that needs documents and has none approved answers with the document message before any status message.

| Case | HTTP | Exact backend message | Backend file:line | Frontend Error message | Status on thrown error? | Frontend file:line |
|---|---|---|---|---|---|---|
| 1a. After the trade expired and the sweep cancelled it (status `cancelled`) | 400 | `Cannot mark paid from status cancelled. Expected: escrow_funded.` | `trades.rs:4474-4479` (Display of status: `types/enums.rs:692-704`) | same text | No (plain Error) | `TA:1013-1016` |
| 1b. Already paid (double tap) | 400 | `Cannot mark paid from status paid. Expected: escrow_funded.` | `trades.rs:4474-4479` | same text | No | `TA:1016` |
| 1c. Disputed | 400 | `Cannot mark paid from status disputed. Expected: escrow_funded.` | same | same text | No | same |
| 1d. Completed (seller released early) | 400 | `Cannot mark paid from status completed. Expected: escrow_funded.` | same | same text | No | same |
| 1e. Released, resolved, created | 400 | `Cannot mark paid from status released. Expected: escrow_funded.` (and `resolved`, `created`) | same | same text | No | same |
| 1f. Caller is not the buyer | 403 | `only the buyer can mark a trade as paid` | `trades.rs:4457-4461` | same text | No | same |
| 1g. Required documents not approved (checked before status) | 400 | `required trade documents must be approved before payment can be marked` | `services/trade_kyc.rs:85-94`, call at `trades.rs:4463` | same text | No | same |
| 1h. Trade id unknown | 404 | `resource not found` | `repo/trade.rs:55-60`, `error.rs:617-620`, `765-768` | same text | No | same |
| 1i. Crypto-quote swap trade, not awaiting payment (rare path) | 400 | `Cannot settle this swap from status <status>. The trade is no longer awaiting payment.` | `trades.rs:4362-4363` | same text | No | same |

Expiry note: the expiry job only cancels trades in `created` or `escrow_funded` (`repo/trade.rs:529-538`, called from `jobs.rs:605-671`). Between `expires_at` and the next sweep the status is still `escrow_funded`, and `mark_paid` has no time check, so it succeeds in that window. After the sweep the status is `cancelled` (row 1a).

### 2.2 Complete / release (`POST /trades/{id}/complete`, handler `trades.rs:3074`)

Allowed source statuses: `escrow_funded`, `paid`, `released` (check at `trades.rs:3099-3107`). The caller check comes first (`3085-3093`).

| Case | HTTP | Exact backend message | Backend file:line | Frontend Error message | Status on thrown error? | Frontend file:line |
|---|---|---|---|---|---|---|
| 2a. After a dispute was opened | 400 | `Cannot complete trade in status disputed. Expected: escrow_funded, paid, or released.` | `trades.rs:3099-3107` | same text | No | `TA:573-580`, rethrown `TA:590-595` |
| 2b. After cancellation | 400 | `Cannot complete trade in status cancelled. Expected: escrow_funded, paid, or released.` | same | same text | No | same |
| 2c. Already completed (double click or second tab) | 400 | `Cannot complete trade in status completed. Expected: escrow_funded, paid, or released.` | same | same text | No | same |
| 2d. Resolved or created | 400 | `Cannot complete trade in status resolved. Expected: escrow_funded, paid, or released.` (and `created`) | same | same text | No | same |
| 2e. Caller is the buyer or an outsider | 403 | `only the seller, vendor, or admin can release/complete a trade` | `trades.rs:3085-3093` | same text | No | same |
| 2f. Escrow release itself fails (settlement math, DB) | 400 or 500, message from `escrow_release.rs` (for example `escrow amount <n> is zero or negative` at `escrow_release.rs:973-977`, or `AppError::internal(..)` texts) | various | `trades.rs:3175-3183` returns the error unchanged | same text | No | same |

A concurrent double submit (two requests both passing the status check) was not traced to a specific message. `repo::trade::complete` has a guard (`repo/trade.rs:310-325`) and the handler turns its `RowNotFound` into a success re-read (`trades.rs:3299-3315`), so the likely outcome for the second request is a 200 with the completed trade, but I did not run it.

### 2.3 Cancel (`POST /trades/{id}/cancel`, handler `trades.rs:3048`)

Decision function `cancel_refusal` (`trades.rs:2765-2782`) and its error mapping (`trades.rs:2784-2799`). Order: outsider, then disputed, then "status cannot reach cancelled", then counterparty, then buyer-after-payment. `user_can_cancel()` is true only for `created` and `escrow_funded` (`types/enums.rs:657-659`). Admin bypasses the last two checks. Only the buyer (or an admin) can cancel; the seller side gets 403.

| Case | HTTP | Exact backend message | Backend file:line | Frontend Error message | Status on thrown error? | Frontend file:line |
|---|---|---|---|---|---|---|
| 3a. Buyer cancels after mark-paid (status `paid`) | 403 | `Cannot cancel after payment has been sent. Please contact support if needed.` | `trades.rs:2779`, `2795-2797` | same text | No | `TA:519-523` |
| 3b. Buyer or seller cancels after early release (status `completed`) | 400 | `Cannot cancel: trade status 'completed' does not allow cancellation.` | `trades.rs:2773-2775`, `2791-2794` | same text | No | `TA:522` |
| 3c. Same for status `released` / `resolved` | 400 | `Cannot cancel: trade status 'released' does not allow cancellation.` (and `'resolved'`) | same | same text | No | same |
| 3d. Already cancelled (double click) | 400 | `Cannot cancel: trade status 'cancelled' does not allow cancellation.` | same | same text | No | same |
| 3e. Cancel while disputed (buyer, seller or admin) | 409 | `This trade is in dispute. A moderator will decide it.` | const `DISPUTED_TRADE_CANCEL_REFUSAL` at `trades.rs:2722`, used at `2770-2772`, `2790` | same text | No | same |
| 3f. Seller (or vendor) cancels a trade that is `created`, `escrow_funded` or `paid` | 403 | `Only the buyer can cancel a trade.` | `trades.rs:2778`, `2787-2789` | same text | No | same |
| 3g. Outsider (not a party, not admin) | 403 | `Only the buyer can cancel a trade.` | `trades.rs:2767`, `2787-2789` | same text | No | same |
| 3h. Status changed between the check and the cancel (race) | 409 | `This trade changed while it was being cancelled. Refresh and try again.` (or the dispute message above if it became disputed). Sent for every status the trade moved to, Cancelled included, so it does not say whether the cancel landed: the classifier reads it as **unconfirmed** and Check status decides (fix round 1, M1). | `trades.rs:2829-2843` | same text | No | same |

Status order detail: for a seller on a `completed` trade the answer is 3b (400), not 3f (403), because the status check runs before the actor check.

### 2.4 Open dispute (`POST /trades/{id}/dispute`, handler `trades.rs:4651`)

Participant check (`4663-4667`), then already-disputed (`4673-4677`), then source status must be `paid` (`4678-4683`; the only dispute edge is `Paid -> Disputed`, `types/enums.rs:648`).

| Case | HTTP | Exact backend message | Backend file:line | Frontend Error message | Status on thrown error? | Frontend file:line |
|---|---|---|---|---|---|---|
| 4a. Dispute already open (double click) | 409 | `a dispute is already open on this trade` | `trades.rs:4673-4677`; race copies at `services/dispute.rs:485-488` and `524-527` | same text | Yes: `DisputeRequestError.status = 409`, `.code = "CONFLICT"` | `TA:723-727`; matcher `isAlreadyDisputedError` `TA:258-265` |
| 4b. After release / completed | 400 (not 422) | `Cannot open dispute from status completed. Expected: paid.` (and `released`) | `trades.rs:4678-4683` | same text | Yes: `.status = 400`, `.code = "VALIDATION_ERROR"` | `TA:723-727` |
| 4c. While funded (`escrow_funded`, buyer has not marked paid) | 400 | `Cannot open dispute from status escrow_funded. Expected: paid.` | `trades.rs:4678-4683` | same text | Yes: `.status = 400` | `TA:723-727` |
| 4d. Cancelled / resolved / created | 400 | `Cannot open dispute from status cancelled. Expected: paid.` (and `resolved`, `created`) | same | same text | Yes: `.status = 400` | same |
| 4e. Race: the escrow was released or refunded after the status check passed (the only 422) | 422 | `this trade's escrow is <escrow status> and can no longer be disputed` | `services/dispute.rs:489-491` | same text | Yes: `.status = 422`, `.code = "UNPROCESSABLE_ENTITY"` | `TA:723-727` |
| 4f. Race: the trade left `paid` after the check passed | 422 | `this trade is <trade status> and can no longer be disputed` | `services/dispute.rs:528-530` | same text | Yes: `.status = 422` | same |
| 4g. Escrow row is not `held` before the freeze | 400 | `escrow cannot be disputed from status '<escrow status>' — must be 'held'` | `services/dispute.rs:21-29`, call at `trades.rs:4701` | same text | Yes: `.status = 400` | same |
| 4h. Trade has no escrow | 400 | `this trade has no escrow, so a dispute cannot be opened on it` | `trades.rs:4694-4698` | same text | Yes: `.status = 400` | same |
| 4i. Caller is not a participant | 403 | `only trade participants can open a dispute` | `trades.rs:4663-4667` | same text | Yes: `.status = 403` | same |
| 4j. Reason shorter than 5 or longer than 2000 characters | 400 | `reason: validation failed (length)` | rule `trades.rs:4643`, formatter `extractors/validated.rs:47-60` | same text | Yes: `.status = 400` | same |

What `isAlreadyDisputedError` matches (`TA:258-265`): true only for a `DisputeRequestError` with `status === 409` (any message), or with `status === 400` and a message matching `/from status disputed/i`. The 400 branch covers an older backend. On the current backend, `Cannot open dispute from status disputed` never occurs, because the `Disputed` status is caught first by the 409 branch (`trades.rs:4673`). A plain `Error` (any other tradesApi method) never matches.

### 2.5 Create trade (`POST /trades`, handler `trades.rs:1353`)

| Case | HTTP | Exact backend message | Backend file:line | Frontend Error message | Status on thrown error? | Frontend file:line |
|---|---|---|---|---|---|---|
| 5a. Taker supplies the crypto (maker is buying), wallet balance below amount plus fee | 400 | `You don't have enough <ASSET> to accept this offer. Required: <R>, available: <A>.` Example: `You don't have enough USDT to accept this offer. Required: 10.250000, available: 3.000000.` | `insufficient_lock_balance_message` `trades.rs:974-998` (text at `987-992`), called at `1900-1908`; formatter `format_minor_units` `trades.rs:546-560` | same text | No | `TA:322-329` |
| 5b. Taker supplies the crypto, no wallet row at all | 400 | `You don't have any <ASSET> in your wallet to accept this offer.` Example: `You don't have any USDT in your wallet to accept this offer.` | `trades.rs:982-983` | same text | No | same |
| 5c. Taker is the buyer and the seller's balance is short (also when the seller has no wallet row) | 400 | `This offer cannot be filled right now — the seller's <ASSET> balance is insufficient.` Example: `This offer cannot be filled right now — the seller's USDT balance is insufficient.` | `trades.rs:984-985` and `993-994` | same text | No | same |
| 5d. Lock refused inside `lock_balance_for_trade_on` (balance dropped between the check and the `UPDATE ... WHERE balance >= $1`) | 400 | `Insufficient balance to lock for this trade. Your available funds may have changed.` | `repo/wallet.rs:603-620` (returns `sqlx::Error::RowNotFound`), mapped at `trades.rs:1910-1926` (text at `1921-1923`) | same text | No | same |
| 5e. Lock fails with any other DB error | 500 | `a database error occurred` | `trades.rs:1925`, `error.rs:767` | same text | No | same |
| 5f. Offer not active (paused, closed, deleted) | 400 | `Offer is not available (status: paused). Only active offers can be traded.` (status word is `active`, `paused`, `closed` or `deleted`, `types/enums.rs:593-601`) | `trades.rs:1430-1435` | same text | No | same |
| 5g. Offer id unknown or hard-deleted | 404 | `resource not found` | `trades.rs:1370`, `error.rs:765-768` | same text | No | same |
| 5h. Fiat amount below the offer minimum (limits changed) | 400 | `Amount <sym><amt> is below offer minimum <sym><min>.` Example: `Amount $5.00 is below offer minimum $10.00.` (symbol from the fiat catalog, two decimals) | `trades.rs:1526-1535`, text built in `api/offers.rs:922-932` | same text | No | same |
| 5i. Fiat amount above the offer maximum | 400 | `Amount <sym><amt> is above offer maximum <sym><max>.` | `trades.rs:1536-1544`, `api/offers.rs:934-944` | same text | No | same |
| 5j. Own offer | 400 | `You cannot start a trade on your own offer.` | `trades.rs:1438-1442` | same text | No | same |
| 5k. Own resell offer | 400 | `You cannot trade on your own resell offer.` and `You cannot trade on a resell of your own offer.` | `trades.rs:1423-1427`, `1457-1461` | same text | No | same |
| 5l. Test and real account mix | 400 | `Test accounts can only trade with other test accounts.` | `trades.rs:1471-1475` | same text | No | same |
| 5m. Blocked either way | 403 | `Cannot initiate a trade with this user. One of you has blocked the other.` | `trades.rs:1479-1483` | same text | No | same |
| 5n. Account suspended | 403 | `Your account is currently suspended. You cannot create new trades.` | `trades.rs:1489-1493` | same text | No | same |
| 5o. Offer needs a verified account | 400 | `This offer requires a verified account.` | `trades.rs:1495-1499` | same text | No | same |
| 5p. Offer needs completed trades | 400 | `This offer requires at least <n> completed trades (you have <m>).` | `trades.rs:1500-1509` | same text | No | same |
| 5q. FX rate unavailable | 400 | `The exchange rate is unavailable right now. Please try again shortly.` | `trades.rs:1580-1588`, text `services/quote.rs:104-105` | same text | No | same |
| 5r. Index price unavailable (market-priced offer) | 400 | `Live price data is unavailable right now. Please try again shortly.` | `trades.rs:1615-1621`, text `services/quote.rs:102-103` | same text | No | same |
| 5s. Resell with no parent price feed | 400 | `Cannot create resell trade right now — price feed is unavailable. Please try again in a moment.` | `trades.rs:1791-1793` | same text | No | same |
| 5t. Quote math failure | 400 | `Quote calculation failed: <detail>` | `trades.rs:1634-1635` | same text | No | same |
| 5u. Mandatory-KYC gate on (config flag) and user not approved | 403 | `KYC verification required to perform this action` (code `KYC_REQUIRED`, `data.currentStatus`) | `extractors/kyc.rs:33-90`, `error.rs:738-740` | same text | No (plain Error, `data` also dropped) | `TA:329` |
| 5v. Solana offer and Solana not enabled for the user | 403 | `Solana support is not available for your account yet` | `trades.rs:1378-1380`, `error.rs:741-743` | same text | No | same |

Figure format in 5a (`format_minor_units`, `trades.rs:546-560`; tests at `trades.rs:6554-6580`):
- Both figures are integer minor units divided by `10^decimals` with exact integer math, always printed with the full decimal width and no trimming: USDT is 6 places (`5.443243`, `10.250000`, `0.500000`), BTC is 8 (`0.00100000`), SOL 9, ETH 18.
- No thousands separator, no unit after the figure (the asset code appears only once, in `enough <ASSET>`), and the line ends with a period after the available figure.
- `Required` is the lock amount, which is crypto amount plus escrow fee (`trades.rs:1854-1863`, plus the BTC gas reserve for BTC wallet-lock offers). `available` is the wallet `balance` column read at `trades.rs:1881-1891` (the lock debits `balance`, so locked funds are not in it).
- Matcher regex that fits both figures: `Required: (\d+\.\d+), available: (\d+\.\d+)\.`

Paths 5a to 5d all answer HTTP 400 `VALIDATION_ERROR` and reach the frontend as a plain `Error` with no status. The only way to tell "did not start for lack of funds" from the other 400s is the message text. Message-prefix families: `You don't have enough ` (5a), `You don't have any ` (5b), `This offer cannot be filled right now` (5c), `Insufficient balance to lock` (5d).

### 2.6 Other read-side errors the trade room will meet

| Case | HTTP | Exact backend message | Backend file:line | Frontend Error message | Status on thrown error? | Frontend file:line |
|---|---|---|---|---|---|---|
| get trade, not a participant | 403 | `you are not a participant in this trade` | `trades.rs:2424-2428` | same text | No | `TA:471-474` |
| get trade, unknown id | 404 | `resource not found` | `trades.rs:2422`, `error.rs:765-768` | same text | No | `TA:471-474` |
| `PATCH /status` to cancelled | 400 | `Use POST /trades/{id}/cancel to cancel a trade - PATCH /status cannot move funds.` | `trades.rs:2724-2725`, `2524-2526` | same text | No | `TA:494-498` |

## 3. http-client paths (all in `src/lib/api/http-client.ts` unless noted)

### 3.1 429

```ts
// http-client.ts:239-248
      // Handle rate limiting
      if (response.status === 429) {
        const error: ApiError = {
          message: "Too many requests. Please wait a moment.",
          status: 429,
          code: "RATE_LIMITED",
          data: await readRateLimitBody(response),
        }
        return { error }
      }
```

The 429 is answered after a server response. The block sits after `await fetch(url, ...)` (`:228-233`) and the `clearTimeout` (`:235-237`). There is no client-side limiter: a grep of `src/lib` for `429`, `rate limit` and `too many` finds this block, `readRateLimitBody` (`:139-153`, which parses the JSON body into `ApiError.data` and returns `undefined` when the body is not JSON), a CoinGecko constant (`constants.ts:80`), comments in `auth/`, and the untracked `trade/commit.ts`. Nothing delays or refuses a request before `fetch`. The message is fixed text and ignores the backend's own `message`:
- message: `Too many requests. Please wait a moment.`
- status: 429, code `RATE_LIMITED`.
- Through `tradesApi`: a plain `Error("Too many requests. Please wait a moment.")` with no status (createTrade, cancel, complete, markPaid). Through `openDispute`: `DisputeRequestError` with `.status = 429` and `.code = "RATE_LIMITED"`.

Backend side: I found no rate limiter on the trade routes (`rate_limited` calls sit in `api/auth.rs:46`, `api/marketplace.rs:176-264`, `api/kyc.rs:1612`, `api/gas.rs`, `api/waitlist.rs`; none in `api/trades.rs`). A 429 on a trade mutation therefore comes from something in front of the app (router or CDN), not from the Rust handlers. I did not verify the infrastructure.

### 3.2 Abort and timeout

```ts
// http-client.ts:216-219 (internal timeout, 30 s, only when no caller signal and skipInternalTimeout is false)
      if (!skipInternalTimeout && !signal) {
        controller = new AbortController()
        timeoutId = setTimeout(() => controller?.abort(), 30000)
      }
// http-client.ts:501-510
      if (error instanceof DOMException && error.name === "AbortError") {
        return { error: { message: "Request was cancelled", status: 0, code: "REQUEST_ABORTED" } }
      }
```

- message `Request was cancelled`, status 0, code `REQUEST_ABORTED`. The 30 s timeout and a caller abort look identical.
- The timeout clears as soon as response headers arrive (`:235-237`), so it only covers the wait for the first byte.
- `completeTrade` passes `skipInternalTimeout: true` (`TA:561-566`), so it has no 30 s timeout; the other four mutations do (they pass no signal).
- Through `tradesApi`: plain `Error("Request was cancelled")` (no status), except `openDispute`, which gives `DisputeRequestError` with `.status = 0`, `.code = "REQUEST_ABORTED"`.

### 3.3 Network failure

```ts
// http-client.ts:512-525
      const isNetworkError = error instanceof TypeError && error.message.includes("fetch")
      const networkMessage = isNetworkError
        ? `Cannot connect to server at ${this.baseUrl}. Please check if the backend is running and that NEXT_PUBLIC_API_URL is set correctly for this environment.`
        : error instanceof Error ? error.message : "Network error"
      return { error: { message: networkMessage, status: 0, code: "NETWORK_ERROR" } }
```

- Chrome's "Failed to fetch" and Firefox's "NetworkError when attempting to fetch resource." contain `fetch`, so they give `Cannot connect to server at <baseUrl>. Please check if the backend is running and that NEXT_PUBLIC_API_URL is set correctly for this environment.`
- Safari's `TypeError: Load failed` does not contain `fetch`, so the message is the raw `Load failed`. Any other `Error` passes its own message through. A non-Error throw gives `Network error`.
- status 0, code `NETWORK_ERROR` in every case.
- `<baseUrl>` is `API_CONFIG.baseUrl` (`src/lib/api/index.ts:42`), so a matcher should use a prefix: `Cannot connect to server at `.

### 3.4 5xx with a JSON body

The body is the standard error JSON. The frontend takes `data.message` verbatim, status is the real status, code is `data.code`. Examples from the backend: 500 `a database error occurred` (code `DATABASE_ERROR`), 500 with an `internal(..)` text (code `INTERNAL_ERROR`), 503 `service unavailable: ...` variants carry the written message (code `SERVICE_UNAVAILABLE`).

`Server error (<status> <statusText>)` appears only when the parsed message is empty or the string `undefined` (`:473-476`). `statusText` is `Unknown` when empty (HTTP/2 responses have an empty `statusText`), so a typical value is `Server error (500 Unknown)`.

`Request failed (<status>)` appears (a) for a JSON body that has neither `message` nor `error` (`:468-469`) and (b) for a non-JSON body with empty text and empty `statusText` (see 3.5).

A JSON content type with an unparsable body gives `Invalid response format` (`:434-438`) with the real status.

### 3.5 Non-JSON bodies (gateway pages, axum plain-text rejections)

`buildNonJsonApiErrorPayload` (`src/lib/api/error-messages.ts:28-58`), called from `http-client.ts:439-447`:

| Condition | message | code |
|---|---|---|
| status 502, 503 or 504 (any body) | `The service is temporarily unavailable. Please try again in a minute.` | `SERVICE_UNAVAILABLE` |
| status >= 500 and HTML (content type `text/html` or body matches `<!doctype html`, `<html`, `<head`, `<body`, `<meta`) | `The service is temporarily unavailable. Please try again in a minute.` | `SERVICE_UNAVAILABLE` |
| any other HTML body (for example 403, 404 from a proxy) | `The service returned an unexpected response. Please try again.` | `NON_JSON_RESPONSE` |
| plain text | `<trimmed body text or statusText or "Request failed (<status>)">` cut to 200 characters | `NON_JSON_RESPONSE` |

The resulting `ApiError.status` is the real HTTP status. A 2xx response that is not JSON also goes through this builder and is returned as `data` (a success with an error-shaped payload), which `extractTrade` then rejects with `Invalid trade data received from server` (`TA:281`); `cancelTrade` would report that as a successful cancel (`TA:534-545`).

## 4. Types, keys and signatures the S2 work needs

### 4.1 `tradesApi` signatures (`TA`)

- `getTrade(tradeId: string): Promise<Trade>` (`TA:465`). GET `/trades/{id}`. Normalizes via `extractTrade` (`TA:267-282`) and `normalizeTrade` (`TA:56-77`).
- `getTrades(params?: TradeQueryParams): Promise<TradeListResponse>` (`TA:345`). GET `/trades` plus query string. On a response without `trades` it returns `{ trades: [], total: 0, hasMore: false }` (`TA:359-362`) instead of throwing.
- Also `getActiveTrades(params?)` (`TA:395`, GET `/trades/active`), `getCompletedTrades(params?)` (`TA:430`, GET `/trades/completed`).
- `createTrade(data: CreateTradeRequest): Promise<Trade>` (`TA:302`), `cancelTrade(tradeId, reason?): Promise<Trade>` (`TA:513`), `completeTrade(tradeId): Promise<Trade>` (`TA:552`), `openDispute(tradeId, data: { reason: string; description?: string }): Promise<Trade>` (`TA:710`), `markPaid(tradeId, data?: { proofUrl?: string; notes?: string }): Promise<Trade>` (`TA:1007`).
- `TradeQueryParams` (`src/types/trade.types.ts:202-207`): `{ status?: TradeStatus; role?: 'buyer' | 'seller'; limit?: number; offset?: number }`. `TradeListResponse` (`:210-214`): `{ trades: Trade[]; total: number; hasMore: boolean }`.
- Mismatch to know about: the frontend sends `status`, `role`, `limit`, `offset` (`buildQueryString`, `TA:285-296`). The backend list handler reads only `status`, `page` and `per_page` (`TradeListParams`, `trades.rs:132-138`; defaults page 1, per_page 20; per_page clamps to 1..=100, `types/pagination.rs`). `role`, `limit` and `offset` are ignored, so the list always returns the first 20 trades. `/trades/active` ignores all params and returns every active trade with `hasMore: false` (`trades.rs:2354-2377`). `status` must be a backend snake_case status (`trades.rs:133`); a legacy value such as `pending` would be rejected by the axum `Query` extractor (plain-text 400).

### 4.2 Trade type fields (`src/types/trade.types.ts:13-129`)

- `id: string` (`:14`), `tradeId: string` (`:15`, not sent by the backend; `normalizeTradeId` falls back, `TA:37-51`), `offerId: string` (`:16`), `buyerId: string` (`:17`), `sellerId: string` (`:18`).
- `status: TradeStatus` (`:61`), where `TradeStatus = TradeStatusCompat` (`:9`) = `TradeState | 'pending' | 'active' | 'completed'` (`src/lib/state-machine/trade-states.ts:17-23`).
- `createdAt: string` (`:68`), `updatedAt: string` (`:69`). Optional: `timeLimitMinutes`, `expiresAt`, `cancelEligible`, `withinUnviewedCancelWindow`, `counterpartyViewed`, `counterpartyViewedAt` (`:124-129`).
- Backend sends `createdAt`, `updatedAt`, `expiresAt`, `cancelEligible`, `offerId`, `buyerId`, `sellerId`, `status`, `cryptocurrency` (camelCase, `TradeResponse` at `trades.rs:163-258`). The backend does not send `startDate`, `completionDate`, `escrowStatus` or `tradeId`, although the type declares them (`startDate` and `escrowStatus` as required, `completionDate` as optional).

### 4.3 Query keys (`src/hooks/api/use-trades.ts:18-28`)

```ts
export const tradeKeys = {
  all: ['trades'] as const,
  lists: () => [...tradeKeys.all, 'list'] as const,                          // ['trades','list']
  list: (params?: TradeQueryParams) => [...tradeKeys.lists(), params] as const, // ['trades','list',params]
  active: () => [...tradeKeys.all, 'active'] as const,                       // ['trades','active']
  completed: () => [...tradeKeys.all, 'completed'] as const,                 // ['trades','completed']
  detail: (id: string) => [...tradeKeys.all, 'detail', id] as const,         // ['trades','detail',id]
  messages: (id: string) => [...tradeKeys.all, 'messages', id] as const,
  events: (id: string) => [...tradeKeys.all, 'events', id] as const,
  kyc: (id: string) => [...tradeKeys.all, 'kyc', id] as const,
}
```

Hooks: `useTrades(params?)` uses `tradeKeys.list(params)`, staleTime 30 s (`:31-43`). `useActiveTrades(params?)` uses `tradeKeys.active()` (the params are not part of the key), staleTime 30 s (`:69-81`). `useCompletedTrades(params?)` uses `tradeKeys.completed()`, staleTime 5 min (`:106-114`). `useTrade(tradeId, enabled = true)` uses `tradeKeys.detail(tradeId)`, staleTime 10 s (`:118-131`). Mutation hooks invalidate `tradeKeys.lists()` and `tradeKeys.active()`; complete also invalidates `tradeKeys.completed()` (`:211-297`); dispute success calls `applyDisputeToTradeCache` (`:318-329`).

### 4.4 `normalizeTradeStatus` and `TradeState`

- `normalizeTradeStatus(status: string, escrowStatus?: string): TradeState` at `src/lib/state-machine/migration-map.ts:45-50`, re-exported from `src/lib/state-machine/index.ts:46-49`. `trades-api.ts` imports it as `normalizeTradeState` (`TA:21`) and applies it in `normalizeTrade` (`TA:62-64`), overwriting `status` on every trade it returns.
- `TradeState` (`src/lib/state-machine/trade-states.ts:7-17`) = `'created' | 'escrow_funded' | 'paid' | 'released' | 'disputed' | 'resolved' | 'cancelled'`. There is no `'completed'` member.
- The map (`migration-map.ts:31-43`): `created`, `escrow_funded`, `paid`, `released`, `disputed`, `resolved`, `cancelled` map to themselves; legacy `pending` to `created` (or `escrow_funded` when `escrowStatus === 'held'`, `:46-48`), `active` to `paid`, `completed` to `released`. Any unknown string maps to `created` (`:49`).
- Consequence (fact, from the code above): the backend status `completed` (terminal, `types/enums.rs:631`) reaches components as `released`, the same value as the backend's intermediate `released` status. After `normalizeTrade`, the two cannot be told apart by `status`. The terminal-set constant `TRADE_TERMINAL_STATES` (`trade-states.ts:25-29`) holds `released`, `resolved`, `cancelled`.

## 5. Differences worth flagging

1. `src/lib/state-machine/trade-states.ts:40-52` (`TRADE_TRANSITIONS`) lists edges the backend refuses: `escrow_funded -> disputed` (backend: 400, rows 4c), seller `escrow_funded -> cancelled` and seller `paid -> cancelled` (backend: 403, row 3f), `created -> paid` (backend: 400, `can_transition_to` at `types/enums.rs:638-651`). Backend edges that the table lacks: `escrow_funded -> released`, `released -> completed`. Intent doc: `qictrader-backend-rs/docs/intended-entity-state-machines.md` was not re-read for this slice.
2. The three `Error`-throwing methods that matter for the "did not start" screens (`createTrade`, `markPaid`, `cancelTrade`, `completeTrade`) lose `status` and `code`. The string is the only signal. Only `openDispute` keeps both.
3. 429 text is client-fixed, so the backend's own rate-limit message can never reach the UI.
4. Untracked files already present in this checkout, not touched or read in depth: `src/lib/trade/commit.ts`, `src/lib/trade/start-errors.ts`, `src/hooks/use-trade-commit.ts`, `src/lib/request-reference.ts`.

## 6. What was not checked

- No request was sent to staging or any server; everything above is from source. The `message` strings were not compared against a live response.
- Concurrent double submits for complete and mark-paid (see 2.2 note) were read but not run.
- The fiat symbol returned by `fiat_catalog_entry` (`api/platform_config.rs`) for 5h and 5i was not looked up per currency.
- `settle_crypto_swap_trade` and `create_crypto_swap_trade` (crypto-quote offers) were read only for their status-check and lock messages; their other refusals are not listed.

## Classification changes in fix round 2 (959-S2)

- 1a `Cannot mark paid from status cancelled.`: the answer names only the status, so it classifies as the neutral `cancelledFirst`. The hook upgrades it to `timeRanOut` only when the trade already in the query cache has an `expiresAt` that has passed (`refineWithTrade`, no network). A Check status read does the same from the GET (`matchCheck(..., now)`).
- 5c `This offer cannot be filled right now …`: a start refusal `offerUnavailable` / `cannotFill`, shown with C7 frame 11's words ("This offer can't take a trade at its minimum right now. No trade was started and nothing moved."), never as the buyer's balance.
- 5h, 5i limits: the backend writes `R1500.00` (no space, no grouping, `api/offers.rs` `amount_below_minimum_message` / `amount_above_maximum_message`); the classifier formats it with `formatCurrencyByCode` into the board's `R 1,500.00` when the symbol is known.
- `TA:322-327` rewrite `Failed to setup escrow for this trade. …`: trades-api replaces any message naming an "escrow wallet" and drops the status, so it may hide a 500. Now `unconfirmed` (Check status decides), no longer a definite refusal.
