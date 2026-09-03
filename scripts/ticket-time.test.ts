/**
 * Ticket 616 — tests for the Clockify-free ticket time tracker.
 *
 * Column-dwell maths is checked exactly against a fixture of Trello action
 * history; session clustering is checked over prefixed, unprefixed, merge, and
 * bot commits with exact expected durations. Run: `bun test scripts/ticket-time.test.ts`.
 */

import { describe, expect, test } from "bun:test"
import {
  buildTransitions,
  clusterSessions,
  computeDwell,
  DEFAULT_SESSION_GAP_MS,
  estimateActiveMs,
  extractTicketId,
  firstEntryInto,
  formatDuration,
  groupCommitsByTicket,
  type Commit,
  type TrelloAction,
  fetchAllCardActions,
  TRELLO_ACTIONS_PAGE_LIMIT,
} from "./ticket-time"

const HOUR = 3600_000
const MIN = 60_000

// Mirrors the shape verified on the affiliate-links card in the ticket:
// created → Development (1h08m) → QA (6m) → Deployment.
const FIXTURE: TrelloAction[] = [
  // API returns newest-first; buildTransitions must sort.
  {
    type: "updateCard",
    date: "2026-07-31T19:14:00.000Z",
    data: {
      listBefore: { id: "l2", name: "QA / Testing (In Progress)" },
      listAfter: { id: "l3", name: "Deployment Queue" },
    },
  },
  {
    type: "updateCard",
    date: "2026-07-31T19:08:00.000Z",
    data: {
      listBefore: { id: "l1", name: "Development (In Progress)" },
      listAfter: { id: "l2", name: "QA / Testing (In Progress)" },
    },
  },
  {
    type: "updateCard",
    date: "2026-07-31T18:00:00.000Z",
    data: {
      listBefore: { id: "l0", name: "Sprint Ready" },
      listAfter: { id: "l1", name: "Development (In Progress)" },
    },
  },
  {
    type: "createCard",
    date: "2026-07-31T17:30:00.000Z",
    data: { list: { id: "l0", name: "Sprint Ready" } },
  },
]

describe("buildTransitions", () => {
  test("sorts newest-first actions into ascending column entries", () => {
    const transitions = buildTransitions(FIXTURE)
    expect(transitions.map((t) => t.list)).toEqual([
      "Sprint Ready",
      "Development (In Progress)",
      "QA / Testing (In Progress)",
      "Deployment Queue",
    ])
    expect(transitions[0].enteredAt.toISOString()).toBe("2026-07-31T17:30:00.000Z")
  })

  test("ignores non-list updateCard actions", () => {
    const noise: TrelloAction[] = [
      { type: "commentCard", date: "2026-07-31T17:00:00.000Z", data: {} },
      { type: "updateCard", date: "2026-07-31T17:05:00.000Z", data: {} }, // no listAfter
    ]
    expect(buildTransitions(noise)).toEqual([])
  })
})

describe("computeDwell", () => {
  // Deployment entered at 19:14; "now" 20:14 → 1h in Deployment.
  const NOW = new Date("2026-07-31T20:14:00.000Z").getTime()

  test("computes exact per-column dwell against the fixture", () => {
    const report = computeDwell(buildTransitions(FIXTURE), NOW)
    expect(report.byList["Sprint Ready"]).toBe(30 * MIN) // 17:30 → 18:00
    expect(report.byList["Development (In Progress)"]).toBe(68 * MIN) // 18:00 → 19:08 = 1h08m
    expect(report.byList["QA / Testing (In Progress)"]).toBe(6 * MIN) // 19:08 → 19:14
    expect(report.byList["Deployment Queue"]).toBe(60 * MIN) // 19:14 → now
  })

  test("total cycle time is created → now", () => {
    const report = computeDwell(buildTransitions(FIXTURE), NOW)
    // 17:30 → 20:14 = 2h44m
    expect(report.totalMs).toBe(2 * HOUR + 44 * MIN)
    expect(report.createdAt?.toISOString()).toBe("2026-07-31T17:30:00.000Z")
  })

  test("sums dwell when a card revisits the same column", () => {
    const revisits = buildTransitions([
      { type: "createCard", date: "2026-08-01T00:00:00.000Z", data: { list: { id: "d", name: "Dev" } } },
      {
        type: "updateCard",
        date: "2026-08-01T01:00:00.000Z",
        data: { listBefore: { id: "d", name: "Dev" }, listAfter: { id: "q", name: "QA" } },
      },
      {
        type: "updateCard",
        date: "2026-08-01T02:00:00.000Z",
        data: { listBefore: { id: "q", name: "QA" }, listAfter: { id: "d", name: "Dev" } },
      },
    ])
    const report = computeDwell(revisits, new Date("2026-08-01T02:30:00.000Z").getTime())
    // Dev: 00:00→01:00 (1h) + 02:00→02:30 (30m) = 1h30m
    expect(report.byList["Dev"]).toBe(HOUR + 30 * MIN)
    expect(report.byList["QA"]).toBe(HOUR)
  })

  test("empty history yields a null report, not a crash", () => {
    const report = computeDwell([], NOW)
    expect(report).toEqual({ createdAt: null, segments: [], byList: {}, totalMs: null })
  })
})

