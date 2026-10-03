import {
  View,
  Text,
  FlatList,
  ActivityIndicator,
  TouchableOpacity,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery } from "@tanstack/react-query"
import { useRouter } from "expo-router"
import {
  ChevronLeft,
  TrendingUp,
  Wallet as WalletIcon,
  Clock,
  CheckCircle2,
} from "lucide-react-native"
import { profileService } from "@/src/services/profile.service"

export default function AffiliateCommissionsScreen() {
  const router = useRouter()
  const { data: stats, isLoading } = useQuery({
    queryKey: ["affiliate-stats"],
    queryFn: () => profileService.getAffiliateStats(),
  })

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
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
            Commission history
          </Text>
        </View>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : !stats?.payouts.length ? (
        <View className="flex-1 items-center justify-center px-6">
          <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
            <TrendingUp size={24} color="#00A3F6" />
          </View>
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark mb-1">
            No commissions yet
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center max-w-[280px]">
            Share your referral link to start earning commissions on every trade your referrals make.
          </Text>
          <TouchableOpacity
            onPress={() => router.push("/(app)/affiliate" as never)}
            className="mt-5 px-5 h-10 rounded-xl bg-brand items-center justify-center"
            activeOpacity={0.85}
          >
            <Text className="text-sm font-semibold text-white">Get my link</Text>
          </TouchableOpacity>
        </View>
      ) : (
        <FlatList
          data={stats.payouts}
          keyExtractor={(item) => item.id}
          contentContainerStyle={{ paddingHorizontal: 20, paddingBottom: 32 }}
          ListHeaderComponent={
            <View className="flex-row gap-3 mb-4">
              <View className="flex-1 rounded-2xl bg-brand/10 border border-brand/20 p-4">
                <View className="w-9 h-9 rounded-xl bg-brand/20 items-center justify-center mb-2">
                  <WalletIcon size={16} color="#00A3F6" />
                </View>
                <Text className="text-base font-bold text-brand">
                  {stats.totalEarnings} {stats.currency}
                </Text>
                <Text className="text-[11px] text-brand/80 mt-0.5">Total earned</Text>
              </View>
              <View className="flex-1 rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
                <View className="w-9 h-9 rounded-xl bg-warning/20 items-center justify-center mb-2">
                  <Clock size={16} color="#F59E0B" />
                </View>
                <Text className="text-base font-bold text-foreground dark:text-foreground-dark">
                  {stats.pendingEarnings} {stats.currency}
                </Text>
                <Text className="text-[11px] text-muted dark:text-muted-dark mt-0.5">Pending</Text>
              </View>
            </View>
          }
          renderItem={({ item }) => {
            const isPaid = item.status === "paid"
            return (
              <View className="bg-surface dark:bg-card-dark rounded-2xl p-4 mb-3 border border-border dark:border-border-dark">
                <View className="flex-row items-center gap-3">
                  <View
                    className="w-10 h-10 rounded-full items-center justify-center"
                    style={{ backgroundColor: isPaid ? "#10B98122" : "#F59E0B22" }}
                  >
                    {isPaid ? (
                      <CheckCircle2 size={16} color="#10B981" />
                    ) : (
                      <Clock size={16} color="#F59E0B" />
                    )}
                  </View>
                  <View className="flex-1">
                    <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                      {item.amount} {stats.currency}
                    </Text>
                    <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
                      {new Date(item.createdAt).toLocaleDateString(undefined, {
                        year: "numeric",
                        month: "short",
                        day: "numeric",
                      })}
                    </Text>
                  </View>
                  <View
                    className="px-2 py-0.5 rounded-full"
                    style={{ backgroundColor: isPaid ? "#10B98122" : "#F59E0B22" }}
                  >
                    <Text
                      className="text-[10px] font-semibold capitalize"
                      style={{ color: isPaid ? "#10B981" : "#F59E0B" }}
                    >
                      {item.status}
                    </Text>
                  </View>
                </View>
              </View>
            )
          }}
        />
      )}
    </SafeAreaView>
  )
}
