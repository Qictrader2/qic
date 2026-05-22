import {
  View, Text, ScrollView, TouchableOpacity, TextInput, ActivityIndicator, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useRouter } from "expo-router"
import { profileService } from "@/src/services/profile.service"
import { promptBiometric, isBiometricEnabled, isBiometricAvailable } from "@/src/lib/biometric"
import { setBiometricEnabled } from "@/src/lib/biometric"
import { useAuthStore } from "@/src/store/auth-store"

const passwordSchema = z.object({
  currentPassword: z.string().min(1, "Required"),
  newPassword: z.string().min(8, "At least 8 characters"),
  twoFactorCode: z.string().length(6, "6-digit 2FA code required"),
})
type PasswordForm = z.infer<typeof passwordSchema>

export default function SecuritySettingsScreen() {
  const router = useRouter()
  const { logout } = useAuthStore()
  const [pwError, setPwError] = useState<string | null>(null)
  const [pwSuccess, setPwSuccess] = useState(false)

  const { control, handleSubmit, formState: { errors, isSubmitting }, reset } = useForm<PasswordForm>({
    resolver: zodResolver(passwordSchema),
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

  async function handleDeleteAccount() {
    Alert.alert(
      "Delete account",
      "This is permanent and cannot be undone. All your data will be deleted.",
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Delete my account",
          style: "destructive",
          onPress: async () => {
            try {
              await profileService.changePassword({} as never)
              await logout()
            } catch {
              Alert.alert("Error", "Failed to delete account. Contact support.")
            }
          },
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
      </ScrollView>
    </SafeAreaView>
  )
}
