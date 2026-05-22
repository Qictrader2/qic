import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { apiClient, ApiError } from "@/src/lib/api/client"

const schema = z.object({
  email: z.string().email("Enter a valid email address"),
})

type Form = z.infer<typeof schema>

export default function ForgotPasswordScreen() {
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
        // Always show success (no user enumeration)
        setSent(true)
      }
    }
  }

  if (sent) {
    return (
      <View className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
        <View className="rounded-xl bg-success-bg p-6">
          <Text className="text-lg font-semibold text-foreground dark:text-foreground-dark mb-2">
            Check your email
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark">
            If that email is registered, you&apos;ll receive a password reset link shortly.
          </Text>
        </View>
      </View>
    )
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <View className="flex-1 justify-center px-6">
        <View className="mb-8">
          <Text className="text-3xl font-bold text-foreground dark:text-foreground-dark">
            Reset password
          </Text>
          <Text className="mt-2 text-sm text-muted dark:text-muted-dark">
            Enter your email and we&apos;ll send a reset link
          </Text>
        </View>

        {serverError ? (
          <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
            <Text className="text-sm text-error">{serverError}</Text>
          </View>
        ) : null}

        <View className="mb-6">
          <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
            Email
          </Text>
          <Controller
            control={control}
            name="email"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-base text-foreground dark:text-foreground-dark"
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
          className="rounded-lg bg-brand py-4 items-center"
          activeOpacity={0.8}
        >
          {isSubmitting ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text className="text-base font-semibold text-white">Send reset link</Text>
          )}
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  )
}
