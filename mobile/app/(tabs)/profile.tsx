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
import { ChevronRight, ShieldCheck, CheckCircle2, AlertCircle } from "lucide-react-native"
import { useAuthStore } from "@/src/store/auth-store"
import { isBiometricEnabled, setBiometricEnabled, isBiometricAvailable, getBiometricType } from "@/src/lib/biometric"
import { SignInPrompt } from "@/src/components/common/SignInPrompt"
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
        <View className="flex-row items-center gap-1.5">
          {value ? (
            <Text className="text-sm text-muted dark:text-muted-dark">{value}</Text>
          ) : null}
          {onPress ? <ChevronRight size={14} color="#94A3B8" /> : null}
        </View>
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
  const { user, logout, isAuthenticated } = useAuthStore()
  const router = useRouter()
  const [biometricOn, setBiometricOn] = useState(false)
  const [biometricType, setBiometricType] = useState("biometric")

  useEffect(() => {
    if (!isAuthenticated) return
    isBiometricEnabled().then(setBiometricOn)
    getBiometricType().then((t) => {
      if (t === "facial") setBiometricType("Face ID")
      else if (t === "fingerprint") setBiometricType("Touch ID")
    })
  }, [isAuthenticated])

  if (!isAuthenticated) {
    return (
      <SignInPrompt
        title="Your profile"
        message="Sign in to access your KYC, payment methods, security settings, and account preferences."
      />
    )
  }

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
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["top"]}>
      <ScrollView className="flex-1 px-5">
        {/* Avatar */}
        <View className="items-center pt-6 pb-4">
          <View className="h-20 w-20 rounded-full bg-brand items-center justify-center mb-3">
            <Text className="text-2xl font-bold text-white">{initials}</Text>
          </View>
          <Text className="text-lg font-semibold text-foreground dark:text-foreground-dark">
            {displayName}
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark">{user?.email}</Text>
          <View className="flex-row items-center gap-2 mt-3">
            <View className="px-2.5 py-1 rounded-full bg-brand-bg flex-row items-center gap-1">
              <ShieldCheck size={11} color="#00A3F6" />
              <Text className="text-xs font-semibold text-brand">KYC L{user?.kycTier ?? 0}</Text>
            </View>
            {user?.emailVerified ? (
              <View className="px-2.5 py-1 rounded-full bg-success-bg flex-row items-center gap-1">
                <CheckCircle2 size={11} color="#10B981" />
                <Text className="text-xs font-semibold text-success">Email verified</Text>
              </View>
            ) : (
              <View className="px-2.5 py-1 rounded-full bg-warning-bg flex-row items-center gap-1">
                <AlertCircle size={11} color="#F59E0B" />
                <Text className="text-xs font-semibold text-warning">Verify email</Text>
              </View>
            )}
          </View>
        </View>

        {/* Account */}
        <SectionHeader title="Account" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark px-4">
          <SettingsRow label="Edit profile" onPress={() => router.push("/(app)/profile-edit")} />
          <SettingsRow label="Payment methods" onPress={() => router.push("/(app)/payment-methods")} />
          <SettingsRow
            label="Identity verification"
            value={`Tier ${user?.kycTier ?? 0}`}
            onPress={() => router.push("/(app)/kyc")}
          />
          <SettingsRow label="My offers" onPress={() => router.push("/(app)/my-offers")} />
        </View>

        {/* Security */}
        <SectionHeader title="Security" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark px-4">
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
            label="Active sessions & devices"
            onPress={() => router.push("/(app)/security-settings")}
          />
        </View>

        {/* Earn */}
        <SectionHeader title="Earn" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark px-4">
          <SettingsRow label="Affiliate program" onPress={() => router.push("/(app)/affiliate")} />
          <SettingsRow label="Reseller dashboard" onPress={() => router.push("/(app)/reseller-dashboard")} />
        </View>

        {/* More */}
        <SectionHeader title="More" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark px-4">
          <SettingsRow label="Fiat balances" onPress={() => router.push("/(app)/fiat-balance")} />
          <SettingsRow label="Transaction history" onPress={() => router.push("/(app)/transactions")} />
          <SettingsRow label="Support" onPress={() => router.push("/(app)/support")} />
          <SettingsRow label="Preferences" onPress={() => router.push("/(app)/preferences")} />
        </View>

        {/* Logout */}
        <View className="mt-6 mb-10">
          <TouchableOpacity
            onPress={handleLogout}
            className="rounded-xl border border-error/40 bg-error-bg py-3.5 items-center"
            activeOpacity={0.85}
          >
            <Text className="text-sm font-semibold text-error">Log out</Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </SafeAreaView>
  )
}
