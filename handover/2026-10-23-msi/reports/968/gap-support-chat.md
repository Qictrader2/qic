# 968 gap fix: support chat as the v5.2 glass panel

Card 968 (Trello xcgVXhJY). This is the scoped follow-up for the gap in phase-5-accheck.md section 5 item 2: on customer pages the chat still opened as the old dark box. Built locally, gated, committed and pushed. No PR, no merge, no Trello.

## 1. Branch and pins

- Frontend branch `968-support-chat-panel`, commit 662e2c183cff10de0e0e6087354730867523a3ba, pushed to origin. Base origin/redesign/906 at 366dca2f (re-fetched before the commit; unchanged).
- Worktree: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968b-root/frontend, with root DESIGN.md copied from origin/redesign/906.
- 17 files, +1664 / -48.

## 2. What changed

| File | Change |
| --- | --- |
| src/components/support/SupportChatPanel.tsx (new) | The customer panel on a Radix dialog. Below 768 it is a modal glass sheet (`gl gl-fade gl-sheet`, top 112, scrim z 40, sheet z 41, `aria-modal`, `data-qt-overlay="sheet"`). At 768 and up it is a non-modal glass popover (`gl gl-fade gl-popover`, 400 x 600, right 24, bottom 24, z 50, no scrim, `data-qt-nonmodal`). It has the band (handle on the phone, avatar, "QicTrader Support", subtitle, Start again, "Close support chat"), bubbles, options, the hand-off bar, the phone "Rather write it yourself? New ticket" note, and the loading and error bubbles. Focus goes to the title on open and back to the opener on Close or Escape. Motion uses tokens. |
| src/components/support/ChatBot.tsx | New `look` prop. `look="customer"` renders the panel with the same state and handlers. The logic, the API calls (/chatbot/interact, /chatbot/submit) and the default markup are unchanged. Message types moved to chat-types.ts. |
| src/components/support/SupportChatWidget.tsx | Records the opener from the event, publishes open or closed, and passes `look` from `shouldShowCustomerChrome`. The 'open-support-chat' event and the FAB are unchanged. |
| src/components/support/support-chat-state.ts (new) | Shared open state. `openSupportChat(opener)` sends a CustomEvent on the same event name. A plain `new Event("open-support-chat")` (for example from /contact-us) falls back to the focused element. |
| src/components/navigation/AccountHub.tsx | The Chat row passes itself as the opener and carries `aria-expanded`. While the chat is open it uses the board's open look, with the caption "Open". |
| src/components/navigation/JumpPalette.tsx | Passes the Jump button as the opener. On close it does not pull focus back while another layer holds it. |
| src/hooks/use-bottom-bar-state.ts | Elements under `data-qt-nonmodal` do not count as covering. The phone sheet still covers the dock, and the popover leaves the island and capsule live. |
| src/app/globals.css | Two additions. (1) `html[data-bottom-bar="visible"]{scroll-padding-bottom:var(--qt-overlay-bottom)}` under the under-768 media query, so a field reached with Tab no longer sits behind the dock (B12, 200% zoom). (2) `.qt-rm-fade` in the layered customer reduced-motion block: opacity only, `var(--dur-quick)` (150 ms), linear. It beats 970's unlayered 0.01 ms rule and is applied to the sheet, the popover and the scrim. |

Staff, public and excluded pages (for example /contact-us, /help and /admin) keep today's FAB and dark box. A test compares their markup with markup captured from 366dca2f.

## 3. Board match (D1a frame 3, D1b frame 2, D1c frame 2)

Matched, measured in Chrome against the rendered boards: the panel box and glass at 390, 768 and 1440; the scrim on the phone only; the band, handle, avatar, title 16/24 and subtitle 13/18; the icon buttons; body padding and gap; bubbles; options; the hand-off row; focus to the title; Escape and Close returning focus to the row (or to the Jump button from Jump); the dock covered, inert and aria-hidden at 390 and back after close; the island untouched at 768 and 1440; the popover non-modal; /contact-us unchanged.

Evidence: /Users/starbird/Workspace/Code/qic/.scratch/968/evidence/support-chat/ (pair-390-panel.png, pair-390-frame.png, pair-768-frame.png, pair-1440-panel.png, pair-1440-frame.png, measurements.json, measurements-nonmodal.json, proof-index.json). The proof used a local production build on port 3978, Puppeteer with a temporary profile (removed), no sign-in, and a session and chatbot replies seeded by request interception. The server it started is stopped.

