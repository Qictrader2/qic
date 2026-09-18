import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { CheckCircle2, ShieldCheck, Clock, XCircle } from "lucide-react-native"
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
  1: ["Trade up to $1,000 / day", "Marketplace access", "Government ID required"],
  2: ["Trade up to $10,000 / day", "Higher withdrawal limits", "ID + selfie liveness"],
  3: ["Trade up to $100,000 / day", "Highest limits available", "Plus proof of address"],
}

const TIER_REQUIREMENTS: Record<number, string[]> = {
  1: ["Government-issued ID"],
  2: ["Government ID", "Selfie liveness check"],
  3: ["Government ID", "Selfie liveness check", "Proof of address"],
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
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
          Identity verification
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mt-1 mb-6">
          Higher tiers unlock larger daily and monthly trade volumes.
        </Text>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : error || !status ? (
          <View className="rounded-xl bg-error-bg p-4">
            <Text className="text-sm text-error text-center">Failed to load KYC status.</Text>
          </View>
        ) : (
          <>
            {/* Current tier card */}
            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-4">
              <View className="flex-row items-center justify-between mb-3">
                <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider">
                  Current status
                </Text>
                <TierBadge tier={status.tier} />
              </View>
              {status.status === "pending" ? (
                <View className="rounded-lg bg-warning-bg px-3 py-2.5 flex-row items-center gap-2">
                  <Clock size={14} color="#F59E0B" />
                  <Text className="text-xs text-warning flex-1">
                    Verification in progress — usually takes 1–2 business days.
                  </Text>
                </View>
              ) : status.status === "rejected" ? (
                <View className="rounded-lg bg-error-bg px-3 py-2.5 flex-row items-start gap-2">
                  <XCircle size={14} color="#EF4444" />
                  <Text className="text-xs text-error flex-1">
                    Rejected: {status.rejectionReason ?? "Contact support for details."}
                  </Text>
                </View>
              ) : status.tier === 0 ? (
                <Text className="text-xs text-muted dark:text-muted-dark">
                  Start verification to unlock trading.
                </Text>
              ) : (
                <View className="flex-row items-center gap-2">
                  <CheckCircle2 size={14} color="#10B981" />
                  <Text className="text-xs text-success">Tier {status.tier} approved</Text>
                </View>
              )}
            </View>

            {/* Next-tier benefits + requirements */}
            {status.tier < 3 ? (
              <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4 mb-3">
                <View className="flex-row items-center gap-2 mb-3">
                  <ShieldCheck size={16} color="#00A3F6" />
                  <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                    Upgrade to Tier {status.tier + 1}
                  </Text>
                </View>
                <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mb-2">
                  What you unlock
                </Text>
                {(TIER_BENEFITS[status.tier + 1] ?? []).map((b) => (
                  <View key={b} className="flex-row items-center gap-2 mb-1.5">
                    <CheckCircle2 size={12} color="#10B981" />
                    <Text className="text-xs text-foreground dark:text-foreground-dark">{b}</Text>
                  </View>
                ))}
                <View className="h-px bg-border dark:bg-border-dark my-3" />
                <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mb-2">
                  What you need
                </Text>
                {(TIER_REQUIREMENTS[status.tier + 1] ?? []).map((r) => (
                  <Text key={r} className="text-xs text-muted dark:text-muted-dark mb-1">
                    • {r}
                  </Text>
                ))}
              </View>
            ) : null}

            {/* CTA */}
            {status.status !== "pending" && status.tier < 3 ? (
              <TouchableOpacity
                onPress={handleStartVerification}
                disabled={starting}
                className="rounded-xl bg-brand py-4 items-center mt-2"
                activeOpacity={0.85}
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
              <View className="rounded-2xl bg-success-bg border border-success/30 p-5 items-center">
                <View className="w-14 h-14 rounded-full bg-success items-center justify-center mb-3">
                  <CheckCircle2 size={24} color="#FFFFFF" />
                </View>
                <Text className="text-base font-semibold text-success">Fully verified</Text>
                <Text className="text-xs text-muted dark:text-muted-dark mt-1 text-center max-w-[240px]">
                  You have the highest verification tier. Trade up to $100,000 per day.
                </Text>
              </View>
            ) : null}
          </>
        )}
      </ScrollView>
    </SafeAreaView>
  )
}
