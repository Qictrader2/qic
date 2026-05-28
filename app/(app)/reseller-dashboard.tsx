import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { resellerService } from "@/src/services/reseller.service"
import { useRouter } from "expo-router"
import Slider from "@react-native-community/slider"
import { useState } from "react"
import {
  TrendingUp,
  Store,
  Percent,
  CheckCircle2,
  ChevronRight,
  Wallet as WalletIcon,
  Sparkles,
  Pencil,
  X,
} from "lucide-react-native"

const STATE_COLORS: Record<string, string> = {
  pending: "#F59E0B",
  preview: "#00A3F6",
  settled: "#10B981",
  contested: "#EF4444",
  "under-review": "#F59E0B",
  released: "#10B981",
}

function StatCard({
  label,
  value,
  Icon,
  color,
}: {
  label: string
  value: string
  Icon: typeof TrendingUp
  color: string
}) {
  return (
    <View className="flex-1 rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
      <View
        className="w-9 h-9 rounded-xl items-center justify-center mb-2"
        style={{ backgroundColor: color + "22" }}
      >
        <Icon size={16} color={color} />
      </View>
      <Text className="text-base font-bold text-foreground dark:text-foreground-dark" numberOfLines={1}>
        {value}
      </Text>
      <Text className="text-[11px] text-muted dark:text-muted-dark mt-0.5" numberOfLines={1}>
        {label}
      </Text>
    </View>
  )
}

