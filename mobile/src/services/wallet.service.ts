import { apiClient } from "@/src/lib/api/client"

export type Currency = "BTC" | "ETH" | "SOL" | "USDT" | "USDC"
export type Network = "bitcoin" | "ethereum" | "solana" | "tron" | "polygon" | "erc20" | "trc20" | "spl"
export type TxStatus = "pending" | "confirming" | "confirmed" | "failed"
export type TxType = "deposit" | "withdrawal" | "transfer" | "trade_lock" | "trade_release"

export interface Wallet {
  id: string
  currency: Currency
  network: Network
  balance: string
  lockedBalance: string
  availableBalance: string
  depositAddress: string
  depositAddressQr: string
}

export interface Transaction {
  id: string
  type: TxType
  currency: Currency
  network: Network
  amount: string
  fee: string
  status: TxStatus
  txHash: string | null
  fromAddress: string | null
  toAddress: string | null
  createdAt: string
  confirmedAt: string | null
  notes: string | null
}

export interface WithdrawRequest {
  currency: Currency
  network: Network
  toAddress: string
  amount: string
  twoFactorCode: string
}

export interface WithdrawFeePreview {
  fee: string
  netAmount: string
  estimatedArrival: string
}

export interface DepositAddress {
  address: string
  qrCode: string
  network: Network
  currency: Currency
  minDeposit: string
  confirmationsRequired: number
}

export const walletService = {
  async getAll(): Promise<Wallet[]> {
    return apiClient.get("/api/v1/wallet")
  },

  async getDepositAddress(currency: Currency, network: Network): Promise<DepositAddress> {
    return apiClient.get(`/api/v1/wallet/deposit-address`, { currency, network })
  },

  async previewWithdrawFee(
    currency: Currency,
    network: Network,
    amount: string
  ): Promise<WithdrawFeePreview> {
    return apiClient.get("/api/v1/wallet/withdraw/fee-preview", { currency, network, amount })
  },

  async withdraw(req: WithdrawRequest): Promise<{ transactionId: string }> {
    return apiClient.post("/api/v1/wallet/withdraw", req)
  },

  async getTransactions(params?: {
    currency?: Currency
    type?: TxType
    page?: number
    limit?: number
  }): Promise<Transaction[]> {
    return apiClient.get("/api/v1/wallet/transactions", params as Record<string, unknown>)
  },

  async internalTransfer(params: {
    fromCurrency: Currency
    toCurrency: Currency
    amount: string
  }): Promise<{ success: boolean }> {
    return apiClient.post("/api/v1/wallet/transfer", params)
  },
}
