import { useEffect, useState } from "react"
import { useRouter, useSegments } from "expo-router"
import { useAuthStore } from "@/src/store/auth-store"
import { secureStorage } from "@/src/lib/storage/secure"
import { apiClient } from "@/src/lib/api/client"
import { isBiometricEnabled, isBiometricAvailable, promptBiometric } from "@/src/lib/biometric"
import { useSessionLifecycle } from "@/src/hooks/use-session-lifecycle"
import { registerForPushNotifications, syncPushToken } from "@/src/lib/push-notifications"

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, login, logout } = useAuthStore()
  const segments = useSegments()
  const router = useRouter()
  const [bootDone, setBootDone] = useState(false)

  useSessionLifecycle()

  useEffect(() => {
    restoreSession()
  }, [])

  async function restoreSession() {
    try {
      const accessToken = await secureStorage.get("qic_access")
      const refreshToken = await secureStorage.get("qic_refresh")

      if (!accessToken || !refreshToken) {
        await logout()
        setBootDone(true)
        return
      }

      // Check biometric gate
      const biometricEnabled = await isBiometricEnabled()
      const biometricAvailable = await isBiometricAvailable()

      if (biometricEnabled && biometricAvailable) {
        const passed = await promptBiometric("Sign in to QicTrader")
        if (!passed) {
          await logout()
          setBootDone(true)
          return
        }
      }

      apiClient.setToken(accessToken)

      const user = await apiClient.get<{
        uid: string
        email: string
        username: string | null
        displayName: string | null
        emailVerified: boolean
        role: "user" | "admin" | "support" | "moderator" | null
        kycTier: number
      }>("/api/v1/me")

      await login(accessToken, refreshToken, {
        uid: user.uid,
        email: user.email,
        username: user.username,
        displayName: user.displayName,
        emailVerified: user.emailVerified,
        role: user.role,
        kycTier: user.kycTier ?? 0,
      })

      // Register push token in background
      registerForPushNotifications()
        .then((token) => { if (token) return syncPushToken(token) })
        .catch(() => {})
    } catch {
      await logout()
    } finally {
      setBootDone(true)
    }
  }

  // Route guard
  useEffect(() => {
    if (!bootDone) return
    const inAuthGroup = segments[0] === "(auth)"
    if (!isAuthenticated && !inAuthGroup) {
      router.replace("/(auth)/login")
    } else if (isAuthenticated && inAuthGroup) {
      router.replace("/(tabs)")
    }
  }, [isAuthenticated, segments, bootDone])

  if (!bootDone) return null

  return <>{children}</>
}
