import { apiClient } from "@/src/lib/api/client"

export interface ResellStats {
  defaultMarkupPercentage: number
  totalEarnings: string
  currency: string
  activeResells: number
  totalTrades: number
  successRate: number
}

export interface ResellTrade {
  id: string
  tradeId: string
  buyerUsername: string
  vendorUsername: string
  cryptocurrency: string
  fiatCurrency: string
  cryptoAmount: string
  fiatAmount: string
  markupPercentage: number
  grossCommission: string
  resellerProfit: string
  amountUsdt: number | null
  displayState: "pending" | "preview" | "contested" | "settled" | "under-review"
  status: "created" | "escrow_funded" | "paid" | "released" | "completed" | "disputed" | "resolved" | "cancelled"
  createdAt: string
  completedAt: string | null
}

export interface ResellOffer {
  id: string
  originalOfferId: string
  resellerId: string
  markup: number
  markupType: "percentage"
  pricePerUnit: string
  status: "active" | "paused" | "completed" | "cancelled"
  minTransaction: string
  maxTransaction: string
  totalTrades: number
  completedTrades: number
  createdAt: string
  cryptocurrency: string
  fiatCurrency: string
  availableAmount: string
  totalProfit: string
}

export const PLATFORM_FEE_RATE = 0.25 // 25% of gross commission

/** Mirror of web's calculateResellerCommission */
export function calculateResellerCommission(
  baseRate: number,
  markup: number,
  amount: number,
  offerType: "buy" | "sell"
): { resoldRate: number; grossCommission: number; netCommission: number } {
  const resoldRate =
    offerType === "sell"
      ? baseRate * (1 + markup / 100)
      : baseRate * (1 - markup / 100)
  const grossCommission =
    offerType === "sell"
      ? amount * (resoldRate - baseRate)
      : amount * (baseRate - resoldRate)
  const netCommission = grossCommission * (1 - PLATFORM_FEE_RATE)
  return { resoldRate, grossCommission, netCommission }
}

export const resellerService = {
  async getStats(): Promise<ResellStats> {
    return apiClient.get<ResellStats>("/api/v1/reseller/stats")
  },

  async getTrades(params?: { status?: string; page?: number; perPage?: number }): Promise<ResellTrade[]> {
    const d = await apiClient.get<unknown>("/api/v1/reseller/trades", params ?? {})
    return Array.isArray(d) ? (d as ResellTrade[]) : ((d as { data: ResellTrade[] }).data ?? [])
  },

  async getActiveResells(): Promise<ResellOffer[]> {
    const d = await apiClient.get<unknown>("/api/v1/reseller/active", {})
    return Array.isArray(d) ? (d as ResellOffer[]) : ((d as { data: ResellOffer[] }).data ?? [])
  },

  async createResell(offerId: string, markupPercentage: number): Promise<{ id: string }> {
    return apiClient.post<{ id: string }>(`/api/v1/reseller/resell/${offerId}`, { markupPercentage })
  },

  async updateDefaultMarkup(markupPercentage: number): Promise<void> {
    await apiClient.patch<void>("/api/v1/reseller/profile/markup", { markupPercentage })
  },
}
