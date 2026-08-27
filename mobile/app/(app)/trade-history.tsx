import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  FlatList,
  ActivityIndicator,
  RefreshControl,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useMemo } from "react"
import { useRouter } from "expo-router"
import {
  ChevronLeft,
  ChevronRight,
  History,
  ArrowUpFromLine,
  ArrowDownToLine,
} from "lucide-react-native"
import { useTradeHistory } from "@/src/hooks/api/use-trade"
import type { Trade, TradeStatus } from "@/src/services/trade.service"

const FILTERS = ["all", "completed", "cancelled", "disputed"] as const
type Filter = (typeof FILTERS)[number]

const FILTER_LABELS: Record<Filter, string> = {
  all: "All",
  completed: "Completed",
  cancelled: "Cancelled",
  disputed: "Disputed",
}

function statusColor(s: TradeStatus): string {
  switch (s) {
    case "completed":
      return "#10B981"
    case "cancelled":
    case "expired":
      return "#6B7280"
    case "disputed":
      return "#EF4444"
    default:
      return "#F59E0B"
  }
}

function statusLabel(s: TradeStatus): string {
  return s.replace(/_/g, " ")
}

function FilterChip({
  label,
  active,
  onPress,
}: {
  label: string
  active: boolean
  onPress: () => void
}) {
  return (
    <TouchableOpacity
      onPress={onPress}
      activeOpacity={0.85}
      className={`px-3 py-1.5 rounded-full border ${
        active
          ? "bg-brand border-brand"
          : "bg-surface dark:bg-card-dark border-border dark:border-border-dark"
      }`}
    >
      <Text
        className={`text-xs font-medium ${
          active ? "text-white" : "text-foreground dark:text-foreground-dark"
        }`}
      >
        {label}
      </Text>
    </TouchableOpacity>
  )
}

function TradeHistoryRow({ trade, onPress }: { trade: Trade; onPress: () => void }) {
  const color = statusColor(trade.status)
  const isBuyer = trade.role === "buyer"
  const Icon = isBuyer ? ArrowDownToLine : ArrowUpFromLine

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
          <Icon size={16} color={color} />
        </View>
        <View className="flex-1">
          <View className="flex-row items-center justify-between">
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
              {isBuyer ? "Bought" : "Sold"} {trade.cryptoAmount} {trade.currency}
            </Text>
            <View
              className="px-1.5 py-0.5 rounded-full"
              style={{ backgroundColor: color + "22" }}
            >
              <Text className="text-[10px] font-semibold capitalize" style={{ color }}>
                {statusLabel(trade.status)}
              </Text>
            </View>
          </View>
          <View className="flex-row items-center justify-between mt-1">
            <Text className="text-xs text-muted dark:text-muted-dark">
              with{" "}
              <Text className="text-foreground dark:text-foreground-dark font-medium">
                {trade.counterparty.username}
              </Text>
            </Text>
            <Text className="text-xs text-foreground dark:text-foreground-dark font-medium">
              {trade.fiatCurrency} {parseFloat(trade.fiatAmount).toLocaleString()}
            </Text>
          </View>
          <Text className="text-[11px] text-muted dark:text-muted-dark mt-1">
            {new Date(trade.createdAt).toLocaleDateString(undefined, {
              year: "numeric",
              month: "short",
              day: "numeric",
            })}
          </Text>
        </View>
        <ChevronRight size={14} color="#94A3B8" />
      </View>
    </TouchableOpacity>
  )
}

export default function TradeHistoryScreen() {
  const router = useRouter()
  const [filter, setFilter] = useState<Filter>("all")
  const { data: trades, isLoading, refetch, isRefetching } = useTradeHistory()

  const filtered = useMemo(() => {
    if (!trades) return []
    if (filter === "all") return trades
    return trades.filter((t) => t.status === filter)
  }, [trades, filter])

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
            Trade history
          </Text>
        </View>
      </View>

      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={{ paddingHorizontal: 20, paddingBottom: 12, gap: 8 }}
      >
        {FILTERS.map((f) => (
          <FilterChip
            key={f}
            label={FILTER_LABELS[f]}
            active={filter === f}
            onPress={() => setFilter(f)}
          />
        ))}
      </ScrollView>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <FlatList
          data={filtered}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <TradeHistoryRow
              trade={item}
              onPress={() =>
                router.push({ pathname: "/(app)/trade/[id]", params: { id: item.id } })
              }
            />
          )}
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          contentContainerStyle={{ paddingHorizontal: 20, paddingBottom: 32 }}
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
                <History size={24} color="#00A3F6" />
              </View>
              <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
                No trade history
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark mt-1 text-center max-w-[240px]">
                {filter === "all"
                  ? "Your completed and past trades will appear here."
                  : `No ${FILTER_LABELS[filter].toLowerCase()} trades yet.`}
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
