import { View, Text, TouchableOpacity } from "react-native"
import { useRouter } from "expo-router"
import { apiClient, ApiError } from "@/src/lib/api/client"
import { useState } from "react"

export default function VerifyEmailPendingScreen() {
  const router = useRouter()
  const [resent, setResent] = useState(false)
  const [resending, setResending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleResend() {
    setResending(true)
    setError(null)
    try {
      await apiClient.post("/api/v1/auth/resend-verification")
      setResent(true)
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "rate_limited") {
        setError("Please wait before requesting another email.")
      } else {
        setError("Failed to resend. Please try again.")
      }
    } finally {
      setResending(false)
    }
  }

  return (
    <View className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
      <View className="mb-8">
        <Text className="text-4xl mb-4">📧</Text>
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-2">
          Verify your email
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark leading-relaxed">
          We sent a verification link to your email. Click it to activate your account.
        </Text>
      </View>

      {resent ? (
        <View className="mb-4 rounded-lg bg-success-bg px-4 py-3">
          <Text className="text-sm text-success">Verification email resent!</Text>
        </View>
      ) : null}

      {error ? (
        <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
          <Text className="text-sm text-error">{error}</Text>
        </View>
      ) : null}

      <TouchableOpacity
        onPress={handleResend}
        disabled={resending || resent}
        className="rounded-lg border border-brand py-4 items-center mb-4"
        activeOpacity={0.8}
      >
        <Text className="text-base font-medium text-brand">
          {resending ? "Sending…" : resent ? "Email sent" : "Resend email"}
        </Text>
      </TouchableOpacity>

      <TouchableOpacity
        onPress={() => router.replace("/(auth)/login")}
        className="py-4 items-center"
      >
        <Text className="text-sm text-muted dark:text-muted-dark">Back to sign in</Text>
      </TouchableOpacity>
    </View>
  )
}
