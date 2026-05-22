import axios, { AxiosError } from "axios"
import { secureStorage } from "@/src/lib/storage/secure"

export type ApiError =
  | { kind: "network" }
  | { kind: "unauthorized" }
  | { kind: "forbidden" }
  | { kind: "not_found" }
  | { kind: "validation"; fields: Record<string, string> }
  | { kind: "rate_limited" }
  | { kind: "server"; status: number; message: string }
  | { kind: "kyc_required" }
  | { kind: "unknown"; message: string }

function toApiError(error: unknown): ApiError {
  if (error instanceof AxiosError) {
    if (!error.response) return { kind: "network" }
    const status = error.response.status
    const data = error.response.data as Record<string, unknown> | undefined
    const message =
      (data?.message as string) ??
      (data?.error as string) ??
      error.message ??
      "Unknown error"

    if (status === 401) return { kind: "unauthorized" }
    if (status === 403) {
      if (message.toLowerCase().includes("kyc")) return { kind: "kyc_required" }
      return { kind: "forbidden" }
    }
    if (status === 404) return { kind: "not_found" }
    if (status === 422 || status === 400) {
      const fields = (data?.details as Record<string, string>) ?? {}
      return { kind: "validation", fields }
    }
    if (status === 429) return { kind: "rate_limited" }
    return { kind: "server", status, message }
  }
  return { kind: "unknown", message: String(error) }
}

let _accessToken: string | null = null
let _isRefreshing = false
let _refreshQueue: Array<(token: string | null) => void> = []

const instance = axios.create({
  baseURL: process.env.EXPO_PUBLIC_API_URL,
  timeout: 30_000,
  headers: { "Content-Type": "application/json" },
})

instance.interceptors.request.use(async (config) => {
  const token = _accessToken ?? (await secureStorage.get("qic_access"))
  if (token) {
    config.headers["Authorization"] = `Bearer ${token}`
  }
  return config
})

instance.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const original = error.config as typeof error.config & { _retry?: boolean }

    if (error.response?.status === 401 && !original?._retry) {
      if (_isRefreshing) {
        return new Promise((resolve, reject) => {
          _refreshQueue.push((token) => {
            if (!token || !original) {
              reject(error)
              return
            }
            original.headers["Authorization"] = `Bearer ${token}`
            original._retry = true
            resolve(instance(original))
          })
        })
      }

      _isRefreshing = true
      try {
        const refreshToken = await secureStorage.get("qic_refresh")
        if (!refreshToken) throw new Error("no refresh token")

        const res = await axios.post(
          `${process.env.EXPO_PUBLIC_API_URL}/api/v1/auth/refresh-token`,
          { refreshToken },
          { timeout: 10_000 }
        )

        const newAccess: string = res.data.accessToken ?? res.data.token
        await secureStorage.set("qic_access", newAccess)
        _accessToken = newAccess

        _refreshQueue.forEach((cb) => cb(newAccess))
        _refreshQueue = []

        if (original) {
          original._retry = true
          original.headers["Authorization"] = `Bearer ${newAccess}`
          return instance(original)
        }
      } catch {
        _refreshQueue.forEach((cb) => cb(null))
        _refreshQueue = []
        await secureStorage.clearAll()
        _accessToken = null
        // Navigation to login handled by auth store listener
      } finally {
        _isRefreshing = false
      }
    }

    return Promise.reject(toApiError(error))
  }
)

export const apiClient = {
  setToken(token: string | null) {
    _accessToken = token
  },

  async get<T>(url: string, params?: Record<string, unknown>): Promise<T> {
    const res = await instance.get<T>(url, { params })
    return res.data
  },

  async post<T>(url: string, data?: unknown): Promise<T> {
    const res = await instance.post<T>(url, data)
    return res.data
  },

  async put<T>(url: string, data?: unknown): Promise<T> {
    const res = await instance.put<T>(url, data)
    return res.data
  },

  async patch<T>(url: string, data?: unknown): Promise<T> {
    const res = await instance.patch<T>(url, data)
    return res.data
  },

  async delete<T>(url: string): Promise<T> {
    const res = await instance.delete<T>(url)
    return res.data
  },

  async upload<T>(url: string, formData: FormData): Promise<T> {
    const res = await instance.post<T>(url, formData, {
      headers: { "Content-Type": "multipart/form-data" },
      timeout: 120_000,
    })
    return res.data
  },
}
