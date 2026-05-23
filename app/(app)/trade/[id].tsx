import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
  Alert,
  TextInput,
  Modal,
} from "react-native"
import { useLocalSearchParams, useRouter } from "expo-router"
import { SafeAreaView } from "react-native-safe-area-context"
import { useState, useEffect, useRef, useCallback } from "react"
import { useTrade, useMarkPaid, useReleaseEscrow, useCancelTrade, useOpenDispute } from "@/src/hooks/api/use-trade"
import { promptBiometric, isBiometricEnabled, isBiometricAvailable } from "@/src/lib/biometric"
import type { TradeStatus } from "@/src/services/trade.service"
import { ApiError } from "@/src/lib/api/client"
import { trackEvent } from "@/src/lib/analytics"

function statusLabel(s: TradeStatus): string {
  const map: Record<TradeStatus, string> = {
    initiated: "Initiated",
    funded: "Funded — awaiting payment",
    payment_pending: "Awaiting payment",
    payment_sent: "Payment sent",
    payment_confirmed: "Payment confirmed",
    completed: "Completed",
    cancelled: "Cancelled",
    disputed: "Disputed",
    expired: "Expired",
  }
  return map[s]
}

function statusBadgeColors(s: TradeStatus): [string, string] {
  switch (s) {
    case "completed": return ["#10B981", "#10B98120"]
    case "disputed": return ["#EF4444", "#EF444420"]
    case "cancelled":
    case "expired": return ["#6B7280", "#6B728020"]
    default: return ["#F59E0B", "#F59E0B20"]
  }
}

