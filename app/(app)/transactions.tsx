import {
  View,
  Text,
  FlatList,
  ActivityIndicator,
  RefreshControl,
  TouchableOpacity,
  Linking,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useTransactions } from "@/src/hooks/api/use-wallet"
import type { Transaction, TxStatus, TxType } from "@/src/services/wallet.service"

function txStatusColor(status: TxStatus): string {
  switch (status) {
    case "confirmed": return "#10B981"
    case "pending":
    case "confirming": return "#F59E0B"
    case "failed": return "#EF4444"
    default: {
      const _exhaustive: never = status
      return _exhaustive
    }
  }
}

function txIcon(type: TxType): string {
  switch (type) {
    case "deposit": return "↓"
    case "withdrawal": return "↑"
    case "transfer": return "⇄"
    case "trade_lock": return "🔒"
    case "trade_release": return "🔓"
    default: {
      const _exhaustive: never = type
      return _exhaustive
    }
  }
}

// Block explorer URLs keyed by network
const EXPLORER_TX_URL: Record<string, (txHash: string) => string> = {
  bitcoin: (h) => `https://mempool.space/tx/${h}`,
  erc20: (h) => `https://etherscan.io/tx/${h}`,
  trc20: (h) => `https://tronscan.org/#/transaction/${h}`,
  spl: (h) => `https://solscan.io/tx/${h}`,
  solana: (h) => `https://solscan.io/tx/${h}`,
}

function TransactionRow({ tx }: { tx: Transaction }) {
  const color = txStatusColor(tx.status)
  const isIncoming = tx.type === "deposit" || tx.type === "trade_release"
  const explorerUrl = tx.txHash && tx.network ? EXPLORER_TX_URL[tx.network]?.(tx.txHash) : null

  return (
    <View className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between">
        <View className="flex-row items-center gap-3 flex-1">
          <View className="h-9 w-9 rounded-full bg-brand-bg items-center justify-center flex-shrink-0">
            <Text className="text-base">{txIcon(tx.type)}</Text>
          </View>
          <View className="flex-1">
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark capitalize">
              {tx.type.replace(/_/g, " ")}
            </Text>
            <Text className="text-xs text-muted dark:text-muted-dark">
              {new Date(tx.createdAt).toLocaleDateString(undefined, {
                month: "short",
                day: "numeric",
                hour: "2-digit",
                minute: "2-digit",
              })}
            </Text>
          </View>
        </View>
        <View className="items-end ml-3">
          <Text className={`text-sm font-semibold ${isIncoming ? "text-success" : "text-error"}`}>
            {isIncoming ? "+" : "-"}{tx.amount} {tx.currency}
          </Text>
          <Text className="text-xs font-medium" style={{ color }}>
            {tx.status}
          </Text>
        </View>
      </View>

      {/* TX hash + explorer link */}
      {tx.txHash ? (
        <View className="mt-3 pt-3 border-t border-border/30 dark:border-border-dark/30 flex-row items-center justify-between">
          <Text className="text-xs text-muted dark:text-muted-dark font-mono flex-1 mr-2" numberOfLines={1}>
            {tx.txHash.slice(0, 20)}…
          </Text>
          {explorerUrl ? (
            <TouchableOpacity onPress={() => Linking.openURL(explorerUrl)} activeOpacity={0.7}>
              <Text className="text-xs text-brand">View ↗</Text>
            </TouchableOpacity>
          ) : null}
        </View>
      ) : null}
    </View>
  )
}

export default function TransactionsScreen() {
  const { data: txs, isLoading, error, refetch, isRefetching } = useTransactions()

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
          Transactions
        </Text>
      </View>
      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : error ? (
        <View className="mx-4 rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">Failed to load transactions. Pull to retry.</Text>
        </View>
      ) : (
        <FlatList
          data={txs ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => <TransactionRow tx={item} />}
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 24 }}
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">📋</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark">
                No transactions yet
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
