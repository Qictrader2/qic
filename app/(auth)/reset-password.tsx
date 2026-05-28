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
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState } from "react"
import { CheckCircle2 } from "lucide-react-native"
import { apiClient } from "@/src/lib/api/client"

const schema = z
  .object({
    password: z.string().min(8, "At least 8 characters"),
    confirmPassword: z.string().min(1, "Required"),
  })
  .refine((d) => d.password === d.confirmPassword, {
    message: "Passwords do not match",
    path: ["confirmPassword"],
  })

type Form = z.infer<typeof schema>

export default function ResetPasswordScreen() {
  const { token } = useLocalSearchParams<{ token?: string }>()
  const router = useRouter()
  const [serverError, setServerError] = useState<string | null>(null)
  const [done, setDone] = useState(false)
  const [showPwd, setShowPwd] = useState(false)

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<Form>({ resolver: zodResolver(schema) })

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
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
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
            Password reset
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark mb-8 text-center max-w-[280px]">
            Your password has been updated. Sign in with your new password.
          </Text>
          <TouchableOpacity
            onPress={() => router.replace("/(auth)/login")}
            className="w-full rounded-xl bg-brand py-4 items-center"
            activeOpacity={0.85}
          >
            <Text className="text-base font-semibold text-white">Sign in</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
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
              Choose a new password
            </Text>
            <Text className="mt-2 text-sm text-muted dark:text-muted-dark text-center">
              At least 8 characters
            </Text>
          </View>

          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          {(["password", "confirmPassword"] as const).map((name) => (
            <View key={name} className="mb-5">
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                {name === "password" ? "New password" : "Confirm password"}
              </Text>
              <Controller
                control={control}
                name={name}
                render={({ field: { onChange, onBlur, value } }) => (
                  <View className="relative">
                    <TextInput
                      className="h-12 rounded-lg border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 pr-14 text-base text-foreground dark:text-foreground-dark"
                      onBlur={onBlur}
                      onChangeText={onChange}
                      value={value}
                      secureTextEntry={!showPwd}
                      autoCapitalize="none"
                      textContentType="newPassword"
                      placeholder="••••••••"
                      placeholderTextColor="#94A3B8"
                      editable={!isSubmitting}
                    />
                    {name === "password" ? (
                      <TouchableOpacity
                        onPress={() => setShowPwd((v) => !v)}
                        className="absolute right-4 top-3.5"
                        hitSlop={8}
                      >
                        <Text className="text-sm text-brand">{showPwd ? "Hide" : "Show"}</Text>
                      </TouchableOpacity>
                    ) : null}
                  </View>
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
            className="h-12 rounded-lg bg-brand items-center justify-center mt-2"
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Reset password</Text>
            )}
          </TouchableOpacity>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  )
}