describe("firstEntryInto", () => {
  test("returns when the card first reached a terminal column", () => {
    const t = buildTransitions(FIXTURE)
    expect(firstEntryInto(t, ["Deployment Queue"])?.toISOString()).toBe(
      "2026-07-31T19:14:00.000Z",
    )
    expect(firstEntryInto(t, ["Done"])).toBeNull()
  })
})

describe("extractTicketId", () => {
  test("pulls numeric and prefixed ticket ids, upper-cased", () => {
    expect(extractTicketId("616: add tracker")).toBe("616")
    expect(extractTicketId("dsp-63: heroku pages")).toBe("DSP-63")
    expect(extractTicketId("PERF-BE-1: gin index")).toBe("PERF-BE-1")
    expect(extractTicketId("  730: leading space")).toBe("730")
    expect(extractTicketId("[TKT-632]: bracketed")).toBe("TKT-632")
  })

  test("rejects conventional-commit types, merges, and bot commits", () => {
    expect(extractTicketId("fix: null check")).toBeNull()
    expect(extractTicketId("feat: add thing")).toBeNull()
    expect(extractTicketId("Merge branch 'main' into feature")).toBeNull()
    expect(extractTicketId("Merge pull request #12 from foo/bar")).toBeNull()
    expect(extractTicketId("Bump lodash from 4.17.20 to 4.17.21")).toBeNull()
    expect(extractTicketId("no colon here 123")).toBeNull()
  })
})

describe("clusterSessions", () => {
  const base = Date.UTC(2026, 7, 1, 9, 0, 0)
  const at = (min: number) => base + min * MIN

  test("groups commits within the gap into one session with exact span", () => {
    // 09:00, 09:30, 10:15 — all within 2h gaps → one session of 1h15m.
    const sessions = clusterSessions([at(0), at(30), at(75)], DEFAULT_SESSION_GAP_MS)
    expect(sessions).toHaveLength(1)
    expect(sessions[0].commitCount).toBe(3)
    expect(sessions[0].endMs - sessions[0].startMs).toBe(75 * MIN)
  })

  test("a gap larger than the threshold starts a new session", () => {
    // 09:00, then +3h (> 2h gap) → two sessions.
    const sessions = clusterSessions([at(0), at(30), at(30 + 180), at(30 + 200)], DEFAULT_SESSION_GAP_MS)
    expect(sessions).toHaveLength(2)
    expect(sessions[0].endMs - sessions[0].startMs).toBe(30 * MIN)
    expect(sessions[1].endMs - sessions[1].startMs).toBe(20 * MIN)
  })

  test("a lone commit is a zero-length session", () => {
    const sessions = clusterSessions([at(0)], DEFAULT_SESSION_GAP_MS)
    expect(sessions).toEqual([{ startMs: at(0), endMs: at(0), commitCount: 1 }])
  })

  test("unsorted timestamps are handled", () => {
    const sessions = clusterSessions([at(75), at(0), at(30)], DEFAULT_SESSION_GAP_MS)
    expect(sessions).toHaveLength(1)
    expect(sessions[0].endMs - sessions[0].startMs).toBe(75 * MIN)
  })
})

