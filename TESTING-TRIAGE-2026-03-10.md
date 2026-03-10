# QicTrader2 — Dev Triage (2026-03-10)

> **207 tickets** remaining in `todo/`. Prioritised per the Tester Guide phase structure.
> Work top-to-bottom. Each phase must be stable before moving to the next.

---

## How to Use This Document

1. Work through phases **top to bottom** — Phase 0 is the highest priority
2. Within each phase, tickets are ordered by dependency and criticality
3. Pick a ticket, open the `.md` file in `stories_ignore/todo/`, read the acceptance criteria
4. When complete, move the file from `todo/` to `done/`
5. If blocked, flag it in the Discord dev channel with the ticket ID

---

## Phase 0 — HIGHEST PRIORITY: Core Design Flows (37 tickets)
*These are the foundational money flows. Nothing works without these.*

### Escrow Logic
| # | Ticket | Summary |
|---|--------|---------|
| 1 | ES-001 | Escrow lock — atomic ledger, single DB transaction locks funds instantly |
| 2 | ES-002 | Escrow release — atomic debit seller, credit buyer, collect fee |
| 3 | ES-003 | Escrow refund — cancel/expiry returns locked funds to seller |
| 4 | ES-004 | Dispute → escrow freeze — funds stay locked until moderator resolves |
| 5 | ES-005 | Moderator resolution dispatch — admin release or refund with audit log |
| 6 | ES-006 | Append-only ledger enforcement — no UPDATE/DELETE on ledger table |
| 7 | ES-007 | Platform fee collection — vendor fee (0.7%) + reseller fee (0.5%) |
| 8 | ES-008 | Deposit detection — credit user balance after N confirmations |
| 9 | ES-009 | Withdrawal processing — build, sign, broadcast tx; gas fee charged |

### Marketplace Flows
| # | Ticket | Summary |
|---|--------|---------|
| 10 | MP-001 | Offer listing with visibility rules — only Active offers with sufficient balance |
| 11 | MP-002 | Offer versioning on edit — each edit creates new OfferVersion |
| 12 | MP-003 | Deterministic quote engine — same inputs = same outputs |
| 13 | MP-004 | FX rate display and persistence — frozen on trade, source + timestamp |
| 14 | MP-005 | Marketplace search and filters |
| 15 | MP-006 | Payment detail access control — only visible to trade parties |
| 16 | MP-007 | Real-time trade notifications — Socket.IO room per trade |
| 17 | MP-008 | Seller offer management — create, edit, pause, resume, close |

### Reseller Flow
| # | Ticket | Summary |
|---|--------|---------|
| 18 | RS-001 | Create resell offer — reseller selects vendor offer + markup (0-50%) |
| 19 | RS-002 | Resell offer pricing display — resell price = vendor price × (1 + markup%) |
| 20 | RS-003 | Trade via resell offer — vendor escrow locked; records buyer, reseller, vendor |
| 21 | RS-004 | Reseller settlement split |
| 22 | RS-005 | Reseller commission on dispute |
| 23 | RS-006 | Reseller dashboard stats |
| 24 | RS-007 | Reseller trade history with commission breakdown |
| 25 | RS-008 | Cannot resell own offer — validation rejects reseller_id == vendor_id |
| 26 | RS-009 | Resell offer lifecycle — Active/Paused/Closed; auto-pause if vendor unavailable |
| 27 | RS-010 | Markup cap enforcement — server-side 0-50% validation |

