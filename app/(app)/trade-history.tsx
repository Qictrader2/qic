import { View, Text, ScrollView, TouchableOpacity, FlatList, ActivityIndicator } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useTradeHistory } from "@/src/hooks/api/use-trade"
import { useRouter } from "expo-router"
import type { Trade, TradeStatus } from "@/src/services/trade.service"

function statusColor(s: TradeStatus): string {
  switch (s) {
    case "completed": return "#10B981"
    case "cancelled":
    case "expired": return "#6B7280"
    case "disputed": return "#EF4444"
    default: return "#F59E0B"
  }
}

function TradeHistoryRow({ trade, onPress }: { trade: Trade; onPress: () => void }) {
  return (
    <TouchableOpacity
      onPress={onPress}
      className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark"
      activeOpacity={0.8}
    >
      <View className="flex-row items-center justify-between mb-1">
        <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
          {trade.role === "buyer" ? "Bought" : "Sold"} {trade.cryptoAmount} {trade.currency}
        </Text>
        <Text className="text-xs font-medium" style={{ color: statusColor(trade.status) }}>
          {trade.status}
        </Text>
      </View>
      <View className="flex-row justify-between">
        <Text className="text-xs text-muted dark:text-muted-dark">
          with {trade.counterparty.username}
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark">
          {new Date(trade.createdAt).toLocaleDateString()}
        </Text>
      </View>
    </TouchableOpacity>
  )
}

export default function TradeHistoryScreen() {
  const router = useRouter()
  const { data: trades, isLoading } = useTradeHistory()

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">Trade History</Text>
      </View>
      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <FlatList
          data={trades ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <TradeHistoryRow
              trade={item}
              onPress={() => router.push({ pathname: "/(app)/trade/[id]", params: { id: item.id } })}
            />
          )}
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 24 }}
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">📜</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark">No trade history</Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