Recorded differences and not-drawn behaviour:

1. The subtitle differs between the boards, and I kept each as drawn: 390 says "Guided answers. A person handles any ticket.", while 768 and 1440 say "Guided answers. A person handles tickets."
2. The boards draw the hand-off as Back plus "Create ticket". The ChatBot's existing options are styled into that bar (tertiary plus a primary that fills the remaining width), so the labels come from the service. D1b note 5 calls the drawn copy a proposal. The 390 board draws no hand-off, so the phone uses the same bar.
3. "New ticket" goes to /contact-us, the same place the Jump "New support ticket" action goes. The board uses `#new-ticket`. The chat closes first.
4. Loading, error and the guest form are not drawn. Loading is a `role=status` bubble ("Thinking..." or "Creating your support ticket...") with a spinner that is hidden under reduced motion. An error is the bot's existing line shown as a bubble. The guest form is built with tokens but cannot appear on customer pages, because they require sign-in.
5. The bubble corner (4 px) and the note radius (14 px) are the board's values. They are not in the DESIGN.md radius table.
6. Tapping the phone scrim closes the sheet. The board is silent on this.
7. The panel stays open when you move between customer pages (open on /profile, tap Wallet, still open on /wallet). The board is silent, and the coordinator decided to keep it.
8. D1a note 11 is not built: "Get help" in the trade room opening the sheet with the trade attached.
9. Two fixes came after the proof run: the "New ticket" underline (removed to match the board; hover underlines and the focus ring is `--foreground`) and the reduced-motion fade. The builder checked both in headless Chrome against the built CSS: under reduced motion the sheet, scrim and popover run 0.15 s linear with opacity only, and without it they are unchanged. The side-by-side pairs predate the underline fix.
10. The toast is unchanged. Under reduced motion it already computes opacity 0.15 s linear, because its transition is covered by the existing layered `.gl,.gl-toast,.gdock` rule.

## 4. Tests

- The new and extended tests cover:
  - panel geometry and classes for both forms
  - opening from the Account row and from Jump
  - Escape and Close returning focus, including after a plain event
  - at 390, the dock covered, inert, aria-hidden and `data-state="covered"`, with the panel carrying `data-qt-overlay="sheet"`, and the dock returning after close
  - the CSS for the dock fade
  - the popover not covering the chrome
  - the Account row `aria-expanded`
  - Jump focus
  - the chatbot requests unchanged
  - the default surface matching 366dca2f markup
  - the scroll-padding rule and its selector
  - the reduced-motion utility
  - the link classes
- Files:
  - support-chat-panel.test.tsx (new)
  - support-chat-openers.test.tsx (new)
  - support-chat-default-surface.test.tsx (new, with the fixture)
  - support-chat-placement.test.tsx (extended)
  - dock-scroll-padding-css.test.ts (new)
  - reduced-motion-fade-css.test.ts (new)
  - css-rules.ts (new helper)
- Focused suites (src/__tests__/navigation, src/__tests__/design, src/__tests__/components/support-chat-*): 32 suites, 1097 tests, all passing. I reran this myself on the final tree.
- Full `bun run test` (round 2 tree, before the underline and reduced-motion round): 422 suites, 5855 passed, 7 failed, 1 skipped. The failures are in 5 suites:
  - TradingLimitsSection, ticket-727 war room, merchant-tier-panel-582 and AdminUserDetailTabs: limits, GMV copy, tier and NGN header assertions, none touching this change. The builder confirmed they fail the same way on a clean 366dca2f checkout.
  - ui-fixes.test.ts: cannot run in this root because it reads qictrader-backend-rs.
- A toast-name run on the final tree passes everything except ui-fixes (same reason).

## 5. Gate

`QIC_GATE_ROOT=/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968b-root .claude/scripts/gate.sh --base origin/redesign/906` on the committed tree: GATE PASS (typecheck, lint with no new errors, build). Receipts: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968b-root/.scratch/gate/20261022T110600.

Scans: the source has no new hex or rgb. The only hex in the diff is the captured base markup in the test fixture. The diff adds no new code comments.
