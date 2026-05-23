import { View, Text, FlatList, ActivityIndicator } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery } from "@tanstack/react-query"
import { profileService } from "@/src/services/profile.service"

export default function AffiliateCommissionsScreen() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ["affiliate-stats"],
    queryFn: () => profileService.getAffiliateStats(),
  })

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">
          Commission History
        </Text>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : !stats?.payouts.length ? (
        <View className="flex-1 items-center justify-center py-20">
          <Text className="text-4xl mb-3">💰</Text>
          <Text className="text-base font-medium text-foreground dark:text-foreground-dark mb-1">
            No commissions yet
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center px-8">
            Share your referral link to start earning commissions.
          </Text>
        </View>
      ) : (
        <>
          <View className="mx-4 mb-4 rounded-xl bg-brand-bg border border-brand/30 p-4 flex-row justify-between">
            <View>
              <Text className="text-xs text-muted dark:text-muted-dark mb-1">Total earned</Text>
              <Text className="text-xl font-bold text-brand">
                {stats.totalEarnings} {stats.currency}
              </Text>
            </View>
            <View>
              <Text className="text-xs text-muted dark:text-muted-dark mb-1">Pending</Text>
              <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">
                {stats.pendingEarnings} {stats.currency}
              </Text>
            </View>
          </View>
          <FlatList
            data={stats.payouts}
            keyExtractor={(item) => item.id}
            renderItem={({ item }) => (
              <View className="flex-row items-center justify-between px-4 py-3.5 border-b border-border/50 dark:border-border-dark/50">
                <View>
                  <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                    {item.amount} {stats.currency}
                  </Text>
                  <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
                    {new Date(item.createdAt).toLocaleDateString()}
                  </Text>
                </View>
                <View
                  className={`px-2.5 py-1 rounded-full ${
                    item.status === "paid" ? "bg-success-bg" : "bg-warning-bg"
                  }`}
                >
                  <Text className={`text-xs font-medium capitalize ${item.status === "paid" ? "text-success" : "text-warning"}`}>
                    {item.status}
                  </Text>
                </View>
              </View>
            )}
          />
        </>
      )}
    </SafeAreaView>
  )
}