### Trade Flow
| # | Ticket | Summary |
|---|--------|---------|
| 28 | TF-001 | Trade creation with atomic escrow lock |
| 29 | TF-002 | Trade state machine validation — invalid transitions return 400 |
| 30 | TF-003 | Payment marking with proof upload |
| 31 | TF-004 | Seller confirms and completes trade |
| 32 | TF-005 | Cancel within unviewed window (< 10 min AND counterparty hasn't viewed) |
| 33 | TF-006 | Counterparty view tracking — first view sets counterparty_viewed_at |
| 34 | TF-007 | Trade expiry background job — auto-cancels expired trades |
| 35 | TF-008 | Trade chat with real-time delivery |
| 36 | TF-009 | Rating and review system — one rating per party per trade; immutable |
| 37 | TF-010 | Trade event audit trail — every state change logged |

---

## Phase 1 — CRITICAL: Platform Foundation (4 tickets)
*If these don't work, nothing else matters.*

| # | Ticket | Summary |
|---|--------|---------|
| 38 | PLATFORM-001 | Platform config loads |
| 39 | VISITOR-001 | Landing page renders |
| 40 | AUTH-004 | Log in with email and password |
| 41 | AUTH-007 | 2FA verification during login |

---

## Phase 2 — CRITICAL: Registration & Identity (7 tickets)
*New users must be able to sign up and verify.*

| # | Ticket | Summary |
|---|--------|---------|
| 42 | AUTH-001 | Sign up with email and password |
| 43 | AUTH-002 | Sign up with Google OAuth |
| 44 | AUTH-003 | Sign up with Apple OAuth |
| 45 | AUTH-008 | Reset forgotten password |
| 46 | AUTH-009 | Log out |
| 47 | USR-001 | Auto-generated username suggestions |
| 48 | KYC-002 | Submit government ID |
| 49 | KYC-003 | Submit selfie with ID |
| 50 | KYC-004 | Submit proof of address |
| 51 | KYC-006 | Resubmit rejected documents |
| 52 | KYC-007 | KYC required prompt — blocked from trading without KYC |

---

## Phase 3 — CRITICAL: Wallet & Money (10 tickets)
*Users must be able to see and manage their funds.*

| # | Ticket | Summary |
|---|--------|---------|
| 53 | WALLET-001 | View wallet balances |
| 54 | WALLET-003 | Withdraw crypto |
| 55 | WALLET-004 | Internal transfer between wallets |
| 56 | WALLET-005 | View transaction history |
| 57 | WALLET-006 | Estimate withdrawal fees |
| 58 | WALLET-008 | Lock/unlock funds |
| 59 | GAP-001 | Wallet deposit addresses generated correctly |
| 60 | GAP-013 | Wallet transfers work end-to-end |
| 61 | GAP-017 | Locked funds can be unlocked |
| 62 | WLT-001 | Additional blockchain support (BTC, ETH, SOL, TRX, XMR, BNB) |

---

## Phase 4 — CRITICAL: Offers & Marketplace (5 tickets)
*The core product — browsing and creating offers.*

| # | Ticket | Summary |
|---|--------|---------|
| 63 | OFFER-001 | Browse buy offers — list and filter |
| 64 | OFFER-003 | View offer detail page |
| 65 | OFFER-004 | Create a sell offer |
| 66 | OFFER-011 | View offer version history |
| 67 | OFFER-012 | Create a direct offer to a specific user |

---

## Phase 5 — CRITICAL: Trading Flow (8 tickets)
*The money-making path — initiating and completing trades.*

| # | Ticket | Summary |
|---|--------|---------|
| 68 | TRADE-003 | Chat with counterparty during trade |
| 69 | TRADE-006 | Confirm payment received (seller) |
| 70 | TRADE-009 | Leave a review after trade |
| 71 | TRADE-012 | Upload proof of payment |
| 72 | TRADE-014 | Track when counterparty views a trade |
| 73 | TRADE-015 | Cancel trade within unviewed window |
| 74 | TRADE-016 | Trade auto-expiry |
| 75 | MPR-014 | DFD 5 — Buyer marks paid + proof of payment |

---

## Phase 6 — CRITICAL: Escrow & Ledger Integrity (6 tickets)
*Money safety — if this breaks, funds are at risk.*

| # | Ticket | Summary |
|---|--------|---------|
| 76 | ESCROW-001 | Custodial escrow for a trade |
| 77 | ESCROW-003 | BTC wallet lock escrow |
| 78 | LDG-005 | Ethereum nonce race condition fix |
| 79 | LDG-008 | Affiliate commission balance_after accuracy |
| 80 | LDG-009 | TradeDebit ledger entries for seller on trade completion |
| 81 | LDG-010 | Ledger-wallet balance reconciliation |

---

## Phase 7 — HIGH: Bugs (2 tickets)
*Active bugs blocking users — fix before feature work.*

| # | Ticket | Summary |
|---|--------|---------|
| 82 | BUG-001 | Dispute status not visible |
| 83 | BUG-002 | Unable to access trade as seller |

---

## Phase 8 — HIGH: Disputes & Moderation (11 tickets)
*Trust & safety — keeping the platform safe.*

| # | Ticket | Summary |
|---|--------|---------|
| 84 | MOD-001 | View moderator dashboard |
| 85 | MOD-002 | View and filter trade disputes |
| 86 | MOD-003 | Review a dispute in detail |
| 87 | MOD-004 | Resolve a trade dispute |
| 88 | MOD-005 | Escalate a dispute |
| 89 | MOD-006 | Communicate with dispute parties |
| 90 | MOD-007 | Review dispute evidence |
| 91 | MOD-008 | View and manage user reports |
| 92 | MOD-009 | Take action on a report |
| 93 | MOD-010 | Warn a user |
| 94 | MOD-011 | Suspend a user |
| 95 | MOD-012 | Ban a user |
| 96 | MOD-013 | Unban a user |
| 97 | MOD-014 | Review a user's full history |
| 98 | MOD-015 | View moderation audit logs |
| 99 | MOD-016 | Recover funds from disputed escrow |
| 100 | ADMIN-007 | Resolve escrow dispute as admin |
| 101 | GAP-008 | Dispute deadline monitor background job |
| 102 | MPR-009 | Reputation and history persistence |

---

## Phase 9 — HIGH: Profile & User Management (10 tickets)
*Users need to manage their accounts.*

| # | Ticket | Summary |
|---|--------|---------|
| 103 | PROFILE-001 | View my profile |
| 104 | PROFILE-002 | Edit my profile |
| 105 | PROFILE-003 | View another trader's profile |
| 106 | PROFILE-007 | View 2FA backup codes |
| 107 | PROFILE-009 | View connected devices |
| 108 | PROFILE-011 | Block a user |
| 109 | PROFILE-012 | Report a user |
| 110 | PRF-004 | Trader bio field |
| 111 | PRF-005 | Profile picture upload |
| 112 | AUTH-014 | Delete account (soft delete) |

---

## Phase 10 — HIGH: Payment Methods & Pricing (9 tickets)
*Payment and price infrastructure.*

| # | Ticket | Summary |
|---|--------|---------|
| 113 | PAYMENT-001 | Add a payment method |
| 114 | PAYMENT-002 | Manage payment methods (edit/delete) |
| 115 | PRICE-001 | View current crypto prices |
| 116 | PRICE-002 | View historical price data |
| 117 | PRICE-003 | Convert between crypto and fiat |
| 118 | PRICE-004 | Create a price alert |
| 119 | PRICE-005 | View network gas prices |
| 120 | GAP-006 | Atomic fee split (inherited bug fix) |
| 121 | GAP-015 | OAuth sync |

---

## Phase 11 — MEDIUM: Admin Panel (10 tickets)
*Platform management tools.*

| # | Ticket | Summary |
|---|--------|---------|
| 122 | ADMIN-001 | View admin dashboard |
| 123 | ADMIN-002 | Manage all users |
| 124 | ADMIN-003 | Review and approve/reject KYC submissions |
| 125 | ADMIN-004 | View treasury balance and health |
| 126 | ADMIN-005 | Execute treasury transfer |
| 127 | ADMIN-006 | View system diagnostics |
| 128 | ADMIN-008 | Manage platform configuration |
| 129 | ADMIN-009 | View trading volume and activity reports |
| 130 | ADMIN-010 | Manage affiliate tiers |
| 131 | VER-001 | Identity document verification software |
| 132 | VER-002 | Manual verification review team |

---

## Phase 12 — MEDIUM: Support & Help (6 tickets)
*User support channels.*

| # | Ticket | Summary |
|---|--------|---------|
| 133 | SUPPORT-002 | View my support tickets |
| 134 | SUPPORT-003 | Reply to a support ticket |
| 135 | SUPPORT-004 | Close a resolved ticket |
| 136 | SUPPORT-005 | Submit a bug report |
| 137 | SUP-004 | AI-powered live chat bot |
| 138 | SUP-006 | Office locations display |

---

## Phase 13 — MEDIUM: Dashboard, Notifications & WebSocket (10 tickets)
*Real-time data and user notifications.*

| # | Ticket | Summary |
|---|--------|---------|
| 139 | DASHBOARD-001 | View trading dashboard |
| 140 | NOTIFY-001 | Receive trade status notifications |
| 141 | NOTIFY-002 | Receive new message notifications |
| 142 | NOTIFY-003 | Receive wallet balance notifications |
| 143 | NOTIFY-004 | Receive affiliate earning notifications |
| 144 | NOTIFY-005 | View and manage all notifications |
| 145 | NOTIFY-006 | Receive real-time updates via WebSocket |
| 146 | NTF-001 | Simplified notification dropdown |
| 147 | GAP-009 | WebSocket offer/notification/price events |
| 148 | WS-001 | WebSocket / Socket.IO backend–frontend alignment |

---

## Phase 14 — MEDIUM: Affiliate Program (16 tickets)
*Referral and commission system.*

| # | Ticket | Summary |
|---|--------|---------|
| 149 | AFF-001 | Affiliate code input at registration |
| 150 | AFFILIATE-002 | Generate and share referral link |
| 151 | AFFILIATE-004 | View earnings history |
| 152 | AFFILIATE-005 | Request affiliate payout |
| 153 | AFL-001 | Novice affiliate tier (entry level) |
| 154 | AFL-003 | Silver affiliate tier |
| 155 | AFL-004 | Gold affiliate tier |
| 156 | AFL-005 | Diamond affiliate tier |
| 157 | AFL-006 | Affiliate commission calculation from escrow |
| 158 | AFL-007 | Tier progression visualisation |
| 159 | AFL-008 | Tier badges & emblems on profile |
| 160 | AFL-009 | Private tier stats breakdown |
| 161 | AFL-010 | Affiliate leaderboard |
| 162 | AFL-011 | Affiliate dashboard visual overhaul |
| 163 | GAP-012 | Affiliate payout mechanism |
| 164 | LDG-008 | Affiliate commission balance_after accuracy |

---

## Phase 15 — MEDIUM: Reseller (5 tickets)
*Reseller user features.*

| # | Ticket | Summary |
|---|--------|---------|
| 165 | RESELLER-001 | Register as a reseller |
| 166 | RESELLER-002 | View reseller profile and stats |
| 167 | RESELLER-004 | Manage active resell positions |
| 168 | RESELLER-005 | Reseller commission on dispute outcome |
| 169 | RESELLER-006 | Initiate a trade via a resell offer |

---

## Phase 16 — LOW: Visitor/Static Pages (8 tickets)
*Informational pages — low risk, low effort.*

| # | Ticket | Summary |
|---|--------|---------|
| 170 | VISITOR-003 | Read about page |
| 171 | VISITOR-004 | Read FAQs |
| 172 | VISITOR-005 | Read trading guide |
| 173 | VISITOR-007 | Read security tips |
| 174 | VISITOR-008 | Read terms and privacy policy |
| 175 | VISITOR-009 | Contact support before signing up |
| 176 | VISITOR-010 | Learn about the affiliate program |
| 177 | VISITOR-011 | Subscribe to newsletter |

---

## Phase 17 — LOW: Mobile & Polish (7 tickets)
*Mobile experience and UI polish.*

| # | Ticket | Summary |
|---|--------|---------|
| 178 | MOBILE-001 | Use QicTrader mobile app |
| 179 | MOBILE-002 | Quick actions from dashboard |
| 180 | MOBILE-003 | Swipeable wallet crypto cards |
| 181 | MOBILE-004 | Receive push notifications |
| 182 | MOBILE-005 | Connect external wallet |
| 183 | REG-001 | Task bar icon hover effect |
| 184 | REG-002 | Task bar icon sizing & alignment |
| 185 | WLT-004 | Card design variety / colour options |

---

## Phase 18 — LOW: Telegram Bot (17 tickets)
*Telegram integration — full bot feature set.*

| # | Ticket | Summary |
|---|--------|---------|
| 186 | TELEGRAM-001 | Start the QicTrader Telegram bot |
| 187 | TELEGRAM-002 | Browse marketplace via Telegram |
| 188 | TELEGRAM-003 | Manage wallet via Telegram |
| 189 | TELEGRAM-004 | Manage trades via Telegram |
| 190 | TELEGRAM-005 | Create an offer via Telegram |
| 191 | TELEGRAM-006 | Receive real-time notifications via Telegram |
| 192 | TELEGRAM-007 | Check crypto prices via Telegram |
| 193 | TELEGRAM-008 | Manage security settings via Telegram |
| 194 | TELEGRAM-009 | Manage payment methods via Telegram |
| 195 | TELEGRAM-010 | View affiliate dashboard via Telegram |
| 196 | TELEGRAM-011 | Contact support via Telegram |
| 197 | TELEGRAM-012 | View bot help |
| 198 | TELEGRAM-013 | Check bot and system status |
| 199 | TELEGRAM-014 | View trader leaderboard |
| 200 | TELEGRAM-015 | View my trading statistics |
| 201 | TELEGRAM-016 | Check and upload KYC via Telegram |
| 202 | TELEGRAM-017 | Manage newsletter subscription via Telegram |

---

## Phase 19 — LOW: Auth Extensions & WhatsApp (5 tickets)
*Additional auth methods and WhatsApp integration.*

| # | Ticket | Summary |
|---|--------|---------|
| 203 | AUTH-005 | Log in with Google OAuth |
| 204 | AUTH-006 | Log in with Apple OAuth |
| 205 | AUTH-012 | Log in via Telegram bot |
| 206 | AUTH-013 | Sign up via Telegram bot |
| 207 | WHATSAPP-001 | Create WhatsApp escrow link |
| 208 | WHATSAPP-002 | View WhatsApp escrow dashboard |

---

## Summary by Priority

| Priority | Phases | Tickets | Description |
|----------|--------|---------|-------------|
| **HIGHEST** | 0 | 37 | Core design flows (escrow, marketplace, reseller, trade) |
| **CRITICAL** | 1–6 | 40 | Platform foundation, auth, wallet, offers, trading, ledger |
| **HIGH** | 7–10 | 32 | Bugs, disputes, profiles, payments, pricing |
| **MEDIUM** | 11–15 | 47 | Admin, support, notifications, affiliates, reseller |
| **LOW** | 16–19 | 37 | Static pages, mobile, Telegram, WhatsApp, auth extensions |
| | | **207** | |

---

*Generated 2026-03-10. Source: TESTER-GUIDE.md priority structure + Trello "To Do" column (207 cards).*
