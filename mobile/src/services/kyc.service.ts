import { apiClient } from "@/src/lib/api/client"

export type KycTier = 0 | 1 | 2 | 3

export interface KycStatus {
  tier: KycTier
  status: "not_started" | "pending" | "approved" | "rejected" | "requires_action"
  provider: "didit" | "sumsub" | null
  rejectionReason: string | null
  submittedAt: string | null
  approvedAt: string | null
}

export interface KycSessionUrl {
  url: string
  sessionId: string
}

export const kycService = {
  async getStatus(): Promise<KycStatus> {
    return apiClient.get("/api/v1/kyc/status")
  },

  async startDiditSession(): Promise<KycSessionUrl> {
    return apiClient.post("/api/v1/kyc/didit/start")
  },

  async startSumsubSession(): Promise<KycSessionUrl> {
    return apiClient.post("/api/v1/kyc/sumsub/start")
  },

  async uploadDocument(formData: FormData): Promise<{ success: boolean }> {
    return apiClient.upload("/api/v1/kyc/documents", formData)
  },
}
