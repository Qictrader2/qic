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
import { ArrowLeftRight, ChevronRight, History } from "lucide-react-native"
import { useActiveTrades } from "@/src/hooks/api/use-trade"
import { useAuthStore } from "@/src/store/auth-store"
import { SignInPrompt } from "@/src/components/common/SignInPrompt"
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
  const initials = trade.counterparty.username
    .split(/[\s_]/)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("")

  return (
    <TouchableOpacity
      onPress={onPress}
      className="bg-surface dark:bg-card-dark rounded-2xl p-4 mb-3 border border-border dark:border-border-dark"
      activeOpacity={0.85}
    >
      <View className="flex-row items-center gap-3">
        <View
          className="w-11 h-11 rounded-full items-center justify-center"
          style={{ backgroundColor: color + "22" }}
        >
          <Text className="text-sm font-bold" style={{ color }}>
            {initials}
          </Text>
        </View>
        <View className="flex-1">
          <View className="flex-row items-center justify-between">
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
              {trade.counterparty.username}
            </Text>
            <View
              className="px-2 py-0.5 rounded-full"
              style={{ backgroundColor: color + "22" }}
            >
              <Text className="text-[10px] font-semibold" style={{ color }}>
                {statusLabel(trade.status)}
              </Text>
            </View>
          </View>
          <View className="flex-row items-center justify-between mt-1">
            <Text className="text-xs text-muted dark:text-muted-dark capitalize">
              {isBuyer ? "Buying" : "Selling"} {trade.cryptoAmount} {trade.currency}
            </Text>
            <Text className="text-xs text-foreground dark:text-foreground-dark font-medium">
              {trade.fiatCurrency} {parseFloat(trade.fiatAmount).toLocaleString()}
            </Text>
          </View>
        </View>
        <ChevronRight size={16} color="#64748B" />
      </View>
    </TouchableOpacity>
  )
}

export default function TradesScreen() {
  const router = useRouter()
  const { isAuthenticated } = useAuthStore()
  const { data: trades, isLoading, error, refetch, isRefetching } = useActiveTrades()

  if (!isAuthenticated) {
    return (
      <SignInPrompt
        title="Your trades"
        message="Sign in to see your active trades, chat with counterparties, and access trade history."
      />
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["top"]}>
      <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
        <View>
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
            Trades
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            Active and recent trades
          </Text>
        </View>
        <TouchableOpacity
          onPress={() => router.push("/(app)/trade-history")}
          className="px-3 h-10 rounded-full bg-surface dark:bg-card-dark border border-border dark:border-border-dark items-center justify-center flex-row gap-1.5"
          activeOpacity={0.85}
        >
          <History size={14} color="#64748B" />
          <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
            History
          </Text>
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
          contentContainerStyle={{ paddingHorizontal: 20, paddingTop: 4, paddingBottom: 32 }}
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
                <ArrowLeftRight size={24} color="#00A3F6" />
              </View>
              <Text className="text-base font-semibold text-foreground dark:text-foreground-dark mb-1">
                No active trades
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark text-center max-w-[240px]">
                Head to the marketplace to find an offer and start your first trade.
              </Text>
              <TouchableOpacity
                onPress={() => router.push("/(tabs)/marketplace")}
                className="mt-5 px-5 py-2.5 rounded-xl bg-brand"
                activeOpacity={0.85}
              >
                <Text className="text-sm font-semibold text-white">Browse marketplace</Text>
              </TouchableOpacity>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
