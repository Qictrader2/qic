# QicTrader Platform — User Stories

**Registration, Profile & Affiliate Fixes**
Derived from Registration Fixes Documents (v1, v2, v3)
Date: 06/02/2026 | Total Stories: 33

---

## Epic 1: Registration Page — Task Bar

### REG-001: Task Bar Icon Hover Effect

**User Story:** As a potential client visiting the registration page, I want task bar icons to show a contrasting light/dark hover effect when I mouse over them, so that I have clear visual feedback indicating which icon I am about to click.

**Acceptance Criteria:**
1. All task bar icons display a contrasting light/dark overlay on hover.
2. Hover effect is smooth (CSS transition) and consistent across all icons.
3. Icons return to their default state when the cursor leaves.
4. Hover effect works across all supported browsers and devices.

---

### REG-002: Task Bar Icon Sizing & Alignment

**User Story:** As a potential client, I want the task bar icons to be centered and appropriately sized, so that the navigation looks professional and is easy to use.

**Acceptance Criteria:**
1. All task bar icons are horizontally centered within their container.
2. Icons are uniformly sized and proportional to the task bar height.
3. Spacing between icons is consistent.
4. Layout is responsive and maintains alignment on mobile, tablet, and desktop.

---

### REG-003: Registration Page Visual Enhancement (Borders/Boxes)

**User Story:** As a potential client, I want the registration page to have bolder and more frequent borders/boxes, so that the page feels fuller and more polished without being overwhelming.

**Acceptance Criteria:**
1. Additional border/box elements are added to the registration page layout.
2. Borders are visually prominent but do not introduce new content or clutter.
3. The overall page does not feel text-heavy or overwhelming.
4. Design is reviewed and approved by UI/UX team.

---

### REG-004: Pre-Registration Support Access

**User Story:** As a potential client who has NOT yet registered, I want the support bar/icon to take me directly to support contacts WITHOUT requiring me to sign up first, so that I can get help with registration issues before creating an account.

**Acceptance Criteria:**
1. The support icon/link on the registration page navigates to the support contact page without requiring authentication.
2. No login or registration gate exists between the registration page and the support page.
3. The support page is fully functional for unauthenticated users.
4. Support tickets submitted by unregistered users are accepted and logged.

---

## Epic 2: Support Window

### SUP-001: Support Window Available Without Registration

**User Story:** As a potential client (registered or unregistered), I want the support window to be accessible at all times, so that I can get assistance regardless of my registration status. Tickets from unregistered users take second priority behind registered users.

**Acceptance Criteria:**
1. Support window is accessible from all pages, including the registration page.
2. Unregistered users can open and submit support tickets.
3. Tickets from unregistered users are flagged as second priority in the support queue.
4. Registered users' tickets remain first priority.

---

### SUP-002: Categorised Support Email Addresses

**User Story:** As a user submitting a support request, I want to choose from distinct email categories (Fraud, General Enquiries, Support), each with a unique email address, so that my enquiry is routed to the correct team immediately.

**Acceptance Criteria:**
1. Support window displays three email options: Fraud, General Enquiries, and Support.
2. Each option has a distinct, dedicated email address.
3. Selecting an option pre-populates the correct recipient address.
4. Email categories are clearly labelled and described.

---

### SUP-003: Complaints, Compliments & Suggestions Option

**User Story:** As a user, I want a dedicated option for submitting complaints, compliments, and suggestions, so that QicTrader can gather more client feedback data to improve the platform.

**Acceptance Criteria:**
1. A Complaints, Compliments & Suggestions option is visible in the support window.
2. Users can select a sub-category (Complaint, Compliment, or Suggestion).
3. Submissions are logged in the backend with the correct category tag.
4. Confirmation message is displayed to the user upon submission.

---

### SUP-004: AI-Powered Live Chat Bot

