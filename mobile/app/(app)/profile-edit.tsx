import {
  View,
  Text,
  ScrollView,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
  Image,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useForm, Controller } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useRouter } from "expo-router"
import {
  ChevronLeft,
  AtSign,
  Phone,
  User,
  FileText,
  Camera,
  AlertTriangle,
  CheckCircle2,
} from "lucide-react-native"
import { profileService } from "@/src/services/profile.service"
import { useAuthStore } from "@/src/store/auth-store"

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
  const [success, setSuccess] = useState(false)

  const {
    control,
    handleSubmit,
    watch,
    formState: { errors, isSubmitting, isDirty },
  } = useForm<Form>({
    resolver: zodResolver(schema),
    values: {
      displayName: profile?.displayName ?? "",
      username: profile?.username ?? "",
      phone: profile?.phone ?? "",
      bio: profile?.bio ?? "",
    },
  })

  const bio = watch("bio") ?? ""

  async function onSubmit(data: Form) {
    setServerError(null)
    try {
      await updateProfile(data)
      setSuccess(true)
      setTimeout(() => router.back(), 1200)
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

  const initials = (profile?.displayName ?? profile?.username ?? "?")
    .split(" ")
    .map((s) => s[0])
    .filter(Boolean)
    .slice(0, 2)
    .join("")
    .toUpperCase()

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1 bg-background dark:bg-background-dark"
    >
      <SafeAreaView className="flex-1" edges={["bottom"]}>
        <View className="px-5 pt-2 pb-3 flex-row items-center justify-between">
          <TouchableOpacity
            onPress={() => router.back()}
            className="w-10 h-10 items-center justify-center -ml-2"
            activeOpacity={0.7}
          >
            <ChevronLeft size={24} color="#64748B" />
          </TouchableOpacity>
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Edit profile
          </Text>
          <View className="w-10" />
        </View>

        <ScrollView
          className="flex-1 px-5"
          keyboardShouldPersistTaps="handled"
          showsVerticalScrollIndicator={false}
        >
          {/* Avatar */}
          <View className="items-center mb-6 mt-2">
            <View className="relative">
              {profile?.avatarUrl ? (
                <Image
                  source={{ uri: profile.avatarUrl }}
                  className="w-24 h-24 rounded-full"
                />
              ) : (
                <View className="w-24 h-24 rounded-full bg-brand/15 items-center justify-center">
                  <Text className="text-2xl font-bold text-brand">{initials}</Text>
                </View>
              )}
              <TouchableOpacity
                className="absolute bottom-0 right-0 w-9 h-9 rounded-full bg-brand items-center justify-center border-2 border-background dark:border-background-dark"
                activeOpacity={0.85}
              >
                <Camera size={14} color="#FFFFFF" />
              </TouchableOpacity>
            </View>
            <Text className="text-xs text-muted dark:text-muted-dark mt-3">
              Tap the camera to change your avatar
            </Text>
          </View>

          {success ? (
            <View className="mb-4 rounded-xl bg-success/10 border border-success/20 px-4 py-3 flex-row items-center gap-2">
              <CheckCircle2 size={14} color="#10B981" />
              <Text className="text-sm text-success flex-1">Profile updated</Text>
            </View>
          ) : null}

          {serverError ? (
            <View className="mb-4 rounded-xl bg-error-bg border border-error/20 px-4 py-3 flex-row items-center gap-2">
              <AlertTriangle size={14} color="#EF4444" />
              <Text className="text-sm text-error flex-1">{serverError}</Text>
            </View>
          ) : null}

          <Field
            label="Display name"
            Icon={User}
            error={errors.displayName?.message}
          >
            <Controller
              control={control}
              name="displayName"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder="Your name"
                  placeholderTextColor="#94A3B8"
                />
              )}
            />
          </Field>

          <Field label="Username" Icon={AtSign} error={errors.username?.message}>
            <Controller
              control={control}
              name="username"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder="yourhandle"
                  placeholderTextColor="#94A3B8"
                  autoCapitalize="none"
                  autoCorrect={false}
                />
              )}
            />
          </Field>

          <Field label="Phone (optional)" Icon={Phone} error={errors.phone?.message}>
            <Controller
              control={control}
              name="phone"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="h-12 rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 text-sm text-foreground dark:text-foreground-dark"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder="+27 xxx xxx xxxx"
                  placeholderTextColor="#94A3B8"
                  keyboardType="phone-pad"
                />
              )}
            />
          </Field>

          <Field label="Bio (optional)" Icon={FileText} error={undefined}>
            <Controller
              control={control}
              name="bio"
              render={({ field: { onChange, onBlur, value } }) => (
                <TextInput
                  className="rounded-xl border border-border dark:border-border-dark bg-background-gray dark:bg-background-secondary-dark px-4 py-3 text-sm text-foreground dark:text-foreground-dark h-24"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value ?? ""}
                  placeholder="Tell traders about yourself…"
                  placeholderTextColor="#94A3B8"
                  multiline
                  textAlignVertical="top"
                  maxLength={300}
                />
              )}
            />
            <Text className="text-xs text-muted dark:text-muted-dark mt-1 text-right">
              {bio.length}/300
            </Text>
          </Field>

          <View className="h-24" />
        </ScrollView>

        <View className="px-5 pt-3 pb-2 border-t border-border dark:border-border-dark bg-background dark:bg-background-dark">
          <TouchableOpacity
            onPress={handleSubmit(onSubmit)}
            disabled={isSubmitting || !isDirty}
            className={`rounded-xl h-12 items-center justify-center ${
              !isDirty ? "bg-brand/40" : "bg-brand"
            }`}
            activeOpacity={0.85}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text className="text-base font-semibold text-white">Save changes</Text>
            )}
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </KeyboardAvoidingView>
  )
}

function Field({
  label,
  Icon,
  error,
  children,
}: {
  label: string
  Icon: typeof User
  error: string | undefined
  children: React.ReactNode
}) {
  return (
    <View className="mb-4">
      <View className="flex-row items-center gap-2 mb-2">
        <Icon size={14} color="#64748B" />
        <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">{label}</Text>
      </View>
      {children}
      {error ? <Text className="mt-1 text-xs text-error">{error}</Text> : null}
    </View>
  )
}
