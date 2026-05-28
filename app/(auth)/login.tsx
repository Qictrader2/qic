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
  Image,
} from "react-native"
import { Link, useRouter } from "expo-router"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useAuthStore } from "@/src/store/auth-store"
import { apiClient, ApiError } from "@/src/lib/api/client"
import { AppleSignInButton, GoogleSignInButton } from "@/src/components/features/auth/OAuthButtons"
import { trackEvent, Sentry } from "@/src/lib/analytics"

const loginSchema = z.object({
  email: z.string().email("Enter a valid email address"),
  password: z.string().min(1, "Password is required"),
})

type LoginForm = z.infer<typeof loginSchema>

interface LoginResponse {
  accessToken: string
  refreshToken: string
  user: {
    uid: string
    email: string
    username: string | null
    displayName: string | null
    emailVerified: boolean
    role: "user" | "admin" | "support" | "moderator" | null
    kycTier: number
  }
  requires2fa?: boolean
  twoFactorToken?: string
  requiresLegalAcceptance?: boolean
}

export default function LoginScreen() {
  const router = useRouter()
  const { login, setRequiresLegalAcceptance } = useAuthStore()
  const [showPassword, setShowPassword] = useState(false)
  const [serverError, setServerError] = useState<string | null>(null)

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<LoginForm>({ resolver: zodResolver(loginSchema) })

  async function onSubmit(data: LoginForm) {
    setServerError(null)
    try {
      const res = await apiClient.post<LoginResponse>("/api/v1/auth/login", data)

      if (res.requires2fa && res.twoFactorToken) {
        router.push({
          pathname: "/(auth)/two-factor",
          params: { twoFactorToken: res.twoFactorToken },
        })
        return
      }

      if (res.requiresLegalAcceptance) {
        setRequiresLegalAcceptance(true)
      }

      await login(res.accessToken, res.refreshToken, {
        uid: res.user.uid,
        email: res.user.email,
        username: res.user.username,
        displayName: res.user.displayName,
        emailVerified: res.user.emailVerified,
        role: res.user.role,
        kycTier: res.user.kycTier ?? 0,
      })
      trackEvent({ name: "login", method: "email" })
      Sentry.setUser({ id: res.user.uid, email: res.user.email })
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "unauthorized") {
        setServerError("Invalid email or password")
      } else if (apiErr.kind === "network") {
        setServerError("Cannot connect to server. Check your connection.")
      } else if (apiErr.kind === "validation") {
        setServerError(Object.values(apiErr.fields)[0] ?? "Validation error")
      } else {
        setServerError("Something went wrong. Please try again.")
      }
    }
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <ScrollView
        contentContainerStyle={{ flexGrow: 1 }}
        keyboardShouldPersistTaps="handled"
      >
        <View className="flex-1 justify-center px-6 py-12">
          {/* Logo */}
          <View className="items-center mb-8">
            <Image
              source={require("@/assets/images/logo.png")}
              style={{ width: 180, height: 60, resizeMode: "contain" }}
            />
          </View>

          {/* Heading — mirrors web "Welcome back" copy */}
          <View className="mb-8 items-center">
            <Text className="text-2xl font-semibold text-foreground dark:text-foreground-dark">
              Welcome back
            </Text>
            <Text className="mt-2 text-sm text-muted dark:text-muted-dark">
              Please enter your details to login
            </Text>
          </View>

          {/* Server error */}
          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          {/* Email */}
          <View className="mb-5">
            <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
              Email Address
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
                  placeholder="Enter your email"
                  placeholderTextColor="#94A3B8"
                  autoCapitalize="none"
                  autoCorrect={false}
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

          {/* Password */}
          <View className="mb-3">
            <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
              Password
            </Text>
            <Controller
              control={control}
              name="password"
              render={({ field: { onChange, onBlur, value } }) => (
                <View className="relative">
                  <TextInput
                    className="h-12 rounded-lg border border-border dark:border-border-dark bg-background-gray dark:bg-surface-dark px-4 pr-14 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value}
                    placeholder="Enter your password"
                    placeholderTextColor="#94A3B8"
                    secureTextEntry={!showPassword}
                    textContentType="password"
                    editable={!isSubmitting}
                  />
                  <TouchableOpacity
                    onPress={() => setShowPassword((v) => !v)}
                    className="absolute right-4 top-3.5"
                    hitSlop={8}
                  >
                    <Text className="text-sm text-brand">{showPassword ? "Hide" : "Show"}</Text>
                  </TouchableOpacity>
                </View>
              )}
            />
            {errors.password ? (
              <Text className="mt-1 text-xs text-error">{errors.password.message}</Text>
            ) : null}
          </View>

          {/* Forgot password — right-aligned, mirrors web */}
          <View className="mb-6 items-end">
            <Link href="/(auth)/forgot-password" asChild>
              <TouchableOpacity hitSlop={8}>
                <Text className="text-sm text-brand">Forgot Password?</Text>
              </TouchableOpacity>
            </Link>
          </View>

          {/* Submit */}
          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="h-12 rounded-lg bg-brand items-center justify-center"
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Sign in</Text>
            )}
          </TouchableOpacity>

          {/* OAuth divider */}
          <View className="mt-6 flex-row items-center gap-3">
            <View className="flex-1 h-px bg-border dark:bg-border-dark" />
            <Text className="text-xs uppercase text-muted dark:text-muted-dark tracking-wider">
              Or continue with
            </Text>
            <View className="flex-1 h-px bg-border dark:bg-border-dark" />
          </View>

          <View className="mt-4 gap-3">
            <GoogleSignInButton />
            {Platform.OS === "ios" ? <AppleSignInButton /> : null}
          </View>

          {/* Sign up link */}
          <View className="mt-8 flex-row justify-center">
            <Text className="text-sm text-muted dark:text-muted-dark">
              Don&apos;t have an account?{" "}
            </Text>
            <Link href="/(auth)/signup" asChild>
              <TouchableOpacity>
                <Text className="text-sm font-medium text-brand">Sign Up</Text>
              </TouchableOpacity>
            </Link>
          </View>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  )
}
