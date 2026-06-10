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

export async function syncPushToken(token: string): Promise<void> {
  await apiClient.post("/api/v1/notifications/register-device", {
    token,
    platform: Platform.OS,
  })
}

export async function unregisterPushToken(): Promise<void> {
  const projectId = process.env.EXPO_PUBLIC_EAS_PROJECT_ID
  const token = await Notifications.getExpoPushTokenAsync(
    projectId ? { projectId } : undefined,
  ).catch(() => null)

  if (!token) return

  await apiClient.post("/api/v1/notifications/unregister-device", {
    token: token.data,
  }).catch(() => {})
}
