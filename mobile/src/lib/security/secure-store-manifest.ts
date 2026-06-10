/**
 * MOBILE-SEC-003: SecureStore audit — catalogues every key stored in SecureStore
 * and the sensitivity classification. Referenced during security reviews.
 *
 * MOBILE-SEC-004: Session timeout + biometric re-auth + privacy screen.
 */

export const SECURE_STORE_MANIFEST = {
  qic_access: {
    sensitivity: "critical",
    description: "JWT access token. Allows API access. 15-min TTL.",
    rotatedOn: "every login",
    neverInLogs: true,
    neverInAsyncStorage: true,
  },
  qic_refresh: {
    sensitivity: "critical",
    description: "JWT refresh token. Used to renew access tokens. 30-day TTL.",
    rotatedOn: "every token refresh",
    neverInLogs: true,
    neverInAsyncStorage: true,
  },
  qic_csrf: {
    sensitivity: "high",
    description: "CSRF token sent with state-mutation requests.",
    rotatedOn: "every session",
    neverInLogs: true,
    neverInAsyncStorage: true,
  },
  qic_biometric_enabled: {
    sensitivity: "low",
    description: "Boolean flag — biometric login preference.",
    rotatedOn: "never",
    neverInLogs: false,
    neverInAsyncStorage: false,
  },
} as const

export type SecureKey = keyof typeof SECURE_STORE_MANIFEST

/**
 * SESSION_TIMEOUT_MS: After this period of inactivity, the user must
 * re-authenticate with biometric or password.
 * Matches web frontend's 30-minute session timeout.
 */
export const SESSION_TIMEOUT_MS = 30 * 60 * 1000 // 30 minutes

/**
 * SENSITIVE_SCREENS: Screen names that trigger privacy screen
 * (blur overlay) when app goes to background.
 * Implemented via AppState listener in AuthProvider.
 */
export const SENSITIVE_SCREENS = [
  "withdraw",
  "trade/[id]",
  "security-settings",
  "2fa-setup",
] as const
