# Bug Tickets: Frontend ↔ Backend Field Mapping Mismatches

Audit date: 2026-03-05

All bugs below are cases where the frontend sends field names that do not
match the backend Rust struct (which uses `#[serde(rename_all = "camelCase")]`).
Serde either rejects the request (missing required field) or silently drops the
unknown field (data loss).

---

## BUG-001 — `createTrade` missing required `paymentMethod`

| | Frontend (`CreateTradeRequest`) | Backend (`CreateTradeRequest`) |
|---|---|---|
| **File** | `types/trade.types.ts` / `services/trades-api.ts` | `api/trades.rs` |
| **Fields sent** | `offerId`, `cryptoAmount`, `fiatAmount`, `exchangeRate?`, `buyerWalletAddress?` | — |
| **Fields expected** | — | `offerId`, `cryptoAmount`, `fiatAmount`, `paymentMethod` |

**Impact:** Backend deserialization fails (422) because `paymentMethod` is a
required `String` with no default. Every `POST /trades` call fails.

---

## BUG-002 — `createTicket` sends `description` instead of `message`

| | Frontend (`CreateTicketRequest`) | Backend (`CreateTicketRequest`) |
|---|---|---|
| **File** | `services/support-api.ts` | `api/support.rs` |
| **Fields sent** | `subject`, `description`, `category`, `tradeId?`, `escrowId?`, `priority?` | — |
| **Fields expected** | — | `subject`, `category`, `message`, `relatedTradeId?` |

**Impact:** Backend deserialization fails (422) — required `message` field is
missing. `description` is silently ignored. `tradeId` is ignored because
backend expects `relatedTradeId`.

---

## BUG-003 — `addMessage` (support ticket) sends `message` instead of `content`

| | Frontend | Backend (`AddTicketMessageRequest`) |
|---|---|---|
| **File** | `services/support-api.ts` | `api/support.rs` |
| **Payload sent** | `{ message, attachmentUrls? }` | — |
| **Payload expected** | — | `{ content }` |

**Impact:** Backend deserialization fails (422) — required `content` field is
missing.

---

## BUG-004 — `submitReport` sends wrong field names

| | Frontend (`CreateReportRequest`) | Backend (`SubmitReportRequest`) |
|---|---|---|
| **File** | `services/reports-api.ts` | `api/reports.rs` |
| **Fields sent** | `targetType`, `targetId`, `reason`, `description`, `offerId?`, `tradeId?` | — |
| **Fields expected** | — | `reportedUserId?`, `reportedTradeId?`, `reportedOfferId?`, `reason`, `description` |

**Impact:** Backend receives only `reason` and `description`. All three
`reported*Id` fields are None, triggering a validation error:
"must specify at least one of: reported_user_id, reported_trade_id,
reported_offer_id". Every report submission fails.

---

## BUG-005 — `deposit` sends `currency` instead of `cryptocurrency`

| | Frontend (`DepositRequest`) | Backend (`DepositRequest`) |
|---|---|---|
| **File** | `types/wallet.types.ts` / `services/wallet-api.ts` | `api/wallet.rs` |
| **Fields sent** | `amount`, `currency`, `txHash?`, `address?` | — |
| **Fields expected** | — | `cryptocurrency`, `amount`, `network`, `txHash` |

**Impact:** Backend deserialization fails (422) — required `cryptocurrency` and
`network` are missing.

---

## BUG-006 — `withdraw` sends `currency`/`address` instead of `cryptocurrency`/`toAddress`

| | Frontend (`WithdrawRequest`) | Backend (`WithdrawRequest`) |
|---|---|---|
| **File** | `types/wallet.types.ts` / `services/wallet-api.ts` | `api/wallet.rs` |
| **Fields sent** | `amount`, `currency`, `address`, `network?` | — |
| **Fields expected** | — | `cryptocurrency`, `amount`, `network`, `toAddress` |

**Impact:** Backend deserialization fails (422) — required `cryptocurrency` and
`toAddress` are missing.

---

## BUG-007 — `validateGasPayment` sends completely wrong payload

| | Frontend | Backend (`ValidateGasRequest`) |
|---|---|---|
| **File** | `services/gas-api.ts` | `api/gas.rs` |
| **Payload sent** | `{ usdtBalance }` | — |
| **Payload expected** | — | `{ network, txHash }` |

**Impact:** Backend deserialization fails (422). The frontend intends to check
gas affordability; the backend expects a tx-hash validation. This is a contract
design mismatch, not just a rename. Backend handler is currently a stub
(`"requires blockchain service"`).

---

## BUG-008 — `executeSponsoredWithdrawal` missing `network` and `token`

