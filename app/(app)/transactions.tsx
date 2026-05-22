import {
  View,
  Text,
  FlatList,
  ActivityIndicator,
  RefreshControl,
  TouchableOpacity,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
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

function TransactionRow({ tx }: { tx: Transaction }) {
  const color = txStatusColor(tx.status)
  const isIncoming = tx.type === "deposit" || tx.type === "trade_release"

  return (
    <View className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between">
        <View className="flex-row items-center gap-3">
          <View className="h-9 w-9 rounded-full bg-brand-bg items-center justify-center">
            <Text className="text-base">{txIcon(tx.type)}</Text>
          </View>
          <View>
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark capitalize">
              {tx.type.replace(/_/g, " ")}
            </Text>
            <Text className="text-xs text-muted dark:text-muted-dark">
              {new Date(tx.createdAt).toLocaleDateString()}
            </Text>
          </View>
        </View>
        <View className="items-end">
          <Text
            className={`text-sm font-semibold ${isIncoming ? "text-success" : "text-error"}`}
          >
            {isIncoming ? "+" : "-"}{tx.amount} {tx.currency}
          </Text>
          <Text className="text-xs font-medium" style={{ color }}>
            {tx.status}
          </Text>
        </View>
      </View>
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
