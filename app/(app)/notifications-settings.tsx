import { View, Text, ScrollView, Switch, ActivityIndicator } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation } from "@tanstack/react-query"
import { apiClient } from "@/src/lib/api/client"
import { useState } from "react"

interface NotifPrefs {
  tradeUpdates: boolean
  paymentReceived: boolean
  offerMatched: boolean
  disputeUpdates: boolean
  marketingEmails: boolean
}

export default function NotificationsSettingsScreen() {
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

  const PREFS: { key: keyof NotifPrefs; label: string; desc: string }[] = [
    { key: "tradeUpdates", label: "Trade updates", desc: "Status changes on your active trades" },
    { key: "paymentReceived", label: "Payment received", desc: "When a buyer marks payment as sent" },
    { key: "offerMatched", label: "Offer matched", desc: "When someone initiates a trade on your offer" },
    { key: "disputeUpdates", label: "Dispute updates", desc: "Updates on open disputes" },
    { key: "marketingEmails", label: "Marketing emails", desc: "News, promotions, and tips" },
  ]

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">Notification Settings</Text>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <ScrollView className="flex-1 px-4">
          <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4 mb-6">
            {PREFS.map(({ key, label, desc }, i) => (
              <View
                key={key}
                className={`flex-row items-center justify-between py-4 ${
                  i < PREFS.length - 1 ? "border-b border-border/50 dark:border-border-dark/50" : ""
                }`}
              >
                <View className="flex-1 mr-4">
                  <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                    {label}
                  </Text>
                  <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">{desc}</Text>
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
        </ScrollView>
      )}
    </SafeAreaView>
  )
}