| | Frontend | Backend (`ExecuteSponsoredWithdrawalRequest`) |
|---|---|---|
| **File** | `services/gas-api.ts` | `api/gas.rs` |
| **Payload sent** | `{ toAddress, amount }` | — |
| **Payload expected** | — | `{ network, token, amount, toAddress }` |

**Impact:** Backend deserialization fails (422) — required `network` and
`token` are missing. Backend handler is currently a stub.

---

## BUG-009 — `getSponsoredWithdrawalEstimate` missing `network` and `token`

| | Frontend | Backend (`EstimateSponsoredWithdrawalRequest`) |
|---|---|---|
| **File** | `services/gas-api.ts` | `api/gas.rs` |
| **Payload sent** | `{ toAddress, amount }` | — |
| **Payload expected** | — | `{ network, token, amount, toAddress }` |

**Impact:** Same as BUG-008. Backend deserialization fails (422). Stub handler.

---

## BUG-010 — `createEscrow` (custodial) sends `cryptoType` instead of `cryptocurrency`

| | Frontend (`CreateEscrowRequest`) | Backend (`CreateCustodialEscrowRequest`) |
|---|---|---|
| **File** | `services/custodial-escrow.service.ts` | `api/escrow.rs` |
| **Fields sent** | `tradeId`, `cryptoType`, `network?`, `amount`, `sellerWalletAddress`, `buyerWalletAddress?` | — |
| **Fields expected** | — | `tradeId`, `cryptocurrency`, `amount` |

**Impact:** Backend deserialization fails (422) — required `cryptocurrency`
is missing (`cryptoType` is silently ignored).

---

## BUG-011 — `createOfferEscrow` sends `cryptoType` instead of `cryptocurrency`

| | Frontend (`CreateOfferEscrowRequest`) | Backend (`CreateOfferEscrowRequest`) |
|---|---|---|
| **File** | `services/offer-escrow.service.ts` | `api/escrow.rs` |
| **Fields sent** | `offerId`, `cryptoType?`, `amount?`, `network?` | — |
| **Fields expected** | — | `offerId`, `cryptocurrency`, `amount` |

**Impact:** Backend deserialization fails (422) — required `cryptocurrency`
is missing.

---

## BUG-012 — `sendMessage` (trade chat) extra `type` field silently dropped

| | Frontend (`SendMessageRequest`) | Backend (`SendMessageRequest`) |
|---|---|---|
| **File** | `types/trade.types.ts` / `services/trades-api.ts` | `api/trades.rs` |
| **Fields sent** | `content`, `type?`, `attachments?` | — |
| **Fields expected** | — | `content` |

**Impact:** Low severity. `type` and `attachments` are silently ignored by
serde. The message is saved without type metadata. Attachments must be
uploaded separately via the multipart endpoint.

---

## BUG-013 — `newsletter/subscribe` sends extra `source` field

| | Frontend | Backend (`SubscribeRequest`) |
|---|---|---|
| **File** | `components/landing/Newsletter.tsx` | `api/newsletter.rs` |
| **Payload sent** | `{ email, source: "landing_page" }` | — |
| **Payload expected** | — | `{ email }` |

**Impact:** Low severity. `source` is silently ignored by serde. Tracking
data about where subscriptions originate is lost.

---

## Summary

| Ticket | Service | Severity | Root Cause |
|--------|---------|----------|------------|
| BUG-001 | trades-api | **Critical** | Missing required field `paymentMethod` |
| BUG-002 | support-api | **Critical** | `description` → should be `message`; `tradeId` → `relatedTradeId` |
| BUG-003 | support-api | **Critical** | `message` → should be `content` |
| BUG-004 | reports-api | **Critical** | `targetType`/`targetId` not mapped to `reportedUserId`/`reportedTradeId`/`reportedOfferId` |
| BUG-005 | wallet-api | **Critical** | `currency` → `cryptocurrency`; missing `network` |
| BUG-006 | wallet-api | **Critical** | `currency` → `cryptocurrency`; `address` → `toAddress` |
| BUG-007 | gas-api | **High** | Completely wrong payload (design mismatch); backend is stub |
| BUG-008 | gas-api | **High** | Missing `network` and `token`; backend is stub |
| BUG-009 | gas-api | **High** | Missing `network` and `token`; backend is stub |
| BUG-010 | custodial-escrow | **Critical** | `cryptoType` → should be `cryptocurrency` |
| BUG-011 | offer-escrow | **Critical** | `cryptoType` → should be `cryptocurrency` |
| BUG-012 | trades-api | **Low** | Extra `type`/`attachments` silently dropped |
| BUG-013 | Newsletter | **Low** | Extra `source` silently dropped |
