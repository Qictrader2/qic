import { View, Text, TouchableOpacity, ActivityIndicator, Image } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useRouter } from "expo-router"
import { useState } from "react"
import { Mail, CheckCircle2 } from "lucide-react-native"
import { apiClient, ApiError } from "@/src/lib/api/client"

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
        setError("Please wait a few minutes before requesting another email.")
      } else {
        setError("Failed to resend. Please try again.")
      }
    } finally {
      setResending(false)
    }
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="flex-1 px-6 items-center justify-center">
        <Image
          source={require("@/assets/images/logo.png")}
          style={{ width: 140, height: 48, marginBottom: 32 }}
          resizeMode="contain"
        />

        <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
          <Mail size={28} color="#00A3F6" />
        </View>
        <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark mb-2 text-center">
          Verify your email
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center max-w-[320px] leading-5">
          We sent a verification link to your inbox. Click it to activate your account and start trading.
        </Text>

        {resent ? (
          <View className="w-full mb-4 rounded-xl bg-success/10 border border-success/20 px-4 py-3 flex-row items-center gap-2">
            <CheckCircle2 size={14} color="#10B981" />
            <Text className="text-sm text-success flex-1">Verification email resent</Text>
          </View>
        ) : null}

        {error ? (
          <View className="w-full mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3">
            <Text className="text-sm text-error">{error}</Text>
          </View>
        ) : null}

        <TouchableOpacity
          onPress={handleResend}
          disabled={resending || resent}
          className={`w-full rounded-xl h-12 items-center justify-center border ${
            resent ? "border-success bg-success/10" : "border-brand bg-brand/5"
          }`}
          activeOpacity={0.85}
        >
          {resending ? (
            <ActivityIndicator color="#00A3F6" />
          ) : (
            <Text
              className={`text-base font-semibold ${
                resent ? "text-success" : "text-brand"
              }`}
            >
              {resent ? "Email sent" : "Resend email"}
            </Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          onPress={() => router.replace("/(auth)/login" as never)}
          className="mt-4 py-3"
          activeOpacity={0.7}
        >
          <Text className="text-sm font-medium text-muted dark:text-muted-dark">Back to sign in</Text>
        </TouchableOpacity>
      </View>
    </SafeAreaView>
  )
}
