import { useEffect, useRef } from "react"
import { AppState, AppStateStatus } from "react-native"
import { useAuthStore } from "@/src/store/auth-store"
import { apiClient } from "@/src/lib/api/client"
import { secureStorage } from "@/src/lib/storage/secure"

const SESSION_CHECK_INTERVAL_MS = 5 * 60 * 1000 // 5 minutes

/**
 * Runs periodic /me checks to detect server-side session revocation.
 * Must be mounted inside AuthProvider when user is authenticated.
 */
export function useSessionLifecycle() {
  const { isAuthenticated, logout, setUser } = useAuthStore()
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const appStateRef = useRef<AppStateStatus>("active")

  async function validateSession() {
    if (!isAuthenticated) return
    try {
      const user = await apiClient.get<{
        uid: string
        email: string
        username: string | null
        displayName: string | null
        emailVerified: boolean
        role: "user" | "admin" | "support" | "moderator" | null
        kycTier: number
      }>("/api/v1/users/me")
      setUser({
        uid: user.uid,
        email: user.email,
        username: user.username,
        displayName: user.displayName,
        emailVerified: user.emailVerified,
        role: user.role,
        kycTier: user.kycTier ?? 0,
      })
    } catch (err) {
      const apiErr = err as { kind?: string }
      if (apiErr?.kind === "unauthorized") {
        await logout()
      }
    }
  }

  useEffect(() => {
    if (!isAuthenticated) return

    // Start periodic check
    intervalRef.current = setInterval(validateSession, SESSION_CHECK_INTERVAL_MS)

    // On foreground from background: re-validate immediately
    const sub = AppState.addEventListener("change", (next) => {
      if (appStateRef.current !== "active" && next === "active") {
        validateSession()
      }
      appStateRef.current = next
    })

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current)
      sub.remove()
    }
  }, [isAuthenticated])
}

/**
 * Silently refreshes access token before it expires.
 */
export async function silentTokenRefresh(): Promise<boolean> {
  try {
    const refreshToken = await secureStorage.get("qic_refresh")
    if (!refreshToken) return false

    const res = await apiClient.post<{ accessToken: string }>("/api/v1/auth/refresh-token", {
      refreshToken,
    })

    await secureStorage.set("qic_access", res.accessToken)
    apiClient.setToken(res.accessToken)
    return true
  } catch {
    return false
  }
}
