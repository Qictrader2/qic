import { apiClient } from "@/src/lib/api/client"
import type { Currency, Network } from "./wallet.service"

export type { Currency, Network }

export type OfferType = "buy" | "sell"
export type PaymentMethodType = "bank_transfer" | "cash" | "mobile_money" | "crypto" | "other"
export type OfferStatus = "active" | "paused" | "deleted"

export interface Offer {
  id: string
  offerType: OfferType
  currency: Currency
  fiatCurrency: string
  pricePerUnit: string
  minAmount: string
  maxAmount: string
  availableAmount: string
  paymentMethods: PaymentMethodType[]
  paymentWindow: number
  terms: string | null
  status: OfferStatus
  tradeCount: number
  completionRate: number
  owner: {
    uid: string
    username: string
    kycTier: number
    tradeCount: number
    completionRate: number
  }
  createdAt: string
}

export interface OfferFilters {
  offerType?: OfferType
  currency?: Currency
  fiatCurrency?: string
  paymentMethod?: PaymentMethodType
  minAmount?: string
  maxAmount?: string
  page?: number
  limit?: number
}

export interface CreateOfferRequest {
  offerType: OfferType
  currency: Currency
  fiatCurrency: string
  pricePerUnit: string
  minAmount: string
  maxAmount: string
  totalAmount: string
  paymentMethods: PaymentMethodType[]
  paymentWindow: number
  terms?: string | undefined
}

export const marketService = {
  async getOffers(filters?: OfferFilters): Promise<Offer[]> {
    return apiClient.get("/api/v1/offers", filters as Record<string, unknown>)
  },

  async getOffer(id: string): Promise<Offer> {
    return apiClient.get(`/api/v1/offers/${id}`)
  },

  async getMyOffers(): Promise<Offer[]> {
    return apiClient.get("/api/v1/offers/mine")
  },

  async createOffer(req: CreateOfferRequest): Promise<Offer> {
    return apiClient.post("/api/v1/offers", req)
  },

  async updateOffer(id: string, updates: Partial<CreateOfferRequest>): Promise<Offer> {
    return apiClient.put(`/api/v1/offers/${id}`, updates)
  },

  async pauseOffer(id: string): Promise<{ success: boolean }> {
    return apiClient.post(`/api/v1/offers/${id}/pause`)
  },

  async resumeOffer(id: string): Promise<{ success: boolean }> {
    return apiClient.post(`/api/v1/offers/${id}/resume`)
  },

  async deleteOffer(id: string): Promise<{ success: boolean }> {
    return apiClient.delete(`/api/v1/offers/${id}`)
  },

  async initiateTrade(offerId: string, amount: string): Promise<{ tradeId: string }> {
    return apiClient.post("/api/v1/trades", { offerId, amount })
  },
}
