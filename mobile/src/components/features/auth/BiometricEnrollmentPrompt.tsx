import { useState } from "react"
import {
  View,
  Text,
  TouchableOpacity,
  ActivityIndicator,
  Switch,
} from "react-native"
import { useRouter } from "expo-router"
import {
  isBiometricAvailable,
  getBiometricType,
  setBiometricEnabled,
  promptBiometric,
} from "@/src/lib/biometric"

interface Props {
  onSkip: () => void
  onEnable: () => void
}

export function BiometricEnrollmentPrompt({ onSkip, onEnable }: Props) {
  const [loading, setLoading] = useState(false)

  async function handleEnable() {
    setLoading(true)
    try {
      const success = await promptBiometric("Confirm your identity to enable biometric login")
      if (success) {
        await setBiometricEnabled(true)
        onEnable()
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <View className="flex-1 bg-background dark:bg-background-dark justify-center px-6">
      <View className="mb-8 items-center">
        <Text className="text-5xl mb-4">🔒</Text>
        <Text className="text-2xl font-bold text-foreground dark:text-foreground-dark text-center mb-2">
          Enable biometric login
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark text-center leading-relaxed">
          Use Face ID or Touch ID to sign in faster next time — no password needed.
        </Text>
      </View>

      <TouchableOpacity
        onPress={handleEnable}
        disabled={loading}
        className="rounded-lg bg-brand py-4 items-center mb-4"
        activeOpacity={0.8}
      >
        {loading ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text className="text-base font-semibold text-white">Enable biometric login</Text>
        )}
      </TouchableOpacity>

      <TouchableOpacity onPress={onSkip} className="py-4 items-center">
        <Text className="text-sm text-muted dark:text-muted-dark">Not now</Text>
      </TouchableOpacity>
    </View>
  )
}
