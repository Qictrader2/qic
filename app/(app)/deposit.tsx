import { View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Share } from "react-native"
import { useLocalSearchParams, useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState } from "react"
import { useDepositAddress } from "@/src/hooks/api/use-wallet"
import type { Currency, Network } from "@/src/services/wallet.service"

export default function DepositScreen() {
  const { currency, network } = useLocalSearchParams<{ currency: string; network: string }>()
  const router = useRouter()
  const { data, isLoading, error } = useDepositAddress(currency ?? "", network ?? "", !!(currency && network))

  async function copyAddress() {
    if (!data?.address) return
    const Clipboard = require("@react-native-clipboard/clipboard")
    Clipboard.default.setString(data.address)
  }

  async function shareAddress() {
    if (!data?.address) return
    await Share.share({ message: data.address })
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-1">
          Deposit {currency}
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-6 capitalize">
          Network: {network}
        </Text>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : error ? (
          <View className="rounded-xl bg-error-bg p-4">
            <Text className="text-sm text-error text-center">Failed to load deposit address.</Text>
          </View>
        ) : data ? (
          <>
            {/* QR Code placeholder — MOBILE-INIT-005 will add proper QR */}
            <View className="items-center mb-6">
              <View className="h-48 w-48 rounded-xl bg-surface dark:bg-surface-dark border-2 border-brand items-center justify-center">
                <Text className="text-xs text-muted dark:text-muted-dark text-center px-4">
                  QR Code{"\n"}{data.address.slice(0, 12)}…
                </Text>
              </View>
            </View>

            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <Text className="text-xs text-muted dark:text-muted-dark mb-1">Deposit address</Text>
              <Text className="text-sm font-medium text-foreground dark:text-foreground-dark break-all">
                {data.address}
              </Text>
            </View>

            <View className="flex-row gap-3 mb-6">
              <TouchableOpacity
                onPress={copyAddress}
                className="flex-1 rounded-lg bg-brand py-3.5 items-center"
                activeOpacity={0.8}
              >
                <Text className="text-sm font-semibold text-white">Copy address</Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={shareAddress}
                className="flex-1 rounded-lg border border-border dark:border-border-dark py-3.5 items-center"
                activeOpacity={0.8}
              >
                <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">Share</Text>
              </TouchableOpacity>
            </View>

            <View className="rounded-xl bg-warning-bg p-4">
              <Text className="text-xs text-warning font-medium mb-1">Important</Text>
              <Text className="text-xs text-muted dark:text-muted-dark leading-relaxed">
                Only send {currency} on the {network} network to this address.
                Minimum deposit: {data.minDeposit} {currency}.
                Requires {data.confirmationsRequired} confirmations.
              </Text>
            </View>
          </>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  )
}
