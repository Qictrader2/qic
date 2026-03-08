import { useEffect, useRef, useState, useCallback } from 'react';
import type { LiveBoardStats, WorkEvent } from '../types';

const RECONNECT_DELAY_MS = 3000;

function parseSelectionUpdatedData(data: string): LiveBoardStats {
  return JSON.parse(data) as LiveBoardStats;
}

function parseAgentProgressData(data: string): { task_id: number; message: string } {
  return JSON.parse(data) as { task_id: number; message: string };
}

export interface UseWorkEventsResult {
  connected: boolean;
}

export function useWorkEvents(onEvent: (event: WorkEvent) => void): UseWorkEventsResult {
  const [connected, setConnected] = useState(false);
  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  const connect = useCallback(() => {
    const url = `${window.location.origin}/api/work/events`;
    const es = new EventSource(url);
    eventSourceRef.current = es;

    es.onopen = () => {
      setConnected(true);
    };

    es.addEventListener('selection_updated', (ev: MessageEvent) => {
      try {
        const data = parseSelectionUpdatedData(ev.data);
        onEventRef.current({ type: 'selection_updated', data });
      } catch {
        // Ignore parse errors
      }
    });

    es.addEventListener('agent_progress', (ev: MessageEvent) => {
      try {
        const data = parseAgentProgressData(ev.data);
        onEventRef.current({ type: 'agent_progress', data });
      } catch {
        // Ignore parse errors
      }
    });

    // ping events are keepalive - ignore them (no listener needed)

    es.onerror = () => {
      es.close();
      eventSourceRef.current = null;
      setConnected(false);
      reconnectTimeoutRef.current = setTimeout(() => {
        reconnectTimeoutRef.current = null;
        connect();
      }, RECONNECT_DELAY_MS);
    };
  }, []);

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimeoutRef.current !== null) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (eventSourceRef.current !== null) {
        eventSourceRef.current.close();
      }
    };
  }, [connect]);

  return { connected };
}
