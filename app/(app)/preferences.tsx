import { View, Text, ScrollView, TouchableOpacity, Switch } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useRouter } from "expo-router"
import {
  ChevronLeft,
  Check,
  Sun,
  Moon,
  Smartphone,
  Globe,
  Banknote,
  Eye,
  Hash,
} from "lucide-react-native"

const CURRENCIES = ["ZAR", "USD", "EUR", "GBP", "NGN", "KES", "GHS"]
const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "af", label: "Afrikaans" },
]
const THEMES = [
  { id: "system", label: "Auto", Icon: Smartphone },
  { id: "light", label: "Light", Icon: Sun },
  { id: "dark", label: "Dark", Icon: Moon },
] as const
type Theme = (typeof THEMES)[number]["id"]

interface Preferences {
  displayCurrency: string
  language: string
  theme: Theme
  compactNumbers: boolean
  showPortfolioValue: boolean
}

const PREFS_KEY = "qic_ui_preferences"

async function loadPrefs(): Promise<Preferences> {
  const raw = await AsyncStorage.getItem(PREFS_KEY)
  return raw
    ? (JSON.parse(raw) as Preferences)
    : {
        displayCurrency: "ZAR",
        language: "en",
        theme: "system",
        compactNumbers: false,
        showPortfolioValue: true,
      }
}

async function savePrefs(prefs: Preferences): Promise<void> {
  await AsyncStorage.setItem(PREFS_KEY, JSON.stringify(prefs))
}

function SectionTitle({ Icon, label }: { Icon: typeof Sun; label: string }) {
  return (
    <View className="flex-row items-center gap-2 mb-2.5 mt-5 px-1">
      <Icon size={14} color="#00A3F6" />
      <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider">
        {label}
      </Text>
    </View>
  )
}

export default function PreferencesScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const { data: prefs } = useQuery({ queryKey: ["ui-prefs"], queryFn: loadPrefs })
  const { mutate: updatePrefs } = useMutation({
    mutationFn: (updated: Preferences) => savePrefs(updated),
    onSuccess: (_, updated) => qc.setQueryData(["ui-prefs"], updated),
  })

  function update(patch: Partial<Preferences>) {
    if (!prefs) return
    updatePrefs({ ...prefs, ...patch })
  }

  if (!prefs) return null

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark" edges={["bottom"]}>
      <View className="px-5 pt-2 pb-3 flex-row items-center">
        <TouchableOpacity
          onPress={() => router.back()}
          className="w-10 h-10 items-center justify-center -ml-2"
          activeOpacity={0.7}
        >
          <ChevronLeft size={24} color="#64748B" />
        </TouchableOpacity>
        <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
          Preferences
        </Text>
      </View>

      <ScrollView className="flex-1 px-5" showsVerticalScrollIndicator={false}>
        <SectionTitle Icon={Sun} label="Appearance" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-1">
          <View className="flex-row">
            {THEMES.map(({ id, label, Icon }) => {
              const isActive = prefs.theme === id
              return (
                <TouchableOpacity
                  key={id}
                  onPress={() => update({ theme: id })}
                  className={`flex-1 py-3 items-center rounded-xl ${isActive ? "bg-brand" : ""}`}
                  activeOpacity={0.85}
                >
                  <Icon size={16} color={isActive ? "#FFFFFF" : "#64748B"} />
                  <Text
                    className={`text-xs font-semibold mt-1 ${
                      isActive ? "text-white" : "text-muted dark:text-muted-dark"
                    }`}
                  >
                    {label}
                  </Text>
                </TouchableOpacity>
              )
            })}
          </View>
        </View>

        <SectionTitle Icon={Banknote} label="Display currency" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark overflow-hidden">
          {CURRENCIES.map((c, i) => {
            const isActive = prefs.displayCurrency === c
            return (
              <TouchableOpacity
                key={c}
                onPress={() => update({ displayCurrency: c })}
                className={`flex-row items-center justify-between px-4 py-3.5 ${
                  i < CURRENCIES.length - 1
                    ? "border-b border-border/40 dark:border-border-dark/40"
                    : ""
                }`}
                activeOpacity={0.7}
              >
                <Text
                  className={`text-sm ${
                    isActive
                      ? "text-brand font-semibold"
                      : "text-foreground dark:text-foreground-dark"
                  }`}
                >
                  {c}
                </Text>
                {isActive ? <Check size={16} color="#00A3F6" /> : null}
              </TouchableOpacity>
            )
          })}
        </View>

        <SectionTitle Icon={Globe} label="Language" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark overflow-hidden">
          {LANGUAGES.map(({ code, label }, i) => {
            const isActive = prefs.language === code
            return (
              <TouchableOpacity
                key={code}
                onPress={() => update({ language: code })}
                className={`flex-row items-center justify-between px-4 py-3.5 ${
                  i < LANGUAGES.length - 1
                    ? "border-b border-border/40 dark:border-border-dark/40"
                    : ""
                }`}
                activeOpacity={0.7}
              >
                <Text
                  className={`text-sm ${
                    isActive
                      ? "text-brand font-semibold"
                      : "text-foreground dark:text-foreground-dark"
                  }`}
                >
                  {label}
                </Text>
                {isActive ? <Check size={16} color="#00A3F6" /> : null}
              </TouchableOpacity>
            )
          })}
        </View>

        <SectionTitle Icon={Eye} label="Display options" />
        <View className="rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark px-4">
          {[
            {
              key: "compactNumbers" as const,
              label: "Compact numbers",
              desc: "Show 1.2K instead of 1,200",
              Icon: Hash,
            },
            {
              key: "showPortfolioValue" as const,
              label: "Show portfolio value",
              desc: "Display total value on wallet screen",
              Icon: Eye,
            },
          ].map(({ key, label, desc, Icon }, i, arr) => (
            <View
              key={key}
              className={`flex-row items-center justify-between py-4 ${
                i < arr.length - 1
                  ? "border-b border-border/40 dark:border-border-dark/40"
                  : ""
              }`}
            >
              <View className="flex-row items-start gap-3 flex-1 mr-4">
                <View className="w-9 h-9 rounded-full bg-brand/10 items-center justify-center mt-0.5">
                  <Icon size={14} color="#00A3F6" />
                </View>
                <View className="flex-1">
                  <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                    {label}
                  </Text>
                  <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">{desc}</Text>
                </View>
              </View>
              <Switch
                value={prefs[key]}
                onValueChange={(val) => update({ [key]: val })}
                trackColor={{ false: "#E2E8F0", true: "#00A3F6" }}
                thumbColor="#fff"
              />
            </View>
          ))}
        </View>

        <View className="h-12" />
      </ScrollView>
    </SafeAreaView>
  )
}
