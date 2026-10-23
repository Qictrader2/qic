# 969-qafix (card 969, size S)

- PR: https://github.com/Qictrader2/Frontend/pull/316 into redesign/906 (not merged). Branch 969-qafix-alertdialog-quote, head 173b78f8e439515197c052933519b5b493ec5a3e, rebased onto redesign/906 60582b38.
- Worktree: /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/969-qafix-root/frontend
- Budget: S, 22 of 90 min, ended.

## Files changed (10 files, +852 / -28)
- src/components/features/wallet/WithdrawModal.tsx: role="alertdialog" from the first frame. New startFreshTronQuote() clears heldTronQuote and rotates quoteSessionId. It runs on Back from either code step when a quote is held, on a continue 409 WITHDRAWAL_FEE_QUOTE_EXPIRED, and on a withdraw POST 409 of the same code (the POST path also closes the code step and clears the code). New showFeeRefreshNotice() keeps the split: inline on the customer surface, toast.info on staff pages. WITHDRAWAL_FEE_CHANGED is unchanged.
- TransferModal.tsx, LockFundsModal.tsx: role derived from the existing material latch (solid = alertdialog), so the role stays alertdialog after Back.
- New tests: src/__tests__/components/ticket-969-qafix-alertdialog.test.tsx (12 tests), src/__tests__/components/ticket-969-qafix-quote-resume.test.tsx (5 tests; a fake gas API with the backend's one-shot hold and same-session Current rule; red on the old code, the second Continue re-sent quote-1).
- Existing tests now query the alertdialog role: ticket-969-withdraw-state-survival, ticket-969-withdraw-fee-refresh (refetch count now 0 for EXPIRED and 1 for CHANGED), ticket-969-board-alignment, ticket-969-initial-focus, ticket-zoqyJwfY-withdraw-amount-precision.

## Evidence
- Backend origin/main: gas.rs continue_withdrawal_fee_quote -> repo extend_hold requires hold_until IS NULL (one-shot). reserve_refresh returns Current (the held quote) inside the minute for the same session, so a plain refetch could not break the loop. A new session is Granted at once. wallet.rs replays by idempotency key before it checks the quote.
- Gate (gate.sh --base refs/remotes/origin/redesign/906), after the rebase: typecheck, lint-no-new-errors and build pass. Receipts: 969-qafix-root/.scratch/gate/20261022T151730.
- Jest --maxWorkers=2 under the gate lock, after the rebase: 30 suites, 366 tests pass (ticket-969*, dialog-shell, 769, 645, 864 quote expiry, 9dl9xx7a step-up, drcwuijn transfers 2FA, use-withdrawal-gas, zoqyJwfY).
- Review: opus55-high APPROVE with no HIGH or MEDIUM findings. LOW 1 (a duplicate assertion) and LOW 2 (prove Continue stays disabled while the fresh quote loads) were fixed. LOW 3 and 4 are below.

## Board calls (owner rule: the board for that exact screen wins)
- Withdraw: G1 lists the security check under opaque, and 969 made the dialog solid from its first frame (single-step rule), so it is alertdialog throughout, including the code steps.
- Transfer: G1 "Send to a user: review and done" is opaque, so alertdialog from review onward.
- Lock: G1 lists "unlock and lock funds" under glass, but B5c (the board drawn for the lock and unlock screens) says the confirm layer is opaque. B5c wins, so lock is alertdialog from confirm, matching 969's opaque material.
- Checked and left as dialog (none has an opaque money commit step): Deposit, ManageWallet, the Edit offer modals (G1 lists them as glass), UserSearch, CreateDirectOffer, the resell EscrowModal, KYCRequirement's main panel. ConfirmDialog, TransactionStatusModal and the 819 notice were already alertdialog.

## Not closed / follow-ups
- PLAN AC2's evidence line "feeQuoteId after Back equals what was entered before Back" no longer holds on TRON: the new quote id is the owner-decided fix. The address, amount, note and key parts of AC2 still hold. Re-QA should read it that way.
- LOW 3 (needs its own card): useWithdraw keeps its idempotency key when amount or address are edited after a transport failure, and a replay then reports the earlier withdrawal. No money moves twice. This is older behaviour that the fix now lets users reach on TRON.
- LOW 4: the WITHDRAWAL_FEE_CHANGED branch has no emitter on backend main. I left it as it was.
- 958-S4 must keep this behaviour when it rebuilds the withdraw flow (stated in the PR).
- No backend card needed. No browser re-check on a review app (that is re-QA's job). No files outside 969's wallet dialogs and tests.
