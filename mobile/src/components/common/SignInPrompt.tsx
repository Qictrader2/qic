import { View, Text, TouchableOpacity, Image } from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { Lock, ArrowRight } from "lucide-react-native"

interface SignInPromptProps {
  /** Headline e.g. "Your trades" */
  title: string
  /** Sub-message explaining what's behind the gate */
  message: string
  /** Where to send users after they sign in (for return-to-route in future) */
  returnTo?: string
}

/**
 * Reusable empty-state for tabs that require authentication.
 * Mirrors the web's "Sign in to view this page" affordance — but never blocks
 * navigation, the user can swipe to another tab freely.
 */
export function SignInPrompt({ title, message }: SignInPromptProps) {
  const router = useRouter()
  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="flex-1 px-6 items-center justify-center">
        <View className="w-20 h-20 rounded-full bg-brand/10 items-center justify-center mb-5">
          <Lock size={28} color="#00A3F6" />
        </View>
        <Text className="text-xl font-semibold text-foreground dark:text-foreground-dark text-center">
          {title}
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark text-center mt-2 max-w-[280px]">
          {message}
        </Text>

        <TouchableOpacity
          onPress={() => router.push("/(auth)/login")}
          className="mt-7 bg-brand px-6 py-3.5 rounded-xl flex-row items-center gap-2"
          activeOpacity={0.85}
        >
          <Text className="text-white text-sm font-semibold">Sign in</Text>
          <ArrowRight size={16} color="#FFFFFF" />
        </TouchableOpacity>
        <TouchableOpacity
          onPress={() => router.push("/(auth)/signup")}
          className="mt-3"
          activeOpacity={0.7}
        >
          <Text className="text-sm text-muted dark:text-muted-dark">
            Don't have an account? <Text className="text-brand font-medium">Sign up</Text>
          </Text>
        </TouchableOpacity>
      </View>
    </SafeAreaView>
  )
}
