import {
  View, Text, ScrollView, TextInput, TouchableOpacity, ActivityIndicator, Linking,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { apiClient } from "@/src/lib/api/client"

const schema = z.object({
  subject: z.string().min(5, "Enter a subject"),
  message: z.string().min(20, "Please describe your issue in more detail"),
})
type Form = z.infer<typeof schema>

const FAQS = [
  { q: "How do I deposit?", a: "Go to Wallet → select a currency → tap Deposit to get your address." },
  { q: "How long do deposits take?", a: "Depends on network. BTC: ~3 confirmations (30 min). ETH: ~12 blocks (2-3 min). SOL: ~1 min." },
  { q: "How does escrow work?", a: "The seller's crypto is locked in escrow. Once you pay and the seller confirms, crypto is released to you." },
  { q: "What if the seller doesn't release?", a: "Use the dispute button on the trade. Our team reviews within 24h." },
  { q: "How do I cancel a trade?", a: "Tap Cancel on the trade detail screen. Only possible before the seller confirms payment." },
]

export default function SupportScreen() {
  const [submitted, setSubmitted] = useState(false)
  const [serverError, setServerError] = useState<string | null>(null)
  const [openFaq, setOpenFaq] = useState<number | null>(null)

  const { control, handleSubmit, formState: { errors, isSubmitting } } = useForm<Form>({
    resolver: zodResolver(schema),
  })

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
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-6">Support</Text>

        {/* FAQ */}
        <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mb-3">Frequently asked questions</Text>
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark mb-6 overflow-hidden">
          {FAQS.map((faq, i) => (
            <View key={i}>
              <TouchableOpacity
                onPress={() => setOpenFaq(openFaq === i ? null : i)}
                className="px-4 py-3.5 flex-row justify-between items-center"
                activeOpacity={0.7}
              >
                <Text className="text-sm text-foreground dark:text-foreground-dark flex-1 mr-2">
                  {faq.q}
                </Text>
                <Text className="text-muted dark:text-muted-dark text-lg">
                  {openFaq === i ? "−" : "+"}
                </Text>
              </TouchableOpacity>
              {openFaq === i ? (
                <View className="px-4 pb-3.5">
                  <Text className="text-sm text-muted dark:text-muted-dark leading-relaxed">{faq.a}</Text>
                </View>
              ) : null}
              {i < FAQS.length - 1 ? (
                <View className="h-px bg-border/50 dark:bg-border-dark/50 mx-4" />
              ) : null}
            </View>
          ))}
        </View>

        {/* Submit ticket */}
        <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mb-3">
          Submit a support ticket
        </Text>

        {submitted ? (
          <View className="rounded-xl bg-success-bg border border-success/30 p-4 items-center">
            <Text className="text-lg mb-1">✅</Text>
            <Text className="text-sm font-semibold text-success">Ticket submitted</Text>
            <Text className="text-xs text-muted dark:text-muted-dark mt-1 text-center">
              We'll get back to you within 24 hours.
            </Text>
          </View>
        ) : (
          <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4">
            {serverError ? (
              <View className="mb-3 rounded-lg bg-error-bg px-3 py-2">
                <Text className="text-xs text-error">{serverError}</Text>
              </View>
            ) : null}

            <View className="mb-3">
              <Text className="mb-1 text-xs font-medium text-muted dark:text-muted-dark">Subject</Text>
              <Controller
                control={control}
                name="subject"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-lg border border-border dark:border-border-dark bg-background dark:bg-background-dark px-3 py-2.5 text-sm text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value}
                    placeholder="e.g. Trade not released after payment"
                    placeholderTextColor="#94A3B8"
                  />
                )}
              />
              {errors.subject?.message ? (
                <Text className="mt-0.5 text-xs text-error">{errors.subject.message}</Text>
              ) : null}
            </View>

            <View className="mb-4">
              <Text className="mb-1 text-xs font-medium text-muted dark:text-muted-dark">Message</Text>
              <Controller
                control={control}
                name="message"
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-lg border border-border dark:border-border-dark bg-background dark:bg-background-dark px-3 py-2.5 text-sm text-foreground dark:text-foreground-dark h-32"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value}
                    placeholder="Describe your issue…"
                    placeholderTextColor="#94A3B8"
                    multiline
                    textAlignVertical="top"
                  />
                )}
              />
              {errors.message?.message ? (
                <Text className="mt-0.5 text-xs text-error">{errors.message.message}</Text>
              ) : null}
            </View>

            <TouchableOpacity
              onPress={handleSubmit(onSubmit)}
              disabled={isSubmitting}
              className="rounded-lg bg-brand py-3.5 items-center"
              activeOpacity={0.8}
            >
              {isSubmitting ? <ActivityIndicator color="#fff" /> : (
                <Text className="text-sm font-semibold text-white">Submit ticket</Text>
              )}
            </TouchableOpacity>
          </View>
        )}

        <TouchableOpacity
          onPress={() => Linking.openURL("mailto:support@qictrader.com")}
          className="mt-4 py-3 items-center"
        >
          <Text className="text-sm text-brand">Or email support@qictrader.com</Text>
        </TouchableOpacity>

        <View className="h-8" />
      </ScrollView>
    </SafeAreaView>
  )
}
