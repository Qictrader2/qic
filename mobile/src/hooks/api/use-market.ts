import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { marketService, OfferFilters, CreateOfferRequest } from "@/src/services/market.service"

export function useOffers(filters?: OfferFilters) {
  return useQuery({
    queryKey: ["offers", filters],
    queryFn: () => marketService.getOffers(filters),
    staleTime: 15_000,
  })
}

export function useOffer(id: string) {
  return useQuery({
    queryKey: ["offer", id],
    queryFn: () => marketService.getOffer(id),
    staleTime: 30_000,
    enabled: !!id,
  })
}

export function useMyOffers() {
  return useQuery({
    queryKey: ["my-offers"],
    queryFn: () => marketService.getMyOffers(),
    staleTime: 30_000,
  })
}

export function useCreateOffer() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (req: CreateOfferRequest) => marketService.createOffer(req),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["offers"] })
      qc.invalidateQueries({ queryKey: ["my-offers"] })
    },
  })
}

export function usePauseOffer() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => marketService.pauseOffer(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["my-offers"] }),
  })
}

export function useDeleteOffer() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => marketService.deleteOffer(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["my-offers"] }),
  })
}

export function useInitiateTrade() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ offerId, amount }: { offerId: string; amount: string }) =>
      marketService.initiateTrade(offerId, amount),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["trades-active"] }),
  })
}
