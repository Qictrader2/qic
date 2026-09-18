import {
  View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Switch,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useRouter } from "expo-router"
import { useQuery } from "@tanstack/react-query"
import { ChevronLeft, TrendingUp, TrendingDown, Eye, EyeOff } from "lucide-react-native"
import { useWallets } from "@/src/hooks/api/use-wallet"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { apiClient } from "@/src/lib/api/client"
import { useState, useEffect } from "react"

type FiatCurrency = "ZAR" | "NGN" | "USD" | "EUR" | "GBP" | "KES" | "GHS"
const FIAT_CURRENCIES: FiatCurrency[] = ["ZAR", "NGN", "USD", "EUR", "GBP", "KES", "GHS"]
const CURRENCY_SYMBOLS: Record<FiatCurrency, string> = {
  ZAR: "R", NGN: "₦", USD: "$", EUR: "€", GBP: "£", KES: "KSh", GHS: "GH₵",
}
const DISPLAY_CURRENCY_KEY = "qic_display_fiat"

interface FiatBalance {
  currency: FiatCurrency
  balance: string
  change24h: string
  changePercent: string
}

export default function FiatBalanceScreen() {
  const router = useRouter()
  const { data: wallets, isLoading } = useWallets()
  const [displayCurrency, setDisplayCurrency] = useState<FiatCurrency>("ZAR")
  const [showEquivalents, setShowEquivalents] = useState(true)

  // Parity note: there is no /wallets/fiat-equivalent endpoint. Like the web,
  // fiat values are derived client-side: USD spot from GET /prices, and the
  // USD->ZAR rate from GET /prices/fx (the backend's only fiat FX reference).
  // Other display currencies have no backend rate source yet.
  const { data: fiatBalances } = useQuery({
    queryKey: ["fiat-balances", displayCurrency, wallets?.length],
    queryFn: async (): Promise<FiatBalance[]> => {
      const prices = await apiClient.get<Array<{ symbol: string; price: number; change24h: number }>>(
        "/api/v1/prices",
      )
      const usdToFiat =
        displayCurrency === "USD"
          ? 1
          : displayCurrency === "ZAR"
            ? (await apiClient.get<{ usdZar: number }>("/api/v1/prices/fx")).usdZar
            : null
      if (usdToFiat === null) return []

      const priceBySymbol = new Map(prices.map((p) => [p.symbol.toUpperCase(), p]))
      let totalFiat = 0
      let weightedChange = 0
      for (const w of wallets ?? []) {
        const spot = priceBySymbol.get(w.currency.toUpperCase())
        if (!spot) continue
        const value = parseFloat(w.balance) * spot.price * usdToFiat
        totalFiat += value
        weightedChange += spot.change24h * value
      }
      const changePercent = totalFiat > 0 ? weightedChange / totalFiat : 0
      return [{
        currency: displayCurrency,
        balance: totalFiat.toFixed(2),
        change24h: ((totalFiat * changePercent) / 100).toFixed(2),
        changePercent: changePercent.toFixed(2),
      }]
    },
    enabled: !!wallets?.length,
  })

  useEffect(() => {
    AsyncStorage.getItem(DISPLAY_CURRENCY_KEY).then((v) => {
      if (v && FIAT_CURRENCIES.includes(v as FiatCurrency)) {
        setDisplayCurrency(v as FiatCurrency)
      }
    })
  }, [])

  function handleCurrencySelect(c: FiatCurrency) {
    setDisplayCurrency(c)
    AsyncStorage.setItem(DISPLAY_CURRENCY_KEY, c)
  }

  const symbol = CURRENCY_SYMBOLS[displayCurrency]

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
        <View className="flex-1">
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Fiat balances
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            Portfolio value in your preferred currency
          </Text>
        </View>
      </View>

      <ScrollView className="flex-1 px-5" showsVerticalScrollIndicator={false}>
        <View className="h-1" />

        {/* Currency switcher */}
        <View className="mb-4">
          <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mb-2">
            Display currency
          </Text>
          <ScrollView horizontal showsHorizontalScrollIndicator={false}>
            <View className="flex-row gap-2">
              {FIAT_CURRENCIES.map((c) => (
                <TouchableOpacity
                  key={c}
                  onPress={() => handleCurrencySelect(c)}
                  className={`px-4 py-2.5 rounded-lg border ${
                    displayCurrency === c
                      ? "bg-brand border-brand"
                      : "border-border dark:border-border-dark bg-surface dark:bg-surface-dark"
                  }`}
                  activeOpacity={0.7}
                >
                  <Text className={`text-sm font-semibold ${displayCurrency === c ? "text-white" : "text-foreground dark:text-foreground-dark"}`}>
                    {c}
                  </Text>
                  <Text className={`text-xs ${displayCurrency === c ? "text-white/70" : "text-muted dark:text-muted-dark"}`}>
                    {CURRENCY_SYMBOLS[c]}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </ScrollView>
        </View>

        {/* Show equivalents toggle */}
        <View className="flex-row items-center justify-between mb-4 bg-surface dark:bg-card-dark rounded-2xl px-4 py-3 border border-border dark:border-border-dark">
          <View className="flex-row items-center gap-2">
            {showEquivalents ? (
              <Eye size={14} color="#64748B" />
            ) : (
              <EyeOff size={14} color="#64748B" />
            )}
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
              Show fiat equivalents
            </Text>
          </View>
          <Switch
            value={showEquivalents}
            onValueChange={setShowEquivalents}
            trackColor={{ false: "#E2E8F0", true: "#00A3F6" }}
            thumbColor="#fff"
          />
        </View>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : (
          <>
            {wallets?.map((wallet) => {
              const fiat = fiatBalances?.find((f) => f.currency === displayCurrency)
              const bal = parseFloat(wallet.balance)
              return (
                <TouchableOpacity
                  key={wallet.id}
                  onPress={() => router.push("/(app)/wallet")}
                  className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark"
                  activeOpacity={0.8}
                >
                  <View className="flex-row items-center justify-between mb-1">
                    <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                      {wallet.currency}
                    </Text>
                    <Text className="text-base font-bold text-foreground dark:text-foreground-dark">
                      {bal.toFixed(6)}
                    </Text>
                  </View>
                  {showEquivalents && fiat ? (
                    <View className="flex-row justify-between items-center mt-0.5">
                      <Text className="text-xs text-muted dark:text-muted-dark">
                        ≈ {symbol}
                        {parseFloat(fiat.balance).toLocaleString(undefined, {
                          maximumFractionDigits: 2,
                        })}{" "}
                        {displayCurrency}
                      </Text>
                      <View className="flex-row items-center gap-1">
                        {parseFloat(fiat.changePercent) >= 0 ? (
                          <TrendingUp size={11} color="#10B981" />
                        ) : (
                          <TrendingDown size={11} color="#EF4444" />
                        )}
                        <Text
                          className={`text-xs font-semibold ${
                            parseFloat(fiat.changePercent) >= 0 ? "text-success" : "text-error"
                          }`}
                        >
                          {parseFloat(fiat.changePercent) >= 0 ? "+" : ""}
                          {fiat.changePercent}%
                        </Text>
                      </View>
                    </View>
                  ) : (
                    <Text className="text-xs text-muted dark:text-muted-dark mt-0.5 capitalize">
                      {wallet.network}
                    </Text>
                  )}
                </TouchableOpacity>
              )
            })}

            {/* Total */}
            {showEquivalents && fiatBalances?.length ? (
              <View className="rounded-xl bg-brand-bg border border-brand/30 p-4 mt-2">
                <Text className="text-xs text-muted dark:text-muted-dark mb-1">Total portfolio value</Text>
                <Text className="text-2xl font-bold text-brand">
                  {symbol}
                  {(fiatBalances.reduce((sum, f) => sum + parseFloat(f.balance), 0))
                    .toLocaleString(undefined, { maximumFractionDigits: 2 })} {displayCurrency}
                </Text>
              </View>
            ) : null}
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
