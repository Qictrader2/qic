import { View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Share } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery } from "@tanstack/react-query"
import { profileService } from "@/src/services/profile.service"
import { useRouter } from "expo-router"

export default function AffiliateScreen() {
  const router = useRouter()
  const { data: stats, isLoading } = useQuery({
    queryKey: ["affiliate-stats"],
    queryFn: () => profileService.getAffiliateStats(),
  })

  async function handleShare() {
    if (!stats?.referralUrl) return
    await Share.share({
      message: `Join QicTrader — South Africa's leading P2P crypto exchange: ${stats.referralUrl}`,
    })
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-6">Affiliate Program</Text>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : stats ? (
          <>
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <Text className="text-xs text-muted dark:text-muted-dark mb-1">Your referral code</Text>
              <Text className="text-2xl font-bold text-brand tracking-wider">{stats.referralCode}</Text>
            </View>

            <View className="flex-row gap-3 mb-4">
              {[
                { label: "Referred", value: String(stats.referredCount) },
                { label: "Pending", value: `${stats.pendingEarnings} ${stats.currency}` },
                { label: "Total earned", value: `${stats.totalEarnings} ${stats.currency}` },
              ].map(({ label, value }) => (
                <View key={label} className="flex-1 rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-3 items-center">
                  <Text className="text-xs text-muted dark:text-muted-dark mb-1 text-center">{label}</Text>
                  <Text className="text-sm font-bold text-foreground dark:text-foreground-dark text-center">{value}</Text>
                </View>
              ))}
            </View>

            <TouchableOpacity
              onPress={handleShare}
              className="rounded-lg bg-brand py-4 items-center mb-3"
              activeOpacity={0.8}
            >
              <Text className="text-base font-semibold text-white">Share referral link</Text>
            </TouchableOpacity>

            <TouchableOpacity
              onPress={() => router.push("/(app)/affiliate-commissions")}
              className="rounded-lg border border-border dark:border-border-dark py-3.5 items-center mb-6"
              activeOpacity={0.8}
            >
              <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">View commission history</Text>
            </TouchableOpacity>
          </>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  )
}
