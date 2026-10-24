# iOS App Store submission — QicTrader (#557)

The app is an Expo + `react-native-webview` shell (`mobile/`) that loads the
QicTrader web app. There is no native feature work; everything below is
account setup, signing, declarations and store metadata.

Record identifiers here as they are issued. **Never put certificates,
provisioning profiles, API keys or passwords in this repo** — see
`.cursor/rules/credential-hygiene.mdc`. Reference the secret's name and where
it lives, not its value.

## Status

| Area | State |
| --- | --- |
| Bundle identifier `com.qictrader.app` | Set in `app.json` |
| Camera / mic / photo usage strings | Set in `app.json` `ios.infoPlist` |
| Privacy manifest (`NSPrivacyAccessedAPITypes`) | Declared in `app.json` `ios.privacyManifests` |
| `eas.json` build profiles | Added (`development`, `preview`, `production`) |
| Config guard | `bun run validate:store` |
| Apple Developer account / team | **Blocked — needs JP** |
| Signing certificate + provisioning profile | **Blocked — needs the Apple account** |
| App Store Connect app record | **Blocked — needs the Apple account** |
| App Store Connect API key (for CI submit) | **Blocked — needs the Apple account** |
| Privacy nutrition labels | **Blocked — entered in App Store Connect** |
| Screenshots per device class | **Blocked — needs design/marketing** |
| Universal links (`apple-app-site-association`) | **Blocked — needs a frontend deploy** |
| Device-matrix + TestFlight testing | **Blocked — needs a signed build** |

## What is genuinely blocked, and on whom

1. **Apple Developer Program membership under QIC Trade Systems Limited — JP.**
   Guideline 3.1.5(b) requires crypto trading apps to be submitted by the
   organisation that operates the exchange, not an individual. An individual
   account will be rejected, so this must be the company entity. Everything
   else on iOS depends on this existing first.
2. **Signing material — whoever holds the Apple account.** Certificates and
   provisioning profiles are generated per team. EAS can manage them
   (`eas credentials`), which is preferable to hand-managed `.p12` files.
3. **`apple-app-site-association` — frontend deploy.** Universal links need
   this served from `https://www.qictrader.com/.well-known/`, containing the
   team ID and bundle identifier. The team ID does not exist yet, so the
   associated-domains entitlement is deliberately **not** in `app.json`:
   shipping the entitlement without the file leaves links silently opening in
   Safari and hides the misconfiguration.
4. **Demo account for review — product decision.** Reviewers need to reach the
   trading surfaces. A staging account is not acceptable to Apple; a
   restricted production demo login has to be designed (what it can see, what
   it must not be able to do, whether it can hold balance). Needs Christian
   and Marcello.
5. **Screenshots and store copy — design/marketing.** Required per device
   class. Not an engineering blocker but it does gate submission.

## Sequence once the Apple account exists

```bash
cd mobile
bun install
bun run validate:store            # config guard must pass first

eas login
eas build:configure
eas credentials                   # let EAS manage certs + profiles

eas build --profile preview  --platform ios   # internal / TestFlight
eas build --profile production --platform ios
eas submit --profile production --platform ios
```

`expo prebuild` is intentionally **not** run as a committed step: `ios/` is
gitignored because it is generated. Configure through `app.json` and config
plugins so the native project stays reproducible.

## Privacy manifest — verify before submitting

`app.json` declares four required-reason APIs (`UserDefaults` CA92.1,
`FileTimestamp` C617.1, `SystemBootTime` 35F9.1, `DiskSpace` E174.1). These
cover React Native and Expo core. **Re-verify against the final dependency
set** before submitting: every third-party SDK added later can introduce a new
required-reason API, and an undeclared one fails App Store validation.

Nutrition labels are entered in App Store Connect, not here. For this app the
honest answers are account information, financial information, and photos
(KYC documents and payment proofs), all linked to identity, none used for
tracking.

## Testing that must happen on a signed build

The shell has never been run on iOS, so treat the first TestFlight round as
real testing rather than a formality.

- Session persistence across cold restarts (WKWebView cookie store is not the
  same as mobile Safari's — this is the single most likely thing to be broken).
- KYC camera and selfie capture end to end, including the permission prompt.
- Socket.IO trade chat receiving live updates while backgrounded and resumed.
- Proof-of-payment upload from both camera and photo library.
- External links (`wa.me`, bank sites, block explorers) opening in Safari
  rather than inside the shell.
- Swipe-back gesture navigating WebView history and not exiting at the root.
- Cold start to interactive WebView under 3s on a mid-tier device.
- Device matrix from iPhone SE through Pro Max, iOS 16+.

Capture the TestFlight crash-free rate before promoting to review.
