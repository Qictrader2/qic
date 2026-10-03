import {
  View,
  Text,
  ScrollView,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  Linking,
  KeyboardAvoidingView,
  Platform,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import {
  ChevronLeft,
  ChevronDown,
  HelpCircle,
  Mail,
  MessageSquare,
  CheckCircle2,
  AlertTriangle,
  LifeBuoy,
} from "lucide-react-native"
import { useRouter } from "expo-router"
import { apiClient } from "@/src/lib/api/client"

const schema = z.object({
  subject: z.string().min(5, "Enter a subject"),
  message: z.string().min(20, "Please describe your issue in more detail"),
})
type Form = z.infer<typeof schema>

const FAQS = [
  {
    q: "How do I deposit?",
    a: "Go to Wallet → select a currency → tap Deposit to get your address.",
  },
  {
    q: "How long do deposits take?",
    a: "Depends on network. BTC: ~3 confirmations (30 min). ETH: ~12 blocks (2-3 min). SOL: ~1 min.",
  },
  {
    q: "How does escrow work?",
    a: "The seller's crypto is locked in escrow. Once you pay and the seller confirms, crypto is released to you.",
  },
  {
    q: "What if the seller doesn't release?",
    a: "Use the dispute button on the trade. Our team reviews within 24h.",
  },
  {
    q: "How do I cancel a trade?",
    a: "Tap Cancel on the trade detail screen. Only possible before the seller confirms payment.",
  },
]

export default function SupportScreen() {
  const router = useRouter()
  const [submitted, setSubmitted] = useState(false)
  const [serverError, setServerError] = useState<string | null>(null)
  const [openFaq, setOpenFaq] = useState<number | null>(null)

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<Form>({ resolver: zodResolver(schema) })

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await apiClient.post("/api/v1/support/tickets", data)
      setSubmitted(true)
    } catch {
      setServerError("Failed to submit ticket. Please try again or email support@qictrader.com")
    }
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
              Support
            </Text>
          </View>
        </View>

        <ScrollView
          className="flex-1 px-5"
          showsVerticalScrollIndicator={false}
          keyboardShouldPersistTaps="handled"
        >
          <View className="rounded-2xl bg-brand/10 border border-brand/20 p-4 mb-5 flex-row items-center gap-3">
            <View className="w-10 h-10 rounded-full bg-brand/20 items-center justify-center">
              <LifeBuoy size={18} color="#00A3F6" />
            </View>
            <View className="flex-1">
              <Text className="text-sm font-semibold text-brand">We're here to help</Text>
              <Text className="text-xs text-brand/80 mt-0.5">Typical response time: under 24h</Text>
            </View>
          </View>

          {/* FAQ */}
          <View className="flex-row items-center gap-2 mb-3">
            <HelpCircle size={14} color="#00A3F6" />
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
              Frequently asked
            </Text>
          </View>
          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark overflow-hidden mb-6">
            {FAQS.map((faq, i) => {
              const isOpen = openFaq === i
              return (
                <View key={i}>
                  <TouchableOpacity
                    onPress={() => setOpenFaq(isOpen ? null : i)}
                    className="px-4 py-3.5 flex-row justify-between items-center"
                    activeOpacity={0.7}
                  >
                    <Text className="text-sm font-medium text-foreground dark:text-foreground-dark flex-1 mr-2">
                      {faq.q}
                    </Text>
                    <View
                      style={{
                        transform: [{ rotate: isOpen ? "180deg" : "0deg" }],
                      }}
                    >
                      <ChevronDown size={16} color="#64748B" />
                    </View>
                  </TouchableOpacity>
                  {isOpen ? (
                    <View className="px-4 pb-3.5">
                      <Text className="text-sm text-muted dark:text-muted-dark leading-5">
                        {faq.a}
                      </Text>
                    </View>
                  ) : null}
                  {i < FAQS.length - 1 ? (
                    <View className="h-px bg-border/40 dark:bg-border-dark/40 mx-4" />
                  ) : null}
                </View>
              )
            })}
          </View>

          {/* Submit ticket */}
          <View className="flex-row items-center gap-2 mb-3">
            <MessageSquare size={14} color="#00A3F6" />
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
              Submit a ticket
            </Text>
          </View>

          {submitted ? (
            <View className="rounded-2xl bg-success/10 border border-success/20 p-5 items-center mb-4">
              <View className="w-12 h-12 rounded-full bg-success/20 items-center justify-center mb-3">
                <CheckCircle2 size={20} color="#10B981" />
              </View>
              <Text className="text-base font-semibold text-success">Ticket submitted</Text>
              <Text className="text-sm text-muted dark:text-muted-dark mt-1 text-center max-w-[280px]">
                We'll get back to you within 24 hours via email.
              </Text>
            </View>
          ) : (
            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-4">
              {serverError ? (
                <View className="mb-3 rounded-xl bg-error-bg border border-error/20 px-3 py-2 flex-row items-center gap-2">
                  <AlertTriangle size={12} color="#EF4444" />
                  <Text className="text-xs text-error flex-1">{serverError}</Text>
                </View>
              ) : null}

              <Text className="mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                Subject
              </Text>
              <Controller
                control={control}
                name="subject"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="h-12 rounded-xl border border-border dark:border-border-dark bg-background dark:bg-background-dark px-4 text-sm text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value ?? ""}
                    placeholder="e.g. Trade not released after payment"
                    placeholderTextColor="#94A3B8"
                  />
                )}
              />
              {errors.subject?.message ? (
                <Text className="mt-1 text-xs text-error">{errors.subject.message}</Text>
              ) : null}

              <Text className="mt-4 mb-2 text-sm font-medium text-foreground dark:text-foreground-dark">
                Message
              </Text>
              <Controller
                control={control}
                name="message"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-xl border border-border dark:border-border-dark bg-background dark:bg-background-dark px-4 py-3 text-sm text-foreground dark:text-foreground-dark h-32"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value ?? ""}
                    placeholder="Describe your issue. Include trade IDs, screenshots if relevant…"
                    placeholderTextColor="#94A3B8"
                    multiline
                    textAlignVertical="top"
                  />
                )}
              />
              {errors.message?.message ? (
                <Text className="mt-1 text-xs text-error">{errors.message.message}</Text>
              ) : null}

              <TouchableOpacity
                onPress={handleSubmit(onSubmit)}
                disabled={isSubmitting}
                className="mt-4 rounded-xl bg-brand h-12 items-center justify-center"
                activeOpacity={0.85}
              >
                {isSubmitting ? (
                  <ActivityIndicator color="#fff" />
                ) : (
                  <Text className="text-sm font-semibold text-white">Submit ticket</Text>
                )}
              </TouchableOpacity>
            </View>
          )}

          <TouchableOpacity
            onPress={() => Linking.openURL("mailto:support@qictrader.com")}
            className="flex-row items-center justify-center gap-2 py-3 mb-6"
            activeOpacity={0.7}
          >
            <Mail size={14} color="#00A3F6" />
            <Text className="text-sm font-medium text-brand">support@qictrader.com</Text>
          </TouchableOpacity>
        </ScrollView>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
