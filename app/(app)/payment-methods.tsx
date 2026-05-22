import {
  View, Text, FlatList, TouchableOpacity, ActivityIndicator, Alert,
} from "react-native"
import { SafeAreaView } from "react-native-safe-area-context"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { profileService } from "@/src/services/profile.service"
import type { PaymentMethod } from "@/src/services/profile.service"

function PaymentMethodRow({ method, onDelete }: { method: PaymentMethod; onDelete: () => void }) {
  return (
    <View className="flex-row items-center justify-between py-3.5 border-b border-border/50 dark:border-border-dark/50">
      <View>
        <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
          {method.label}
        </Text>
        <Text className="text-xs text-muted dark:text-muted-dark capitalize">
          {method.type.replace(/_/g, " ")}
          {method.isDefault ? " · Default" : ""}
        </Text>
      </View>
      <TouchableOpacity
        onPress={onDelete}
        className="px-3 py-1.5 rounded-lg bg-error-bg"
        activeOpacity={0.7}
      >
        <Text className="text-xs text-error">Remove</Text>
      </TouchableOpacity>
    </View>
  )
}

export default function PaymentMethodsScreen() {
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
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <View className="px-4 pt-2 pb-3">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark">Payment Methods</Text>
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
            <PaymentMethodRow
              method={item}
              onDelete={() => handleDelete(item.id, item.label)}
            />
          )}
          contentContainerStyle={{ paddingHorizontal: 16 }}
          ListEmptyComponent={
            <View className="items-center justify-center py-20">
              <Text className="text-4xl mb-3">💳</Text>
              <Text className="text-base font-medium text-foreground dark:text-foreground-dark mb-1">
                No payment methods
              </Text>
              <Text className="text-sm text-muted dark:text-muted-dark text-center">
                Payment methods are linked to offers.
                You can specify payment details when creating an offer.
              </Text>
            </View>
          }
        />
      )}
    </SafeAreaView>
  )
}
