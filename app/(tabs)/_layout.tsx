import { Tabs } from "expo-router"
import { Platform } from "react-native"
import { Home, Store, ArrowLeftRight, Wallet, User } from "lucide-react-native"
import { useColorScheme } from "@/components/useColorScheme"

export default function TabLayout() {
  const colorScheme = useColorScheme()
  const isDark = colorScheme === "dark"

  return (
    <Tabs
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: "#00A3F6",
        tabBarInactiveTintColor: isDark ? "#94A3B8" : "#64748B",
        tabBarStyle: {
          backgroundColor: isDark ? "#191F2A" : "#FFFFFF",
          borderTopColor: isDark ? "rgba(255,255,255,0.1)" : "#E2E8F0",
          paddingBottom: Platform.OS === "ios" ? 20 : 8,
          paddingTop: 8,
          height: Platform.OS === "ios" ? 88 : 68,
        },
        tabBarLabelStyle: {
          fontFamily: "Poppins_500Medium",
          fontSize: 11,
          marginTop: 2,
        },
      }}
    >
      <Tabs.Screen
        name="index"
        options={{
          title: "Home",
          tabBarIcon: ({ color, focused }) => (
            <Home size={22} color={color} strokeWidth={focused ? 2.4 : 2} />
          ),
        }}
      />
      <Tabs.Screen
        name="marketplace"
        options={{
          title: "Market",
          tabBarIcon: ({ color, focused }) => (
            <Store size={22} color={color} strokeWidth={focused ? 2.4 : 2} />
          ),
        }}
      />
      <Tabs.Screen
        name="trades"
        options={{
          title: "Trades",
          tabBarIcon: ({ color, focused }) => (
            <ArrowLeftRight size={22} color={color} strokeWidth={focused ? 2.4 : 2} />
          ),
        }}
      />
      <Tabs.Screen
        name="notifications"
        options={{
          // Hidden: alerts are surfaced via the profile menu / notification bell instead.
          // Keeping the screen file so the route still resolves.
          href: null,
        }}
      />
      <Tabs.Screen
        name="two"
        options={{ href: null }}
      />
      <Tabs.Screen
        name="profile"
        options={{
          title: "Profile",
          tabBarIcon: ({ color, focused }) => (
            <User size={22} color={color} strokeWidth={focused ? 2.4 : 2} />
          ),
        }}
      />
    </Tabs>
  )
}
