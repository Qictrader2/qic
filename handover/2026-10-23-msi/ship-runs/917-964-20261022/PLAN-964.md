<!-- qic-plan
{
  "body_sha256": "744e62a9da4b1d19db9d75afdf53d84a861b657609864226657cfbd7d18d5c12",
  "budget_minutes": 300,
  "card_id": "6ad8804c85d8a9f8bead5d9f",
  "ci_allowance_minutes": 45,
  "created_at": "2026-10-22T14:30:14.724884+00:00",
  "designs": [],
  "frontend": false,
  "implementation_gates": [],
  "local_evidence": [],
  "risk_areas": [
    "authorization"
  ],
  "severity": "security",
  "size": "L",
  "size_table": "2026-10-16",
  "source": {
    "backend": "8fda17fa8f713b5cdc5d1bfef494bf72dd54edc6",
    "frontend": "a73d2708f634a4d9ace4cdcff79ffddd3b64ade2",
    "root": "0cdc6464e5b673130027ba83ef6c78bc4231b453"
  },
  "status": "ready",
  "ticket_hash": "30435028c7a7b6059ed5386a4cf3f69c6559c7d29506a81bf02241421e35652b",
  "track": "FULL",
  "version": 1,
  "watches": [
    {
      "path": "src/api/admin.rs",
      "repo": "backend",
      "requested_path": "src/api/admin.rs",
      "sha256": "3812f980f239851bdf9af9fd42442fc7b93ddfb579ef9e191cfdca315396aa16"
    },
    {
      "path": "src/api/auth.rs",
      "repo": "backend",
      "requested_path": "src/api/auth.rs",
      "sha256": "3ee2a73601bc990443a0c6b433cd400e57e74a2540f523b66ae5a3b384ebcad1"
    },
    {
      "path": "src/api/users.rs",
      "repo": "backend",
      "requested_path": "src/api/users.rs",
      "sha256": "72141887441acbabc033de199a765ee1bb8a79a7f362cccf11ba95e54054dd90"
    },
    {
      "path": "src/error.rs",
      "repo": "backend",
      "requested_path": "src/error.rs",
      "sha256": "865a63384399195c68fe7b3089a1111a788f2a3ac4f70f4da76f576659915d23"
    },
    {
      "path": "src/extractors/validated.rs",
      "repo": "backend",
      "requested_path": "src/extractors/validated.rs",
      "sha256": "ed51bf17cc1c4f376961332fe0e229c4fc1264bf1146cbe5a400f27196a09c43"
    },
    {
      "path": "src/services",
      "repo": "backend",
      "requested_path": "src/services",
      "sha256": "284dc3637b252bdb30c1a83127898d7a93ed5e74b136f4236e62b5e241d79919"
    },
    {
      "path": "tests",
      "repo": "backend",
      "requested_path": "tests",
      "sha256": "6209d85515d2e8f63914eba591f1c404cdaade481215ab40d20076998ad61229"
    },
    {
      "path": "src/app/(auth)/reset-password/page.tsx",
      "repo": "frontend",
      "requested_path": "src/app/(auth)/reset-password/page.tsx",
      "sha256": "7ec6433a953d07929ae257b6be6a383dbe1a4e89527738c1b6617b33710dbcf0"
    },
    {
      "path": "src/app/(auth)/signup/page.tsx",
      "repo": "frontend",
      "requested_path": "src/app/(auth)/signup/page.tsx",
      "sha256": "7632bb4508f381a403bc203a335266b5fb958f7f9840ac5a42b4fd3072e6a374"
    },
    {
      "path": "src/components/features/settings/security/ChangePasswordSection.tsx",
      "repo": "frontend",
      "requested_path": "src/components/features/settings/security/ChangePasswordSection.tsx",
      "sha256": "1a593e462d422c29879a43c74bdfc7081edcdd20eaedf3d0ed3a00d6bfd664d5"
    },
    {
      "path": "src/lib/api/http-client.ts",
      "repo": "frontend",
      "requested_path": "src/lib/api/http-client.ts",
      "sha256": "17dd4b27f0f63c3f24a3e9543524374602ec2140ba06082787912d4109ccd7fb"
    },
    {
      "path": "DESIGN.md",
      "repo": "root",
      "requested_path": "DESIGN.md",
      "sha256": "3c1f6971404627c8cd5d608c698e448965ec88c4b001d5806663d278cd16126d"
    }
  ]
}
-->

# Ticket 964: Passwords: the server should enforce all five password rules on sign-up, reset and change

