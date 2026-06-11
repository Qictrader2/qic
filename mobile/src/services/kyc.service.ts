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
  provider?: "didit" | "sumsub"
}

export const kycService = {
  async getStatus(): Promise<KycStatus> {
    return apiClient.get("/api/v1/kyc/status")
  },

  // Backend exposes one provider-agnostic session endpoint (POST /kyc/session,
  // empty body) that returns { provider: "didit", sessionId, url, vendorData }.
  // The old /kyc/didit/start and /kyc/sumsub/start paths never existed; Sumsub
  // is retired on the web (its token route 500s on prod).
  async startDiditSession(): Promise<KycSessionUrl> {
    return apiClient.post("/api/v1/kyc/session", {})
  },

  async startSumsubSession(): Promise<KycSessionUrl> {
    return apiClient.post("/api/v1/kyc/session", {})
  },

  async uploadDocument(formData: FormData): Promise<{ success: boolean }> {
    return apiClient.upload("/api/v1/kyc/documents", formData)
  },
}
