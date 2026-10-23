Trello: 917 https://trello.com/c/WOXYFLpt (with 964 https://trello.com/c/0Bji1QKU on the backend)

I raised the minimum on the reset-password page and the change-password section from 6 and 8 characters to 12, the same as sign-up, with the same wording as the server: "Password must be at least 12 characters".

- Reset page: in-page length check, placeholder and a persistent "At least 12 characters" hint under the field (with `aria-describedby`). I removed the native `minLength` so the page's own message explains the rule, and the error banner now has `role="alert"` and respects reduced motion.
- Change password: length check, helper text, and the strength meter now reads Weak under 12 characters.
- Both pages still show the server's message on a refusal, so a missing symbol is explained by the backend's per-rule message until the redesign cards 955 and 956 add the full checklist.
- `PASSWORD_MIN_LENGTH` is now 12 and only these two screens use it, so sign-in is unaffected.
- Tests: two new jest files (13 tests).
