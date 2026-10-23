# API contract: password policy (917 + 964)

Endpoints: POST /api/v1/auth/signup (field `password`), POST /api/v1/auth/reset-password (field `newPassword`), POST /api/v1/users/password/change (field `newPassword`; `currentPassword` is never policy-checked).

Rules, in this order, ids are the frontend checklist keys:

| id | passes when | message |
| --- | --- | --- |
| length | `chars().count() >= 12` | password must be at least 12 characters |
| uppercase | any ASCII A-Z | password must contain an uppercase letter |
| lowercase | any ASCII a-z | password must contain a lowercase letter |
| number | any ASCII 0-9 | password must contain a number |
| symbol | any `char::is_ascii_punctuation` | password must contain a symbol, such as ! ? @ # |

Refusal: HTTP 400

```json
{
  "error": "<messages joined with '; '>",
  "message": "<messages joined with '; '>",
  "code": "VALIDATION_ERROR",
  "status": 400,
  "details": { "unmetPasswordRules": [ { "rule": "length", "message": "password must be at least 12 characters" } ] }
}
```

The check runs before any database read or write on all three endpoints (after the sign-up rate limit). The body never contains the password.

Frontend (917, main branch, current pages only): reset page and ChangePasswordSection refuse under 12 with "Password must be at least 12 characters"; they already render the server's `message` on a 400.
