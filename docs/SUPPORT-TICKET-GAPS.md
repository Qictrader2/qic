# Support Ticket & Notification Gaps

## Current Flow

### Support Tickets (authenticated)
`Contact Us form → POST /support/ticket → PostgreSQL (support_tickets + ticket_messages) → NOTHING`

### Guest Contact
`Contact Us form → POST /support/contact → PostgreSQL (guest_contact_requests) → NOTHING`

### Feedback (complaint/compliment/suggestion)
`Feedback form → POST /support/feedback → PostgreSQL (feedback) → NOTHING`

### Bug Reports (authenticated)
`Bug form → POST /bug-reports (Rust) → PostgreSQL → Discord webhook (MISSING URL) → silently swallowed`

### Bug Reports (guest / backend failure)
`Bug form → Firestore (bug_reports) → Discord webhook (MISSING URL) + Ledgerly API (may work)`

### Newsletter Signup
`Newsletter form → POST /api/newsletter/subscribe → Firestore → Discord webhook (MISSING URL)`

## Missing Environment Variables

| Variable | Status | Impact |
|----------|--------|--------|
| `DISCORD_BUG_REPORT_WEBHOOK_URL` | Not set anywhere (not even in `.env.example`) | Bug report Discord notifications silently swallowed |
| `DISCORD_NEWSLETTER_WEBHOOK_URL` | Listed but blank in `.env.development` | Newsletter Discord notifications silently swallowed |
| `NEXT_PUBLIC_COINGECKO_API_KEY` | Not set locally | Falls back to free CoinGecko API (rate-limited) |
| `NEXT_PUBLIC_EXCHANGERATE_API_KEY` | Not set locally | Falls back to free ExchangeRate-API tier |

## Key Problems

1. **No email provider** — No SendGrid, Resend, Nodemailer, or any email service. The UI tells users "We'll reply to your email" but no outbound email exists.

2. **No staff notification** — Tickets, contacts, and feedback go into PostgreSQL with zero notification. No one knows when a new ticket arrives unless they query the DB or check the mod panel.

3. **Discord webhooks not configured** — Bug reports and newsletter signups try to notify via Discord but all webhook URLs are blank.

4. **Silent failures** — The Discord bug report route returns `200 OK` even when the webhook URL is missing (by design, to not expose config issues to the client), but this means failures are invisible.

5. **Mobile app field mismatches** — Mobile contact form sends `{ name, email, subject, message }` but backend requires `category`. Mobile ticket `addMessage` sends `{ message }` but backend expects `{ content }`.

## TODO

- [ ] Choose and integrate an email provider (SendGrid / Resend)
- [ ] Send email notification to staff on new ticket/contact/feedback
- [ ] Send confirmation email to user on ticket creation
- [ ] Set up `DISCORD_BUG_REPORT_WEBHOOK_URL` in all environments
- [ ] Set up `DISCORD_NEWSLETTER_WEBHOOK_URL` in all environments
- [ ] Add `DISCORD_BUG_REPORT_WEBHOOK_URL` to `.env.example`
- [ ] Fix mobile app payload mismatches (category, content)
- [ ] Add moderator dashboard notification/badge for new tickets
