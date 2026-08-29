#!/usr/bin/env bun
/**
 * Ticket 616 — Clockify-free ticket time tracker.
 *
 * Produces a per-ticket time report from two signals we already generate,
 * with NO new services and NO new secrets:
 *
 *   1. Trello column dwell — each `updateCard:idList` action timestamps a
 *      column move, so we can compute wall-clock time spent in each workflow
 *      column (cycle time, stage bottlenecks). A card parked overnight counts.
 *   2. Git commit-session clustering — commits carry `TICKET-ID:` prefixes.
 *      Grouping by ticket and clustering commit timestamps into work sessions
 *      (a gap > threshold starts a new session) gives an *estimated active
 *      effort* signal for calibrating estimates.
 *
 * The git number is labelled "estimated active" on purpose: agent-heavy work
 * compresses commit timestamps, so sessions understate thinking/review time.
 * It is never presented as authoritative logged time.
 *
 * Read-only. Reads Trello creds from `~/.qictrader-secrets/trello.env`
 * (referenced by env var NAME only, never printed) and local git history.
 *
 * Run:
 *   bun scripts/ticket-time.ts --since 2026-08-01 [--until 2026-08-31]
 *   bun scripts/ticket-time.ts --board R7WQRSJ9 --repo . --repo frontend
 *
 * Test:
 *   bun test scripts/ticket-time.test.ts
 */

import { execFileSync } from "node:child_process"

// ─── Pure core (unit-tested) ─────────────────────────────────────────────

/** A Trello action, trimmed to what we consume. */
export interface TrelloAction {
  type: string
  date: string // ISO 8601
  data: {
    list?: { id: string; name: string }
    listBefore?: { id: string; name: string }
    listAfter?: { id: string; name: string }
  }
}

/** The card entered `list` at `enteredAt`. */
export interface Transition {
  list: string
  enteredAt: Date
}

/** Dwell in a single column occupancy. */
export interface DwellSegment {
  list: string
  enteredAt: Date
  leftAt: Date | null
  dwellMs: number
}

export interface DwellReport {
  createdAt: Date | null
  segments: DwellSegment[]
  /** Summed dwell per column (a card can revisit a column). */
  byList: Record<string, number>
  /** created → now, or null if we never saw a start. */
  totalMs: number | null
}

/**
 * Normalise raw Trello actions into an ascending list of column entries.
 * `createCard` seeds the initial column; each list-move `updateCard` adds an
 * entry into `listAfter`. Actions arrive newest-first from the API, so we sort.
 */
