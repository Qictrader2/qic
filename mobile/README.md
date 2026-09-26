# QicTrader Mobile (Android-first)

Native shell (Expo / React Native) around the **real QicTrader web app**.
The WebView renders `https://staging.qictrader.com` by default, so UI and
features are 1:1 with web **by construction** — every web deploy is
instantly live in the app with no store release.

## Why a shell, not a rewrite

The product changes fast. A parallel native UI would drift from web
within days. The shell keeps a single source of truth (the `frontend/`
Next.js app) and adds only what a wrapper must provide natively:

| Concern | Where handled |
| --- | --- |
| Session persistence (cookies + localStorage) | `App.tsx` WebView props |
| Android hardware back = web history back | `BackHandler` in `App.tsx` |
| Camera/mic for KYC liveness (`getUserMedia`) | runtime permission pre-request on KYC routes |
| File upload (payment proof, KYC docs) | native Android file chooser (built into the WebView) |
| External links (WhatsApp, banks, mailto:, tel:) | `onShouldStartLoadWithRequest` → OS |
| Offline / load-failure screen with retry | `App.tsx` error state |
| Splash + adaptive icon (brand `#00A3F6`) | `app.json` + `assets/` (copied from `frontend/public/`) |

## Environment

Default target is **staging** (`https://staging.qictrader.com`, which
talks to the staging backend — staging creds work as-is). Override at
build time:

```bash
EXPO_PUBLIC_WEB_APP_URL=https://www.qictrader.com   # prod build — JP sign-off required
```

Config lives in `src/config.ts` (target URL, internal-host allowlist,
KYC path hints).

## Build an APK (local, no EAS account needed)

Requires JDK 17 and the Android SDK (`ANDROID_HOME`); both are on JP's Mac.

```bash
cd mobile
bun install
bunx expo prebuild --platform android   # generates android/ (gitignored)
cd android && ./gradlew assembleRelease
# APK: android/app/build/outputs/apk/release/app-release.apk
```

Install on a device: `adb install -r app-release.apk`

> The generated `android/` project signs release builds with the debug
> keystore so a shareable APK works out of the box. Before any Play
> Store submission, generate a proper upload keystore and configure
> `signingConfigs.release`.

## Dev loop

```bash
bun run android   # Expo dev build on emulator/device
```

## Repo notes

- Lives in the `Qictrader2/qic` monorepo under `mobile/`.
- `Qictrader2/qictrader-mobile` on GitHub is an empty bootstrap (rules
  docs only, no app code); this monorepo folder supersedes it for the
  Android shell.