**User Story:** As a potential client, I want the live chat hyperlink to open an AI chatbot that prompts me with FAQs to narrow down my enquiry, so that a summary of my issue can be generated and provided to the QicTrader agent who receives my ticket.

**Acceptance Criteria:**
1. Clicking the Live Chat link opens an AI chatbot interface.
2. The bot presents relevant FAQ prompts based on user input to narrow the enquiry.
3. Upon completion, the bot generates a structured summary of the enquiry.
4. The summary is attached to the support ticket that is forwarded to a QicTrader agent.
5. The bot is accessible to both registered and unregistered users.

---

### SUP-005: Call Option with Pre-Selection Menu

**User Story:** As a user wanting phone support, I want an option-select feature (IVR-style menu) before being forwarded to a QicTrader agent, so that my call is routed to the correct department.

**Acceptance Criteria:**
1. The call option presents a menu of department/topic selections before connecting.
2. User selection is captured and forwarded to the receiving agent.
3. If no selection is made, a default/general queue is used.
4. Menu options align with the support email categories (Fraud, General, Support, etc.).

---

### SUP-006: Office Locations Display

**User Story:** As a potential client, I want to see QicTrader's office locations within the support section, so that I can verify the company's legitimacy and visit in person if needed.

**Acceptance Criteria:**
1. Office locations (addresses) are displayed in the support window.
2. Locations include at minimum: physical address, city, and country.
3. Information is accessible to both registered and unregistered users.

---

### SUP-007: 5-Level Support Ticket Priority System

**User Story:** As a QicTrader support agent, I want incoming tickets to be auto-classified into 5 priority levels, so that critical issues like fraud are handled first and lower-priority items are queued accordingly.

**Acceptance Criteria:**
1. Priority 1 (Highest): Fraud ticket logs.
2. Priority 2: Technical difficulty tickets.
3. Priority 3: General enquiries and escalated complaints.
4. Priority 4: Standard complaints.
5. Priority 5 (Lowest): Compliments and suggestions.
6. Tickets are auto-assigned a priority based on the category selected by the user.
7. Agents can manually override priority if needed.
8. Priority is visually indicated in the agent's ticket dashboard.

---

## Epic 3: Username Generation & Validation

### USR-001: Auto-Generated Username Suggestions

**User Story:** As a new user registering on QicTrader, I want the system to generate usable username suggestions for me, so that I can quickly pick a valid username without having to guess available options.

**Acceptance Criteria:**
1. During registration, the username field offers at least 3 auto-generated suggestions.
2. Suggestions are unique and not already taken.
3. User can click a suggestion to populate the field, or type their own.
4. Suggestions refresh if the user requests new options.

---

### USR-002: Username Restrictions Info Bubble

**User Story:** As a new user, I want an info bubble next to the username field that explains restrictions (e.g., no profanity, no slurs, character limits), so that I know the rules before submitting.

**Acceptance Criteria:**
1. An info (i) icon is displayed next to the username input field.
2. Clicking/hovering the icon reveals a tooltip listing all username restrictions.
3. Restrictions include: no profanity/slurs, character limits, and allowed/disallowed special characters.
4. Tooltip is dismissible and does not block the input field.

---

### USR-003: Username Validation Warnings (Symbols, Profanity, Duplicates)

**User Story:** As a new user, I want real-time yellow warning bubbles when I enter symbols, profanity, or an already-taken username, so that I can correct my input immediately without submitting an invalid form.

**Acceptance Criteria:**
1. A yellow caution bubble appears when the username contains restricted symbols.
2. A red/yellow warning bubble appears when profanity or slurs are detected.
3. A yellow bubble appears when the entered username is already taken, stating it is unavailable.
4. Warnings appear in real-time as the user types (debounced).
5. Profanity filter covers a comprehensive blocklist including common variations.
6. Duplicate check queries the backend in real-time.

---

## Epic 4: Email Address & Password

### EML-001: AI-Based Email Address Validation

