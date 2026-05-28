import { View, Text, TouchableOpacity, ActivityIndicator } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import { apiClient } from "@/src/lib/api/client"

export default function VerifyEmailScreen() {
  const { token } = useLocalSearchParams<{ token?: string }>()
  const router = useRouter()
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle")

  async function verify() {
    if (!token) return
    setStatus("loading")
    try {
      await apiClient.post("/api/v1/auth/verify-email", { token })
      setStatus("success")
    } catch {
      setStatus("error")
    }
  }

  // Auto-verify if token is in the URL
  useState(() => { if (token) verify() })

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center px-6">
      {status === "loading" ? (
        <ActivityIndicator color="#00A3F6" />
      ) : status === "success" ? (
        <>
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-2">Email verified ✓</Text>
          <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center">
            Your email has been verified. You can now sign in.
          </Text>
          <TouchableOpacity
            onPress={() => router.replace("/(auth)/login")}
            className="w-full rounded-lg bg-brand py-4 items-center"
          >
            <Text className="text-base font-semibold text-white">Sign in</Text>
          </TouchableOpacity>
        </>
      ) : status === "error" ? (
        <>
          <Text className="text-2xl font-bold text-error mb-2">Verification failed</Text>
          <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center">
            This link may have expired. Request a new verification email from the sign-in screen.
          </Text>
          <TouchableOpacity
            onPress={() => router.replace("/(auth)/login")}
            className="w-full rounded-lg border border-brand py-4 items-center"
          >
            <Text className="text-base font-medium text-brand">Back to sign in</Text>
          </TouchableOpacity>
        </>
      ) : (
        <>
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-2">Verify your email</Text>
          <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center">
            Click the link in your email to verify your account, or paste the token below.
          </Text>
          <TouchableOpacity
            onPress={() => router.replace("/(auth)/login")}
            className="w-full rounded-lg border border-border dark:border-border-dark py-4 items-center"
          >
            <Text className="text-base font-medium text-foreground dark:text-foreground-dark">Back to sign in</Text>
          </TouchableOpacity>
        </>
      )}
    </SafeAreaView>
  )
}
