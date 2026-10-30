# Android Play Store submission — QicTrader (#558)

The app is an Expo + `react-native-webview` shell (`mobile/`) that loads the
QicTrader web app. Android is the more mature side: package, permissions and
intent filters are already configured and the shell has been run on Android.

Record identifiers here as they are issued. **Never commit the keystore,
keystore passwords, or the Play service-account JSON** — see
`.cursor/rules/credential-hygiene.mdc`. `mobile/.gitignore` already excludes
`*.jks` and `*.p8`; keep it that way.

## Status

| Area | State |
| --- | --- |
| Package `com.qictrader.app`, `versionCode 1` | Set in `app.json` |
| Permissions (INTERNET, CAMERA, RECORD_AUDIO, MODIFY_AUDIO_SETTINGS) | Set in `app.json` |
| Intent filters for `staging.` and `www.qictrader.com` | Set in `app.json` |
| `eas.json` production profile building an AAB | Added, with `autoIncrement` |
| Config guard | `bun run validate:store` |
| Release upload keystore | **Blocked — needs JP, only a debug keystore exists** |
| Play App Signing enrolment | **Blocked — needs the keystore + Play Console** |
| Play Console app record | **Blocked — needs the Google org account** |
| Data Safety form | **Blocked — completed in Play Console** |
| Financial-features declaration | **Blocked — needs a product/legal answer** |
| Store listing assets (screenshots, 1024x500 feature graphic) | **Blocked — needs design/marketing** |
| Deep-link verification (`assetlinks.json`) | **Blocked — needs the release cert fingerprint** |
| Release-AAB testing + pre-launch report | **Blocked — needs a signed build** |

## What is genuinely blocked, and on whom

1. **Release upload keystore — JP.** Only a debug keystore exists today
   (`mobile/README.md`). Generating a keystore is a credential operation, so
   it should be done by JP and stored in the team secret store, never in the
   repo. Losing the upload key is recoverable under Play App Signing; losing
   the app signing key is not, which is exactly why enrolling in Play App
   Signing matters.
2. **Play Console app under the QIC Trade Systems Limited org — JP.** Read
   `ops/google-entity-accounts.md` before creating anything Google-side; there
   is a defined ownership matrix and creating this under a personal account
   would have to be undone later.
3. **`assetlinks.json` — frontend deploy, after the keystore exists.** It must
   be served from `https://www.qictrader.com/.well-known/` and contain the
   SHA-256 fingerprint of the **release** signing certificate. Because that
   fingerprint does not exist yet, `autoVerify` is deliberately left `false`
   in `app.json`, and `validate-store-config.mjs` fails the build if someone
   flips it without the file being served. Enable both together, then verify:

   ```bash
   adb shell pm verify-app-links --re-verify com.qictrader.app
   adb shell pm get-app-links com.qictrader.app   # expect: verified
   ```

4. **Financial-features declaration — product/legal.** Play has a declaration
   form for crypto and financial apps. Getting this wrong is a common cause of
   removal after publication, so it needs a definitive answer on what QicTrader
   is classified as in each market it lists in.
5. **Data Safety form — must match actual behaviour.** For this app: account
   data, financial info, photos (KYC and payment proofs), device identifiers.
   It has to reflect what the *web app inside the WebView* does, not just the
   shell, which is a common source of mismatch.

## Sequence once the keystore and Play Console app exist

```bash
cd mobile
bun install
bun run validate:store            # config guard must pass first

eas login
eas build:configure
eas credentials                   # upload the release keystore to EAS

eas build --profile preview  --platform android   # APK, internal
eas build --profile production --platform android # AAB for Play
eas submit --profile production --platform android
```

Rollout: internal testing track → closed testing → production at 10%.

## Testing that must happen on the release AAB, not a debug build

Debug and release differ in ways that matter here (minification, WebView
debugging, signing), so a green debug run proves little.

- WebView cookie and session persistence across cold restarts.
- KYC camera capture and document upload, including the runtime permission
  prompt path in `src/config.ts` (`KYC_PATH_HINTS`).
- Socket.IO trade chat live updates, including after backgrounding.
- Hardware back button: navigates WebView history, exits only at the root.
  This is implemented in `App.tsx` — verify specifically in release mode.
- Pull-to-refresh.
- External link routing to the browser or the relevant native app.
- Offline error screen and retry.
- Low-RAM device cold start; Android 10+ across small and large screens.

Run the Play pre-launch report on the internal track and require zero crashes
plus a review of its accessibility and security warnings before promoting.
