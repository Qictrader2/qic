import { apiClient } from "@/src/lib/api/client"

/** MOBILE-LAUNCH-006: Crash reporting (Sentry) + structured analytics */

const isSentryAvailable = () => {
  try {
    require("@sentry/react-native")
    return true
  } catch {
    return false
  }
}

export const Sentry = {
  captureException(err: unknown, context?: Record<string, unknown>) {
    if (__DEV__) {
      console.error("[Sentry]", err, context)
      return
    }
    if (!isSentryAvailable()) return
    const S = require("@sentry/react-native")
    if (context) {
      S.withScope((scope: { setContext: (k: string, v: unknown) => void }) => {
        scope.setContext("extra", context)
        S.captureException(err)
      })
    } else {
      S.captureException(err)
    }
  },

  captureMessage(msg: string, level: "info" | "warning" | "error" = "info") {
    if (__DEV__) {
      console.log(`[Sentry ${level}]`, msg)
      return
    }
    if (!isSentryAvailable()) return
    const S = require("@sentry/react-native")
    S.captureMessage(msg, level)
  },

  setUser(user: { id: string; email?: string }) {
    if (!isSentryAvailable()) return
    const S = require("@sentry/react-native")
    S.setUser(user)
  },

  clearUser() {
    if (!isSentryAvailable()) return
    const S = require("@sentry/react-native")
    S.setUser(null)
  },
}

/** Analytics events — mirror web's GA4 event names exactly */
type AnalyticsEvent =
  | { name: "login"; method: "email" | "apple" | "google" }
  | { name: "sign_up"; method: "email" | "apple" | "google" }
  | { name: "trade_initiated"; currency: string; amount: string }
  | { name: "trade_paid"; tradeId: string }
  | { name: "trade_released"; tradeId: string }
  | { name: "trade_cancelled"; tradeId: string; reason: string }
  | { name: "trade_disputed"; tradeId: string }
  | { name: "withdrawal_submitted"; currency: string; amount: string }
  | { name: "deposit_address_viewed"; currency: string; network: string }
  | { name: "offer_created"; offerType: string; currency: string }
  | { name: "kyc_started"; tier: number; provider: string }
  | { name: "screen_view"; screen_name: string }

export function trackEvent(event: AnalyticsEvent) {
  if (__DEV__) {
    console.log("[Analytics]", event)
    return
  }
  // Fire to backend analytics endpoint (mirrors web's GA4 measurement protocol call)
  apiClient
    .post("/api/v1/analytics/event", {
      ...event,
      platform: "mobile",
      ts: Date.now(),
    })
    .catch(() => {})
}
