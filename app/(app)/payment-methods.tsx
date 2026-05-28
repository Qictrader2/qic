import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useRouter } from "expo-router"
import {
  ChevronLeft,
  CreditCard,
  Smartphone,
  Banknote,
  Coins,
  Plus,
  Trash2,
  Star,
} from "lucide-react-native"
import { profileService } from "@/src/services/profile.service"
import type { PaymentMethod } from "@/src/services/profile.service"

function iconFor(type: string): { Icon: typeof CreditCard; color: string } {
  switch (type) {
    case "bank_transfer":
      return { Icon: Banknote, color: "#10B981" }
    case "mobile_money":
      return { Icon: Smartphone, color: "#F59E0B" }
    case "crypto":
      return { Icon: Coins, color: "#8B5CF6" }
    case "cash":
      return { Icon: Banknote, color: "#64748B" }
    default:
      return { Icon: CreditCard, color: "#00A3F6" }
  }
}

function PaymentMethodCard({
  method,
  onDelete,
}: {
  method: PaymentMethod
  onDelete: () => void
}) {
  const { Icon, color } = iconFor(method.type)
  return (
    <View className="bg-surface dark:bg-card-dark rounded-2xl p-4 mb-3 border border-border dark:border-border-dark">
      <View className="flex-row items-center justify-between">
        <View className="flex-row items-center gap-3 flex-1">
          <View
            className="w-10 h-10 rounded-full items-center justify-center"
            style={{ backgroundColor: color + "22" }}
          >
            <Icon size={18} color={color} />
          </View>
          <View className="flex-1">
            <View className="flex-row items-center gap-1.5">
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                {method.label}
              </Text>
              {method.isDefault ? (
                <View className="px-1.5 py-0.5 rounded-full bg-brand/10 flex-row items-center gap-1">
                  <Star size={9} color="#00A3F6" fill="#00A3F6" />
                  <Text className="text-[10px] font-semibold text-brand">Default</Text>
                </View>
              ) : null}
            </View>
            <Text className="text-xs text-muted dark:text-muted-dark capitalize mt-0.5">
              {method.type.replace(/_/g, " ")}
            </Text>
          </View>
        </View>
        <TouchableOpacity
          onPress={onDelete}
          className="w-9 h-9 rounded-full bg-error/10 items-center justify-center"
          activeOpacity={0.7}
        >
          <Trash2 size={14} color="#EF4444" />
        </TouchableOpacity>
      </View>
    </View>
  )
}

export default function PaymentMethodsScreen() {
  const router = useRouter()
  const qc = useQueryClient()
  const { data: methods, isLoading } = useQuery({
    queryKey: ["payment-methods"],
    queryFn: () => profileService.getPaymentMethods(),
  })
  const { mutateAsync: deleteMethod } = useMutation({
    mutationFn: (id: string) => profileService.deletePaymentMethod(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["payment-methods"] }),
  })

  function handleDelete(id: string, label: string) {
    Alert.alert("Remove payment method", `Remove "${label}"?`, [
      { text: "Cancel", style: "cancel" },
      { text: "Remove", style: "destructive", onPress: () => deleteMethod(id) },
    ])
  }

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
        <View className="flex-1">
          <Text className="text-base font-semibold text-foreground dark:text-foreground-dark">
            Payment methods
          </Text>
        </View>
      </View>

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#00A3F6" />
        </View>
      ) : (
        <FlatList
          data={methods ?? []}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <PaymentMethodCard
              method={item}
              onDelete={() => handleDelete(item.id, item.label)}
            />
          )}
          contentContainerStyle={{ paddingHorizontal: 20, paddingTop: 4, paddingBottom: 32 }}
          ListHeaderComponent={
            <View className="rounded-2xl bg-brand/10 border border-brand/20 p-4 mb-4 flex-row items-center gap-3">
              <View className="w-10 h-10 rounded-full bg-brand/20 items-center justify-center">
                <CreditCard size={18} color="#00A3F6" />
              </View>
              <View className="flex-1">
                <Text className="text-sm font-semibold text-brand">
                  Linked to your offers
                </Text>
                <Text className="text-xs text-brand/80 mt-0.5">
                  Add payment details when creating offers to attract more buyers.
                </Text>
              </View>
            </View>
          }
          ListEmptyComponent={
            <View className="items-center justify-center py-16">
              <View className="w-16 h-16 rounded-full bg-brand/10 items-center justify-center mb-4">
                <CreditCard size={24} color="#00A3F6" />
              </View>
              <Text className="text-base font-semibold text-foreground dark:text-foreground-dark mb-1">
                No payment methods
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark text-center max-w-[260px]">
                Payment methods are linked to your offers. You can specify payment details when creating one.
              </Text>
              <TouchableOpacity
                onPress={() => router.push("/(app)/create-offer" as never)}
                className="mt-5 px-5 h-10 rounded-xl bg-brand flex-row items-center gap-1.5"
                activeOpacity={0.85}
              >
                <Plus size={14} color="#FFFFFF" />
                <Text className="text-sm font-semibold text-white">Create an offer</Text>
              </TouchableOpacity>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
