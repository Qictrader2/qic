import { View, Text, TouchableOpacity, Image } from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { Stack, useRouter } from "expo-router"
import { Compass, ArrowLeft } from "lucide-react-native"

export default function NotFoundScreen() {
  const router = useRouter()

  return (
    <>
      <Stack.Screen options={{ title: "Page not found", headerShown: false }} />
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
        <View className="flex-1 items-center justify-center px-6">
          <Image
            source={require("@/assets/images/logo.png")}
            style={{ width: 140, height: 48, marginBottom: 32 }}
            resizeMode="contain"
          />

          <View className="w-20 h-20 rounded-full bg-brand/10 items-center justify-center mb-5">
            <Compass size={36} color="#00A3F6" />
          </View>

          <Text className="text-5xl font-bold text-foreground dark:text-foreground-dark mb-2">
            404
          </Text>
          <Text className="text-xl font-semibold text-foreground dark:text-foreground-dark mb-2">
            Page not found
          </Text>
          <Text className="text-sm text-muted dark:text-muted-dark text-center mb-8 max-w-[300px] leading-5">
            The screen you were looking for doesn't exist or has moved. Let's get you back home.
          </Text>

          <TouchableOpacity
            onPress={() => router.replace("/(tabs)" as never)}
            className="rounded-xl bg-brand px-6 h-12 items-center justify-center flex-row gap-2"
            activeOpacity={0.85}
          >
            <ArrowLeft size={16} color="#FFFFFF" />
            <Text className="text-base font-semibold text-white">Back to home</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    </>
  )
}
