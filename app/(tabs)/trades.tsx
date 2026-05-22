import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useActiveTrades } from "@/src/hooks/api/use-trade"
import type { Trade, TradeStatus } from "@/src/services/trade.service"

function statusColor(status: TradeStatus): string {
  switch (status) {
    case "initiated": return "#3B82F6"
    case "funded": return "#F59E0B"
    case "payment_pending": return "#F59E0B"
    case "payment_sent": return "#8B5CF6"
    case "payment_confirmed": return "#10B981"
    case "completed": return "#10B981"
    case "cancelled": return "#6B7280"
    case "disputed": return "#EF4444"
    case "expired": return "#6B7280"
    default: {
      const _exhaustive: never = status
      return _exhaustive
    }
  }
}

function statusLabel(status: TradeStatus): string {
  switch (status) {
    case "initiated": return "Initiated"
    case "funded": return "Funded"
    case "payment_pending": return "Awaiting payment"
    case "payment_sent": return "Payment sent"
    case "payment_confirmed": return "Payment confirmed"
    case "completed": return "Completed"
    case "cancelled": return "Cancelled"
    case "disputed": return "Disputed"
    case "expired": return "Expired"
    default: {
      const _exhaustive: never = status
      return _exhaustive
    }
  }
}

function TradeRow({ trade, onPress }: { trade: Trade; onPress: () => void }) {
  const color = statusColor(trade.status)
  const isBuyer = trade.role === "buyer"

  return (
    <TouchableOpacity
      onPress={onPress}
      className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark"
      activeOpacity={0.8}
    >
      <View className="flex-row items-center justify-between mb-2">
        <View className="flex-row items-center gap-2">
          <View className={`px-2 py-0.5 rounded-full`} style={{ backgroundColor: color + "20" }}>
            <Text className="text-xs font-semibold" style={{ color }}>
              {statusLabel(trade.status)}
            </Text>
          </View>
          <Text className="text-xs text-muted dark:text-muted-dark capitalize">
            {isBuyer ? "Buying" : "Selling"}
          </Text>
        </View>
        <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
          {trade.cryptoAmount} {trade.currency}
        </Text>
      </View>

      <View className="flex-row items-center justify-between">
        <Text className="text-xs text-muted dark:text-muted-dark">
          with {trade.counterparty.username}
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark">
          {trade.fiatCurrency} {parseFloat(trade.fiatAmount).toLocaleString()}
        </Text>
      </View>
    </TouchableOpacity>
  )
}

export default function TradesScreen() {
  const router = useRouter()
  const { data: trades, isLoading, error, refetch, isRefetching } = useActiveTrades()

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">Trades</Text>
        <TouchableOpacity
          onPress={() => router.push("/(app)/trade-history")}
          className="px-3 py-1.5 rounded-lg bg-brand-bg"
        >
          <Text className="text-xs font-medium text-brand">History</Text>
        </TouchableOpacity>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : error ? (
        <View className="mx-4 rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">Failed to load trades.</Text>
        </View>
      ) : (
        <FlatList
          data={trades ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <TradeRow
              trade={item}
              onPress={() =>
                router.push({ pathname: "/(app)/trade/[id]", params: { id: item.id } })
              }
            />
          )}
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 24 }}
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">🔄</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark mb-1">
                No active trades
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark">
                Visit the Marketplace to start trading
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
