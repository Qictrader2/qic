# Phase 0 INVESTIGATE brief (L2 worker), run 917-964-20261022

You are the L2 worker for phase 0 of a combined QicTrader ship of cards 917 and 964. You gather FACTS with commands and write them as citations. You do not fix anything, do not commit, do not push, do not touch Trello, and do not touch staging.

## Ticket
- 917 https://trello.com/c/WOXYFLpt "Passwords: reset and change accept shorter passwords than sign-up"
- 964 https://trello.com/c/0Bji1QKU "Passwords: the server should enforce all five password rules on sign-up, reset and change"
- Plans (read both in full first): /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022/PLAN-917.md and PLAN-964.md (964 may be refreshed; use the newest PLAN-964.md in that folder).
- API contract: /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022/api-contract.md

## Where
- Backend worktree: /Users/starbird/Workspace/Code/qic/.scratch/964/wt/qictrader-backend-rs (branch 964-password-policy, base origin/main 8fda17f)
- Frontend worktree: /Users/starbird/Workspace/Code/qic/.scratch/964/wt/frontend (base a73d2708)
- Run dir (write all logs and outputs here): /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022
- Do NOT set CARGO_TARGET_DIR. A shared build dir is configured by /Users/starbird/Workspace/Code/qic/.cargo/config.toml. Always set CARGO_INCREMENTAL=0. If cargo prints "Blocking waiting for file lock", that is another session; wait.
- Minted test DB for this phase: qic_631_917_964_20261022_1 (use it in DATABASE_URL for every cargo test: postgres://postgres@127.0.0.1:5433/qic_631_917_964_20261022_1)

## What to establish (checklist items 1 to 8)
1. Reproduction by running something. Write ONE temporary reproduction test file in the backend worktree at tests/password_policy_964_repro.rs (it is not committed; BOIL replaces it). Using the same harness patterns as existing tests (look at tests/auth_middleware.rs, tests/legal_acceptance.rs, tests/jwt_audit_2026_05_revocation.rs for how they build the app router and post JSON), assert TODAY's behaviour, so the tests PASS at HEAD and document the gap:
   a. POST /api/v1/auth/reset-password with {"token":"not-a-real-token","newPassword":"abc123"} (6 chars): the response is NOT a password validation error (today it reaches token lookup and says "invalid or expired reset token").
   b. POST /api/v1/users/password/change as a signed-in user with a correct currentPassword and newPassword "Abcdefg1" (8 chars, no symbol): today returns 200 (needs DB; use the minted DB and the user/session helpers from jwt_audit_2026_05_revocation.rs).
   c. POST /api/v1/auth/signup with a valid body and password "abcdefghij1!" (12 chars, no uppercase): today it is NOT refused with a validation error naming uppercase (assert the status is not 400 or, if 400, the message does not mention uppercase).
   Run it once: `( echo "DATABASE_URL=postgres://postgres@127.0.0.1:5433/<minted>"; DATABASE_URL=postgres://postgres@127.0.0.1:5433/<minted> REQUIRE_TEST_DB=1 CARGO_INCREMENTAL=0 RUST_MIN_STACK=2147483648 cargo test --test password_policy_964_repro 2>&1; echo "EXIT=$?" ) | tee <run dir>/phase-0-repro.log`. If a case cannot be built cheaply, say so and why; do not spend more than 15 minutes on the harness.
2. Root-cause code bodies, cited with `sed -n a,bp` (not only grep line hits): SignupRequest (src/api/auth.rs), ResetPasswordRequest (src/api/auth.rs), ChangePasswordRequest (src/api/users.rs), reset_password body (src/api/auth.rs), change_password body (src/api/users.rs), the start of signup up to hash_password (src/api/auth.rs), format_validation_errors (src/extractors/validated.rs), AppError::validation_with_details and its into_response branch (src/error.rs). Frontend: reset-password page validation block and inputs (frontend/src/app/(auth)/reset-password/page.tsx), ChangePasswordSection validatePassword, strength meter and helper text (frontend/src/components/features/settings/security/ChangePasswordSection.tsx).
3. Every caller and entry point: `grep -rn "hash_password(" src`, `grep -rn "SignupRequest\|ResetPasswordRequest\|ChangePasswordRequest" src tests`, the route lines for signup, reset-password and password/change, and `grep -rln "auth/signup\|reset-password\|password/change" tests` to list tests that call those routes. One line each on how the fix affects it. Name explicitly which hash_password callers are OUT of scope (admin create user, admin.rs test-account provisioning, services/test_accounts.rs, backup codes, email OTP).
4. Outcomes: the three endpoints today return one validation string; after the fix they return a 400 with details.unmetPasswordRules. List every frontend consumer of those three API calls (`grep -rn "resetPassword\|changePassword\|signup(" frontend/src/lib/api`, then the pages) and what each does with an error (cite the lines that show err.message being displayed). Also list the test fixtures that sign up or reset/change with passwords the policy will refuse (grep tests for "password" literals near signup/reset/change calls; the plan names tests/signup_default_fiat_782.rs, tests/legal_acceptance.rs, tests/banned_words_visibility.rs).
5. Intent-doc conflicts: grep qictrader-backend-rs/docs/intended-entity-state-machines.md, docs/as-built/as-built-state-machines.md and docs/design/*.md for "password" and quote any rule about password length or policy.
6. Product questions: there are none open (owner decisions 13 Oct, 16 Oct, Christian 21 Oct are in the plans). Record "does not apply" unless you find a conflict in item 5.
7. Case matrix for a held/skip money state: does not apply (no money path). Say so.
8. Record each reproduced gap as a deterministic check named "Reproduced: ..." with status pass.

Also cite: `git -C <backend wt> rev-parse HEAD` and `git merge-base HEAD origin/main` (base), and the warm-up log /Users/starbird/Workspace/Code/qic/.scratch/964/warm-cargo-3.log tail (as a log-backed citation of the warm-up, not replayed).

## Citation rules (the gate replays them)
- Every citation: {"claim","cwd","cmd","stdout","exit_code"} with stdout captured by actually running cmd (python3 subprocess or heredoc capture), never retyped. Keep each command under 180 s. Cap at about 15 citations; combine related sed ranges into one cmd with `;`.
- Cargo commands are run ONCE, tee'd to a log, and cited log-backed with an extra "log" field (path relative to cwd) instead of being replayed; the cited stdout must be a verbatim block inside the log. End the tee'd subshell with `; echo "EXIT=$?"` and cite that line.
- No secrets, tokens or passwords from ~/.config/qic/.env in any output.

## Output
Write /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022/phase-0-facts.json with keys: citations (list), facts (one entry per checklist item 1-8: "applies: ..." or "does not apply: ...", each referencing citation indexes), callers (list of {site, effect}), out_of_scope (list), fixtures_to_update (list), frontend_consumers (list), notes. Then run, from /Users/starbird/Workspace/Code/qic, a self-check that every citation's cmd re-runs to the same stdout (skip log-backed ones). Append one line to the run dir's run.log at least every 10 minutes (`echo "$(date -u +%FT%TZ) L2 phase0: <what just finished, what next>" >> run.log`).
Rules: no git stash/reset/checkout/restore; do not edit any tracked file; the only new file in the worktree is tests/password_policy_964_repro.rs. If one command waits more than 5 minutes, stop and report it (command, seconds waited, is the process alive/using CPU/files changing).
Return at most ten lines: what reproduced, counts, the facts file path.
