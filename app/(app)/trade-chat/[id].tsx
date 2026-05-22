import {
  View,
  Text,
  FlatList,
  TextInput,
  TouchableOpacity,
  KeyboardAvoidingView,
  Platform,
  ActivityIndicator,
} from "react-native"
import { useLocalSearchParams } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useRef, useEffect } from "react"
import { useTradeMessages, useSendMessage } from "@/src/hooks/api/use-trade"
import { useAuthStore } from "@/src/store/auth-store"
import { subscribeToTrade } from "@/src/lib/socket"
import { useQueryClient } from "@tanstack/react-query"
import type { TradeMessage } from "@/src/services/trade.service"

function MessageBubble({ message, isOwn }: { message: TradeMessage; isOwn: boolean }) {
  return (
    <View className={`mb-2 max-w-xs ${isOwn ? "self-end" : "self-start"}`}>
      {!isOwn ? (
        <Text className="text-xs text-muted dark:text-muted-dark mb-0.5 ml-1">
          {message.senderUsername}
        </Text>
      ) : null}
      <View
        className={`rounded-2xl px-4 py-2.5 ${
          isOwn
            ? "bg-brand rounded-tr-sm"
            : "bg-surface dark:bg-surface-dark border border-border dark:border-border-dark rounded-tl-sm"
        }`}
      >
        <Text className={`text-sm ${isOwn ? "text-white" : "text-foreground dark:text-foreground-dark"}`}>
          {message.content}
        </Text>
      </View>
      <Text className={`text-xs text-muted dark:text-muted-dark mt-0.5 ${isOwn ? "text-right mr-1" : "ml-1"}`}>
        {new Date(message.createdAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
      </Text>
    </View>
  )
}

export default function TradeChatScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const { user } = useAuthStore()
  const qc = useQueryClient()
  const { data: messages, isLoading } = useTradeMessages(id ?? "")
  const { mutateAsync: sendMessage, isPending: sending } = useSendMessage()
  const [input, setInput] = useState("")
  const flatRef = useRef<FlatList>(null)

  // Realtime subscription
  useEffect(() => {
    if (!id) return
    const unsubscribe = subscribeToTrade(
      id,
      () => qc.invalidateQueries({ queryKey: ["trade", id] }),
      () => qc.invalidateQueries({ queryKey: ["trade-messages", id] })
    )
    return unsubscribe
  }, [id])

  useEffect(() => {
    if (messages?.length) {
      flatRef.current?.scrollToEnd({ animated: true })
    }
  }, [messages?.length])

  async function handleSend() {
    const text = input.trim()
    if (!text || sending) return
    setInput("")
    try {
      await sendMessage({ tradeId: id ?? "", content: text })
    } catch {
      setInput(text)
    }
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
      keyboardVerticalOffset={90}
    >
      <SafeAreaView className="flex-1" edges={["bottom"]}>
        {isLoading ? (
          <View className="flex-1 items-center justify-center">
            <ActivityIndicator color="#00A3F6" />
          </View>
        ) : (
          <FlatList
            ref={flatRef}
            data={messages ?? []}
            keyExtractor={(item) => item.id}
            renderItem={({ item }) => (
              <MessageBubble
                message={item}
                isOwn={item.senderId === user?.uid}
              />
            )}
            contentContainerStyle={{ padding: 16, flexGrow: 1 }}
            ListEmptyComponent={
              <View className="flex-1 items-center justify-center py-20">
                <Text className="text-sm text-muted dark:text-muted-dark">No messages yet</Text>
              </View>
            }
          />
        )}

        {/* Input bar */}
        <View className="flex-row items-end px-4 py-3 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark gap-2">
          <TextInput
            className="flex-1 rounded-2xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4 py-2.5 text-sm text-foreground dark:text-foreground-dark max-h-28"
            value={input}
            onChangeText={setInput}
            placeholder="Type a message…"
            placeholderTextColor="#94A3B8"
            multiline
            returnKeyType="default"
          />
          <TouchableOpacity
            onPress={handleSend}
            disabled={!input.trim() || sending}
            className={`h-10 w-10 rounded-full items-center justify-center ${
              input.trim() ? "bg-brand" : "bg-muted/30"
            }`}
            activeOpacity={0.8}
          >
            {sending ? (
              <ActivityIndicator color="#fff" size="small" />
            ) : (
              <Text className="text-white text-lg">↑</Text>
            )}
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}
