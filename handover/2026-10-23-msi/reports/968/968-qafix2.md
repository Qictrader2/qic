# 968-qafix2: menus over the consent banner, Customise focus return

- PR: https://github.com/Qictrader2/Frontend/pull/343 (base redesign/906, not merged)
- Branch 968-qafix2-menus-over-consent, head 12ffb9e7 (one commit on 6fbb8c56)
- Worktree /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/968-qafix2-root/frontend (.next deleted)
- Budget S: ended (37 of 90 min)

## Files (11)
- src/hooks/use-bottom-bar-state.ts: MENU_LAYER_PROPS (data-qt-overlay="menu" + data-qt-nonmodal), the #337 chat-popover rule (banner counts it, the dock, header and Jump do not)
- src/components/ui/dropdown-menu.tsx: spread on DropdownMenuContent and DropdownMenuSubContent (shared primitive)
- src/components/navigation/UserProfileDropdown.tsx: spread on profile-menu-panel (968-staff-menu-d0 file, merged)
- src/components/features/wallet/MoreMenu.tsx: spread on the 768+ raw Radix non-modal menu (958-S5 file, merged in #342; landed during this run)
- src/components/ui/dialog.tsx: returnFocusTo (waits for the opener's inert to clear; stops on focusin, pointerdown, disconnect or after 2 s), used for returnFocusRef and opener paths
- src/components/consent/ConsentBanner.tsx, ConsentSettings.tsx: Customise always returns focus to Customise; after Save, Accept or Reject focus goes to the prior element, else <main> (temporary tabindex plus data-qt-focus-landing, removed on blur)
- src/app/globals.css: `html main[data-qt-focus-landing]:focus-visible{outline:none}`
- tests: consent-menu-layers.test.tsx (new, 9), consent-customise-focus-customer.test.tsx (new, 15), consent-covered-css.test.ts (+1)

## Root cause, bug 2
On the customer surface, the dialog panel's exit animation (--dur-exit 180 ms) finishes before the scrim's (--dur-base 200 ms). Radix restores focus while the scrim's RemoveScroll lock still keeps the banner inert, so Chrome drops the focus call. Chrome samples: at 190 ms focus is on BODY with the banner inert; at 250 ms the banner is free.

## Already covered (no change)
- Modal Radix menus (RoomMoreMenu, OfferRowMenu, ExportMenu) are covered through the body scroll lock.
- ui/popover.tsx, FilterBar, TradesFilters and NotificationsGlass are role=dialog.
- The phone sheets are dialogs. At 390 the bell is a link.

## Tests and gate
- gate.sh --base refs/remotes/origin/redesign/906 PASS after the rebase (receipt 968-qafix2-root/.scratch/gate/20261023T120745).
- 23 related suites, 410 tests pass.
- Without the fix: the staff menu, DropdownMenu and MoreMenu tests fail (data-covered or data-qt-overlay missing), and the Customise tests fail 10 of 10 at activeElement === body.
- Real Chrome 390x844, local build, nothing stored:
  - Staff menu: 14 of 14 entries hit-test to themselves, the dock stays visible, and Reject all is hit again after Escape.
  - Customise focus: on Customise 8 of 8; after Save it goes to <main>.
  - Control run without the fix: the same 6 entries land on the banner, and focus is on BODY 8 of 8.
  - Evidence: /Users/starbird/Workspace/Code/qic/.scratch/968/evidence-qafix2/

## Reviews (opus55-high, foreground)
1. APPROVE, with LOWs: the <main> tabindex is never removed, the focus ring on <main>, no pointerdown stop, and the popover still hides the dock. I fixed the first three.
2. APPROVE after the fixes, with LOWs: MyOfferCard not covered (ownership, below), a leftover blur listener in the refused-focus branch, a tab switch blurring <main>, thin tests for the 2 s and disconnect stops, and a stale base (I rebased).
3. APPROVE after the rebase and the MoreMenu spread, with LOWs: at 768+ the chat popover now closes when focus enters a registered menu (matches D1c "never stacks"; no test), and the modal menus rely on the scroll lock.

## Outside ownership / follow-ups
- src/components/features/offers/MyOfferCard.tsx: the hand-rolled /my-offers row menu is still not registered. 957-S10 owns the file and has not merged. Its rebuild should use the primitives or spread MENU_LAYER_PROPS.
- Non-modal ui/popover.tsx popovers still hide the dock (old behaviour).
- No backend dependency. Not re-tested on the review app (that is re-QA's job).
