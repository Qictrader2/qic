import { View, Text, TextInput, TouchableOpacity, ActivityIndicator } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState } from "react"
import { apiClient } from "@/src/lib/api/client"

const schema = z.object({
  password: z.string().min(8, "At least 8 characters"),
  confirmPassword: z.string().min(1, "Required"),
}).refine((d) => d.password === d.confirmPassword, {
  message: "Passwords do not match",
  path: ["confirmPassword"],
})
type Form = z.infer<typeof schema>

export default function ResetPasswordScreen() {
  const { token } = useLocalSearchParams<{ token?: string }>()
  const router = useRouter()
  const [serverError, setServerError] = useState<string | null>(null)
  const [done, setDone] = useState(false)

  const { control, handleSubmit, formState: { errors, isSubmitting } } = useForm<Form>({
    resolver: zodResolver(schema),
  })

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await apiClient.post("/api/v1/auth/reset-password", { token, password: data.password })
      setDone(true)
    } catch {
      setServerError("Failed to reset password. The link may have expired.")
    }
  }

  if (done) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center px-6">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-2">Password reset ✓</Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center">
          Your password has been updated. Sign in with your new password.
        </Text>
        <TouchableOpacity
          onPress={() => router.replace("/(auth)/login")}
          className="w-full rounded-lg bg-brand py-4 items-center"
        >
          <Text className="text-base font-semibold text-white">Sign in</Text>
        </TouchableOpacity>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark px-6 justify-center">
      <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-2">Reset password</Text>
      <Text className="text-sm text-muted dark:text-muted-dark mb-8">Choose a new password for your account.</Text>

      {serverError ? (
        <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
          <Text className="text-sm text-error">{serverError}</Text>
        </View>
      ) : null}

      {(["password", "confirmPassword"] as const).map((name) => (
        <View key={name} className="mb-4">
          <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
            {name === "password" ? "New password" : "Confirm password"}
          </Text>
          <Controller
            control={control}
            name={name}
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value}
                secureTextEntry
                autoCapitalize="none"
              />
            )}
          />
          {errors[name]?.message ? (
            <Text className="mt-1 text-xs text-error">{errors[name]?.message}</Text>
          ) : null}
        </View>
      ))}

      <TouchableOpacity
        onPress={handleSubmit(onSubmit)}
        disabled={isSubmitting}
        className="rounded-lg bg-brand py-4 items-center mt-2"
        activeOpacity={0.8}
      >
        {isSubmitting ? <ActivityIndicator color="#fff" /> : (
          <Text className="text-base font-semibold text-white">Reset password</Text>
        )}
      </TouchableOpacity>
    </SafeAreaView>
  )
}
