# Qictrader — Complete Feature Inventory

Full catalog of every feature across the backend API and frontend application.

---

## Table of Contents

1. [Authentication & Account Management](#1-authentication--account-management)
2. [User Profiles & Social](#2-user-profiles--social)
3. [Marketplace — Offers](#3-marketplace--offers)
4. [Trading System](#4-trading-system)
5. [Escrow System](#5-escrow-system)
6. [Wallet System](#6-wallet-system)
7. [Custodial Multi-Chain Wallets](#7-custodial-multi-chain-wallets)
8. [Gas & Treasury Management](#8-gas--treasury-management)
9. [Price Feeds & Conversion](#9-price-feeds--conversion)
10. [Payment Methods](#10-payment-methods)
11. [KYC & Verification](#11-kyc--verification)
12. [Affiliate & Referral Program](#12-affiliate--referral-program)
13. [Reseller Program](#13-reseller-program)
14. [WhatsApp Escrow Links](#14-whatsapp-escrow-links)
15. [Notifications](#15-notifications)
16. [Real-Time (WebSocket)](#16-real-time-websocket)
17. [Support & Help](#17-support--help)
18. [Bug Reporting](#18-bug-reporting)
19. [Newsletter](#19-newsletter)
20. [Moderator Tools](#20-moderator-tools)
21. [Admin Tools](#21-admin-tools)
22. [Dashboard & Analytics](#22-dashboard--analytics)
23. [Settings & Preferences](#23-settings--preferences)
24. [Blockchain Integrations](#24-blockchain-integrations)
25. [Frontend UI/UX](#25-frontend-uiux)
26. [Platform Configuration](#26-platform-configuration)
27. [System & Infrastructure](#27-system--infrastructure)
28. [Reputation System](#28-reputation-system)
29. [Append-Only Financial Ledger](#29-append-only-financial-ledger)
30. [Offer Event Tracking](#30-offer-event-tracking)
31. [Soft Delete & Data Preservation](#31-soft-delete--data-preservation)
32. [Platform Fees](#32-platform-fees)
33. [Client Applications](#33-client-applications)

---

## 1. Authentication & Account Management

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/signup` | No | Register new user (email, password, username, referral code) |
| POST | `/auth/login` | No | Login with email/password, returns tokens or 2FA challenge |
| POST | `/auth/2fa/verify-login` | No | Verify TOTP code to complete 2FA login |
| POST | `/auth/refresh-token` | No | Exchange refresh token for new access token |
| POST | `/auth/oauth-sync` | Yes | Sync OAuth provider (Google, Apple) with existing account |
| GET | `/auth/verify` | Yes | Verify current token is valid |
| POST | `/auth/logout` | Yes | Logout current session |
| POST | `/auth/logout-all` | Yes | Revoke all active sessions |
| DELETE | `/auth/delete/:userId` | Yes | Soft-deactivate user account |
| POST | `/auth/reactivate/:userId` | Admin | Reactivate deactivated account |

### Frontend Features

- Email/password registration with username validation and real-time availability check
- Email/password login with "remember me"
- OAuth sign-in (Google, Apple) with account linking
- 2FA setup wizard with QR code generation
- 2FA verification during login
- 2FA disable flow
- Forgot password / reset password flow
- Token refresh with automatic retry
- Session idle timeout with warning modal
- Account deletion with confirmation
- Auth state persistence across browser sessions (Redux + localStorage)

---

## 2. User Profiles & Social

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/users/me` | Yes | Get current user profile |
| PATCH | `/users/me` | Yes | Update current user profile |
| GET | `/users/me/stats` | Yes | Get trading statistics |
| GET | `/users/profile` | Yes | Get full profile |
| PUT | `/users/profile` | Yes | Update profile fields |
| GET | `/users/settings` | Yes | Get user settings |
| PUT | `/users/settings` | Yes | Update settings |
| GET | `/users/2fa/status` | Yes | Get 2FA enabled status |
| POST | `/users/2fa/setup` | Yes | Generate 2FA secret + QR code |
| POST | `/users/2fa/verify` | Yes | Verify and enable 2FA |
| POST | `/users/2fa/disable` | Yes | Disable 2FA |
| POST | `/users/password/change` | Yes | Change password |
| GET | `/users/sessions` | Yes | List active sessions |
| DELETE | `/users/sessions/:id` | Yes | Revoke a session |
| GET | `/users/activity-log` | Yes | Get activity log |
| GET | `/users/wallet` | Yes | Get linked wallet info |
| POST | `/users/wallet/link` | Yes | Link external wallet address |
| DELETE | `/users/wallet/unlink` | Yes | Unlink wallet |
| GET | `/users/top` | Public | Get top traders leaderboard |
| GET | `/users/search` | Public | Search users by name |
| GET | `/users/check-username/:name` | Public | Check username availability |
| GET | `/users/:id/public` | Public | Get public profile |
| GET | `/users/:id/ratings` | Public | Get user ratings |
| POST | `/users/:id/report` | Yes | Report a user |
| POST | `/users/:id/block` | Yes | Block a user |
| DELETE | `/users/:id/block` | Yes | Unblock a user |
| POST | `/users/:id/untrust` | Yes | Mark user as untrusted |
| DELETE | `/users/:id/untrust` | Yes | Remove untrust |

### Frontend Features

- Profile page (own and public view)
- Profile editing: display name, bio, avatar upload, country
- Username change with availability check
- Public profile with trading stats, ratings, verification badge
- Top traders leaderboard
- User search by display name
- Block/unblock users (blocked users can't initiate trades)
- Trust/untrust users
- User reporting with reason
- Rating display (stars + comments)
- Last active indicator
- Member-since date

---

## 3. Marketplace — Offers

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/offers` | Public | List all active offers (paginated, filterable) |
| GET | `/offers/buy` | Public | List buy offers only |
| GET | `/offers/sell` | Public | List sell offers only |
| GET | `/offers/user/:userId` | Public | List offers by a specific user |
| GET | `/offers/:id` | Public | Get offer details |
| GET | `/offers/:id/versions` | Yes | Get offer version history |
| POST | `/offers` | Yes | Create new offer |
| PUT | `/offers/:id` | Yes | Update offer (creates new version) |
| DELETE | `/offers/:id` | Yes | Soft-delete offer |
| PATCH | `/offers/:id/pause` | Yes | Pause offer |
| PATCH | `/offers/:id/resume` | Yes | Resume offer |
| PATCH | `/offers/:id/close` | Yes | Close offer |

### Frontend Features

- Marketplace browse with grid/list view toggle
- Buy offers page, sell offers page
- Offer filtering: cryptocurrency, fiat currency, payment method, price range, status
- Offer sorting: price, rating, trade count, date
- Offer pagination
- Offer detail page with creator info, success rate, rating
- Create offer form: type (buy/sell), crypto, fiat, price, min/max limits, payment methods, description, escrow toggle, network selection
- Pricing modes: fixed price, market rate + premium percentage
- Edit offer (creates versioned snapshot)
- Pause/resume/close/delete offer
- Offer status indicators (active, paused, closed)
- Escrow badge on offers requiring escrow
- BTC lock info display for sell offers
- Real-time offer updates via WebSocket

---

## 4. Trading System

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/trades` | Yes | Create trade from offer |
| GET | `/trades` | Yes | List user's trades (filterable) |
| GET | `/trades/active` | Yes | List active trades |
| GET | `/trades/completed` | Yes | List completed trades |
| GET | `/trades/:id` | Yes | Get trade details |
| PATCH | `/trades/:id/status` | Yes | Update trade status |
| POST | `/trades/:id/cancel` | Yes | Cancel trade |
| POST | `/trades/:id/complete` | Yes | Complete trade |
| GET | `/trades/:id/events` | Yes | Get audit trail events |
| GET | `/trades/:id/ledger` | Yes | Get ledger entries for trade |
| POST | `/trades/:id/messages` | Yes | Send chat message |
| GET | `/trades/:id/messages` | Yes | Get chat messages |
| POST | `/trades/:id/attachments` | Yes | Upload single attachment |
| POST | `/trades/:id/attachments/multiple` | Yes | Upload multiple attachments |
| POST | `/trades/:id/rating` | Yes | Submit rating |

### Trade State Machine

```
created → escrow_funded → paid → released → (completed)
                              ↘ disputed → resolved
created → cancelled
```

### Frontend Features

- Create trade from offer with amount selection
- Trade detail page with status stepper
- Real-time trade chat with text + image + file attachments
- Attachment upload with progress bar
- Typing indicators in chat
- Message read receipts
- Payment proof upload
- Trade status updates: mark as paid, confirm release
- Trade cancellation with reason
- Trade completion confirmation
- Trade rating (1-5 stars + comment) after completion
- Trade history with filtering (status, role, date)
- Trade audit trail (event log)
- Active trades list
- Completed trades list
- Pricing snapshot captured at trade creation (immutable)

---

## 5. Escrow System

### Backend API Endpoints

**Custodial Escrow (per-trade):**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/escrow/custodial/create` | Yes | Create custodial escrow for trade |
| GET | `/escrow/trade/:tradeId` | Yes | Get escrow by trade |
| GET | `/escrow/trade/:tradeId/wallet` | Yes | Get escrow wallet |
| GET | `/escrow/trade/:tradeId/balance` | Yes | Check escrow balance |
| POST | `/escrow/trade/:tradeId/confirm-deposit` | Yes | Confirm deposit |
| GET | `/escrow/wallet/:id` | Yes | Get escrow wallet by ID |

**Offer Escrow (pre-funded for sell offers):**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/escrow/offer/create` | Yes | Create escrow for sell offer |
| GET | `/escrow/offer/:offerId` | Yes | Get offer escrow |
| GET | `/escrow/offer/:offerId/balance` | Yes | Check offer escrow balance |
| POST | `/escrow/offer/:offerId/confirm-deposit` | Yes | Confirm offer escrow deposit |

**Legacy & Actions:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/escrow` | Yes | List all escrows |
| GET | `/escrow/active` | Yes | List active escrows |
| GET | `/escrow/stats` | Yes | Get escrow statistics |
| GET | `/escrow/:id` | Yes | Get escrow by ID |
| POST | `/escrow/:tradeId/create` | Yes | Create escrow (legacy) |
| POST | `/escrow/:tradeId/link` | Yes | Link on-chain escrow |
| POST | `/escrow/:id/release` | Yes | Release funds to buyer |
| POST | `/escrow/:id/dispute` | Yes | Open dispute |
| POST | `/escrow/:id/refund` | Yes | Refund to seller |
| POST | `/escrow/:id/resolve-to-seller` | Mod | Resolve dispute → seller |
| POST | `/escrow/:id/resolve-to-buyer` | Mod | Resolve dispute → buyer |
| POST | `/escrow/:id/sync` | Yes | Sync from blockchain |

### Escrow Types

1. **Custodial (Platform-managed)** — Platform generates a temporary wallet, seller deposits crypto, platform releases to buyer on completion
2. **On-chain (Smart Contract)** — Ethereum smart contract holds funds, released by contract logic
3. **Offer Escrow (Pre-funded)** — Seller pre-funds escrow when creating a sell offer
4. **BTC Wallet Lock** — Locks BTC in seller's custodial wallet (no separate escrow address)

### Frontend Features

- Escrow creation flow with network selection
- Escrow wallet address display with QR code
- Escrow funding instructions
- Escrow balance checking with auto-refresh
- Deposit confirmation
- Escrow progress stepper (pending → held → released)
- Escrow statistics dashboard
- Active escrows list
- Dispute initiation with reason
- Escrow release/refund actions

---

## 6. Wallet System

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/wallet` | Yes | Get wallet overview |
| GET | `/wallet/balance` | Yes | Get balances |
| GET | `/wallet/transactions` | Yes | Transaction history |
| GET | `/wallet/pending` | Yes | Pending transactions |
| GET | `/wallet/transfers` | Yes | Internal transfers |
| GET | `/wallet/deposit-address` | Yes | Get all deposit addresses |
| GET | `/wallet/deposit-address/:crypto` | Yes | Deposit address for crypto |
| GET | `/wallet/deposit-address/:crypto/:network` | Yes | Deposit address for crypto/network |
| GET | `/wallet/deposit-addresses` | Yes | All deposit addresses |
| GET | `/wallet/locks` | Yes | Locked balances |
| GET | `/wallet/locks/:offerId` | Yes | Locked balance for offer |
| GET | `/wallet/locks/estimate-fee` | Yes | Estimate unlock fee |
| GET | `/wallet/available-balance` | Yes | Available BTC balance |
| POST | `/wallet/locks/check` | Yes | Check if funds can be locked |
| POST | `/wallet/locks/lock` | Yes | Lock funds for offer |
| POST | `/wallet/locks/unlock` | Yes | Unlock funds |
| POST | `/wallet/deposit` | Yes | Record deposit |
| POST | `/wallet/withdraw` | Yes | Withdraw to external address |
| POST | `/wallet/transfer` | Yes | Internal user-to-user transfer |

### Frontend Features

- Multi-currency wallet overview (BTC, ETH, SOL, TRX, USDT)
- Balance display with USD equivalent
- Locked vs available balance
- Deposit flow: select currency → get address → wait for confirmation
- Withdraw flow: select currency → enter address + amount → confirm
- Internal transfer: select recipient (ID or email) → amount → confirm
- Transaction history with filtering (type, currency, status)
- Pending transactions list
- Transaction detail view with explorer links
- Currency-specific wallet management pages
- Network selection for multi-network tokens (USDT)
- Fee estimation before withdrawal

---

## 7. Custodial Multi-Chain Wallets

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/custodial-wallet/generate` | Yes | Generate HD wallet (all chains) |
| GET | `/custodial-wallet` | Yes | Get wallet info |
| GET | `/custodial-wallet/all` | Yes | Get all chain wallets |
| GET | `/custodial-wallet/chains` | Yes | Get all chain addresses |
| GET | `/custodial-wallet/balance` | Yes | Balance for specific chain |
| GET | `/custodial-wallet/balance/all` | Yes | Balances for all chains |
| GET | `/custodial-wallet/deposit-address` | Yes | Deposit address for chain |
| GET | `/custodial-wallet/history` | Yes | All transaction history |
| GET | `/custodial-wallet/history/:network` | Yes | History for specific network |
| POST | `/custodial-wallet/send` | Yes | Send funds |
| POST | `/custodial-wallet/export` | Yes | Export mnemonic (password required) |
| GET | `/custodial-wallet/networks` | Public | Supported networks |
| GET | `/custodial-wallet/tokens` | Public | Tokens for network |

### Supported Networks

| Network | Native Token | Tokens | Address Format |
|---------|-------------|--------|----------------|
| Bitcoin Mainnet | BTC | — | Native SegWit (bc1...) |
| Ethereum Mainnet | ETH | ERC-20 | 0x... |
| Solana Mainnet | SOL | SPL (USDT) | Base58 |
| Tron Mainnet | TRX | TRC-20 (USDT) | T... |

### Frontend Features

- Wallet generation on signup (automatic)
- Multi-chain wallet overview with all addresses
- Per-network balance display (native + tokens)
- Deposit address display with QR code and copy button
- Send form: select network → token → recipient → amount → confirm
- Transaction history per network
- Mnemonic export (requires password re-entry)
- Network/token info display
- Deposit warnings per network

---

## 8. Gas & Treasury Management

### Backend API Endpoints

**User-Facing:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/gas/estimate` | Yes | Quick gas estimate |
| POST | `/gas/estimate/live` | Yes | Full simulation estimate |
| GET | `/gas/estimate/breakdown` | Yes | Detailed fee breakdown |
| GET | `/gas/estimate/:type` | Yes | Estimate by transaction type |
| GET | `/gas/chain-params` | Yes | Energy/bandwidth prices |
| GET | `/gas/price` | Yes | TRX/USDT price |
| POST | `/gas/validate` | Yes | Validate user can pay gas |
| GET | `/gas/withdrawal-fee` | Yes | Gas fee for USDT withdrawal |
| GET | `/gas/system-status` | Yes | System ready for withdrawals |
| POST | `/gas/withdraw/estimate` | Yes | Estimate sponsored withdrawal |
| POST | `/gas/withdraw/sponsored` | Yes | Execute treasury-sponsored withdrawal |

**Admin Treasury:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/gas/treasury/status` | Admin | Treasury status |
| GET | `/gas/treasury/balances` | Admin | Treasury balances |
| POST | `/gas/treasury/swap` | Admin | Manual USDT → TRX swap |
| POST | `/gas/treasury/check-swap` | Admin | Check thresholds + trigger auto-swap |
| POST | `/gas/treasury/force-swap` | Admin | Force swap (testing) |
| GET | `/gas/treasury/history` | Admin | Operation history |

### Features

- Gas fee estimation across all chains
- Tron energy/bandwidth price calculation
- Treasury-sponsored USDT withdrawals (platform pays TRX gas)
- Automatic USDT → TRX swapping when treasury TRX is low
- Per-user daily gas usage tracking and limits
- Admin treasury monitoring and manual intervention

---

## 9. Price Feeds & Conversion

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/prices` | Public | All crypto prices (USD) |
| GET | `/prices/:coinId` | Public | Single price |
| POST | `/prices/convert/to-usd` | Public | Crypto → USD conversion |
| POST | `/prices/convert/from-usd` | Public | USD → crypto conversion |

### Features

- CoinGecko API integration for real-time prices
- Supported: BTC, ETH, SOL, USDT, USDC, TRX
- Redis caching with 5-minute TTL
- In-memory fallback cache
- Real-time price broadcast via WebSocket
- Frontend price display on marketplace and wallet pages

---

## 10. Payment Methods

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/payment-methods` | Yes | List user's methods |
| POST | `/payment-methods` | Yes | Add method |
| PUT | `/payment-methods/:id` | Yes | Update method |
| DELETE | `/payment-methods/:id` | Yes | Remove method |

### Frontend Features

- Add/edit/delete payment methods
- Payment method selection when creating offers
- Display on offer detail pages

---

## 11. KYC & Verification

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/kyc/tiers` | Public | Available KYC tiers |
| GET | `/kyc/limits/:level` | Public | Limits for KYC level |
| GET | `/kyc/status` | Yes | User's KYC status |
| POST | `/kyc/submit` | Yes | Submit for review |
| POST | `/kyc/documents/upload` | Yes | Upload document (encrypted) |
| GET | `/kyc/documents` | Yes | List documents |
| GET | `/kyc/documents/:id` | Yes | Document metadata |
| GET | `/kyc/documents/:id/download` | Yes | Download + decrypt document |
| DELETE | `/kyc/documents/:id` | Yes | Delete pending document |

### Document Types

- Government ID: passport, driver's license, national ID
- Selfie (photo holding ID)
- Proof of address

### Frontend Features

- KYC settings tab with tier display
- Document upload with drag-and-drop
- File validation (JPEG, PNG, PDF; max 10MB)
- Document status tracking (pending, approved, rejected, expired)
- Rejection reason display
- Progress indicator (3 required documents)
- KYC tier limits display (withdrawal limits, trade limits)
- Submit for verification button

---

## 12. Affiliate & Referral Program

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/affiliate/stats` | Yes | Dashboard statistics |
| GET | `/affiliate/referrals` | Yes | List referrals |
| GET | `/affiliate/earnings` | Yes | Earnings history |
| POST | `/affiliate/generate-link` | Yes | Generate referral link |
| GET | `/affiliate/tiers` | Yes | Available tiers |
| GET | `/affiliate/payouts` | Yes | Payout history |
| POST | `/affiliate/request-payout` | Yes | Request payout |

### Frontend Features

- Affiliate dashboard with stats (referrals, earnings, tier)
- Referral code display + copy
- Referral link generator (with campaign parameter support)
- Referrals list with status
- Earnings tracking: total, pending, paid
- Payout request flow
- Payout history
- Tier system: Bronze → Silver → Gold → Platinum → Diamond
- Tier progress bar with benefits display
- Commission rate display per tier
- Real-time affiliate events via WebSocket

---

## 13. Reseller Program

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/reseller/stats` | Yes | Reseller dashboard stats |
| GET | `/reseller/trades` | Yes | Reseller trade history |
| POST | `/reseller/resell/:offerId` | Yes | Create resell offer |
| GET | `/reseller/active` | Yes | Active resell offers |
| POST | `/reseller/buy/:resellOfferId` | Yes | Buy from resell offer |

### Frontend Features

- Reseller dashboard with profit tracking
- Create resell offer from existing marketplace offer
- Markup configuration (percentage or fixed amount)
- Active resells list
- Resell marketplace browse
- Resell calculation breakdown (original price + markup)
- Resell trade history

---

## 14. WhatsApp Escrow Links

### Frontend Features

- WhatsApp escrow dashboard
- Create escrow link (amount, currency, expiration)
- QR code generation for sharing
- Link sharing
- Link statistics (views, clicks, completions)
- Link cancellation
- Link list view with status

---

## 15. Notifications

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/notifications` | Yes | List notifications (paginated) |
| POST | `/notifications/:id/read` | Yes | Mark as read |
| DELETE | `/notifications/:id` | Yes | Delete notification |
| POST | `/notifications/read-all` | Yes | Mark all as read |

### Notification Types

- Trade: new trade, status change, message received
- Escrow: funded, released, disputed, refunded
- Offer: new offer, status change
- Wallet: deposit confirmed, withdrawal completed
- Affiliate: new referral, earning, tier upgrade
- System: announcements, maintenance
- KYC: status change, document reviewed

### Frontend Features

- Notification center page
- Header notification bell with unread count badge
- Real-time notification delivery via WebSocket
- Mark as read (individual + all)
- Notification filtering by type
- Notification pagination

---

## 16. Real-Time (WebSocket)

### Server Events (→ Client)

| Event | Description |
|-------|-------------|
| `offer:update` | Offer modified |
| `offer:new` | New offer created |
| `offer:removed` | Offer removed |
| `trade:update` | Trade status changed |
| `trade:message` | New trade chat message |
| `trade:typing` | Typing indicator |
| `notification:new` | New notification |
| `notification:unread_count` | Unread count update |
| `price:update` | Cryptocurrency price update |
| `affiliate:earning` | New affiliate earning |
| `affiliate:referral` | New referral |
| `affiliate:payout` | Payout status update |
| `affiliate:tier_upgrade` | Tier upgraded |

### Client Events (→ Server)

| Event | Description |
|-------|-------------|
| `join:offers` / `leave:offers` | Subscribe/unsubscribe to offer updates |
| `join:trade` / `leave:trade` | Join/leave trade chat room |
| `join:notifications` | Subscribe to notifications |
| `join:prices` / `leave:prices` | Subscribe to price updates |
| `join:affiliate` | Subscribe to affiliate events |
| `trade:send_message` | Send chat message |
| `trade:typing` | Send typing indicator |
| `presence:online` / `presence:away` | Presence updates |

### Features

- Firebase token-authenticated connections
- Room-based messaging
- Redis-backed connection tracking (in-memory fallback)
- User presence/online status
- Auto-reconnect on disconnect

---

## 17. Support & Help

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/support/ticket` | Yes | Create support ticket |
| GET | `/support/tickets` | Yes | List user's tickets |
| GET | `/support/tickets/:id` | Yes | Get ticket details |
| POST | `/support/tickets/:id/message` | Yes | Add message to ticket |
| POST | `/support/tickets/:id/close` | Yes | Close ticket |

### Frontend Features

- Create support ticket with category and priority
- Categories: account, trade, payment, security, verification, general, bug, feature, other
- Priority: low, medium, high, urgent
- Ticket list with status
- Ticket detail with message thread
- Attachment support in messages
- Ticket closing

### Static Help Pages

- FAQ page
- Help center
- Security tips guide
- Trading guide
- How escrow works guide
- Contact form

---

## 18. Bug Reporting

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/bug-reports` | Yes | Submit bug report |
| GET | `/bug-reports` | Yes | List user's reports |
| GET | `/bug-reports/:id` | Yes | Get report details |

### Frontend Features

- Bug report form with description
- Device info auto-capture
- Screenshot/video upload with progress
- Discord webhook integration for team notification
- Report list view

---

## 19. Newsletter

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/newsletter/subscribe` | Public | Subscribe email |
| POST | `/newsletter/unsubscribe` | Public | Unsubscribe email |

### Frontend Features

- Newsletter signup form on landing page
- Email capture with validation

---

## 20. Moderator Tools

### Backend API Endpoints

**Reports:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/mod/reports` | Mod | List all reports |
| GET | `/mod/reports/stats` | Mod | Report statistics |
| POST | `/mod/reports/:id/action` | Mod | Take action on report |
| POST | `/mod/reports/:id/assign` | Mod | Assign report |
| POST | `/mod/reports/:id/dismiss` | Mod | Dismiss report |
| POST | `/mod/reports/:id/claim` | Mod | Claim report |

**Disputes:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/mod/disputes` | Mod | List all disputes |
| GET | `/mod/disputes/:id` | Mod | Dispute details |
| GET | `/mod/disputes/:id/messages` | Mod | Dispute messages |
| POST | `/mod/disputes/:id/messages` | Mod | Send moderator message |
| GET | `/mod/disputes/:id/evidence` | Mod | Dispute evidence |
| POST | `/mod/disputes/:id/comment` | Mod | Add internal comment |
| POST | `/mod/disputes/:id/notes` | Mod | Add internal notes |
| POST | `/mod/disputes/:id/assign` | Mod | Assign to moderator |
| POST | `/mod/disputes/:id/resolve` | Mod | Resolve dispute |
| POST | `/mod/disputes/:id/escalate` | Mod | Escalate dispute |
| POST | `/mod/disputes/:id/claim` | Mod | Claim dispute |
| POST | `/mod/disputes/:id/evidence/upload` | Mod | Upload evidence (integrity hashed) |
| GET | `/mod/disputes/:id/evidence/verify` | Mod | Verify evidence chain integrity |
| GET | `/mod/disputes/:id/audit` | Mod | Dispute audit trail |

**User Moderation:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/mod/users/:id/review` | Mod | User review data |
| GET | `/mod/users/:id/history` | Mod | Moderation history |
| POST | `/mod/users/:id/warn` | Mod | Warn user |
| POST | `/mod/users/:id/suspend` | Mod | Suspend user |
| POST | `/mod/users/:id/ban` | Mod | Ban user |
| POST | `/mod/users/:id/unban` | Mod | Unban user |
| POST | `/mod/users/:id/lift-suspension` | Mod | Lift suspension |

**Escrow Actions:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/mod/escrow/:id/release` | Mod | Release escrow to buyer |
| POST | `/mod/escrow/:id/refund` | Mod | Refund escrow to seller |

**Stats & Logs:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/mod/stats/dashboard` | Mod | Dashboard overview |
| GET | `/mod/stats/moderator` | Mod | Moderator performance |
| GET | `/mod/stats/platform-health` | Mod | Platform health |
| GET | `/mod/audit` | Mod | Global audit logs |
| GET | `/mod/logs` | Mod | Activity logs |

### Frontend Features

- Moderator dashboard with stats overview
- Priority queue for urgent items
- Dispute management: list, detail, resolution (buyer/seller), escalation
- Dispute evidence viewing and upload with integrity verification
- Report management: list, detail, action, dismiss, assign
- User management: search, detail, warn, suspend, ban, unban
- User moderation history
- Audit log viewer
- Treasury recovery tools
- Keyboard shortcuts for quick actions

---

## 21. Admin Tools

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/admin/dashboard` | Admin | System analytics |
| POST | `/admin/resolve` | Admin | Manually resolve escrow |
| GET | `/admin/logs` | Admin | Admin activity logs |
| GET | `/admin/treasury/balance` | Admin | Treasury balance |
| GET | `/admin/treasury/health` | Admin | Treasury health check |
| GET | `/admin/treasury/transactions` | Admin | Treasury transactions |
| GET | `/admin/treasury/user-eligibility/:id` | Admin | User eligibility |
| GET | `/admin/diagnostics` | Admin | System diagnostics |
| POST | `/admin/diagnostics/test-encryption` | Admin | Test encryption |
| POST | `/admin/diagnostics/test-solana-rpc` | Admin | Test Solana RPC |

---

## 22. Dashboard & Analytics

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/dashboard` | Yes | User dashboard data |
| GET | `/dashboard/summary` | Yes | Trading summary |

### Frontend Features

- Main dashboard with trading overview
- Total trades, completed trades, active offers
- Profit display (value + currency)
- Escrow balance
- Trading volume chart
- Affiliate stats summary
- Quick action buttons (create offer, view trades)

---

## 23. Settings & Preferences

### Frontend Features

**General Tab:**
- Display name, bio, avatar
- Username change
- Country selection

**Security Tab:**
- 2FA setup/disable
- Password change
- Active sessions management
- Account deletion

**Verification Tab:**
- KYC document upload
- KYC status display
- Tier limits display

**Notification Preferences:**
- Trade update notifications toggle
- Escrow release notifications toggle
- Security alert notifications toggle
- Marketing notifications toggle

**Trading Preferences:**
- Preferred fiat currency
- Daily/monthly trading limits
- Default payment methods

**Display Preferences:**
- Dark/light theme toggle
- Language selection

---

## 24. Blockchain Integrations

| Chain | Native | Tokens | Operations |
|-------|--------|--------|------------|
| Bitcoin | BTC | — | Send, receive, balance, UTXO management, fee estimation |
| Ethereum | ETH | ERC-20 | Send, receive, balance, gas estimation, smart contract escrow |
| Solana | SOL | SPL (USDT) | Send, receive, balance, token accounts |
| Tron | TRX | TRC-20 (USDT) | Send, receive, balance, energy/bandwidth, sponsored withdrawals |

### Common Operations

- HD wallet generation from mnemonic (BIP-39/BIP-44)
- Address derivation per chain
- Balance querying (native + tokens)
- Transaction signing and broadcasting
- Transaction history
- Fee/gas estimation
- Platform fee (1%) on outgoing transactions
- Encrypted mnemonic/private key storage
- Mnemonic export with password verification

---

## 25. Frontend UI/UX

### Theme & Appearance
- Dark/light mode with system detection
- Theme persistence across sessions

### Navigation
- Main navbar with logo, marketplace, wallet, notifications, profile
- Mobile hamburger menu
- User dropdown with profile, settings, dashboard, logout
- Breadcrumbs on inner pages

### UI Components
- Loading skeletons for all data-heavy pages
- Toast notifications (success, error, warning, info)
- Modals/dialogs for confirmations
- Tooltips on hover
- Copy-to-clipboard buttons
- QR code display for addresses
- Star rating inputs
- File upload with drag-and-drop
- Responsive tables with mobile cards
- Charts (Recharts) for volume/price data
- Empty state illustrations
- Error boundaries with fallback UI

### State Management
- Redux for auth, UI, offers, prices
- Zustand for domain stores (wallet, trade, notification, moderation)
- React Query for API data caching, refetching, optimistic updates

---

## 26. Platform Configuration

### Backend API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/config/cryptos` | Public | Supported cryptocurrencies |
| GET | `/config/cryptos/:symbol` | Public | Crypto config |
| GET | `/config/cryptos/:symbol/networks` | Public | Networks for crypto |
| GET | `/config/networks` | Public | Supported networks |

---

## 27. System & Infrastructure

### Health Checks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | Public | Basic liveness check |
| GET | `/health/detailed` | Public | Detailed health (Redis, WS, memory) |
| GET | `/api/v1/health` | Yes | Authenticated health check |
| GET | `/api/v1/version` | Public | API version |

### Background Jobs

- **Auto-swap monitor** — periodic USDT → TRX treasury conversion
- **Dispute deadline monitor** — checks every 5 minutes, auto-escalates expired disputes, locks evidence

### Middleware Stack

- Helmet (HTTP security headers)
- CORS
- Compression (gzip)
- Morgan (request logging)
- Express JSON parser (10MB limit)
- Request ID generation
- Firebase JWT verification
- Role-based access control
- Zod schema validation
- Multer file upload handling
- Global error handler

---

## 28. Reputation System

### Backend Service (`src/services/reputation/`)

**Reputation Score: 0–100**, computed from weighted components:

| Component | Weight | Calculation |
|-----------|--------|-------------|
| Completed trades | 40% | Linear scale 0 → cap (100 trades), then flat |
| Average rating | 30% | Average of 1–5 star ratings (zero ratings = zero component) |
| Success rate | 20% | Completed / (total − cancelled) |
| Account age | 10% | Linear scale 0 → 365 days, then flat |

**Penalties:**
- Dispute losses: −5 points each
- Excessive cancellation rate (above 30% threshold): scaled penalty

**Triggers:**
- Recalculated on trade completion, trade cancellation, new rating, dispute resolution
- Updated on user profile as `reputation` field

### Backend Files

- `calculateReputation.ts` — Pure function, deterministic, no DB access
- `updateUserReputation.ts` — Gathers metrics from Firestore and writes updated score
- `types/reputation.ts` — `ReputationInput`, `ReputationResult`, weights, caps

---

## 29. Append-Only Financial Ledger

### Backend Service (`src/services/ledger/`)

Every financial movement is recorded as an immutable ledger entry. This is the single source of truth for balances.

### Ledger Entry Structure

| Field | Type | Description |
|-------|------|-------------|
| `userId` | string | Account holder |
| `entryType` | enum | `trade_credit`, `trade_debit`, `escrow_lock`, `escrow_release`, `withdrawal`, `deposit`, `fee`, `refund` |
| `direction` | enum | `credit` or `debit` |
| `amount` | number | Always positive |
| `currency` | string | BTC, USDT, ETH, SOL, etc. |
| `network` | string | bitcoinMainnet, tronMainnet, etc. |
| `tradeId` | string? | Associated trade |
| `offerId` | string? | Associated offer |
| `escrowId` | string? | Associated escrow |
| `txHash` | string? | Blockchain transaction hash |
| `description` | string | Human-readable description |
| `metadata` | object? | Additional context |

### Operations

- `createEntry(params)` — Append a new ledger entry (never update or delete)
- `getBalance(userId, currency)` — Compute balance from creditTotal − debitTotal
- `getBalanceHistory(userId, currency)` — Timeline of balance changes
- Trade endpoint: `GET /trades/:id/ledger` — Ledger entries for a specific trade

### Integrity

- Entries are **append-only** — no updates, no deletes
- Balance is always derived from the sum of entries, never stored as a mutable field
- Used for audit trail and dispute resolution evidence

---

## 30. Offer Event Tracking

### Backend Service (`src/services/offerEvent/`)

Every offer state change is recorded as an immutable event.

### Event Types

| Event | Description |
|-------|-------------|
| `created` | Offer created |
| `updated` | Offer fields changed (new version created) |
| `paused` | Offer paused by owner |
| `resumed` | Offer resumed by owner |
| `closed` | Offer closed by owner |
| `deleted` | Offer soft-deleted |
| `escrow_funded` | Escrow wallet funded for sell offer |
| `escrow_released` | Escrow funds released |

### Offer Versioning

- Every update creates a new immutable `offer_version` snapshot
- Trades reference the `offerVersionId` at the time of trade creation
- This ensures pricing and terms are locked at trade time

### Backend Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/offers/:id/versions` | Yes | Get offer version history |

---

## 31. Soft Delete & Data Preservation

### Backend Service (`src/services/softDelete/`)

Financial records are never hard-deleted. Soft deletion preserves data integrity for audits and dispute resolution.

**Soft-deleted records:**
- Users — `discardedAt` timestamp set, account deactivated, can be reactivated
- Offers — `isDeleted` flag set, removed from marketplace, version history preserved
- Trades — never deleted (financial records are permanent)
- Escrows — never deleted
- Ledger entries — never deleted (append-only by design)

**Reactivation:**
- Admin endpoint: `POST /auth/reactivate/:userId`
- Restores account access, preserves all historical data

---

## 32. Platform Fees

### Configuration (`src/config/platformFees.ts`)

| Fee | Rate | Applied To |
|-----|------|-----------|
| Transaction fee | 1% | Outgoing cryptocurrency transfers |
| Escrow release fee | 1% | Escrow releases to buyer |
| Withdrawal fee | Variable | Network-specific gas + platform fee |

- Fee configuration is centralized and can be adjusted without code changes
- Fees are calculated before transaction signing and displayed to user for confirmation
- Fee amounts are recorded in ledger entries

---

## 33. Client Applications

> The Qictrader backend serves three client applications. Each consumes the same REST API
> and WebSocket server. The frontend (web) is the primary client; the mobile app and
> Telegram bot provide secondary access.

### 33.1 Mobile App (React Native / Expo)

**Technology:** React Native, Expo Router, Redux + React Query, Socket.IO, Firebase

**Screens (50+):**
- Auth: login, signup, 2FA verification, forgot password
- Tabs: dashboard, wallet, marketplace, trades, profile
- Offers: list, create, detail, fund escrow, resell
- Trades: detail with chat, review/rating
- Wallet: deposit, withdraw, transfer, history, connect external wallet
- Settings: profile, security (2FA, password, sessions), notifications, payment methods
- KYC: status, document upload
- Affiliate: dashboard (stats, referrals, earnings)
- Reseller: dashboard, application form
- Support: tickets, FAQ, guides
- Notifications: notification center
- Price alerts: create, list, manage

**Mobile-Specific Features:**
- Push notifications via Firebase Cloud Messaging (FCM)
- Biometric potential via Expo APIs
- Camera/document picker for KYC uploads
- Secure token storage (`expo-secure-store`)
- Deep linking support
- Platform-specific UI (iOS safe areas, Android back button)
- Haptic feedback (`expo-haptics`)
- External wallet connection (WalletConnect)

**Feature Gap — Price Alerts:**
The mobile app has a price alerts UI and API client (`price-alerts-api.ts`) that calls:
- `GET /price-alerts` — List user's alerts
- `POST /price-alerts` — Create alert
- `DELETE /price-alerts/:id` — Delete alert

**These endpoints do not exist in the current backend.** This is a planned feature that needs backend implementation in the Rust rewrite.

### 33.2 Telegram Bot (Grammy)

**Technology:** Grammy (Telegram Bot framework), TypeScript, Socket.IO client, Redis

**Commands (30+):**

| Category | Commands |
|----------|----------|
| **General** | `/start`, `/help`, `/ping`, `/status`, `/info`, `/stats` |
| **Auth** | `/login`, `/signup`, `/logout`, `/account` |
| **Marketplace** | `/market`, `/myoffers`, `/newoffer` (6-step wizard), `/view_<id>`, `/manage_<id>` |
| **Trading** | `/mytrades`, `/trade_<id>`, trade actions (complete, cancel, message, escrow) |
| **Wallet** | `/wallet`, `/balance`, `/deposit`, `/history` |
| **Profile** | `/profile`, `/editprofile`, `/settings` |
| **Affiliate** | `/affiliate` (stats, referrals, earnings, tiers, payouts) |
| **Reseller** | `/reseller` (dashboard, application) |
| **Security** | `/security` (2FA, password, sessions, activity log) |
| **KYC** | `/kyc` (status, documents, tiers) |
| **Support** | `/support` (create/view tickets, messages) |
| **Prices** | `/prices`, `/convert` |
| **Other** | `/top` (leaderboard), `/payments`, `/newsletter` |

**Conversation Flows:**
- Offer creation wizard (6 steps: type → crypto → fiat → payment methods → pricing → terms)
- Login flow (email → password → 2FA if enabled)
- Signup flow (email → password → display name)
- Trade message flow
- Dispute submission flow
- Profile edit flow
- Support ticket creation flow

**Real-Time Notifications:**
- WebSocket connection to backend via Socket.IO
- Events: trade created/updated/completed/cancelled, escrow held/released/disputed, new messages, wallet deposits/withdrawals
- Redis mapping (`user:{userId}:telegram` → `telegramId`) for notification routing
- Markdown-formatted notification messages

**QR Code Generation:**
- Deposit address QR codes via `qrcode` library

---

## Summary Statistics

| Metric | Count |
|--------|-------|
| Backend API endpoints | **150+** |
| Frontend pages/routes (web) | **45+** |
| Mobile app screens | **50+** |
| Telegram bot commands | **30+** |
| WebSocket events | **20+** |
| Supported blockchains | **4** (Bitcoin, Ethereum, Solana, Tron) |
| Supported cryptocurrencies | **6** (BTC, ETH, SOL, TRX, USDT, USDC) |
| User roles | **4** (user, moderator, admin, super_admin) |
| KYC document types | **3** |
| Notification types | **7** |
| Background jobs | **2** |
| Client applications | **3** (Web, Mobile, Telegram) |

---

## Appendix: Backend Feature Gap — Price Alerts

The mobile app references a price alerts feature that has no backend implementation:

```
GET    /price-alerts          — List user's price alerts
POST   /price-alerts          — Create price alert (crypto, target price, direction)
DELETE /price-alerts/:id      — Delete alert
```

This requires a new Firestore collection (`price_alerts`), a background job to check prices against alert thresholds, and push notification delivery when alerts trigger. **Must be included in the Rust rewrite.**