export function buildTransitions(actions: TrelloAction[]): Transition[] {
  const transitions: Transition[] = []
  const sorted = [...actions].sort(
    (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
  )
  for (const action of sorted) {
    if (action.type === "createCard" && action.data.list) {
      transitions.push({ list: action.data.list.name, enteredAt: new Date(action.date) })
    } else if (action.type === "updateCard" && action.data.listAfter) {
      transitions.push({
        list: action.data.listAfter.name,
        enteredAt: new Date(action.date),
      })
    }
  }
  return transitions
}

/**
 * Compute exact dwell per column from ordered transitions. Dwell in
 * transition[i] is transition[i+1].enteredAt − transition[i].enteredAt; the
 * final (current) column is open, so its dwell runs to `now`.
 */
export function computeDwell(transitions: Transition[], nowMs: number): DwellReport {
  if (transitions.length === 0) {
    return { createdAt: null, segments: [], byList: {}, totalMs: null }
  }
  const ordered = [...transitions].sort(
    (a, b) => a.enteredAt.getTime() - b.enteredAt.getTime(),
  )
  const segments: DwellSegment[] = []
  const byList: Record<string, number> = {}

  for (let i = 0; i < ordered.length; i++) {
    const enteredAt = ordered[i].enteredAt
    const leftAt = i + 1 < ordered.length ? ordered[i + 1].enteredAt : null
    const endMs = leftAt ? leftAt.getTime() : nowMs
    const dwellMs = Math.max(0, endMs - enteredAt.getTime())
    segments.push({ list: ordered[i].list, enteredAt, leftAt, dwellMs })
    byList[ordered[i].list] = (byList[ordered[i].list] ?? 0) + dwellMs
  }

  const createdAt = ordered[0].enteredAt
  return {
    createdAt,
    segments,
    byList,
    totalMs: Math.max(0, nowMs - createdAt.getTime()),
  }
}

/** First time the card entered any of `listNames`, or null if it never did. */
export function firstEntryInto(
  transitions: Transition[],
  listNames: readonly string[],
): Date | null {
  const wanted = new Set(listNames)
  const ordered = [...transitions].sort(
    (a, b) => a.enteredAt.getTime() - b.enteredAt.getTime(),
  )
  for (const t of ordered) {
    if (wanted.has(t.list)) return t.enteredAt
  }
  return null
}

/**
 * Extract a ticket id from a commit subject.
 *
 * Matches a leading `TICKET-ID:` prefix where the id contains at least one
 * digit — so real ids (`616`, `DSP-63`, `PERF-BE-1`) match, but
 * conventional-commit types (`fix:`, `feat:`), merge commits, and dependency
 * bumps do not. Returns the id upper-cased for stable grouping, or null.
 */
export function extractTicketId(subject: string): string | null {
  const m = /^\s*\[?([A-Za-z0-9][A-Za-z0-9-]*)\]?\s*:/.exec(subject)
  if (!m) return null
  const id = m[1]
  if (!/\d/.test(id)) return null // reject `fix:`, `feat:`, etc.
  return id.toUpperCase()
}

export interface Commit {
  hash: string
  subject: string
  date: Date
  author: string
}

export interface Session {
  startMs: number
  endMs: number
  commitCount: number
}

/**
 * Cluster ascending timestamps into work sessions. A gap strictly greater than
 * `gapMs` between consecutive commits starts a new session. A lone commit is a
 * zero-length session.
 */
export function clusterSessions(timestampsMs: number[], gapMs: number): Session[] {
  const sorted = [...timestampsMs].sort((a, b) => a - b)
  const sessions: Session[] = []
  for (const ts of sorted) {
    const cur = sessions[sessions.length - 1]
    if (!cur || ts - cur.endMs > gapMs) {
      sessions.push({ startMs: ts, endMs: ts, commitCount: 1 })
    } else {
      cur.endMs = ts
      cur.commitCount += 1
    }
  }
  return sessions
}

/** Estimated active effort (ms) = sum of session spans. */
export function estimateActiveMs(commits: Commit[], gapMs: number): number {
  const sessions = clusterSessions(
    commits.map((c) => c.date.getTime()),
    gapMs,
  )
  return sessions.reduce((sum, s) => sum + (s.endMs - s.startMs), 0)
}

export interface GroupedCommits {
  byTicket: Map<string, Commit[]>
  unattributed: Commit[]
}

/**
 * Group commits by extracted ticket id. Commits with no resolvable id go into
 * an explicit `unattributed` bucket — never silently dropped.
 *
 * If `knownIds` is provided, a commit whose id isn't in the set is also treated
 * as unattributed (its ticket matches no card).
 */
export function groupCommitsByTicket(
  commits: Commit[],
  knownIds?: ReadonlySet<string>,
): GroupedCommits {
  const byTicket = new Map<string, Commit[]>()
  const unattributed: Commit[] = []
  for (const commit of commits) {
    const id = extractTicketId(commit.subject)
    if (id === null || (knownIds && !knownIds.has(id))) {
      unattributed.push(commit)
      continue
    }
    const bucket = byTicket.get(id)
    if (bucket) bucket.push(commit)
    else byTicket.set(id, [commit])
  }
  return { byTicket, unattributed }
}

/** Format a millisecond duration as a compact `1h08m` / `6m` / `45s`. */
export function formatDuration(ms: number): string {
  if (ms <= 0) return "0m"
  const totalMin = Math.floor(ms / 60000)
  const h = Math.floor(totalMin / 60)
  const m = totalMin % 60
  if (h > 0) return `${h}h${String(m).padStart(2, "0")}m`
  if (m > 0) return `${m}m`
  return `${Math.floor(ms / 1000)}s`
}

export const DEFAULT_SESSION_GAP_MS = 2 * 60 * 60 * 1000 // 2h
export const DEFAULT_BOARD = "R7WQRSJ9" // Project One (shortlink)
const TRELLO_BASE = "https://api.trello.com/1"

// ─── Side-effecting edges (not unit-tested) ──────────────────────────────

interface Cli {
  board: string
  since: Date | null
  until: Date | null
  repos: string[]
  gapMs: number
}

function parseArgs(argv: string[]): Cli {
  const cli: Cli = {
    board: DEFAULT_BOARD,
    since: null,
    until: null,
    repos: [],
    gapMs: DEFAULT_SESSION_GAP_MS,
  }
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === "--board") cli.board = argv[++i] ?? cli.board
    else if (a === "--since") cli.since = new Date(argv[++i] ?? "")
    else if (a === "--until") cli.until = new Date(argv[++i] ?? "")
    else if (a === "--repo") cli.repos.push(argv[++i] ?? ".")
    else if (a === "--gap-hours") cli.gapMs = Number(argv[++i]) * 3600_000
  }
  if (cli.repos.length === 0) cli.repos = [".", "frontend", "qictrader-backend-rs"]
  return cli
}

