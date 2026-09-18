/**
 * MOBILE-NOTIF-003: Deep linking + background notification handling.
 *
 * Expo Router handles URL-based deep links automatically via the scheme
 * defined in app.json ("qictrader://"). This module handles:
 * 1. Notification tap → navigate to correct screen
 * 2. Background notification data processing
 */

import * as Notifications from "expo-notifications"
import { router } from "expo-router"
import { useEffect } from "react"
import { useQueryClient } from "@tanstack/react-query"

type NotifData = {
  type: "trade_update" | "payment_received" | "offer_matched" | "dispute_update" | "kyc_update"
  tradeId?: string
  offerId?: string
}

function navigateFromNotification(data: NotifData) {
  switch (data.type) {
    case "trade_update":
    case "payment_received":
    case "dispute_update":
      if (data.tradeId) {
        router.push({ pathname: "/(app)/trade/[id]", params: { id: data.tradeId } })
      }
      break
    case "offer_matched":
      if (data.offerId) {
        router.push({ pathname: "/(app)/offer/[id]", params: { id: data.offerId } })
      } else {
        router.push("/(tabs)/marketplace")
      }
      break
    case "kyc_update":
      router.push("/(app)/kyc")
      break
    default: {
      const _exhaustive: never = data.type
      void _exhaustive
    }
  }
}

/**
 * Hook: subscribes to notification interactions and routes accordingly.
 * Mount once at the root of the authenticated app.
 */
export function useNotificationDeepLink() {
  const qc = useQueryClient()

  useEffect(() => {
    // Handle notification tap when app was in foreground or background
    const responseSub = Notifications.addNotificationResponseReceivedListener((response) => {
      const data = response.notification.request.content.data as NotifData
      if (data?.type) navigateFromNotification(data)
      // Invalidate relevant query caches
      qc.invalidateQueries({ queryKey: ["notifications"] })
    })

    // Handle notification received in foreground — refresh caches
    const receivedSub = Notifications.addNotificationReceivedListener((notification) => {
      const data = notification.request.content.data as NotifData
      if (data?.type === "trade_update" || data?.type === "payment_received") {
        qc.invalidateQueries({ queryKey: ["trades-active"] })
      }
      if (data?.tradeId) {
        qc.invalidateQueries({ queryKey: ["trade", data.tradeId] })
        qc.invalidateQueries({ queryKey: ["trade-messages", data.tradeId] })
      }
      qc.invalidateQueries({ queryKey: ["notifications"] })
    })

    return () => {
      responseSub.remove()
      receivedSub.remove()
    }
  }, [])
}

/**
 * Returns the initial notification that launched the app (cold start).
 * Call at app boot to handle taps on notifications received while app was killed.
 */
export async function handleInitialNotification(): Promise<void> {
  const response = await Notifications.getLastNotificationResponseAsync()
  if (!response) return
  const data = response.notification.request.content.data as NotifData
  if (data?.type) navigateFromNotification(data)
}

/**
 * Deep link URL mapping (for reference — Expo Router handles these automatically):
 * qictrader://trade/[id]          → /(app)/trade/[id]
 * qictrader://offer/[id]          → /(app)/offer/[id]
 * qictrader://kyc                 → /(app)/kyc
 * qictrader://wallet              → /(tabs)/ (wallet tab)
 * qictrader://marketplace         → /(tabs)/marketplace
 */
