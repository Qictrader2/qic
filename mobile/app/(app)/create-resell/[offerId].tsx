import {
  View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query"
import { ChevronLeft, Repeat, TrendingUp, AlertTriangle, Sparkles } from "lucide-react-native"
import { resellerService, calculateResellerCommission } from "@/src/services/reseller.service"
import { marketService } from "@/src/services/market.service"
import Slider from "@react-native-community/slider"
import type { ApiError } from "@/src/lib/api/client"

const MIN_MARKUP = 0
const MAX_MARKUP_SELL = 100
const MAX_MARKUP_BUY = 50

export default function CreateResellScreen() {
  const { offerId } = useLocalSearchParams<{ offerId: string }>()
  const router = useRouter()
  const qc = useQueryClient()

  const { data: offer, isLoading: loadingOffer } = useQuery({
    queryKey: ["offer", offerId],
    queryFn: () => marketService.getOffer(offerId ?? ""),
    enabled: !!offerId,
  })

  const { data: stats } = useQuery({
    queryKey: ["reseller-stats"],
    queryFn: () => resellerService.getStats(),
  })

  const defaultMarkup = stats?.defaultMarkupPercentage ?? 2.5
  const [markup, setMarkup] = useState(defaultMarkup)
  const [serverError, setServerError] = useState<string | null>(null)

  const offerType = offer?.offerType ?? "sell"
  const baseRate = parseFloat(offer?.pricePerUnit ?? "0")
  const sampleAmount = 1 // preview on 1 unit

  const preview = baseRate > 0
    ? calculateResellerCommission(baseRate, markup, sampleAmount, offerType)
    : null

  const { mutateAsync: createResell, isPending } = useMutation({
    mutationFn: () => resellerService.createResell(offerId ?? "", markup),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["reseller-stats"] })
      qc.invalidateQueries({ queryKey: ["reseller-active"] })
      Alert.alert("Resell offer created!", "Your listing is now live on the marketplace.", [
        { text: "View dashboard", onPress: () => router.replace("/(app)/reseller-dashboard") },
        { text: "Done", onPress: () => router.back() },
      ])
    },
    onError: (err) => {
      const apiErr = err as unknown as ApiError
      if (apiErr.kind === "validation" && Object.keys(apiErr.fields).includes("markup")) {
        setServerError("Invalid markup percentage.")
      } else if (apiErr.kind === "server" && apiErr.message.includes("ALREADY_EXISTS")) {
        setServerError("You already have an active resell offer for this listing.")
      } else {
        setServerError("Failed to create resell offer. Please try again.")
      }
    },
  })

  if (loadingOffer || !offer) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center">
        <ActivityIndicator color="#00A3F6" />
      </SafeAreaView>
    )
  }

  const maxMarkup = offerType === "buy" ? MAX_MARKUP_BUY : MAX_MARKUP_SELL

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
            Create resell offer
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            Earn the spread above the base rate
          </Text>
        </View>
      </View>

      <ScrollView className="flex-1 px-5" keyboardShouldPersistTaps="handled" showsVerticalScrollIndicator={false}>
        <View className="rounded-2xl bg-brand/10 border border-brand/20 p-4 mb-5 flex-row items-center gap-3">
          <View className="w-10 h-10 rounded-full bg-brand/20 items-center justify-center">
            <Repeat size={18} color="#00A3F6" />
          </View>
          <View className="flex-1">
            <Text className="text-sm font-semibold text-brand">No capital required</Text>
            <Text className="text-xs text-brand/80 mt-0.5">
              The vendor's escrow funds the trade — you earn the difference.
            </Text>
          </View>
        </View>

        {serverError ? (
          <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3 flex-row items-center gap-2">
            <AlertTriangle size={14} color="#EF4444" />
            <Text className="text-sm text-error flex-1">{serverError}</Text>
          </View>
        ) : null}

        {/* Base offer summary */}
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-4">
          <View className="flex-row justify-between mb-2">
            <Text className="text-xs font-medium text-muted dark:text-muted-dark uppercase tracking-wider">
              Base listing
            </Text>
            <View
              className={`px-2 py-0.5 rounded-full ${
                offerType === "buy" ? "bg-success/10" : "bg-info/10"
              }`}
            >
              <Text
                className={`text-[10px] font-bold uppercase tracking-wider ${
                  offerType === "buy" ? "text-success" : "text-info"
                }`}
              >
                {offerType === "buy" ? "Buying" : "Selling"}
              </Text>
            </View>
          </View>
          <Text className="text-base font-bold text-foreground dark:text-foreground-dark mb-1">
            {offer.currency} · {offer.fiatCurrency}{" "}
            {parseFloat(offer.pricePerUnit).toLocaleString()}
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark">
            {offer.owner.username} · {offer.owner.completionRate}% completion
          </Text>
        </View>

        {/* Markup slider */}
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-4">
          <View className="flex-row justify-between items-baseline mb-2">
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
              Your markup
            </Text>
            <Text className="text-3xl font-bold text-brand">{markup.toFixed(1)}%</Text>
          </View>
          <Slider
            minimumValue={MIN_MARKUP}
            maximumValue={maxMarkup}
            step={0.1}
            value={markup}
            onValueChange={setMarkup}
            minimumTrackTintColor="#00A3F6"
            maximumTrackTintColor="#E2E8F0"
            thumbTintColor="#00A3F6"
            style={{ marginVertical: 4 }}
          />
          <View className="flex-row justify-between">
            <Text className="text-xs text-muted dark:text-muted-dark">0%</Text>
            <Text className="text-xs text-muted dark:text-muted-dark">{maxMarkup}%</Text>
          </View>
        </View>

        {/* Profit preview */}
        {preview && baseRate > 0 ? (
          <View className="rounded-2xl bg-success/10 border border-success/20 p-4 mb-6">
            <View className="flex-row items-center gap-2 mb-3">
              <TrendingUp size={14} color="#10B981" />
              <Text className="text-xs font-bold text-success uppercase tracking-wider">
                Commission preview · per 1 {offer.currency}
              </Text>
            </View>
            <View className="flex-row justify-between mb-1.5">
              <Text className="text-xs text-muted dark:text-muted-dark">Your resold rate</Text>
              <Text className="text-xs font-semibold text-foreground dark:text-foreground-dark">
                {offer.fiatCurrency}{" "}
                {preview.resoldRate.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </Text>
            </View>
            <View className="flex-row justify-between mb-1.5">
              <Text className="text-xs text-muted dark:text-muted-dark">Gross commission</Text>
              <Text className="text-xs font-semibold text-foreground dark:text-foreground-dark">
                {offer.fiatCurrency}{" "}
                {preview.grossCommission.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </Text>
            </View>
            <View className="h-px bg-success/20 my-2" />
            <View className="flex-row justify-between items-center">
              <Text className="text-xs font-medium text-success">Your net (after 25% fee)</Text>
              <Text className="text-base font-bold text-success">
                {offer.fiatCurrency}{" "}
                {preview.netCommission.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </Text>
            </View>
          </View>
        ) : null}

        <View className="h-24" />
      </ScrollView>

      <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
        <TouchableOpacity
          onPress={() => createResell()}
          disabled={isPending}
          className="rounded-xl bg-brand h-12 items-center justify-center flex-row gap-2"
          activeOpacity={0.85}
        >
          {isPending ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <>
              <Sparkles size={16} color="#FFFFFF" />
              <Text className="text-base font-semibold text-white">
                Publish at {markup.toFixed(1)}% markup
              </Text>
            </>
          )}
        </TouchableOpacity>
      </View>
    </SafeAreaView>
  )
}
