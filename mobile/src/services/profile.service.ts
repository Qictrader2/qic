import { apiClient } from "@/src/lib/api/client"

export interface UserProfile {
  uid: string
  email: string
  username: string | null
  displayName: string | null
  phone: string | null
  bio: string | null
  avatarUrl: string | null
  country: string | null
  language: string | null
  emailVerified: boolean
  kycTier: number
  tradeCount: number
  completionRate: number
  averageRating: number
  createdAt: string
}

export interface UpdateProfileRequest {
  displayName?: string | undefined
  username?: string | undefined
  phone?: string | undefined
  bio?: string | undefined
  country?: string | undefined
  language?: string | undefined
}

export interface PaymentMethod {
  id: string
  type: string
  label: string
  details: Record<string, string>
  isDefault: boolean
  createdAt: string
}

export interface AffiliateStats {
  referralCode: string
  referralUrl: string
  referredCount: number
  pendingEarnings: string
  totalEarnings: string
  currency: string
  payouts: Array<{
    id: string
    amount: string
    status: string
    createdAt: string
  }>
}

export const profileService = {
  async getProfile(): Promise<UserProfile> {
    return apiClient.get("/api/v1/me")
  },

  async updateProfile(req: UpdateProfileRequest): Promise<UserProfile> {
    return apiClient.patch("/api/v1/me", req)
  },

  async uploadAvatar(formData: FormData): Promise<{ avatarUrl: string }> {
    return apiClient.upload("/api/v1/me/avatar", formData)
  },

  async getPaymentMethods(): Promise<PaymentMethod[]> {
    return apiClient.get("/api/v1/me/payment-methods")
  },

  async addPaymentMethod(
    method: Omit<PaymentMethod, "id" | "createdAt">
  ): Promise<PaymentMethod> {
    return apiClient.post("/api/v1/me/payment-methods", method)
  },

  async deletePaymentMethod(id: string): Promise<{ success: boolean }> {
    return apiClient.delete(`/api/v1/me/payment-methods/${id}`)
  },

  async getAffiliateStats(): Promise<AffiliateStats> {
    return apiClient.get("/api/v1/affiliate/stats")
  },

  async changePassword(req: {
    currentPassword: string
    newPassword: string
    twoFactorCode: string
  }): Promise<{ success: boolean }> {
    return apiClient.post("/api/v1/me/change-password", req)
  },
}