describe("estimateActiveMs + groupCommitsByTicket", () => {
  const base = Date.UTC(2026, 7, 1, 9, 0, 0)
  const commit = (subject: string, min: number): Commit => ({
    hash: `${subject}-${min}`,
    subject,
    author: "dev",
    date: new Date(base + min * MIN),
  })

  test("sums session spans as estimated active effort", () => {
    const commits = [
      commit("616: a", 0),
      commit("616: b", 45),
      commit("616: c", 45 + 200), // > 2h gap → new session
      commit("616: d", 45 + 230),
    ]
    // Session 1: 0→45 = 45m; session 2: 200→230 relative = 30m → 1h15m.
    expect(estimateActiveMs(commits, DEFAULT_SESSION_GAP_MS)).toBe(75 * MIN)
  })

  test("routes unprefixed / merge / bot commits to the unattributed bucket", () => {
    const commits = [
      commit("616: real", 0),
      commit("Merge branch 'x'", 10),
      commit("Bump dep from 1 to 2", 20),
      commit("chore no id", 30),
    ]
    const { byTicket, unattributed } = groupCommitsByTicket(commits)
    expect(byTicket.get("616")).toHaveLength(1)
    expect(unattributed).toHaveLength(3)
  })

  test("treats a ticket id that matches no known card as unattributed", () => {
    const commits = [commit("616: known", 0), commit("999: unknown card", 10)]
    const known = new Set(["616"])
    const { byTicket, unattributed } = groupCommitsByTicket(commits, known)
    expect(byTicket.get("616")).toHaveLength(1)
    expect(byTicket.has("999")).toBe(false)
    expect(unattributed.map((c) => c.subject)).toEqual(["999: unknown card"])
  })

  test("a card with no matching commits reports zero active time", () => {
    expect(estimateActiveMs([], DEFAULT_SESSION_GAP_MS)).toBe(0)
  })
})

describe("formatDuration", () => {
  test("formats hours, minutes, seconds compactly", () => {
    expect(formatDuration(68 * MIN)).toBe("1h08m")
    expect(formatDuration(6 * MIN)).toBe("6m")
    expect(formatDuration(45 * 1000)).toBe("45s")
    expect(formatDuration(0)).toBe("0m")
    expect(formatDuration(2 * HOUR + 44 * MIN)).toBe("2h44m")
  })
})

describe("fetchAllCardActions", () => {
  const creds = { key: "k", token: "t" }

  /** A page of `n` actions whose ids are unique and ordered newest-first. */
  const page = (n: number, offset = 0): TrelloAction[] =>
    Array.from({ length: n }, (_, i) => ({
      id: `a${offset + i}`,
      type: "updateCard",
      date: new Date(Date.UTC(2026, 0, 1)).toISOString(),
      data: {},
    }))

  test("a single short page needs no second request", async () => {
    const calls: Record<string, string>[] = []
    const actions = await fetchAllCardActions("card1", creds, async (_p, _c, params) => {
      calls.push(params)
      return page(3)
    })

    expect(actions).toHaveLength(3)
    expect(calls).toHaveLength(1)
    expect(calls[0]!.before).toBeUndefined()
  })

  test("follows the before cursor until a short page ends it", async () => {
    // The bug this guards: one limit=1000 request silently drops the OLDEST
    // actions, which is where createCard lives, so cycle time comes out short
    // with no sign anything was missing.
    const seen: (string | undefined)[] = []
    let call = 0
    const actions = await fetchAllCardActions("card1", creds, async (_p, _c, params) => {
      seen.push(params.before)
      call += 1
      if (call === 1) return page(TRELLO_ACTIONS_PAGE_LIMIT, 0)
      if (call === 2) return page(TRELLO_ACTIONS_PAGE_LIMIT, TRELLO_ACTIONS_PAGE_LIMIT)
      return page(7, 2 * TRELLO_ACTIONS_PAGE_LIMIT)
    })

    expect(actions).toHaveLength(2 * TRELLO_ACTIONS_PAGE_LIMIT + 7)
    expect(seen[0]).toBeUndefined()
    // Each subsequent request asks for everything before the oldest seen so far.
    expect(seen[1]).toBe(`a${TRELLO_ACTIONS_PAGE_LIMIT - 1}`)
    expect(seen[2]).toBe(`a${2 * TRELLO_ACTIONS_PAGE_LIMIT - 1}`)
  })

  test("an empty first page yields nothing and stops", async () => {
    let calls = 0
    const actions = await fetchAllCardActions("card1", creds, async () => {
      calls += 1
      return []
    })
    expect(actions).toEqual([])
    expect(calls).toBe(1)
  })

  test("a full page whose oldest action has no id stops instead of looping", async () => {
    // Without this guard the same request would repeat forever, since there is
    // no cursor to advance past.
    let calls = 0
    const actions = await fetchAllCardActions("card1", creds, async () => {
      calls += 1
      return Array.from({ length: TRELLO_ACTIONS_PAGE_LIMIT }, () => ({
        type: "updateCard",
        date: new Date(Date.UTC(2026, 0, 1)).toISOString(),
        data: {},
      }))
    })
    expect(calls).toBe(1)
    expect(actions).toHaveLength(TRELLO_ACTIONS_PAGE_LIMIT)
  })
})
