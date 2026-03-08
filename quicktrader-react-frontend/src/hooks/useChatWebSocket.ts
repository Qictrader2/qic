import { useEffect, useRef, useState, useCallback } from 'react';
import type { ChatWsEvent } from '../types';

const MAX_RETRIES = 30;
const MAX_BACKOFF_MS = 30_000;
const CLOSE_CODE_RETRY_INDEFINITELY = 4001;

function getWsBaseUrl(): string {
  const { protocol, host } = window.location;
  return protocol === 'https:' ? `wss://${host}` : `ws://${host}`;
}

export interface UseChatWebSocketResult {
  connected: boolean;
  lastSeq: number;
  subscribe: (convId: string, lastSeq?: number) => void;
  unsubscribe: () => void;
}

export function useChatWebSocket(onMessage: (event: ChatWsEvent) => void): UseChatWebSocketResult {
  const [connected, setConnected] = useState(false);
  const [lastSeq, setLastSeq] = useState(0);
  const wsRef = useRef<WebSocket | null>(null);
  const retryCountRef = useRef(0);
  const retryTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onMessageRef = useRef(onMessage);
  onMessageRef.current = onMessage;

  const currentConvIdRef = useRef<string | null>(null);
  const currentLastSeqRef = useRef<number | undefined>(undefined);

  const connect = useCallback((convId: string, lastSeqParam?: number) => {
    const base = getWsBaseUrl();
    const path = `/api/chat/ws/${convId}`;
    const query = lastSeqParam !== undefined ? `?last_seq=${lastSeqParam}` : '';
    const url = `${base}${path}${query}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      retryCountRef.current = 0;
    };

    ws.onmessage = (ev: MessageEvent) => {
      try {
        const parsed = JSON.parse(ev.data as string) as ChatWsEvent;
        if (parsed.type === 'message_chunk' && 'sequence' in parsed) {
          setLastSeq(parsed.sequence);
        }
        if (parsed.type === 'connected' && 'last_seq' in parsed) {
          setLastSeq(parsed.last_seq);
        }
        onMessageRef.current(parsed);
      } catch {
        // Ignore parse errors
      }
    };

    ws.onclose = (ev: CloseEvent) => {
      wsRef.current = null;
      setConnected(false);

      const convId = currentConvIdRef.current;
      if (convId === null) return;

      const shouldRetryIndefinitely = ev.code === CLOSE_CODE_RETRY_INDEFINITELY;
      const withinMaxRetries = retryCountRef.current < MAX_RETRIES;

      if (shouldRetryIndefinitely || withinMaxRetries) {
        const delay = Math.min(
          1000 * Math.pow(2, retryCountRef.current),
          MAX_BACKOFF_MS
        );
        retryCountRef.current += 1;
        retryTimeoutRef.current = setTimeout(() => {
          retryTimeoutRef.current = null;
          connect(convId, currentLastSeqRef.current);
        }, delay);
      }
    };

    ws.onerror = () => {
      // Close handler will handle reconnect
    };
  }, []);

  const subscribe = useCallback((convId: string, lastSeqParam?: number) => {
    if (retryTimeoutRef.current !== null) {
      clearTimeout(retryTimeoutRef.current);
      retryTimeoutRef.current = null;
    }
    if (wsRef.current !== null) {
      wsRef.current.close();
      wsRef.current = null;
    }
    currentConvIdRef.current = convId;
    currentLastSeqRef.current = lastSeqParam;
    retryCountRef.current = 0;
    connect(convId, lastSeqParam);
  }, [connect]);

  const unsubscribe = useCallback(() => {
    currentConvIdRef.current = null;
    currentLastSeqRef.current = undefined;
    if (retryTimeoutRef.current !== null) {
      clearTimeout(retryTimeoutRef.current);
      retryTimeoutRef.current = null;
    }
    if (wsRef.current !== null) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setConnected(false);
  }, []);

  useEffect(() => {
    return () => {
      if (retryTimeoutRef.current !== null) {
        clearTimeout(retryTimeoutRef.current);
      }
      if (wsRef.current !== null) {
        wsRef.current.close();
      }
    };
  }, []);

  return { connected, lastSeq, subscribe, unsubscribe };
}
