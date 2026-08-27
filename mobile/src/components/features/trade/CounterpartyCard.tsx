import { View, Text, TouchableOpacity } from "react-native"
import { ThumbsUp, ShieldCheck } from "lucide-react-native"

interface Counterparty {
  uid: string
  username: string
  kycTier: number
}

/**
 * Mirrors web's ParticipantsCard — counterparty avatar with initials,
 * name + verification badge, KYC tier, and role (buyer/seller).
 */
export function CounterpartyCard({
  counterparty,
  role,
}: {
  counterparty: Counterparty
  role: "buyer" | "seller"
}) {
  const isVerified = counterparty.kycTier >= 2
  const name = counterparty.username || "trader"
  const initials = name
    .split(/[\s_]/)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("")

  // Counterparty role is the opposite of the user's role
  const counterRole = role === "buyer" ? "Seller" : "Buyer"

  return (
    <View className="bg-surface dark:bg-card-dark border border-border dark:border-border-dark rounded-2xl p-4 flex-row items-center gap-3">
      <View className="w-12 h-12 rounded-full bg-brand/20 items-center justify-center">
        <Text className="text-base font-bold text-brand">{initials}</Text>
      </View>
      <View className="flex-1">
        <View className="flex-row items-center gap-1.5 flex-wrap">
          <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
            {name}
          </Text>
          {isVerified ? (
            <View className="bg-success-bg rounded-full px-1.5 py-0.5 flex-row items-center gap-0.5">
              <ShieldCheck size={10} color="#10B981" />
              <Text className="text-[10px] font-semibold text-success">Verified</Text>
            </View>
          ) : null}
        </View>
        <View className="flex-row items-center gap-3 mt-1">
          <Text className="text-xs text-muted dark:text-muted-dark">
            {counterRole} · KYC L{counterparty.kycTier}
          </Text>
        </View>
      </View>
    </View>
  )
}
