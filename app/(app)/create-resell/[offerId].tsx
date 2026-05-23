import {
  View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query"
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
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4" keyboardShouldPersistTaps="handled">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-1">
          Create Resell Offer
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-6">
          Set your markup above the base rate. You earn the spread when a buyer takes your listing.
        </Text>

        {serverError ? (
          <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
            <Text className="text-sm text-error">{serverError}</Text>
          </View>
        ) : null}

        {/* Base offer summary */}
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
          <View className="flex-row justify-between mb-1">
            <Text className="text-xs text-muted dark:text-muted-dark">Listing</Text>
            <View className={`px-2 py-0.5 rounded-full ${offerType === "buy" ? "bg-success-bg" : "bg-error-bg"}`}>
              <Text className={`text-xs font-semibold uppercase ${offerType === "buy" ? "text-success" : "text-error"}`}>
                {offerType}
              </Text>
            </View>
          </View>
          <Text className="text-sm font-medium text-foreground dark:text-foreground-dark mb-1">
            {offer.currency} · {offer.fiatCurrency}
          </Text>
          <Text className="text-base font-bold text-foreground dark:text-foreground-dark">
            Base rate: {offer.fiatCurrency} {parseFloat(offer.pricePerUnit).toLocaleString()}
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-1">
            {offer.owner.username} · {offer.owner.completionRate}% completion
          </Text>
        </View>

        {/* Markup slider */}
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
          <View className="flex-row justify-between items-baseline mb-1">
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">Your markup</Text>
            <Text className="text-2xl font-bold text-brand">{markup.toFixed(1)}%</Text>
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
          <View className="rounded-xl bg-success-bg border border-success/30 p-4 mb-6">
            <Text className="text-xs font-semibold text-success mb-2 uppercase tracking-wide">
              Commission preview (per 1 {offer.currency})
            </Text>
            <View className="flex-row justify-between mb-1">
              <Text className="text-xs text-muted dark:text-muted-dark">Your resold rate</Text>
              <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                {offer.fiatCurrency} {preview.resoldRate.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </Text>
            </View>
            <View className="flex-row justify-between mb-1">
              <Text className="text-xs text-muted dark:text-muted-dark">Gross commission</Text>
              <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                {offer.fiatCurrency} {preview.grossCommission.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </Text>
            </View>
            <View className="flex-row justify-between">
              <Text className="text-xs text-muted dark:text-muted-dark">Your net (after 25% fee)</Text>
              <Text className="text-xs font-bold text-success">
                {offer.fiatCurrency} {preview.netCommission.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </Text>
            </View>
          </View>
        ) : null}

        <TouchableOpacity
          onPress={() => createResell()}
          disabled={isPending}
          className="rounded-lg bg-brand py-4 items-center mb-8"
          activeOpacity={0.8}
        >
          {isPending ? <ActivityIndicator color="#fff" /> : (
            <Text className="text-base font-semibold text-white">
              Create resell offer at {markup.toFixed(1)}% markup
            </Text>
          )}
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  )
}
