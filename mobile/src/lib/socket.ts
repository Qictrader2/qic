import { io, Socket } from "socket.io-client"
import { secureStorage } from "@/src/lib/storage/secure"

let _socket: Socket | null = null

/** Async initializer — call once on app startup / login */
export async function connectSocket(): Promise<Socket> {
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

/** Synchronous accessor — returns the socket if already connected, null otherwise */
export function getSocket(): Socket | null {
  return _socket?.connected ? _socket : null
}

/** Legacy async alias so existing code compiling against the old signature still works */
export async function getSocketAsync(): Promise<Socket> {
  return connectSocket()
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
  connectSocket().then((socket) => {
    socket.emit("join_trade", { tradeId })
    socket.on(`trade:${tradeId}:update`, onUpdate)
    socket.on(`trade:${tradeId}:message`, onMessage)
  })

  return () => {
    connectSocket().then((socket) => {
      socket.off(`trade:${tradeId}:update`, onUpdate)
      socket.off(`trade:${tradeId}:message`, onMessage)
      socket.emit("leave_trade", { tradeId })
    })
  }
}
