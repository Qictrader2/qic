import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  TextInput,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { useLocalSearchParams, useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { ShieldCheck, ThumbsUp, Clock } from "lucide-react-native"
import { useOffer, useInitiateTrade } from "@/src/hooks/api/use-market"
import { useAuthStore } from "@/src/store/auth-store"
import { ApiError } from "@/src/lib/api/client"

const schema = z.object({
  amount: z.string().refine((v) => parseFloat(v) > 0, "Enter a valid amount"),
})
type Form = z.infer<typeof schema>

export default function OfferDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const router = useRouter()
  const { isAuthenticated } = useAuthStore()
  const { data: offer, isLoading, error } = useOffer(id ?? "")
  const { mutateAsync: initiateTrade } = useInitiateTrade()
  const [serverError, setServerError] = useState<string | null>(null)

  const {
    control,
    handleSubmit,
    watch,
    formState: { errors, isSubmitting },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  const amount = watch("amount") ?? "0"
  const fiatTotal = offer
    ? (parseFloat(amount || "0") * parseFloat(offer.pricePerUnit)).toFixed(2)
    : "0.00"

  async function onSubmit(data: Form) {
    setServerError(null)
    if (!isAuthenticated) {
      router.push("/(auth)/login")
      return
    }
    try {
      const { tradeId } = await initiateTrade({ offerId: id ?? "", amount: data.amount })
      router.replace({ pathname: "/(app)/trade/[id]", params: { id: tradeId } })
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "validation") {
        setServerError(Object.values(apiErr.fields)[0] ?? "Validation error")
      } else if (apiErr.kind === "forbidden") {
        setServerError("You don't meet the requirements for this offer.")
      } else {
        setServerError("Failed to initiate trade. Please try again.")
      }
    }
  }

  if (isLoading) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center">
        <ActivityIndicator color="#00A3F6" />
      </SafeAreaView>
    )
  }

  if (error || !offer) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark px-4 justify-center">
        <View className="rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">Failed to load offer.</Text>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1">
        <ScrollView className="flex-1 px-4 py-4" keyboardShouldPersistTaps="handled">
          {/* Trader card — mirrors web offer-detail trader hero */}
          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-3">
            <View className="flex-row items-center gap-3">
              <View className="w-14 h-14 rounded-full bg-brand/20 items-center justify-center">
                <Text className="text-lg font-bold text-brand">
                  {offer.owner.username
                    .split(/[\s_]/)
                    .slice(0, 2)
                    .map((w) => w[0]?.toUpperCase() ?? "")
                    .join("")}
                </Text>
              </View>
              <View className="flex-1">
                <View className="flex-row items-center gap-1.5 flex-wrap">
                  <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
                    {offer.owner.username}
                  </Text>
                  {offer.owner.kycTier >= 2 ? (
                    <View className="bg-success-bg rounded-full px-1.5 py-0.5">
                      <Text className="text-[10px] font-semibold text-success">✓ Verified</Text>
                    </View>
                  ) : null}
                </View>
                <View className="flex-row items-center gap-2 mt-1">
                  <View className="flex-row items-center gap-1">
                    <ThumbsUp size={11} color="#64748B" />
                    <Text className="text-xs text-muted dark:text-muted-dark">
                      {Math.round(offer.owner.completionRate)}%
                    </Text>
                  </View>
                  <Text className="text-xs text-muted dark:text-muted-dark">·</Text>
                  <Text className="text-xs text-muted dark:text-muted-dark">
                    {offer.owner.tradeCount} trades
                  </Text>
                  <Text className="text-xs text-muted dark:text-muted-dark">·</Text>
                  <Text className="text-xs text-muted dark:text-muted-dark">
                    KYC L{offer.owner.kycTier}
                  </Text>
                </View>
              </View>
              <View
                className={`px-2.5 py-1 rounded-full ${
                  offer.offerType === "buy" ? "bg-success-bg" : "bg-info-bg"
                }`}
              >
                <Text
                  className={`text-[10px] font-bold tracking-wider ${
                    offer.offerType === "buy" ? "text-success" : "text-info"
                  }`}
                >
                  {offer.offerType === "buy" ? "BUYING" : "SELLING"}
                </Text>
              </View>
            </View>
          </View>

          {/* Price + limits */}
          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-3">
            <Text className="text-xs text-muted dark:text-muted-dark">Price per {offer.currency}</Text>
            <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mt-1">
              {offer.fiatCurrency} {parseFloat(offer.pricePerUnit).toLocaleString()}
            </Text>
            <View className="h-px bg-border dark:bg-border-dark my-3" />
            <View className="flex-row justify-between mb-1.5">
              <Text className="text-xs text-muted dark:text-muted-dark">Trade limits</Text>
              <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                {parseFloat(offer.minAmount)} – {parseFloat(offer.maxAmount)} {offer.currency}
              </Text>
            </View>
            <View className="flex-row justify-between mb-1.5">
              <Text className="text-xs text-muted dark:text-muted-dark">Payment methods</Text>
              <Text
                className="text-xs font-medium text-foreground dark:text-foreground-dark text-right flex-1 ml-4 capitalize"
                numberOfLines={2}
              >
                {offer.paymentMethods.map((m) => m.replace(/_/g, " ")).join(", ")}
              </Text>
            </View>
            <View className="flex-row justify-between items-center">
              <Text className="text-xs text-muted dark:text-muted-dark">Payment window</Text>
              <View className="flex-row items-center gap-1">
                <Clock size={11} color="#64748B" />
                <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                  {offer.paymentWindow} min
                </Text>
              </View>
            </View>
          </View>

          {/* Escrow protection */}
          <View className="rounded-2xl bg-success-bg p-3.5 mb-3 flex-row items-center gap-2.5">
            <ShieldCheck size={16} color="#10B981" />
            <Text className="text-xs text-success flex-1">
              Escrow protected — crypto is held by QicTrader until the trade is released.
            </Text>
          </View>

          {offer.terms ? (
            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-3">
              <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mb-2">
                Trader's terms
              </Text>
              <Text className="text-sm text-foreground dark:text-foreground-dark leading-5">
                {offer.terms}
              </Text>
            </View>
          ) : null}

          {/* Quote / trade form */}
          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-3">
            <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mb-3">
              {offer.offerType === "buy" ? "Sell amount" : "Buy amount"}
            </Text>

            {serverError ? (
              <View className="mb-3 rounded-lg bg-error-bg px-3 py-2">
                <Text className="text-xs text-error">{serverError}</Text>
              </View>
            ) : null}

            <Controller
              control={control}
              name="amount"
              render={({ field: { onChange, onBlur, value } }) => (
                <View className="relative">
                  <TextInput
                    className="h-14 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 pr-20 text-xl font-semibold text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value}
                    placeholder="0.00"
                    placeholderTextColor="#94A3B8"
                    keyboardType="decimal-pad"
                    editable={!isSubmitting}
                  />
                  <View className="absolute right-4 top-4">
                    <Text className="text-sm font-semibold text-muted dark:text-muted-dark">
                      {offer.currency}
                    </Text>
                  </View>
                </View>
              )}
            />
            {errors.amount ? (
              <Text className="mt-1 text-xs text-error">{errors.amount.message}</Text>
            ) : null}

            <View className="mt-3 flex-row items-center justify-between">
              <Text className="text-xs text-muted dark:text-muted-dark">You'll {offer.offerType === "buy" ? "receive" : "pay"}</Text>
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                {offer.fiatCurrency} {parseFloat(amount || "0") > 0 ? fiatTotal : "0.00"}
              </Text>
            </View>
            <Text className="text-[11px] text-muted dark:text-muted-dark mt-1 text-right">
              Min {parseFloat(offer.minAmount)} · Max {parseFloat(offer.maxAmount)} {offer.currency}
            </Text>
          </View>

          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="rounded-xl bg-brand py-4 items-center mb-3"
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">
                {!isAuthenticated
                  ? "Sign in to trade"
                  : offer.offerType === "buy"
                    ? "Start selling"
                    : "Start buying"}
              </Text>
            )}
          </TouchableOpacity>

          {isAuthenticated && (
            <TouchableOpacity
              onPress={() =>
                router.push({
                  pathname: "/(app)/create-resell/[offerId]",
                  params: { offerId: offer.id },
                })
              }
              className="rounded-xl border border-brand bg-brand-bg py-3.5 items-center mb-8"
              activeOpacity={0.85}
            >
              <Text className="text-sm font-semibold text-brand">Resell this offer</Text>
            </TouchableOpacity>
          )}
        </ScrollView>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
