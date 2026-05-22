import { useState } from "react"
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
} from "react-native"
import { Link, useRouter } from "expo-router"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { apiClient, ApiError } from "@/src/lib/api/client"
import { useAuthStore } from "@/src/store/auth-store"

const signupSchema = z
  .object({
    email: z.string().email("Enter a valid email address"),
    username: z
      .string()
      .min(3, "Username must be at least 3 characters")
      .max(30, "Username must be at most 30 characters")
      .regex(/^[a-zA-Z0-9_]+$/, "Only letters, numbers and underscores"),
    password: z.string().min(12, "Password must be at least 12 characters"),
    confirmPassword: z.string().min(1, "Please confirm your password"),
    acceptedLegal: z.literal(true, {
      errorMap: () => ({ message: "You must accept the Terms & Conditions" }),
    }),
  })
  .refine((d) => d.password === d.confirmPassword, {
    message: "Passwords do not match",
    path: ["confirmPassword"],
  })

type SignupForm = z.infer<typeof signupSchema>

export default function SignupScreen() {
  const router = useRouter()
  const { setRequiresLegalAcceptance } = useAuthStore()
  const [serverError, setServerError] = useState<string | null>(null)
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({})

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<SignupForm>({ resolver: zodResolver(signupSchema) })

  async function onSubmit(data: SignupForm) {
    setServerError(null)
    setFieldErrors({})
    try {
      await apiClient.post("/api/v1/auth/signup", {
        email: data.email,
        username: data.username,
        password: data.password,
        acceptedLegal: true,
      })
      router.replace("/(auth)/verify-email-pending")
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "validation") {
        setFieldErrors(apiErr.fields)
      } else if (apiErr.kind === "network") {
        setServerError("Cannot connect to server. Check your connection.")
      } else {
        setServerError("Sign up failed. Please try again.")
      }
    }
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <ScrollView contentContainerStyle={{ flexGrow: 1 }} keyboardShouldPersistTaps="handled">
        <View className="flex-1 justify-center px-6 py-12">
          <View className="mb-8">
            <Text className="text-3xl font-bold text-foreground dark:text-foreground-dark">
              Create account
            </Text>
            <Text className="mt-2 text-sm text-muted dark:text-muted-dark">
              Start trading on QicTrader
            </Text>
          </View>

          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          {(["email", "username", "password", "confirmPassword"] as const).map((field) => (
            <View key={field} className="mb-4">
              <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark capitalize">
                {field === "confirmPassword" ? "Confirm password" : field}
              </Text>
              <Controller
                control={control}
                name={field}
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value as string}
                    placeholder={
                      field === "email"
                        ? "you@example.com"
                        : field === "username"
                        ? "yourhandle"
                        : "••••••••••••"
                    }
                    placeholderTextColor="#94A3B8"
                    autoCapitalize="none"
                    autoCorrect={false}
                    secureTextEntry={field === "password" || field === "confirmPassword"}
                    keyboardType={field === "email" ? "email-address" : "default"}
                    textContentType={
                      field === "email"
                        ? "emailAddress"
                        : field === "password" || field === "confirmPassword"
                        ? "password"
                        : "username"
                    }
                    editable={!isSubmitting}
                  />
                )}
              />
              {(errors[field] ?? fieldErrors[field]) ? (
                <Text className="mt-1 text-xs text-error">
                  {errors[field]?.message ?? fieldErrors[field]}
                </Text>
              ) : null}
            </View>
          ))}

          {/* Legal checkbox */}
          <Controller
            control={control}
            name="acceptedLegal"
            render={({ field: { onChange, value } }) => (
              <TouchableOpacity
                onPress={() => onChange(value ? undefined : true)}
                className="mb-6 flex-row items-start gap-3"
                activeOpacity={0.7}
              >
                <View
                  className={`mt-0.5 h-5 w-5 rounded border-2 items-center justify-center ${
                    value ? "bg-brand border-brand" : "border-border dark:border-border-dark"
                  }`}
                >
                  {value ? <Text className="text-white text-xs font-bold">✓</Text> : null}
                </View>
                <Text className="flex-1 text-sm text-muted dark:text-muted-dark">
                  I accept the{" "}
                  <Text className="text-brand">Terms & Conditions</Text>
                  {" "}and{" "}
                  <Text className="text-brand">Privacy Policy</Text>
                </Text>
              </TouchableOpacity>
            )}
          />
          {errors.acceptedLegal ? (
            <Text className="-mt-4 mb-4 text-xs text-error">{errors.acceptedLegal.message}</Text>
          ) : null}

          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="rounded-lg bg-brand py-4 items-center"
            activeOpacity={0.8}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Create account</Text>
            )}
          </TouchableOpacity>

          <View className="mt-6 flex-row justify-center">
            <Text className="text-sm text-muted dark:text-muted-dark">
              Already have an account?{" "}
            </Text>
            <Link href="/(auth)/login" asChild>
              <TouchableOpacity>
                <Text className="text-sm font-medium text-brand">Sign in</Text>
              </TouchableOpacity>
            </Link>
          </View>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  )
}