Severity: security
Track: FULL
Size: L
Budget: 300 min
CI allowance: 45 min
Size table: 2026-10-16
Risk areas: authorization
Frontend: False

## Expected behaviour
Sources: card 964's Context, Expected behaviour and Mandatory QA coverage sections (Christian's 21 Oct 2026 feedback on the 906 redesign: at least 12 characters, an uppercase letter, a lowercase letter, a number and a symbol, with matching backend enforcement on sign-up, reset and change password); the prior preparation's 906 v5.2 references (I6 decisions item 10 and the E6/D board rule wording 'At least 12 characters, An uppercase letter, A lowercase letter, A number, A symbol, such as ! ? @ #'); and the existing sign-up checklist in frontend/src/app/(auth)/signup/page.tsx:264-270.

The five rules match the existing sign-up checklist character for character: length = at least 12 characters, counted as Unicode scalar values the way the validator crate's length check does today; uppercase = at least one ASCII A-Z; lowercase = at least one ASCII a-z; number = at least one ASCII 0-9; symbol = at least one ASCII punctuation character (Rust char::is_ascii_punctuation, which is exactly the 32-character set in the sign-up regex). A space, an accented letter or an emoji does not count as a symbol or a letter. Defining every rule in ASCII terms means the server never refuses a password that the browser checklist shows as met.

POST /api/v1/auth/signup, POST /api/v1/auth/reset-password and POST /api/v1/users/password/change each run the same pure check on the new password before any database write. A password that misses one or more rules gets HTTP 400 with code VALIDATION_ERROR and this body: message and error are the per-rule messages joined with '; ', and details.unmetPasswordRules is an ordered array of {rule, message} with rule ids that match the frontend checklist keys: length, uppercase, lowercase, number, symbol. Per-rule messages follow the backend's lowercase style: 'password must be at least 12 characters', 'password must contain an uppercase letter', 'password must contain a lowercase letter', 'password must contain a number', 'password must contain a symbol, such as ! ? @ #'.

Worked examples: 'Abcdefgh1!x' (11 characters) -> [length]. 'abcdefghij1!' -> [uppercase]. 'ABCDEFGHIJ1!' -> [lowercase]. 'Abcdefghijk!' -> [number]. 'Abcdefghijk1' -> [symbol]. 'longenoughpassword123' -> [uppercase, symbol]. '' -> all five, in that order. 'Abcdefghij1!' (exactly 12) -> accepted. 'Abcdefghij1 ' (space instead of symbol) -> [symbol].

A refused reset does not use up the reset link, so the user can retry with a compliant password. A refused change leaves the stored password and every session unchanged and sends no 'Password changed' notification. A refused sign-up creates no user row. Accepted passwords behave exactly as today: sign-up returns the pending-verification response, reset revokes all sessions and notifies, change keeps the current session, revokes the others and notifies.

Sign-in, account deletion confirmation, 2FA setup and disable, viewing backup codes, mnemonic export and change-password's currentPassword verify an existing password and never apply the policy. An account whose stored password breaks the new rules, for example 'longenoughpassword123' or an 8-character admin-created password, still signs in (card: 'Existing passwords keep working at sign-in; the rules apply only when a password is set').

