import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { useLocalSearchParams, useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState } from "react"
import { useWithdraw, useWithdrawFeePreview } from "@/src/hooks/api/use-wallet"
import { promptBiometric, isBiometricEnabled, isBiometricAvailable } from "@/src/lib/biometric"
import { ApiError } from "@/src/lib/api/client"
import type { Currency, Network } from "@/src/services/wallet.service"

const schema = z.object({
  toAddress: z.string().min(20, "Enter a valid wallet address"),
  amount: z.string().refine((v) => parseFloat(v) > 0, "Enter an amount greater than 0"),
  twoFactorCode: z.string().length(6, "Enter your 6-digit 2FA code"),
})

type Form = z.infer<typeof schema>

export default function WithdrawScreen() {
  const { currency, network } = useLocalSearchParams<{ currency: string; network: string }>()
  const router = useRouter()
  const { mutateAsync: withdraw } = useWithdraw()
  const [serverError, setServerError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const {
    control,
    handleSubmit,
    watch,
    formState: { errors, isSubmitting },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  const amount = watch("amount") ?? "0"
  const { data: feePreview } = useWithdrawFeePreview(currency ?? "", network ?? "", amount)

  async function onSubmit(data: Form) {
    setServerError(null)

    // Biometric re-auth required before withdraw (SEC rule)
    const biometricEnabled = await isBiometricEnabled()
    const biometricAvailable = await isBiometricAvailable()
    if (biometricEnabled && biometricAvailable) {
      const passed = await promptBiometric("Confirm withdrawal")
      if (!passed) {
        setServerError("Biometric verification failed.")
        return
      }
    }

    try {
      await withdraw({
        currency: currency as Currency,
        network: network as Network,
        toAddress: data.toAddress,
        amount: data.amount,
        twoFactorCode: data.twoFactorCode,
      })
      setSuccess(true)
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "validation") {
        setServerError(Object.values(apiErr.fields)[0] ?? "Validation error")
      } else if (apiErr.kind === "unauthorized") {
        setServerError("Invalid 2FA code.")
      } else if (apiErr.kind === "forbidden") {
        setServerError("Insufficient balance or withdrawal not allowed.")
      } else {
        setServerError("Withdrawal failed. Please try again.")
      }
    }
  }

  if (success) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
        <View className="items-center">
          <Text className="text-5xl mb-4">✅</Text>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            Withdrawal submitted
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8">
            Your withdrawal is being processed. Check transaction history for updates.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-lg bg-brand px-8 py-3.5"
          >
            <Text className="text-base font-semibold text-white">Done</Text>
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
      <SafeAreaView className="flex-1">
        <ScrollView className="flex-1 px-4 py-4" keyboardShouldPersistTaps="handled">
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-1">
            Withdraw {currency}
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark mb-6 capitalize">
            Network: {network}
          </Text>

          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          {/* Address */}
          <View className="mb-4">
            <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
              Destination address
            </Text>
            <Controller
              control={control}
              name="toAddress"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  placeholder={`${currency} address on ${network}`}
                  placeholderTextColor="#94A3B8"
                  autoCapitalize="none"
                  autoCorrect={false}
                  editable={!isSubmitting}
                />
              )}
            />
            {errors.toAddress ? (
              <Text className="mt-1 text-xs text-error">{errors.toAddress.message}</Text>
            ) : null}
          </View>

          {/* Amount */}
          <View className="mb-4">
            <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
              Amount
            </Text>
            <Controller
              control={control}
              name="amount"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  placeholder="0.00"
                  placeholderTextColor="#94A3B8"
                  keyboardType="decimal-pad"
                  editable={!isSubmitting}
                />
              )}
            />
            {errors.amount ? (
              <Text className="mt-1 text-xs text-error">{errors.amount.message}</Text>
            ) : null}
          </View>

          {/* Fee preview */}
          {feePreview ? (
            <View className="mb-4 rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4">
              <View className="flex-row justify-between mb-1">
                <Text className="text-xs text-muted dark:text-muted-dark">Network fee</Text>
                <Text className="text-xs text-foreground dark:text-foreground-dark">
                  {feePreview.fee} {currency}
                </Text>
              </View>
              <View className="flex-row justify-between">
                <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                  You receive
                </Text>
                <Text className="text-xs font-semibold text-foreground dark:text-foreground-dark">
                  {feePreview.netAmount} {currency}
                </Text>
              </View>
            </View>
          ) : null}

          {/* 2FA */}
          <View className="mb-6">
            <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
              2FA code
            </Text>
            <Controller
              control={control}
              name="twoFactorCode"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-base text-center tracking-widest text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  placeholder="000000"
                  placeholderTextColor="#94A3B8"
                  keyboardType="number-pad"
                  maxLength={6}
                  editable={!isSubmitting}
                />
              )}
            />
            {errors.twoFactorCode ? (
              <Text className="mt-1 text-xs text-error">{errors.twoFactorCode.message}</Text>
            ) : null}
          </View>

          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="rounded-lg bg-brand py-4 items-center mb-8"
            activeOpacity={0.8}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Confirm withdrawal</Text>
            )}
          </TouchableOpacity>
        </ScrollView>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
