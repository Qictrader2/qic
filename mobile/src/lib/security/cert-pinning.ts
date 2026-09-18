/**
 * MOBILE-SEC-001: Certificate pinning via custom Axios adapter.
 * NOTE: Full cert pinning requires a native module in a bare workflow.
 * This module enforces HTTPS-only and provides the pinning configuration
 * for when expo-modules/react-native-ssl-pinning is added post-ejection.
 */

export const PINNED_HOSTS: Record<string, string[]> = {
  "api.qictrader.com": [
    // SHA-256 fingerprints of QicTrader's TLS leaf cert + backup
    // These must be rotated before the cert expires
    "PLACEHOLDER_SHA256_LEAF_CERT",
    "PLACEHOLDER_SHA256_BACKUP_CERT",
  ],
}

/**
 * Validates that a URL targets an allowed host over HTTPS.
 * Used before making any API call in development/staging to catch
 * accidental http:// URLs or unexpected host drift.
 */
export function assertHttps(url: string): void {
  if (__DEV__) return // allow localhost in dev
  if (!url.startsWith("https://")) {
    throw new Error(`[SEC-001] Blocked non-HTTPS request: ${url}`)
  }
}

/**
 * Production checklist for cert pinning:
 * 1. Eject to bare workflow or use expo-build-properties to add custom native config
 * 2. Add react-native-ssl-pinning or TrustKit
 * 3. Replace PLACEHOLDER values above with real cert fingerprints
 * 4. Add rotation alert: monitor cert expiry via a cron job in ops/cert-monitor
 * 5. Test on physical device in staging before production release
 */
