Trello: 964 https://trello.com/c/0Bji1QKU and 917 https://trello.com/c/WOXYFLpt

I added one pure password check, `services::password_policy`, and run it on sign-up, password reset and change password. A new password must have at least 12 characters, an uppercase letter, a lowercase letter, a number and a symbol. Before this, sign-up only checked length (12), reset accepted 6 characters and change accepted 8, and nothing checked the other four rules.

What changed
- `PasswordRule` enum (length, uppercase, lowercase, number, symbol) with `unmet_rules` and `enforce`. The rules use ASCII definitions so they match the sign-up page's checklist exactly.
- A refusal is HTTP 400 `VALIDATION_ERROR` with `details.unmetPasswordRules`, one `{rule, message}` per missing rule, in a fixed order. The redesign cards 955 and 956 read this list.
- The check runs before any database work: right after the sign-up rate limit, as the first step of reset (so a refused reset never uses up the emailed link), and before the current-password check on change. It never runs on login or on `currentPassword`, so existing passwords keep signing in.
- I removed the three old per-field length limits so each refusal is reported once.
- Tests: 20 unit tests for the rules, and `tests/password_policy_964.rs` with 12 DB tests (refusals on all three endpoints, token survives a refused reset, hash and sessions untouched by a refused change, legacy weak passwords still sign in and can be changed). I updated the test fixtures that signed up with passwords the policy now refuses.

Out of scope: admin create-user and test-account provisioning still use their own minimum of 8. That needs its own card.
