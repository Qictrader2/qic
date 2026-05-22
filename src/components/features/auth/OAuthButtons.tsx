import { Platform, View, Text, TouchableOpacity, ActivityIndicator } from "react-native"
import { useRouter } from "expo-router"
import { useState } from "react"
import { apiClient, ApiError } from "@/src/lib/api/client"
import { useAuthStore } from "@/src/store/auth-store"

interface OAuthResponse {
  accessToken: string
  refreshToken: string
  user: {
    uid: string
    email: string
    username: string | null
    displayName: string | null
    emailVerified: boolean
    role: "user" | "admin" | "support" | "moderator" | null
    kycTier: number
  }
}

export function AppleSignInButton() {
  const { login } = useAuthStore()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  if (Platform.OS !== "ios") return null

  async function handleAppleSignIn() {
    setLoading(true)
    setError(null)
    try {
      const AppleAuthentication = require("expo-apple-authentication")
      const credential = await AppleAuthentication.signInAsync({
        requestedScopes: [
          AppleAuthentication.AppleAuthenticationScope.FULL_NAME,
          AppleAuthentication.AppleAuthenticationScope.EMAIL,
        ],
      })

      const res = await apiClient.post<OAuthResponse>("/api/v1/auth/oauth/apple", {
        identityToken: credential.identityToken,
        fullName: credential.fullName,
      })

      await login(res.accessToken, res.refreshToken, {
        uid: res.user.uid,
        email: res.user.email,
        username: res.user.username,
        displayName: res.user.displayName,
        emailVerified: res.user.emailVerified,
        role: res.user.role,
        kycTier: res.user.kycTier ?? 0,
      })
    } catch (err) {
      const e = err as { code?: string }
      if (e?.code === "ERR_REQUEST_CANCELED") return
      const apiErr = err as ApiError
      if (apiErr?.kind === "validation") {
        setError(Object.values(apiErr.fields)[0] ?? "Apple sign-in failed")
      } else {
        setError("Apple sign-in failed. Please try again.")
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <View>
      {error ? (
        <View className="mb-3 rounded-lg bg-error-bg px-4 py-2">
          <Text className="text-xs text-error text-center">{error}</Text>
        </View>
      ) : null}
      <TouchableOpacity
        onPress={handleAppleSignIn}
        disabled={loading}
        className="flex-row items-center justify-center rounded-lg bg-black dark:bg-white py-3.5 px-4 gap-3"
        accessibilityLabel="Sign in with Apple"
        activeOpacity={0.85}
      >
        {loading ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <>
            <Text className="text-white dark:text-black text-lg">🍎</Text>
            <Text className="text-base font-medium text-white dark:text-black">
              Sign in with Apple
            </Text>
          </>
        )}
      </TouchableOpacity>
    </View>
  )
}

export function GoogleSignInButton() {
  const { login } = useAuthStore()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleGoogleSignIn() {
    setLoading(true)
    setError(null)
    try {
      const { GoogleSignin } = require("@react-native-google-signin/google-signin")
      GoogleSignin.configure({
        webClientId: process.env.EXPO_PUBLIC_GOOGLE_WEB_CLIENT_ID,
        iosClientId: process.env.EXPO_PUBLIC_GOOGLE_IOS_CLIENT_ID,
      })

      await GoogleSignin.hasPlayServices()
      const { data } = await GoogleSignin.signIn()

      if (!data?.idToken) throw new Error("no id token")

      const res = await apiClient.post<OAuthResponse>("/api/v1/auth/oauth/google", {
        idToken: data.idToken,
      })

      await login(res.accessToken, res.refreshToken, {
        uid: res.user.uid,
        email: res.user.email,
        username: res.user.username,
        displayName: res.user.displayName,
        emailVerified: res.user.emailVerified,
        role: res.user.role,
        kycTier: res.user.kycTier ?? 0,
      })
    } catch (err) {
      const e = err as { code?: string }
      if (e?.code === "SIGN_IN_CANCELLED") return
      setError("Google sign-in failed. Please try again.")
    } finally {
      setLoading(false)
    }
  }

  return (
    <View>
      {error ? (
        <View className="mb-3 rounded-lg bg-error-bg px-4 py-2">
          <Text className="text-xs text-error text-center">{error}</Text>
        </View>
      ) : null}
      <TouchableOpacity
        onPress={handleGoogleSignIn}
        disabled={loading}
        className="flex-row items-center justify-center rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark py-3.5 px-4 gap-3"
        accessibilityLabel="Sign in with Google"
        activeOpacity={0.85}
      >
        {loading ? (
          <ActivityIndicator color="#00A3F6" />
        ) : (
          <>
            <Text className="text-lg">G</Text>
            <Text className="text-base font-medium text-foreground dark:text-foreground-dark">
              Sign in with Google
            </Text>
          </>
        )}
      </TouchableOpacity>
    </View>
  )
}
