import "react-native-gesture-handler/jestSetup"

// Mock expo-secure-store
jest.mock("expo-secure-store", () => ({
  getItemAsync: jest.fn().mockResolvedValue(null),
  setItemAsync: jest.fn().mockResolvedValue(undefined),
  deleteItemAsync: jest.fn().mockResolvedValue(undefined),
}))

// Mock expo-router
jest.mock("expo-router", () => ({
  useRouter: () => ({
    push: jest.fn(),
    replace: jest.fn(),
    back: jest.fn(),
  }),
  useLocalSearchParams: jest.fn().mockReturnValue({}),
  useSegments: jest.fn().mockReturnValue([]),
  Link: "Link",
  router: {
    push: jest.fn(),
    replace: jest.fn(),
    back: jest.fn(),
  },
}))

// Mock expo-local-authentication
jest.mock("expo-local-authentication", () => ({
  hasHardwareAsync: jest.fn().mockResolvedValue(false),
  isEnrolledAsync: jest.fn().mockResolvedValue(false),
  supportedAuthenticationTypesAsync: jest.fn().mockResolvedValue([]),
  authenticateAsync: jest.fn().mockResolvedValue({ success: false }),
  AuthenticationType: { FINGERPRINT: 1, FACIAL_RECOGNITION: 2, IRIS: 3 },
}))

// Mock expo-notifications
jest.mock("expo-notifications", () => ({
  getPermissionsAsync: jest.fn().mockResolvedValue({ status: "denied" }),
  requestPermissionsAsync: jest.fn().mockResolvedValue({ status: "denied" }),
  setNotificationHandler: jest.fn(),
  addNotificationResponseReceivedListener: jest.fn().mockReturnValue({ remove: jest.fn() }),
  addNotificationReceivedListener: jest.fn().mockReturnValue({ remove: jest.fn() }),
  getLastNotificationResponseAsync: jest.fn().mockResolvedValue(null),
  getExpoPushTokenAsync: jest.fn().mockResolvedValue({ data: "ExponentPushToken[test]" }),
  AndroidImportance: { MAX: 5, HIGH: 4 },
  setNotificationChannelAsync: jest.fn(),
}))

// Mock expo-device
jest.mock("expo-device", () => ({
  isDevice: false,
  modelName: "Jest Test Device",
}))

// Mock @react-native-async-storage/async-storage
jest.mock("@react-native-async-storage/async-storage", () =>
  require("@react-native-async-storage/async-storage/jest/async-storage-mock")
)

// Mock react-native-webview
jest.mock("react-native-webview", () => {
  const { View } = require("react-native")
  return { WebView: View }
})

// Suppress noisy console warnings in tests
const SUPPRESSED = [
  "Warning: An update to",
  "Warning: ReactDOM.render",
  "Warning: Each child in a list",
]
const originalWarn = console.warn
console.warn = (...args: Parameters<typeof console.warn>) => {
  if (typeof args[0] === "string" && SUPPRESSED.some((s) => (args[0] as string).includes(s))) return
  originalWarn.apply(console, args)
}
