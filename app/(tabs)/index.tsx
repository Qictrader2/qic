import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  Image,
  Dimensions,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import {
  ShieldCheck,
  Zap,
  Wallet as WalletIcon,
  TrendingUp,
  Repeat,
  ArrowRight,
  Search,
  Plus,
  Bell,
  CheckCircle2,
  Users,
  type LucideIcon,
} from "lucide-react-native"
import { useAuthStore } from "@/src/store/auth-store"
import { useWallets } from "@/src/hooks/api/use-wallet"
import { useActiveTrades } from "@/src/hooks/api/use-trade"

const { width: SCREEN_WIDTH } = Dimensions.get("window")
const heroImg = require("@/assets/images/hero.png")

function TrustBadge({ icon: Icon, label }: { icon: LucideIcon; label: string }) {
  return (
    <View className="flex-row items-center gap-1.5">
      <Icon size={14} color="#00A3F6" />
      <Text className="text-xs text-muted dark:text-muted-dark">{label}</Text>
    </View>
  )
}

function StepCard({ n, title, body, icon: Icon }: { n: number; title: string; body: string; icon: LucideIcon }) {
  return (
    <View className="bg-surface dark:bg-card-dark rounded-2xl p-4 border border-border dark:border-border-dark">
      <View className="flex-row items-center gap-3 mb-2">
        <View className="w-10 h-10 rounded-full bg-brand items-center justify-center">
          <Icon size={18} color="#FFFFFF" />
        </View>
        <View className="flex-row items-baseline gap-2">
          <Text className="text-xs font-semibold text-brand">Step {n}</Text>
          <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">{title}</Text>
        </View>
      </View>
      <Text className="text-xs text-muted dark:text-muted-dark leading-5">{body}</Text>
    </View>
  )
}

function FeatureRow({ icon: Icon, title, body }: { icon: LucideIcon; title: string; body: string }) {
  return (
    <View className="flex-row gap-3 py-3">
      <View className="w-10 h-10 rounded-lg bg-brand/10 items-center justify-center mt-0.5">
        <Icon size={18} color="#00A3F6" />
      </View>
      <View className="flex-1">
        <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">{title}</Text>
        <Text className="text-xs text-muted dark:text-muted-dark mt-0.5 leading-5">{body}</Text>
      </View>
    </View>
  )
}

function StatTile({ label, value }: { label: string; value: string }) {
  return (
    <View className="flex-1 bg-surface dark:bg-card-dark rounded-2xl p-3 border border-border dark:border-border-dark">
      <Text className="text-xs text-muted dark:text-muted-dark">{label}</Text>
      <Text className="text-lg font-bold text-foreground dark:text-foreground-dark mt-1">{value}</Text>
    </View>
  )
}

function QuickAction({ icon: Icon, label, onPress, color = "#00A3F6" }: { icon: LucideIcon; label: string; onPress: () => void; color?: string }) {
  return (
    <TouchableOpacity onPress={onPress} className="flex-1 items-center" activeOpacity={0.7}>
      <View className="w-12 h-12 rounded-full bg-brand/10 items-center justify-center mb-1.5">
        <Icon size={20} color={color} />
      </View>
      <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">{label}</Text>
    </TouchableOpacity>
  )
}