**User Story:** As a new user, I want the system to detect errors or non-existing email addresses and warn me with a yellow bubble, so that I don't register with an invalid email that will prevent me from verifying my account.

**Acceptance Criteria:**
1. On blur or after typing, the system validates the email format.
2. A yellow warning bubble is displayed if the email format is invalid or the domain does not exist.
3. Validation includes DNS/MX record lookup for the email domain where possible.
4. Common typos (e.g., gmial.com) are detected and a suggestion is offered.
5. The user can proceed only with a valid email address.

---

### EML-002: Strong Password Generator

**User Story:** As a new user, I want the system to suggest strong passwords for me (similar to the username generator), so that I can easily create a secure account without manually crafting a complex password.

**Acceptance Criteria:**
1. A "Suggest Password" button or auto-suggestion is available next to the password field.
2. Generated passwords meet all complexity requirements (see EML-003).
3. User can accept the suggestion with one click to populate the password field.
4. Generated password is copyable to clipboard.
5. Multiple suggestions can be requested.

---

### EML-003: Password Complexity Requirements & Hint Bubble

**User Story:** As a new user, I want a visible hint bubble displaying password requirements that remains visible until my password meets all criteria, so that I know exactly what is needed.

**Acceptance Criteria:**
1. Password requirements: minimum 11 characters, at least 1 uppercase letter, at least 1 lowercase letter, at least 1 symbol, at least 1 number.
2. A hint bubble is displayed next to the password field listing all requirements.
3. Each requirement shows a checkmark or changes colour as it is satisfied in real-time.
4. The hint bubble remains visible until all requirements are met and the password is confirmed.
5. The password and confirm-password fields must match before the user can proceed.

---

## Epic 5: Affiliate Code at Registration

### AFF-001: Affiliate Code Input at Registration

**User Story:** As a new user who was referred but did not use an affiliate link, I want an option during registration to either paste the affiliate's link or enter their unique affiliate code manually, so that the referring affiliate is properly credited.

**Acceptance Criteria:**
1. The registration form includes an optional "Affiliate Code" field.
2. The field accepts either a full affiliate link or a short affiliate code.
3. The system validates the code/link against existing affiliates in the database.
4. If valid, the affiliate relationship is linked to the new user's account in perpetuity.
5. If invalid, a warning message is shown and the user can still proceed without it.
6. The affiliate link persists permanently — it affects commission distribution and escrow calculations for users inside and outside the programme.

---

## Epic 6: My Profile — Username & Email Display

### PRF-001: Display Username and Email on Profile

**User Story:** As a registered trader, I want my username (not my first name) and email address to be visible on my profile page, so that I can confirm my identity at a glance.

**Acceptance Criteria:**
1. The profile page header displays the user's username (not first name).
2. The user's email address is displayed below or beside the username.
3. Both are styled small and non-distracting — they do not dominate the page.
4. Username and email are not editable directly from this display (edit via settings).

---

## Epic 7: Notifications Simplification

### NTF-001: Simplified Notification Dropdown

**User Story:** As a trader, I want the profile dropdown to show only "My Profile" and a "Notifications" portal link (not inline notification previews), so that the dropdown is clean and not cluttered with too many notification windows.

**Acceptance Criteria:**
1. Clicking the profile icon opens a dropdown with only: My Profile, Notifications (portal link).
2. The Notifications option navigates to a dedicated notifications page/portal.
3. Inline notification previews are removed from the profile dropdown.
4. The notification bell icon remains in its current position and shows an unread count badge.

---

## Epic 8: Profile Information Enhancements

### PRF-002: Trade Name Display Preferences

**User Story:** As a trader, I want to choose how my name is displayed in trades — full name, initial + surname, or hidden (showing only my trading username) — so that I can control my privacy during P2P transactions.

**Acceptance Criteria:**
1. Profile settings include a "Name Display" option with three choices: Full Name, Initial + Surname, or Hidden (username only).
2. The selected preference is applied in all trade chat windows.
3. Default is "Hidden" (username only) for new accounts.
4. Changes take effect immediately on save.

