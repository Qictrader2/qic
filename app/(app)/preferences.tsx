import { View, Text, ScrollView, TouchableOpacity, Switch } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"

const CURRENCIES = ["ZAR", "USD", "EUR", "GBP", "NGN", "KES", "GHS"]
const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "af", label: "Afrikaans" },
]
const THEMES = ["system", "light", "dark"] as const
type Theme = typeof THEMES[number]

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

export default function PreferencesScreen() {
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

  function SectionHeader({ title }: { title: string }) {
    return (
      <Text className="text-xs font-semibold text-muted dark:text-muted-dark uppercase tracking-wider mt-6 mb-2">
        {title}
      </Text>
    )
  }

  if (!prefs) return null

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark pt-4 mb-2">
          Preferences
        </Text>

        <SectionHeader title="Display currency" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4 overflow-hidden">
          {CURRENCIES.map((c, i) => (
            <TouchableOpacity
              key={c}
              onPress={() => update({ displayCurrency: c })}
              className={`flex-row items-center justify-between py-3.5 ${
                i < CURRENCIES.length - 1 ? "border-b border-border/50 dark:border-border-dark/50" : ""
              }`}
              activeOpacity={0.7}
            >
              <Text className="text-sm text-foreground dark:text-foreground-dark">{c}</Text>
              {prefs.displayCurrency === c ? (
                <Text className="text-brand font-medium">✓</Text>
              ) : null}
            </TouchableOpacity>
          ))}
        </View>

        <SectionHeader title="Theme" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark overflow-hidden">
          <View className="flex-row">
            {THEMES.map((t) => (
              <TouchableOpacity
                key={t}
                onPress={() => update({ theme: t })}
                className={`flex-1 py-3 items-center ${
                  prefs.theme === t ? "bg-brand" : ""
                }`}
                activeOpacity={0.8}
              >
                <Text className={`text-sm font-medium capitalize ${prefs.theme === t ? "text-white" : "text-muted dark:text-muted-dark"}`}>
                  {t}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>

        <SectionHeader title="Language" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4 overflow-hidden">
          {LANGUAGES.map(({ code, label }, i) => (
            <TouchableOpacity
              key={code}
              onPress={() => update({ language: code })}
              className={`flex-row items-center justify-between py-3.5 ${
                i < LANGUAGES.length - 1 ? "border-b border-border/50 dark:border-border-dark/50" : ""
              }`}
              activeOpacity={0.7}
            >
              <Text className="text-sm text-foreground dark:text-foreground-dark">{label}</Text>
              {prefs.language === code ? (
                <Text className="text-brand font-medium">✓</Text>
              ) : null}
            </TouchableOpacity>
          ))}
        </View>

        <SectionHeader title="Display options" />
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark px-4">
          {[
            {
              key: "compactNumbers" as const,
              label: "Compact numbers",
              desc: "Show 1.2K instead of 1,200",
            },
            {
              key: "showPortfolioValue" as const,
              label: "Show portfolio value",
              desc: "Display total value on wallet screen",
            },
          ].map(({ key, label, desc }, i, arr) => (
            <View
              key={key}
              className={`flex-row items-center justify-between py-3.5 ${
                i < arr.length - 1 ? "border-b border-border/50 dark:border-border-dark/50" : ""
              }`}
            >
              <View className="flex-1 mr-4">
                <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
                  {label}
                </Text>
                <Text className="text-xs text-muted dark:text-muted-dark mt-0.5">{desc}</Text>
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

        <View className="h-8" />
      </ScrollView>
    </SafeAreaView>
  )
}
