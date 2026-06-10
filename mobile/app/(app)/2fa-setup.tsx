import {
  View,
  Text,
  ScrollView,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
  Platform,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useRouter } from "expo-router"
import QRCode from "react-native-qrcode-svg"
import { Clipboard } from "react-native"
import {
  ChevronLeft,
  ShieldCheck,
  Copy,
  CheckCircle2,
  Smartphone,
  Scan,
  KeySquare,
} from "lucide-react-native"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { apiClient } from "@/src/lib/api/client"

const verifySchema = z.object({
  code: z.string().length(6, "Enter 6-digit code"),
})
type VerifyForm = z.infer<typeof verifySchema>

export default function TwoFASetupScreen() {
  const router = useRouter()
  const [qrUrl, setQrUrl] = useState<string | null>(null)
  const [secret, setSecret] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const [done, setDone] = useState(false)

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<VerifyForm>({
    resolver: zodResolver(verifySchema),
  })

  async function startSetup() {
    setLoading(true)
    setError(null)
    try {
      const res = await apiClient.post<{ qrCodeUrl: string; secret: string }>(
        "/api/v1/auth/2fa/setup",
      )
      setQrUrl(res.qrCodeUrl)
      setSecret(res.secret)
    } catch {
      setError("Failed to start 2FA setup. Please try again.")
    } finally {
      setLoading(false)
    }
  }

  async function onVerify(data: VerifyForm) {
    setError(null)
    try {
      await apiClient.post("/api/v1/auth/2fa/verify", { code: data.code })
      setDone(true)
    } catch {
      setError("Invalid code. Try again.")
    }
  }

  function copySecret() {
    if (!secret) return
    Clipboard.setString(secret)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  if (done) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
        <View className="flex-1 items-center justify-center px-6">
          <View className="w-20 h-20 rounded-full bg-success/10 items-center justify-center mb-5">
            <ShieldCheck size={36} color="#10B981" />
          </View>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            2FA enabled
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8 max-w-[320px]">
            Your account is now protected with two-factor authentication. You'll be asked for a code each time you sign in or withdraw.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-xl bg-brand px-8 h-12 items-center justify-center"
            activeOpacity={0.85}
          >
            <Text className="text-base font-semibold text-white">Done</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <View className="px-5 pt-2 pb-3 flex-row items-center">
        <TouchableOpacity
          onPress={() => router.back()}
          className="w-10 h-10 items-center justify-center -ml-2"
          activeOpacity={0.7}
        >
          <ChevronLeft size={24} color="#64748B" />
        </TouchableOpacity>
        <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
          Two-factor authentication
        </Text>
      </View>

      <ScrollView className="flex-1 px-5" keyboardShouldPersistTaps="handled" showsVerticalScrollIndicator={false}>
        {!qrUrl ? (
          <>
            <View className="items-center mt-2 mb-6">
              <View className="w-16 h-16 rounded-2xl bg-brand/10 items-center justify-center mb-4">
                <ShieldCheck size={28} color="#00A3F6" />
              </View>
              <Text className="text-xl font-bold text-foreground dark:text-foreground-dark text-center">
                Add an extra layer of security
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark text-center mt-2 max-w-[320px]">
                2FA is required for withdrawals, escrow releases, and account changes.
              </Text>
            </View>

            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-6">
              {[
                { Icon: Smartphone, title: "Install an authenticator app", body: "Google Authenticator, Authy, or 1Password." },
                { Icon: Scan, title: "Scan the QR code", body: "Open the app and scan the code we'll show you." },
                { Icon: KeySquare, title: "Enter the code", body: "Type the 6-digit code your app shows." },
              ].map(({ Icon, title, body }, i, arr) => (
                <View
                  key={title}
                  className={`flex-row gap-3 py-2.5 ${
                    i < arr.length - 1 ? "border-b border-border/40 dark:border-border-dark/40" : ""
                  }`}
                >
                  <View className="w-9 h-9 rounded-full bg-brand/10 items-center justify-center">
                    <Icon size={16} color="#00A3F6" />
                  </View>
                  <View className="flex-1">
                    <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                      {title}
                    </Text>
                    <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">{body}</Text>
                  </View>
                </View>
              ))}
            </View>

            {error ? (
              <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3">
                <Text className="text-sm text-error">{error}</Text>
              </View>
            ) : null}

            <TouchableOpacity
              onPress={startSetup}
              disabled={loading}
              className="rounded-xl bg-brand h-12 items-center justify-center mb-8"
              activeOpacity={0.85}
            >
              {loading ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text className="text-base font-semibold text-white">Start setup</Text>
              )}
            </TouchableOpacity>
          </>
        ) : (
          <>
            <View className="items-center mb-5 mt-2">
              <Text className="text-sm text-muted dark:text-muted-dark text-center mb-4 max-w-[320px]">
                Scan this QR code with your authenticator app.
              </Text>
              <View className="p-4 rounded-2xl bg-white shadow-sm border border-border dark:border-border-dark">
                <QRCode value={qrUrl} size={200} color="#000000" backgroundColor="#FFFFFF" />
              </View>
            </View>

            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-4">
              <View className="flex-row items-center justify-between mb-1.5">
                <Text className="text-xs font-medium text-muted dark:text-muted-dark uppercase tracking-wider">
                  Manual entry code
                </Text>
                <TouchableOpacity
                  onPress={copySecret}
                  className="flex-row items-center gap-1 px-2 py-1 rounded-md bg-brand/10"
                  activeOpacity={0.7}
                >
                  {copied ? (
                    <>
                      <CheckCircle2 size={11} color="#10B981" />
                      <Text className="text-[11px] text-success font-semibold">Copied</Text>
                    </>
                  ) : (
                    <>
                      <Copy size={11} color="#00A3F6" />
                      <Text className="text-[11px] text-brand font-semibold">Copy</Text>
                    </>
                  )}
                </TouchableOpacity>
              </View>
              <Text className="text-sm font-mono font-medium text-foreground dark:text-foreground-dark tracking-wider">
                {secret}
              </Text>
            </View>

            {error ? (
              <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3">
                <Text className="text-sm text-error">{error}</Text>
              </View>
            ) : null}

            <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
              Enter the 6-digit code from your app
            </Text>
            <Controller
              control={control}
              name="code"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="h-14 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-2xl text-center tracking-[6px] font-semibold text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder="000000"
                  placeholderTextColor="#94A3B8"
                  keyboardType="number-pad"
                  maxLength={6}
                  autoFocus={Platform.OS === "ios"}
                />
              )}
            />
            {errors.code?.message ? (
              <Text className="mt-1 text-xs text-error">{errors.code.message}</Text>
            ) : null}

            <TouchableOpacity
              onPress={handleSubmit(onVerify)}
              disabled={isSubmitting}
              className="mt-5 rounded-xl bg-brand h-12 items-center justify-center mb-8"
              activeOpacity={0.85}
            >
              {isSubmitting ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text className="text-base font-semibold text-white">Verify and enable</Text>
              )}
            </TouchableOpacity>
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
