import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
  RefreshControl,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useMyOffers, usePauseOffer, useDeleteOffer } from "@/src/hooks/api/use-market"
import { useRouter } from "expo-router"
import {
  Plus,
  Pause,
  Play,
  Pencil,
  Trash2,
  Store,
  ChevronRight,
  ThumbsUp,
} from "lucide-react-native"
import type { Offer } from "@/src/services/market.service"

const CURRENCY_COLORS: Record<string, string> = {
  BTC: "#F7931A",
  ETH: "#627EEA",
  USDT: "#26A17B",
  USDC: "#2775CA",
  SOL: "#9945FF",
}

function StatusBadge({ status }: { status: Offer["status"] }) {
  const color =
    status === "active"
      ? "#10B981"
      : status === "paused"
        ? "#F59E0B"
        : "#6B7280"
  return (
    <View
      className="px-2 py-0.5 rounded-full"
      style={{ backgroundColor: color + "22" }}
    >
      <Text className="text-[10px] font-semibold capitalize" style={{ color }}>
        {status}
      </Text>
    </View>
  )
}

function MyOfferRow({
  offer,
  onPause,
  onResume,
  onDelete,
  onEdit,
  onView,
}: {
  offer: Offer
  onPause: () => void
  onResume: () => void
  onDelete: () => void
  onEdit: () => void
  onView: () => void
}) {
  const isBuy = offer.offerType === "buy"
  const isActive = offer.status === "active"
  const accent = CURRENCY_COLORS[offer.currency] ?? "#00A3F6"

  return (
    <View className="bg-surface dark:bg-card-dark rounded-2xl border border-border dark:border-border-dark overflow-hidden mb-3">
      <View style={{ height: 3, backgroundColor: accent + "33" }} />

      <TouchableOpacity onPress={onView} activeOpacity={0.85} className="p-4">
        <View className="flex-row items-center justify-between">
          <View className="flex-row items-center gap-2">
            <View
              className="w-10 h-10 rounded-full items-center justify-center"
              style={{ backgroundColor: accent + "22" }}
            >
              <Text className="text-sm font-bold" style={{ color: accent }}>
                {offer.currency}
              </Text>
            </View>
            <View>
              <View className="flex-row items-center gap-1.5">
                <View
                  className={`px-2 py-0.5 rounded-full ${
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
                <StatusBadge status={offer.status} />
              </View>
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mt-1">
                {offer.fiatCurrency} {parseFloat(offer.pricePerUnit).toLocaleString()}
              </Text>
            </View>
          </View>
          <ChevronRight size={16} color="#94A3B8" />
        </View>

        <View className="flex-row items-center gap-4 mt-3">
          <View className="flex-row items-center gap-1">
            <ThumbsUp size={11} color="#64748B" />
            <Text className="text-xs text-muted dark:text-muted-dark">
              {offer.tradeCount} trades
            </Text>
          </View>
          <Text className="text-xs text-muted dark:text-muted-dark">
            {parseFloat(offer.minAmount)} – {parseFloat(offer.maxAmount)} {offer.currency}
          </Text>
        </View>
      </TouchableOpacity>

      <View className="border-t border-border/40 dark:border-border-dark/40 flex-row">
        {isActive ? (
          <ActionButton icon={Pause} label="Pause" onPress={onPause} color="#F59E0B" />
        ) : (
          <ActionButton icon={Play} label="Resume" onPress={onResume} color="#10B981" />
        )}
        <View className="w-px bg-border/40 dark:bg-border-dark/40" />
        <ActionButton icon={Pencil} label="Edit" onPress={onEdit} color="#64748B" />
        <View className="w-px bg-border/40 dark:bg-border-dark/40" />
        <ActionButton icon={Trash2} label="Delete" onPress={onDelete} color="#EF4444" />
      </View>
    </View>
  )
}

function ActionButton({
  icon: Icon,
  label,
  onPress,
  color,
}: {
  icon: typeof Pause
  label: string
  onPress: () => void
  color: string
}) {
  return (
    <TouchableOpacity
      onPress={onPress}
      className="flex-1 py-3 items-center justify-center flex-row gap-1.5"
      activeOpacity={0.7}
    >
      <Icon size={13} color={color} />
      <Text className="text-xs font-semibold" style={{ color }}>
        {label}
      </Text>
    </TouchableOpacity>
  )
}

export default function MyOffersScreen() {
  const router = useRouter()
  const { data: offers, isLoading, refetch, isRefetching } = useMyOffers()
  const { mutateAsync: pauseOffer } = usePauseOffer()
  const { mutateAsync: deleteOffer } = useDeleteOffer()
  const { mutateAsync: resumeOffer } = usePauseOffer()

  function handleDelete(offer: Offer) {
    Alert.alert("Delete offer", `Delete this ${offer.offerType} offer for ${offer.currency}?`, [
      { text: "Cancel", style: "cancel" },
      { text: "Delete", style: "destructive", onPress: () => deleteOffer(offer.id) },
    ])
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
        <View>
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
            My offers
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            {offers?.length ?? 0} listings
          </Text>
        </View>
        <TouchableOpacity
          onPress={() => router.push("/(app)/create-offer")}
          className="px-3 h-10 rounded-full bg-brand items-center justify-center flex-row gap-1"
          activeOpacity={0.85}
        >
          <Plus size={14} color="#FFFFFF" />
          <Text className="text-xs font-semibold text-white">New</Text>
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
              onEdit={() => router.push({ pathname: "/(app)/edit-offer/[id]", params: { id: item.id } })}
              onView={() => router.push({ pathname: "/(app)/offer/[id]", params: { id: item.id } })}
            />
          )}
          refreshControl={
            <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
          }
          contentContainerStyle={{ paddingHorizontal: 20, paddingTop: 4, paddingBottom: 32 }}
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
                <Store size={24} color="#00A3F6" />
              </View>
              <Text className="text-base font-semibold text-foreground dark:text-foreground-dark mb-1">
                No offers yet
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark text-center max-w-[240px]">
                Post an offer to start trading with users from across the platform.
              </Text>
              <TouchableOpacity
                onPress={() => router.push("/(app)/create-offer")}
                className="mt-5 px-5 py-2.5 rounded-xl bg-brand flex-row items-center gap-1.5"
                activeOpacity={0.85}
              >
                <Plus size={14} color="#FFFFFF" />
                <Text className="text-sm font-semibold text-white">Create offer</Text>
              </TouchableOpacity>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
