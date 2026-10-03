import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { tradeService } from "@/src/services/trade.service"

export function useActiveTrades() {
  return useQuery({
    queryKey: ["trades-active"],
    queryFn: () => tradeService.getActive(),
    staleTime: 15_000,
  })
}

export function useTradeHistory(params?: { page?: number; limit?: number }) {
  return useQuery({
    queryKey: ["trades-history", params],
    queryFn: () => tradeService.getHistory(params),
    staleTime: 60_000,
  })
}

export function useTrade(id: string) {
  return useQuery({
    queryKey: ["trade", id],
    queryFn: () => tradeService.getTrade(id),
    staleTime: 10_000,
    enabled: !!id,
  })
}

export function useMarkPaid() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => tradeService.markPaid(id),
    onSuccess: (_, id) => {
      qc.invalidateQueries({ queryKey: ["trade", id] })
      qc.invalidateQueries({ queryKey: ["trades-active"] })
    },
  })
}

export function useReleaseEscrow() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, twoFactorCode }: { id: string; twoFactorCode: string }) =>
      tradeService.releaseEscrow(id, twoFactorCode),
    onSuccess: (_, { id }) => {
      qc.invalidateQueries({ queryKey: ["trade", id] })
      qc.invalidateQueries({ queryKey: ["trades-active"] })
      qc.invalidateQueries({ queryKey: ["wallets"] })
    },
  })
}

export function useCancelTrade() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, reason }: { id: string; reason?: string }) =>
      tradeService.cancelTrade(id, reason),
    onSuccess: (_, { id }) => {
      qc.invalidateQueries({ queryKey: ["trade", id] })
      qc.invalidateQueries({ queryKey: ["trades-active"] })
    },
  })
}

export function useOpenDispute() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      id,
      reason,
      evidenceUrls,
    }: {
      id: string
      reason: string
      evidenceUrls: string[]
    }) => tradeService.openDispute(id, reason, evidenceUrls),
    onSuccess: (_, { id }) => qc.invalidateQueries({ queryKey: ["trade", id] }),
  })
}

export function useTradeMessages(tradeId: string) {
  return useQuery({
    queryKey: ["trade-messages", tradeId],
    queryFn: () => tradeService.getMessages(tradeId),
    staleTime: 5_000,
    enabled: !!tradeId,
  })
}

export function useSendMessage() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      tradeId,
      content,
      attachmentUrl,
    }: {
      tradeId: string
      content: string
      attachmentUrl?: string
    }) => tradeService.sendMessage(tradeId, content, attachmentUrl),
    onSuccess: (_, { tradeId }) =>
      qc.invalidateQueries({ queryKey: ["trade-messages", tradeId] }),
  })
}