---

### PRF-003: International Region Code for Phone Numbers

**User Story:** As a trader, I want a region/country code selector (e.g., +27 for South Africa) next to my cell number input, so that my phone number is stored with the correct international format.

**Acceptance Criteria:**
1. The phone number input field is prefixed with a country code dropdown.
2. The dropdown includes all international dialling codes with country flags.
3. Default is auto-detected based on user's location or set to +27 (ZA) as fallback.
4. The full international number (code + number) is stored in the database.

---

### PRF-004: Trader Bio

**User Story:** As a trader, I want to add a bio/description to my profile, so that I can market myself to other traders and they can learn about my trading style and experience.

**Acceptance Criteria:**
1. Profile settings include a "Bio" text area.
2. Bio has a reasonable character limit (e.g., 500 characters).
3. Bio is visible on the trader's public profile when viewed by other traders.
4. Bio supports basic text only (no HTML, links, or images).
5. Profanity filter applies to bio content.

---

### PRF-005: Profile Picture Upload

**User Story:** As a trader, I want to upload a profile picture, so that my profile looks more professional and recognisable compared to showing just my initials.

**Acceptance Criteria:**
1. Profile settings include an option to upload a profile picture.
2. Accepted formats: JPG, PNG. Max file size: 5MB.
3. Image is cropped/resized to a standard avatar dimension.
4. Profile picture replaces the initials avatar wherever the user's avatar is shown.
5. Users without a picture continue to see their initials as the default.

---

## Epic 9: Identity Verification

### VER-001: Identity Document Verification Software

**User Story:** As QicTrader, I want a proper identity confirmation/verification software integrated into the upload flow, so that users cannot upload random images and get falsely verified. The current system accepts any image.

