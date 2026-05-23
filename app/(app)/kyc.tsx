import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useKycStatus } from "@/src/hooks/api/use-kyc"
import { kycService, KycTier } from "@/src/services/kyc.service"
import { useState } from "react"
import { useRouter } from "expo-router"

function TierBadge({ tier }: { tier: KycTier }) {
  const labels = ["Unverified", "Basic", "Intermediate", "Advanced"]
  const colors = ["#6B7280", "#F59E0B", "#3B82F6", "#10B981"]
  return (
    <View
      className="px-3 py-1 rounded-full"
      style={{ backgroundColor: (colors[tier] ?? "#6B7280") + "20" }}
    >
      <Text className="text-sm font-semibold" style={{ color: colors[tier] ?? "#6B7280" }}>
        Tier {tier} — {labels[tier]}
      </Text>
    </View>
  )
}

const TIER_BENEFITS: Record<number, string[]> = {
  1: ["Trade up to $500 per day", "Access to marketplace", "Basic deposit/withdraw"],
  2: ["Trade up to $5,000 per day", "Higher withdrawal limits", "Priority support"],
  3: ["Unlimited trading", "Highest limits", "Dedicated account manager"],
}

export default function KycScreen() {
  const router = useRouter()
  const { data: status, isLoading, error, refetch, isRefetching } = useKycStatus()
  const [starting, setStarting] = useState(false)

  async function handleStartVerification() {
    setStarting(true)
    try {
      // Navigate to the WebView screen which handles starting the session
      router.push({
        pathname: "/(app)/kyc-webview/[provider]",
        params: { provider: "didit" },
      })
    } finally {
      setStarting(false)
    }
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView
        className="flex-1 px-4 py-4"
        refreshControl={
          <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
        }
      >
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark mb-6">
          Identity Verification
        </Text>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : error || !status ? (
          <View className="rounded-xl bg-error-bg p-4">
            <Text className="text-sm text-error text-center">Failed to load KYC status.</Text>
          </View>
        ) : (
          <>
            {/* Current tier */}
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <Text className="text-sm text-muted dark:text-muted-dark mb-2">Current status</Text>
              <TierBadge tier={status.tier} />
              {status.status === "pending" ? (
                <View className="mt-3 rounded-lg bg-warning-bg px-3 py-2">
                  <Text className="text-xs text-warning">
                    Verification in progress — usually takes 1–2 business days.
                  </Text>
                </View>
              ) : null}
              {status.status === "rejected" ? (
                <View className="mt-3 rounded-lg bg-error-bg px-3 py-2">
                  <Text className="text-xs text-error">
                    Rejected: {status.rejectionReason ?? "Contact support for details."}
                  </Text>
                </View>
              ) : null}
            </View>

            {/* Tier benefits */}
            {status.tier < 3 ? (
              <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-6">
                <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mb-3">
                  Tier {status.tier + 1} benefits
                </Text>
                {(TIER_BENEFITS[status.tier + 1] ?? []).map((b) => (
                  <View key={b} className="flex-row items-center gap-2 mb-2">
                    <Text className="text-brand">✓</Text>
                    <Text className="text-sm text-foreground dark:text-foreground-dark">{b}</Text>
                  </View>
                ))}
              </View>
            ) : null}

            {/* CTA */}
            {status.status !== "pending" && status.tier < 3 ? (
              <TouchableOpacity
                onPress={handleStartVerification}
                disabled={starting}
                className="rounded-lg bg-brand py-4 items-center"
                activeOpacity={0.8}
              >
                {starting ? (
                  <ActivityIndicator color="#fff" />
                ) : (
                  <Text className="text-base font-semibold text-white">
                    Start Tier {status.tier + 1} verification
                  </Text>
                )}
              </TouchableOpacity>
            ) : null}

            {status.tier === 3 ? (
              <View className="rounded-xl bg-success-bg border border-success/30 p-4 items-center">
                <Text className="text-lg mb-1">🎉</Text>
                <Text className="text-sm font-semibold text-success">Fully verified</Text>
                <Text className="text-xs text-muted dark:text-muted-dark mt-1 text-center">
                  You have the highest verification tier. Enjoy unlimited trading.
                </Text>
              </View>
            ) : null}
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
