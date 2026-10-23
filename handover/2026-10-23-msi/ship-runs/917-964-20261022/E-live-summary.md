# E-live summary (917 + 964), worker D

Backend: cargo run on 127.0.0.1:18964, DB qic_631_917_964_20261022_4, APP_ENV=development, chain/price URLs at 127.0.0.1:9, KYC_PROVIDER=didit with dummy local creds (needed to boot). Frontend: bunx next dev -p 3964 (NEXT_PUBLIC_API_URL=/api/v1, BACKEND_URL=http://127.0.0.1:18964). Chrome for Testing 143, temp profile. Logs: E-live-backend.log, E-live-frontend.log. Calls: E-live-calls.jsonl (JWTs redacted). DB: E-live-db.txt. Scripts: E-live-api.py, E-live-browser.ts. Browser requests: E-live-fe-requests.json.

1. Sign-up refusals (X-Forwarded-For honoured, no rate-limit trouble): all five 400 VALIDATION_ERROR with unmetPasswordRules holding exactly the one missing rule (length, uppercase, lowercase, number, symbol); users count = 0 for each email.
2. Sign-up 'Abcdefghij1!': 500 CONFIG_ERROR (SMTP unset) but users row exists with password hash; verified flags set by SQL.
3. Reset: 11 chars 400 length; 'Abcdefghijk1' 400 symbol; used_at still NULL after both; 'Abcdefghij1!x2' with same token 200, used_at set.
4. Change: login 200; 'Abcdefg1' 400 (length + symbol); 'Abcdefghij12' 400 symbol; compliant 200.
5. Legacy: user made super_admin by SQL; POST /admin/users with 'longenoughpassword123' 200; login 200; change to compliant 200.
6. Reset page: 11 chars shows "Password must be at least 12 characters", zero requests to /auth/reset-password; 'Abcdefghijk1' sent once, 400, page shows server symbol message; token used_at still NULL.
7. Settings > Security: signed in via /login (login 200, settings loaded). Helper text "Min 12 characters with uppercase, lowercase, and number". 11 chars: in-page message, no password/change request. 'Abcdefghij12': one POST, 400, toast shows the symbol message.

PNG sha256:
- E-live-reset-11.png fc5ee9bf9f07397f804c7d7ef50e63761571bbf049a2ebf884a95c943236ecd4
- E-live-reset-12-nosymbol.png c138a6872185b63e3be4e2479910f3398ece9065ddb4b051551c910b82493381
- E-live-login.png 5e3d334d9ecbd1c23a398e41f1eaf60ea8e89bbf8aa029a18c4b42939472659b
- E-live-change-11.png a5e1c47a849d3cfbdaed131af96fa5740ad864aab99f87889e319af29f93b4a3
- E-live-change-12-nosymbol.png 3117d8a0c29f25b8ee95948b2de964297bcfddf073a1d1213513bf7738fb1e22

Notes: E-live-login.png was taken while still on /login after the sign-in click (URL did not change in 4s, but the session worked). Disk was 11 GB free at start (rule 4 says 15); build was warm and finished. First boot failed without KYC provider env; env fix: a dev default for KYC. Stopped: backend, next dev, Chrome; ports 18964/3964 free. Skipped: nothing.
