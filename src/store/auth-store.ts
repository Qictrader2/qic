import { create } from "zustand"
import { createJSONStorage, persist } from "zustand/middleware"
import AsyncStorage from "@react-native-async-storage/async-storage"
import { secureStorage } from "@/src/lib/storage/secure"
import { apiClient } from "@/src/lib/api/client"

export type UserRole = "user" | "admin" | "support" | "moderator"

export interface AuthUser {
  uid: string
  email: string
  username: string | null
  displayName: string | null
  emailVerified: boolean
  role: UserRole | null
  kycTier: number
}

interface AuthState {
  user: AuthUser | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
  requiresLegalAcceptance: boolean

  setUser: (user: AuthUser | null) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  setRequiresLegalAcceptance: (value: boolean) => void
  login: (accessToken: string, refreshToken: string, user: AuthUser) => Promise<void>
  logout: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
      requiresLegalAcceptance: false,

      setUser: (user) =>
        set((state) => {
          if (state.user?.uid === user?.uid && state.user?.email === user?.email) {
            return state
          }
          return { user, isAuthenticated: !!user, error: null }
        }),

      setLoading: (isLoading) => set({ isLoading }),
      setError: (error) => set({ error }),
      setRequiresLegalAcceptance: (requiresLegalAcceptance) =>
        set({ requiresLegalAcceptance }),

      login: async (accessToken, refreshToken, user) => {
        await secureStorage.set("qic_access", accessToken)
        await secureStorage.set("qic_refresh", refreshToken)
        apiClient.setToken(accessToken)
        set({ user, isAuthenticated: true, error: null })
      },

      logout: async () => {
        apiClient.setToken(null)
        await secureStorage.clearAll()
        set({
          user: null,
          isAuthenticated: false,
          error: null,
          requiresLegalAcceptance: false,
        })
      },
    }),
    {
      name: "auth-storage",
      storage: createJSONStorage(() => AsyncStorage),
      // Only persist non-sensitive user profile — tokens live in SecureStore
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)
