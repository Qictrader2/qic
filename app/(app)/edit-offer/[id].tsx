import { View, Text, TextInput, TouchableOpacity, ActivityIndicator, ScrollView } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useOffer } from "@/src/hooks/api/use-market"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { marketService } from "@/src/services/market.service"
import { useState } from "react"
import { ApiError } from "@/src/lib/api/client"

const schema = z.object({
  pricePerUnit: z.string().refine((v) => parseFloat(v) > 0, "Enter a valid price"),
  minAmount: z.string().refine((v) => parseFloat(v) > 0),
  maxAmount: z.string().refine((v) => parseFloat(v) > 0),
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

  const { control, handleSubmit, formState: { errors, isSubmitting } } = useForm<Form>({
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
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4" keyboardShouldPersistTaps="handled">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-6">
          Edit Offer
        </Text>

        {serverError ? (
          <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
            <Text className="text-sm text-error">{serverError}</Text>
          </View>
        ) : null}

        {[
          { name: "pricePerUnit" as const, label: "Price per unit" },
          { name: "minAmount" as const, label: "Min amount" },
          { name: "maxAmount" as const, label: "Max amount" },
          { name: "paymentWindow" as const, label: "Payment window (min)" },
        ].map(({ name, label }) => (
          <View key={name} className="mb-4">
            <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
              {label}
            </Text>
            <Controller
              control={control}
              name={name}
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={String(value ?? "")}
                  keyboardType="decimal-pad"
                />
              )}
            />
            {errors[name]?.message ? (
              <Text className="mt-1 text-xs text-error">{String(errors[name]?.message)}</Text>
            ) : null}
          </View>
        ))}

        <View className="mb-6">
          <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">Terms</Text>
          <Controller
            control={control}
            name="terms"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark h-24"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value ?? ""}
                multiline
                textAlignVertical="top"
              />
            )}
          />
        </View>

        <TouchableOpacity
          onPress={handleSubmit(onSubmit)}
          disabled={isSubmitting}
          className="rounded-lg bg-brand py-4 items-center mb-8"
          activeOpacity={0.8}
        >
          {isSubmitting ? <ActivityIndicator color="#fff" /> : (
            <Text className="text-base font-semibold text-white">Save changes</Text>
          )}
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  )
}
