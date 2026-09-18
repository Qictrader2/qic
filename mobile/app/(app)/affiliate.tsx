import { View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Share } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery } from "@tanstack/react-query"
import { profileService } from "@/src/services/profile.service"
import { useRouter } from "expo-router"
import { Users, TrendingUp, Wallet as WalletIcon, Share2, History, Copy, Check } from "lucide-react-native"
import { useState } from "react"
import { Clipboard } from "react-native"

function StatCard({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof Users
  label: string
  value: string
}) {
  return (
    <View className="flex-1 rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
      <View className="w-9 h-9 rounded-full bg-brand/10 items-center justify-center mb-2">
        <Icon size={16} color="#00A3F6" />
      </View>
      <Text className="text-xs text-muted dark:text-muted-dark">{label}</Text>
      <Text className="text-base font-bold text-foreground dark:text-foreground-dark mt-0.5">
        {value}
      </Text>
    </View>
  )
}

export default function AffiliateScreen() {
  const router = useRouter()
  const [copied, setCopied] = useState(false)
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

  function handleCopy() {
    if (!stats?.referralCode) return
    Clipboard.setString(stats.referralCode)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4" showsVerticalScrollIndicator={false}>
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
          Affiliate program
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mt-1 mb-6">
          Earn commission across three referral levels.
        </Text>

        {isLoading ? (
          <View className="items-center py-20">
            <ActivityIndicator color="#00A3F6" />
          </View>
        ) : stats ? (
          <>
            {/* Referral code card */}
            <View className="rounded-2xl bg-brand p-5 mb-4">
              <Text className="text-xs text-white/80 uppercase tracking-wider">Your referral code</Text>
              <View className="flex-row items-center justify-between mt-2">
                <Text className="text-3xl font-bold text-white tracking-wider">
                  {stats.referralCode}
                </Text>
                <TouchableOpacity
                  onPress={handleCopy}
                  className="bg-white/20 rounded-lg px-3 py-2 flex-row items-center gap-1.5"
                  activeOpacity={0.85}
                >
                  {copied ? (
                    <>
                      <Check size={14} color="#FFFFFF" />
                      <Text className="text-xs text-white font-semibold">Copied</Text>
                    </>
                  ) : (
                    <>
                      <Copy size={14} color="#FFFFFF" />
                      <Text className="text-xs text-white font-semibold">Copy</Text>
                    </>
                  )}
                </TouchableOpacity>
              </View>
            </View>

            {/* Stats grid */}
            <View className="flex-row gap-3 mb-4">
              <StatCard icon={Users} label="Referred" value={String(stats.referredCount)} />
              <StatCard
                icon={WalletIcon}
                label="Pending"
                value={`${stats.pendingEarnings} ${stats.currency}`}
              />
            </View>
            <View className="flex-row gap-3 mb-6">
              <StatCard
                icon={TrendingUp}
                label="Total earned"
                value={`${stats.totalEarnings} ${stats.currency}`}
              />
              <View className="flex-1" />
            </View>

            {/* Actions */}
            <TouchableOpacity
              onPress={handleShare}
              className="rounded-xl bg-brand py-4 items-center flex-row justify-center gap-2 mb-3"
              activeOpacity={0.85}
            >
              <Share2 size={16} color="#FFFFFF" />
              <Text className="text-base font-semibold text-white">Share referral link</Text>
            </TouchableOpacity>

            <TouchableOpacity
              onPress={() => router.push("/(app)/affiliate-commissions")}
              className="rounded-xl border border-border dark:border-border-dark py-3.5 items-center flex-row justify-center gap-2 mb-6"
              activeOpacity={0.85}
            >
              <History size={16} color="#64748B" />
              <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                View commission history
              </Text>
            </TouchableOpacity>

            {/* How it works */}
            <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
              <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mb-3">
                How it works
              </Text>
              {[
                "Share your code with friends",
                "They sign up and start trading",
                "You earn a % on every trade — up to 3 levels deep",
              ].map((s, i) => (
                <View key={s} className="flex-row gap-3 items-start mb-2.5 last:mb-0">
                  <View className="w-6 h-6 rounded-full bg-brand/10 items-center justify-center">
                    <Text className="text-xs font-bold text-brand">{i + 1}</Text>
                  </View>
                  <Text className="text-sm text-foreground dark:text-foreground-dark flex-1">
                    {s}
                  </Text>
                </View>
              ))}
            </View>
          </>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  )
}
