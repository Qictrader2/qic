import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
  Image,
  ScrollView,
} from "react-native"
import { Link, useRouter } from "expo-router"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { CheckCircle2 } from "lucide-react-native"
import { apiClient, ApiError } from "@/src/lib/api/client"

const schema = z.object({
  email: z.string().email("Enter a valid email address"),
})

type Form = z.infer<typeof schema>

export default function ForgotPasswordScreen() {
  const router = useRouter()
  const [sent, setSent] = useState(false)
  const [serverError, setServerError] = useState<string | null>(null)

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await apiClient.post("/api/v1/auth/forgot-password", data)
      setSent(true)
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "network") {
        setServerError("Cannot connect to server.")
      } else {
        setSent(true)
      }
    }
  }

  if (sent) {
    return (
      <SafeAreaScreen>
        <View className="flex-1 px-6 items-center justify-center">
          <Image
            source={require("@/assets/images/logo.png")}
            style={{ width: 140, height: 48, marginBottom: 32 }}
            resizeMode="contain"
          />
          <View className="w-16 h-16 rounded-full bg-success-bg items-center justify-center mb-4">
            <CheckCircle2 size={32} color="#10B981" />
          </View>
          <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark mb-2">
            Check your email
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center max-w-[280px]">
            If that email is registered, you&apos;ll receive a password reset link within a few minutes.
          </Text>
          <TouchableOpacity
            onPress={() => router.replace("/(auth)/login")}
            className="w-full rounded-xl bg-brand py-4 items-center"
            activeOpacity={0.85}
          >
            <Text className="text-base font-semibold text-white">Back to sign in</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaScreen>
    )
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <ScrollView contentContainerStyle={{ flexGrow: 1 }} keyboardShouldPersistTaps="handled">
        <View className="flex-1 justify-center px-6 py-12">
          <View className="items-center mb-8">
            <Image
              source={require("@/assets/images/logo.png")}
              style={{ width: 160, height: 54 }}
              resizeMode="contain"
            />
          </View>

          <View className="mb-8 items-center">
            <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark">
              Reset your password
            </Text>
            <Text className="mt-2 text-sm text-muted dark:text-muted-dark text-center max-w-[280px]">
              Enter the email you signed up with and we&apos;ll send a reset link.
            </Text>
          </View>

          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          <View className="mb-6">
            <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
              Email address
            </Text>
            <Controller
              control={control}
              name="email"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="h-12 rounded-lg border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 text-base text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  placeholder="you@example.com"
                  placeholderTextColor="#94A3B8"
                  autoCapitalize="none"
                  keyboardType="email-address"
                  textContentType="emailAddress"
                  editable={!isSubmitting}
                />
              )}
            />
            {errors.email ? (
              <Text className="mt-1 text-xs text-error">{errors.email.message}</Text>
            ) : null}
          </View>

          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="h-12 rounded-lg bg-brand items-center justify-center"
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Send reset link</Text>
            )}
          </TouchableOpacity>

          <View className="mt-6 items-center">
            <Link href="/(auth)/login" asChild>
              <TouchableOpacity>
                <Text className="text-sm text-brand">Back to sign in</Text>
              </TouchableOpacity>
            </Link>
          </View>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  )
}

function SafeAreaScreen({ children }: { children: React.ReactNode }) {
  const { SafeAreaView } = require("react-native-safe-area-context")
  return <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">{children}</SafeAreaView>
}