**Acceptance Criteria:**
1. Uploaded ID documents are validated using document verification software (OCR + liveness/authenticity checks).
2. The system rejects obviously invalid uploads (e.g., non-document images, blurry photos, mismatched data).
3. Accepted document types are clearly listed (passport, national ID, driver's licence).
4. Users receive clear feedback on rejection reasons.
5. Fallback: a manual verification queue exists for edge cases.

---

### VER-002: Manual Verification Review Team

**User Story:** As QicTrader, I want a dedicated verification team (or admin panel) to review identity document uploads that cannot be auto-verified, so that no fraudulent accounts slip through.

**Acceptance Criteria:**
1. An admin panel lists all pending verification submissions.
2. Reviewers can approve, reject, or request re-upload with a reason.
3. Users are notified of the outcome via email and in-app notification.
4. SLA: manual reviews are completed within 24–48 hours.
5. Audit trail logs which reviewer handled each submission.

---

## Epic 10: Help & Support (Profile Section)

### HLP-001: Step-by-Step Help Articles / Guides

**User Story:** As a trader, I want the Help & Support section of my profile to contain links to step-by-step guide articles, so that I can self-serve and resolve common issues without contacting support.

**Acceptance Criteria:**
1. The Help & Support section in the profile menu contains a list of help articles.
2. Articles are categorised (e.g., Getting Started, Verification, Trading, Wallet).
3. Each article provides a step-by-step walkthrough with screenshots or visuals.
4. Articles are searchable.
5. Articles are maintained and updated by the content/support team.

---

## Epic 11: Wallet Overview

### WLT-001: Additional Blockchain Support

**User Story:** As a trader, I want more blockchain networks available for crypto transfers, so that I have greater flexibility in how I move funds.

**Acceptance Criteria:**
1. At minimum, add support for Monero (XMR) and BNB (Binance Coin) networks.
2. Each new blockchain has a corresponding wallet address and deposit/withdrawal flow.
3. Blockchain options are selectable during transfer initiation.
4. Network fees are displayed before confirming a transaction.

---

### WLT-002: Solana Wallet Icon

**User Story:** As a trader, I want the Solana wallet option to display its proper icon, so that it is visually consistent with all other crypto wallets that already have icons.

**Acceptance Criteria:**
1. The official Solana (SOL) icon/logo is displayed next to the Solana wallet option.
2. Icon size and style are consistent with all other crypto wallet icons.
3. Icon is visible in the wallet overview and in any wallet selection dropdowns.

---

### WLT-003: Transaction Timestamps (Date + Time)

**User Story:** As a trader, I want wallet transaction records to show both date AND time (not just date), so that I can accurately track when each transaction occurred.

**Acceptance Criteria:**
1. All wallet transaction entries display a full timestamp: date and time.
2. Time is displayed in the user's local timezone.
3. Format is clear and consistent (e.g., 2026-02-06 14:32 SAST).
4. Applies to all transaction types: deposits, withdrawals, transfers.

---

### WLT-004: Card Design Variety / Colour Options

**User Story:** As a trader, I want more colour options and/or different visual designs for my wallet cards, so that the wallet section looks more appealing and personalised.

**Acceptance Criteria:**
1. At least 3–5 different card colour/design themes are available.
2. Users can select their preferred card design in wallet settings.
3. Selected design is applied to the wallet overview card display.
4. Designs are visually distinct and professional.

---

## Epic 12: Affiliate Programme — Tier System & Commissions

### AFL-001: Novice Affiliate Tier (Entry Level)

**User Story:** As a new trader entering the affiliate programme, I want to start at a "Novice" tier, so that I have an entry-level rank from day one with clear requirements to progress.

**Acceptance Criteria:**
1. Novice tier is the default for all new affiliate programme participants.
2. Requirements: $10 trading volume, 0–4 affiliates.
3. Commission: 5% of direct affiliate trading fee (A1 only).
4. No medal/badge is displayed for Novice tier.
5. Tier is visible in the trader's own affiliate dashboard.

---

### AFL-002: Bronze Affiliate Tier

**User Story:** As an affiliate trader who has met the Bronze requirements, I want to be promoted to Bronze tier with increased commission rates, so that I am rewarded for growing my affiliate base.

**Acceptance Criteria:**
1. Requirements: $50 trading volume AND 5–10 affiliates (both conditions required).
2. Commission: 10% of A1 (direct affiliate trading fee) + 4% of A2 (indirect affiliate trading fee).
3. Bronze badge/emblem (bolt/gear icon) is displayed on the trader's profile.
4. Promotion is automatic when both conditions are met.

---

### AFL-003: Silver Affiliate Tier

**User Story:** As an affiliate trader who has met the Silver requirements, I want to be promoted to Silver tier with expanded multi-level commission rates.

**Acceptance Criteria:**
1. Requirements: $8,000 trading volume AND 10–50 affiliates.
2. Commission: 15% A1 + 8% A2 + 3% A3 (third-level indirect affiliates).
3. Silver badge/emblem (silver coins icon) is displayed.
4. Promotion is automatic when both conditions are met.

---

### AFL-004: Gold Affiliate Tier

**User Story:** As an affiliate trader who has met the Gold requirements, I want to be promoted to Gold tier with premium commission rates.

**Acceptance Criteria:**
1. Requirements: $50,000 trading volume AND 50–100 affiliates.
2. Commission: 20% A1 + 10% A2 + 5% A3.
3. Gold badge/emblem (gold ingots icon, with a refined gold colour — less yellow) is displayed.
4. Promotion is automatic when both conditions are met.

---

### AFL-005: Diamond Affiliate Tier

**User Story:** As a top affiliate trader who has met the Diamond requirements, I want to be promoted to Diamond tier with the highest commission rates and additional rewards (gift cards).

**Acceptance Criteria:**
1. Requirements: $300,000 trading volume AND 100–200 affiliates.
2. Commission: 25% A1 + 12% A2 + 7% A3.
3. Diamond badge/emblem (diamond icon) is displayed.
4. Additional reward: gift cards (specifics TBD by business).
5. Promotion is automatic when both conditions are met.

---

### AFL-006: Affiliate Commission Calculation from Escrow

**User Story:** As QicTrader, I want affiliate commissions to be calculated as a percentage of the 1% escrow fee charged on completed trades, so that commissions are only paid on successful transactions and deducted from existing platform revenue.

**Acceptance Criteria:**
1. Escrow fee is 1% of the trade value.
2. Affiliate commission is calculated as the tier's percentage of the escrow fee (not the trade value).
3. Example: 10,000 ZAR trade → 100 ZAR escrow fee → Bronze (10%) = 10 ZAR paid to affiliate.
4. Commission is only calculated and paid after the trade is marked as completed.
5. Commission is split correctly across A1, A2, and A3 levels based on the affiliate chain.
6. Commission supports both ZAR and USDT denominations.

---

## Epic 13: Affiliate Programme — UI, Cosmetics & Leaderboard

### AFL-007: Tier Progression Visualisation

**User Story:** As a trader, I want to see a visual progress bar or tracker showing my current progress toward the next tier (e.g., 2/4 affiliates, $7.64/$10 volume), so that I am motivated to keep growing.

**Acceptance Criteria:**
1. The affiliate dashboard displays progress bars for both affiliate count and trading volume.
2. Current values and target values are shown numerically (e.g., 2/4 affiliates; $7.64/$10).
3. Progress bars fill proportionally.
4. Both conditions (volume AND affiliates) must be met for promotion, and this is clearly communicated.

---

### AFL-008: Tier Badges & Emblems on Profile

**User Story:** As a trader, I want a tier-specific emblem/badge displayed on my profile that other traders can see, so that my affiliate rank is publicly recognised.

**Acceptance Criteria:**
1. Each tier has a unique emblem: Novice = plain medal, Bronze = bolt/gear, Silver = silver coins, Gold = gold ingots, Diamond = diamond.
2. The emblem is displayed on the trader's public profile.
3. Other traders can see the emblem/rank but NOT the detailed stats breakdown.
4. The emblem is also visible in the affiliate dashboard for the trader themselves.

---

### AFL-009: Private Tier Stats Breakdown

**User Story:** As a trader, I want to see my full affiliate stats breakdown (A1, A2, A3 commissions, volume, affiliate count) on my own dashboard, but I want this data hidden from other traders, so that my detailed performance is private.

**Acceptance Criteria:**
1. The trader's own affiliate dashboard shows: current tier, A1/A2/A3 commission earnings, total volume, total affiliates, and progress to next tier.
2. When another trader views this trader's profile, they see ONLY the tier rank/badge.
3. No commission amounts, volume, or affiliate counts are visible to other traders.
4. Privacy is enforced at the API level, not just the UI.

---

### AFL-010: Affiliate Leaderboard

**User Story:** As a trader, I want to see a public affiliate leaderboard ranking all affiliates from #1 downward with their lifetime earnings displayed, so that healthy competition is encouraged.

**Acceptance Criteria:**
1. A leaderboard page/section is accessible from the affiliate programme area.
2. Affiliates are ranked by lifetime earnings (highest to lowest).
3. Each entry shows: rank number, username, tier badge, and lifetime earnings.
4. The leaderboard updates in real-time or near real-time as trades complete.
5. The logged-in trader's own rank is highlighted.

---

### AFL-011: Affiliate Dashboard Visual Overhaul

**User Story:** As a trader, I want the affiliate tier display to look visually engaging with distinct colours, icons, and clear progression between tiers, so that the programme feels rewarding and not boring.

**Acceptance Criteria:**
1. Each tier has a unique colour scheme and emblem (as defined in AFL-008).
2. Visual progression between tiers is evident (e.g., increasing richness of design).
3. The current tier is prominently highlighted.
4. The UI matches the reference screenshots provided in the design documents.
5. Design is approved by the UI/UX team before release.