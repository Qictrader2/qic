import {
  View,
  Text,
  FlatList,
  ActivityIndicator,
  RefreshControl,
  TouchableOpacity,
  Linking,
  ScrollView,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useMemo } from "react"
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  ArrowLeftRight,
  Lock,
  Unlock,
  ExternalLink,
  Receipt,
} from "lucide-react-native"
import { useTransactions } from "@/src/hooks/api/use-wallet"
import type { Transaction, TxStatus, TxType } from "@/src/services/wallet.service"

const TYPE_FILTERS = ["all", "deposit", "withdrawal", "transfer", "trade"] as const
type TypeFilter = (typeof TYPE_FILTERS)[number]

const TYPE_FILTER_LABELS: Record<TypeFilter, string> = {
  all: "All",
  deposit: "Deposits",
  withdrawal: "Withdrawals",
  transfer: "Transfers",
  trade: "Trade locks",
}

const EXPLORER_TX_URL: Record<string, (txHash: string) => string> = {
  bitcoin: (h) => `https://mempool.space/tx/${h}`,
  erc20: (h) => `https://etherscan.io/tx/${h}`,
  trc20: (h) => `https://tronscan.org/#/transaction/${h}`,
  spl: (h) => `https://solscan.io/tx/${h}`,
  solana: (h) => `https://solscan.io/tx/${h}`,
}

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

function txIconConfig(type: TxType): { Icon: typeof ArrowDownToLine; color: string } {
  switch (type) {
    case "deposit": return { Icon: ArrowDownToLine, color: "#10B981" }
    case "withdrawal": return { Icon: ArrowUpFromLine, color: "#EF4444" }
    case "transfer": return { Icon: ArrowLeftRight, color: "#3B82F6" }
    case "trade_lock": return { Icon: Lock, color: "#F59E0B" }
    case "trade_release": return { Icon: Unlock, color: "#10B981" }
    default: {
      const _exhaustive: never = type
      return _exhaustive
    }
  }
}

function txLabel(type: TxType): string {
  return {
    deposit: "Deposit",
    withdrawal: "Withdrawal",
    transfer: "Internal transfer",
    trade_lock: "Trade lock",
    trade_release: "Trade release",
  }[type]
}

function formatRelative(d: string): string {
  const date = new Date(d)
  const diff = Date.now() - date.getTime()
  const mins = Math.floor(diff / 60_000)
  if (mins < 1) return "just now"
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}d ago`
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric" })
}

function TransactionRow({ tx }: { tx: Transaction }) {
  const statusColor = txStatusColor(tx.status)
  const { Icon, color } = txIconConfig(tx.type)
  const isIncoming = tx.type === "deposit" || tx.type === "trade_release"
  const explorerUrl = tx.txHash && tx.network ? EXPLORER_TX_URL[tx.network]?.(tx.txHash) : null

  return (
    <View className="bg-surface dark:bg-card-dark rounded-2xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center gap-3">
        <View
          className="w-11 h-11 rounded-full items-center justify-center"
          style={{ backgroundColor: color + "22" }}
        >
          <Icon size={18} color={color} />
        </View>
        <View className="flex-1">
          <View className="flex-row items-center justify-between">
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
              {txLabel(tx.type)}
            </Text>
            <Text
              className={`text-sm font-semibold ${isIncoming ? "text-success" : "text-foreground dark:text-foreground-dark"}`}
            >
              {isIncoming ? "+" : "-"}
              {tx.amount} {tx.currency}
            </Text>
          </View>
          <View className="flex-row items-center justify-between mt-0.5">
            <Text className="text-xs text-muted dark:text-muted-dark">
              {formatRelative(tx.createdAt)}
            </Text>
            <View
              className="px-1.5 py-0.5 rounded-full"
              style={{ backgroundColor: statusColor + "22" }}
            >
              <Text className="text-[10px] font-semibold capitalize" style={{ color: statusColor }}>
                {tx.status}
              </Text>
            </View>
          </View>
        </View>
      </View>

      {tx.txHash ? (
        <View className="mt-3 pt-3 border-t border-border/40 dark:border-border-dark/40 flex-row items-center justify-between">
          <Text
            className="text-[11px] text-muted dark:text-muted-dark font-mono flex-1 mr-2"
            numberOfLines={1}
          >
            {tx.txHash.slice(0, 12)}…{tx.txHash.slice(-6)}
          </Text>
          {explorerUrl ? (
            <TouchableOpacity
              onPress={() => Linking.openURL(explorerUrl)}
              className="flex-row items-center gap-1"
              activeOpacity={0.7}
            >
              <Text className="text-[11px] text-brand font-medium">View</Text>
              <ExternalLink size={11} color="#00A3F6" />
            </TouchableOpacity>
          ) : null}
        </View>
      ) : null}
    </View>
  )
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

export default function TransactionsScreen() {
  const [filter, setFilter] = useState<TypeFilter>("all")
  const { data: txs, isLoading, error, refetch, isRefetching } = useTransactions()

  const filtered = useMemo(() => {
    if (!txs) return []
    if (filter === "all") return txs
    if (filter === "trade") {
      return txs.filter((t) => t.type === "trade_lock" || t.type === "trade_release")
    }
    return txs.filter((t) => t.type === filter)
  }, [txs, filter])

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <View className="px-5 pt-2 pb-3">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
          Transactions
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
          Deposits, withdrawals, transfers and trade activity
        </Text>
      </View>

      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={{ paddingHorizontal: 20, paddingBottom: 12, gap: 8 }}
      >
        {TYPE_FILTERS.map((f) => (
          <FilterChip
            key={f}
            label={TYPE_FILTER_LABELS[f]}
            active={filter === f}
            onPress={() => setFilter(f)}
          />
        ))}
      </ScrollView>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : error ? (
        <View className="mx-5 rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">
            Failed to load transactions. Pull to retry.
          </Text>
        </View>
      ) : (
        <FlatList
          data={filtered}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => <TransactionRow tx={item} />}
          contentContainerStyle={{ paddingHorizontal: 20, paddingBottom: 32 }}
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
                <Receipt size={24} color="#00A3F6" />
              </View>
              <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
                No transactions
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark mt-1 text-center max-w-[240px]">
                {filter === "all"
                  ? "Your deposits, withdrawals, and trade activity will appear here."
                  : `No ${TYPE_FILTER_LABELS[filter].toLowerCase()} yet.`}
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
