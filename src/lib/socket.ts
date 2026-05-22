import { io, Socket } from "socket.io-client"
import { secureStorage } from "@/src/lib/storage/secure"

let _socket: Socket | null = null

export async function getSocket(): Promise<Socket> {
  if (_socket?.connected) return _socket

  const token = await secureStorage.get("qic_access")

  _socket = io(process.env.EXPO_PUBLIC_WS_URL ?? process.env.EXPO_PUBLIC_API_URL ?? "", {
    transports: ["websocket"],
    auth: { token },
    reconnection: true,
    reconnectionAttempts: 5,
    reconnectionDelay: 1000,
  })

  return _socket
}

export function disconnectSocket() {
  if (_socket) {
    _socket.disconnect()
    _socket = null
  }
}

export function subscribeToTrade(
  tradeId: string,
  onUpdate: (data: unknown) => void,
  onMessage: (data: unknown) => void
): () => void {
  getSocket().then((socket) => {
    socket.emit("join_trade", { tradeId })
    socket.on(`trade:${tradeId}:update`, onUpdate)
    socket.on(`trade:${tradeId}:message`, onMessage)
  })

  return () => {
    getSocket().then((socket) => {
      socket.off(`trade:${tradeId}:update`, onUpdate)
      socket.off(`trade:${tradeId}:message`, onMessage)
      socket.emit("leave_trade", { tradeId })
    })
  }
}
