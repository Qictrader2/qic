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
import { useState } from "react"
import { useOffers } from "@/src/hooks/api/use-market"
import type { Offer, OfferType } from "@/src/services/market.service"

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
          <View
            className={`px-2 py-0.5 rounded-full ${isBuy ? "bg-success-bg" : "bg-error-bg"}`}
          >
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
        <View>
          <Text className="text-xs text-muted dark:text-muted-dark">
            Limits: {offer.minAmount}–{offer.maxAmount} {offer.currency}
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            {offer.paymentMethods.join(", ").replace(/_/g, " ")}
          </Text>
        </View>
        <View className="items-end">
          <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
            {offer.owner.username}
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark">
            {offer.owner.completionRate}% · {offer.owner.tradeCount} trades
          </Text>
        </View>
      </View>
    </TouchableOpacity>
  )
}

export default function MarketplaceScreen() {
  const router = useRouter()
  const [tab, setTab] = useState<OfferType>("buy")
  const { data: offers, isLoading, error, refetch, isRefetching } = useOffers({ offerType: tab })

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
          Marketplace
        </Text>
        <View className="flex-row gap-2">
          <TouchableOpacity
            onPress={() => router.push("/(app)/my-offers")}
            className="px-3 py-1.5 rounded-lg bg-surface dark:bg-surface-dark border border-border dark:border-border-dark"
          >
            <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">My offers</Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => router.push("/(app)/create-offer")}
            className="px-3 py-1.5 rounded-lg bg-brand"
          >
            <Text className="text-xs font-medium text-white">+ Offer</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Buy / Sell tabs */}
      <View className="flex-row mx-4 mb-3 rounded-lg bg-surface dark:bg-surface-dark p-1 border border-border dark:border-border-dark">
        {(["buy", "sell"] as OfferType[]).map((t) => (
          <TouchableOpacity
            key={t}
            onPress={() => setTab(t)}
            className={`flex-1 py-2 rounded-md items-center ${tab === t ? "bg-brand" : ""}`}
            activeOpacity={0.8}
          >
            <Text
              className={`text-sm font-medium capitalize ${
                tab === t ? "text-white" : "text-muted dark:text-muted-dark"
              }`}
            >
              {t}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : error ? (
        <View className="mx-4 rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">Failed to load offers. Pull to retry.</Text>
        </View>
      ) : (
        <FlatList
          data={offers ?? []}
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
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">🏪</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark">
                No offers right now
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
