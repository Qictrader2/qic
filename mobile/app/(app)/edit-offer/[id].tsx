import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState } from "react"
import { ChevronLeft, AlertTriangle } from "lucide-react-native"
import { useOffer } from "@/src/hooks/api/use-market"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { marketService } from "@/src/services/market.service"

const schema = z.object({
  pricePerUnit: z.string().refine((v) => parseFloat(v) > 0, "Enter a valid price"),
  minAmount: z.string().refine((v) => parseFloat(v) > 0, "Enter a min"),
  maxAmount: z.string().refine((v) => parseFloat(v) > 0, "Enter a max"),
  paymentWindow: z.string().refine((v) => parseInt(v) >= 15, "Minimum 15 min"),
  terms: z.string().optional(),
})
type Form = z.infer<typeof schema>

export default function EditOfferScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const router = useRouter()
  const qc = useQueryClient()
  const { data: offer, isLoading } = useOffer(id ?? "")
  const [serverError, setServerError] = useState<string | null>(null)

  const { mutateAsync: updateOffer } = useMutation({
    mutationFn: (data: Form) =>
      marketService.updateOffer(id ?? "", {
        pricePerUnit: data.pricePerUnit,
        minAmount: data.minAmount,
        maxAmount: data.maxAmount,
        paymentWindow: parseInt(data.paymentWindow),
        ...(data.terms !== undefined ? { terms: data.terms } : {}),
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["offer", id] })
      qc.invalidateQueries({ queryKey: ["my-offers"] })
    },
  })

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting, isDirty },
  } = useForm<Form>({
    resolver: zodResolver(schema),
    ...(offer
      ? {
          defaultValues: {
            pricePerUnit: offer.pricePerUnit,
            minAmount: offer.minAmount,
            maxAmount: offer.maxAmount,
            paymentWindow: String(offer.paymentWindow),
            terms: offer.terms ?? "",
          },
        }
      : {}),
  })

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await updateOffer(data)
      router.back()
    } catch {
      setServerError("Failed to update offer. Please try again.")
    }
  }

  if (isLoading) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center">
        <ActivityIndicator color="#00A3F6" />
      </SafeAreaView>
    )
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1" edges={["bottom"]}>
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
              Edit offer
            </Text>
            {offer ? (
              <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
                {offer.offerType === "buy" ? "Buying" : "Selling"} {offer.currency} ·{" "}
                {offer.fiatCurrency}
              </Text>
            ) : null}
          </View>
        </View>

        <ScrollView className="flex-1 px-5" keyboardShouldPersistTaps="handled" showsVerticalScrollIndicator={false}>
          {serverError ? (
            <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3 flex-row items-center gap-2">
              <AlertTriangle size={14} color="#EF4444" />
              <Text className="text-sm text-error flex-1">{serverError}</Text>
            </View>
          ) : null}

          {[
            { name: "pricePerUnit" as const, label: `Price per ${offer?.currency ?? "unit"}` },
            { name: "minAmount" as const, label: "Min trade size" },
            { name: "maxAmount" as const, label: "Max trade size" },
            { name: "paymentWindow" as const, label: "Payment window (minutes)" },
          ].map(({ name, label }) => (
            <View key={name} className="mb-4">
              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                {label}
              </Text>
              <Controller
                control={control}
                name={name}
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-base text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={String(value ?? "")}
                    keyboardType={name === "paymentWindow" ? "number-pad" : "decimal-pad"}
                  />
                )}
              />
              {errors[name]?.message ? (
                <Text className="mt-1 text-xs text-error">{String(errors[name]?.message)}</Text>
              ) : null}
            </View>
          ))}

          <View className="mb-6">
            <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
              Terms (optional)
            </Text>
            <Controller
              control={control}
              name="terms"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 py-3 text-sm text-foreground dark:text-foreground-dark h-28"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder="Describe your terms…"
                  placeholderTextColor="#94A3B8"
                  multiline
                  textAlignVertical="top"
                />
              )}
            />
          </View>

          <View className="h-24" />
        </ScrollView>

        <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting || !isDirty}
            className={`rounded-xl h-12 items-center justify-center ${
              !isDirty ? "bg-brand/40" : "bg-brand"
            }`}
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Save changes</Text>
            )}
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
