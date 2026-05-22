import { Stack } from "expo-router"

export default function AppLayout() {
  return (
    <Stack>
      <Stack.Screen name="deposit" options={{ title: "Deposit", presentation: "modal" }} />
      <Stack.Screen name="withdraw" options={{ title: "Withdraw", presentation: "modal" }} />
      <Stack.Screen name="transactions" options={{ title: "Transaction History" }} />
      <Stack.Screen name="offer/[id]" options={{ title: "Offer" }} />
      <Stack.Screen name="create-offer" options={{ title: "Create Offer", presentation: "modal" }} />
      <Stack.Screen name="trade/[id]" options={{ title: "Trade" }} />
      <Stack.Screen name="trade-chat/[id]" options={{ title: "Chat" }} />
      <Stack.Screen name="trade-history" options={{ title: "Trade History" }} />
      <Stack.Screen name="kyc" options={{ title: "Verify Identity" }} />
      <Stack.Screen name="notifications-settings" options={{ title: "Notification Settings" }} />
      <Stack.Screen name="profile-edit" options={{ title: "Edit Profile" }} />
      <Stack.Screen name="security-settings" options={{ title: "Security" }} />
      <Stack.Screen name="payment-methods" options={{ title: "Payment Methods" }} />
      <Stack.Screen name="2fa-setup" options={{ title: "Two-Factor Auth", presentation: "modal" }} />
      <Stack.Screen name="affiliate" options={{ title: "Affiliate" }} />
      <Stack.Screen name="support" options={{ title: "Support" }} />
    </Stack>
  )
}
