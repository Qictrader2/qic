import * as LocalAuthentication from "expo-local-authentication"
import { secureStorage } from "@/src/lib/storage/secure"

export type BiometricType = "fingerprint" | "facial" | "iris" | "none"

export async function getBiometricType(): Promise<BiometricType> {
  const types = await LocalAuthentication.supportedAuthenticationTypesAsync()
  if (types.includes(LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION)) {
    return "facial"
  }
  if (types.includes(LocalAuthentication.AuthenticationType.FINGERPRINT)) {
    return "fingerprint"
  }
  if (types.includes(LocalAuthentication.AuthenticationType.IRIS)) {
    return "iris"
  }
  return "none"
}

export async function isBiometricAvailable(): Promise<boolean> {
  const compatible = await LocalAuthentication.hasHardwareAsync()
  const enrolled = await LocalAuthentication.isEnrolledAsync()
  return compatible && enrolled
}

export async function isBiometricEnabled(): Promise<boolean> {
  const val = await secureStorage.get("qic_biometric_enabled")
  return val === "true"
}

export async function setBiometricEnabled(enabled: boolean): Promise<void> {
  await secureStorage.set("qic_biometric_enabled", enabled ? "true" : "false")
}

export async function promptBiometric(reason: string): Promise<boolean> {
  const available = await isBiometricAvailable()
  if (!available) return false

  const result = await LocalAuthentication.authenticateAsync({
    promptMessage: reason,
    cancelLabel: "Cancel",
    disableDeviceFallback: false,
    fallbackLabel: "Use passcode",
  })

  return result.success
}
