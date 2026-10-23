# BOIL builder A: password policy module and handler wiring (backend src)

Read first: <run dir>/builder-rules.md, <run dir>/api-contract.md, and Implementation steps 1 to 6 of <run dir>/PLAN-964.md. Run dir: /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022

You own exactly these files in /Users/starbird/Workspace/Code/qic/.scratch/964/wt/qictrader-backend-rs:
- src/services/password_policy.rs (new)
- src/services/mod.rs (one `pub mod password_policy;` line, alphabetical)
- src/api/auth.rs (SignupRequest.password attribute, ResetPasswordRequest.new_password attribute, the enforce calls in signup and reset_password, and parity unit tests in its existing #[cfg(test)] module)
- src/api/users.rs (ChangePasswordRequest.new_password attribute, the enforce call in change_password)

Build, types first:
1. `pub enum PasswordRule { Length, Uppercase, Lowercase, Number, Symbol }` deriving Debug, Clone, Copy, PartialEq, Eq, Serialize with `#[serde(rename_all = "camelCase")]` (ids serialize as length, uppercase, lowercase, number, symbol). `PasswordRule::ALL` in that order, `pub const MIN_LENGTH: usize = 12`, `fn message(self) -> &'static str` with the five messages in the contract (exhaustive match, no `_ =>`), and `fn is_met_by(self, password: &str) -> bool`.
2. `pub fn unmet_rules(password: &str) -> Vec<PasswordRule>` pure, always in ALL order: length = `chars().count() >= MIN_LENGTH`; uppercase/lowercase/number/symbol = any ASCII upper / lower / digit / `is_ascii_punctuation`.
3. `pub fn enforce(password: &str) -> crate::error::Result<()>`: Ok on empty list, else `AppError::validation_with_details(json!({"error": msg, "message": msg, "code": "VALIDATION_ERROR", "status": 400, "details": {"unmetPasswordRules": [{"rule": <id>, "message": <message>}, ...]}}))` with msg the messages joined by "; ". Mirror the existing deactivate body shape in src/api/auth.rs (find it with `grep -n validation_with_details src/api/auth.rs`). Check src/error.rs to confirm a ValidationWithDetails body is returned verbatim as HTTP 400 at this base (920 added rendered_code(); confirm it does not change that branch).
4. Signup: delete the `length(min = 12 ...)` validate attribute on SignupRequest.password; call `crate::services::password_policy::enforce(&req.password)?` right after the signup-per-IP rate limit and before the accepted_legal check.
5. Reset: delete the `length(min = 6 ...)` attribute on ResetPasswordRequest.new_password (keep the struct's derive(Validate)); call enforce as the FIRST statement of reset_password, before require_db and the token hash/lookup.
6. Change: delete `length(min = 8)` on ChangePasswordRequest.new_password (keep current_password `length(min = 1)`); call enforce on new_password before find_by_id and verify_password. Never call it on current_password, login, delete account, 2FA, backup codes or mnemonic export.
7. Unit tests in password_policy.rs `#[cfg(test)]`: one per rule (missing exactly that rule gives exactly [rule]) using the plan's worked examples; 11 vs 12 boundary; '' gives all five in order; 'longenoughpassword123' gives [uppercase, symbol]; each of the 32 ASCII punctuation chars satisfies symbol alone; space, 'é', '€' and an emoji do not satisfy symbol/upper/lower; 'É' does not satisfy uppercase; 12 x 'é' plus nothing else gives [uppercase, lowercase, number, symbol] (length met by char count); 'Abcdefghij1!' and 'RotatedAfterAudit2026!secure' give []; enforce() body: status 400 via into_response, code VALIDATION_ERROR, details.unmetPasswordRules ids and messages match, message equals the joined messages, and the serialized body does not contain the password text. Test names state the invariant.
8. In src/api/auth.rs tests: the three request structs still deserialize from camelCase JSON and `.validate()` no longer refuses an 11-character password at the struct level (the policy now owns that), so there is exactly one length message.

Checks (tee each to <run dir>/A-<what>.log):
- `CARGO_INCREMENTAL=0 cargo test --lib password_policy` and `cargo test --lib api::auth::` and `cargo test --lib api::users::` (lib tests need no DB, but echo DATABASE_URL anyway).
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` once at the end.
- `grep -n "let _ =\|\.ok();\|_ =>\|unwrap()\|expect(" src/services/password_policy.rs` (only inside #[cfg(test)] is allowed for unwrap/expect).
Do not touch tests/ (builder B owns it). Report: files changed, test result lines, clippy result, log paths.
