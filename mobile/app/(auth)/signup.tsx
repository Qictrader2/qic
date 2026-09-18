import { useState } from "react"
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
import { SafeAreaView } from "react-native-safe-area-context"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { Check, Eye, EyeOff } from "lucide-react-native"
import { apiClient, ApiError } from "@/src/lib/api/client"

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
  const [serverError, setServerError] = useState<string | null>(null)
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({})
  const [showPwd, setShowPwd] = useState(false)

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
      router.replace("/(auth)/verify-email-pending" as never)
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
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <KeyboardAvoidingView
        behavior={Platform.OS === "ios" ? "padding" : "height"}
        className="flex-1"
      >
        <ScrollView
          contentContainerStyle={{ flexGrow: 1 }}
          keyboardShouldPersistTaps="handled"
          showsVerticalScrollIndicator={false}
        >
          <View className="flex-1 justify-center px-6 py-8">
            <View className="items-center mb-6">
              <Image
                source={require("@/assets/images/logo.png")}
                style={{ width: 160, height: 54 }}
                resizeMode="contain"
              />
            </View>

            <View className="mb-6 items-center">
              <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark">
                Create your account
              </Text>
              <Text className="mt-2 text-sm text-muted dark:text-muted-dark text-center max-w-[300px]">
                Start trading on QicTrader in minutes.
              </Text>
            </View>

            {serverError ? (
              <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3">
                <Text className="text-sm text-error">{serverError}</Text>
              </View>
            ) : null}

            <Field
              label="Email"
              error={errors.email?.message ?? fieldErrors.email}
            >
              <Controller
                control={control}
                name="email"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value ?? ""}
                    placeholder="you@example.com"
                    placeholderTextColor="#94A3B8"
                    autoCapitalize="none"
                    autoCorrect={false}
                    keyboardType="email-address"
                    textContentType="emailAddress"
                    editable={!isSubmitting}
                  />
                )}
              />
            </Field>

            <Field
              label="Username"
              error={errors.username?.message ?? fieldErrors.username}
            >
              <Controller
                control={control}
                name="username"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value ?? ""}
                    placeholder="yourhandle"
                    placeholderTextColor="#94A3B8"
                    autoCapitalize="none"
                    autoCorrect={false}
                    editable={!isSubmitting}
                  />
                )}
              />
            </Field>

            <Field
              label="Password"
              error={errors.password?.message ?? fieldErrors.password}
            >
              <Controller
                control={control}
                name="password"
                render={({ field: { onChange, onBlur, value } }) => (
                  <View className="relative">
                    <TextInput
                      className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 pr-12 text-base text-foreground dark:text-foreground-dark"
                      onBlur={onBlur}
                      onChangeText={onChange}
                      value={value ?? ""}
                      placeholder="••••••••••••"
                      placeholderTextColor="#94A3B8"
                      autoCapitalize="none"
                      autoCorrect={false}
                      secureTextEntry={!showPwd}
                      textContentType="newPassword"
                      editable={!isSubmitting}
                    />
                    <TouchableOpacity
                      onPress={() => setShowPwd((v) => !v)}
                      className="absolute right-3 top-3.5"
                      hitSlop={8}
                    >
                      {showPwd ? (
                        <EyeOff size={18} color="#64748B" />
                      ) : (
                        <Eye size={18} color="#64748B" />
                      )}
                    </TouchableOpacity>
                  </View>
                )}
              />
            </Field>

            <Field
              label="Confirm password"
              error={errors.confirmPassword?.message ?? fieldErrors.confirmPassword}
            >
              <Controller
                control={control}
                name="confirmPassword"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value ?? ""}
                    placeholder="••••••••••••"
                    placeholderTextColor="#94A3B8"
                    autoCapitalize="none"
                    autoCorrect={false}
                    secureTextEntry={!showPwd}
                    textContentType="newPassword"
                    editable={!isSubmitting}
                  />
                )}
              />
            </Field>

            {/* Legal checkbox */}
            <Controller
              control={control}
              name="acceptedLegal"
              render={({ field: { onChange, value } }) => (
                <TouchableOpacity
                  onPress={() => onChange(value ? undefined : true)}
                  className="mb-1 flex-row items-start gap-3"
                  activeOpacity={0.7}
                >
                  <View
                    className={`mt-0.5 h-5 w-5 rounded-md border items-center justify-center ${
                      value ? "bg-brand border-brand" : "border-border dark:border-border-dark"
                    }`}
                  >
                    {value ? <Check size={12} color="#FFFFFF" /> : null}
                  </View>
                  <Text className="flex-1 text-sm text-muted dark:text-muted-dark leading-5">
                    I accept the{" "}
                    <Text className="text-brand font-medium">Terms & Conditions</Text>
                    {" "}and{" "}
                    <Text className="text-brand font-medium">Privacy Policy</Text>
                  </Text>
                </TouchableOpacity>
              )}
            />
            {errors.acceptedLegal ? (
              <Text className="mb-4 ml-8 text-xs text-error">{errors.acceptedLegal.message}</Text>
            ) : (
              <View className="mb-5" />
            )}

            <TouchableOpacity
              onPress={handleSubmit(onSubmit)}
              disabled={isSubmitting}
              className="h-12 rounded-xl bg-brand items-center justify-center"
              activeOpacity={0.85}
            >
              {isSubmitting ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text className="text-base font-semibold text-white">Create account</Text>
              )}
            </TouchableOpacity>

            <View className="mt-6 flex-row justify-center">
              <Text className="text-sm text-muted dark:text-muted-dark">Already have an account? </Text>
              <Link href={"/(auth)/login" as never} asChild>
                <TouchableOpacity>
                  <Text className="text-sm font-semibold text-brand">Sign in</Text>
                </TouchableOpacity>
              </Link>
            </View>

            <TouchableOpacity
              onPress={() => router.push("/(tabs)/marketplace" as never)}
              className="mt-3 items-center"
              activeOpacity={0.7}
            >
              <Text className="text-sm text-muted dark:text-muted-dark">
                or browse the marketplace as a guest
              </Text>
            </TouchableOpacity>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  )
}

function Field({
  label,
  error,
  children,
}: {
  label: string
  error: string | undefined
  children: React.ReactNode
}) {
  return (
    <View className="mb-4">
      <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
        {label}
      </Text>
      {children}
      {error ? <Text className="mt-1 text-xs text-error">{error}</Text> : null}
    </View>
  )
}
