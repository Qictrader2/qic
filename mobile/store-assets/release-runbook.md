# MOBILE-LAUNCH-007: Release Runbook

Pre-flight checklist and step-by-step instructions for every production release.

---

## Pre-release checklist

### Code quality
- [ ] `bun run typecheck` passes with zero errors
- [ ] `bun run lint` passes with zero warnings
- [ ] `bun test` passes (all unit + component tests green)
- [ ] `maestro test .maestro/` critical paths pass on simulator
- [ ] Sentry DSN is set in EAS Secrets (`SENTRY_DSN`)
- [ ] `EXPO_PUBLIC_API_URL` points to production (`https://api.qictrader.com`)

### Sync check
- [ ] `./scripts/sync-from-web.sh --check` passes (no drift from web types)

### Version bump
- [ ] `app.json` → `version` incremented (semver: major.minor.patch)
- [ ] `ios.buildNumber` and `android.versionCode` are set to `remote` (EAS auto-increments)
- [ ] Git tag created: `git tag mobile-v{version} && git push --tags`

### App Store Connect
- [ ] New version created in App Store Connect
- [ ] Release notes drafted (`store-assets/app-store-metadata.md` → "What's New")
- [ ] Screenshots updated if any UI changed
- [ ] Age rating, privacy details, data safety confirmed current

### Google Play Console
- [ ] New internal release created in Play Console
- [ ] Release notes drafted (`store-assets/play-store-metadata.md` → "Release Notes")
- [ ] Data safety form reviewed for any new data collection

---

## Build + submit commands

```bash
# 1. Pull latest, sync, verify
git pull --rebase origin main
./scripts/sync-from-web.sh
bun install
bun run typecheck
bun run lint
bun test

# 2. Tag the release
git tag mobile-v$(cat app.json | python3 -c "import sys,json; print(json.load(sys.stdin)['expo']['version'])")
git push --tags

# 3. Build both platforms
eas build --profile production --platform all --non-interactive

# 4. Wait for builds to complete (check EAS dashboard)
# iOS: https://expo.dev/accounts/qictrader/projects/qictrader/builds
# Android: same

# 5. Submit
eas submit --profile production --platform ios --latest
eas submit --profile production --platform android --latest
```

---

## Post-submit

### iOS App Store
- Apple review typically 24–48h for standard review
- Expedited review available via App Store Connect for critical fixes
- Phased release: Apple manages 7-day rollout automatically
- Monitor Sentry crash rate in first 24h post-rollout
- If crash rate > 1%: pause in App Store Connect → "Pause Phased Release"

### Google Play
- After approval, set staged rollout: **10%** first
- Monitor for 24h → if clean, increase to **25% → 50% → 100%**
- Rollout pause: Play Console → Release → Pause rollout

---

## OTA (EAS Update) — JS-only fixes

Use for non-native code fixes only. Cannot update native modules.

```bash
# Verify on staging branch first
eas update --branch staging --message "MOBILE-XXX: describe fix"
# Test on a staging build

# Then push to production
eas update --branch production --message "MOBILE-XXX: describe fix"
```

Rollback: `eas update --branch production --republish --group {previous-update-group-id}`

---

## Rollback (native build)

### iOS
1. Log into App Store Connect
2. Previous version → "Request Expedited Review" or re-submit previous .ipa via EAS
3. Apple approves hotfix builds within hours for critical bugs

### Android
1. Play Console → Release → Managed publishing
2. Halt current staged rollout
3. Promote previous release from internal track to production

---

## Monitoring post-release

1. **Sentry**: `https://sentry.io/organizations/qictrader/projects/qictrader-mobile/`
   — Alert thresholds: crash-free rate < 99.5% triggers immediate review
2. **EAS dashboard**: build logs, OTA delivery stats
3. **App Store Connect**: crash reports, ratings, TestFlight feedback
4. **Play Console**: Android Vitals, crash clusters, ANR rate (target < 0.47%)

---

## First-time store setup checklist (LAUNCH-001 through 006)

### Apple (LAUNCH-001)
- [ ] App created in App Store Connect (`com.qictrader.app`)
- [ ] Distribution certificate created (via `eas credentials`)
- [ ] Push notification certificate (APNs) created (via `eas credentials`)
- [ ] App Privacy details filled
- [ ] App Review contact set

### Google (LAUNCH-002)
- [ ] App created in Google Play Console (`com.qictrader.app`)
- [ ] Upload keystore configured (via `eas credentials`)
- [ ] FCM project configured: `google-services.json` added to EAS Secrets
- [ ] Data safety form completed

### Sentry (LAUNCH-003)
- [ ] Project created at sentry.io
- [ ] `SENTRY_DSN` added to EAS Secrets (all profiles)
- [ ] `SENTRY_AUTH_TOKEN` added to EAS Secrets (for source map upload)
- [ ] Alerting rules: crash rate threshold set

### Analytics (LAUNCH-004)
- [ ] GA4 property created for mobile
- [ ] Mobile stream configured
- [ ] `EXPO_PUBLIC_GA_MEASUREMENT_ID` set in EAS Secrets

### App Store screenshots (LAUNCH-005)
- [ ] 5 screenshots for iPhone 6.9"
- [ ] 5 screenshots for iPhone 6.5"
- [ ] App preview video (optional)
- [ ] All in `store-assets/screenshots/` and uploaded to App Store Connect

### Play Store screenshots (LAUNCH-006)
- [ ] 5 screenshots for phone (portrait)
- [ ] Feature graphic (1024×500)
- [ ] All in `store-assets/screenshots/android/` and uploaded to Play Console
