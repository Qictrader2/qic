import { View, Text } from "react-native"
import { ShieldCheck, AlertCircle, CheckCircle2, Clock, XCircle, AlertTriangle } from "lucide-react-native"
import type { TradeStatus } from "@/src/services/trade.service"

/**
 * Mirrors web's TradeStatusBanner — full-width color-coded banner that
 * sits at the top of /trade/[id] with an icon, status label, and short
 * description of what's happening.
 */
export function TradeStatusBanner({ status, role }: { status: TradeStatus; role: "buyer" | "seller" }) {
  const meta = bannerMeta(status, role)

  return (
    <View
      className="px-4 py-3.5 flex-row items-center gap-3"
      style={{ backgroundColor: meta.bg }}
    >
      <View className="w-9 h-9 rounded-full items-center justify-center" style={{ backgroundColor: meta.iconBg }}>
        <meta.Icon size={18} color={meta.color} />
      </View>
      <View className="flex-1">
        <Text className="text-sm font-semibold" style={{ color: meta.color }}>
          {meta.title}
        </Text>
        <Text className="text-xs mt-0.5" style={{ color: meta.color, opacity: 0.8 }}>
          {meta.body}
        </Text>
      </View>
    </View>
  )
}

function bannerMeta(status: TradeStatus, role: "buyer" | "seller") {
  const isBuyer = role === "buyer"

  switch (status) {
    case "initiated":
      return {
        Icon: Clock,
        color: "#3B82F6",
        bg: "rgba(59, 130, 246, 0.08)",
        iconBg: "rgba(59, 130, 246, 0.15)",
        title: "Trade initiated",
        body: "Waiting for escrow to be funded.",
      }
    case "funded":
      return {
        Icon: ShieldCheck,
        color: "#F59E0B",
        bg: "rgba(245, 158, 11, 0.08)",
        iconBg: "rgba(245, 158, 11, 0.15)",
        title: "Crypto in escrow",
        body: isBuyer
          ? "Send the fiat payment off-platform and mark as paid."
          : "Awaiting payment from the buyer.",
      }
    case "payment_pending":
      return {
        Icon: Clock,
        color: "#F59E0B",
        bg: "rgba(245, 158, 11, 0.08)",
        iconBg: "rgba(245, 158, 11, 0.15)",
        title: "Awaiting payment",
        body: isBuyer ? "Complete the payment now to keep the trade." : "Waiting for buyer to send funds.",
      }
    case "payment_sent":
      return {
        Icon: ShieldCheck,
        color: "#8B5CF6",
        bg: "rgba(139, 92, 246, 0.08)",
        iconBg: "rgba(139, 92, 246, 0.15)",
        title: "Payment sent",
        body: isBuyer
          ? "Wait for the seller to confirm receipt and release."
          : "Confirm receipt before releasing the crypto.",
      }
    case "payment_confirmed":
      return {
        Icon: CheckCircle2,
        color: "#10B981",
        bg: "rgba(16, 185, 129, 0.08)",
        iconBg: "rgba(16, 185, 129, 0.15)",
        title: "Payment confirmed",
        body: isBuyer
          ? "Seller is releasing the crypto."
          : "Release the crypto to complete the trade.",
      }
    case "completed":
      return {
        Icon: CheckCircle2,
        color: "#10B981",
        bg: "rgba(16, 185, 129, 0.08)",
        iconBg: "rgba(16, 185, 129, 0.15)",
        title: "Trade completed",
        body: "Funds have been released. Don't forget to rate your counterparty.",
      }
    case "cancelled":
      return {
        Icon: XCircle,
        color: "#6B7280",
        bg: "rgba(107, 114, 128, 0.08)",
        iconBg: "rgba(107, 114, 128, 0.15)",
        title: "Trade cancelled",
        body: "This trade was cancelled. Any locked funds have been released.",
      }
    case "expired":
      return {
        Icon: XCircle,
        color: "#6B7280",
        bg: "rgba(107, 114, 128, 0.08)",
        iconBg: "rgba(107, 114, 128, 0.15)",
        title: "Trade expired",
        body: "The payment window passed before payment was received.",
      }
    case "disputed":
      return {
        Icon: AlertTriangle,
        color: "#EF4444",
        bg: "rgba(239, 68, 68, 0.08)",
        iconBg: "rgba(239, 68, 68, 0.15)",
        title: "Dispute open",
        body: "A moderator is reviewing this trade. Upload evidence in chat.",
      }
    default: {
      const _exhaustive: never = status
      return {
        Icon: AlertCircle,
        color: "#6B7280",
        bg: "rgba(107, 114, 128, 0.08)",
        iconBg: "rgba(107, 114, 128, 0.15)",
        title: "Unknown status",
        body: _exhaustive,
      }
    }
  }
}
