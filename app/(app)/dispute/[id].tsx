import {
  View, Text, TextInput, TouchableOpacity, ActivityIndicator, ScrollView,
  Image, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import * as ImagePicker from "expo-image-picker"
import { tradeService } from "@/src/services/trade.service"
import { useQueryClient } from "@tanstack/react-query"
import { ApiError } from "@/src/lib/api/client"

const DISPUTE_REASONS = [
  "Seller hasn't released after payment confirmed",
  "Buyer claims paid but I have no record",
  "Payment received but wrong amount",
  "Fraud / scam attempt",
  "Technical issue",
  "Other",
]

const schema = z.object({
  reason: z.string().min(1, "Select a reason"),
  details: z.string().min(20, "Describe the issue in at least 20 characters"),
})
type Form = z.infer<typeof schema>

export default function DisputeScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const router = useRouter()
  const qc = useQueryClient()
  const [evidence, setEvidence] = useState<Array<{ uri: string; name: string }>>([])
  const [submitting, setSubmitting] = useState(false)
  const [serverError, setServerError] = useState<string | null>(null)
  const [submitted, setSubmitted] = useState(false)

  const {
    control, handleSubmit, setValue, watch,
    formState: { errors },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  const selectedReason = watch("reason")

  async function addEvidence() {
    if (evidence.length >= 5) {
      Alert.alert("Limit reached", "You can upload up to 5 evidence files.")
      return
    }
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: false,
      quality: 0.8,
    })
    if (!result.canceled && result.assets[0]) {
      const asset = result.assets[0]
      setEvidence((prev) => [
        ...prev,
        { uri: asset.uri, name: asset.fileName ?? `evidence-${prev.length + 1}.jpg` },
      ])
    }
  }

  async function uploadEvidence(items: typeof evidence): Promise<string[]> {
    const urls: string[] = []
    for (const item of items) {
      const formData = new FormData()
      formData.append("file", { uri: item.uri, name: item.name, type: "image/jpeg" } as never)
      const res = await tradeService.uploadProofOfPayment(id ?? "", formData)
      urls.push(res.url)
    }
    return urls
  }

  async function onSubmit(data: Form) {
    setServerError(null)
    setSubmitting(true)
    try {
      const evidenceUrls = evidence.length > 0 ? await uploadEvidence(evidence) : []
      await tradeService.openDispute(id ?? "", `${data.reason}: ${data.details}`, evidenceUrls)
      qc.invalidateQueries({ queryKey: ["trade", id] })
      setSubmitted(true)
    } catch (err) {
      const apiErr = err as ApiError
      setServerError(
        apiErr.kind === "forbidden"
          ? "You cannot open a dispute on this trade."
          : "Failed to submit dispute. Please try again."
      )
    } finally {
      setSubmitting(false)
    }
  }

  if (submitted) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
        <View className="items-center">
          <Text className="text-5xl mb-4">⚖️</Text>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
            Dispute opened
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8">
            Our team will review the evidence and resolve within 24–48 hours.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-lg bg-brand px-8 py-3.5"
          >
            <Text className="text-base font-semibold text-white">Back to trade</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4" keyboardShouldPersistTaps="handled">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2">
          Open Dispute
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-6 leading-relaxed">
          Only open a dispute if you cannot resolve the issue directly with the counterparty
          in the trade chat.
        </Text>

        {serverError ? (
          <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
            <Text className="text-sm text-error">{serverError}</Text>
          </View>
        ) : null}

        {/* Reason */}
        <View className="mb-4">
          <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
            Reason
          </Text>
          <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark overflow-hidden">
            {DISPUTE_REASONS.map((r, i) => (
              <TouchableOpacity
                key={r}
                onPress={() => setValue("reason", r)}
                className={`flex-row items-center justify-between px-4 py-3.5 ${
                  i < DISPUTE_REASONS.length - 1
                    ? "border-b border-border/50 dark:border-border-dark/50"
                    : ""
                }`}
                activeOpacity={0.7}
              >
                <Text className="text-sm text-foreground dark:text-foreground-dark flex-1 mr-3">
                  {r}
                </Text>
                {selectedReason === r ? (
                  <View className="h-5 w-5 rounded-full bg-brand items-center justify-center">
                    <Text className="text-white text-xs">✓</Text>
                  </View>
                ) : (
                  <View className="h-5 w-5 rounded-full border-2 border-border dark:border-border-dark" />
                )}
              </TouchableOpacity>
            ))}
          </View>
          {errors.reason ? (
            <Text className="mt-1 text-xs text-error">{errors.reason.message}</Text>
          ) : null}
        </View>

        {/* Details */}
        <View className="mb-4">
          <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">
            Details
          </Text>
          <Controller
            control={control}
            name="details"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark h-32"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value}
                placeholder="Describe the issue clearly. Include payment method, amounts, dates."
                placeholderTextColor="#94A3B8"
                multiline
                textAlignVertical="top"
              />
            )}
          />
          {errors.details ? (
            <Text className="mt-1 text-xs text-error">{errors.details.message}</Text>
          ) : null}
        </View>

        {/* Evidence */}
        <View className="mb-6">
          <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
            Evidence ({evidence.length}/5)
          </Text>
          <View className="flex-row flex-wrap gap-2 mb-2">
            {evidence.map((e, i) => (
              <View key={i} className="relative">
                <Image
                  source={{ uri: e.uri }}
                  className="h-20 w-20 rounded-lg"
                  resizeMode="cover"
                />
                <TouchableOpacity
                  onPress={() => setEvidence((prev) => prev.filter((_, idx) => idx !== i))}
                  className="absolute -top-1.5 -right-1.5 h-5 w-5 rounded-full bg-error items-center justify-center"
                >
                  <Text className="text-white text-xs font-bold">×</Text>
                </TouchableOpacity>
              </View>
            ))}
            {evidence.length < 5 ? (
              <TouchableOpacity
                onPress={addEvidence}
                className="h-20 w-20 rounded-lg border-2 border-dashed border-brand/50 bg-brand-bg/30 items-center justify-center"
                activeOpacity={0.7}
              >
                <Text className="text-brand text-2xl">+</Text>
              </TouchableOpacity>
            ) : null}
          </View>
        </View>

        <TouchableOpacity
          onPress={handleSubmit(onSubmit)}
          disabled={submitting}
          className="rounded-lg bg-error py-4 items-center mb-8"
          activeOpacity={0.8}
        >
          {submitting ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text className="text-base font-semibold text-white">Submit dispute</Text>
          )}
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  )
}