function requireTrelloCreds(): { key: string; token: string } {
  const key = process.env.TRELLO_API_KEY
  const token = process.env.TRELLO_API_TOKEN
  if (!key || !token) {
    throw new Error(
      "TRELLO_API_KEY / TRELLO_API_TOKEN not set. Source ~/.qictrader-secrets/trello.env first.",
    )
  }
  return { key, token }
}

async function trelloGet<T>(
  path: string,
  creds: { key: string; token: string },
  params: Record<string, string> = {},
): Promise<T> {
  const url = new URL(`${TRELLO_BASE}${path}`)
  for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v)
  url.searchParams.set("key", creds.key)
  url.searchParams.set("token", creds.token)
  const res = await fetch(url)
  if (!res.ok) {
    // Never echo the URL (it carries the token) — reference by path only.
    throw new Error(`Trello GET ${path} failed: ${res.status}`)
  }
  return (await res.json()) as T
}

interface TrelloCard {
  id: string
  idShort: number
  name: string
}

function collectCommits(repoPath: string, since: Date | null, until: Date | null): Commit[] {
  const args = ["-C", repoPath, "log", "--no-merges", "--pretty=format:%H%x1f%aI%x1f%an%x1f%s"]
  if (since) args.push(`--since=${since.toISOString()}`)
  if (until) args.push(`--until=${until.toISOString()}`)
  let out = ""
  try {
    out = execFileSync("git", args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 })
  } catch {
    // A missing/uninitialised repo path shouldn't abort the whole report.
    process.stderr.write(`warning: could not read git log in ${repoPath}\n`)
    return []
  }
  return out
    .split("\n")
    .filter((line) => line.trim().length > 0)
    .map((line) => {
      const [hash, iso, author, ...subjectParts] = line.split("\x1f")
      return {
        hash,
        date: new Date(iso),
        author,
        subject: subjectParts.join("\x1f"),
      }
    })
}

function knownIdsFromCards(cards: TrelloCard[]): Set<string> {
  const ids = new Set<string>()
  for (const card of cards) {
    ids.add(String(card.idShort))
    const fromName = extractTicketId(card.name)
    if (fromName) ids.add(fromName)
  }
  return ids
}

async function main(): Promise<void> {
  const cli = parseArgs(process.argv.slice(2))
  const creds = requireTrelloCreds()

  const cards = await trelloGet<TrelloCard[]>(`/boards/${cli.board}/cards`, creds, {
    fields: "idShort,name",
  })

  // Gather commits across the configured repos.
  const allCommits = cli.repos.flatMap((r) => collectCommits(r, cli.since, cli.until))
  const knownIds = knownIdsFromCards(cards)
  const { byTicket, unattributed } = groupCommitsByTicket(allCommits, knownIds)

  const nowMs = Date.now()
  const rows: string[] = []
  rows.push(
    ["Ticket", "Cycle", "Active(est)", "Commits", "Top column"].join("\t"),
  )

  for (const card of cards.sort((a, b) => a.idShort - b.idShort)) {
    const actions = await trelloGet<TrelloAction[]>(`/cards/${card.id}/actions`, creds, {
      filter: "createCard,updateCard",
      limit: "1000",
    })
    const transitions = buildTransitions(actions)
    const dwell = computeDwell(transitions, nowMs)

    // Skip cards created entirely outside the window when a window is set.
    if (cli.since && dwell.createdAt && dwell.createdAt < cli.since) {
      // still include if it saw commits in-window
    }

    const idShort = String(card.idShort)
    const idFromName = extractTicketId(card.name)
    const commits = [
      ...(byTicket.get(idShort) ?? []),
      ...(idFromName ? (byTicket.get(idFromName) ?? []) : []),
    ]
    const activeMs = estimateActiveMs(commits, cli.gapMs)

    const topColumn =
      Object.entries(dwell.byList).sort((a, b) => b[1] - a[1])[0]?.[0] ?? "—"

    rows.push(
      [
        `#${card.idShort}`,
        dwell.totalMs !== null ? formatDuration(dwell.totalMs) : "—",
        `${formatDuration(activeMs)}${commits.length ? "" : " (no commits)"}`,
        String(commits.length),
        topColumn,
      ].join("\t"),
    )
  }

  process.stdout.write(rows.join("\n") + "\n")

  if (unattributed.length > 0) {
    const activeMs = estimateActiveMs(unattributed, cli.gapMs)
    process.stdout.write(
      `\nUnattributed commits (ticket id matched no card): ${unattributed.length}, ` +
        `estimated active ${formatDuration(activeMs)}\n`,
    )
  }
}

// Bun sets import.meta.main for the entrypoint; guard so tests can import
// the pure functions without triggering network / git calls.
if (import.meta.main) {
  main().catch((err) => {
    process.stderr.write(`ticket-time failed: ${err instanceof Error ? err.message : err}\n`)
    process.exit(1)
  })
}
