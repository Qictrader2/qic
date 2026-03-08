# Software Development Philosophy

Schalk's approach to building software — a type-driven, critique-looped, pull-based workflow designed for human-AI collaboration.

## Core Beliefs

1. **Projects contain stories** — a story is the atomic unit of deliverable work
2. **Start with the types** — let the compiler guide you to correctness
3. **YAGNI** — write the minimum necessary code, but no less
4. **Single commit cycles** — one story = one commit (or one feature branch for larger work)
5. **Critique loops** — build, critique, fix, repeat until clean
6. **Every transition gets a comment** — full audit trail from creation to done
7. **Testable definition of done** — every story has verifiable completion criteria

## Story Lifecycle

```mermaid
stateDiagram-v2
    direction TB

    classDef creation fill:#2563eb,color:#ffffff,stroke:#1d4ed8
    classDef active fill:#d97706,color:#ffffff,stroke:#b45309
    classDef review fill:#7c3aed,color:#ffffff,stroke:#6d28d9
    classDef terminal fill:#059669,color:#ffffff,stroke:#047857
    classDef rejected fill:#dc2626,color:#ffffff,stroke:#b91c1c
    classDef side fill:#475569,color:#ffffff,stroke:#334155

    [*] --> todo:::creation : PM creates story\nw/ definition of done

    todo --> in_progress:::active : Dev picks up\n(/dev or take_next)
    in_progress --> ready_for_review:::review : Dev completes\ncritique loop done

    ready_for_review --> under_review:::review : Reviewer picks up\n(/harden or take_next_review)
    under_review --> done:::terminal : Approved\n(zero issues found)
    under_review --> ready_for_review : Fixes applied\n(needs fresh eyes)
    under_review --> todo:::rejected : Rejected\n(moved to top of backlog)

    todo --> blocked:::side : Unmet dependency
    blocked --> todo : Dependency resolved
    todo --> abandoned:::side : Won't do
    done --> archived:::side : Cleanup
```

## The Development Loop (Build → Critique → Fix)

This is the core inner loop that happens while a story is `in_progress`:

```mermaid
flowchart TD
    classDef start fill:#2563eb,color:#ffffff,stroke:#1d4ed8
    classDef work fill:#d97706,color:#ffffff,stroke:#b45309
    classDef check fill:#7c3aed,color:#ffffff,stroke:#6d28d9
    classDef decision fill:#475569,color:#ffffff,stroke:#334155
    classDef done fill:#059669,color:#ffffff,stroke:#047857

    A[Start: Read story + DoD]:::start --> B[Define types first]:::work
    B --> C[Let compiler guide\nimplementation]:::work
    C --> D[Build the feature\nminimum necessary code]:::work
    D --> E[Run /critique]:::check
    E --> F{Critical issues?}:::decision
    F -->|Yes| G[Fix critical issues only\nbugs, security, logic errors\nstyle violations]:::work
    G --> H[New context: run /critique again]:::check
    H --> F
    F -->|No| I[Commit + move to\nready_for_review]:::done
```

### What /critique focuses on

| Priority | Category | Examples |
|----------|----------|---------|
| **HIGH** | Bugs | Logic errors, crashes, race conditions |
| **HIGH** | Security | Vulnerabilities, injection, auth gaps |
| **HIGH** | Logic inconsistencies | Code contradicts its own intent |
| **MEDIUM** | Style violations | Breaks project rules/conventions |
| **LOW** | Tangential | Minor style preferences, bikeshedding |

The developer decides which critique findings to fix. Tangential findings are often ignored. The loop repeats in a **fresh context** each time (to avoid anchoring bias).

## The Review Loop (Harden)

Once a story reaches `ready_for_review`, the harden loop takes over:

```mermaid
flowchart TD
    classDef start fill:#2563eb,color:#ffffff,stroke:#1d4ed8
    classDef review fill:#7c3aed,color:#ffffff,stroke:#6d28d9
    classDef fix fill:#d97706,color:#ffffff,stroke:#b45309
    classDef decision fill:#475569,color:#ffffff,stroke:#334155
    classDef done fill:#059669,color:#ffffff,stroke:#047857
    classDef comment fill:#0891b2,color:#ffffff,stroke:#0e7490

    A[Pick up review task]:::start --> B[Read ALL code changes]:::review
    B --> C[Compare against DoD\nand acceptance criteria]:::review
    C --> D[Post review findings\ncomment on task]:::comment
    D --> E{Issues found?}:::decision
    E -->|Yes| F[Fix issues]:::fix
    F --> G[Post fixes comment\non task]:::comment
    G --> H[Leave in ready_for_review\nfor fresh reviewer]:::review
    H --> A
    E -->|Zero issues| I[Mark done]:::done
```

Key rules:
- **Always comment before fixing** — audit trail
- **Always comment after fixing** — documents what changed
- **Never mark done if you made fixes** — needs fresh eyes
- The `/sdlc` command automates this loop (dev → harden × N → done)

## Story Dependencies

