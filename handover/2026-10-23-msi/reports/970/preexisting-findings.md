# Ticket 970: findings that exist on base (investigated, no code changed)

Base = `b1cbfe6e^` (04fed798). 970 = `b1cbfe6e`. Frontend worktree `.scratch/906-redesign/970-root/frontend`.

## 6. /offer/[id] shows "Price (Market) 0.00 ZAR/USDT" when the market rate is missing

Verdict: same on base. 970 did not touch this path.

- Base `src/app/(offers)/offer/[id]/page.tsx:470-473`:
  `const displayPrice = offer?.pricePerUnit && offer.pricePerUnit > 0 ? offer.pricePerUnit : liveRate || 0` and
  `const isMarketRate = offer?.pricingMode === "market_plus_premium"`.
- Base render at lines 547 (`Price{isMarketRate ? " (Market)" : ""}`) and 550-554 (`formatCurrencyByCode(displayPrice, ...) || "—"`);
  the second use at lines 635-638. Current file has the same code at 473-476, 550-557, 638-641 (base 550-554, 638; shifted by +3 lines only).
- `git diff b1cbfe6e^ b1cbfe6e -- 'src/app/(offers)/offer/[id]/page.tsx'` has three hunks: the lucide import line (AlertCircle removed),
  the new ErrorState import, and the error branch (old lines 451-466 replaced by ErrorState). A grep of that diff for
  `displayPrice|liveRate|formatCurrencyByCode` returns 0 matches.
- Source of the 0: `src/hooks/api/use-offer-detail-page.ts:63` `effectiveRate = currentQuote?.effectiveRate ?? offer?.pricePerUnit ?? 0`,
  exposed as `liveRate` at line 360. A missing quote and missing price gives 0.
- Source of the "0.00": `src/config/currency.config.ts:66-86` `formatCurrencyByCode` returns `(0).toFixed(2)` for 0, so the `|| "—"`
  fallback never fires. Both `currency.config.ts` and the hook are unchanged by 970 (`git diff --stat b1cbfe6e^ b1cbfe6e` on them is empty).
- Fabricated-zero pattern, matches the contract rule "never invent 0 or 0.00". Fixing it is outside this ticket's diff.

## 7a. /notifications crashes (React error #130) on an unknown type such as "security"

Verdict: same on base. 970 did not change the map.

- Base `src/app/(dashboard)/notifications/page.tsx:52-62` and current `:54-64` hold the identical map
  `NotificationIcon: Record<NotificationType, typeof Bell>` with nine keys: trade, message, offer, escrow, affiliate, wallet, payment, system, support.
  `diff` of the two 11-line blocks prints no difference.
- Base `:571` `const Icon = NotificationIcon[notification.type]`, current `:620` same line. For a type outside the map `Icon` is `undefined`,
  and `<Icon .../>` (base `:595`, current `:644`) throws React error #130 (element type is undefined).
- `NotificationType` (`src/types/notification.types.ts:7-16`) has no "security" member; `git diff --stat b1cbfe6e^ b1cbfe6e -- src/types` is empty.
- The 970 diff of this file adds the ErrorState branch, stats statuses and (in this fix round) the Mark All Read gating; none of it reads the map.

## 7b. Signed-in header overflows to 1583px at 1440

Verdict: the header code is unchanged by 970; I could not measure base. What I could establish is source identity only.

- `git diff --stat b1cbfe6e^ b1cbfe6e -- src/components/navigation src/components/common/mainHeader.tsx src/components/ui/user-avatar.tsx` is empty.
  `MAIN_NAV_ROUTES` (`src/config/routes.ts:68-`) is unchanged; the routes.ts diff only appends (`git diff` has no removed lines).
- Header row: `src/components/common/mainHeader.tsx:56` `lg:px-24` (96px per side) and the desktop group at `:61`
  `hidden items-center gap-2 lg:flex` has no `min-w-0`, `flex-wrap` or overflow control. `NavLinks` (`src/components/navigation/NavLinks.tsx`,
  desktop variant) gives every link `px-4 text-sm font-medium` with `gap-1`; for a signed-in user there are 9 links (one with a "Coming Soon" badge),
  then the bell, `HeaderBalance` and `UserProfileDropdown showName`. All of those classes are identical on base.
- The 970 evidence screenshot `evidence/customer/d-notifications-1440-data.png` shows nav labels already wrapping ("My Offers", "Create Offer",
  "Virtual Card", "Loyalty Programme") at 1440, which is the overflow pressure.
- Only shared things 970 changed that this header could inherit: customer `ui/button` classes (the header and nav use plain `<button>` and `<Link>`
  with their own classes, not `ui/button`) and customer-scoped globals. `common/button.tsx` and `common/input.tsx` were deleted by 970 but have no importers.
- Not verified: the 1583px figure itself, on base or on 970. Base signed-in pages were not captured (staging sign-in was out of scope for the proof run).
  A base measurement needs one signed-in capture of `/notifications` at 1440 on staging.qictrader.com (`document.documentElement.scrollWidth`).
