/**
 * Platform config — fetches feature flags from backend once per session.
 * Mirrors web's config/feature-flags.ts + GET /api/v1/config/public.
 */
import { apiClient } from "@/src/lib/api/client"
import AsyncStorage from "@react-native-async-storage/async-storage"

const CONFIG_CACHE_KEY = "qic_platform_config"
const CONFIG_TTL_MS = 5 * 60_000 // 5 minutes

export interface PlatformConfig {
  enableNgn: boolean
  resellFeatureEnabled: boolean
  solanaFeatureEnabled: boolean
  maintenanceMode: boolean
  minWithdrawAmounts: Record<string, string>
  maxWithdrawAmounts: Record<string, string>
  depositNetworks: Record<string, string[]>
  supportedCurrencies: string[]
}

const DEFAULT_CONFIG: PlatformConfig = {
  enableNgn: false,
  resellFeatureEnabled: true,
  solanaFeatureEnabled: false,
  maintenanceMode: false,
  minWithdrawAmounts: { BTC: "0.0001", ETH: "0.01", USDT: "10", SOL: "0.1" },
  maxWithdrawAmounts: { BTC: "10", ETH: "100", USDT: "50000", SOL: "1000" },
  depositNetworks: {
    BTC: ["bitcoin"],
    ETH: ["erc20"],
    USDT: ["erc20", "trc20", "spl"],
    SOL: ["solana"],
  },
  supportedCurrencies: ["BTC", "ETH", "USDT", "SOL"],
}

let inMemoryConfig: PlatformConfig | null = null
let lastFetch = 0

export async function getPlatformConfig(): Promise<PlatformConfig> {
  const now = Date.now()
  if (inMemoryConfig && now - lastFetch < CONFIG_TTL_MS) return inMemoryConfig

  try {
    const raw = await apiClient.get<Partial<PlatformConfig>>("/api/v1/config/public")
    inMemoryConfig = { ...DEFAULT_CONFIG, ...raw }
    lastFetch = now
    await AsyncStorage.setItem(CONFIG_CACHE_KEY, JSON.stringify({ config: inMemoryConfig, ts: now }))
    return inMemoryConfig
  } catch {
    // Fallback to cached or default
    try {
      const cached = await AsyncStorage.getItem(CONFIG_CACHE_KEY)
      if (cached) {
        const { config } = JSON.parse(cached) as { config: PlatformConfig; ts: number }
        inMemoryConfig = config
        return config
      }
    } catch {}
    return DEFAULT_CONFIG
  }
}

export function usePlatformConfig() {
  const { useQuery } = require("@tanstack/react-query")
  return useQuery({
    queryKey: ["platform-config"],
    queryFn: getPlatformConfig,
    staleTime: CONFIG_TTL_MS,
    gcTime: 10 * 60_000,
  }) as { data: PlatformConfig | undefined }
}
