# BOIL builder C: frontend reset and change-password minimum (917)

Read first: <run dir>/builder-rules.md, <run dir>/api-contract.md, PLAN-917.md Implementation steps 5 to 8 and its "Acceptance and evidence", and /Users/starbird/Workspace/Code/qic/DESIGN.md (no new colours, fonts, radius or shadows). Design notes: /Users/starbird/Workspace/Code/qic/.scratch/ticket-preparation/runtime/cards/6ace53cb94a8f2588bcb54be/1920767d5ee3fa45a355/design-notes.md and the PNG boards in that card's design/ folder (open the PNGs and match the copy). Run dir: /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022

You own exactly these files in /Users/starbird/Workspace/Code/qic/.scratch/964/wt/frontend:
- src/app/(auth)/reset-password/page.tsx
- src/components/features/settings/security/ChangePasswordSection.tsx
- src/lib/constants.ts (PASSWORD_MIN_LENGTH only)
- src/__tests__/components/reset-password-min-length-917.test.tsx (new)
- src/__tests__/components/change-password-min-length-917.test.tsx (new)

Change:
1. constants.ts: PASSWORD_MIN_LENGTH = 12 and use it on both pages.
2. Reset page: refuse `newPassword.length < PASSWORD_MIN_LENGTH` with "Password must be at least 12 characters" (derive the number from the constant); placeholder "Enter new password (min 12 characters)"; remove the native minLength attributes (keep required) so the in-page message explains the rule; keep rendering the server error message on a failed request (it is now the per-rule message, for example "password must contain a symbol, such as ! ? @ #").
3. ChangePasswordSection: length check < 12 with "Password must be at least 12 characters"; helper text "Min 12 characters with uppercase, lowercase, and number"; strength meter length thresholds 12 and 16 instead of 8 and 12; keep the uppercase/lowercase/number checks; keep showing the server message on failure.
Do not add a symbol check or a requirements checklist here: the redesign cards 955/956 own that UI; the server's per-rule message covers symbol until then.
4. Tests (jest + testing-library, match existing tests in src/__tests__/components for mocking style; mock next/navigation and the api modules):
   - reset: 11 characters in both fields shows the message and authApi.resetPassword is not called; 12 characters calls it once; the inputs have no minLength attribute; a rejected call with Error("password must contain a symbol, such as ! ? @ #") renders that text.
   - change: 8 and 11 characters show the message and usersApi.changePassword is not called; "Abcdefghij12" calls it once; helper text says 12; a rejected call renders the server message.

Checks (tee each to <run dir>/C-<what>.log with `; echo "EXIT=$?"`):
- `bunx jest src/__tests__/components/reset-password-min-length-917.test.tsx src/__tests__/components/change-password-min-length-917.test.tsx` (use the repo's own test script form if jest is wired differently; read package.json).
- `bun run typecheck` (or the repo's tsc script) and `bun run lint` on the changed files; report any pre-existing lint debt separately from new findings.
- `bun run build` once at the end.
Report: files changed, test/typecheck/lint/build result lines, log paths.
