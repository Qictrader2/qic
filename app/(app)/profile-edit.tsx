import {
  View, Text, ScrollView, TextInput, TouchableOpacity, ActivityIndicator, Image, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { profileService } from "@/src/services/profile.service"
import { useAuthStore } from "@/src/store/auth-store"
import { useRouter } from "expo-router"

const schema = z.object({
  displayName: z.string().min(2).max(60),
  username: z.string().min(3).max(32).regex(/^[a-zA-Z0-9_]+$/, "Only letters, numbers, underscores"),
  phone: z.string().optional(),
  bio: z.string().max(300).optional(),
})
type Form = z.infer<typeof schema>

export default function ProfileEditScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const { setUser } = useAuthStore()
  const { data: profile, isLoading } = useQuery({
    queryKey: ["my-profile"],
    queryFn: () => profileService.getProfile(),
  })
  const { mutateAsync: updateProfile } = useMutation({
    mutationFn: (req: Parameters<typeof profileService.updateProfile>[0]) =>
      profileService.updateProfile(req),
    onSuccess: (updated) => {
      qc.setQueryData(["my-profile"], updated)
      setUser({
        uid: updated.uid,
        email: updated.email,
        username: updated.username,
        displayName: updated.displayName,
        emailVerified: updated.emailVerified,
        role: null,
        kycTier: updated.kycTier,
      })
    },
  })
  const [serverError, setServerError] = useState<string | null>(null)

  const { control, handleSubmit, formState: { errors, isSubmitting } } = useForm<Form>({
    resolver: zodResolver(schema),
    values: {
      displayName: profile?.displayName ?? "",
      username: profile?.username ?? "",
      phone: profile?.phone ?? "",
      bio: profile?.bio ?? "",
    },
  })

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await updateProfile(data)
      router.back()
    } catch {
      setServerError("Failed to update profile. Please try again.")
    }
  }

  if (isLoading) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center">
        <ActivityIndicator color="#00A3F6" />
      </SafeAreaView>
    )
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        {serverError ? (
          <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
            <Text className="text-sm text-error">{serverError}</Text>
          </View>
        ) : null}

        {[
          { name: "displayName" as const, label: "Display name", placeholder: "Your name" },
          { name: "username" as const, label: "Username", placeholder: "yourhandle" },
          { name: "phone" as const, label: "Phone (optional)", placeholder: "+27 xxx xxx xxxx" },
        ].map(({ name, label, placeholder }) => (
          <View key={name} className="mb-4">
            <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">{label}</Text>
            <Controller
              control={control}
              name={name}
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder={placeholder}
                  placeholderTextColor="#94A3B8"
                  autoCapitalize="none"
                />
              )}
            />
            {errors[name]?.message ? (
              <Text className="mt-1 text-xs text-error">{String(errors[name]?.message)}</Text>
            ) : null}
          </View>
        ))}

        <View className="mb-6">
          <Text className="mb-1.5 text-sm font-medium text-foreground dark:text-foreground-dark">Bio (optional)</Text>
          <Controller
            control={control}
            name="bio"
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark h-24"
                onBlur={onBlur}
                onChangeText={onChange}
                value={value ?? ""}
                placeholder="Tell traders about yourself…"
                placeholderTextColor="#94A3B8"
                multiline
                textAlignVertical="top"
              />
            )}
          />
        </View>

        <TouchableOpacity
          onPress={handleSubmit(onSubmit)}
          disabled={isSubmitting}
          className="rounded-lg bg-brand py-4 items-center mb-8"
          activeOpacity={0.8}
        >
          {isSubmitting ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text className="text-base font-semibold text-white">Save changes</Text>
          )}
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  )
}
