import { View, Text, TouchableOpacity, ActivityIndicator, Image } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useEffect, useState } from "react"
import { CheckCircle2, XCircle, Mail } from "lucide-react-native"
import { apiClient } from "@/src/lib/api/client"

export default function VerifyEmailScreen() {
  const { token } = useLocalSearchParams<{ token?: string }>()
  const router = useRouter()
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">(
    token ? "loading" : "idle",
  )

  useEffect(() => {
    if (!token) return
    apiClient
      .post("/api/v1/auth/verify-email", { token })
      .then(() => setStatus("success"))
      .catch(() => setStatus("error"))
  }, [token])

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="flex-1 px-6 items-center justify-center">
        <Image
          source={require("@/assets/images/logo.png")}
          style={{ width: 140, height: 48, marginBottom: 32 }}
          resizeMode="contain"
        />

        {status === "loading" ? (
          <>
            <ActivityIndicator color="#00A3F6" />
            <Text className="mt-3 text-sm text-muted dark:text-muted-dark">
              Verifying your email…
            </Text>
          </>
        ) : status === "success" ? (
          <>
            <View className="w-16 h-16 rounded-full bg-success-bg items-center justify-center mb-4">
              <CheckCircle2 size={32} color="#10B981" />
            </View>
            <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark mb-2">
              Email verified
            </Text>
            <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center max-w-[280px]">
              Your email has been verified. You can now sign in to your account.
            </Text>
            <TouchableOpacity
              onPress={() => router.replace("/(auth)/login")}
              className="w-full rounded-xl bg-brand py-4 items-center"
              activeOpacity={0.85}
            >
              <Text className="text-base font-semibold text-white">Sign in</Text>
            </TouchableOpacity>
          </>
        ) : status === "error" ? (
          <>
            <View className="w-16 h-16 rounded-full bg-error-bg items-center justify-center mb-4">
              <XCircle size={32} color="#EF4444" />
            </View>
            <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark mb-2">
              Verification failed
            </Text>
            <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center max-w-[280px]">
              This link may have expired. Request a new verification email from the sign-in screen.
            </Text>
            <TouchableOpacity
              onPress={() => router.replace("/(auth)/login")}
              className="w-full rounded-xl border border-brand py-4 items-center"
              activeOpacity={0.85}
            >
              <Text className="text-base font-medium text-brand">Back to sign in</Text>
            </TouchableOpacity>
          </>
        ) : (
          <>
            <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
              <Mail size={28} color="#00A3F6" />
            </View>
            <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark mb-2">
              Check your inbox
            </Text>
            <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center max-w-[280px]">
              We've sent you a verification link. Click it from this device to verify your email.
            </Text>
            <TouchableOpacity
              onPress={() => router.replace("/(auth)/login")}
              className="w-full rounded-xl border border-border dark:border-border-dark py-4 items-center"
              activeOpacity={0.85}
            >
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark">
                Back to sign in
              </Text>
            </TouchableOpacity>
          </>
        )}
      </View>
    </SafeAreaView>
  )
}
