import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
  Image,
  ScrollView,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useEffect, useRef, useMemo } from "react"
import { useQueryClient } from "@tanstack/react-query"
import {
  ChevronRight,
  ShieldCheck,
  ThumbsUp,
  Search,
  SlidersHorizontal,
  Plus,
  Bitcoin,
} from "lucide-react-native"
import { useOffers } from "@/src/hooks/api/use-market"
import type { Currency, Offer, OfferType, PaymentMethodType } from "@/src/services/market.service"
import { useAuthStore } from "@/src/store/auth-store"
import { getSocket } from "@/src/lib/socket"

const CURRENCY_SHORT: Record<string, string> = {
  BTC: "Bitcoin",
  ETH: "Ethereum",
  USDT: "Tether",
  USDC: "USD Coin",
  SOL: "Solana",
}

const CURRENCY_COLORS: Record<string, string> = {
  BTC: "#F7931A",
  ETH: "#627EEA",
  USDT: "#26A17B",
  USDC: "#2775CA",
  SOL: "#9945FF",
}

const PAYMENT_METHOD_LABELS: Record<PaymentMethodType, string> = {
  bank_transfer: "Bank transfer",
  cash: "Cash",
  mobile_money: "Mobile money",
  crypto: "Crypto",
  other: "Other",
}

function initials(name: string): string {
  return name
    .split(/[\s_]/)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("")
}

function fmtAmount(value: string | number, max = 2): string {
  const n = typeof value === "string" ? parseFloat(value) : value
  if (Number.isNaN(n)) return "—"
  return n.toLocaleString(undefined, { maximumFractionDigits: max })
}

/**
 * MobileOfferCard — mirrors web's MobileOfferCard.tsx layout:
 *   1. Top: gradient accent + creator section (avatar, username, online dot, completion rate)
 *   2. Price block
 *   3. Payment methods row
 *   4. Action button row
 */
function OfferCard({ offer, onPress }: { offer: Offer; onPress: () => void }) {
  const isBuy = offer.offerType === "buy"
  const accent = CURRENCY_COLORS[offer.currency] ?? "#00A3F6"
  const completion = offer.owner?.completionRate ?? offer.completionRate ?? 0
  const traderName = offer.owner?.username ?? "trader"
  const traderInitials = initials(traderName)
  const isVerified = (offer.owner?.kycTier ?? 0) >= 2

  return (
    <TouchableOpacity
      onPress={onPress}
      activeOpacity={0.85}
      className="bg-surface dark:bg-card-dark rounded-2xl border border-border dark:border-border-dark overflow-hidden mb-3"
    >
      {/* Gradient accent strip */}
      <View style={{ height: 3, backgroundColor: accent + "33" }} />

      <View className="p-4">
        {/* Creator row */}
        <View className="flex-row items-center gap-3">
          <View
            className="w-12 h-12 rounded-full items-center justify-center"
            style={{ backgroundColor: accent + "22" }}
          >
            <Text className="text-base font-bold" style={{ color: accent }}>
              {traderInitials}
            </Text>
          </View>
          <View className="flex-1">
            <View className="flex-row items-center gap-1.5 flex-wrap">
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                {traderName}
              </Text>
              {isVerified && (
                <View className="bg-success-bg rounded-full px-1.5 py-0.5">
                  <Text className="text-[10px] font-semibold text-success">✓ Verified</Text>
                </View>
              )}
            </View>
            <View className="flex-row items-center gap-2 mt-1">
              <View className="flex-row items-center gap-1">
                <ThumbsUp size={11} color="#64748B" />
                <Text className="text-[11px] text-muted dark:text-muted-dark">
                  {Math.round(completion)}% · {offer.owner?.tradeCount ?? offer.tradeCount} trades
                </Text>
              </View>
            </View>
          </View>
          <View
            className={`px-2.5 py-1 rounded-full ${
              isBuy ? "bg-success-bg" : "bg-info-bg"
            }`}
          >
            <Text
              className={`text-[10px] font-bold tracking-wider ${
                isBuy ? "text-success" : "text-info"
              }`}
            >
              {isBuy ? "BUYING" : "SELLING"}
            </Text>
          </View>
        </View>

        {/* Price section */}
        <View className="mt-4 flex-row items-baseline justify-between">
          <View>
            <Text className="text-[11px] text-muted dark:text-muted-dark">
              {isBuy ? "Buying" : "Selling"} {offer.currency}
            </Text>
            <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">
              {offer.fiatCurrency} {fmtAmount(offer.pricePerUnit)}
            </Text>
          </View>
          <View className="items-end">
            <Text className="text-[11px] text-muted dark:text-muted-dark">Limits</Text>
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
              {fmtAmount(offer.minAmount, 4)} – {fmtAmount(offer.maxAmount, 4)}
            </Text>
          </View>
        </View>

        {/* Payment methods */}
        <View className="flex-row flex-wrap gap-1.5 mt-3">
          {offer.paymentMethods.slice(0, 3).map((m) => (
            <View
              key={m}
              className="bg-background-gray dark:bg-background-secondary-dark rounded-md px-2 py-1"
            >
              <Text className="text-[11px] text-muted-foreground dark:text-muted-dark">
                {PAYMENT_METHOD_LABELS[m] ?? m}
              </Text>
            </View>
          ))}
          {offer.paymentMethods.length > 3 && (
            <View className="bg-background-gray dark:bg-background-secondary-dark rounded-md px-2 py-1">
              <Text className="text-[11px] text-muted-foreground dark:text-muted-dark">
                +{offer.paymentMethods.length - 3}
              </Text>
            </View>
          )}
        </View>

        {/* Action row */}
        <View className="mt-4 flex-row items-center justify-between">
          <View className="flex-row items-center gap-1.5">
            <ShieldCheck size={12} color="#10B981" />
            <Text className="text-[11px] text-muted dark:text-muted-dark">Escrow protected</Text>
          </View>
          <View className="flex-row items-center gap-1">
            <Text className="text-xs font-semibold text-brand">
              {isBuy ? "Sell to trader" : "Buy now"}
            </Text>
            <ChevronRight size={14} color="#00A3F6" />
          </View>
        </View>
      </View>
    </TouchableOpacity>
  )
}

