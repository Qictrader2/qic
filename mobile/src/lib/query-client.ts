import { QueryClient } from "@tanstack/react-query"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { createAsyncStoragePersister } from "@tanstack/query-async-storage-persister"
import { persistQueryClient } from "@tanstack/react-query-persist-client"

/** PERF-001: FlashList recommended estimated item heights */
export const LIST_ITEM_HEIGHTS = {
  offer: 96,
  trade: 84,
  transaction: 72,
  notification: 76,
  wallet: 112,
} as const

/** PERF-002: Offline-first stale times */
export const STALE_TIMES = {
  wallets: 30_000,
  offers: 15_000,
  activeTrades: 15_000,
  notifications: 30_000,
  kycStatus: 60_000,
  profile: 5 * 60_000,
  tradeHistory: 60_000,
  transactions: 30_000,
} as const

/** PERF-002: GC times (how long unused data stays in cache) */
export const GC_TIMES = {
  wallets: 5 * 60_000,
  offers: 2 * 60_000,
  activeTrades: 2 * 60_000,
  notifications: 60_000,
  kycStatus: 10 * 60_000,
  profile: 30 * 60_000,
  tradeHistory: 30 * 60_000,
  transactions: 5 * 60_000,
} as const

export function createQueryClient(): QueryClient {
  const client = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: STALE_TIMES.wallets,
        gcTime: GC_TIMES.wallets,
        retry: (failureCount, error) => {
          const apiErr = error as { kind?: string }
          // Don't retry auth errors, forbidden, or not found
          if (
            apiErr?.kind === "unauthorized" ||
            apiErr?.kind === "forbidden" ||
            apiErr?.kind === "not_found"
          ) {
            return false
          }
          return failureCount < 2
        },
        retryDelay: (attempt) => Math.min(1000 * 2 ** attempt, 10_000),
        networkMode: "offlineFirst",
      },
      mutations: {
        networkMode: "online",
      },
    },
  })

  // Persist to AsyncStorage for offline support — guarded so it degrades
  // gracefully in Expo Go where the native module may not be available.
  try {
    const persister = createAsyncStoragePersister({
      storage: AsyncStorage,
      key: "qic-rq-cache",
      throttleTime: 1000,
    })

    persistQueryClient({
      queryClient: client as unknown as Parameters<typeof persistQueryClient>[0]["queryClient"],
      persister,
      maxAge: 24 * 60 * 60_000,
      buster: "v1",
      dehydrateOptions: {
        shouldDehydrateQuery: (query) => {
          const key = query.queryKey[0] as string
          return !["wallets", "transactions"].includes(key)
        },
      },
    })
  } catch {
    // AsyncStorage native module unavailable (Expo Go) — skip persistence
  }

  return client
}
