import {
  View,
  Text,
  ScrollView,
  RefreshControl,
  TouchableOpacity,
  ActivityIndicator,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useWallets } from "@/src/hooks/api/use-wallet"
import type { Wallet } from "@/src/services/wallet.service"

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

function WalletCard({ wallet, onDeposit, onWithdraw }: {
  wallet: Wallet
  onDeposit: () => void
  onWithdraw: () => void
}) {
  const color = CURRENCY_COLORS[wallet.currency] ?? "#00A3F6"
  const symbol = CURRENCY_SYMBOLS[wallet.currency] ?? wallet.currency

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

export default function WalletScreen() {
  const router = useRouter()
  const { data: wallets, isLoading, error, refetch, isRefetching } = useWallets()

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">Wallet</Text>
        <TouchableOpacity
          onPress={() => router.push("/(app)/transactions")}
          className="px-3 py-1.5 rounded-lg bg-brand-bg"
        >
          <Text className="text-xs font-medium text-brand">History</Text>
        </TouchableOpacity>
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
