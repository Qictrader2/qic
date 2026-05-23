import {
  View, Text, ScrollView, TouchableOpacity, ActivityIndicator, TextInput,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useRouter } from "expo-router"
import { useState } from "react"
import { useWallets } from "@/src/hooks/api/use-wallet"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { walletService } from "@/src/services/wallet.service"
import type { Currency } from "@/src/services/wallet.service"
import { ApiError } from "@/src/lib/api/client"

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

  const currencies = wallets?.map((w) => w.currency) ?? []

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
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
        <View className="items-center">
          <Text className="text-5xl mb-4">⇄</Text>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            Transfer complete
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-lg bg-brand px-8 py-3.5 mt-6"
          >
            <Text className="text-base font-semibold text-white">Done</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-6">
          Internal Transfer
        </Text>

        {isLoading ? (
          <ActivityIndicator color="#00A3F6" />
        ) : (
          <>
            {serverError ? (
              <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
                <Text className="text-sm text-error">{serverError}</Text>
              </View>
            ) : null}

            <View className="mb-4">
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                From
              </Text>
              <View className="flex-row flex-wrap gap-2">
                {currencies.map((c) => (
                  <TouchableOpacity
                    key={c}
                    onPress={() => setFromCurrency(c)}
                    className={`px-4 py-2.5 rounded-lg border ${
                      fromCurrency === c
                        ? "bg-brand border-brand"
                        : "border-border dark:border-border-dark"
                    }`}
                  >
                    <Text className={`text-sm font-medium ${fromCurrency === c ? "text-white" : "text-foreground dark:text-foreground-dark"}`}>
                      {c}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>
            </View>

            <View className="mb-4">
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                To
              </Text>
              <View className="flex-row flex-wrap gap-2">
                {currencies.filter((c) => c !== fromCurrency).map((c) => (
                  <TouchableOpacity
                    key={c}
                    onPress={() => setToCurrency(c)}
                    className={`px-4 py-2.5 rounded-lg border ${
                      toCurrency === c
                        ? "bg-brand border-brand"
                        : "border-border dark:border-border-dark"
                    }`}
                  >
                    <Text className={`text-sm font-medium ${toCurrency === c ? "text-white" : "text-foreground dark:text-foreground-dark"}`}>
                      {c}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>
            </View>

            <View className="mb-6">
              <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
                Amount
              </Text>
              <TextInput
                className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                value={amount}
                onChangeText={setAmount}
                placeholder="0.00"
                placeholderTextColor="#94A3B8"
                keyboardType="decimal-pad"
              />
            </View>

            <TouchableOpacity
              onPress={handleTransfer}
              disabled={!fromCurrency || !toCurrency || !amount || isPending}
              className={`rounded-lg py-4 items-center ${
                fromCurrency && toCurrency && amount ? "bg-brand" : "bg-muted/30"
              }`}
              activeOpacity={0.8}
            >
              {isPending ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text className={`text-base font-semibold ${fromCurrency && toCurrency && amount ? "text-white" : "text-muted"}`}>
                  Transfer
                </Text>
              )}
            </TouchableOpacity>
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
