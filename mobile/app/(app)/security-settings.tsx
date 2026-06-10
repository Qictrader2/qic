import {
  View, Text, ScrollView, TouchableOpacity, TextInput, ActivityIndicator, Alert,
  FlatList,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useEffect } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useRouter } from "expo-router"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { profileService } from "@/src/services/profile.service"
import { promptBiometric, isBiometricEnabled, isBiometricAvailable } from "@/src/lib/biometric"
import { setBiometricEnabled } from "@/src/lib/biometric"
import { useAuthStore } from "@/src/store/auth-store"
import { apiClient } from "@/src/lib/api/client"

const passwordSchema = z.object({
  currentPassword: z.string().min(1, "Required"),
  newPassword: z.string().min(8, "At least 8 characters"),
  twoFactorCode: z.string().length(6, "6-digit 2FA code required"),
})
type PasswordForm = z.infer<typeof passwordSchema>

interface ActiveSession {
  sessionId: string
  device: string
  ip: string
  location: string
  lastActiveAt: string
  isCurrent: boolean
}

function SessionRow({ session, onRevoke }: { session: ActiveSession; onRevoke: () => void }) {
  return (
    <View className="py-3 border-b border-border/30 dark:border-border-dark/30 last:border-0">
      <View className="flex-row items-start justify-between">
        <View className="flex-1 mr-3">
          <View className="flex-row items-center gap-2">
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
              {session.device}
            </Text>
            {session.isCurrent ? (
              <View className="px-1.5 py-0.5 rounded-full bg-success-bg">
                <Text className="text-xs text-success font-medium">This device</Text>
              </View>
            ) : null}
          </View>
          <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">
            {session.ip} · {session.location}
          </Text>
          <Text className="text-xs text-muted dark:text-muted-dark">
            Last active: {new Date(session.lastActiveAt).toLocaleDateString(undefined, {
              month: "short", day: "numeric", hour: "2-digit", minute: "2-digit"
            })}
          </Text>
        </View>
        {!session.isCurrent ? (
          <TouchableOpacity
            onPress={onRevoke}
            className="px-3 py-1.5 rounded-lg border border-error bg-error-bg"
            activeOpacity={0.8}
          >
            <Text className="text-xs font-medium text-error">Revoke</Text>
          </TouchableOpacity>
        ) : null}
      </View>
    </View>
  )
}