## Diagnosis
Implementation gap, verified by reading source at backend pin 8fda17fa (no live reproduction attempted; nothing on staging was touched). Re-checked against the current snapshot: the watched-source delta since the prior draft (backend f043a141 -> 8fda17fa) is card 920's money_request work (new src/services/money_request.rs, a new AppError::ReplayedRefusal variant and rendered_code() in src/error.rs, new tests) and touches none of the password paths; frontend and root watched files are unchanged. The server checks password length only, and differently on each of the three paths that set a password: (1) Sign-up: SignupRequest in qictrader-backend-rs/src/api/auth.rs:63 has #[validate(length(min = 12))] and nothing else. (2) Reset: ResetPasswordRequest in src/api/auth.rs:138 has #[validate(length(min = 6))]; reset_password (src/api/auth.rs:2805-2867) looks up the token, then hashes and stores whatever passes. (3) Change: the ticket points at auth.rs, but the handler is change_password in src/api/users.rs:1593 (route POST /api/v1/users/password/change), and ChangePasswordRequest (src/api/users.rs:141-146) has #[validate(length(min = 8))] on new_password, not 6. No backend code anywhere checks for an uppercase letter, a lowercase letter, a digit or a symbol: there is no password_policy module or character-class check on passwords. So the five rules exist only in the browser: frontend/src/app/(auth)/signup/page.tsx:264-270 checks all five, frontend/src/components/features/settings/security/ChangePasswordSection.tsx:35-38 checks 8 characters plus upper, lower and digit (no symbol), and frontend/src/app/(auth)/reset-password/page.tsx:57 checks 6 characters. A direct API call skips all of them. Card 917 (reset/change length) has not merged: auth.rs:138 and users.rs:144 still carry min 6 and min 8 at this pin. Error plumbing that already exists: ValidatedJson (src/extractors/validated.rs) joins validator messages into one string with no structure. AppError::validation_with_details (src/error.rs:534) carries a structured body that IntoResponse returns verbatim as HTTP 400 before the new rendered_code() path runs (src/error.rs:906-907), with a precedent body in deactivate (src/api/auth.rs:2900-2909). The frontend http-client already passes data.message to the thrown Error and keeps data.details (frontend/src/lib/api/http-client.ts:463-482). All three forms show err.message (signup/page.tsx:751-754 and 803-810, reset-password/page.tsx:76-78, ChangePasswordSection.tsx:92-95). Sign-in never applies a length rule: LoginRequest is length(min = 1) (auth.rs:88) and the login path only calls verify_password. Existing hashes are therefore not at risk unless the new check is wrongly wired into login or another password-confirmation path.

## Scope
Included (backend only, qictrader-backend-rs): a new pure module src/services/password_policy.rs; wiring it into signup and reset_password in src/api/auth.rs and change_password in src/api/users.rs; removing the old per-endpoint length attributes (auth.rs:63 min 12, auth.rs:138 min 6, users.rs:144 min 8) so each refusal reports once, in one shape; unit tests; endpoint integration tests; updating CI test fixtures that sign up with now-refused passwords.

Excluded: (1) Frontend checklist, labels, count ('4 of 5 requirements met') and the shared component. These belong to cards 955 (906-E) and 956 (906-D), which consume details.unmetPasswordRules. No frontend change is needed for this card, because all three forms already show the server's message, so no design package applies. (2) Admin create-user (src/api/admin.rs:3639, length(min = 8)) and operator/test-account provisioning (src/services/test_accounts.rs and the admin provisioning route in src/api/admin.rs). The card names only sign-up, reset and change; extending the policy to admin paths would be a separate card. Admin create-user also stays usable as the staging fixture for a legacy weak password (AC 'existing passwords can still sign in'). (3) Forcing existing users to rotate non-compliant passwords. Explicitly not wanted: the rules apply only when a password is set. (4) Maximum length, breached-password lists and password reuse. These were not requested.

Dependencies: card 917 (WOXYFLpt) covers the reset/change length gap on the same struct fields and has not merged at backend pin 8fda17fa. The card says to do 964 together with 917 or after it; the policy's length rule is 917's fix, so ship both in one change. If 917 merges separately first and adds length(min = 12) attributes, remove them in favour of the policy so the length message is not reported twice. Rollout: backend-only, no migration, no config, no data change, plain revert. Safe to deploy before 955/956: the sign-up page already blocks non-compliant passwords client-side; reset (6 characters) and change (8 characters, no symbol) forms will show the server's per-rule message as their existing error text or toast until 955/956 replace them.

