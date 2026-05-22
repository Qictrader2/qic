import { View, Text, FlatList, TouchableOpacity, ActivityIndicator } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { apiClient } from "@/src/lib/api/client"

interface Notification {
  id: string
  type: string
  title: string
  body: string
  read: boolean
  data: Record<string, unknown> | null
  createdAt: string
}

function useNotifications() {
  return useQuery({
    queryKey: ["notifications"],
    queryFn: () => apiClient.get<Notification[]>("/api/v1/notifications"),
    staleTime: 30_000,
  })
}

function useMarkAllRead() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: () => apiClient.post<{ success: boolean }>("/api/v1/notifications/read-all"),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["notifications"] }),
  })
}

function NotifRow({ notif, onRead }: { notif: Notification; onRead: () => void }) {
  return (
    <TouchableOpacity
      onPress={onRead}
      className={`px-4 py-3.5 border-b border-border/50 dark:border-border-dark/50 ${
        !notif.read ? "bg-brand-bg/40 dark:bg-brand/10" : ""
      }`}
      activeOpacity={0.7}
    >
      <View className="flex-row items-start gap-3">
        {!notif.read ? (
          <View className="h-2 w-2 rounded-full bg-brand mt-1.5" />
        ) : (
          <View className="h-2 w-2 mt-1.5" />
        )}
        <View className="flex-1">
          <Text className="text-sm font-medium text-foreground dark:text-foreground-dark mb-0.5">
            {notif.title}
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark">{notif.body}</Text>
          <Text className="text-xs text-muted dark:text-muted-dark mt-1">
            {new Date(notif.createdAt).toLocaleDateString()}
          </Text>
        </View>
      </View>
    </TouchableOpacity>
  )
}

export default function NotificationsScreen() {
  const qc = useQueryClient()
  const { data: notifs, isLoading, error } = useNotifications()
  const { mutateAsync: markAllRead } = useMarkAllRead()

  async function handleMarkRead(id: string) {
    try {
      await apiClient.post(`/api/v1/notifications/${id}/read`)
      qc.invalidateQueries({ queryKey: ["notifications"] })
    } catch {}
  }

  const unread = notifs?.filter((n) => !n.read).length ?? 0

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <View className="flex-row items-center gap-2">
          <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark">
            Notifications
          </Text>
          {unread > 0 ? (
            <View className="h-5 w-5 rounded-full bg-brand items-center justify-center">
              <Text className="text-xs font-bold text-white">{unread}</Text>
            </View>
          ) : null}
        </View>
        {unread > 0 ? (
          <TouchableOpacity onPress={() => markAllRead()} activeOpacity={0.7}>
            <Text className="text-xs text-brand">Mark all read</Text>
          </TouchableOpacity>
        ) : null}
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : error ? (
        <View className="mx-4 rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">Failed to load notifications.</Text>
        </View>
      ) : (
        <FlatList
          data={notifs ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <NotifRow notif={item} onRead={() => handleMarkRead(item.id)} />
          )}
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">🔔</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark">
                No notifications
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
