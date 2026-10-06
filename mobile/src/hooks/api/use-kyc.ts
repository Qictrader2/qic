import { useQuery } from "@tanstack/react-query"
import { kycService } from "@/src/services/kyc.service"

export function useKycStatus() {
  return useQuery({
    queryKey: ["kyc-status"],
    queryFn: () => kycService.getStatus(),
    staleTime: 60_000,
  })
}
