import * as SecureStore from "expo-secure-store"

type SecureKey = "qic_access" | "qic_refresh" | "qic_csrf" | "qic_biometric_enabled"

export const secureStorage = {
  async get(key: SecureKey): Promise<string | null> {
    return SecureStore.getItemAsync(key)
  },

  async set(key: SecureKey, value: string): Promise<void> {
    await SecureStore.setItemAsync(key, value, {
      keychainAccessible: SecureStore.WHEN_UNLOCKED_THIS_DEVICE_ONLY,
    })
  },

  async clear(key: SecureKey): Promise<void> {
    await SecureStore.deleteItemAsync(key)
  },

  async clearAll(): Promise<void> {
    await Promise.all([
      SecureStore.deleteItemAsync("qic_access"),
      SecureStore.deleteItemAsync("qic_refresh"),
      SecureStore.deleteItemAsync("qic_csrf"),
      SecureStore.deleteItemAsync("qic_biometric_enabled"),
    ])
  },
}
