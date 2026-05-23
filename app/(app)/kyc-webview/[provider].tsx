import { View, Text, TouchableOpacity, ActivityIndicator, Alert } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState, useRef } from "react"
import { WebView } from "react-native-webview"
import { kycService } from "@/src/services/kyc.service"
import { useQueryClient } from "@tanstack/react-query"

export default function KycWebViewScreen() {
  const { provider } = useLocalSearchParams<{ provider: "didit" | "sumsub" }>()
  const router = useRouter()
  const qc = useQueryClient()
  const [url, setUrl] = useState<string | null>(null)
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [starting, setStarting] = useState(false)
  const webViewRef = useRef<WebView>(null)

  async function startSession() {
    setStarting(true)
    try {
      const res =
        provider === "sumsub"
          ? await kycService.startSumsubSession()
          : await kycService.startDiditSession()
      setUrl(res.url)
      setSessionId(res.sessionId)
    } catch {
      Alert.alert("Error", "Failed to start verification. Please try again.", [
        { text: "OK", onPress: () => router.back() },
      ])
    } finally {
      setStarting(false)
    }
  }

  // Start session immediately on mount
  useState(() => { startSession() })

  function handleNavigationChange(navState: { url: string }) {
    // Detect completion/failure redirects from the provider
    if (
      navState.url.includes("kyc-complete") ||
      navState.url.includes("verification-complete") ||
      navState.url.includes("success")
    ) {
      qc.invalidateQueries({ queryKey: ["kyc-status"] })
      Alert.alert(
        "Verification submitted",
        "We'll review your documents and notify you within 1–2 business days.",
        [{ text: "OK", onPress: () => router.replace("/(app)/kyc") }]
      )
    }
    if (navState.url.includes("kyc-failed") || navState.url.includes("error")) {
      Alert.alert("Verification failed", "Please try again or contact support.", [
        { text: "OK", onPress: () => router.back() },
      ])
    }
  }

  if (starting || !url) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center">
        <ActivityIndicator color="#00A3F6" size="large" />
        <Text className="mt-4 text-sm text-muted dark:text-muted-dark">
          Starting verification…
        </Text>
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <View className="flex-row items-center justify-between px-4 py-3 border-b border-border dark:border-border-dark">
        <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
          Identity Verification
        </Text>
        <TouchableOpacity onPress={() => router.back()} className="px-3 py-1.5">
          <Text className="text-sm text-muted dark:text-muted-dark">Cancel</Text>
        </TouchableOpacity>
      </View>
      <WebView
        ref={webViewRef}
        source={{ uri: url }}
        onLoadEnd={() => setLoading(false)}
        onNavigationStateChange={handleNavigationChange}
        renderLoading={() => (
          <View className="absolute inset-0 items-center justify-center bg-background dark:bg-background-dark">
            <ActivityIndicator color="#00A3F6" />
          </View>
        )}
        startInLoadingState
        allowsInlineMediaPlayback
        mediaPlaybackRequiresUserAction={false}
        style={{ flex: 1 }}
      />
    </SafeAreaView>
  )
}
