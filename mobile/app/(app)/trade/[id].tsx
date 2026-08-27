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
import { MessageCircle, ShieldCheck, Star, Upload, AlertTriangle, X } from "lucide-react-native"
import { useTrade, useMarkPaid, useReleaseEscrow, useCancelTrade, useOpenDispute } from "@/src/hooks/api/use-trade"
import { promptBiometric, isBiometricEnabled, isBiometricAvailable } from "@/src/lib/biometric"
import type { TradeStatus } from "@/src/services/trade.service"
import { ApiError } from "@/src/lib/api/client"
import { trackEvent } from "@/src/lib/analytics"
import { TradeStatusBanner } from "@/src/components/features/trade/TradeStatusBanner"
import { CounterpartyCard } from "@/src/components/features/trade/CounterpartyCard"

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

  return (
    <SafeAreaView className="flex-1 bg-background dark:bg-background-dark">
      <ScrollView
        className="flex-1"
        refreshControl={
          <RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#00A3F6" />
        }
      >
        {/* Status banner (full-width, color-coded) */}
        <TradeStatusBanner status={trade.status} role={trade.role} />

        <View className="px-4 py-4">
          {/* Live expiry countdown for active trades */}
          {trade.expiresAt && ["initiated", "funded", "payment_pending", "payment_sent"].includes(trade.status) ? (
            <ExpiryCountdown expiresAt={trade.expiresAt} />
          ) : null}

          {/* Counterparty */}
          <CounterpartyCard counterparty={trade.counterparty} role={trade.role} />

          {/* Amount summary */}
          <View className="mt-3 rounded-2xl bg-surface dark:bg-card-dark border border-border dark:border-border-dark p-4">
            <View className="flex-row items-end justify-between">
              <View>
                <Text className="text-xs text-muted dark:text-muted-dark">
                  You {trade.role === "buyer" ? "receive" : "send"}
                </Text>
                <Text className="text-xl font-bold text-foreground dark:text-foreground-dark mt-0.5">
                  {trade.cryptoAmount} {trade.currency}
                </Text>
              </View>
              <View className="items-end">
                <Text className="text-xs text-muted dark:text-muted-dark">
                  You {trade.role === "buyer" ? "send" : "receive"}
                </Text>
                <Text className="text-base font-semibold text-foreground dark:text-foreground-dark mt-0.5">
                  {trade.fiatCurrency} {parseFloat(trade.fiatAmount).toLocaleString()}
                </Text>
              </View>
            </View>
            <View className="h-px bg-border dark:bg-border-dark my-3" />
            <View className="flex-row justify-between">
              <Text className="text-xs text-muted dark:text-muted-dark">Price</Text>
              <Text className="text-xs font-medium text-foreground dark:text-foreground-dark">
                {trade.fiatCurrency} {parseFloat(trade.pricePerUnit).toLocaleString()} / {trade.currency}
              </Text>
            </View>
            <View className="flex-row justify-between mt-1">
              <Text className="text-xs text-muted dark:text-muted-dark">Payment method</Text>
              <Text className="text-xs font-medium text-foreground dark:text-foreground-dark capitalize">
                {trade.paymentMethod.replace(/_/g, " ")}
              </Text>
            </View>
            <View className="flex-row justify-between mt-1">
              <Text className="text-xs text-muted dark:text-muted-dark">Trade ID</Text>
              <Text className="text-xs font-mono text-foreground dark:text-foreground-dark">
                #{trade.id.slice(0, 8)}
              </Text>
            </View>
          </View>

          {/* Escrow protection note */}
          <View className="mt-3 rounded-2xl bg-success-bg p-3.5 flex-row items-center gap-2.5">
            <ShieldCheck size={16} color="#10B981" />
            <Text className="text-xs text-success flex-1">
              Crypto held in custodial escrow — neither party can move funds until release.
            </Text>
          </View>

          {releaseError ? (
            <View className="mt-3 rounded-lg bg-error-bg px-4 py-3">
              <Text className="text-sm text-error">{releaseError}</Text>
            </View>
          ) : null}

          {/* Chat */}
          <TouchableOpacity
            onPress={() => router.push({ pathname: "/(app)/trade-chat/[id]", params: { id: trade.id } })}
            className="mt-3 rounded-xl border border-border dark:border-border-dark bg-surface dark:bg-card-dark py-3.5 flex-row items-center justify-center gap-2"
            activeOpacity={0.85}
          >
            <MessageCircle size={16} color="#00A3F6" />
            <Text className="text-sm font-semibold text-brand">Open chat</Text>
          </TouchableOpacity>

          {/* Buyer: mark paid */}
          {trade.role === "buyer" && trade.status === "funded" ? (
            <TouchableOpacity
              onPress={handleMarkPaid}
              disabled={markingPaid}
              className="rounded-xl bg-brand py-4 items-center mt-3"
              activeOpacity={0.85}
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
              className="rounded-xl border border-brand bg-brand-bg py-3.5 flex-row items-center justify-center gap-2 mt-3"
              activeOpacity={0.85}
            >
              <Upload size={16} color="#00A3F6" />
              <Text className="text-sm font-semibold text-brand">Upload proof of payment</Text>
            </TouchableOpacity>
          ) : null}

          {/* Seller: release */}
          {trade.role === "seller" && trade.status === "payment_confirmed" ? (
            <TouchableOpacity
              onPress={handleRelease}
              disabled={releasing}
              className="rounded-xl bg-success py-4 items-center mt-3"
              activeOpacity={0.85}
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
              className="rounded-xl border border-error/40 py-3.5 flex-row items-center justify-center gap-2 mt-3"
              activeOpacity={0.85}
            >
              <X size={16} color="#EF4444" />
              <Text className="text-sm font-semibold text-error">Cancel trade</Text>
            </TouchableOpacity>
          ) : null}

          {/* Dispute */}
          {["payment_sent", "payment_confirmed"].includes(trade.status) ? (
            <TouchableOpacity
              onPress={() =>
                router.push({ pathname: "/(app)/dispute/[id]", params: { id: trade.id } })
              }
              className="rounded-xl border border-error/40 bg-error-bg py-3.5 flex-row items-center justify-center gap-2 mt-3"
              activeOpacity={0.85}
            >
              <AlertTriangle size={16} color="#EF4444" />
              <Text className="text-sm font-semibold text-error">Open dispute</Text>
            </TouchableOpacity>
          ) : null}

          {/* Rate counterparty for completed trade */}
          {trade.status === "completed" ? (
            <TouchableOpacity
              onPress={() => setShowRating(true)}
              className="rounded-xl border border-border dark:border-border-dark py-3.5 flex-row items-center justify-center gap-2 mt-3"
              activeOpacity={0.85}
            >
              <Star size={16} color="#F59E0B" />
              <Text className="text-sm font-semibold text-foreground dark:text-foreground-dark">
                Rate {trade.counterparty.username}
              </Text>
            </TouchableOpacity>
          ) : null}

          <View className="h-8" />
        </View>
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
