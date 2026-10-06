import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  ScrollView,
  Image,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import * as ImagePicker from "expo-image-picker"
import {
  ChevronLeft,
  Scale,
  AlertTriangle,
  CheckCircle2,
  Plus,
  X,
  ImagePlus,
} from "lucide-react-native"
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
    control,
    handleSubmit,
    setValue,
    watch,
    formState: { errors },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  const selectedReason = watch("reason")
  const details = watch("details") ?? ""

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
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
        <View className="flex-1 items-center justify-center px-6">
          <View className="w-20 h-20 rounded-full bg-success/10 items-center justify-center mb-5">
            <CheckCircle2 size={36} color="#10B981" />
          </View>
          <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-2 text-center">
            Dispute opened
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8 max-w-[320px]">
            Our team will review the evidence and resolve within 24–48 hours. You'll receive an update via email and in-app notification.
          </Text>
          <TouchableOpacity
            onPress={() => router.back()}
            className="rounded-xl bg-brand px-8 h-12 items-center justify-center"
            activeOpacity={0.85}
          >
            <Text className="text-base font-semibold text-white">Back to trade</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    )
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1" edges={["bottom"]}>
        <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
          <TouchableOpacity
            onPress={() => router.back()}
            className="w-10 h-10 items-center justify-center -ml-2"
            activeOpacity={0.7}
          >
            <ChevronLeft size={24} color="#64748B" />
          </TouchableOpacity>
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Open dispute
          </Text>
          <View className="w-10" />
        </View>

        <ScrollView
          className="flex-1 px-5"
          keyboardShouldPersistTaps="handled"
          showsVerticalScrollIndicator={false}
        >
          <View className="rounded-2xl bg-warning-bg border border-warning/20 p-4 mb-5 flex-row items-start gap-3">
            <View className="w-9 h-9 rounded-full bg-warning/20 items-center justify-center">
              <Scale size={16} color="#F59E0B" />
            </View>
            <View className="flex-1">
              <Text className="text-sm font-semibold text-warning mb-1">
                Last resort
              </Text>
              <Text className="text-xs text-warning/90 leading-5">
                Only open a dispute if you can't resolve the issue with the counterparty in the trade chat. Be honest — our team reviews all evidence.
              </Text>
            </View>
          </View>

          {serverError ? (
            <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3 flex-row items-center gap-2">
              <AlertTriangle size={14} color="#EF4444" />
              <Text className="text-sm text-error flex-1">{serverError}</Text>
            </View>
          ) : null}

          {/* Reason */}
          <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
            What's the issue?
          </Text>
          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark overflow-hidden mb-4">
            {DISPUTE_REASONS.map((r, i) => {
              const selected = selectedReason === r
              return (
                <TouchableOpacity
                  key={r}
                  onPress={() => setValue("reason", r)}
                  className={`flex-row items-center justify-between px-4 py-3.5 ${
                    i < DISPUTE_REASONS.length - 1
                      ? "border-b border-border/40 dark:border-border-dark/40"
                      : ""
                  }`}
                  activeOpacity={0.7}
                >
                  <Text className="text-sm text-foreground dark:text-foreground-dark flex-1 mr-3">
                    {r}
                  </Text>
                  <View
                    className={`h-5 w-5 rounded-full items-center justify-center ${
                      selected
                        ? "bg-brand"
                        : "border-2 border-border dark:border-border-dark"
                    }`}
                  >
                    {selected ? <CheckCircle2 size={12} color="#FFFFFF" /> : null}
                  </View>
                </TouchableOpacity>
              )
            })}
          </View>
          {errors.reason ? (
            <Text className="-mt-3 mb-3 text-xs text-error">{errors.reason.message}</Text>
          ) : null}

          {/* Details */}
          <View className="flex-row items-center justify-between mb-2">
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
              Details
            </Text>
            <Text className="text-xs text-muted dark:text-muted-dark">
              {details.length}/500
            </Text>
          </View>
          <Controller
            control={control}
            name="details"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 py-3 text-sm text-foreground dark:text-foreground-dark h-32"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value ?? ""}
                placeholder="Describe the issue clearly. Include payment method, amounts, dates, and what went wrong."
                placeholderTextColor="#94A3B8"
                multiline
                textAlignVertical="top"
                maxLength={500}
              />
            )}
          />
          {errors.details ? (
            <Text className="mt-1 text-xs text-error">{errors.details.message}</Text>
          ) : null}

          {/* Evidence */}
          <View className="mt-5">
            <View className="flex-row items-center justify-between mb-3">
              <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                Evidence
              </Text>
              <Text className="text-xs text-muted dark:text-muted-dark">{evidence.length}/5</Text>
            </View>
            <View className="flex-row flex-wrap gap-3">
              {evidence.map((e, i) => (
                <View key={i} className="relative">
                  <Image
                    source={{ uri: e.uri }}
                    className="h-20 w-20 rounded-xl"
                    resizeMode="cover"
                  />
                  <TouchableOpacity
                    onPress={() => setEvidence((prev) => prev.filter((_, idx) => idx !== i))}
                    className="absolute -top-1.5 -right-1.5 h-6 w-6 rounded-full bg-error items-center justify-center border-2 border-background dark:border-background-dark"
                    activeOpacity={0.85}
                  >
                    <X size={11} color="#FFFFFF" />
                  </TouchableOpacity>
                </View>
              ))}
              {evidence.length < 5 ? (
                <TouchableOpacity
                  onPress={addEvidence}
                  className="h-20 w-20 rounded-xl border-2 border-dashed border-brand/40 bg-brand/5 items-center justify-center"
                  activeOpacity={0.7}
                >
                  <ImagePlus size={20} color="#00A3F6" />
                  <Text className="text-[10px] text-brand mt-1 font-medium">Add</Text>
                </TouchableOpacity>
              ) : null}
            </View>
            <Text className="text-xs text-muted dark:text-muted-dark mt-2">
              Screenshots of payment proof, chat messages, or transaction history help us resolve faster.
            </Text>
          </View>

          <View className="h-24" />
        </ScrollView>

        <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={submitting}
            className="rounded-xl bg-error h-12 items-center justify-center flex-row gap-2"
            activeOpacity={0.85}
          >
            {submitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <>
                <Scale size={16} color="#FFFFFF" />
                <Text className="text-base font-semibold text-white">Submit dispute</Text>
              </>
            )}
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
