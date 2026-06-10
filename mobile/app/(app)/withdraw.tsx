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
import {
  ChevronLeft,
  CheckCircle2,
  ShieldCheck,
  AlertTriangle,
  Fingerprint,
} from "lucide-react-native"
import { useWithdraw, useWithdrawFeePreview, useWallets } from "@/src/hooks/api/use-wallet"
import type { Wallet } from "@/src/services/wallet.service"
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
  const { data: wallets } = useWallets()
  const [serverError, setServerError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const wallet = wallets?.find((w: Wallet) => w.currency === currency && w.network === network)
  const availableBalance = wallet?.balance ?? "0"

  const {
    control,
    handleSubmit,
    watch,
    setValue,
    formState: { errors, isSubmitting },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  const amount = watch("amount") ?? "0"
  const { data: feePreview } = useWithdrawFeePreview(currency ?? "", network ?? "", amount)

  async function onSubmit(data: Form) {
    setServerError(null)

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
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
        <View className="flex-1 items-center justify-center px-6">
          <View className="w-20 h-20 rounded-full bg-success/10 items-center justify-center mb-5">
            <CheckCircle2 size={36} color="#10B981" />
          </View>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            Withdrawal submitted
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8 max-w-[320px]">
            Your withdrawal is being processed and will appear in your transaction history once on-chain.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-xl bg-brand px-8 h-12 items-center justify-center"
            activeOpacity={0.85}
          >
            <Text className="text-base font-semibold text-white">Done</Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => router.push("/(app)/transactions" as never)}
            className="mt-4 py-2"
            activeOpacity={0.7}
          >
            <Text className="text-sm font-medium text-brand">View transactions</Text>
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
      <SafeAreaView className="flex-1" edges={["bottom"]}>
        <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
          <TouchableOpacity onPress={() => router.back()} className="w-10 h-10 items-center justify-center -ml-2" activeOpacity={0.7}>
            <ChevronLeft size={24} color="#64748B" />
          </TouchableOpacity>
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Withdraw {currency}
          </Text>
          <View className="w-10" />
        </View>

        <ScrollView className="flex-1 px-5" keyboardShouldPersistTaps="handled" showsVerticalScrollIndicator={false}>
          {/* Balance card */}
          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-4">
            <Text className="text-xs text-muted dark:text-muted-dark mb-1">Available balance</Text>
            <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
              {parseFloat(availableBalance).toFixed(8)}{" "}
              <Text className="text-base font-medium text-muted dark:text-muted-dark">{currency}</Text>
            </Text>
            <View className="flex-row items-center gap-1.5 mt-2">
              <View className="w-1.5 h-1.5 rounded-full bg-success" />
              <Text className="text-xs text-muted dark:text-muted-dark capitalize">
                {network?.replace(/_/g, " ")} network
              </Text>
            </View>
          </View>

          {serverError ? (
            <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3 flex-row items-center gap-2">
              <AlertTriangle size={14} color="#EF4444" />
              <Text className="text-sm text-error flex-1">{serverError}</Text>
            </View>
          ) : null}

          {/* Address */}
          <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
            Destination address
          </Text>
          <Controller
            control={control}
            name="toAddress"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-sm text-foreground dark:text-foreground-dark"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value ?? ""}
                placeholder={`${currency} address`}
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

          {/* Amount */}
          <View className="mt-4 flex-row items-center justify-between">
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">Amount</Text>
            <TouchableOpacity
              onPress={() => setValue("amount", availableBalance)}
              className="px-2.5 py-1 rounded-md bg-brand/10"
              activeOpacity={0.7}
            >
              <Text className="text-xs font-semibold text-brand">MAX</Text>
            </TouchableOpacity>
          </View>
          <View className="mt-2">
            <Controller
              control={control}
              name="amount"
              render={({ field: { onChange, onBlur, value } }) => (
                <View className="relative">
                  <TextInput
                    className="h-14 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 pr-20 text-xl font-semibold text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value ?? ""}
                    placeholder="0.00"
                    placeholderTextColor="#94A3B8"
                    keyboardType="decimal-pad"
                    editable={!isSubmitting}
                  />
                  <View className="absolute right-4 top-4">
                    <Text className="text-sm font-semibold text-muted dark:text-muted-dark">{currency}</Text>
                  </View>
                </View>
              )}
            />
            {errors.amount ? (
              <Text className="mt-1 text-xs text-error">{errors.amount.message}</Text>
            ) : null}
          </View>

          {/* Fee preview */}
          {feePreview ? (
            <View className="mt-4 rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
              <View className="flex-row justify-between py-1.5">
                <Text className="text-xs text-muted dark:text-muted-dark">Network fee</Text>
                <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                  {feePreview.fee} {currency}
                </Text>
              </View>
              <View className="h-px bg-border/50 dark:bg-border-dark/50 my-1" />
              <View className="flex-row justify-between py-1.5">
                <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                  You receive
                </Text>
                <Text className="text-sm font-bold text-foreground dark:text-foreground-dark">
                  {feePreview.netAmount} {currency}
                </Text>
              </View>
            </View>
          ) : null}

          {/* 2FA */}
          <Text className="mt-5 mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
            2FA code
          </Text>
          <Controller
            control={control}
            name="twoFactorCode"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-xl text-center tracking-[6px] font-semibold text-foreground dark:text-foreground-dark"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value ?? ""}
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

          <View className="mt-4 rounded-xl bg-warning-bg border border-warning/20 p-3 flex-row items-start gap-2">
            <AlertTriangle size={14} color="#F59E0B" />
            <Text className="text-xs text-warning flex-1 leading-5">
              Withdrawals to the wrong address or wrong network are{" "}
              <Text className="font-bold">irreversible</Text>. Always double-check before confirming.
            </Text>
          </View>

          <View className="mt-3 rounded-xl bg-brand/10 p-3 flex-row items-center gap-2">
            <ShieldCheck size={14} color="#00A3F6" />
            <Text className="text-xs text-brand flex-1">
              Biometric and 2FA required for every withdrawal.
            </Text>
          </View>

          <View className="h-24" />
        </ScrollView>

        <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="rounded-xl bg-brand h-12 items-center justify-center flex-row gap-2"
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <>
                <Fingerprint size={18} color="#FFFFFF" />
                <Text className="text-base font-semibold text-white">Confirm withdrawal</Text>
              </>
            )}
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
