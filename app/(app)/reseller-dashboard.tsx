import {
  View, Text, ScrollView, TouchableOpacity, ActivityIndicator,
  Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { resellerService } from "@/src/services/reseller.service"
import { useRouter } from "expo-router"
import Slider from "@react-native-community/slider"
import { useState } from "react"

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <View className="flex-1 rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-3 items-center">
      <Text className="text-xs text-muted dark:text-muted-dark mb-1 text-center">{label}</Text>
      <Text className="text-sm font-bold text-foreground dark:text-foreground-dark text-center">{value}</Text>
    </View>
  )
}

export default function ResellerDashboardScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const [editingMarkup, setEditingMarkup] = useState(false)
  const [markupDraft, setMarkupDraft] = useState(2.5)

  const { data: stats, isLoading } = useQuery({
    queryKey: ["reseller-stats"],
    queryFn: () => resellerService.getStats(),
  })

  const { data: trades } = useQuery({
    queryKey: ["reseller-trades"],
    queryFn: () => resellerService.getTrades({ perPage: 10 }),
  })

  const { mutateAsync: saveMarkup, isPending: savingMarkup } = useMutation({
    mutationFn: (m: number) => resellerService.updateDefaultMarkup(m),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["reseller-stats"] })
      setEditingMarkup(false)
    },
  })

  function startEditMarkup() {
    setMarkupDraft(stats?.defaultMarkupPercentage ?? 2.5)
    setEditingMarkup(true)
  }

  const DISPLAY_STATE_COLORS: Record<string, string> = {
    pending: "text-warning",
    preview: "text-brand",
    settled: "text-success",
    contested: "text-error",
    "under-review": "text-warning",
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <View className="flex-row items-center justify-between mb-4">
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">Reseller</Text>
          <TouchableOpacity
            onPress={() => router.push("/(tabs)/marketplace")}
            className="px-3 py-1.5 rounded-lg bg-brand"
          >
            <Text className="text-xs font-semibold text-white">Browse to resell</Text>
          </TouchableOpacity>
        </View>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : stats ? (
          <>
            <View className="flex-row gap-3 mb-4">
              <StatCard label="Total earned" value={`${stats.totalEarnings} ${stats.currency}`} />
              <StatCard label="Active resells" value={String(stats.activeResells)} />
              <StatCard label="Success rate" value={`${stats.successRate}%`} />
            </View>

            {/* Default markup editor */}
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <View className="flex-row items-center justify-between mb-2">
                <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">Default markup</Text>
                {!editingMarkup ? (
                  <TouchableOpacity onPress={startEditMarkup} className="px-3 py-1 rounded-lg bg-brand/10">
                    <Text className="text-xs font-medium text-brand">Edit</Text>
                  </TouchableOpacity>
                ) : (
                  <TouchableOpacity
                    onPress={() => saveMarkup(markupDraft)}
                    disabled={savingMarkup}
                    className="px-3 py-1 rounded-lg bg-brand"
                  >
                    {savingMarkup ? <ActivityIndicator color="#fff" size="small" /> : (
                      <Text className="text-xs font-medium text-white">Save</Text>
                    )}
                  </TouchableOpacity>
                )}
              </View>
              <Text className="text-3xl font-bold text-brand mb-2">
                {editingMarkup ? markupDraft.toFixed(1) : stats.defaultMarkupPercentage.toFixed(1)}%
              </Text>
              {editingMarkup ? (
                <>
                  <Slider
                    minimumValue={0}
                    maximumValue={50}
                    step={0.1}
                    value={markupDraft}
                    onValueChange={setMarkupDraft}
                    minimumTrackTintColor="#00A3F6"
                    maximumTrackTintColor="#E2E8F0"
                    thumbTintColor="#00A3F6"
                  />
                  <View className="flex-row justify-between">
                    <Text className="text-xs text-muted dark:text-muted-dark">0%</Text>
                    <Text className="text-xs text-muted dark:text-muted-dark">50%</Text>
                  </View>
                </>
              ) : (
                <Text className="text-xs text-muted dark:text-muted-dark">
                  Applied automatically to new resell offers
                </Text>
              )}
            </View>

            {/* Recent resell trades */}
            {(trades?.length ?? 0) > 0 ? (
              <View>
                <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mb-3">Recent commissions</Text>
                {trades?.map((t) => (
                  <TouchableOpacity
                    key={t.id}
                    onPress={() => router.push({ pathname: "/(app)/trade/[id]", params: { id: t.tradeId } })}
                    className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark"
                    activeOpacity={0.8}
                  >
                    <View className="flex-row justify-between items-center mb-1">
                      <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                        {t.cryptoAmount} {t.cryptocurrency}
                      </Text>
                      <Text className={`text-xs font-medium capitalize ${DISPLAY_STATE_COLORS[t.displayState] ?? "text-muted"}`}>
                        {t.displayState}
                      </Text>
                    </View>
                    <View className="flex-row justify-between">
                      <Text className="text-xs text-muted dark:text-muted-dark">
                        {t.markupPercentage}% markup
                      </Text>
                      <Text className="text-xs font-medium text-success">
                        +{t.resellerProfit} {t.fiatCurrency}
                      </Text>
                    </View>
                  </TouchableOpacity>
                ))}
              </View>
            ) : (
              <View className="items-center py-10">
                <Text className="text-3xl mb-3">💹</Text>
                <Text className="text-sm font-medium text-foreground dark:text-foreground-dark mb-2">No commissions yet</Text>
                <Text className="text-xs text-muted dark:text-muted-dark text-center">
                  Go to Marketplace, find an offer, and tap Resell to start.
                </Text>
              </View>
            )}
          </>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  )
}
