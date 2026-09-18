import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  TextInput,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useRouter } from "expo-router"
import { useState } from "react"
import {
  ChevronLeft,
  ArrowDown,
  ArrowLeftRight,
  CheckCircle2,
  AlertTriangle,
} from "lucide-react-native"
import { useWallets } from "@/src/hooks/api/use-wallet"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { walletService } from "@/src/services/wallet.service"
import type { Currency, Wallet } from "@/src/services/wallet.service"
import { ApiError } from "@/src/lib/api/client"

const CURRENCY_COLORS: Record<string, string> = {
  BTC: "#F7931A",
  ETH: "#627EEA",
  USDT: "#26A17B",
  USDC: "#2775CA",
  SOL: "#9945FF",
}

function CurrencyPill({
  currency,
  active,
  onPress,
}: {
  currency: Currency
  active: boolean
  onPress: () => void
}) {
  const color = CURRENCY_COLORS[currency] ?? "#00A3F6"
  return (
    <TouchableOpacity
      onPress={onPress}
      activeOpacity={0.85}
      className={`flex-row items-center gap-2 px-4 h-11 rounded-xl border ${
        active
          ? "bg-brand border-brand"
          : "bg-surface dark:bg-card-dark border-border dark:border-border-dark"
      }`}
    >
      <View
        className="w-6 h-6 rounded-full items-center justify-center"
        style={{ backgroundColor: active ? "rgba(255,255,255,0.25)" : color + "22" }}
      >
        <Text
          className="text-[10px] font-bold"
          style={{ color: active ? "#FFFFFF" : color }}
        >
          {currency.slice(0, 1)}
        </Text>
      </View>
      <Text
        className={`text-sm font-semibold ${
          active ? "text-white" : "text-foreground dark:text-foreground-dark"
        }`}
      >
        {currency}
      </Text>
    </TouchableOpacity>
  )
}

export default function InternalTransferScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const { data: wallets, isLoading } = useWallets()
  const [fromCurrency, setFromCurrency] = useState<Currency | null>(null)
  const [toCurrency, setToCurrency] = useState<Currency | null>(null)
  const [amount, setAmount] = useState("")
  const [serverError, setServerError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const { mutateAsync: transfer, isPending } = useMutation({
    mutationFn: (params: Parameters<typeof walletService.internalTransfer>[0]) =>
      walletService.internalTransfer(params),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["wallets"] }),
  })

  const currencies: Currency[] = (wallets ?? []).map((w: Wallet) => w.currency)
  const fromWallet = wallets?.find((w: Wallet) => w.currency === fromCurrency)

  async function handleTransfer() {
    if (!fromCurrency || !toCurrency || !amount) return
    if (fromCurrency === toCurrency) {
      setServerError("Select different currencies.")
      return
    }
    setServerError(null)
    try {
      await transfer({ fromCurrency, toCurrency, amount })
      setSuccess(true)
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "validation") {
        setServerError(Object.values(apiErr.fields)[0] ?? "Validation error")
      } else if (apiErr.kind === "forbidden") {
        setServerError("Insufficient balance.")
      } else {
        setServerError("Transfer failed. Please try again.")
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
            Transfer complete
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8 max-w-[320px]">
            {amount} {fromCurrency} has been converted to {toCurrency} at current market rates.
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

  const canSubmit = fromCurrency && toCurrency && amount && fromCurrency !== toCurrency

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1" edges={["bottom"]}>
        <View className="px-5 pt-2 pb-3 flex-row items-center">
          <TouchableOpacity
            onPress={() => router.back()}
            className="w-10 h-10 items-center justify-center -ml-2"
            activeOpacity={0.7}
          >
            <ChevronLeft size={24} color="#64748B" />
          </TouchableOpacity>
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Internal transfer
          </Text>
        </View>

        <ScrollView className="flex-1 px-5" keyboardShouldPersistTaps="handled" showsVerticalScrollIndicator={false}>
          <View className="rounded-2xl bg-brand/10 border border-brand/20 p-4 mb-5 flex-row items-center gap-3">
            <View className="w-10 h-10 rounded-full bg-brand/20 items-center justify-center">
              <ArrowLeftRight size={18} color="#00A3F6" />
            </View>
            <View className="flex-1">
              <Text className="text-sm font-semibold text-brand">Convert between assets</Text>
              <Text className="text-xs text-brand/80 mt-0.5">
                Instant swap at live market rates, zero on-chain fees.
              </Text>
            </View>
          </View>

          {isLoading ? (
            <View className="py-12 items-center">
              <ActivityIndicator color="#00A3F6" />
            </View>
          ) : (
            <>
              {serverError ? (
                <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3 flex-row items-center gap-2">
                  <AlertTriangle size={14} color="#EF4444" />
                  <Text className="text-sm text-error flex-1">{serverError}</Text>
                </View>
              ) : null}

              {/* From */}
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                From
              </Text>
              <View className="flex-row flex-wrap gap-2 mb-4">
                {currencies.map((c) => (
                  <CurrencyPill
                    key={c}
                    currency={c}
                    active={fromCurrency === c}
                    onPress={() => setFromCurrency(c)}
                  />
                ))}
              </View>

              {fromWallet ? (
                <View className="flex-row items-center justify-between mb-4 -mt-2">
                  <Text className="text-xs text-muted dark:text-muted-dark">
                    Available: {parseFloat(fromWallet.balance).toFixed(8)} {fromWallet.currency}
                  </Text>
                  <TouchableOpacity
                    onPress={() => setAmount(fromWallet.balance)}
                    className="px-2.5 py-1 rounded-md bg-brand/10"
                    activeOpacity={0.7}
                  >
                    <Text className="text-xs font-semibold text-brand">MAX</Text>
                  </TouchableOpacity>
                </View>
              ) : null}

              {/* Arrow */}
              <View className="items-center my-2">
                <View className="w-9 h-9 rounded-full bg-surface dark:bg-card-dark border border-border dark:border-border-dark items-center justify-center">
                  <ArrowDown size={16} color="#64748B" />
                </View>
              </View>

              {/* To */}
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                To
              </Text>
              <View className="flex-row flex-wrap gap-2 mb-5">
                {currencies
                  .filter((c) => c !== fromCurrency)
                  .map((c) => (
                    <CurrencyPill
                      key={c}
                      currency={c}
                      active={toCurrency === c}
                      onPress={() => setToCurrency(c)}
                    />
                  ))}
              </View>

              {/* Amount */}
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                Amount
              </Text>
              <View className="relative mb-2">
                <TextInput
                  className="h-14 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 pr-20 text-xl font-semibold text-foreground dark:text-foreground-dark"
                  value={amount}
                  onChangeText={setAmount}
                  placeholder="0.00"
                  placeholderTextColor="#94A3B8"
                  keyboardType="decimal-pad"
                />
                {fromCurrency ? (
                  <View className="absolute right-4 top-4">
                    <Text className="text-sm font-semibold text-muted dark:text-muted-dark">
                      {fromCurrency}
                    </Text>
                  </View>
                ) : null}
              </View>

              <View className="h-24" />
            </>
          )}
        </ScrollView>

        <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
          <TouchableOpacity
            onPress={handleTransfer}
            disabled={!canSubmit || isPending}
            className={`rounded-xl h-12 items-center justify-center flex-row gap-2 ${
              canSubmit ? "bg-brand" : "bg-brand/40"
            }`}
            activeOpacity={0.85}
          >
            {isPending ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <>
                <ArrowLeftRight size={16} color="#FFFFFF" />
                <Text className="text-base font-semibold text-white">Confirm transfer</Text>
              </>
            )}
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
