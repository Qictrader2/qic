import {
  useFonts,
  Poppins_400Regular,
  Poppins_500Medium,
  Poppins_600SemiBold,
  Poppins_700Bold,
} from "@expo-google-fonts/poppins"
import { Stack } from "expo-router"
import * as SplashScreen from "expo-splash-screen"
import { useEffect, useState } from "react"
import { Provider } from "react-redux"
import { QueryClientProvider } from "@tanstack/react-query"
import { GestureHandlerRootView } from "react-native-gesture-handler"
import { SafeAreaProvider } from "react-native-safe-area-context"
import * as Sentry from "@sentry/react-native"
import { store } from "@/src/store"
import { AuthProvider } from "@/src/components/features/auth/AuthProvider"
import { createQueryClient } from "@/src/lib/query-client"
import { handleInitialNotification, useNotificationDeepLink } from "@/src/lib/notification-deep-link"

// Initialise Sentry as early as possible — before any other imports that might throw
Sentry.init({
  dsn: process.env.EXPO_PUBLIC_SENTRY_DSN ?? "",
  enabled: !__DEV__,
  environment: process.env.EXPO_PUBLIC_ENV ?? "production",
  tracesSampleRate: 0.2,
  _experiments: {
    profilesSampleRate: 0.1,
  },
})

SplashScreen.preventAutoHideAsync()

function AppWithDeepLinks({ children }: { children: React.ReactNode }) {
  useNotificationDeepLink()
  return <>{children}</>
}

function RootLayout() {
  const [queryClient] = useState(() => createQueryClient())

  const [fontsLoaded, fontError] = useFonts({
    Poppins_400Regular,
    Poppins_500Medium,
    Poppins_600SemiBold,
    Poppins_700Bold,
  })

  useEffect(() => {
    if (fontError) throw fontError
  }, [fontError])

  useEffect(() => {
    if (fontsLoaded) {
      SplashScreen.hideAsync()
      handleInitialNotification().catch(() => {})
    }
  }, [fontsLoaded])

  if (!fontsLoaded) {
    return null
  }

  return (
    <GestureHandlerRootView style={{ flex: 1 }}>
      <SafeAreaProvider>
        <Provider store={store}>
          <QueryClientProvider client={queryClient}>
            <AuthProvider>
              <AppWithDeepLinks>
                <Stack screenOptions={{ headerShown: false }}>
                  <Stack.Screen name="(auth)" />
                  <Stack.Screen name="(tabs)" />
                  <Stack.Screen name="(app)" />
                  <Stack.Screen name="+not-found" />
                </Stack>
              </AppWithDeepLinks>
            </AuthProvider>
          </QueryClientProvider>
        </Provider>
      </SafeAreaProvider>
    </GestureHandlerRootView>
  )
}

export default Sentry.wrap(RootLayout)
