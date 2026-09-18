import { View, Text, TouchableOpacity, ActivityIndicator, Alert } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useLocalSearchParams, useRouter } from "expo-router"
import { useState, useRef, useEffect } from "react"
import { WebView } from "react-native-webview"
import { ChevronLeft } from "lucide-react-native"
import { kycService } from "@/src/services/kyc.service"
import { useQueryClient } from "@tanstack/react-query"

export default function KycWebViewScreen() {
  const { provider } = useLocalSearchParams<{ provider: "didit" | "sumsub" }>()
  const router = useRouter()
  const qc = useQueryClient()
  const [url, setUrl] = useState<string | null>(null)
  const [, setSessionId] = useState<string | null>(null)
  const [, setLoading] = useState(true)
  const [starting, setStarting] = useState(true)
  const webViewRef = useRef<WebView>(null)

  useEffect(() => {
    let cancelled = false
    async function startSession() {
      try {
        const res =
          provider === "sumsub"
            ? await kycService.startSumsubSession()
            : await kycService.startDiditSession()
        if (cancelled) return
        setUrl(res.url)
        setSessionId(res.sessionId)
      } catch {
        if (cancelled) return
        Alert.alert("Error", "Failed to start verification. Please try again.", [
          { text: "OK", onPress: () => router.back() },
        ])
      } finally {
        if (!cancelled) setStarting(false)
      }
    }
    startSession()
    return () => {
      cancelled = true
    }
  }, [provider])

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
      <View className="flex-row items-center justify-between px-5 py-3 border-b border-border dark:border-border-dark">
        <TouchableOpacity
          onPress={() => router.back()}
          className="w-10 h-10 items-center justify-center -ml-2"
          activeOpacity={0.7}
        >
          <ChevronLeft size={24} color="#64748B" />
        </TouchableOpacity>
        <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
          Identity verification
        </Text>
        <View className="w-10" />
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
