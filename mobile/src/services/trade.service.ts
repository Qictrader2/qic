import { apiClient } from "@/src/lib/api/client"

export type TradeStatus =
  | "initiated"
  | "funded"
  | "payment_pending"
  | "payment_sent"
  | "payment_confirmed"
  | "completed"
  | "cancelled"
  | "disputed"
  | "expired"

export type TradeRole = "buyer" | "seller"

export interface Trade {
  id: string
  status: TradeStatus
  role: TradeRole
  offerId: string
  currency: string
  fiatCurrency: string
  cryptoAmount: string
  fiatAmount: string
  pricePerUnit: string
  paymentMethod: string
  paymentWindow: number
  expiresAt: string | null
  counterparty: {
    uid: string
    username: string
    kycTier: number
  }
  escrowAddress: string | null
  proofOfPaymentUrl: string | null
  createdAt: string
  updatedAt: string
  disputeReason: string | null
}

export interface TradeMessage {
  id: string
  tradeId: string
  senderId: string
  senderUsername: string
  content: string
  attachmentUrl: string | null
  createdAt: string
}

export const tradeService = {
  async getActive(): Promise<Trade[]> {
    return apiClient.get("/api/v1/trades/active")
  },

  async getHistory(params?: { page?: number; limit?: number }): Promise<Trade[]> {
    return apiClient.get("/api/v1/trades/history", params as Record<string, unknown>)
  },

  async getTrade(id: string): Promise<Trade> {
    return apiClient.get(`/api/v1/trades/${id}`)
  },

  async markPaid(id: string): Promise<Trade> {
    return apiClient.post(`/api/v1/trades/${id}/mark-paid`)
  },

  async releaseEscrow(id: string, twoFactorCode: string): Promise<Trade> {
    return apiClient.post(`/api/v1/trades/${id}/release`, { twoFactorCode })
  },

  async cancelTrade(id: string, reason?: string): Promise<Trade> {
    return apiClient.post(`/api/v1/trades/${id}/cancel`, { reason })
  },

  async openDispute(id: string, reason: string, evidenceUrls: string[]): Promise<Trade> {
    return apiClient.post(`/api/v1/trades/${id}/dispute`, { reason, evidenceUrls })
  },

  async uploadProofOfPayment(id: string, formData: FormData): Promise<{ url: string }> {
    return apiClient.upload(`/api/v1/trades/${id}/proof-of-payment`, formData)
  },

  async getMessages(id: string): Promise<TradeMessage[]> {
    return apiClient.get(`/api/v1/trades/${id}/messages`)
  },

  async sendMessage(
    id: string,
    content: string,
    attachmentUrl?: string
  ): Promise<TradeMessage> {
    return apiClient.post(`/api/v1/trades/${id}/messages`, { content, attachmentUrl })
  },
}
