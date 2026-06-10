import { View, Text, ScrollView, Switch, ActivityIndicator, TouchableOpacity } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation } from "@tanstack/react-query"
import { apiClient } from "@/src/lib/api/client"
import { useState } from "react"
import { useRouter } from "expo-router"
import {
  ChevronLeft,
  Bell,
  Banknote,
  Store,
  Scale,
  Mail,
  ArrowLeftRight,
} from "lucide-react-native"

interface NotifPrefs {
  tradeUpdates: boolean
  paymentReceived: boolean
  offerMatched: boolean
  disputeUpdates: boolean
  marketingEmails: boolean
}

const PREFS: { key: keyof NotifPrefs; label: string; desc: string; Icon: typeof Bell; color: string }[] = [
  {
    key: "tradeUpdates",
    label: "Trade updates",
    desc: "Status changes on your active trades",
    Icon: ArrowLeftRight,
    color: "#00A3F6",
  },
  {
    key: "paymentReceived",
    label: "Payment received",
    desc: "When a buyer marks payment as sent",
    Icon: Banknote,
    color: "#10B981",
  },
  {
    key: "offerMatched",
    label: "Offer matched",
    desc: "When someone initiates a trade on your offer",
    Icon: Store,
    color: "#F59E0B",
  },
  {
    key: "disputeUpdates",
    label: "Dispute updates",
    desc: "Updates on open disputes",
    Icon: Scale,
    color: "#EF4444",
  },
  {
    key: "marketingEmails",
    label: "Marketing emails",
    desc: "News, promotions, and trading tips",
    Icon: Mail,
    color: "#8B5CF6",
  },
]

export default function NotificationsSettingsScreen() {
  const router = useRouter()
  const { data: prefs, isLoading } = useQuery<NotifPrefs>({
    queryKey: ["notif-prefs"],
    queryFn: () => apiClient.get("/api/v1/notifications/preferences"),
  })
  const { mutate: updatePref } = useMutation({
    mutationFn: (updates: Partial<NotifPrefs>) =>
      apiClient.patch("/api/v1/notifications/preferences", updates),
  })
  const [local, setLocal] = useState<Partial<NotifPrefs>>({})

  function toggle(key: keyof NotifPrefs, val: boolean) {
    setLocal((prev) => ({ ...prev, [key]: val }))
    updatePref({ [key]: val })
  }

  const effective = { ...(prefs ?? {}), ...local } as NotifPrefs

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
        <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
          Notifications
        </Text>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <ScrollView className="flex-1 px-5" showsVerticalScrollIndicator={false}>
          <View className="rounded-2xl bg-brand/10 border border-brand/20 p-4 mb-5 flex-row items-center gap-3">
            <View className="w-10 h-10 rounded-full bg-brand/20 items-center justify-center">
              <Bell size={18} color="#00A3F6" />
            </View>
            <View className="flex-1">
              <Text className="text-sm font-semibold text-brand">Stay informed</Text>
              <Text className="text-xs text-brand/80 mt-0.5">
                Get push and email updates for important account events
              </Text>
            </View>
          </View>

          <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark overflow-hidden">
            {PREFS.map(({ key, label, desc, Icon, color }, i) => (
              <View
                key={key}
                className={`flex-row items-center justify-between px-4 py-4 ${
                  i < PREFS.length - 1
                    ? "border-b border-border/40 dark:border-border-dark/40"
                    : ""
                }`}
              >
                <View className="flex-row items-start gap-3 flex-1 mr-4">
                  <View
                    className="w-9 h-9 rounded-full items-center justify-center mt-0.5"
                    style={{ backgroundColor: color + "22" }}
                  >
                    <Icon size={14} color={color} />
                  </View>
                  <View className="flex-1">
                    <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                      {label}
                    </Text>
                    <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">{desc}</Text>
                  </View>
                </View>
                <Switch
                  value={effective[key] ?? false}
                  onValueChange={(val) => toggle(key, val)}
                  trackColor={{ false: "#E2E8F0", true: "#00A3F6" }}
                  thumbColor="#fff"
                />
              </View>
            ))}
          </View>

          <Text className="text-xs text-muted dark:text-muted-dark mt-4 text-center max-w-[300px] mx-auto">
            You can also manage push notifications in your device settings.
          </Text>

          <View className="h-12" />
        </ScrollView>
      )}
    </SafeAreaView>
  )
}