/** Format remaining seconds as MM:SS */
function formatCountdown(seconds: number): string {
  if (seconds <= 0) return "00:00"
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`
}

function ExpiryCountdown({ expiresAt }: { expiresAt: string }) {
  const [remaining, setRemaining] = useState(() => {
    const diff = Math.floor((new Date(expiresAt).getTime() - Date.now()) / 1000)
    return Math.max(0, diff)
  })

  useEffect(() => {
    if (remaining <= 0) return
    const timer = setInterval(() => {
      setRemaining((prev) => {
        if (prev <= 1) {
          clearInterval(timer)
          return 0
        }
        return prev - 1
      })
    }, 1000)
    return () => clearInterval(timer)
  }, [])

  const isUrgent = remaining > 0 && remaining < 300 // < 5 min
  const color = remaining === 0 ? "#EF4444" : isUrgent ? "#F59E0B" : "#10B981"

  return (
    <View className="flex-row items-center gap-1.5 py-2 px-3 rounded-lg bg-surface dark:bg-surface-dark border border-border dark:border-border-dark mb-4">
      <Text className="text-xs text-muted dark:text-muted-dark">Payment window:</Text>
      <Text className="text-sm font-bold font-mono" style={{ color }}>
        {remaining === 0 ? "Expired" : formatCountdown(remaining)}
      </Text>
      {isUrgent && remaining > 0 ? (
        <Text className="text-xs text-warning">⚠ Expiring soon</Text>
      ) : null}
    </View>
  )
}

const QUICK_REPLIES = [
  "I've sent the payment",
  "Please check your account",
  "Payment confirmed, please release",
  "I'm ready to proceed",
  "Please contact support if there's an issue",
]

export default function TradeDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const router = useRouter()
  const { data: trade, isLoading, error, refetch, isRefetching } = useTrade(id ?? "")
  const { mutateAsync: markPaid, isPending: markingPaid } = useMarkPaid()
  const { mutateAsync: releaseEscrow, isPending: releasing } = useReleaseEscrow()
  const { mutateAsync: cancelTrade, isPending: cancelling } = useCancelTrade()
  const { mutateAsync: openDispute, isPending: disputing } = useOpenDispute()

  const [show2FA, setShow2FA] = useState(false)
  const [twoFactorCode, setTwoFactorCode] = useState("")
  const [releaseError, setReleaseError] = useState<string | null>(null)
  const [showRating, setShowRating] = useState(false)
  const [rating, setRating] = useState(0)
  const [ratingComment, setRatingComment] = useState("")
  const prevStatusRef = useRef<TradeStatus | null>(null)

  // Detect trade completion to prompt rating
  useEffect(() => {
    if (!trade) return
    if (prevStatusRef.current && prevStatusRef.current !== "completed" && trade.status === "completed") {
      setShowRating(true)
    }
    prevStatusRef.current = trade.status
  }, [trade?.status])

  async function handleMarkPaid() {
    try {
      await markPaid(id ?? "")
      trackEvent({ name: "trade_paid", tradeId: id ?? "" })
    } catch {
      Alert.alert("Error", "Failed to mark as paid. Please try again.")
    }
  }

  async function handleRelease() {
    setReleaseError(null)
    const biometricEnabled = await isBiometricEnabled()
    const biometricAvailable = await isBiometricAvailable()
    if (biometricEnabled && biometricAvailable) {
      const passed = await promptBiometric("Confirm escrow release")
      if (!passed) {
        setReleaseError("Biometric verification failed.")
        return
      }
    }
    setShow2FA(true)
  }

  async function handleConfirmRelease() {
    setReleaseError(null)
    try {
      await releaseEscrow({ id: id ?? "", twoFactorCode })
      setShow2FA(false)
      setTwoFactorCode("")
      trackEvent({ name: "trade_released", tradeId: id ?? "" })
    } catch (err) {
      const apiErr = err as ApiError
      if (apiErr.kind === "unauthorized") {
        setReleaseError("Invalid 2FA code.")
      } else {
        setReleaseError("Release failed. Please try again.")
      }
    }
  }

  async function handleCancel() {
    Alert.alert("Cancel trade", "Are you sure you want to cancel this trade?", [
      { text: "No", style: "cancel" },
      {
        text: "Cancel trade",
        style: "destructive",
        onPress: async () => {
          try {
            await cancelTrade({ id: id ?? "" })
            trackEvent({ name: "trade_cancelled", tradeId: id ?? "", reason: "user_initiated" })
          } catch {
            Alert.alert("Error", "Failed to cancel trade.")
          }
        },
      },
    ])
  }

  async function submitRating() {
    // POST to backend rating endpoint
    try {
      const { apiClient } = require("@/src/lib/api/client")
      await apiClient.post(`/api/v1/trades/${id}/rating`, { rating, comment: ratingComment })
    } catch {}
    setShowRating(false)
  }

  if (isLoading) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark items-center justify-center">
        <ActivityIndicator color="#00A3F6" />
      </SafeAreaView>
    )
  }

  if (error || !trade) {
    return (
      <SafeAreaView className="flex-1 bg-background dark:bg-background-dark px-4 justify-center">
        <View className="rounded-xl bg-error-bg p-4">
          <Text className="text-sm text-error text-center">Failed to load trade.</Text>
        </View>
      </SafeAreaView>
    )
  }

  const [badgeColor, badgeBg] = statusBadgeColors(trade.status)

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView
        className="flex-1 px-4 py-4"
        refreshControl={
          <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
        }
      >
        {/* Status badge */}
        <View className="flex-row items-center gap-2 mb-4">
          <View className="px-3 py-1 rounded-full" style={{ backgroundColor: badgeBg }}>
            <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
              {statusLabel(trade.status)}
            </Text>
          </View>
          <Text className="text-xs text-muted dark:text-muted-dark capitalize">{trade.role}</Text>
        </View>

        {/* Live expiry countdown for active trades */}
        {trade.expiresAt && ["initiated", "funded", "payment_pending", "payment_sent"].includes(trade.status) ? (
          <ExpiryCountdown expiresAt={trade.expiresAt} />
        ) : null}

        {/* Trade info */}
        <View className="rounded-xl bg-surface dark:bg-surface-dark border border-border dark:border-border-dark p-4 mb-4">
          {[
            ["Crypto amount", `${trade.cryptoAmount} ${trade.currency}`],
            ["Fiat amount", `${trade.fiatCurrency} ${parseFloat(trade.fiatAmount).toLocaleString()}`],
            ["Price", `${trade.fiatCurrency} ${parseFloat(trade.pricePerUnit).toLocaleString()}`],
            ["Payment method", trade.paymentMethod.replace(/_/g, " ")],
            ["Counterparty", trade.counterparty.username],
            ["Trade ID", `#${trade.id.slice(0, 8)}`],
          ].map(([label, value]) => (
            <View
              key={label}
              className="flex-row justify-between py-2 border-b border-border/50 dark:border-border-dark/50 last:border-0"
            >
              <Text className="text-sm text-muted dark:text-muted-dark">{label}</Text>
              <Text className="text-sm text-foreground dark:text-foreground-dark font-medium flex-shrink ml-4 text-right">
                {value}
              </Text>
            </View>
          ))}
        </View>

        {releaseError ? (
          <View className="mb-4 rounded-lg bg-error-bg px-4 py-3">
            <Text className="text-sm text-error">{releaseError}</Text>
          </View>
        ) : null}

        {/* Chat button */}
        <TouchableOpacity
          onPress={() => router.push({ pathname: "/(app)/trade-chat/[id]", params: { id: trade.id } })}
          className="rounded-lg border border-brand bg-brand-bg py-3.5 items-center mb-3"
          activeOpacity={0.8}
        >
          <Text className="text-sm font-medium text-brand">Open chat</Text>
        </TouchableOpacity>

        {/* Buyer: mark paid */}
        {trade.role === "buyer" && trade.status === "funded" ? (
          <TouchableOpacity
            onPress={handleMarkPaid}
            disabled={markingPaid}
            className="rounded-lg bg-brand py-4 items-center mb-3"
            activeOpacity={0.8}
          >
            {markingPaid ? <ActivityIndicator color="#fff" /> : (
              <Text className="text-base font-semibold text-white">I have paid</Text>
            )}
          </TouchableOpacity>
        ) : null}

        {/* Buyer: upload proof */}
        {trade.role === "buyer" && ["funded", "payment_pending"].includes(trade.status) ? (
          <TouchableOpacity
            onPress={() =>
              router.push({ pathname: "/(app)/proof-of-payment", params: { id: trade.id } })
            }
            className="rounded-lg border border-brand bg-brand-bg py-3.5 items-center mb-3"
            activeOpacity={0.8}
          >
            <Text className="text-sm font-medium text-brand">Upload proof of payment</Text>
          </TouchableOpacity>
        ) : null}

        {/* Seller: release */}
        {trade.role === "seller" && trade.status === "payment_confirmed" ? (
          <TouchableOpacity
            onPress={handleRelease}
            disabled={releasing}
            className="rounded-lg bg-success py-4 items-center mb-3"
            activeOpacity={0.8}
          >
            {releasing ? <ActivityIndicator color="#fff" /> : (
              <Text className="text-base font-semibold text-white">Release crypto</Text>
            )}
          </TouchableOpacity>
        ) : null}

        {/* Cancel */}
        {["initiated", "funded", "payment_pending"].includes(trade.status) ? (
          <TouchableOpacity
            onPress={handleCancel}
            disabled={cancelling}
            className="rounded-lg border border-error bg-error-bg py-3.5 items-center mb-3"
            activeOpacity={0.8}
          >
            <Text className="text-sm font-medium text-error">Cancel trade</Text>
          </TouchableOpacity>
        ) : null}

        {/* Dispute */}
        {["payment_sent", "payment_confirmed"].includes(trade.status) ? (
          <TouchableOpacity
            onPress={() =>
              router.push({ pathname: "/(app)/dispute/[id]", params: { id: trade.id } })
            }
            className="rounded-lg border border-error bg-error-bg py-3.5 items-center mb-3"
            activeOpacity={0.8}
          >
            <Text className="text-sm font-medium text-error">Open dispute</Text>
          </TouchableOpacity>
        ) : null}

        {/* Rate counterparty for completed trade */}
        {trade.status === "completed" ? (
          <TouchableOpacity
            onPress={() => setShowRating(true)}
            className="rounded-lg border border-border dark:border-border-dark py-3.5 items-center mb-3"
            activeOpacity={0.8}
          >
            <Text className="text-sm font-medium text-foreground dark:text-foreground-dark">
              ⭐ Rate {trade.counterparty.username}
            </Text>
          </TouchableOpacity>
        ) : null}

        <View className="h-8" />
      </ScrollView>

      {/* 2FA release modal */}
      <Modal visible={show2FA} transparent animationType="slide">
        <View className="flex-1 justify-end bg-black/50">
          <View className="bg-background dark:bg-background-dark rounded-t-2xl p-6">
            <Text className="text-lg font-bold text-foreground dark:text-foreground-dark mb-2">
              Confirm release
            </Text>
            <Text className="text-sm text-muted dark:text-muted-dark mb-4">
              Enter your 6-digit 2FA code to release crypto to the buyer.
            </Text>
            {releaseError ? (
              <View className="mb-3 rounded-lg bg-error-bg px-4 py-2">
                <Text className="text-xs text-error">{releaseError}</Text>
              </View>
            ) : null}
            <TextInput
              className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-2xl text-center tracking-widest text-foreground dark:text-foreground-dark mb-4"
              value={twoFactorCode}
              onChangeText={setTwoFactorCode}
              placeholder="000000"
              placeholderTextColor="#94A3B8"
              keyboardType="number-pad"
              maxLength={6}
              autoFocus
            />
            <TouchableOpacity
              onPress={handleConfirmRelease}
              disabled={twoFactorCode.length < 6 || releasing}
              className="rounded-lg bg-success py-4 items-center mb-3"
              activeOpacity={0.8}
            >
              {releasing ? <ActivityIndicator color="#fff" /> : (
                <Text className="text-base font-semibold text-white">Release</Text>
              )}
            </TouchableOpacity>
            <TouchableOpacity
              onPress={() => { setShow2FA(false); setTwoFactorCode("") }}
              className="py-3 items-center"
            >
              <Text className="text-sm text-muted dark:text-muted-dark">Cancel</Text>
            </TouchableOpacity>
          </View>
        </View>
      </Modal>

      {/* Rating modal */}
      <Modal visible={showRating} transparent animationType="slide">
        <View className="flex-1 justify-end bg-black/50">
          <View className="bg-background dark:bg-background-dark rounded-t-2xl p-6">
            <Text className="text-lg font-bold text-foreground dark:text-foreground-dark mb-1">
              Rate {trade.counterparty.username}
            </Text>
            <Text className="text-sm text-muted dark:text-muted-dark mb-5">
              How was your experience with this trade?
            </Text>

            {/* Star rating */}
            <View className="flex-row justify-center gap-3 mb-5">
              {[1, 2, 3, 4, 5].map((star) => (
                <TouchableOpacity key={star} onPress={() => setRating(star)} activeOpacity={0.7}>
                  <Text className="text-3xl">{star <= rating ? "⭐" : "☆"}</Text>
                </TouchableOpacity>
              ))}
            </View>

            <TextInput
              className="rounded-lg border border-border dark:border-border-dark bg-surface dark:bg-surface-dark px-3 py-3 text-sm text-foreground dark:text-foreground-dark mb-4 h-20"
              value={ratingComment}
              onChangeText={setRatingComment}
              placeholder="Add a comment (optional)"
              placeholderTextColor="#94A3B8"
              multiline
              textAlignVertical="top"
            />
            <TouchableOpacity
              onPress={submitRating}
              disabled={rating === 0}
              className={`rounded-lg py-4 items-center mb-3 ${rating > 0 ? "bg-brand" : "bg-muted/30"}`}
              activeOpacity={0.8}
            >
              <Text className="text-base font-semibold text-white">Submit rating</Text>
            </TouchableOpacity>
            <TouchableOpacity onPress={() => setShowRating(false)} className="py-3 items-center">
              <Text className="text-sm text-muted dark:text-muted-dark">Skip</Text>
            </TouchableOpacity>
          </View>
        </View>
      </Modal>
    </SafeAreaView>
  )
}
