/**
 * Structured analytics + crash reporting.
 * Sentry is initialised in app/_layout.tsx.
 * This module re-exports the Sentry instance and GA4-parity trackEvent.
 *
 * Analytics events mirror web's GA4 event names + payloads exactly
 * (same event name, same param keys, extra platform: "mobile" discriminator).
 */
import * as SentryRN from "@sentry/react-native"
import { apiClient } from "@/src/lib/api/client"

// Re-export Sentry so callers don't need to import @sentry/react-native directly
export const Sentry = {
  captureException(err: unknown, context?: Record<string, unknown>) {
    if (__DEV__) {
      console.error("[Sentry]", err, context)
      return
    }
    if (context) {
      SentryRN.withScope((scope) => {
        scope.setContext("extra", context)
        SentryRN.captureException(err)
      })
    } else {
      SentryRN.captureException(err)
    }
  },

  captureMessage(msg: string, level: SentryRN.SeverityLevel = "info") {
    if (__DEV__) {
      console.log(`[Sentry ${level}]`, msg)
      return
    }
    SentryRN.captureMessage(msg, level)
  },

  setUser(user: { id: string; email?: string }) {
    SentryRN.setUser(user)
  },

  clearUser() {
    SentryRN.setUser(null)
  },

  addBreadcrumb(crumb: SentryRN.Breadcrumb) {
    SentryRN.addBreadcrumb(crumb)
  },
}

/** GA4-parity event union — mirrors web's analytics event catalogue */
export type AnalyticsEvent =
  | { name: "login"; method: "email" | "apple" | "google" }
  | { name: "sign_up"; method: "email" | "apple" | "google" }
  | { name: "logout" }
  | { name: "trade_initiated"; currency: string; amount: string; offerId: string }
  | { name: "trade_paid"; tradeId: string }
  | { name: "trade_released"; tradeId: string }
  | { name: "trade_cancelled"; tradeId: string; reason: string }
  | { name: "trade_disputed"; tradeId: string }
  | { name: "trade_completed"; tradeId: string; currency: string; amount: string }
  | { name: "withdrawal_submitted"; currency: string; amount: string; network: string }
  | { name: "deposit_address_viewed"; currency: string; network: string }
  | { name: "offer_created"; offerType: string; currency: string }
  | { name: "offer_edited"; offerId: string }
  | { name: "offer_deactivated"; offerId: string }
  | { name: "kyc_started"; tier: number; provider: string }
  | { name: "kyc_completed"; tier: number }
  | { name: "resell_created"; offerId: string; markup: number }
  | { name: "screen_view"; screen_name: string }

export function trackEvent(event: AnalyticsEvent) {
  if (__DEV__) {
    console.log("[Analytics]", event)
    return
  }

  // Add breadcrumb to Sentry for context on crashes
  SentryRN.addBreadcrumb({ category: "analytics", message: event.name, data: event, level: "info" })

  // KNOWN GAP (parity audit 2026-06-11): the backend has no /analytics/event
  // route (web uses GA4/GTM directly, not a backend relay). Events are kept as
  // Sentry breadcrumbs only until a backend endpoint or mobile GA4 SDK lands.
}
