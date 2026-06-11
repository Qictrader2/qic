import * as Notifications from "expo-notifications"
import * as Device from "expo-device"
import { Platform } from "react-native"
import { apiClient } from "@/src/lib/api/client"

Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowAlert: true,
    shouldPlaySound: true,
    shouldSetBadge: true,
    shouldShowBanner: true,
    shouldShowList: true,
  }),
})

export async function registerForPushNotifications(): Promise<string | null> {
  if (!Device.isDevice) {
    return null
  }

  const { status: existing } = await Notifications.getPermissionsAsync()
  let finalStatus = existing

  if (existing !== "granted") {
    const { status } = await Notifications.requestPermissionsAsync()
    finalStatus = status
  }

  if (finalStatus !== "granted") {
    return null
  }

  if (Platform.OS === "android") {
    await Notifications.setNotificationChannelAsync("default", {
      name: "Default",
      importance: Notifications.AndroidImportance.MAX,
      vibrationPattern: [0, 250, 250, 250],
    })

    await Notifications.setNotificationChannelAsync("trades", {
      name: "Trade alerts",
      importance: Notifications.AndroidImportance.HIGH,
      vibrationPattern: [0, 250],
    })
  }

  const projectId = process.env.EXPO_PUBLIC_EAS_PROJECT_ID
  const token = await Notifications.getExpoPushTokenAsync(
    projectId ? { projectId } : undefined,
  )

  return token.data
}

// KNOWN GAP (parity audit 2026-06-11): the backend has no
// /notifications/register-device or /notifications/unregister-device routes —
// push delivery is mobile-only and needs a backend device registry first.
// These are explicit no-ops (with a dev warning) until that endpoint exists,
// so login/logout flows don't fire requests that are guaranteed to 404.
export async function syncPushToken(token: string): Promise<void> {
  if (__DEV__) {
    console.warn(
      `[push] device token registration skipped — backend endpoint not implemented (token: ${token.slice(0, 12)}…)`,
    )
  }
}

export async function unregisterPushToken(): Promise<void> {
  if (__DEV__) {
    console.warn("[push] device token unregistration skipped — backend endpoint not implemented")
  }
}