```mermaid
flowchart LR
    classDef independent fill:#2563eb,color:#ffffff,stroke:#1d4ed8
    classDef dependent fill:#d97706,color:#ffffff,stroke:#b45309
    classDef blocked fill:#475569,color:#ffffff,stroke:#334155

    subgraph Parallel
        A[Story A]:::independent
        B[Story B]:::independent
        C[Story C]:::independent
    end

    subgraph Sequential
        D[Story D]:::dependent --> E[Story E]:::dependent
        E --> F[Story F]:::dependent
    end

    A ~~~ D
```

- Stories without dependencies can be built **in parallel**
- Stories with dependencies must be built **in sequence**
- The PM system tracks `blocked_by` / `blocks` relationships with cycle detection

## The Comment Trail

Every status transition produces a comment. The full audit trail looks like:

```mermaid
flowchart TD
    classDef pm fill:#2563eb,color:#ffffff,stroke:#1d4ed8
    classDef dev fill:#d97706,color:#ffffff,stroke:#b45309
    classDef review fill:#7c3aed,color:#ffffff,stroke:#6d28d9
    classDef done fill:#059669,color:#ffffff,stroke:#047857

    A["PM creates story\n💬 Requirements + DoD"]:::pm
    A --> B["Dev picks up\n💬 Approach notes"]:::dev
    B --> C["Dev completes\n💬 What was built + critique results"]:::dev
    C --> D["Reviewer finds issues\n💬 Review findings"]:::review
    D --> E["Reviewer fixes\n💬 Fixes applied"]:::review
    E --> F["Fresh reviewer approves\n💬 Approved - zero issues"]:::review
    F --> G["Done\n💬 User can still comment"]:::done
```

## Definition of Done (DoD)

Every story must have a testable DoD before work begins. The default DoD is:

1. **Compiles** — zero build errors
2. **Tests pass** — all existing + new tests per acceptance criteria
3. **Acceptance criteria met** — every criterion verifiably satisfied
4. **No regressions** — what worked before still works
5. **Clean commit** — descriptive message referencing task ID

### Test Hierarchy

In order of preference:

1. **Property-based tests** — the gold standard for pure functions
2. **Integration tests** — for multi-component workflows
3. **Unit tests** — for isolated logic
4. **Manual testing** — only when automation is genuinely impractical

## Language Preferences

Preferred languages share a common trait: **strict type systems that catch errors at compile time**.

- **Rust** — systems, backend, CLI tools
- **Elm / Lamdera** — frontend, full-stack web apps
- **Haskell** — when maximum type safety matters
- **PureScript** — typed functional frontend alternative

The type system is not just a safety net — it's the **development methodology**. Start with types, let the compiler tell you what's missing.

## YAGNI in Practice

- Write the minimum code that satisfies the story's DoD
- No speculative features, no "might need this later"
- Three similar lines of code > premature abstraction
- If a helper is used once, inline it
- Simple and working beats elegant and theoretical

## Command Reference

| Command | Role | Purpose |
|---------|------|---------|
| `/pm` | Project Manager | Create stories with DoD, manage backlog |
| `/dev` | Developer | Pick up and implement stories |
| `/critique` | Code Reviewer | Review pending changes for critical issues |
| `/harden` | Hardener | Review → comment → fix → comment loop |
| `/sdlc` | Orchestrator | Automates dev → harden × N → done |
| `/review` | Reviewer | Manual review of tasks and bugs |

## End-to-End Flow

```mermaid
flowchart TB
    classDef pm fill:#2563eb,color:#ffffff,stroke:#1d4ed8
    classDef dev fill:#d97706,color:#ffffff,stroke:#b45309
    classDef critique fill:#dc2626,color:#ffffff,stroke:#b91c1c
    classDef review fill:#7c3aed,color:#ffffff,stroke:#6d28d9
    classDef done fill:#059669,color:#ffffff,stroke:#047857
    classDef auto fill:#0891b2,color:#ffffff,stroke:#0e7490

    PM["/pm creates story\nRequirements + DoD"]:::pm
    PM --> DEV["/dev picks up story\nStarts with types"]:::dev
    DEV --> BUILD["Build feature\nMinimum necessary code"]:::dev
    BUILD --> CRIT["/critique reviews\nPending changes"]:::critique
    CRIT --> FIX{"Critical\nissues?"}
    FIX -->|Yes| FIXEM["Fix issues\nFresh context"]:::dev
    FIXEM --> CRIT
    FIX -->|No| RFR["Move to\nready_for_review"]:::review
    RFR --> HARD["/harden picks up\nReview all changes"]:::review
    HARD --> ISSUES{"Issues\nfound?"}
    ISSUES -->|Yes| HFIX["Fix + comment\nBack to review"]:::review
    HFIX --> HARD
    ISSUES -->|No| DONE["Mark done ✓"]:::done

    SDLC["/sdlc automates\nthis entire flow"]:::auto
    SDLC -.-> DEV
    SDLC -.-> HARD
```
