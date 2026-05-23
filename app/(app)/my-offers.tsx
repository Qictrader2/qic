import {
  View, Text, FlatList, TouchableOpacity, ActivityIndicator, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useMyOffers, usePauseOffer, useDeleteOffer } from "@/src/hooks/api/use-market"
import { useRouter } from "expo-router"
import type { Offer } from "@/src/services/market.service"

function MyOfferRow({
  offer,
  onPause,
  onResume,
  onDelete,
  onEdit,
}: {
  offer: Offer
  onPause: () => void
  onResume: () => void
  onDelete: () => void
  onEdit: () => void
}) {
  const isBuy = offer.offerType === "buy"
  const isActive = offer.status === "active"

  return (
    <View className="bg-surface dark:bg-surface-dark rounded-xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between mb-3">
        <View className="flex-row items-center gap-2">
          <View className={`px-2 py-0.5 rounded-full ${isBuy ? "bg-success-bg" : "bg-error-bg"}`}>
            <Text className={`text-xs font-semibold ${isBuy ? "text-success" : "text-error"}`}>
              {isBuy ? "BUY" : "SELL"}
            </Text>
          </View>
          <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
            {offer.currency}
          </Text>
        </View>
        <View className={`px-2 py-0.5 rounded-full ${isActive ? "bg-success-bg" : "bg-warning-bg"}`}>
          <Text className={`text-xs font-medium capitalize ${isActive ? "text-success" : "text-warning"}`}>
            {offer.status}
          </Text>
        </View>
      </View>

      <View className="flex-row justify-between mb-3">
        <Text className="text-xs text-muted dark:text-muted-dark">
          {offer.fiatCurrency} {parseFloat(offer.pricePerUnit).toLocaleString()}
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark">
          {offer.minAmount}–{offer.maxAmount} {offer.currency}
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark">
          {offer.tradeCount} trades
        </Text>
      </View>

      <View className="flex-row gap-2">
        {isActive ? (
          <TouchableOpacity
            onPress={onPause}
            className="flex-1 rounded-lg border border-warning bg-warning-bg py-2 items-center"
            activeOpacity={0.7}
          >
            <Text className="text-xs font-medium text-warning">Pause</Text>
          </TouchableOpacity>
        ) : (
          <TouchableOpacity
            onPress={onResume}
            className="flex-1 rounded-lg border border-success bg-success-bg py-2 items-center"
            activeOpacity={0.7}
          >
            <Text className="text-xs font-medium text-success">Resume</Text>
          </TouchableOpacity>
        )}
        <TouchableOpacity
          onPress={onEdit}
          className="flex-1 rounded-lg border border-border dark:border-border-dark py-2 items-center"
          activeOpacity={0.7}
        >
          <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">Edit</Text>
        </TouchableOpacity>
        <TouchableOpacity
          onPress={onDelete}
          className="flex-1 rounded-lg border border-error bg-error-bg py-2 items-center"
          activeOpacity={0.7}
        >
          <Text className="text-xs font-medium text-error">Delete</Text>
        </TouchableOpacity>
      </View>
    </View>
  )
}

export default function MyOffersScreen() {
  const router = useRouter()
  const { data: offers, isLoading } = useMyOffers()
  const { mutateAsync: pauseOffer } = usePauseOffer()
  const { mutateAsync: deleteOffer } = useDeleteOffer()
  const { mutateAsync: resumeOffer } = usePauseOffer() // reuse — backend handles via resume

  function handleDelete(offer: Offer) {
    Alert.alert("Delete offer", `Delete this ${offer.offerType} offer for ${offer.currency}?`, [
      { text: "Cancel", style: "cancel" },
      { text: "Delete", style: "destructive", onPress: () => deleteOffer(offer.id) },
    ])
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">My Offers</Text>
        <TouchableOpacity
          onPress={() => router.push("/(app)/create-offer")}
          className="px-3 py-1.5 rounded-lg bg-brand"
        >
          <Text className="text-xs font-medium text-white">+ New</Text>
        </TouchableOpacity>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <FlatList
          data={offers ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <MyOfferRow
              offer={item}
              onPause={() => pauseOffer(item.id)}
              onResume={() => resumeOffer(item.id)}
              onDelete={() => handleDelete(item)}
              onEdit={() =>
                router.push({ pathname: "/(app)/edit-offer/[id]", params: { id: item.id } })
              }
            />
          )}
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 24 }}
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">🏪</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark mb-2">
                No offers yet
              </Text>
              <TouchableOpacity
                onPress={() => router.push("/(app)/create-offer")}
                className="rounded-lg bg-brand px-6 py-3"
              >
                <Text className="text-sm font-semibold text-white">Create your first offer</Text>
              </TouchableOpacity>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
