import { kycService, KycStatus } from "@/src/services/kyc.service"
import { walletService, Wallet } from "@/src/services/wallet.service"
import { tradeService, Trade } from "@/src/services/trade.service"
import { marketService, Offer } from "@/src/services/market.service"

// Mock the API client
jest.mock("@/src/lib/api/client", () => ({
  apiClient: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    patch: jest.fn(),
    delete: jest.fn(),
    upload: jest.fn(),
    setToken: jest.fn(),
  },
}))

import { apiClient } from "@/src/lib/api/client"
const mockGet = apiClient.get as jest.Mock
const mockPost = apiClient.post as jest.Mock
const mockDelete = apiClient.delete as jest.Mock

afterEach(() => jest.clearAllMocks())

describe("kycService", () => {
  test("getStatus calls correct endpoint and returns typed status", async () => {
    const expected: KycStatus = {
      tier: 1,
      status: "approved",
      provider: "didit",
      rejectionReason: null,
      submittedAt: "2026-01-01T00:00:00Z",
      approvedAt: "2026-01-02T00:00:00Z",
    }
    mockGet.mockResolvedValueOnce(expected)
    const result = await kycService.getStatus()
    expect(mockGet).toHaveBeenCalledWith("/api/v1/kyc/status")
    expect(result.tier).toBe(1)
    expect(result.status).toBe("approved")
  })

  test("startDiditSession returns url and sessionId", async () => {
    const expected = { url: "https://verify.didit.me/session/abc", sessionId: "abc" }
    mockPost.mockResolvedValueOnce(expected)
    const result = await kycService.startDiditSession()
    expect(mockPost).toHaveBeenCalledWith("/api/v1/kyc/didit/start")
    expect(result.url).toBe(expected.url)
    expect(result.sessionId).toBe("abc")
  })
})

describe("walletService", () => {
  test("getAll returns wallet array", async () => {
    const wallets: Wallet[] = [
      {
        id: "w1",
        currency: "USDT",
        network: "tron",
        balance: "100.00",
        lockedBalance: "0.00",
        availableBalance: "100.00",
        depositAddress: "TRX123",
        depositAddressQr: "data:image/png;base64,abc",
      },
    ]
    mockGet.mockResolvedValueOnce(wallets)
    const result = await walletService.getAll()
    expect(mockGet).toHaveBeenCalledWith("/api/v1/wallet")
    expect(result).toHaveLength(1)
    expect(result[0]?.currency).toBe("USDT")
    expect(result[0]?.availableBalance).toBe("100.00")
  })

  test("withdraw posts correct payload", async () => {
    mockPost.mockResolvedValueOnce({ transactionId: "tx123" })
    const result = await walletService.withdraw({
      currency: "USDT",
      network: "tron",
      toAddress: "TRX456",
      amount: "50.00",
      twoFactorCode: "123456",
    })
    expect(mockPost).toHaveBeenCalledWith("/api/v1/wallet/withdraw", {
      currency: "USDT",
      network: "tron",
      toAddress: "TRX456",
      amount: "50.00",
      twoFactorCode: "123456",
    })
    expect(result.transactionId).toBe("tx123")
  })
})

describe("tradeService", () => {
  test("getActive returns trade array", async () => {
    mockGet.mockResolvedValueOnce([])
    const result = await tradeService.getActive()
    expect(mockGet).toHaveBeenCalledWith("/api/v1/trades/active")
    expect(Array.isArray(result)).toBe(true)
  })

  test("markPaid posts to correct endpoint", async () => {
    const trade = { id: "t1", status: "payment_sent" } as Partial<Trade>
    mockPost.mockResolvedValueOnce(trade)
    const result = await tradeService.markPaid("t1")
    expect(mockPost).toHaveBeenCalledWith("/api/v1/trades/t1/mark-paid")
    expect(result.id).toBe("t1")
  })

  test("releaseEscrow requires twoFactorCode", async () => {
    mockPost.mockResolvedValueOnce({ id: "t1", status: "completed" })
    await tradeService.releaseEscrow("t1", "654321")
    expect(mockPost).toHaveBeenCalledWith("/api/v1/trades/t1/release", { twoFactorCode: "654321" })
  })
})

describe("marketService", () => {
  test("getOffers passes filters as query params", async () => {
    mockGet.mockResolvedValueOnce([])
    await marketService.getOffers({ offerType: "buy", currency: "BTC" })
    expect(mockGet).toHaveBeenCalledWith("/api/v1/offers", { offerType: "buy", currency: "BTC" })
  })

  test("initiateTrade returns tradeId", async () => {
    mockPost.mockResolvedValueOnce({ tradeId: "trade-abc" })
    const result = await marketService.initiateTrade("offer-123", "0.05")
    expect(mockPost).toHaveBeenCalledWith("/api/v1/trades", { offerId: "offer-123", amount: "0.05" })
    expect(result.tradeId).toBe("trade-abc")
  })
})
