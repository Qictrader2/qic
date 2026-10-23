# BOIL builder B: backend integration tests and fixtures (backend tests/)

Read first: <run dir>/builder-rules.md, <run dir>/api-contract.md, PLAN-964.md steps 7 to 8 and the "Acceptance and evidence" sections of PLAN-964.md and PLAN-917.md. Run dir: /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022. Phase 0 facts (callers, fixtures, harness notes): <run dir>/phase-0-facts.json.

You own exactly these files in /Users/starbird/Workspace/Code/qic/.scratch/964/wt/qictrader-backend-rs:
- tests/password_policy_964.rs (new; this replaces the phase-0 reproduction file tests/password_policy_964_repro.rs, which you delete when your file covers its cases)
- tests/signup_default_fiat_782.rs, tests/legal_acceptance.rs, tests/banned_words_visibility.rs: ONLY the password literals used for sign-up (or reset/change) that the policy now refuses; replace with a compliant value such as "LongEnough-Pass123". Do not change insert_user/login fixtures that hash directly and sign in: they prove legacy passwords still work.
- any other tests/*.rs that sign up, reset or change with a now-refused password (find them: `grep -rln "auth/signup\|reset-password\|password/change" tests`), same rule.

Builder A is adding src/services/password_policy.rs with `PasswordRule`, `unmet_rules`, `enforce` and wiring it into the three handlers at the same time. Write against the contract; the response shape is fixed in api-contract.md.

tests/password_policy_964.rs, using the harness of tests/jwt_audit_2026_05_revocation.rs and tests/legal_acceptance.rs (test_state, user creation, token issue; DB tests skip without DATABASE_URL unless REQUIRE_TEST_DB is set, the same way those files do):
No-DB or rate-limit-safe cases:
- signup refuses each of 'Abcdefgh1!x', 'abcdefghij1!', 'ABCDEFGHIJ1!', 'Abcdefghijk!', 'Abcdefghijk1' with 400, code VALIDATION_ERROR, details.unmetPasswordRules exactly [the one missing rule] and message containing that rule's message. Use distinct client IPs or headers if the signup rate limit (5 per IP) would trip; read tests/pentest_002_auth_rate_limit.rs to see how IP is derived.
- reset-password refuses the same five with the same assertions, before token lookup (a bogus token still gets the rule error, not "invalid or expired reset token").
DB cases (CI Postgres, REQUIRE_TEST_DB):
- reset: seed a real reset token for a user (repo::user::create_password_reset_token with services::auth::hash_reset_token, find the signatures). An 11-char password and each single-rule failure are refused, password_reset_tokens.used_at stays NULL, password_hash unchanged; then 'Abcdefghij1!' with the SAME token returns 200, used_at set, hash changed, all sessions deleted.
- reset with a 12-char compliant password and an unknown token still returns 400 "invalid or expired reset token".
- change: signed-in user with two sessions; newPassword of 8, 9, 10, 11 chars and each single-rule failure are refused (400 naming the rule), hash unchanged, both sessions remain, no new 'Password changed' notification row. Compliant 'Abcdefghij1!' returns 200, current session kept, the other revoked, new password signs in.
- change with a compliant new password and a WRONG currentPassword returns 401 "current password is incorrect", hash unchanged.
- signup: compliant 'Abcdefghij1!' returns today's success; each failing password creates no users row for its email.
- legacy: users whose stored hash is hash_password('longenoughpassword123') and hash_password('Abcdef12') sign in through POST /api/v1/auth/login with no 400 and no 401 (success, or the existing new-device/2FA challenge); a wrong password still gets 401. The 'longenoughpassword123' user can change to a compliant password using that weak currentPassword, and a non-compliant newPassword is refused while the weak password still signs in afterwards.
Test names state the invariant (e.g. refused_reset_keeps_token_usable).

Checks (tee each to <run dir>/B-<what>.log, DATABASE_URL echoed first, `; echo "EXIT=$?"` last):
- `cargo test --test password_policy_964` on your minted DB, 3 runs in a row in one log.
- every fixture file you changed: `cargo test --test <file>` on the minted DB.
- `cargo test --test auth_middleware --test pentest_002_auth_rate_limit --test jwt_audit_2026_05_revocation` on the minted DB (they name the changed routes).
Your tests need builder A's code; if A is not finished when you are ready to run, wait (poll for src/services/password_policy.rs and `grep -n "password_policy::enforce" src/api/auth.rs src/api/users.rs`) rather than editing src/. Report: files changed, per-target test result lines, log paths.
