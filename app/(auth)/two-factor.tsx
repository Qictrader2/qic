import { View, Text, TextInput, TouchableOpacity, ActivityIndicator } from "react-native"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useRef, useState } from "react"
import { apiClient, ApiError } from "@/src/lib/api/client"
import { useAuthStore } from "@/src/store/auth-store"

interface TwoFactorResponse {
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

export default function TwoFactorScreen() {
  const { twoFactorToken } = useLocalSearchParams<{ twoFactorToken: string }>()
  const router = useRouter()
  const { login } = useAuthStore()
  const [code, setCode] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [useBackupCode, setUseBackupCode] = useState(false)
  const inputRef = useRef<TextInput>(null)

  async function onSubmit() {
    if (code.length < (useBackupCode ? 8 : 6)) return
    setIsSubmitting(true)
    setError(null)
    try {
      const res = await apiClient.post<TwoFactorResponse>("/api/v1/auth/2fa/verify-login", {
        twoFactorToken,
        code: code.trim(),
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
      const apiErr = err as ApiError
      if (apiErr.kind === "unauthorized" || apiErr.kind === "validation") {
        setError("Invalid code. Please try again.")
      } else {
        setError("Something went wrong. Please try again.")
      }
      setCode("")
      inputRef.current?.focus()
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <View className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
      <View className="mb-8">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-2">
          Two-factor authentication
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark">
          {useBackupCode
            ? "Enter one of your 8-character backup codes"
            : "Enter the 6-digit code from your authenticator app"}
        </Text>
      </View>

      {error ? (
        <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
          <Text className="text-sm text-error">{error}</Text>
        </View>
      ) : null}

      <TextInput
        ref={inputRef}
        className="mb-6 rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-4 text-2xl text-center tracking-widest text-foreground dark:text-foreground-dark"
        value={code}
        onChangeText={(v) => {
          setCode(v)
          if (!useBackupCode && v.length === 6) {
            setTimeout(onSubmit, 100)
          }
        }}
        placeholder={useBackupCode ? "XXXXXXXX" : "000000"}
        placeholderTextColor="#94A3B8"
        keyboardType={useBackupCode ? "default" : "number-pad"}
        maxLength={useBackupCode ? 8 : 6}
        autoFocus
        editable={!isSubmitting}
      />

      <TouchableOpacity
        onPress={onSubmit}
        disabled={isSubmitting || code.length < (useBackupCode ? 8 : 6)}
        className="rounded-lg bg-brand py-4 items-center mb-4"
        activeOpacity={0.8}
      >
        {isSubmitting ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text className="text-base font-semibold text-white">Verify</Text>
        )}
      </TouchableOpacity>

      <TouchableOpacity
        onPress={() => { setUseBackupCode((v) => !v); setCode("") }}
        className="py-3 items-center"
      >
        <Text className="text-sm text-brand">
          {useBackupCode ? "Use authenticator app instead" : "Use a backup code"}
        </Text>
      </TouchableOpacity>

      <TouchableOpacity onPress={() => router.replace("/(auth)/login")} className="py-3 items-center">
        <Text className="text-sm text-muted dark:text-muted-dark">Back to sign in</Text>
      </TouchableOpacity>
    </View>
  )
}
