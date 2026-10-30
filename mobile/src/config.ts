/**
 * Which QicTrader web deployment this shell renders.
 *
 * Override at build time with EXPO_PUBLIC_WEB_APP_URL, e.g.
 *   EXPO_PUBLIC_WEB_APP_URL=https://www.qictrader.com ./gradlew assembleRelease
 *
 * Default is STAGING per the current build target (Android APK against
 * staging creds). Switch the default to production only with JP sign-off.
 */
export const WEB_APP_URL =
  process.env.EXPO_PUBLIC_WEB_APP_URL ?? "https://staging.qictrader.com";

/**
 * Hosts that stay inside the app's WebView. Anything else (wa.me,
 * banking apps, block explorers, mailto:, tel:, intent: …) is handed to
 * the OS so the right native app opens — same behaviour a mobile
 * browser gives the web app.
 */
export const INTERNAL_HOSTS: RegExp[] = [
  /(^|\.)qictrader\.com$/i,
  // Heroku-hosted frontends (review apps / staging / prod fallbacks).
  /^qictrader-frontend[a-z0-9-]*\.herokuapp\.com$/i,
];

/**
 * URL fragments that indicate an identity-verification flow. When the
 * WebView navigates to one of these we pre-request the Android CAMERA /
 * RECORD_AUDIO runtime permissions so getUserMedia liveness checks
 * inside the page succeed on first try.
 */
export const KYC_PATH_HINTS =
  /kyc|verif|didit|sumsub|liveness|selfie|identity/i;

/** Appended to the WebView user agent so web analytics / support can
 *  distinguish the Android app from mobile Chrome. */
export const USER_AGENT_SUFFIX = "QicTraderApp/1.0 (Android)";
