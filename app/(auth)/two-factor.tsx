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
import { useLocalSearchParams, useRouter } from "expo-router"
import { useEffect, useRef, useState } from "react"
import { SafeAreaView } from "react-native-safe-area-context"
import { ShieldCheck, ChevronLeft, KeyRound } from "lucide-react-native"
import { apiClient, ApiError } from "@/src/lib/api/client"
import { useAuthStore } from "@/src/store/auth-store"

interface TwoFactorResponse {
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
}

const OTP_LENGTH = 6
const BACKUP_LENGTH = 8

export default function TwoFactorScreen() {
  const { twoFactorToken } = useLocalSearchParams<{ twoFactorToken: string }>()
  const router = useRouter()
  const { login } = useAuthStore()
  const [code, setCode] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [useBackupCode, setUseBackupCode] = useState(false)
  const inputRef = useRef<TextInput>(null)

  useEffect(() => {
    const t = setTimeout(() => inputRef.current?.focus(), 250)
    return () => clearTimeout(t)
  }, [useBackupCode])

  async function onSubmit(submitCode?: string) {
    const c = (submitCode ?? code).trim()
    const expectedLen = useBackupCode ? BACKUP_LENGTH : OTP_LENGTH
    if (c.length < expectedLen) return

    setIsSubmitting(true)
    setError(null)
    try {
      const res = await apiClient.post<TwoFactorResponse>("/api/v1/auth/2fa/verify-login", {
        twoFactorToken,
        code: c,
      })
      await login(res.accessToken, res.refreshToken, {
        uid: res.user.uid,
        email: res.user.email,
        username: res.user.username,
        displayName: res.user.displayName,
        emailVerified: res.user.emailVerified,
        role: res.user.role,
        kycTier: res.user.kycTier ?? 0,
      })
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "unauthorized" || apiErr.kind === "validation") {
        setError("Invalid code. Please try again.")
      } else {
        setError("Something went wrong. Please try again.")
      }
      setCode("")
      inputRef.current?.focus()
    } finally {
      setIsSubmitting(false)
    }
  }

  function handleChange(v: string) {
    const expected = useBackupCode ? BACKUP_LENGTH : OTP_LENGTH
    const clean = useBackupCode ? v.toUpperCase().slice(0, expected) : v.replace(/\D/g, "").slice(0, expected)
    setCode(clean)
    if (!useBackupCode && clean.length === OTP_LENGTH) {
      setTimeout(() => onSubmit(clean), 80)
    }
  }

  const expectedLen = useBackupCode ? BACKUP_LENGTH : OTP_LENGTH
  const digits = Array.from({ length: expectedLen }, (_, i) => code[i] ?? "")

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <KeyboardAvoidingView
        behavior={Platform.OS === "ios" ? "padding" : "height"}
        className="flex-1"
      >
        <ScrollView
          className="flex-1"
          contentContainerStyle={{ flexGrow: 1, justifyContent: "center" }}
          keyboardShouldPersistTaps="handled"
        >
          <View className="px-6 pb-8">
            <TouchableOpacity
              onPress={() => router.replace("/(auth)/login" as never)}
              className="self-start w-10 h-10 items-center justify-center -ml-2 mb-4"
              activeOpacity={0.7}
            >
              <ChevronLeft size={24} color="#64748B" />
            </TouchableOpacity>

            <View className="items-center mb-8">
              <View className="w-16 h-16 rounded-2xl bg-brand/10 items-center justify-center mb-4">
                <ShieldCheck size={28} color="#00A3F6" />
              </View>
              <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark text-center">
                Two-factor authentication
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark text-center mt-2 max-w-[300px]">
                {useBackupCode
                  ? "Enter one of your 8-character backup codes."
                  : "Enter the 6-digit code from your authenticator app."}
              </Text>
            </View>

            {error ? (
              <View className="mb-5 rounded-xl bg-error-bg px-4 py-3 border border-error/20">
                <Text className="text-sm text-error text-center">{error}</Text>
              </View>
            ) : null}

            <TouchableOpacity activeOpacity={1} onPress={() => inputRef.current?.focus()}>
              <View className="flex-row justify-center gap-2 mb-6">
                {digits.map((d, i) => {
                  const filled = d !== ""
                  const isCursor = i === code.length
                  return (
                    <View
                      key={i}
                      className={`rounded-xl border items-center justify-center ${
                        filled
                          ? "border-brand bg-brand/10"
                          : isCursor
                            ? "border-brand bg-surface dark:bg-card-dark"
                            : "border-border dark:border-border-dark bg-surface dark:bg-card-dark"
                      }`}
                      style={{
                        width: useBackupCode ? 36 : 46,
                        height: 56,
                      }}
                    >
                      <Text
                        className={`font-semibold text-foreground dark:text-foreground-dark ${
                          useBackupCode ? "text-base" : "text-2xl"
                        }`}
                      >
                        {d}
                      </Text>
                    </View>
                  )
                })}
              </View>
            </TouchableOpacity>

            <TextInput
              ref={inputRef}
              value={code}
              onChangeText={handleChange}
              keyboardType={useBackupCode ? "default" : "number-pad"}
              maxLength={expectedLen}
              autoComplete="one-time-code"
              textContentType={useBackupCode ? "none" : "oneTimeCode"}
              autoCapitalize="characters"
              autoCorrect={false}
              autoFocus
              editable={!isSubmitting}
              caretHidden
              className="absolute opacity-0 -z-10"
              style={{ width: 1, height: 1, position: "absolute" }}
            />

            <TouchableOpacity
              onPress={() => onSubmit()}
              disabled={isSubmitting || code.length < expectedLen}
              className={`rounded-xl h-12 items-center justify-center ${
                code.length < expectedLen ? "bg-brand/40" : "bg-brand"
              }`}
              activeOpacity={0.85}
            >
              {isSubmitting ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text className="text-base font-semibold text-white">Verify</Text>
              )}
            </TouchableOpacity>

            <TouchableOpacity
              onPress={() => {
                setUseBackupCode((v) => !v)
                setCode("")
                setError(null)
              }}
              className="mt-5 flex-row items-center justify-center gap-2 py-2"
              activeOpacity={0.7}
            >
              <KeyRound size={14} color="#00A3F6" />
              <Text className="text-sm text-brand font-medium">
                {useBackupCode ? "Use authenticator app" : "Use a backup code"}
              </Text>
            </TouchableOpacity>

            <View className="mt-6 rounded-xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
              <Text className="text-xs text-muted dark:text-muted-dark text-center leading-5">
                Lost access to your authenticator?{" "}
                <Text
                  className="text-brand font-medium"
                  onPress={() => router.push("/(app)/support" as never)}
                >
                  Contact support
                </Text>
                .
              </Text>
            </View>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  )
}
