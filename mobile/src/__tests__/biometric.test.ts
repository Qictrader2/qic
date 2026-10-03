import { promptBiometric, isBiometricAvailable, getBiometricType } from "@/src/lib/biometric"

jest.mock("expo-local-authentication", () => ({
  AuthenticationType: {
    FACIAL_RECOGNITION: 2,
    FINGERPRINT: 1,
    IRIS: 3,
  },
  supportedAuthenticationTypesAsync: jest.fn(),
  hasHardwareAsync: jest.fn(),
  isEnrolledAsync: jest.fn(),
  authenticateAsync: jest.fn(),
}))

import * as LocalAuthentication from "expo-local-authentication"
const mockSupported = LocalAuthentication.supportedAuthenticationTypesAsync as jest.Mock
const mockHasHardware = LocalAuthentication.hasHardwareAsync as jest.Mock
const mockIsEnrolled = LocalAuthentication.isEnrolledAsync as jest.Mock
const mockAuthenticate = LocalAuthentication.authenticateAsync as jest.Mock

jest.mock("@/src/lib/storage/secure", () => ({
  secureStorage: {
    get: jest.fn().mockResolvedValue(null),
    set: jest.fn().mockResolvedValue(undefined),
    clear: jest.fn().mockResolvedValue(undefined),
    clearAll: jest.fn().mockResolvedValue(undefined),
  },
}))

afterEach(() => jest.clearAllMocks())

describe("biometric", () => {
  test("getBiometricType returns 'facial' when face ID is available", async () => {
    mockSupported.mockResolvedValueOnce([
      LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION,
    ])
    const type = await getBiometricType()
    expect(type).toBe("facial")
  })

  test("getBiometricType returns 'fingerprint' when touch ID is available", async () => {
    mockSupported.mockResolvedValueOnce([
      LocalAuthentication.AuthenticationType.FINGERPRINT,
    ])
    const type = await getBiometricType()
    expect(type).toBe("fingerprint")
  })

  test("getBiometricType returns 'none' when no biometrics", async () => {
    mockSupported.mockResolvedValueOnce([])
    const type = await getBiometricType()
    expect(type).toBe("none")
  })

  test("isBiometricAvailable returns true when hardware + enrolled", async () => {
    mockHasHardware.mockResolvedValueOnce(true)
    mockIsEnrolled.mockResolvedValueOnce(true)
    const available = await isBiometricAvailable()
    expect(available).toBe(true)
  })

  test("isBiometricAvailable returns false when not enrolled", async () => {
    mockHasHardware.mockResolvedValueOnce(true)
    mockIsEnrolled.mockResolvedValueOnce(false)
    const available = await isBiometricAvailable()
    expect(available).toBe(false)
  })

  test("promptBiometric returns true on success", async () => {
    mockHasHardware.mockResolvedValueOnce(true)
    mockIsEnrolled.mockResolvedValueOnce(true)
    mockAuthenticate.mockResolvedValueOnce({ success: true })
    const result = await promptBiometric("Test")
    expect(result).toBe(true)
  })

  test("promptBiometric returns false on failure", async () => {
    mockHasHardware.mockResolvedValueOnce(true)
    mockIsEnrolled.mockResolvedValueOnce(true)
    mockAuthenticate.mockResolvedValueOnce({ success: false })
    const result = await promptBiometric("Test")
    expect(result).toBe(false)
  })

  test("promptBiometric returns false when biometric not available", async () => {
    mockHasHardware.mockResolvedValueOnce(false)
    mockIsEnrolled.mockResolvedValueOnce(false)
    const result = await promptBiometric("Test")
    expect(result).toBe(false)
  })
})