export default function HomeScreen() {
  const router = useRouter()
  const { isAuthenticated, user } = useAuthStore()

  // ── Unauthenticated landing
  if (!isAuthenticated) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["top"]}>
        <ScrollView showsVerticalScrollIndicator={false} contentContainerStyle={{ paddingBottom: 40 }}>
          {/* Top bar with logo + sign in */}
          <View className="flex-row items-center justify-between px-5 pt-2 pb-3">
            <View className="flex-row items-center gap-2">
              <Image source={require("@/assets/images/logo-small.png")} style={{ width: 28, height: 28 }} resizeMode="contain" />
              <Text className="text-base font-bold text-foreground dark:text-foreground-dark">QicTrader</Text>
            </View>
            <View className="flex-row gap-2">
              <TouchableOpacity
                onPress={() => router.push("/(auth)/login")}
                className="px-4 py-1.5 rounded-lg"
                activeOpacity={0.7}
              >
                <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">Sign in</Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={() => router.push("/(auth)/signup")}
                className="px-4 py-1.5 rounded-lg bg-brand"
                activeOpacity={0.85}
              >
                <Text className="text-sm font-semibold text-white">Sign up</Text>
              </TouchableOpacity>
            </View>
          </View>

          {/* Hero */}
          <View className="px-5 pt-4">
            <Text className="text-3xl leading-tight font-medium text-foreground dark:text-foreground-dark">
              Trade Crypto — Or <Text className="text-brand-blue-light">Resell</Text> Trades Without Using Your Own Capital
            </Text>
            <Text className="text-sm text-muted dark:text-muted-dark mt-3 leading-6">
              Automatically mark up existing offers and earn the difference — no inventory, no manual profit splitting.
            </Text>

            <Image
              source={heroImg}
              style={{
                width: SCREEN_WIDTH - 40,
                height: ((SCREEN_WIDTH - 40) * 2) / 3,
                marginTop: 20,
                borderRadius: 20,
              }}
              resizeMode="contain"
            />

            <TouchableOpacity
              onPress={() => router.push("/(tabs)/marketplace")}
              className="mt-5 bg-brand py-4 rounded-xl items-center"
              activeOpacity={0.85}
            >
              <Text className="text-white text-base font-semibold">Start Trading</Text>
            </TouchableOpacity>
            <TouchableOpacity
              onPress={() => router.push("/(auth)/signup")}
              className="mt-2.5 bg-foreground/[0.06] dark:bg-white/[0.08] py-4 rounded-xl items-center border border-foreground/15 dark:border-white/15"
              activeOpacity={0.85}
            >
              <Text className="text-foreground dark:text-foreground-dark text-base font-semibold">Create an account</Text>
            </TouchableOpacity>

            <View className="flex-row flex-wrap gap-4 mt-4 justify-center">
              <TrustBadge icon={WalletIcon} label="No capital required" />
              <TrustBadge icon={ShieldCheck} label="Escrow protected" />
              <TrustBadge icon={Zap} label="Instant payouts" />
            </View>
          </View>

          {/* How it works */}
          <View className="mt-10 px-5">
            <Text className="text-xs font-semibold text-brand uppercase tracking-wider mb-2">How it works</Text>
            <Text className="text-2xl font-medium text-foreground dark:text-foreground-dark mb-4">
              Three steps to your first trade
            </Text>
            <View className="gap-3">
              <StepCard n={1} icon={Users} title="Pick an offer" body="Browse buy and sell offers from verified traders across South Africa. Filter by currency, payment method, and price." />
              <StepCard n={2} icon={ShieldCheck} title="Trade safely with escrow" body="Crypto is locked in custodial escrow the moment you start a trade. Send or receive fiat off-platform — funds release once both parties confirm." />
              <StepCard n={3} icon={Zap} title="Earn from every trade" body="Resellers can relist any active offer with a markup and earn the difference. No capital required — the platform handles the splits." />
            </View>
          </View>

          {/* Why QicTrader */}
          <View className="mt-10 px-5">
            <Text className="text-xs font-semibold text-brand uppercase tracking-wider mb-2">Why QicTrader</Text>
            <Text className="text-2xl font-medium text-foreground dark:text-foreground-dark mb-3">
              Built for South African crypto traders
            </Text>
            <View className="bg-surface dark:bg-card-dark rounded-2xl p-4 border border-border dark:border-border-dark">
              <FeatureRow icon={ShieldCheck} title="Custodial escrow" body="Your crypto is held by QicTrader until both sides confirm — no off-platform trust required." />
              <View className="h-px bg-border dark:bg-border-dark" />
              <FeatureRow icon={CheckCircle2} title="Tiered KYC" body="Start trading at low limits with just an email. Unlock higher limits with ID + selfie + proof of address." />
              <View className="h-px bg-border dark:bg-border-dark" />
              <FeatureRow icon={Repeat} title="Resell offers" body="List a vendor's offer at your own markup. Earn the spread on every trade you facilitate." />
              <View className="h-px bg-border dark:bg-border-dark" />
              <FeatureRow icon={TrendingUp} title="ZAR and USD pricing" body="Live FX, fee transparency, and atomic quotes so the price you see is the price you get." />
            </View>
          </View>

          {/* Bottom CTA */}
          <View className="mt-10 mx-5 rounded-3xl overflow-hidden">
            <View className="bg-brand p-6">
              <Text className="text-white text-2xl font-medium leading-snug">
                Ready to start trading?
              </Text>
              <Text className="text-white/90 text-sm mt-2">
                Browse the marketplace in seconds. Sign up only when you're ready to trade.
              </Text>
              <View className="flex-row gap-3 mt-5">
                <TouchableOpacity
                  onPress={() => router.push("/(tabs)/marketplace")}
                  className="flex-1 bg-white py-3.5 rounded-xl items-center"
                  activeOpacity={0.85}
                >
                  <Text className="text-brand font-semibold">Browse market</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  onPress={() => router.push("/(auth)/signup")}
                  className="flex-1 bg-white/15 border border-white/30 py-3.5 rounded-xl items-center"
                  activeOpacity={0.85}
                >
                  <Text className="text-white font-semibold">Sign up</Text>
                </TouchableOpacity>
              </View>
            </View>
          </View>
        </ScrollView>
      </SafeAreaView>
    )
  }

  // ── Authenticated dashboard
  return <AuthedHome />
}

