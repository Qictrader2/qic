import { useEffect } from "react"
import { useRouter, useSegments } from "expo-router"
import { useAuthStore } from "@/src/store/auth-store"
import { secureStorage } from "@/src/lib/storage/secure"
import { apiClient } from "@/src/lib/api/client"

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, login, logout } = useAuthStore()
  const segments = useSegments()
  const router = useRouter()

  // On mount: restore token from SecureStore and validate session
  useEffect(() => {
    async function restoreSession() {
      try {
        const accessToken = await secureStorage.get("qic_access")
        const refreshToken = await secureStorage.get("qic_refresh")
        if (!accessToken || !refreshToken) {
          await logout()
          return
        }
        apiClient.setToken(accessToken)
        // Validate token by hitting /me
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
      } catch {
        await logout()
      }
    }
    restoreSession()
  }, [])

  // Route guard: redirect based on auth state
  useEffect(() => {
    const inAuthGroup = segments[0] === "(auth)"
    if (!isAuthenticated && !inAuthGroup) {
      router.replace("/(auth)/login")
    } else if (isAuthenticated && inAuthGroup) {
      router.replace("/(tabs)")
    }
  }, [isAuthenticated, segments])

  return <>{children}</>
}
