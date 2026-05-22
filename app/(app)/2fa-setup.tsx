import {
  View, Text, ScrollView, TextInput, TouchableOpacity, ActivityIndicator, Image,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useRouter } from "expo-router"
import { apiClient } from "@/src/lib/api/client"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"

type Step = "show_qr" | "verify" | "done"

const verifySchema = z.object({
  code: z.string().length(6, "Enter 6-digit code"),
})
type VerifyForm = z.infer<typeof verifySchema>

export default function TwoFASetupScreen() {
  const router = useRouter()
  const [step, setStep] = useState<Step>("show_qr")
  const [qrUrl, setQrUrl] = useState<string | null>(null)
  const [secret, setSecret] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const { control, handleSubmit, formState: { errors, isSubmitting } } = useForm<VerifyForm>({
    resolver: zodResolver(verifySchema),
  })

  async function startSetup() {
    setLoading(true)
    setError(null)
    try {
      const res = await apiClient.post<{ qrCodeUrl: string; secret: string }>("/api/v1/auth/2fa/setup")
      setQrUrl(res.qrCodeUrl)
      setSecret(res.secret)
      setStep("show_qr")
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
      setStep("done")
    } catch {
      setError("Invalid code. Try again.")
    }
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
          Two-Factor Authentication
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-6 leading-relaxed">
          Add an extra layer of security. 2FA is required for withdrawals and escrow releases.
        </Text>

        {step === "done" ? (
          <View className="items-center py-12">
            <Text className="text-5xl mb-4">🔐</Text>
            <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">2FA Enabled</Text>
            <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8">
              Your account is now protected with two-factor authentication.
            </Text>
            <TouchableOpacity
              onPress={() => router.back()}
              className="rounded-lg bg-brand px-8 py-3.5"
              activeOpacity={0.8}
            >
              <Text className="text-base font-semibold text-white">Done</Text>
            </TouchableOpacity>
          </View>
        ) : !qrUrl ? (
          <>
            {error ? (
              <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
                <Text className="text-sm text-error">{error}</Text>
              </View>
            ) : null}
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-6">
              {[
                "1. Download an authenticator app (Google Authenticator, Authy)",
                "2. Tap the button below to get your QR code",
                "3. Scan the QR code in your authenticator app",
                "4. Enter the 6-digit code to confirm",
              ].map((step) => (
                <Text key={step} className="text-sm text-muted dark:text-muted-dark mb-2 leading-relaxed">
                  {step}
                </Text>
              ))}
            </View>
            <TouchableOpacity
              onPress={startSetup}
              disabled={loading}
              className="rounded-lg bg-brand py-4 items-center"
              activeOpacity={0.8}
            >
              {loading ? <ActivityIndicator color="#fff" /> : (
                <Text className="text-base font-semibold text-white">Set up 2FA</Text>
              )}
            </TouchableOpacity>
          </>
        ) : (
          <>
            <View className="items-center mb-6">
              <View className="h-48 w-48 rounded-xl bg-surface dark:bg-surface-dark border-2 border-brand items-center justify-center">
                <Text className="text-xs text-muted dark:text-muted-dark text-center px-4">
                  QR Code{"\n"}(scan with authenticator app)
                </Text>
              </View>
            </View>

            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <Text className="text-xs text-muted dark:text-muted-dark mb-1">Manual entry code</Text>
              <Text className="text-sm font-mono font-medium text-foreground dark:text-foreground-dark tracking-wider">
                {secret}
              </Text>
            </View>

            {error ? (
              <View className="mb-3 rounded-lg bg-error-bg px-4 py-2">
                <Text className="text-xs text-error">{error}</Text>
              </View>
            ) : null}

            <View className="mb-6">
              <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
                Verify — enter your authenticator code
              </Text>
              <Controller
                control={control}
                name="code"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-2xl text-center tracking-widest text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value}
                    placeholder="000000"
                    placeholderTextColor="#94A3B8"
                    keyboardType="number-pad"
                    maxLength={6}
                    autoFocus
                  />
                )}
              />
              {errors.code?.message ? (
                <Text className="mt-1 text-xs text-error">{errors.code.message}</Text>
              ) : null}
            </View>

            <TouchableOpacity
              onPress={handleSubmit(onVerify)}
              disabled={isSubmitting}
              className="rounded-lg bg-brand py-4 items-center"
              activeOpacity={0.8}
            >
              {isSubmitting ? <ActivityIndicator color="#fff" /> : (
                <Text className="text-base font-semibold text-white">Verify and enable</Text>
              )}
            </TouchableOpacity>
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
