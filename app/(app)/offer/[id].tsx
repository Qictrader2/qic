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
import { useOffer, useInitiateTrade } from "@/src/hooks/api/use-market"
import { ApiError } from "@/src/lib/api/client"

const schema = z.object({
  amount: z.string().refine((v) => parseFloat(v) > 0, "Enter a valid amount"),
})
type Form = z.infer<typeof schema>

export default function OfferDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const router = useRouter()
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
          {/* Offer header */}
          <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
            <View className="flex-row items-center justify-between mb-3">
              <View
                className={`px-2.5 py-1 rounded-full ${
                  offer.offerType === "buy" ? "bg-success-bg" : "bg-error-bg"
                }`}
              >
                <Text
                  className={`text-xs font-semibold uppercase ${
                    offer.offerType === "buy" ? "text-success" : "text-error"
                  }`}
                >
                  {offer.offerType}
                </Text>
              </View>
              <Text className="text-lg font-bold text-foreground dark:text-foreground-dark">
                {offer.currency} / {offer.fiatCurrency}
              </Text>
            </View>

            <View className="flex-row justify-between mb-2">
              <Text className="text-sm text-muted dark:text-muted-dark">Price</Text>
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                {offer.fiatCurrency} {parseFloat(offer.pricePerUnit).toLocaleString()}
              </Text>
            </View>
            <View className="flex-row justify-between mb-2">
              <Text className="text-sm text-muted dark:text-muted-dark">Limits</Text>
              <Text className="text-sm text-foreground dark:text-foreground-dark">
                {offer.minAmount}–{offer.maxAmount} {offer.currency}
              </Text>
            </View>
            <View className="flex-row justify-between mb-2">
              <Text className="text-sm text-muted dark:text-muted-dark">Payment</Text>
              <Text className="text-sm text-foreground dark:text-foreground-dark">
                {offer.paymentMethods.map((m) => m.replace(/_/g, " ")).join(", ")}
              </Text>
            </View>
            <View className="flex-row justify-between">
              <Text className="text-sm text-muted dark:text-muted-dark">Payment window</Text>
              <Text className="text-sm text-foreground dark:text-foreground-dark">
                {offer.paymentWindow} min
              </Text>
            </View>
          </View>

          {/* Seller info */}
          <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mb-2">
              {offer.owner.username}
            </Text>
            <Text className="text-xs text-muted dark:text-muted-dark">
              {offer.owner.completionRate}% completion · {offer.owner.tradeCount} trades · KYC tier{" "}
              {offer.owner.kycTier}
            </Text>
          </View>

          {offer.terms ? (
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <Text className="text-xs font-medium text-muted dark:text-muted-dark mb-1">Terms</Text>
              <Text className="text-sm text-foreground dark:text-foreground-dark">{offer.terms}</Text>
            </View>
          ) : null}

          {/* Trade form */}
          {serverError ? (
            <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{serverError}</Text>
            </View>
          ) : null}

          <View className="mb-4">
            <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
              Amount ({offer.currency})
            </Text>
            <Controller
              control={control}
              name="amount"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  placeholder={`${offer.minAmount}–${offer.maxAmount}`}
                  placeholderTextColor="#94A3B8"
                  keyboardType="decimal-pad"
                  editable={!isSubmitting}
                />
              )}
            />
            {errors.amount ? (
              <Text className="mt-1 text-xs text-error">{errors.amount.message}</Text>
            ) : null}
            {parseFloat(amount) > 0 ? (
              <Text className="mt-1.5 text-xs text-muted dark:text-muted-dark">
                = {offer.fiatCurrency} {fiatTotal}
              </Text>
            ) : null}
          </View>

          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting}
            className="rounded-lg bg-brand py-4 items-center mb-8"
            activeOpacity={0.8}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">
                {offer.offerType === "buy" ? "Start buying" : "Start selling"}
              </Text>
            )}
          </TouchableOpacity>
        </ScrollView>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