function ChipFilter<T extends string>({
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
      activeOpacity={0.8}
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

const CURRENCY_FILTERS = ["all", "USDT", "BTC", "ETH", "SOL"] as const
const POLL_INTERVAL_MS = 30_000

export default function MarketplaceScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const { isAuthenticated } = useAuthStore()

  const [tab, setTab] = useState<OfferType>("buy")
  const [currencyFilter, setCurrencyFilter] = useState<(typeof CURRENCY_FILTERS)[number]>("all")

  const queryArgs = useMemo(() => {
    const args: { offerType: OfferType; currency?: Currency } = { offerType: tab }
    if (currencyFilter !== "all") args.currency = currencyFilter as Currency
    return args
  }, [tab, currencyFilter])

  const { data: offers, isLoading, error, refetch, isRefetching } = useOffers(queryArgs)
  const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    if (!isAuthenticated) {
      pollTimerRef.current = setInterval(
        () => qc.invalidateQueries({ queryKey: ["offers"] }),
        POLL_INTERVAL_MS,
      )
      return () => {
        if (pollTimerRef.current) clearInterval(pollTimerRef.current)
      }
    }

    const socket = getSocket()
    const invalidate = () => qc.invalidateQueries({ queryKey: ["offers"] })

    if (socket) {
      socket.emit("join_marketplace")
      socket.on("offer_created", invalidate)
      socket.on("offer_updated", invalidate)
      socket.on("offer_deactivated", invalidate)
    }
    pollTimerRef.current = setInterval(invalidate, POLL_INTERVAL_MS)

    return () => {
      if (socket) {
        socket.off("offer_created", invalidate)
        socket.off("offer_updated", invalidate)
        socket.off("offer_deactivated", invalidate)
        socket.emit("leave_marketplace")
      }
      if (pollTimerRef.current) clearInterval(pollTimerRef.current)
    }
  }, [isAuthenticated])

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["top"]}>
      {/* Header */}
      <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
        <View>
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
            Marketplace
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            P2P crypto trading with escrow
          </Text>
        </View>
        <View className="flex-row gap-2">
          <TouchableOpacity
            onPress={() => router.push("/(app)/marketplace-filters")}
            className="w-10 h-10 rounded-full bg-surface dark:bg-card-dark border border-border dark:border-border-dark items-center justify-center"
            activeOpacity={0.7}
          >
            <SlidersHorizontal size={16} color="#64748B" />
          </TouchableOpacity>
          {isAuthenticated && (
            <TouchableOpacity
              onPress={() => router.push("/(app)/create-offer")}
              className="px-3 h-10 rounded-full bg-brand items-center justify-center flex-row gap-1"
              activeOpacity={0.85}
            >
              <Plus size={14} color="#FFFFFF" />
              <Text className="text-xs font-semibold text-white">Offer</Text>
            </TouchableOpacity>
          )}
        </View>
      </View>

      {/* Buy / Sell tabs */}
      <View className="mx-5 flex-row rounded-xl bg-surface dark:bg-card-dark p-1 border border-border dark:border-border-dark">
        {(["buy", "sell"] as OfferType[]).map((t) => (
          <TouchableOpacity
            key={t}
            onPress={() => setTab(t)}
            activeOpacity={0.85}
            className={`flex-1 py-2.5 rounded-lg items-center ${
              tab === t ? "bg-brand" : ""
            }`}
          >
            <Text
              className={`text-sm font-semibold capitalize ${
                tab === t ? "text-white" : "text-muted dark:text-muted-dark"
              }`}
            >
              {t === "buy" ? "Buy crypto" : "Sell crypto"}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* Currency chips */}
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={{ paddingHorizontal: 20, paddingTop: 12, paddingBottom: 8, gap: 8 }}
      >
        {CURRENCY_FILTERS.map((c) => (
          <ChipFilter
            key={c}
            label={c === "all" ? "All assets" : (CURRENCY_SHORT[c] ?? c)}
            active={currencyFilter === c}
            onPress={() => setCurrencyFilter(c)}
          />
        ))}
      </ScrollView>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
          <Text className="text-xs text-muted dark:text-muted-dark mt-2">Loading offers…</Text>
        </View>
      ) : error ? (
        <View className="mx-5 rounded-xl bg-error-bg p-4 mt-4">
          <Text className="text-sm text-error text-center">
            Couldn't load offers. Pull down to retry.
          </Text>
        </View>
      ) : (
        <FlatList
          data={offers ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <OfferCard
              offer={item}
              onPress={() =>
                router.push({ pathname: "/(app)/offer/[id]", params: { id: item.id } })
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
                <Search size={24} color="#00A3F6" />
              </View>
              <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
                No matching offers
              </Text>
              <Text className="text-xs text-muted dark:text-muted-dark mt-1">
                Try a different asset or check back soon
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