export default function ResellerDashboardScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const [editingMarkup, setEditingMarkup] = useState(false)
  const [markupDraft, setMarkupDraft] = useState(2.5)

  const { data: stats, isLoading, refetch, isRefetching } = useQuery({
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

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <ScrollView
        className="flex-1"
        refreshControl={
          <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
        }
        showsVerticalScrollIndicator={false}
      >
        <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
          <View>
            <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
              Reseller
            </Text>
            <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
              Earn commissions on resold offers
            </Text>
          </View>
          <TouchableOpacity
            onPress={() => router.push("/(tabs)/marketplace" as never)}
            className="px-3 h-10 rounded-full bg-brand items-center justify-center flex-row gap-1"
            activeOpacity={0.85}
          >
            <Sparkles size={14} color="#FFFFFF" />
            <Text className="text-xs font-semibold text-white">Find offers</Text>
          </TouchableOpacity>
        </View>

        {isLoading ? (
          <View className="items-center py-20">
            <ActivityIndicator color="#00A3F6" />
          </View>
        ) : stats ? (
          <View className="px-5">
            <View className="flex-row gap-3 mb-4">
              <StatCard
                label="Total earned"
                value={`${stats.totalEarnings} ${stats.currency}`}
                Icon={WalletIcon}
                color="#10B981"
              />
              <StatCard
                label="Active resells"
                value={String(stats.activeResells)}
                Icon={Store}
                color="#00A3F6"
              />
            </View>
            <View className="flex-row gap-3 mb-5">
              <StatCard
                label="Success rate"
                value={`${stats.successRate}%`}
                Icon={CheckCircle2}
                color="#3B82F6"
              />
              <StatCard
                label="Default markup"
                value={`${stats.defaultMarkupPercentage.toFixed(1)}%`}
                Icon={Percent}
                color="#F59E0B"
              />
            </View>

            {/* Markup editor */}
            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-5">
              <View className="flex-row items-center justify-between mb-2">
                <View>
                  <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                    Default markup
                  </Text>
                  <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
                    Applied automatically to new resells
                  </Text>
                </View>
                {!editingMarkup ? (
                  <TouchableOpacity
                    onPress={startEditMarkup}
                    className="px-3 h-8 rounded-full bg-brand/10 flex-row items-center gap-1"
                    activeOpacity={0.85}
                  >
                    <Pencil size={12} color="#00A3F6" />
                    <Text className="text-xs font-semibold text-brand">Edit</Text>
                  </TouchableOpacity>
                ) : (
                  <View className="flex-row gap-2">
                    <TouchableOpacity
                      onPress={() => setEditingMarkup(false)}
                      className="w-8 h-8 rounded-full bg-surface dark:bg-card-dark border border-border dark:border-border-dark items-center justify-center"
                      activeOpacity={0.7}
                    >
                      <X size={14} color="#64748B" />
                    </TouchableOpacity>
                    <TouchableOpacity
                      onPress={() => saveMarkup(markupDraft)}
                      disabled={savingMarkup}
                      className="px-3 h-8 rounded-full bg-brand flex-row items-center"
                      activeOpacity={0.85}
                    >
                      {savingMarkup ? (
                        <ActivityIndicator color="#fff" size="small" />
                      ) : (
                        <Text className="text-xs font-semibold text-white">Save</Text>
                      )}
                    </TouchableOpacity>
                  </View>
                )}
              </View>
              <Text className="text-3xl font-bold text-brand mt-2 mb-3">
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
              ) : null}
            </View>

            {/* Recent commissions */}
            <View className="flex-row items-center justify-between mb-3">
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                Recent commissions
              </Text>
              {(trades?.length ?? 0) > 0 ? (
                <Text className="text-xs text-muted dark:text-muted-dark">
                  {trades?.length ?? 0} trades
                </Text>
              ) : null}
            </View>

            {(trades?.length ?? 0) > 0 ? (
              <View className="mb-8">
                {trades?.map((t) => {
                  const stateColor = STATE_COLORS[t.displayState] ?? "#64748B"
                  return (
                    <TouchableOpacity
                      key={t.id}
                      onPress={() =>
                        router.push({ pathname: "/(app)/trade/[id]", params: { id: t.tradeId } })
                      }
                      className="bg-surface dark:bg-card-dark rounded-2xl p-4 mb-3 border border-border dark:border-border-dark"
                      activeOpacity={0.85}
                    >
                      <View className="flex-row items-center justify-between">
                        <View className="flex-row items-center gap-3 flex-1">
                          <View className="w-9 h-9 rounded-full bg-success/10 items-center justify-center">
                            <TrendingUp size={14} color="#10B981" />
                          </View>
                          <View className="flex-1">
                            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                              {t.cryptoAmount} {t.cryptocurrency}
                            </Text>
                            <View className="flex-row items-center gap-2 mt-0.5">
                              <View
                                className="px-1.5 py-0.5 rounded-full"
                                style={{ backgroundColor: stateColor + "22" }}
                              >
                                <Text
                                  className="text-[10px] font-semibold capitalize"
                                  style={{ color: stateColor }}
                                >
                                  {t.displayState}
                                </Text>
                              </View>
                              <Text className="text-[11px] text-muted dark:text-muted-dark">
                                {t.markupPercentage}% markup
                              </Text>
                            </View>
                          </View>
                        </View>
                        <View className="items-end">
                          <Text className="text-sm font-bold text-success">
                            +{t.resellerProfit}
                          </Text>
                          <Text className="text-[11px] text-muted dark:text-muted-dark">
                            {t.fiatCurrency}
                          </Text>
                        </View>
                        <ChevronRight size={14} color="#94A3B8" className="ml-2" />
                      </View>
                    </TouchableOpacity>
                  )
                })}
              </View>
            ) : (
              <View className="items-center py-10 mb-8">
                <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
                  <TrendingUp size={24} color="#00A3F6" />
                </View>
                <Text className="text-base font-semibold text-foreground dark:text-foreground-dark mb-1">
                  No commissions yet
                </Text>
                <Text className="text-sm text-muted dark:text-muted-dark text-center max-w-[260px]">
                  Find an offer in the marketplace and tap{" "}
                  <Text className="font-semibold text-brand">Resell</Text> to start earning.
                </Text>
                <TouchableOpacity
                  onPress={() => router.push("/(tabs)/marketplace" as never)}
                  className="mt-5 px-5 h-10 rounded-xl bg-brand flex-row items-center gap-1.5"
                  activeOpacity={0.85}
                >
                  <Store size={14} color="#FFFFFF" />
                  <Text className="text-sm font-semibold text-white">Browse marketplace</Text>
                </TouchableOpacity>
              </View>
            )}
          </View>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  )
}
