import { View, Text, ScrollView, TouchableOpacity, ActivityIndicator, Share, Linking } from "react-native"
import { useLocalSearchParams } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useEffect } from "react"
import QRCode from "react-native-qrcode-svg"
import { Clipboard } from "react-native"
import { useDepositAddress } from "@/src/hooks/api/use-wallet"
import { trackEvent } from "@/src/lib/analytics"

const BLOCK_EXPLORERS: Record<string, (addr: string, network: string) => string> = {
  bitcoin: (addr) => `https://mempool.space/address/${addr}`,
  erc20: (addr) => `https://etherscan.io/address/${addr}`,
  trc20: (addr) => `https://tronscan.org/#/address/${addr}`,
  spl: (addr) => `https://solscan.io/account/${addr}`,
  solana: (addr) => `https://solscan.io/account/${addr}`,
}

export default function DepositScreen() {
  const { currency, network } = useLocalSearchParams<{ currency: string; network: string }>()
  const { data, isLoading, error } = useDepositAddress(currency ?? "", network ?? "", !!(currency && network))
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (data?.address && currency && network) {
      trackEvent({ name: "deposit_address_viewed", currency, network })
    }
  }, [data?.address])

  async function copyAddress() {
    if (!data?.address) return
    Clipboard.setString(data.address)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  async function shareAddress() {
    if (!data?.address) return
    await Share.share({ message: data.address })
  }

  function openExplorer() {
    if (!data?.address || !network) return
    const url = BLOCK_EXPLORERS[network]?.(data.address, network)
    if (url) Linking.openURL(url)
  }

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView className="flex-1 px-4 py-4">
        <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mb-1">
          Deposit {currency}
        </Text>
        <Text className="text-sm text-muted dark:text-muted-dark mb-6 capitalize">
          Network: {network?.replace(/_/g, " ")}
        </Text>

        {isLoading ? (
          <View className="items-center py-20"><ActivityIndicator color="#00A3F6" /></View>
        ) : error ? (
          <View className="rounded-xl bg-error-bg p-4">
            <Text className="text-sm text-error text-center">Failed to load deposit address.</Text>
          </View>
        ) : data ? (
          <>
            {/* QR Code */}
            <View className="items-center mb-6">
              <View className="p-4 rounded-2xl bg-white shadow-sm border border-border dark:border-border-dark">
                <QRCode
                  value={data.address}
                  size={200}
                  color="#000000"
                  backgroundColor="#FFFFFF"
                />
              </View>
              <Text className="text-xs text-muted dark:text-muted-dark mt-2">
                Scan to get address
              </Text>
            </View>

            {/* Address box */}
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              <Text className="text-xs text-muted dark:text-muted-dark mb-1">Deposit address</Text>
              <Text className="text-sm font-medium text-foreground dark:text-foreground-dark break-all font-mono">
                {data.address}
              </Text>
            </View>

            {/* Actions */}
            <View className="flex-row gap-3 mb-4">
              <TouchableOpacity
                onPress={copyAddress}
                className={`flex-1 rounded-lg py-3.5 items-center ${copied ? "bg-success" : "bg-brand"}`}
                activeOpacity={0.8}
              >
                <Text className="text-sm font-semibold text-white">
                  {copied ? "✓ Copied!" : "Copy address"}
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={shareAddress}
                className="flex-1 rounded-lg border border-border dark:border-border-dark py-3.5 items-center"
                activeOpacity={0.8}
              >
                <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">Share</Text>
              </TouchableOpacity>
            </View>

            {BLOCK_EXPLORERS[network ?? ""] ? (
              <TouchableOpacity
                onPress={openExplorer}
                className="mb-4 py-2 items-center"
                activeOpacity={0.7}
              >
                <Text className="text-xs text-brand underline">View on block explorer ↗</Text>
              </TouchableOpacity>
            ) : null}

            {/* Info card */}
            <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
              {[
                ["Minimum deposit", `${data.minDeposit} ${currency}`],
                ["Confirmations required", data.confirmationsRequired.toString()],
                ["Estimated arrival", `~${(data.confirmationsRequired * 10)} minutes`],
              ].map(([label, value]) => (
                <View key={label} className="flex-row justify-between py-2 border-b border-border/30 last:border-0">
                  <Text className="text-xs text-muted dark:text-muted-dark">{label}</Text>
                  <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">{value}</Text>
                </View>
              ))}
            </View>

            <View className="rounded-xl bg-warning-bg p-4">
              <Text className="text-xs text-warning font-semibold mb-1">⚠ Important</Text>
              <Text className="text-xs text-muted dark:text-muted-dark leading-relaxed">
                Only send {currency} on the {network?.replace(/_/g, " ")} network to this address.
                Sending any other asset will result in permanent loss.
              </Text>
            </View>
          </>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  )
}
