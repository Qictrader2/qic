# Qictrader — Full Rewrite Plan

**Rust (Backend) + Elm (Frontend) + API-First Design + MCP Safety Layer**

---

## Table of Contents

0. [Development Principles](#0-development-principles)
1. [Stack Viability Assessment](#1-stack-viability-assessment)
2. [API-First Design Pattern](#2-api-first-design-pattern)
3. [Rust Backend Architecture](#3-rust-backend-architecture)
4. [Elm Frontend Architecture](#4-elm-frontend-architecture)
5. [Database Schema (PostgreSQL)](#5-database-schema-postgresql)
6. [MCP Safety Layer](#6-mcp-safety-layer)
7. [Migration Strategy & Timeline](#7-migration-strategy--timeline)
8. [Security Comparison](#8-security-comparison)
9. [Infrastructure & Deployment](#9-infrastructure--deployment)
10. [Risks & Mitigations](#10-risks--mitigations)
11. [Verdict: Elm/Rust vs Ruby/Rails](#11-verdict-elmrust-vs-rubyrails)
12. [Reference Architecture — Twolebot Patterns](#12-reference-architecture--patterns-extracted-from-twolebot)

---

> **WARNING — `twolebot-main/` is a REFERENCE PROJECT ONLY.**
> It must **never** be deployed, built, or included in any Qictrader build artifact.
> It exists solely as an architectural reference for Rust patterns, crate choices,
> testing practices, and code quality standards. When this workspace becomes a git
> repo, add `twolebot-main/` to the root `.gitignore`.

---

## 0. Development Principles

> These principles govern all development on this project. They are non-negotiable.

### Pure Functional Style

- Pure functions by default, everywhere possible
- Immutable data — transform, don't mutate
- Robust error handling at every level
- Descriptive logging for observability
- Design processes to be resumable

### Error Handling

- Use `Result` types over exceptions and panics
- No unsafe unwraps. No panics.
- Handle errors explicitly — never swallow them
- Propagate errors with context
- Validate at system boundaries (user input, external APIs); trust internal code paths

### Types-First Development

Always start by defining or updating the **types first** (enums, structs, custom types, type aliases), then let the compiler guide you through the remaining changes. The compiler errors are your task list.

Maintain a centralized types file. All shared types live there. When requirements change, alter the types first — then follow the compiler errors to every call site that needs updating.

### Testing

No mocks unless explicitly asked. Preference order:

1. Property-based tests
2. Integration tests
3. Unit tests

Use pure functions to make code testable without mocking.

### Implementation

- Actually implement — no mocks, no TODOs, no placeholders
- Avoid over-engineering; only build what's needed now
- Don't design for hypothetical future needs
- Three similar lines > premature abstraction
- Keep solutions simple and focused

---

## 1. Stack Viability Assessment

### The Short Answer

**Yes — Rust + Elm + API-first is a viable and in several ways superior alternative to Ruby/Rails for a P2P trading platform.** The trade-off is longer development time in exchange for dramatically stronger correctness and safety guarantees.

### Rust for Backend

| Factor | Assessment |
|--------|-----------|
| **Memory safety** | Rust's ownership system eliminates use-after-free, buffer overflows, and data races at compile time. No GC pauses. These are entire vulnerability classes that simply cannot exist. |
| **Type safety** | Algebraic data types + exhaustive pattern matching means every state is explicitly handled. No null pointer exceptions, no unhandled cases. |
| **Concurrency** | Fearless concurrency via the borrow checker. Async/await with Tokio. Ideal for handling blockchain RPC calls, WebSocket connections, and database queries concurrently. |
| **Performance** | 10-100x faster than Node.js/Ruby for CPU-bound work. Near-zero memory overhead. Critical for real-time price feeds, escrow monitoring, and multi-chain wallet operations. |
| **Web frameworks** | Axum (built on Tower/Hyper) is production-ready, well-documented, and used at scale (Discord, Cloudflare, AWS). |
| **Database** | SQLx provides compile-time checked SQL queries — a typo in a query is a compile error, not a runtime crash. |
| **Ecosystem maturity** | Rust web ecosystem is younger than Rails but production-ready. Major gaps: no equivalent to Rails generators/scaffolding, less "batteries included". |
| **Development speed** | 1.5-2.5x slower than Ruby/TypeScript for initial development. The compiler is strict — but what compiles is far more likely to be correct. |
| **Hiring** | Smaller talent pool than Ruby/JS. However, Rust developers tend to be experienced and produce high-quality code. |

**Key Rust crates for this project:**

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP framework (async, Tower-based) |
| `sqlx` | Compile-time verified PostgreSQL queries |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization (zero-cost abstractions) |
| `tower` | Middleware (rate limiting, timeouts, auth) |
| `tower-http` | HTTP-specific middleware (CORS, compression, tracing) |
| `jsonwebtoken` | JWT encoding/decoding |
| `argon2` | Password hashing |
| `aes-gcm` | AES-256-GCM encryption for wallet keys |
| `totp-rs` | TOTP 2FA |
| `ethers-rs` | Ethereum interaction |
| `solana-sdk` | Solana interaction |
| `redis` | Redis client (async) |
| `tracing` | Structured logging |
| `utoipa` | OpenAPI spec generation from code |
| `validator` | Struct-level validation |

### Elm for Frontend

| Factor | Assessment |
|--------|-----------|
| **Zero runtime exceptions** | Elm guarantees no runtime crashes. In 8+ years, no Elm application has produced a runtime exception in production. For a financial UI, this is extraordinary. |
| **Pure functions** | Every function is pure — same input always gives same output. No hidden state mutations. No "it works on my machine" bugs. |
| **The Elm Architecture (TEA)** | Model → Update → View. Every state change is explicit and traceable. No surprise re-renders, no state synchronization bugs. |
| **Exhaustive pattern matching** | The compiler forces you to handle every possible case. Forgot to handle the `Disputed` escrow state? Compile error. |
| **Immutable data** | All data is immutable by default. No accidental mutations, no stale references. |
| **Refactoring confidence** | Change a type and the compiler shows every place that needs updating. Large refactors are safe. |
| **JS interop** | Elm communicates with JavaScript via Ports (outgoing commands + incoming subscriptions). Firebase Auth, WebSocket, and clipboard APIs work through ports. |
| **Ecosystem size** | Smaller than React. Fewer pre-built components. You'll build more from scratch, but what you build will be reliable. |
| **CSS/Styling** | `elm-ui` (layout without CSS) or `elm-css` (typed CSS). No global CSS conflicts. |
| **Learning curve** | Functional programming paradigm requires mindset shift. No classes, no `this`, no mutation. Developers who learn it tend to strongly prefer it. |

**Key Elm packages for this project:**

| Package | Purpose |
|---------|---------|
| `elm/http` | HTTP requests |
| `elm/json` | JSON encoding/decoding |
| `elm/url` | URL parsing and routing |
| `elm/time` | Time handling |
| `elm/file` | File uploads |
| `elm-community/typed-svg` | SVG charts |
| `mdgriffith/elm-ui` | Layout and styling (no CSS) |
| `elm-explorations/markdown` | Markdown rendering |
| `NoRedInk/elm-json-decode-pipeline` | Ergonomic JSON decoders |
| `rtfeldman/elm-spa` | Single-page app routing |

### API-First: Why Both Languages Benefit

In an API-first approach, the **OpenAPI specification is written before any code**. This is particularly powerful with Rust + Elm because:

1. **Rust**: `utoipa` generates the OpenAPI spec from Rust type definitions. Types are the source of truth. If the API changes, the spec changes automatically.
2. **Elm**: Code generators (`elm-open-api`) produce type-safe HTTP clients from the spec. Decoders, encoders, and request functions are generated automatically.
3. **Contract guarantee**: If the backend changes an endpoint's response shape, the Elm frontend gets a compile error — not a runtime crash hours later.

This creates a **compile-time contract between frontend and backend**, which is something neither React+Node nor Rails+ERB can provide.

---

## 2. API-First Design Pattern

### What API-First Means

API-first means the API contract is the **first artifact** — designed and agreed upon before any implementation code is written.

```
Step 1: Write OpenAPI 3.1 specification
Step 2: Review + agree on contract
Step 3: Generate server stubs (Rust) + client code (Elm) from spec
Step 4: Implement server logic against generated types
Step 5: Frontend consumes generated client — always type-safe
```

### OpenAPI Specification Structure

```
openapi/
├── openapi.yaml              # Root spec file
├── paths/
│   ├── auth.yaml             # /api/v1/auth/* endpoints
│   ├── users.yaml            # /api/v1/users/* endpoints
│   ├── offers.yaml           # /api/v1/offers/* endpoints
│   ├── trades.yaml           # /api/v1/trades/* endpoints
│   ├── escrow.yaml           # /api/v1/escrow/* endpoints
│   ├── wallet.yaml           # /api/v1/wallet/* endpoints
│   ├── custodial-wallet.yaml # /api/v1/custodial-wallet/* endpoints
│   ├── gas.yaml              # /api/v1/gas/* endpoints
│   ├── prices.yaml           # /api/v1/prices/* endpoints
│   ├── kyc.yaml              # /api/v1/kyc/* endpoints
│   ├── affiliate.yaml        # /api/v1/affiliate/* endpoints
│   ├── reseller.yaml         # /api/v1/reseller/* endpoints
│   ├── notifications.yaml    # /api/v1/notifications/* endpoints
│   ├── support.yaml          # /api/v1/support/* endpoints
│   ├── mod.yaml              # /api/v1/mod/* endpoints
│   ├── admin.yaml            # /api/v1/admin/* endpoints
│   └── config.yaml           # /api/v1/config/* endpoints
├── components/
│   ├── schemas/
│   │   ├── user.yaml         # User, PublicProfile, UserSettings
│   │   ├── offer.yaml        # Offer, CreateOfferRequest
│   │   ├── trade.yaml        # Trade, TradeMessage, CreateTradeRequest
│   │   ├── escrow.yaml       # Escrow, EscrowStats
│   │   ├── wallet.yaml       # Wallet, WalletBalance, Transaction
│   │   ├── notification.yaml # Notification
│   │   └── common.yaml       # Pagination, Error, Timestamp
│   ├── parameters/
│   │   └── common.yaml       # Shared query/path parameters
│   ├── responses/
│   │   └── errors.yaml       # 400, 401, 403, 404, 500
│   └── securitySchemes/
│       └── bearer.yaml       # Bearer token auth
└── scripts/
    ├── generate-rust.sh      # Generate Rust server types
    ├── generate-elm.sh       # Generate Elm client code
    └── validate.sh           # Validate spec consistency
```

### Example: Offer Endpoints Spec

```yaml
# openapi/paths/offers.yaml
/api/v1/offers:
  get:
    operationId: listOffers
    summary: List marketplace offers
    tags: [Offers]
    parameters:
      - $ref: '../components/parameters/common.yaml#/Limit'
      - $ref: '../components/parameters/common.yaml#/Offset'
      - name: type
        in: query
        schema:
          $ref: '../components/schemas/offer.yaml#/OfferType'
      - name: cryptocurrency
        in: query
        schema:
          $ref: '../components/schemas/offer.yaml#/CryptoCurrency'
      - name: fiat_currency
        in: query
        schema:
          $ref: '../components/schemas/offer.yaml#/FiatCurrency'
      - name: sort_by
        in: query
        schema:
          type: string
          enum: [price, created_at, trades]
    responses:
      '200':
        description: Paginated list of offers
        content:
          application/json:
            schema:
              $ref: '../components/schemas/offer.yaml#/OfferListResponse'
  post:
    operationId: createOffer
    summary: Create a new offer
    tags: [Offers]
    security:
      - BearerAuth: []
    requestBody:
      required: true
      content:
        application/json:
          schema:
            $ref: '../components/schemas/offer.yaml#/CreateOfferRequest'
    responses:
      '201':
        description: Offer created
        content:
          application/json:
            schema:
              $ref: '../components/schemas/offer.yaml#/OfferResponse'
      '400':
        $ref: '../components/responses/errors.yaml#/BadRequest'
      '401':
        $ref: '../components/responses/errors.yaml#/Unauthorized'
```

### Generated Code Flow

```
openapi.yaml
    │
    ├──→ utoipa (Rust) ──→ Type-safe request/response structs
    │                       + validation + serialization
    │
    └──→ elm-open-api ──→ Type-safe HTTP client module
                          + JSON decoders/encoders
                          + Request functions
```

The frontend and backend are **mechanically guaranteed** to agree on every request/response shape, every enum variant, every optional field. A mismatch is a compile error on both sides.

---

## 3. Rust Backend Architecture

### Project Structure

```
qictrader-api/
├── Cargo.toml
├── Cargo.lock
├── openapi/                      # OpenAPI spec (source of truth)
│   └── ...
├── migrations/                   # SQLx migrations
│   ├── 001_create_users.sql
│   ├── 002_create_offers.sql
│   ├── ...
├── src/
│   ├── main.rs                   # Entry point, server startup
│   ├── config.rs                 # Environment config (typed, validated)
│   ├── app.rs                    # Axum router assembly
│   ├── error.rs                  # Error types (AppError enum)
│   │
│   ├── api/                      # HTTP layer (thin — delegates to services)
│   │   ├── mod.rs
│   │   ├── auth.rs               # Auth endpoints
│   │   ├── users.rs              # User endpoints
│   │   ├── offers.rs             # Offer endpoints
│   │   ├── trades.rs             # Trade endpoints
│   │   ├── escrow.rs             # Escrow endpoints
│   │   ├── wallet.rs             # Wallet endpoints
│   │   ├── custodial_wallet.rs   # Custodial wallet endpoints
│   │   ├── gas.rs                # Gas/treasury endpoints
│   │   ├── prices.rs             # Price feed endpoints
│   │   ├── kyc.rs                # KYC endpoints
│   │   ├── affiliate.rs          # Affiliate endpoints
│   │   ├── reseller.rs           # Reseller endpoints
│   │   ├── notifications.rs      # Notification endpoints
│   │   ├── support.rs            # Support ticket endpoints
│   │   ├── moderation.rs         # Moderator endpoints
│   │   └── admin.rs              # Admin endpoints
│   │
│   ├── models/                   # Database models (SQLx FromRow)
│   │   ├── mod.rs
│   │   ├── user.rs               # User, UserSettings
│   │   ├── offer.rs              # Offer, OfferVersion
│   │   ├── trade.rs              # Trade, TradeMessage
│   │   ├── escrow.rs             # Escrow, EscrowWallet
│   │   ├── wallet.rs             # CustodialWallet, WalletBalance, WalletChain
│   │   ├── ledger.rs             # LedgerEntry (append-only)
│   │   ├── notification.rs
│   │   ├── rating.rs
│   │   ├── kyc.rs
│   │   ├── affiliate.rs
│   │   └── safety_event.rs       # MCP safety events
│   │
│   ├── services/                 # Business logic (pure where possible)
│   │   ├── mod.rs
│   │   ├── auth/
│   │   │   ├── firebase.rs       # Firebase token verification
│   │   │   ├── tokens.rs         # JWT creation/verification
│   │   │   └── two_factor.rs     # TOTP setup/verify
│   │   ├── trading/
│   │   │   ├── create_trade.rs   # Atomic trade creation
│   │   │   ├── complete_trade.rs
│   │   │   ├── cancel_trade.rs
│   │   │   └── state_machine.rs  # Trade state transitions (type-safe)
│   │   ├── escrow/
│   │   │   ├── create.rs
│   │   │   ├── release.rs
│   │   │   ├── dispute.rs
│   │   │   └── balance.rs
│   │   ├── wallet/
│   │   │   ├── generate.rs       # HD wallet generation
│   │   │   ├── send.rs           # Transaction sending
│   │   │   ├── balance.rs        # Balance queries
│   │   │   └── encryption.rs     # AES-256-GCM for keys
│   │   ├── blockchain/
│   │   │   ├── bitcoin.rs
│   │   │   ├── ethereum.rs
│   │   │   ├── solana.rs
│   │   │   └── tron.rs
│   │   ├── pricing.rs            # CoinGecko integration
│   │   ├── reputation.rs         # Reputation calculation
│   │   ├── notification.rs       # Notification creation
│   │   ├── ledger.rs             # Append-only ledger writes
│   │   └── mcp/
│   │       ├── server.rs         # MCP server implementation
│   │       ├── tools.rs          # MCP tools (freeze, flag, etc.)
│   │       ├── resources.rs      # MCP resources (trade patterns, etc.)
│   │       └── rules.rs          # Safety rules engine
│   │
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs               # Firebase JWT extraction + verification
│   │   ├── rate_limit.rs         # Per-endpoint rate limiting
│   │   ├── request_id.rs         # X-Request-Id injection
│   │   └── logging.rs            # Structured request/response logging
│   │
│   ├── extractors/               # Axum extractors (typed request parsing)
│   │   ├── mod.rs
│   │   ├── auth.rs               # AuthUser extractor
│   │   ├── pagination.rs         # Pagination params extractor
│   │   └── validated.rs          # Validated<T> extractor (auto-validation)
│   │
│   ├── types/                    # Shared types (generated from OpenAPI)
│   │   ├── mod.rs
│   │   ├── api.rs                # Request/response types
│   │   ├── enums.rs              # TradeStatus, EscrowStatus, Role, etc.
│   │   └── money.rs              # Money type (amount + currency, integer-based)
│   │
│   ├── ws/                       # WebSocket (Axum native)
│   │   ├── mod.rs
│   │   ├── handler.rs            # Connection handler
│   │   ├── rooms.rs              # Room management (trade, offers, prices)
│   │   └── events.rs             # Event types
│   │
│   └── jobs/                     # Background tasks (Tokio tasks)
│       ├── mod.rs
│       ├── auto_swap.rs          # Treasury USDT → TRX
│       ├── deadline_monitor.rs   # Dispute deadline checker
│       ├── price_updater.rs      # Periodic price fetch
│       └── mcp_scanner.rs        # Periodic risk assessment
│
├── tests/
│   ├── api/                      # Integration tests per endpoint
│   ├── services/                 # Unit tests for business logic
│   └── fixtures/                 # Test data
│
├── Dockerfile
├── docker-compose.yml
└── sqlx-data.json                # Offline query verification cache
```

### Why Rust's Type System Matters for This Project

In the current Node.js backend, trade status is a string. Nothing prevents setting it to `"banana"`. In Rust:

```rust
// Trade status is an enum — only valid states exist
#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "trade_status", rename_all = "snake_case")]
pub enum TradeStatus {
    Created,
    EscrowFunded,
    Paid,
    Released,
    Disputed,
    Resolved,
    Cancelled,
}

// State transitions are enforced at compile time
impl TradeStatus {
    pub fn can_transition_to(&self, target: &TradeStatus) -> bool {
        matches!(
            (self, target),
            (TradeStatus::Created, TradeStatus::EscrowFunded)
                | (TradeStatus::Created, TradeStatus::Cancelled)
                | (TradeStatus::EscrowFunded, TradeStatus::Paid)
                | (TradeStatus::Paid, TradeStatus::Released)
                | (TradeStatus::Paid, TradeStatus::Disputed)
                | (TradeStatus::Disputed, TradeStatus::Resolved)
        )
    }
}

// Money is never a float — it's a typed integer in smallest units
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Money {
    pub amount: i64,     // Satoshis for BTC, wei for ETH, cents for USD
    pub currency: Currency,
}

impl Money {
    pub fn checked_sub(self, other: Money) -> Result<Money, InsufficientFunds> {
        if self.currency != other.currency {
            return Err(InsufficientFunds::CurrencyMismatch);
        }
        let result = self.amount.checked_sub(other.amount)
            .ok_or(InsufficientFunds::Overflow)?;
        if result < 0 {
            return Err(InsufficientFunds::NotEnough { available: self, required: other });
        }
        Ok(Money { amount: result, currency: self.currency })
    }
}
```

The current backend has:
- Float money values → Rust makes this impossible with the `Money` type
- Race conditions on balance updates → Rust's `SELECT FOR UPDATE` in SQLx transactions
- Missing validation on many routes → Rust's `Validated<T>` extractor rejects invalid input before the handler runs
- Nullable fields causing runtime crashes → Rust's `Option<T>` forces explicit handling

### Compile-Time SQL Verification

```rust
// SQLx checks this query against the actual database at compile time.
// Wrong column name? Wrong type? Compile error.
let trade = sqlx::query_as!(
    Trade,
    r#"
    SELECT id, trade_number, offer_id, buyer_id, seller_id,
           cryptocurrency, crypto_amount, fiat_amount, fiat_currency,
           status as "status: TradeStatus",
           escrow_status as "escrow_status: EscrowStatus",
           created_at, updated_at
    FROM trades
    WHERE id = $1 AND (buyer_id = $2 OR seller_id = $2)
    "#,
    trade_id,
    user_id
)
.fetch_optional(&pool)
.await?;

match trade {
    Some(t) => Ok(Json(TradeResponse::from(t))),
    None => Err(AppError::NotFound("Trade not found")),
}
```

If the `trades` table schema changes and this query becomes invalid, `cargo build` fails immediately. No runtime surprises.

---

## 4. Elm Frontend Architecture

### Project Structure

```
qictrader-web/
├── elm.json                      # Elm package manifest
├── src/
│   ├── Main.elm                  # Entry point, routing, subscriptions
│   ├── Route.elm                 # URL → Route parser
│   ├── Session.elm               # Auth session state
│   │
│   ├── Api/                      # Generated from OpenAPI spec
│   │   ├── Auth.elm              # Auth API client
│   │   ├── Users.elm             # Users API client
│   │   ├── Offers.elm            # Offers API client
│   │   ├── Trades.elm            # Trades API client
│   │   ├── Escrow.elm            # Escrow API client
│   │   ├── Wallet.elm            # Wallet API client
│   │   ├── CustodialWallet.elm   # Custodial wallet API client
│   │   ├── Gas.elm               # Gas API client
│   │   ├── Prices.elm            # Prices API client
│   │   ├── Kyc.elm               # KYC API client
│   │   ├── Affiliate.elm         # Affiliate API client
│   │   ├── Notifications.elm     # Notifications API client
│   │   ├── Support.elm           # Support API client
│   │   ├── Mod.elm               # Moderator API client
│   │   └── Admin.elm             # Admin API client
│   │
│   ├── Types/                    # Generated from OpenAPI schemas
│   │   ├── User.elm
│   │   ├── Offer.elm
│   │   ├── Trade.elm
│   │   ├── Escrow.elm
│   │   ├── Wallet.elm
│   │   ├── Notification.elm
│   │   ├── Money.elm
│   │   └── Common.elm            # Pagination, Errors, etc.
│   │
│   ├── Page/                     # Pages (each is a TEA module)
│   │   ├── Home.elm
│   │   ├── Login.elm
│   │   ├── Signup.elm
│   │   ├── ForgotPassword.elm
│   │   ├── Dashboard.elm
│   │   ├── Marketplace.elm
│   │   ├── Offer/
│   │   │   ├── Create.elm
│   │   │   ├── Detail.elm
│   │   │   ├── Edit.elm
│   │   │   └── FundEscrow.elm
│   │   ├── Trade/
│   │   │   ├── Detail.elm        # Trade page with chat
│   │   │   ├── History.elm
│   │   │   └── Active.elm
│   │   ├── Wallet/
│   │   │   ├── Overview.elm
│   │   │   ├── Deposit.elm
│   │   │   ├── Withdraw.elm
│   │   │   └── Manage.elm
│   │   ├── Escrow/
│   │   │   ├── List.elm
│   │   │   └── Detail.elm
│   │   ├── Profile/
│   │   │   ├── Own.elm
│   │   │   └── Public.elm
│   │   ├── Settings/
│   │   │   ├── General.elm
│   │   │   ├── Security.elm
│   │   │   └── Verification.elm
│   │   ├── Affiliate.elm
│   │   ├── Reseller.elm
│   │   ├── Notifications.elm
│   │   ├── Support/
│   │   │   ├── Create.elm
│   │   │   └── Detail.elm
│   │   ├── Moderator/
│   │   │   ├── Dashboard.elm
│   │   │   ├── Disputes.elm
│   │   │   ├── DisputeDetail.elm
│   │   │   ├── Reports.elm
│   │   │   ├── Users.elm
│   │   │   └── UserDetail.elm
│   │   └── Static/
│   │       ├── About.elm
│   │       ├── Faq.elm
│   │       ├── SecurityTips.elm
│   │       ├── TradingGuide.elm
│   │       └── HowEscrowWorks.elm
│   │
│   ├── Component/                # Reusable UI components
│   │   ├── Header.elm
│   │   ├── Footer.elm
│   │   ├── Sidebar.elm
│   │   ├── Toast.elm
│   │   ├── Modal.elm
│   │   ├── LoadingSkeleton.elm
│   │   ├── EmptyState.elm
│   │   ├── OfferCard.elm
│   │   ├── TradeChat.elm
│   │   ├── EscrowStepper.elm
│   │   ├── RatingStars.elm
│   │   ├── QrCode.elm
│   │   ├── CopyButton.elm
│   │   ├── FileUpload.elm
│   │   ├── Pagination.elm
│   │   ├── FilterBar.elm
│   │   ├── StatusBadge.elm
│   │   └── Chart.elm
│   │
│   ├── Port/                     # JavaScript interop
│   │   ├── Firebase.elm          # Firebase Auth ports
│   │   ├── WebSocket.elm         # WebSocket ports
│   │   ├── Clipboard.elm         # Copy to clipboard
│   │   ├── LocalStorage.elm      # Session persistence
│   │   └── FileReader.elm        # File reading for uploads
│   │
│   └── Shared/
│       ├── Theme.elm             # Dark/light theme colors
│       ├── Icons.elm             # SVG icons
│       ├── Formatter.elm         # Date, money, address formatters
│       └── Validation.elm        # Form validation helpers
│
├── ports/                        # JavaScript port implementations
│   ├── firebase.js               # Firebase SDK integration
│   ├── websocket.js              # Socket.IO or native WebSocket
│   ├── clipboard.js              # Clipboard API
│   └── storage.js                # localStorage
│
├── static/                       # Static assets
│   ├── index.html
│   ├── favicon.ico
│   └── images/
│
├── tests/                        # Elm tests
│   ├── PageTests/
│   ├── ApiTests/
│   └── TypeTests/
│
└── scripts/
    ├── generate-api.sh           # Generate Api/ and Types/ from OpenAPI
    └── build.sh                  # Production build
```

### How Elm Handles What React Does

| React Pattern | Elm Equivalent |
|--------------|----------------|
| `useState` / `useReducer` | `Model` + `update` function (every page) |
| `useEffect` | `subscriptions` + `Cmd` (commands) |
| Redux / Zustand | Built into TEA — no external state library needed |
| React Query | Custom `RemoteData` type (Loading / Success / Failure) |
| `null` / `undefined` | `Maybe a` (compiler forces handling of `Nothing`) |
| `try/catch` | `Result error value` (compiler forces handling of `Err`) |
| TypeScript interfaces | Elm records + custom types (actually enforced, not erasable) |
| React Router | `Route.elm` + `Browser.Navigation` |
| CSS Modules / Tailwind | `elm-ui` (no CSS at all) or `elm-css` (typed CSS) |

### Elm Port Pattern for Firebase Auth

```elm
-- Port/Firebase.elm
port module Port.Firebase exposing (..)

-- Outgoing (Elm → JS)
port signInWithEmailPassword : { email : String, password : String } -> Cmd msg
port signOut : () -> Cmd msg
port refreshToken : () -> Cmd msg

-- Incoming (JS → Elm)
port onAuthStateChanged : (Json.Decode.Value -> msg) -> Sub msg
port onSignInSuccess : (Json.Decode.Value -> msg) -> Sub msg
port onSignInError : (String -> msg) -> Sub msg
```

```javascript
// ports/firebase.js
app.ports.signInWithEmailPassword.subscribe(async ({ email, password }) => {
  try {
    const credential = await signInWithEmailAndPassword(auth, email, password);
    const idToken = await credential.user.getIdToken();
    app.ports.onSignInSuccess.send({
      uid: credential.user.uid,
      email: credential.user.email,
      idToken: idToken,
    });
  } catch (error) {
    app.ports.onSignInError.send(error.message);
  }
});
```

### Why Elm's Guarantees Matter for a Trading Platform

In the current React frontend:
- A null `escrowStatus` could crash the escrow stepper
- A WebSocket message with an unexpected shape could silently fail
- A race condition between two API calls could show stale balances
- Forgetting to handle a new `TradeStatus` variant could leave the UI broken

In Elm:
- `escrowStatus` is `Maybe EscrowStatus` — the compiler forces you to handle `Nothing`
- WebSocket messages go through a JSON decoder — malformed messages produce `Err` which you must handle
- All state updates are sequential through `update` — no race conditions possible
- Adding a new variant to `TradeStatus` causes compile errors everywhere it's used until handled

---

## 5. Database Schema (PostgreSQL)

The schema is identical to what was designed in the previous Ruby/Rails plan. Rust's SQLx works directly with PostgreSQL and provides the same schema-level guarantees.

Key design decisions:
- **`BIGINT` for all monetary values** — satoshis, wei, cents. No floats.
- **Foreign keys on all relationships** — `trades.offer_id REFERENCES offers(id)`
- **CHECK constraints** — `status IN ('created', 'escrow_funded', ...)`, `amount > 0`, `buyer_id != seller_id`
- **Append-only ledger** — `REVOKE UPDATE, DELETE ON ledger_entries FROM app_user`
- **Encrypted columns** — 2FA secrets, wallet mnemonics, private keys
- **Soft deletes** — `discarded_at TIMESTAMPTZ` on financial records
- **UUID primary keys** — no sequential ID information leakage
- **Composite indexes** — for all common query patterns

See the previous REWRITE_PLAN's Section 3 for the full SQL schema. It applies unchanged — PostgreSQL is the database regardless of whether the backend is Rust or Ruby.

---

## 6. MCP Safety Layer

### Rust MCP Server

Rust is an excellent language for implementing an MCP server because:
- Low latency for real-time fraud checks
- Memory safety for a security-critical component
- Strong typing for tool/resource definitions

```rust
// src/services/mcp/tools.rs

use mcp_sdk::{Tool, ToolInput, ToolResult};

pub struct FreezeAccountTool;

impl Tool for FreezeAccountTool {
    fn name(&self) -> &str { "freeze_account" }

    fn description(&self) -> &str {
        "Temporarily freeze a user account due to suspicious activity. \
         Requires human approval before execution."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "user_id": { "type": "string", "format": "uuid" },
                "reason": { "type": "string" },
                "severity": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"]
                }
            },
            "required": ["user_id", "reason", "severity"]
        })
    }

    async fn execute(&self, input: ToolInput) -> ToolResult {
        // High-risk: queue for human approval instead of executing
        let event = SafetyEvent::new(
            EventType::AccountFreezeRequested,
            input.user_id,
            input.reason,
            input.severity,
        );
        self.event_store.append(event).await?;
        self.notify_moderators(event).await?;
        ToolResult::success("Account freeze queued for moderator approval")
    }
}
```

### MCP Tools (same as previous plan)

| Tool | Risk | Human Approval |
|------|------|---------------|
| `freeze_account` | HIGH | Yes |
| `flag_transaction` | MEDIUM | No (auto-flag, human reviews) |
| `pause_withdrawal` | HIGH | Yes (above threshold) |
| `escalate_dispute` | LOW | No |
| `request_kyc_review` | MEDIUM | No |
| `adjust_risk_score` | LOW | No |
| `block_ip_range` | HIGH | Yes |
| `generate_compliance_report` | LOW | No |

### MCP Resources

| Resource | Description |
|----------|-------------|
| `user_activity` | Login history, IP addresses, device fingerprints |
| `trade_patterns` | Frequency, volume, counterparty analysis |
| `withdrawal_history` | Amounts, destinations, frequency |
| `risk_metrics` | Current scores, alerts, flags |
| `dispute_history` | Past disputes, resolutions |
| `platform_metrics` | System-wide fraud rates, baselines |

---

## 7. Migration Strategy & Timeline

### Phase 1: Foundation (Weeks 1-4)

**Deliverables:** Running Rust API server with auth, core models, database

| Task | Details |
|------|---------|
| OpenAPI spec v1 | Auth, users, offers, trades endpoints specified |
| Rust project setup | Axum, SQLx, Tokio, tower-http |
| PostgreSQL schema | All migrations for core tables |
| Authentication | Firebase token verification middleware |
| Authorization | Role-based middleware (user/mod/admin/super_admin) |
| Rate limiting | tower-based per-endpoint rate limiter with Redis |
| Error handling | Typed `AppError` enum with proper HTTP status mapping |
| Health endpoints | Liveness + readiness with DB and Redis checks |
| CI/CD | `cargo clippy`, `cargo test`, `cargo audit`, Docker build |
| Elm project setup | elm-spa scaffolding, routing, theme, ports for Firebase |
| Elm API generation | Generate client from OpenAPI spec |

### Phase 2: Trading Core (Weeks 5-8)

**Deliverables:** Full offer/trade/escrow lifecycle

| Task | Details |
|------|---------|
| OpenAPI spec v2 | Escrow, wallet, gas endpoints |
| Offers CRUD | With state machine, versioning, validation |
| Atomic trade creation | `BEGIN` → insert trade + update offer + create ledger entry → `COMMIT` |
| Escrow system | Custodial + offer escrow + dispute flow |
| Wallet balances | `SELECT FOR UPDATE` locking, no race conditions |
| Ledger | Append-only, immutable, transactional |
| Trade messaging | With input sanitization |
| Trade ratings | Post-completion rating system |
| Elm marketplace | Offer browse, filter, sort, create/edit |
| Elm trade pages | Trade detail, chat, status stepper |
| Elm escrow UI | Escrow creation, funding, balance display |

### Phase 3: Blockchain & Wallets (Weeks 9-12)

**Deliverables:** Multi-chain wallet operations

| Task | Details |
|------|---------|
| HD wallet generation | BIP-39/BIP-44 for BTC, ETH, SOL, TRX |
| AES-256-GCM encryption | Wallet keys encrypted at rest, no fallback keys |
| Solana service | SOL + SPL token transfers |
| Tron service | TRX + TRC-20 transfers, energy/bandwidth |
| Ethereum service | ETH + ERC-20 transfers |
| Bitcoin service | UTXO management, SegWit transactions |
| Gas treasury | Sponsored withdrawals, auto-swap |
| Price feeds | CoinGecko integration, Redis cache, WebSocket broadcast |
| Elm wallet pages | Deposit, withdraw, transfer, history |
| Elm custodial wallet | Generate, view addresses, send, export mnemonic |

### Phase 4: MCP Safety + WebSocket (Weeks 13-15)

**Deliverables:** Real-time features and AI safety layer

| Task | Details |
|------|---------|
| WebSocket server | Axum native WebSocket with rooms |
| Real-time events | Trade updates, messages, prices, notifications |
| MCP server | Rust MCP implementation with tools + resources |
| Fraud rules | Velocity checks, amount thresholds, pattern matching |
| Risk scoring | Per-user scoring, periodic batch assessment |
| Safety event logging | `mcp_safety_events` table + admin view |
| Elm WebSocket | Ports for socket connection, room management |
| Elm real-time | Live trade chat, price updates, notifications |

### Phase 5: Supporting Features (Weeks 16-19)

**Deliverables:** KYC, affiliate, moderation, admin

| Task | Details |
|------|---------|
| KYC system | Encrypted document upload, moderator review |
| Affiliate program | Referrals, earnings, tiers, payouts |
| Reseller system | Markup offers, reseller trades |
| Notifications | DB notifications + WebSocket delivery |
| Support tickets | Create, message, close |
| Moderation tools | Disputes, reports, user actions, audit logs |
| Admin dashboard | System analytics, treasury, diagnostics |
| WhatsApp escrow links | Link generation, QR codes, tracking |
| Elm moderator pages | Full moderator dashboard |
| Elm settings | General, security, verification tabs |
| Elm affiliate/reseller | Dashboard + management pages |

### Phase 6: Polish + Migration (Weeks 20-22)

**Deliverables:** Production-ready system

| Task | Details |
|------|---------|
| Data migration | Firestore → PostgreSQL migration scripts |
| Dual-write validation | Both systems running in parallel |
| Frontend parity | Feature-by-feature comparison against current UI |
| Integration testing | Full API test suite against OpenAPI spec |
| Performance testing | Load testing under production traffic patterns |
| Security audit | `cargo audit` + `clippy` + manual review |
| Penetration testing | External security assessment |
| Cutover plan | Blue-green deployment, rollback strategy |
| Monitoring | Prometheus + Grafana dashboards |

---

## 8. Security Comparison

| Vulnerability (Current) | Current Stack | Rust + Elm + API-First |
|---|---|---|
| No rate limiting | CRITICAL — not applied | tower-based middleware, applied globally, per-route configurable |
| Hardcoded encryption keys | CRITICAL — fallback to dev keys | `config.rs` fails to start if keys missing. No fallbacks. Compile-time checked. |
| 2FA tokens leak auth credentials | CRITICAL — idToken in JWT payload | Server-side session in Redis. 2FA token contains only session ID. |
| 2FA secrets in plaintext | CRITICAL — raw in Firestore | AES-256-GCM encrypted column. Decrypted only for verification. |
| TOTP code logged to console | CRITICAL — `console.log(expectedCode)` | `tracing` crate with field redaction. Secrets never appear in logs. |
| No Firestore security rules | CRITICAL — direct DB access | PostgreSQL. No client-side DB access exists. All access via API. |
| Float money values | MEDIUM — JavaScript `number` | `Money` struct with `i64` amount. Arithmetic overflow is checked. |
| Race conditions on transfers | HIGH — batch without transactional read | `sqlx::Transaction` with `SELECT FOR UPDATE`. Serialized access. |
| No input validation on routes | HIGH — schemas exist but not applied | `Validated<T>` extractor. If struct has validation, it runs. No opt-in. |
| Missing auth on user routes | HIGH — `createUser` public | Axum middleware layers. Routes without auth middleware don't compile (type mismatch). |
| String-typed enums | MEDIUM — status can be any string | Rust enums. Invalid state = compile error. |
| Unhandled error cases | HIGH — catch-all `any` types | `Result<T, E>` everywhere. Unhandled errors = compile error. |
| Frontend runtime crashes | MEDIUM — null/undefined | Elm: zero runtime exceptions since 2016. `Maybe` + `Result` enforced. |
| API contract drift | HIGH — frontend/backend out of sync | OpenAPI spec → generated Rust types + Elm client. Drift = compile error on both sides. |
| No audit trail | MEDIUM — no systematic tracking | Append-only `ledger_entries` + `mcp_safety_events`. DB-level immutability. |
| No fraud detection | None | MCP safety layer with AI + rules engine |
| Unsafe concurrency | MEDIUM — JS single-threaded, no locking | Rust borrow checker prevents data races at compile time |

---

## 9. Infrastructure & Deployment

```
┌─────────────────────────────────────────────┐
│           Load Balancer (Cloudflare)         │
└─────────────────┬───────────────────────────┘
                  │
        ┌─────────┴─────────┐
        ▼                   ▼
┌──────────────┐   ┌──────────────┐
│  Rust API    │   │  Rust API    │   ← Axum (multi-threaded, async)
│  Instance 1  │   │  Instance 2  │     10-100x lower memory than Node/Ruby
└──────┬───────┘   └──────┬───────┘
       │                  │
       ▼                  ▼
┌─────────────────────────────────┐
│   PostgreSQL 16 (Managed)       │   ← Primary + read replica
│   + pgcrypto, pg_trgm, uuid    │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│   Redis 7 (Managed)             │   ← Rate limiting, sessions, cache, WS
└─────────────────────────────────┘

┌─────────────────────────────────┐
│   Elm SPA (Static CDN)          │   ← Cloudflare Pages / S3 + CloudFront
│   ~200KB compiled JS (gzipped)  │     No server required
└─────────────────────────────────┘

┌─────────────────────────────────┐
│   MCP Safety Server (Sidecar)   │   ← Rust binary, same host or container
└─────────────────────────────────┘
```

### Resource Comparison

| Metric | Current (Node.js) | Rust |
|--------|-------------------|------|
| Memory per instance | 200-500 MB | 10-50 MB |
| Startup time | 2-5 seconds | <100 ms |
| Requests/sec (API) | ~5,000 | ~50,000+ |
| Compiled frontend size | ~1-3 MB (React+deps) | ~100-200 KB (Elm) |
| Cold start (serverless) | 1-3 seconds | <50 ms |

---

## 10. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| **Rust learning curve** | High | High | Start with experienced Rustacean as lead. Extensive documentation. Rust's compiler errors are famously helpful. |
| **Elm ecosystem gaps** | Medium | Medium | Use ports for JS libraries (Firebase, Socket.IO). Build custom components where needed. Elm's small size means components are small and predictable. |
| **Longer development time** | High | Medium | ~22 weeks vs ~16 for Rails. But: fewer bugs in production, less maintenance long-term. The compiler catches what tests miss. |
| **Elm/Firebase interop** | Medium | Medium | Well-documented port pattern. Firebase JS SDK works through ports. No deep Elm-Firebase integration needed — just token passing. |
| **Hiring difficulty** | Medium | High | Rust is growing rapidly (most-loved language 8 years running). Elm community is smaller but dedicated. Remote hiring expands the pool. |
| **Blockchain crate maturity** | Medium | Medium | `ethers-rs` is mature. `solana-sdk` is official. Tron and Bitcoin may need custom HTTP client wrappers to JSON-RPC endpoints. |
| **Migration complexity** | Medium | High | Same as any rewrite. Firestore → PostgreSQL migration, dual-write period, feature parity verification. |
| **WebSocket in Elm** | Low | Low | Axum has native WebSocket support. Elm receives messages through ports. Pattern is well-established. |

---

## 11. Verdict: Elm/Rust vs Ruby/Rails

### Side-by-Side Comparison

| Dimension | Ruby/Rails | Rust/Elm |
|-----------|-----------|----------|
| **Development speed** | Fast (convention over configuration) | Slower (compiler is strict, more explicit) |
| **Time to MVP** | ~16 weeks | ~22 weeks |
| **Runtime safety** | Good (Rails has many guards) | Exceptional (compiler prevents most bugs) |
| **Memory safety** | GC handles it (no manual memory) | Ownership system (zero-cost, no GC) |
| **Type safety** | Optional (Sorbet) or dynamic | Mandatory (Rust + Elm both statically typed) |
| **Performance** | Good enough for most loads | 10-100x faster, 10x less memory |
| **Frontend reliability** | React — good with TypeScript | Elm — zero runtime exceptions |
| **API contract enforcement** | Manual (tests + docs) | Compile-time (OpenAPI → generated types both sides) |
| **Concurrency bugs** | Possible (Ruby has GIL, but async issues) | Impossible (borrow checker prevents data races) |
| **Ecosystem maturity** | Excellent (gems for everything) | Good but smaller (fewer ready-made solutions) |
| **Hiring** | Moderate pool | Smaller pool but growing rapidly |
| **Long-term maintenance** | Good (Rails conventions) | Excellent (if it compiles, it's likely correct) |
| **Financial app suitability** | Good (ActiveRecord, Money gem) | Excellent (type-safe Money, compile-time SQL) |
| **Operational cost** | Moderate (requires more instances) | Low (fewer instances, less memory) |

### Recommendation

**For Qictrader specifically — a platform handling real money, cryptocurrency, escrow, and multi-chain wallets — the Rust/Elm stack is the stronger choice** despite the longer development timeline.

The reasoning:

1. **Money demands correctness.** Every vulnerability in the security audit stems from a type mismatch, a missing check, or an unhandled case. Rust and Elm make these compile errors, not production incidents.

2. **API-first eliminates contract drift.** The current codebase has validation schemas that exist but aren't applied. With API-first + code generation, the contract is enforced mechanically.

3. **The investment pays off in maintenance.** The extra 6 weeks of development buys dramatically fewer production bugs, lower operational costs, and safer refactoring for years to come.

4. **The MCP safety layer is natural in Rust.** Low-latency fraud detection, concurrent blockchain monitoring, and real-time risk scoring are exactly what Rust excels at.

5. **Elm eliminates an entire class of frontend bugs.** For a financial UI where showing the wrong balance or crashing mid-trade has real consequences, Elm's zero-runtime-exception guarantee is not a luxury — it's a requirement.

The trade-off is real: Rust and Elm require more upfront investment in learning and development. But for a financial application handling custodial wallets and escrow, **correctness is not optional**.

### Timeline Summary

| Phase | Duration | Deliverables |
|-------|----------|-------------|
| 1. Foundation | 4 weeks | Auth, core models, OpenAPI spec, Elm setup |
| 2. Trading Core | 4 weeks | Offers, trades, escrow, wallet balances, ledger |
| 3. Blockchain & Wallets | 4 weeks | Multi-chain HD wallets, send/receive, gas treasury |
| 4. MCP Safety + WebSocket | 3 weeks | Real-time features, fraud detection, risk scoring |
| 5. Supporting Features | 4 weeks | KYC, affiliate, moderation, admin, notifications |
| 6. Polish + Migration | 3 weeks | Data migration, testing, security audit, cutover |
| **Total** | **22 weeks** | Full production-ready rewrite |

---

## 12. Reference Architecture — Patterns Extracted from Twolebot

> The `twolebot-main/` directory contains a production Rust application (Telegram bot
> with Claude AI integration, Axum HTTP server, SQLite storage, MCP server, Elm frontend).
> While it is a different domain, its Rust architecture, crate choices, testing practices,
> and code quality standards are directly applicable to Qictrader. This section captures
> every transferable pattern.

### 12.1 Project Structure Principles

**Single-crate with module-based organisation (not workspace):**

```
src/
├── lib.rs               # Public module exports + crate-level lint attrs
├── main.rs              # Entry point, CLI, startup orchestration
├── config.rs            # Typed configuration (CLI + runtime DB)
├── error.rs             # Central error enum + Result alias
│
├── api/                 # HTTP handlers (thin — delegates to services)
├── models/              # Database row types
├── services/            # Business logic
├── storage/             # Database access layer
├── middleware/           # Axum middleware
├── types/               # Shared types / enums
├── jobs/                # Background tasks (Tokio spawned)
└── ...domain modules
```

**Key principles observed:**
- Each module exposes a focused public API via `mod.rs`
- `lib.rs` re-exports the crate's `Result` type and `Error` type for ergonomic use
- `main.rs` is purely orchestration — no business logic
- Optional components are gated behind `Option<Arc<T>>` — the app starts without them

**Qictrader adaptation:**

```
qictrader-api/src/
├── lib.rs
├── main.rs
├── config.rs
├── error.rs
├── api/                 # Axum handlers (thin, delegate to services)
├── models/              # SQLx FromRow structs
├── services/            # Business logic (trading, escrow, wallet, blockchain, mcp)
├── storage/             # Repository pattern over PostgreSQL
├── middleware/           # Auth, rate limit, request ID, logging
├── extractors/          # Axum typed extractors (AuthUser, Validated<T>, Pagination)
├── types/               # Shared types, enums, Money
├── ws/                  # WebSocket handler + rooms
└── jobs/                # Background Tokio tasks (price feed, deadline monitor, MCP scanner)
```

### 12.2 Crate Choices — Proven Stack

The following crates are validated in twolebot production and should be adopted for Qictrader:

| Category | Crate | Version | Twolebot Usage | Qictrader Usage |
|----------|-------|---------|----------------|-----------------|
| **Async runtime** | `tokio` | 1 | `features = ["full", "signal"]` | Same — full runtime + graceful shutdown |
| **HTTP framework** | `axum` | 0.7 | `features = ["tokio", "json", "ws"]` | Same — add `ws` for trade chat |
| **HTTP middleware** | `tower-http` | 0.5 | `features = ["fs", "cors"]` | `features = ["cors", "compression", "trace"]` |
| **HTTP client** | `reqwest` | 0.12 | `features = ["json", "stream", "rustls-tls"]` | Same — for blockchain RPC, CoinGecko, Firebase |
| **Serialization** | `serde` + `serde_json` | 1 | Everywhere | Everywhere |
| **Error handling** | `thiserror` | 1 | Central `TwolebotError` enum | Central `AppError` enum |
| **Error context** | `anyhow` | 1 | For subsystem errors | For service-layer error context |
| **CLI** | `clap` | 4 | `features = ["derive", "env"]` | Same — for server config |
| **Logging** | `tracing` + `tracing-subscriber` | 0.1 / 0.3 | `features = ["json", "env-filter"]` | Same |
| **Time** | `chrono` | 0.4 | `features = ["serde"]` | Same |
| **UUIDs** | `uuid` | 1 | `features = ["v4", "serde"]` | Same |
| **JSON Schema** | `schemars` | 1.0 | MCP tool schemas | OpenAPI schema generation |
| **MCP server** | `rmcp` | 0.14 | `features = ["server", "transport-io", "transport-streamable-http-server"]` | Same |

**Additional crates for Qictrader (not in twolebot):**

| Crate | Purpose |
|-------|---------|
| `sqlx` | Compile-time verified PostgreSQL queries (twolebot uses rusqlite for SQLite) |
| `tower` | Rate limiting, timeouts, auth middleware |
| `jsonwebtoken` | JWT encoding/decoding |
| `argon2` | Password hashing |
| `aes-gcm` | AES-256-GCM encryption for wallet keys |
| `totp-rs` | TOTP 2FA |
| `ethers-rs` | Ethereum interaction |
| `solana-sdk` | Solana interaction |
| `redis` | Async Redis client |
| `utoipa` | OpenAPI spec generation from code |
| `validator` | Struct-level validation |

### 12.3 Error Handling Pattern

Twolebot uses a central error enum with `thiserror` — adopt this pattern exactly:

```rust
use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {message}")]
    NotFound { message: String },

    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Forbidden: {message}")]
    Forbidden { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Blockchain error: {message}")]
    Blockchain { message: String },

    #[error("Escrow error: {message}")]
    Escrow { message: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            AppError::Forbidden { .. } => StatusCode::FORBIDDEN,
            AppError::Validation { .. } => StatusCode::BAD_REQUEST,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = serde_json::json!({ "error": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}
```

**Key patterns from twolebot:**
- Named constructors: `AppError::not_found("Trade not found")` instead of struct literals
- `#[from]` for automatic conversion from library errors (`sqlx::Error`, `std::io::Error`)
- `IntoResponse` impl so handlers can return `Result<Json<T>, AppError>` directly
- `anyhow::Error` used only for subsystem boundaries where granular variants aren't needed

### 12.4 Crate-Level Lint Discipline

Twolebot enforces strict linting at the crate level — **adopt this exactly**:

```rust
// lib.rs and main.rs — top of file
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
```

**What this means:**
- **No `.unwrap()` in production code** — forces `?` operator or explicit error handling
- **No `.expect()` in production code** — same discipline
- **No `panic!()` in production code** — no hidden abort paths
- **Tests are exempt** — `.unwrap()` is fine in tests for clarity
- This is the single most impactful code quality rule from twolebot

### 12.5 Async Architecture Patterns

**Tokio patterns observed in twolebot:**

| Pattern | Usage | Qictrader Equivalent |
|---------|-------|---------------------|
| `CancellationToken` | Graceful shutdown coordination | Same — signal all background tasks to stop |
| `tokio::spawn` | Background tasks (polling, broadcasting) | Background tasks (price updater, deadline monitor) |
| `tokio::task::spawn_blocking` | Blocking DB operations on threadpool | Not needed with async SQLx, but useful for crypto ops |
| `tokio::sync::mpsc` | Message passing between components | Trade events, WebSocket messages |
| `tokio::sync::broadcast` | Pub/sub for events | Price updates, trade status changes |
| `tokio::sync::watch` | Shared state updates | Tunnel URL, service health status |
| `tokio::sync::Mutex` | Async mutexes (rare) | Wallet balance locking (prefer DB locks) |
| `tokio::select!` | Wait on multiple futures | Shutdown signal + update processing |
| Signal handling | `Ctrl+C` + `SIGTERM` | Same |
| 5-second shutdown timeout | Graceful drain | Same |

**Startup orchestration pattern (from twolebot `main.rs`):**
1. Parse CLI args
2. Initialize tracing
3. Load config (fail fast if invalid)
4. Initialize storage components as `Arc<T>`
5. Create optional components wrapped in `Option<Arc<T>>`
6. Build router with builder pattern
7. Spawn background tasks
8. Bind TCP listener
9. Await shutdown signal
10. Cancel all tasks, await with timeout

### 12.6 Shared State via Arc

Twolebot passes all shared state through `Arc<T>`:

```rust
let prompt_feed = Arc::new(PromptFeed::new(&config.db_path)?);
let response_feed = Arc::new(ResponseFeed::new(&config.db_path)?);
let message_store = Arc::new(MessageStore::new(&config.db_path)?);
```

**Qictrader equivalent:**
```rust
let db_pool = Arc::new(PgPool::connect(&config.database_url).await?);
let redis_pool = Arc::new(RedisPool::new(&config.redis_url)?);
let trade_service = Arc::new(TradeService::new(db_pool.clone()));
let escrow_service = Arc::new(EscrowService::new(db_pool.clone()));
```

Axum's `State<T>` extractor wraps the `Arc` — no manual cloning in handlers.

### 12.7 Builder Pattern for Router

Twolebot assembles the Axum router incrementally with a builder:

```rust
let mut builder = RouterBuilder::new(app_state)
    .config(router_config)
    .static_dir(config.frontend_dir.clone())
    .mcp(mcp_state)
    .setup(setup_state);
if let Some(ws) = work_state.clone() {
    builder = builder.work(ws);
}
let router = builder.build();
```

**Adopt this for Qictrader** — optional route groups (admin, moderation, MCP) are
conditionally attached based on configuration.

### 12.8 Testing Practices

Twolebot has 13 test files covering integration, E2E, security, property-based, and
compile-fail tests. Adopt all these layers:

#### Test Categories

| Category | Twolebot Example | Qictrader Equivalent |
|----------|-----------------|---------------------|
| **E2E lifecycle** | `e2e_flow.rs` — prompt enqueue → running → completed | Trade creation → escrow funded → paid → released |
| **Security** | `security_tests.rs` — path traversal, CORS, access control | Path traversal, JWT validation, role enforcement |
| **MCP contract** | `mcp_contract.rs` — MCP tool schema validation | MCP safety tool contract tests |
| **MCP integration** | `mcp_integration.rs` — MCP server round-trip | MCP fraud detection integration |
| **DB migration** | `runtime_db_unification.rs` — schema migration verification | SQLx migration verification |
| **Architecture** | `architecture_compile_fail.rs` — `trybuild` compile-fail tests | Module boundary enforcement |
| **Property-based** | `proptest` in e2e and security tests | Property tests for Money arithmetic, state transitions |
| **Autowork policy** | `work_autowork_policy.rs` — background task behavior | Background task behavior (deadline monitor, price updater) |

#### Test Libraries

| Library | Version | Purpose |
|---------|---------|---------|
| `proptest` | 1.0 | Property-based testing — generates random inputs |
| `tempfile` | 3 | Temporary directories for isolated test databases |
| `trybuild` | 1 | Compile-fail tests — verifies architecture boundaries |
| `tower` | 0.5 | `ServiceExt::oneshot()` for testing Axum routers without HTTP |

#### Test Patterns to Adopt

**1. Isolated test databases with `tempfile`:**
```rust
#[test]
fn test_trade_lifecycle() {
    let dir = tempdir().unwrap();
    let pool = setup_test_db(dir.path()).unwrap();
    // ... test against isolated DB
}
```

**2. Property-based tests with `proptest`:**
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_money_subtraction_never_wraps(
        a in 0i64..i64::MAX / 2,
        b in 0i64..i64::MAX / 2,
    ) {
        let result = Money::new(a, Currency::Usd)
            .checked_sub(Money::new(b, Currency::Usd));
        if a >= b {
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap().amount, a - b);
        } else {
            prop_assert!(result.is_err());
        }
    }
}
```

**3. Compile-fail tests with `trybuild`:**
```rust
#[test]
fn architecture_boundaries_are_enforced() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/private_escrow_internals.rs");
    t.compile_fail("tests/ui/direct_db_access_from_handler.rs");
}
```

**4. Security property tests:**
```rust
proptest! {
    #[test]
    fn prop_path_traversal_variants_blocked(
        prefix in "[a-z0-9]{1,10}",
        depth in 1usize..5,
        suffix in "[a-z]{1,10}"
    ) {
        // Verify ALL path traversal variants are blocked
    }
}
```

**5. Router testing without HTTP (using `tower::ServiceExt`):**
```rust
#[tokio::test]
async fn test_unauthorized_access_returns_401() {
    let router = create_test_router();
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/trades")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### 12.9 CI/CD Pipeline

Twolebot's GitHub Actions pipeline — adopt the same structure:

```yaml
# .github/workflows/tests.yml
name: Tests
on:
  push:
    branches: ["**"]
  pull_request:
jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2    # Cache cargo artifacts
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all-targets
      - run: cargo test --test mcp_contract -- --nocapture
      - run: cargo audit                 # Security audit of dependencies
```

**Additional Qictrader CI steps:**
- `cargo sqlx prepare --check` — verify offline SQL query cache is up to date
- `cargo fmt --check` — enforce formatting
- `cargo tarpaulin` — code coverage (twolebot uses 50% threshold)

### 12.10 Configuration Approach

Twolebot uses a layered config strategy:

| Layer | Mechanism | Priority |
|-------|-----------|----------|
| CLI arguments | `clap` with `#[derive(Parser)]` | Highest |
| Runtime database | Secrets/settings in SQLite | Medium |
| Defaults | Hardcoded in `Config` struct | Lowest |

**Qictrader adaptation:**

| Layer | Mechanism | Priority |
|-------|-----------|----------|
| CLI arguments | `clap` with `#[derive(Parser)]` | Highest |
| Environment variables | `clap`'s `env` feature | High |
| Config file | `config` crate (TOML) | Medium |
| Defaults | Hardcoded in `Config` struct | Lowest |

**Fail-fast principle (from twolebot):**
- Missing required config = application refuses to start
- No fallback encryption keys
- No fallback database URLs
- Config validation runs before any service initialization

### 12.11 Graceful Degradation

Twolebot starts without optional components — adopt this:

```rust
// Optional: Telegram sender (runs without if no token)
let telegram_sender: Option<Arc<TelegramSender>> = match &config.telegram_token {
    Some(token) => Some(Arc::new(TelegramSender::new(token)?)),
    None => {
        tracing::info!("No Telegram token — running without");
        None
    }
};
```

**Qictrader equivalents:**
- Start without Redis → use in-memory rate limiter, warn in logs
- Start without blockchain RPC → disable withdrawal processing, warn
- Start without MCP → disable fraud scanning, warn
- **Never** start without PostgreSQL — that's a hard dependency

### 12.12 Observability

Twolebot uses dual logging — adopt the `tracing` side:

```rust
// Structured logging with tracing
tracing::info!(
    trade_id = %trade.id,
    buyer = %trade.buyer_id,
    seller = %trade.seller_id,
    amount = trade.crypto_amount,
    "Trade created"
);

// Environment-based filter
// RUST_LOG=qictrader_api=debug,sqlx=warn,tower_http=info
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())  // JSON output for production
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

**Field redaction for sensitive data:**
```rust
tracing::info!(
    user_id = %user.id,
    // NEVER log: passwords, tokens, private keys, 2FA secrets
    "User authenticated"
);
```

### 12.13 Feature Gating for Heavy Tests

Twolebot gates expensive integration tests behind Cargo features:

```toml
[features]
default = []
spawn-tests = []  # Gate heavy integration tests

[[test]]
name = "spawned_claude_mcp"
required-features = ["spawn-tests"]
```

**Qictrader equivalent:**
```toml
[features]
default = []
blockchain-tests = []  # Gate tests that hit real blockchain RPCs
load-tests = []        # Gate performance/load tests
```

This keeps `cargo test` fast for development while allowing full integration testing in CI.

### 12.14 Summary — Twolebot Principles to Carry Forward

| # | Principle | Implementation |
|---|-----------|---------------|
| 1 | **No `.unwrap()` in production** | `#![cfg_attr(not(test), deny(clippy::unwrap_used))]` |
| 2 | **Central error enum** | `AppError` with `thiserror`, `IntoResponse`, named constructors |
| 3 | **Crate-level `Result` alias** | `pub type Result<T> = std::result::Result<T, AppError>;` |
| 4 | **Arc for shared state** | All services and stores wrapped in `Arc<T>` |
| 5 | **Optional components** | `Option<Arc<T>>` — app starts without non-critical services |
| 6 | **Builder pattern for router** | `RouterBuilder::new(state).auth(auth).work(work).build()` |
| 7 | **CancellationToken for shutdown** | Coordinated graceful shutdown across all spawned tasks |
| 8 | **Property-based testing** | `proptest` for Money arithmetic, state machines, input validation |
| 9 | **Compile-fail tests** | `trybuild` to enforce module boundaries at compile time |
| 10 | **Security tests as first-class** | Path traversal, CORS, access control tests in dedicated file |
| 11 | **Feature-gated heavy tests** | `cargo test` stays fast; CI runs `--features blockchain-tests` |
| 12 | **Fail-fast config** | Missing DB URL or encryption key = server refuses to start |
| 13 | **Structured logging** | `tracing` with JSON output, env filter, field redaction |
| 14 | **Cargo cache in CI** | `Swatinem/rust-cache@v2` — cuts CI time significantly |
| 15 | **`tempfile` for test isolation** | Every test gets its own temporary database/directory |
