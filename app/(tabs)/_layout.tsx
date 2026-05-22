import { Tabs } from "expo-router"
import { Platform } from "react-native"
import { useColorScheme } from "@/components/useColorScheme"

export default function TabLayout() {
  const colorScheme = useColorScheme()
  const isDark = colorScheme === "dark"

  return (
    <Tabs
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: "#00A3F6",
        tabBarInactiveTintColor: isDark ? "#94A3B8" : "#475569",
        tabBarStyle: {
          backgroundColor: isDark ? "#191F2A" : "#FFFFFF",
          borderTopColor: isDark ? "rgba(255,255,255,0.1)" : "#E2E8F0",
          paddingBottom: Platform.OS === "ios" ? 20 : 8,
          height: Platform.OS === "ios" ? 88 : 64,
        },
        tabBarLabelStyle: {
          fontFamily: "Poppins_500Medium",
          fontSize: 11,
        },
      }}
    >
      <Tabs.Screen
        name="index"
        options={{
          title: "Wallet",
          tabBarIcon: ({ color }) => (
            <TabIcon name="wallet" color={color as string} />
          ),
        }}
      />
      <Tabs.Screen
        name="marketplace"
        options={{
          title: "Market",
          tabBarIcon: ({ color }) => (
            <TabIcon name="market" color={color as string} />
          ),
        }}
      />
      <Tabs.Screen
        name="trades"
        options={{
          title: "Trades",
          tabBarIcon: ({ color }) => (
            <TabIcon name="trades" color={color as string} />
          ),
        }}
      />
      <Tabs.Screen
        name="notifications"
        options={{
          title: "Alerts",
          tabBarIcon: ({ color }) => (
            <TabIcon name="bell" color={color as string} />
          ),
        }}
      />
      <Tabs.Screen
        name="profile"
        options={{
          title: "Profile",
          tabBarIcon: ({ color }) => (
            <TabIcon name="person" color={color as string} />
          ),
        }}
      />
    </Tabs>
  )
}

function TabIcon({ name, color }: { name: string; color: string }) {
  // Temporary text icon until MOBILE-INIT-005 provides proper SVG icons
  const icons: Record<string, string> = {
    wallet: "💳",
    market: "🏪",
    trades: "🔄",
    bell: "🔔",
    person: "👤",
  }
  const { Text } = require("react-native")
  return <Text style={{ fontSize: 20, color }}>{icons[name] ?? "●"}</Text>
}