function AuthedHome() {
  const router = useRouter()
  const { user } = useAuthStore()
  const { data: wallets } = useWallets()
  const { data: trades } = useActiveTrades()

  const firstName = (user?.displayName ?? user?.username ?? user?.email ?? "trader").split(" ")[0]
  const activeTradeCount = trades?.length ?? 0

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["top"]}>
      <ScrollView showsVerticalScrollIndicator={false} contentContainerStyle={{ paddingBottom: 32 }}>
        {/* Header */}
        <View className="flex-row items-center justify-between px-5 pt-2 pb-3">
          <View>
            <Text className="text-xs text-muted dark:text-muted-dark">Welcome back</Text>
            <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mt-0.5">
              Hi, {firstName}
            </Text>
          </View>
          <TouchableOpacity
            onPress={() => router.push("/(tabs)/notifications")}
            className="w-10 h-10 rounded-full bg-surface dark:bg-card-dark items-center justify-center border border-border dark:border-border-dark"
            activeOpacity={0.7}
          >
            <Bell size={18} color="#64748B" />
          </TouchableOpacity>
        </View>

        {/* Stats */}
        <View className="px-5 mt-2">
          <View className="flex-row gap-3">
            <StatTile label="Active trades" value={String(activeTradeCount)} />
            <StatTile label="KYC tier" value={`L${user?.kycTier ?? 0}`} />
          </View>
        </View>

        {/* Quick actions */}
        <View className="mx-5 mt-4 bg-surface dark:bg-card-dark rounded-2xl p-4 border border-border dark:border-border-dark">
          <View className="flex-row">
            <QuickAction icon={Search} label="Buy" onPress={() => router.push("/(tabs)/marketplace")} />
            <QuickAction icon={Plus} label="Sell" onPress={() => router.push("/(app)/create-offer")} />
            <QuickAction icon={WalletIcon} label="Wallet" onPress={() => router.push("/(app)/wallet")} />
            <QuickAction icon={Repeat} label="Resell" onPress={() => router.push("/(app)/reseller-dashboard")} />
          </View>
        </View>

        {/* Wallet summary */}
        <TouchableOpacity
          onPress={() => router.push("/(app)/wallet")}
          activeOpacity={0.85}
          className="mx-5 mt-4 bg-brand p-5 rounded-2xl"
        >
          <View className="flex-row items-center justify-between mb-2">
            <Text className="text-white/80 text-xs">Total balance</Text>
            <ArrowRight size={16} color="rgba(255,255,255,0.8)" />
          </View>
          <Text className="text-white text-2xl font-bold">
            {wallets && wallets.length > 0 ? `${wallets.length} assets` : "Get started"}
          </Text>
          <Text className="text-white/80 text-xs mt-1">Tap to view balances and transactions</Text>
        </TouchableOpacity>

        {/* Active trades */}
        <View className="px-5 mt-6">
          <View className="flex-row items-center justify-between mb-3">
            <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">Active trades</Text>
            <TouchableOpacity onPress={() => router.push("/(tabs)/trades")}>
              <Text className="text-xs text-brand font-medium">See all</Text>
            </TouchableOpacity>
          </View>
          {activeTradeCount === 0 ? (
            <View className="bg-surface dark:bg-card-dark rounded-2xl p-5 border border-border dark:border-border-dark items-center">
              <Text className="text-sm text-muted dark:text-muted-dark text-center">
                No active trades. Head to the marketplace to find an offer.
              </Text>
              <TouchableOpacity
                onPress={() => router.push("/(tabs)/marketplace")}
                className="mt-3 px-4 py-2 rounded-lg bg-brand"
              >
                <Text className="text-white text-xs font-semibold">Browse marketplace</Text>
              </TouchableOpacity>
            </View>
          ) : (
            <View className="gap-2">
              {trades?.slice(0, 3).map((t) => (
                <TouchableOpacity
                  key={t.id}
                  onPress={() => router.push({ pathname: "/(app)/trade/[id]", params: { id: t.id } })}
                  className="bg-surface dark:bg-card-dark rounded-2xl p-4 border border-border dark:border-border-dark flex-row items-center justify-between"
                  activeOpacity={0.85}
                >
                  <View>
                    <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                      {t.cryptoAmount} {t.currency}
                    </Text>
                    <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
                      {t.fiatAmount} {t.fiatCurrency} · {t.status}
                    </Text>
                  </View>
                  <ArrowRight size={16} color="#64748B" />
                </TouchableOpacity>
              ))}
            </View>
          )}
        </View>
      </ScrollView>
    </SafeAreaView>
  )
}
