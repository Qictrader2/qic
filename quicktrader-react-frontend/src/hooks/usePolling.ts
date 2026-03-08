import { useEffect, useRef } from 'react';

/**
 * Generic polling hook. Calls fn immediately on mount, then every intervalMs.
 * Cleans up on unmount. Only runs when enabled is true (defaults to true).
 */
export function usePolling(
  fn: () => Promise<void>,
  intervalMs: number,
  enabled: boolean = true
): void {
  const fnRef = useRef(fn);
  fnRef.current = fn;

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;

    const run = async (): Promise<void> => {
      try {
        await fnRef.current();
      } catch {
        // Caller handles errors; we just avoid unhandled rejection
      }
      if (!cancelled) {
        timeoutId = window.setTimeout(run, intervalMs);
      }
    };

    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    run();

    return () => {
      cancelled = true;
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }
    };
  }, [intervalMs, enabled]);
}
