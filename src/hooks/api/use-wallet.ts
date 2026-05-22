import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { walletService, WithdrawRequest } from "@/src/services/wallet.service"

export function useWallets() {
  return useQuery({
    queryKey: ["wallets"],
    queryFn: () => walletService.getAll(),
    staleTime: 30_000,
  })
}

export function useDepositAddress(currency: string, network: string, enabled = true) {
  return useQuery({
    queryKey: ["deposit-address", currency, network],
    queryFn: () =>
      walletService.getDepositAddress(
        currency as Parameters<typeof walletService.getDepositAddress>[0],
        network as Parameters<typeof walletService.getDepositAddress>[1]
      ),
    enabled: enabled && !!currency && !!network,
    staleTime: 60_000 * 10,
  })
}

export function useWithdrawFeePreview(currency: string, network: string, amount: string) {
  return useQuery({
    queryKey: ["withdraw-fee", currency, network, amount],
    queryFn: () =>
      walletService.previewWithdrawFee(
        currency as Parameters<typeof walletService.previewWithdrawFee>[0],
        network as Parameters<typeof walletService.previewWithdrawFee>[1],
        amount
      ),
    enabled: !!currency && !!network && parseFloat(amount) > 0,
    staleTime: 15_000,
  })
}

export function useWithdraw() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (req: WithdrawRequest) => walletService.withdraw(req),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["wallets"] })
      qc.invalidateQueries({ queryKey: ["transactions"] })
    },
  })
}

export function useTransactions(filters?: Parameters<typeof walletService.getTransactions>[0]) {
  return useQuery({
    queryKey: ["transactions", filters],
    queryFn: () => walletService.getTransactions(filters),
    staleTime: 30_000,
  })
}