## Implementation
1. Create qictrader-backend-rs/src/services/password_policy.rs (types first). Add a #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)] #[serde(rename_all = "camelCase")] enum PasswordRule { Length, Uppercase, Lowercase, Number, Symbol } with a const ALL in display order, pub const MIN_LENGTH: usize = 12, and fn message(self) -> &'static str returning the five lowercase messages listed in expected_behaviour. Add pub fn unmet_rules(password: &str) -> Vec<PasswordRule>: pure, no allocation beyond the Vec, using chars().count() >= MIN_LENGTH, any(char::is_ascii_uppercase), any(is_ascii_lowercase), any(is_ascii_digit), any(is_ascii_punctuation), always in ALL order. Add pub fn enforce(password: &str) -> crate::error::Result<()> that returns Ok on an empty list. Otherwise it returns AppError::validation_with_details(json!({"error": msg, "message": msg, "code": "VALIDATION_ERROR", "status": 400, "details": {"unmetPasswordRules": [{"rule": ..., "message": ...}]}})), where msg is the messages joined with "; ", mirroring the deactivate body at src/api/auth.rs:2900-2909. Never log or echo the password.
2. Register it with `pub mod password_policy;` in src/services/mod.rs (alphabetical, between platform_fee at line 69 and preferred_currency at line 70).
3. src/api/auth.rs signup: delete the #[validate(length(min = 12, ...))] attribute on SignupRequest.password (keep Validate for email/username). Call crate::services::password_policy::enforce(&req.password)? directly after the signup-per-IP enforce_rate_limit (auth.rs:442-447) and before the accepted_legal check (auth.rs:454), so refused attempts still count toward PENTEST-002 limits and no DB access happens first.
4. src/api/auth.rs reset_password: delete the length(min = 6) attribute on ResetPasswordRequest.new_password (the struct keeps #[derive(Validate)] for ValidatedJson). Call password_policy::enforce(&req.new_password)? as the first statement, before require_db/hash_reset_token/find_valid_reset_token (auth.rs:2809-2815), so a refused password never reaches set_password_hash, mark_reset_token_used or delete_all_sessions.
5. src/api/users.rs change_password: delete length(min = 8) on ChangePasswordRequest.new_password and keep length(min = 1) on current_password. Call password_policy::enforce(&req.new_password)? before find_by_id/verify_password (users.rs:1598-1607). currentPassword is never policy-checked.
6. Unit tests in password_policy.rs #[cfg(test)]: one test per rule (password missing exactly that rule -> exactly [rule]); 11 vs 12 characters boundary; empty string -> all five in order; multiple misses keep order; each of the 32 ASCII punctuation characters satisfies symbol; space, 'é', '€' and an emoji do not satisfy symbol/upper/lower; non-ASCII uppercase 'É' does not satisfy uppercase; a compliant 12-character password -> empty; enforce() error body has status 400, code VALIDATION_ERROR, details.unmetPasswordRules ids and messages, and contains no password text.
7. New integration test qictrader-backend-rs/tests/password_policy_964.rs, following the jwt_audit_2026_05_revocation.rs and legal_acceptance.rs harness patterns. No-DB part: signup and reset each refuse the five failing passwords from expected_behaviour with the named rule. DB part, run in CI with REQUIRE_TEST_DB: a refused reset leaves the token usable, and a compliant reset with the same token then succeeds; a refused change keeps the old hash and the other sessions; a compliant sign-up, reset and change are accepted; a user inserted with hash('longenoughpassword123') signs in, and can change to a compliant password using that weak current password.
8. Update CI fixtures that sign up with passwords the policy now refuses: tests/signup_default_fiat_782.rs:48 and tests/legal_acceptance.rs:242 ('longenoughpassword123'), tests/banned_words_visibility.rs:166 ('correct-horse-battery-staple'). Replace them with a compliant value such as 'LongEnough-Pass123'. Leave the insert_user/login fixtures in legal_acceptance.rs (296-615) and the direct hash_password fixtures in waitlist_virtual_card_endpoint.rs:66, demand_signal_endpoint.rs:69 and legal_idor.rs:77 as they are: they hash directly and never call a policy endpoint, and the legal_acceptance ones also prove that legacy passwords keep working. Re-check tests/auth_middleware.rs:322-344 (signup_validates_email and signup_validates_username_length still fail in ValidatedJson on email/username; signup_validates_password_length with 'short' now fails in the policy with an error that contains lowercase 'password') and tests/pentest_002_auth_rate_limit.rs:229-241 and 319-338 ('password12345' bodies: the policy runs after the rate limiter, so the 6th signup still gets 429 RATE_LIMITED, and after a login lockout signup gets 400, not 429); with step 3's ordering all pass unchanged. tests/jwt_audit_2026_05_revocation.rs:336 already uses a compliant new password ('RotatedAfterAudit2026!secure').
9. Targeted local checks only: cargo test --lib password_policy, plus the new integration test's no-DB cases. CI runs the full suite, including DB tests. Then staging QA per acceptance.

## Risks
- Security/authentication: this changes the guard on every self-service credential-setting endpoint. Wiring the policy into login or into another password-confirmation path (delete account, 2FA setup or disable, backup codes, mnemonic export, change-password's currentPassword) would lock out existing users whose passwords predate the rules. Only the three new-password fields may call it, and the AC for existing passwords tests sign-in plus a change that uses a weak current password.
- Reset-token consumption: if the policy check ran after find_valid_reset_token/mark_reset_token_used or delete_all_sessions, a refused reset would burn the emailed link or log the user out everywhere. The check must come first, and an integration test proves the same token still works afterwards.
- Server/browser rule mismatch: the server must not be stricter than the checklist. ASCII definitions match signup/page.tsx:264-270 exactly. One residual edge case: JavaScript length counts UTF-16 units, so 10 ASCII characters plus one emoji shows 12 in the browser but 11 on the server, and the server refuses it with the length message. ChangePasswordSection.tsx counts any non-alphanumeric character (including a space) as a symbol, so until 956 replaces it, a space-only 'symbol' gets a server refusal naming the symbol rule.
- Interim UX before 955/956 ship: reset (client minimum 6) and change (client minimum 8, no symbol check) will submit passwords the server now refuses. Users see the server's per-rule message as plain error text or a toast, not the redesigned checklist. This is intended enforcement, but support may see questions.
- API contract: details.unmetPasswordRules (rule ids length, uppercase, lowercase, number, symbol) becomes a contract that 955 and 956 consume. Renaming the ids later breaks those cards.
- Overlap with card 917 on the same struct fields (still unmerged at backend 8fda17fa): shipping them separately risks a duplicated length message or a merge conflict. Implement once, together.
- Error-rendering drift: card 920 just added AppError::ReplayedRefusal and a rendered_code() path in src/error.rs. ValidationWithDetails still short-circuits to its verbatim body at error.rs:906-907; if a later change routes it through rendered_code()/user_message(), details.unmetPasswordRules would be lost. The unit test on enforce()'s rendered response guards this.
- CI fixtures: three DB test files sign up with passwords that are now refused (signup_default_fiat_782.rs:48, legal_acceptance.rs:242, banned_words_visibility.rs:166) and will fail until updated. Other suites or QA scripts that call these endpoints with weak passwords will also start getting 400.
- Staging QA accounts: running the change or reset ACs on the shared QA accounts could leave a password that cannot be restored to its previous non-compliant value, and that desyncs ~/.config/qic/.env. Use throwaway accounts for change and reset, and never change the shared QA logins.
- Admin create-user (admin.rs:3639, minimum 8) and operator test-account provisioning remain outside the policy by scope. Admin-created accounts can still hold weak passwords until a separate card says otherwise.
- No money, ledger, state-machine, migration or background-job change. Rollback is a plain code revert with no data to undo.

## Acceptance and evidence

### AC1: On staging, sign-up, reset and change password each refuse a password that misses any of the five rules, and say which rule is missing.
- Setup: Unit tests for services::password_policy::unmet_rules (mandatory QA AC3: each rule and the combined check): inputs 'Abcdefgh1!x' (11), 'abcdefghij1!', 'ABCDEFGHIJ1!', 'Abcdefghijk!', 'Abcdefghijk1', 'Abcdefghij1 ' (space), 'Abcdefghijé1', '' and 'longenoughpassword123'.
  Expected: In that order: [length], [uppercase], [lowercase], [number], [symbol], [symbol], [symbol], [length, uppercase, lowercase, number, symbol], [uppercase, symbol]. Every one of the 32 ASCII punctuation characters satisfies symbol on its own. enforce() returns HTTP 400 VALIDATION_ERROR with details.unmetPasswordRules carrying matching rule ids and messages, and the body never contains the password.
  Evidence: cargo test --lib password_policy, run locally as a targeted check and in backend CI on the PR; cite the CI log lines.
- Setup: Integration (no DB): POST /api/v1/auth/signup with a valid email, username, acceptedLegal true and preferredCurrency ZAR, once for each of the five single-rule failing passwords. Separately, POST /api/v1/auth/reset-password with any token and each of the five passwords.
  Expected: Every request returns 400, code VALIDATION_ERROR, and details.unmetPasswordRules containing exactly the one missing rule. message names that rule (for example 'password must contain a symbol, such as ! ? @ #'). Sign-up refuses before any DB access. Reset refuses before token lookup.
  Evidence: tests/password_policy_964.rs no-DB cases, run in backend CI on the PR (and locally as a targeted check).
- Setup: Integration (CI Postgres): (a) A user with a valid reset token submits each failing password to /auth/reset-password. (b) A signed-in user with two sessions submits the correct currentPassword with each failing newPassword to /users/password/change. (c) Sign-up with each failing password, then count users by that email.
  Expected: (a) 400 naming the rule. The token stays unused (password_reset_tokens.used_at is still null), sessions are not deleted, and a follow-up compliant reset with the same token returns 200. (b) 400 naming the rule. password_hash is unchanged, so the old password still signs in. Both sessions remain, and no 'Password changed' notification row is written. (c) 400 each time, and no users row exists for the email.
  Evidence: tests/password_policy_964.rs DB cases, executed by backend CI with REQUIRE_TEST_DB; cite the CI log.
- Setup: Staging, after deploy, through direct API calls (bypassing the browser checks) and through the UI: sign-up with a fresh throwaway email; forgot-password then reset with the emailed link for a throwaway account; change password signed in as a throwaway account. Each uses 'Abcdefgh1!x', 'abcdefghij1!', 'ABCDEFGHIJ1!', 'Abcdefghijk!' and 'Abcdefghijk1'. Shared QA logins are not used for reset or change.
  Expected: All 15 attempts return 400 VALIDATION_ERROR naming the single missing rule in message and details.unmetPasswordRules. The reset and change forms show that message as their error text or toast. No account is created, the reset link still works afterwards, and the throwaway account's password is unchanged.
  Evidence: qic-qa staging run: recorded request/response pairs (password values redacted to rule labels) plus screenshots of the reset and change error states; the throwaway account is torn down afterwards per the throwaway-account recipe.

### AC2: A password that meets all five rules is accepted on all three.
- Setup: Unit: unmet_rules('Abcdefghij1!') (exactly 12 characters, all five classes) and a longer compliant value such as 'RotatedAfterAudit2026!secure'.
  Expected: Both return an empty list, and enforce() returns Ok(()).
  Evidence: cargo test --lib password_policy in backend CI on the PR.
- Setup: Integration (CI Postgres): compliant 'Abcdefghij1!' submitted to sign-up (new email, acceptedLegal true, ZAR), to reset-password with a valid token, and to password/change with the correct current password and a second session open.
  Expected: Sign-up returns the existing pending-email-verification success and creates the user with a hash. Reset returns 200, marks the token used, revokes all sessions, and lets the new password sign in. Change returns 200, keeps the current session, revokes the other one (existing JWT-AUDIT-2026-05 behaviour), and lets the new password sign in. Existing suites jwt_audit_2026_05_revocation, signup_default_fiat_782, legal_acceptance, banned_words_visibility, auth_middleware and pentest_002_auth_rate_limit pass with updated compliant fixtures where step 8 names them.
  Evidence: tests/password_policy_964.rs plus the updated existing suites, green in backend CI; cite the CI log.
- Setup: Staging, after deploy: the same throwaway flows as the refusal case, using a compliant 12-character password on sign-up, reset and change.
  Expected: All three succeed with today's success responses and UI states, and the account can then sign in with the new password through the scripted sign-in (qic-qa-signin agent).
  Evidence: qic-qa staging run: recorded responses and screenshots of the three success states; throwaway accounts cleaned up afterwards.

### AC3: Customers whose existing password breaks the new rules can still sign in.
- Setup: Integration (CI Postgres): insert an email-verified user whose password_hash is hash_password('longenoughpassword123') (fails uppercase and symbol), and a second user with an 8-character 'legacypw' hash (fails four rules). POST /api/v1/auth/login for each with the correct password.
  Expected: Neither login is refused by the policy. The response is the normal success, or the existing new-device emailed-code or 2FA challenge, never 400 VALIDATION_ERROR and never 401. A wrong password still gets the normal 401.
  Evidence: tests/password_policy_964.rs legacy-login cases, plus the existing legal_acceptance.rs insert_user/login tests (which use 'longenoughpassword123'), green in backend CI.
- Setup: Integration (CI Postgres): a legacy user signs in, then calls /users/password/change with currentPassword 'longenoughpassword123' and a compliant newPassword. A separate request tries a non-compliant newPassword.
  Expected: The compliant change returns 200, which shows that currentPassword is verified, not policy-checked. The non-compliant newPassword returns 400 naming its rules, and the legacy password still signs in afterwards.
  Evidence: tests/password_policy_964.rs, green in backend CI.
- Setup: Staging, after deploy: create a throwaway account through admin create-user (outside this policy, minimum 8) with a 10-character lowercase+digit password, flip email_verified as the throwaway-account recipe requires, then sign in through the qic-qa-signin agent.
  Expected: Sign-in succeeds, through the trusted-device or emailed-code step as built, with no password-rule error. The account is then torn down.
  Evidence: qic-qa staging run: sign-in response or screenshot, plus a teardown record.

## Designs
No frontend work.

## Design decision
Not applicable.

## Implementation approval
No additional implementation approval gate recorded by preparation.