export default function SecuritySettingsScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const { logout } = useAuthStore()
  const [pwError, setPwError] = useState<string | null>(null)
  const [pwSuccess, setPwSuccess] = useState(false)

  const { control, handleSubmit, formState: { errors, isSubmitting }, reset } = useForm<PasswordForm>({
    resolver: zodResolver(passwordSchema),
  })

  const { data: sessions, isLoading: sessionsLoading } = useQuery({
    queryKey: ["active-sessions"],
    queryFn: () => apiClient.get<ActiveSession[]>("/api/v1/auth/sessions"),
    staleTime: 30_000,
  })

  const { mutate: revokeSession, isPending: revoking } = useMutation({
    mutationFn: (sessionId: string) =>
      apiClient.delete<void>(`/api/v1/auth/sessions/${sessionId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["active-sessions"] }),
    onError: () => Alert.alert("Error", "Failed to revoke session."),
  })

  const { mutate: revokeAllSessions, isPending: revokingAll } = useMutation({
    mutationFn: () => apiClient.delete<void>("/api/v1/auth/sessions/all"),
    onSuccess: () => logout(),
    onError: () => Alert.alert("Error", "Failed to revoke all sessions."),
  })

  async function onChangePassword(data: PasswordForm) {
    setPwError(null)
    const biometricEnabled = await isBiometricEnabled()
    const biometricAvailable = await isBiometricAvailable()
    if (biometricEnabled && biometricAvailable) {
      const passed = await promptBiometric("Confirm password change")
      if (!passed) {
        setPwError("Biometric verification failed.")
        return
      }
    }
    try {
      await profileService.changePassword(data)
      setPwSuccess(true)
      reset()
    } catch {
      setPwError("Failed to change password. Check your current password and 2FA code.")
    }
  }

  function handleRevokeSession(session: ActiveSession) {
    Alert.alert(
      "Revoke session",
      `Revoke access for "${session.device}" (${session.ip})?`,
      [
        { text: "Cancel", style: "cancel" },
        { text: "Revoke", style: "destructive", onPress: () => revokeSession(session.sessionId) },
      ]
    )
  }

  function handleRevokeAll() {
    Alert.alert(
      "Sign out everywhere",
      "This will sign you out of all devices including this one.",
      [
        { text: "Cancel", style: "cancel" },
        { text: "Sign out all", style: "destructive", onPress: () => revokeAllSessions() },
      ]
    )
  }

  function handleDeleteAccount() {
    Alert.alert(
      "Delete account",
      "Account deletion is processed manually by our support team to ensure all funds and open trades are settled first. Tap continue to open a support request.",
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Continue",
          style: "destructive",
          onPress: () => router.push("/(app)/support" as never),
        },
      ]
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-6">Security</Text>

        {/* Change password */}
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-6">
          <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark mb-4">Change password</Text>

          {pwSuccess ? (
            <View className="rounded-lg bg-success-bg px-4 py-3 mb-3">
              <Text className="text-sm text-success">Password changed successfully.</Text>
            </View>
          ) : null}
          {pwError ? (
            <View className="rounded-lg bg-error-bg px-4 py-3 mb-3">
              <Text className="text-sm text-error">{pwError}</Text>
            </View>
          ) : null}

          {[
            { name: "currentPassword" as const, label: "Current password", secure: true },
            { name: "newPassword" as const, label: "New password", secure: true },
            { name: "twoFactorCode" as const, label: "2FA code", secure: false },
          ].map(({ name, label, secure }) => (
            <View key={name} className="mb-3">
              <Text className="mb-1 text-xs font-medium text-muted dark:text-muted-dark">{label}</Text>
              <Controller
                control={control}
                name={name}
                render={({ field: { onChange, onBlur, value } }) => (
                  <TextInput
                    className="rounded-lg border border-border dark:border-border-dark bg-background dark:bg-background-dark px-3 py-2.5 text-sm text-foreground dark:text-foreground-dark"
                    onBlur={onBlur}
                    onChangeText={onChange}
                    value={value}
                    secureTextEntry={secure}
                    keyboardType={name === "twoFactorCode" ? "number-pad" : "default"}
                    maxLength={name === "twoFactorCode" ? 6 : undefined}
                  />
                )}
              />
              {errors[name]?.message ? (
                <Text className="mt-0.5 text-xs text-error">{String(errors[name]?.message)}</Text>
              ) : null}
            </View>
          ))}

          <TouchableOpacity
            onPress={handleSubmit(onChangePassword)}
            disabled={isSubmitting}
            className="rounded-lg bg-brand py-3 items-center mt-2"
            activeOpacity={0.8}
          >
            {isSubmitting ? <ActivityIndicator color="#fff" /> : (
              <Text className="text-sm font-semibold text-white">Change password</Text>
            )}
          </TouchableOpacity>
        </View>

        {/* Active sessions */}
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-6">
          <View className="flex-row items-center justify-between mb-3">
            <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">Active sessions</Text>
            <TouchableOpacity
              onPress={handleRevokeAll}
              disabled={revokingAll}
              activeOpacity={0.8}
            >
              <Text className="text-xs text-error font-medium">Sign out all</Text>
            </TouchableOpacity>
          </View>

          {sessionsLoading ? (
            <ActivityIndicator color="#00A3F6" />
          ) : !sessions?.length ? (
            <Text className="text-sm text-muted dark:text-muted-dark">No sessions found.</Text>
          ) : (
            sessions.map((session) => (
              <SessionRow
                key={session.sessionId}
                session={session}
                onRevoke={() => handleRevokeSession(session)}
              />
            ))
          )}
        </View>

        {/* Danger zone */}
        <View className="rounded-xl bg-error-bg border border-error/30 p-4">
          <Text className="text-sm font-semibold text-error mb-2">Danger zone</Text>
          <Text className="text-xs text-muted dark:text-muted-dark mb-4">
            Deleting your account is permanent and cannot be undone. All wallets, trades, and data will be removed.
          </Text>
          <TouchableOpacity
            onPress={handleDeleteAccount}
            className="rounded-lg border border-error py-2.5 items-center"
            activeOpacity={0.8}
          >
            <Text className="text-sm font-medium text-error">Delete my account</Text>
          </TouchableOpacity>
        </View>

        <View className="h-8" />
      </ScrollView>
    </SafeAreaView>
  )
}
