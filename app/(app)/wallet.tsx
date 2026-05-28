import {
  View,
  Text,
  ScrollView,
  RefreshControl,
  TouchableOpacity,
  ActivityIndicator,
  Dimensions,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { LineChart } from "react-native-chart-kit"
import { useWallets } from "@/src/hooks/api/use-wallet"
import { apiClient } from "@/src/lib/api/client"
import type { Wallet } from "@/src/services/wallet.service"

const SCREEN_WIDTH = Dimensions.get("window").width

const CURRENCY_COLORS: Record<string, string> = {
  BTC: "#F7931A",
  ETH: "#627EEA",
  SOL: "#9945FF",
  USDT: "#26A17B",
  USDC: "#2775CA",
}

const CURRENCY_SYMBOLS: Record<string, string> = {
  BTC: "₿",
  ETH: "Ξ",
  SOL: "◎",
  USDT: "₮",
  USDC: "$",
}

interface PriceHistory {
  currency: string
  prices: number[] // last 7 data points
  change24h: number
  currentPrice: number
}

function usePriceHistory(currency: string, enabled: boolean) {
  return useQuery({
    queryKey: ["price-history", currency],
    queryFn: () => apiClient.get<PriceHistory>(`/api/v1/market/price-history?currency=${currency}&period=7d`),
    enabled,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
  })
}

function MiniSparkline({ prices, color }: { prices: number[]; color: string }) {
  if (prices.length < 2) return null
  return (
    <LineChart
      data={{ labels: [], datasets: [{ data: prices, color: () => color, strokeWidth: 2 }] }}
      width={80}
      height={32}
      withDots={false}
      withInnerLines={false}
      withOuterLines={false}
      withHorizontalLabels={false}
      withVerticalLabels={false}
      chartConfig={{
        backgroundGradientFrom: "transparent",
        backgroundGradientTo: "transparent",
        color: () => color,
        strokeWidth: 2,
      }}
      bezier
      style={{ paddingRight: 0, paddingTop: 0 }}
    />
  )
}

function WalletCard({ wallet, onDeposit, onWithdraw }: {
  wallet: Wallet
  onDeposit: () => void
  onWithdraw: () => void
}) {
  const color = CURRENCY_COLORS[wallet.currency] ?? "#00A3F6"
  const symbol = CURRENCY_SYMBOLS[wallet.currency] ?? wallet.currency
  const { data: history } = usePriceHistory(wallet.currency, true)

  return (
    <View className="rounded-xl bg-surface dark:bg-surface-dark p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between mb-3">
        <View className="flex-row items-center gap-2">
          <View
            className="h-10 w-10 rounded-full items-center justify-center"
            style={{ backgroundColor: color + "20" }}
          >
            <Text className="text-base font-bold" style={{ color }}>{symbol}</Text>
          </View>
          <View>
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
              {wallet.currency}
            </Text>
            <Text className="text-xs text-muted dark:text-muted-dark capitalize">
              {wallet.network}
            </Text>
          </View>
        </View>

        <View className="flex-row items-center gap-3">
          {history?.prices && history.prices.length >= 2 ? (
            <View className="items-end">
              <MiniSparkline prices={history.prices} color={history.change24h >= 0 ? "#10B981" : "#EF4444"} />
              <Text
                className="text-xs font-medium"
                style={{ color: history.change24h >= 0 ? "#10B981" : "#EF4444" }}
              >
                {history.change24h >= 0 ? "+" : ""}{history.change24h.toFixed(2)}%
              </Text>
            </View>
          ) : null}
          <View className="items-end">
            <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
              {parseFloat(wallet.availableBalance).toFixed(8).replace(/\.?0+$/, "")}
            </Text>
            {parseFloat(wallet.lockedBalance) > 0 ? (
              <Text className="text-xs text-warning">
                {wallet.lockedBalance} locked
              </Text>
            ) : null}
          </View>
        </View>
      </View>

      <View className="flex-row gap-2">
        <TouchableOpacity
          onPress={onDeposit}
          className="flex-1 rounded-lg bg-brand-bg py-2.5 items-center"
          activeOpacity={0.8}
        >
          <Text className="text-sm font-medium text-brand">Deposit</Text>
        </TouchableOpacity>
        <TouchableOpacity
          onPress={onWithdraw}
          className="flex-1 rounded-lg border border-border dark:border-border-dark py-2.5 items-center"
          activeOpacity={0.8}
        >
          <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
            Withdraw
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  )
}

/** Portfolio total value chart */
function PortfolioChart({ wallets }: { wallets: Wallet[] }) {
  const { data: portfolioHistory, isLoading } = useQuery({
    queryKey: ["portfolio-history"],
    queryFn: () => apiClient.get<{ values: number[]; labels: string[]; totalUsd: number; change24h: number }>(
      "/api/v1/market/portfolio-history?period=7d"
    ),
    staleTime: 60_000,
  })

  if (isLoading) {
    return (
      <View className="h-36 items-center justify-center">
        <ActivityIndicator color="#00A3F6" />
      </View>
    )
  }

  if (!portfolioHistory?.values || portfolioHistory.values.length < 2) return null

  const change24h = portfolioHistory.change24h ?? 0
  const chartColor = change24h >= 0 ? "#10B981" : "#EF4444"

  return (
    <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
      <View className="flex-row justify-between items-start mb-3">
        <View>
          <Text className="text-xs text-muted dark:text-muted-dark">Portfolio value</Text>
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
            ${portfolioHistory.totalUsd.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </Text>
        </View>
        <View
          className="px-2 py-1 rounded-full"
          style={{ backgroundColor: chartColor + "20" }}
        >
          <Text className="text-xs font-semibold" style={{ color: chartColor }}>
            {change24h >= 0 ? "+" : ""}{change24h.toFixed(2)}% 24h
          </Text>
        </View>
      </View>

      <LineChart
        data={{
          labels: portfolioHistory.labels ?? [],
          datasets: [{ data: portfolioHistory.values, color: () => chartColor, strokeWidth: 2 }],
        }}
        width={SCREEN_WIDTH - 64}
        height={100}
        withDots={false}
        withInnerLines={false}
        withOuterLines={false}
        withHorizontalLabels={false}
        withVerticalLabels={false}
        chartConfig={{
          backgroundGradientFrom: "transparent",
          backgroundGradientTo: "transparent",
          color: () => chartColor,
          strokeWidth: 2,
          propsForBackgroundLines: { stroke: "transparent" },
        }}
        bezier
        style={{ paddingRight: 0, marginLeft: -16 }}
      />
    </View>
  )
}

export default function WalletScreen() {
  const router = useRouter()
  const { data: wallets, isLoading, error, refetch, isRefetching } = useWallets()

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">Wallet</Text>
        <View className="flex-row gap-2">
          <TouchableOpacity
            onPress={() => router.push("/(app)/fiat-balance")}
            className="px-3 py-1.5 rounded-lg bg-brand-bg"
          >
            <Text className="text-xs font-medium text-brand">Fiat ≈</Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => router.push("/(app)/transactions")}
            className="px-3 py-1.5 rounded-lg bg-surface dark:bg-surface-dark border border-border dark:border-border-dark"
          >
            <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">History</Text>
          </TouchableOpacity>
        </View>
      </View>

      <ScrollView
        className="flex-1 px-4"
        refreshControl={
          <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
        }
        showsVerticalScrollIndicator={false}
      >
        {isLoading ? (
          <View className="flex-1 items-center justify-center py-20">
            <ActivityIndicator color="#00A3F6" />
          </View>
        ) : error ? (
          <View className="rounded-xl bg-error-bg p-4 mt-4">
            <Text className="text-sm text-error text-center">
              Failed to load wallets. Pull to retry.
            </Text>
          </View>
        ) : !wallets?.length ? (
          <View className="items-center justify-center py-20">
            <Text className="text-4xl mb-3">💳</Text>
            <Text className="text-base font-medium text-foreground dark:text-foreground-dark">
              No wallets yet
            </Text>
          </View>
        ) : (
          <>
            <PortfolioChart wallets={wallets} />

            {wallets.map((wallet) => (
              <WalletCard
                key={wallet.id}
                wallet={wallet}
                onDeposit={() =>
                  router.push({
                    pathname: "/(app)/deposit",
                    params: { currency: wallet.currency, network: wallet.network },
                  })
                }
                onWithdraw={() =>
                  router.push({
                    pathname: "/(app)/withdraw",
                    params: { currency: wallet.currency, network: wallet.network },
                  })
                }
              />
            ))}
            <View className="h-8" />
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
