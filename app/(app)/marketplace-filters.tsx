import {
  View, Text, FlatList, TouchableOpacity, ActivityIndicator, TextInput,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useDeferredValue } from "react"
import { useOffers } from "@/src/hooks/api/use-market"
import { useRouter } from "expo-router"
import type { Offer, OfferType, PaymentMethodType, Currency } from "@/src/services/market.service"

const CURRENCIES: (Currency | "ALL")[] = ["ALL", "BTC", "ETH", "SOL", "USDT", "USDC"]
const PAYMENT_METHODS: (PaymentMethodType | "ALL")[] = [
  "ALL", "bank_transfer", "cash", "mobile_money", "crypto", "other",
]

function FilterChip({
  label, selected, onPress,
}: { label: string; selected: boolean; onPress: () => void }) {
  return (
    <TouchableOpacity
      onPress={onPress}
      className={`px-3 py-1.5 rounded-full border mr-2 ${
        selected
          ? "bg-brand border-brand"
          : "border-border dark:border-border-dark bg-surface dark:bg-surface-dark"
      }`}
      activeOpacity={0.7}
    >
      <Text className={`text-xs font-medium ${selected ? "text-white" : "text-muted dark:text-muted-dark"}`}>
        {label}
      </Text>
    </TouchableOpacity>
  )
}

function OfferRow({ offer, onPress }: { offer: Offer; onPress: () => void }) {
  const isBuy = offer.offerType === "buy"
  return (
    <TouchableOpacity
      onPress={onPress}
      className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark"
      activeOpacity={0.8}
    >
      <View className="flex-row items-center justify-between mb-2">
        <View className="flex-row items-center gap-2">
          <View className={`px-2 py-0.5 rounded-full ${isBuy ? "bg-success-bg" : "bg-error-bg"}`}>
            <Text className={`text-xs font-semibold ${isBuy ? "text-success" : "text-error"}`}>
              {isBuy ? "BUY" : "SELL"}
            </Text>
          </View>
          <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
            {offer.currency}
          </Text>
        </View>
        <Text className="text-base font-bold text-foreground dark:text-foreground-dark">
          {offer.fiatCurrency} {parseFloat(offer.pricePerUnit).toLocaleString()}
        </Text>
      </View>
      <View className="flex-row items-center justify-between">
        <Text className="text-xs text-muted dark:text-muted-dark">
          {offer.minAmount}–{offer.maxAmount} {offer.currency}
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark">
          {offer.owner.username} · {offer.owner.completionRate}%
        </Text>
      </View>
    </TouchableOpacity>
  )
}

const SORT_OPTIONS = [
  { label: "Best price", value: "price" },
  { label: "Most trades", value: "trades" },
  { label: "Completion", value: "completion" },
] as const

type SortBy = typeof SORT_OPTIONS[number]["value"]

export default function MarketplaceFiltersScreen() {
  const router = useRouter()
  const [tab, setTab] = useState<OfferType>("buy")
  const [currency, setCurrency] = useState<Currency | "ALL">("ALL")
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethodType | "ALL">("ALL")
  const [sortBy, setSortBy] = useState<SortBy>("price")
  const [search, setSearch] = useState("")
  const deferredSearch = useDeferredValue(search)

  const { data: offers, isLoading } = useOffers({
    offerType: tab,
    ...(currency !== "ALL" ? { currency } : {}),
    ...(paymentMethod !== "ALL" ? { paymentMethod } : {}),
  })

  const filtered = (offers ?? []).filter((o) =>
    !deferredSearch ||
    o.owner.username.toLowerCase().includes(deferredSearch.toLowerCase()) ||
    o.currency.toLowerCase().includes(deferredSearch.toLowerCase())
  )

  const sorted = [...filtered].sort((a, b) => {
    if (sortBy === "price") {
      return tab === "buy"
        ? parseFloat(a.pricePerUnit) - parseFloat(b.pricePerUnit)
        : parseFloat(b.pricePerUnit) - parseFloat(a.pricePerUnit)
    }
    if (sortBy === "trades") return b.owner.tradeCount - a.owner.tradeCount
    return b.owner.completionRate - a.owner.completionRate
  })

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      {/* Search */}
      <View className="px-4 py-2">
        <TextInput
          className="rounded-xl border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-4 py-3 text-sm text-foreground dark:text-foreground-dark"
          value={search}
          onChangeText={setSearch}
          placeholder="Search by user or currency…"
          placeholderTextColor="#94A3B8"
          autoCapitalize="none"
          clearButtonMode="while-editing"
        />
      </View>

      {/* Buy/Sell tabs */}
      <View className="flex-row mx-4 mb-2 rounded-lg bg-surface dark:bg-surface-dark p-1 border border-border dark:border-border-dark">
        {(["buy", "sell"] as OfferType[]).map((t) => (
          <TouchableOpacity
            key={t}
            onPress={() => setTab(t)}
            className={`flex-1 py-2 rounded-md items-center ${tab === t ? "bg-brand" : ""}`}
            activeOpacity={0.8}
          >
            <Text className={`text-sm font-medium capitalize ${tab === t ? "text-white" : "text-muted dark:text-muted-dark"}`}>
              {t}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* Currency filter */}
      <FlatList
        horizontal
        data={CURRENCIES}
        keyExtractor={(c) => c}
        renderItem={({ item }) => (
          <FilterChip
            label={item}
            selected={currency === item}
            onPress={() => setCurrency(item)}
          />
        )}
        contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 8 }}
        showsHorizontalScrollIndicator={false}
      />

      {/* Payment method filter */}
      <FlatList
        horizontal
        data={PAYMENT_METHODS}
        keyExtractor={(m) => m}
        renderItem={({ item }) => (
          <FilterChip
            label={item === "ALL" ? "All payments" : item.replace(/_/g, " ")}
            selected={paymentMethod === item}
            onPress={() => setPaymentMethod(item)}
          />
        )}
        contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 8 }}
        showsHorizontalScrollIndicator={false}
      />

      {/* Sort */}
      <View className="flex-row px-4 pb-2 gap-2">
        {SORT_OPTIONS.map(({ label, value }) => (
          <TouchableOpacity
            key={value}
            onPress={() => setSortBy(value)}
            className={`px-3 py-1.5 rounded-full border ${
              sortBy === value
                ? "bg-brand/10 border-brand"
                : "border-border dark:border-border-dark"
            }`}
            activeOpacity={0.7}
          >
            <Text className={`text-xs font-medium ${sortBy === value ? "text-brand" : "text-muted dark:text-muted-dark"}`}>
              {label}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* Results */}
      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <FlatList
          data={sorted}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <OfferRow
              offer={item}
              onPress={() =>
                router.push({ pathname: "/(app)/offer/[id]", params: { id: item.id } })
              }
            />
          )}
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 24 }}
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <Text className="text-base text-muted dark:text-muted-dark">No offers found</Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
