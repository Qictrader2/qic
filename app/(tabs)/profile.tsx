import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  Switch,
  Alert,
} from "react-native"
import { useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useAuthStore } from "@/src/store/auth-store"
import { isBiometricEnabled, setBiometricEnabled, isBiometricAvailable, getBiometricType } from "@/src/lib/biometric"
import { useState, useEffect } from "react"

function SettingsRow({
  label,
  value,
  onPress,
  right,
}: {
  label: string
  value?: string
  onPress?: () => void
  right?: React.ReactNode
}) {
  return (
    <TouchableOpacity
      onPress={onPress}
      disabled={!onPress}
      className="flex-row items-center justify-between py-3.5 border-b border-border/50 dark:border-border-dark/50"
      activeOpacity={onPress ? 0.7 : 1}
    >
      <Text className="text-sm text-foreground dark:text-foreground-dark">{label}</Text>
      {right ?? (
        <Text className="text-sm text-muted dark:text-muted-dark">
          {value}{onPress ? " →" : ""}
        </Text>
      )}
    </TouchableOpacity>
  )
}

function SectionHeader({ title }: { title: string }) {
  return (
    <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mt-6 mb-2">
      {title}
    </Text>
  )
}

export default function ProfileScreen() {
  const { user, logout } = useAuthStore()
  const router = useRouter()
  const [biometricOn, setBiometricOn] = useState(false)
  const [biometricType, setBiometricType] = useState("biometric")

  useEffect(() => {
    isBiometricEnabled().then(setBiometricOn)
    getBiometricType().then((t) => {
      if (t === "facial") setBiometricType("Face ID")
      else if (t === "fingerprint") setBiometricType("Touch ID")
    })
  }, [])

  async function toggleBiometric(val: boolean) {
    const available = await isBiometricAvailable()
    if (!available) {
      Alert.alert("Not available", "Biometric authentication is not set up on this device.")
      return
    }
    await setBiometricEnabled(val)
    setBiometricOn(val)
  }

  async function handleLogout() {
    Alert.alert("Log out", "Are you sure you want to log out?", [
      { text: "Cancel", style: "cancel" },
      {
        text: "Log out",
        style: "destructive",
        onPress: async () => await logout(),
      },
    ])
  }

  const displayName = user?.displayName ?? user?.username ?? user?.email ?? "User"
  const initials = displayName
    .split(" ")
    .slice(0, 2)
    .map((n) => n[0]?.toUpperCase() ?? "")
    .join("")

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4">
        {/* Avatar */}
        <View className="items-center py-6">
          <View className="h-20 w-20 rounded-full bg-brand items-center justify-center mb-3">
            <Text className="text-2xl font-bold text-white">{initials}</Text>
          </View>
          <Text className="text-lg font-semibold text-foreground dark:text-foreground-dark">
            {displayName}
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark">{user?.email}</Text>
          <View className="flex-row items-center gap-2 mt-2">
            <View className="px-2 py-0.5 rounded-full bg-brand-bg">
              <Text className="text-xs font-medium text-brand">KYC Tier {user?.kycTier ?? 0}</Text>
            </View>
            {user?.emailVerified ? (
              <View className="px-2 py-0.5 rounded-full bg-success-bg">
                <Text className="text-xs font-medium text-success">Verified</Text>
              </View>
            ) : (
              <View className="px-2 py-0.5 rounded-full bg-warning-bg">
                <Text className="text-xs font-medium text-warning">Email unverified</Text>
              </View>
            )}
          </View>
        </View>

        {/* Account */}
        <SectionHeader title="Account" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4">
          <SettingsRow label="Edit profile" onPress={() => router.push("/(app)/profile-edit")} />
          <SettingsRow label="Payment methods" onPress={() => router.push("/(app)/payment-methods")} />
          <SettingsRow
            label="Identity verification"
            value={`Tier ${user?.kycTier ?? 0}`}
            onPress={() => router.push("/(app)/kyc")}
          />
        </View>

        {/* Security */}
        <SectionHeader title="Security" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4">
          <SettingsRow label={biometricType} right={
            <Switch
              value={biometricOn}
              onValueChange={toggleBiometric}
              trackColor={{ false: "#E2E8F0", true: "#00A3F6" }}
              thumbColor="#fff"
            />
          } />
          <SettingsRow
            label="Two-factor authentication"
            onPress={() => router.push("/(app)/2fa-setup")}
          />
          <SettingsRow
            label="Security settings"
            onPress={() => router.push("/(app)/security-settings")}
          />
        </View>

        {/* More */}
        <SectionHeader title="More" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4">
          <SettingsRow label="Affiliate program" onPress={() => router.push("/(app)/affiliate")} />
          <SettingsRow label="Support" onPress={() => router.push("/(app)/support")} />
          <SettingsRow label="Preferences" onPress={() => router.push("/(app)/preferences")} />
        </View>

        {/* Logout */}
        <View className="mt-6 mb-10">
          <TouchableOpacity
            onPress={handleLogout}
            className="rounded-lg border border-error bg-error-bg py-3.5 items-center"
            activeOpacity={0.8}
          >
            <Text className="text-sm font-medium text-error">Log out</Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </SafeAreaView>
  )
}
