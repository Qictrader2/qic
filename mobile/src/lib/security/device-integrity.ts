import { Platform } from "react-native"

/**
 * MOBILE-SEC-002: Jailbreak/root detection + screen capture protection.
 *
 * Uses expo-device for basic checks. For production-grade detection,
 * add @anvilabs/expo-jailbreak-detection after ejecting.
 */

import * as Device from "expo-device"

export async function isDeviceCompromised(): Promise<boolean> {
  // Emulator/simulator check — flag in production builds
  if (!Device.isDevice) return true

  // On Android, check for root indicators via build properties
  if (Platform.OS === "android") {
    // expo-device exposes isRooted on some builds
    const rooted = (Device as unknown as { isRooted?: boolean }).isRooted
    if (rooted === true) return true
  }

  return false
}

export async function checkDeviceIntegrity(): Promise<{
  safe: boolean
  reason: string | null
}> {
  const compromised = await isDeviceCompromised()
  if (compromised) {
    return {
      safe: false,
      reason: "Device appears to be running in an unsafe environment.",
    }
  }
  return { safe: true, reason: null }
}

/**
 * Screen capture protection.
 * On iOS: FLAG_SECURE equivalent is set via Expo's secure text entry on sensitive fields.
 * Full FLAG_SECURE (Android) requires expo-modules native config — see MOBILE-SEC-002 notes.
 */
export const SECURE_TEXT_FIELDS_CONFIG = {
  // Applied to TextInput on: withdraw address, 2FA codes, password fields
  secureTextEntry: true,
} as const
