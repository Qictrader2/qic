/**
 * X5EO4D6X War Room: shared number/currency formatting helpers.
 *
 * Centralises ZAR / crypto / delta formatting used across the war-room
 * dashboard so styling stays consistent. Also re-usable elsewhere as we
 * pull `toLocaleString(...)` out of components.
 */

const ZAR_FORMATTER = new Intl.NumberFormat("en-ZA", {
  style: "currency",
  currency: "ZAR",
  maximumFractionDigits: 0,
})

const ZAR_FORMATTER_WITH_CENTS = new Intl.NumberFormat("en-ZA", {
  style: "currency",
  currency: "ZAR",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
})

const COMPACT_ZAR = new Intl.NumberFormat("en-ZA", {
  style: "currency",
  currency: "ZAR",
  notation: "compact",
  maximumFractionDigits: 1,
})

/** `R 4,200,000` (no decimals — for big headline KPI numbers). */
export function formatZar(amount: number | null | undefined): string {
  if (amount == null || !Number.isFinite(amount)) return "R 0"
  return ZAR_FORMATTER.format(amount).replace("ZAR", "R").trim()
}

/** `R 12,345.67` (with decimals — for per-row money cells). */
export function formatZarPrecise(amount: number | null | undefined): string {
  if (amount == null || !Number.isFinite(amount)) return "R 0.00"
  return ZAR_FORMATTER_WITH_CENTS.format(amount).replace("ZAR", "R").trim()
}

/** `R 4.2M` — for axis labels and tight cells. */
export function formatZarCompact(amount: number | null | undefined): string {
  if (amount == null || !Number.isFinite(amount)) return "R 0"
  return COMPACT_ZAR.format(amount).replace("ZAR", "R").trim()
}

/**
 * Crypto with currency-aware decimal precision. BTC: 8 dp; ETH/SOL: 6 dp;
 * stablecoins: 2 dp.
 */
export function formatCrypto(
  amount: number | null | undefined,
  currency: string,
): string {
  if (amount == null || !Number.isFinite(amount)) return `0 ${currency}`
  const dp = decimalsFor(currency)
  return `${amount.toLocaleString("en-US", {
    minimumFractionDigits: dp,
    maximumFractionDigits: dp,
  })} ${currency}`
}

function decimalsFor(currency: string): number {
  switch (currency.toUpperCase()) {
    case "BTC":
      return 8
    case "ETH":
    case "SOL":
      return 6
    case "USDT":
    case "USDC":
    default:
      return 2
  }
}

/**
 * `+18.4%` (positive) / `-5.2%` (negative). Returns null when delta is
 * undefined (denominator was zero — caller should hide the badge).
 */
export function formatDelta(pct: number | null | undefined): string | null {
  if (pct == null || !Number.isFinite(pct)) return null
  const sign = pct > 0 ? "+" : ""
  return `${sign}${pct.toFixed(1)}%`
}

/** Tailwind text colour for a delta value. Zero = neutral. */
export function deltaColor(pct: number | null | undefined): string {
  if (pct == null || !Number.isFinite(pct)) return "text-muted-foreground"
  if (pct > 0.5) return "text-green-500"
  if (pct < -0.5) return "text-red-500"
  return "text-muted-foreground"
}

/** Compact integer `1,234` (no currency). */
export function formatInt(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return "0"
  return Math.round(n).toLocaleString("en-ZA")
}

/**
 * `120 bps` (2 dp default). Used for the war-room fee-yield KPI.
 * `null` / `undefined` / non-finite renders as a single dash.
 */
export function formatBps(
  bps: number | null | undefined,
  digits = 1,
): string {
  if (bps == null || !Number.isFinite(bps)) return "—"
  return `${bps.toFixed(digits)} bps`
}

/** `12.4%` from a fraction in [0, 1]. Same `—` fallback. */
export function formatPercent(
  fraction: number | null | undefined,
  digits = 1,
): string {
  if (fraction == null || !Number.isFinite(fraction)) return "—"
  return `${(fraction * 100).toFixed(digits)}%`
}

/**
 * Convert seconds to a compact human duration (e.g. `47s`, `4 min`,
 * `2.1 hr`, `1.3 days`). Investors don't want "00:04:32" — they want
 * a unit that fits on a chip.
 */
export function formatDuration(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return "—"
  if (seconds < 60) return `${Math.round(seconds)}s`
  const minutes = seconds / 60
  if (minutes < 60) return `${minutes.toFixed(minutes < 10 ? 1 : 0)} min`
  const hours = minutes / 60
  if (hours < 48) return `${hours.toFixed(hours < 10 ? 1 : 0)} hr`
  const days = hours / 24
  return `${days.toFixed(days < 10 ? 1 : 0)} days`
}

/** "2 min ago" / "just now" / "1 hr ago" — for activity feed. */
export function formatRelative(iso: string): string {
  const ts = new Date(iso).getTime()
  if (!Number.isFinite(ts)) return iso
  const diffSec = Math.max(0, Math.floor((Date.now() - ts) / 1000))
  if (diffSec < 10) return "just now"
  if (diffSec < 60) return `${diffSec}s ago`
  const diffMin = Math.floor(diffSec / 60)
  if (diffMin < 60) return `${diffMin} min ago`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr} hr ago`
  const diffDay = Math.floor(diffHr / 24)
  if (diffDay < 30) return `${diffDay}d ago`
  return new Date(iso).toLocaleDateString("en-ZA")
}
