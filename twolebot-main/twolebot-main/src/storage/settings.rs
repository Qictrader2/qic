use crate::error::{Result, TwolebotError};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// User settings for twolebot (separate from config which contains secrets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Show "Using tool: X" messages in output (default: false)
    #[serde(default)]
    pub show_tool_messages: bool,

    /// Show "Thinking:" messages in output (default: false)
    #[serde(default)]
    pub show_thinking_messages: bool,

    /// Show tool result messages in output (default: false)
    #[serde(default)]
    pub show_tool_results: bool,

    /// Whether the semantic indexer is paused (default: false = running)
    #[serde(default)]
    pub semantic_paused: bool,

    /// Max ONNX/OpenMP threads for semantic embedding work.
    #[serde(default = "default_omp_num_threads")]
    pub omp_num_threads: u16,

    /// Telegram username allowed to use this bot. None = allow everyone.
    #[serde(default)]
    pub allowed_username: Option<String>,

    /// Whether Telegram threading (topics in private chats) has been enabled.
    #[serde(default)]
    pub threading_enabled: bool,

    /// Chat harness used for Telegram/Web prompt execution (e.g. "claude", "echo").
    #[serde(default = "default_chat_harness")]
    pub chat_harness: String,

    /// Claude model to use (e.g. "claude-opus-4-6"). Overrides the CLI default.
    /// Stored per-instance in SQLite so multiple processes can use different models.
    #[serde(default = "default_claude_model")]
    pub claude_model: String,

    /// Role prompt template for the dev (implementation) phase of the SDLC loop.
    /// Task context is prepended automatically. Use $ARGUMENTS for task-specific instructions.
    #[serde(default = "default_dev_role_prompt")]
    pub dev_role_prompt: String,

    /// Role prompt template for the harden (review + fix) phase of the SDLC loop.
    /// Supports {iteration} and {max_iterations} placeholders.
    /// Use $ARGUMENTS for task-specific instructions.
    #[serde(default = "default_harden_role_prompt")]
    pub harden_role_prompt: String,

    /// Role prompt template for the PM (project manager) role.
    /// Used when /pm command is invoked in chat. Use $ARGUMENTS for user instructions.
    #[serde(default = "default_pm_role_prompt")]
    pub pm_role_prompt: String,
}

pub const DEFAULT_OMP_NUM_THREADS: u16 = 2;
const MIN_OMP_NUM_THREADS: u16 = 1;
const MAX_OMP_NUM_THREADS: u16 = 32;

fn default_omp_num_threads() -> u16 {
    DEFAULT_OMP_NUM_THREADS
}

fn default_chat_harness() -> String {
    "claude".to_string()
}

pub const DEFAULT_CLAUDE_MODEL: &str = "claude-opus-4-6";

fn default_claude_model() -> String {
    DEFAULT_CLAUDE_MODEL.to_string()
}

pub fn default_dev_role_prompt() -> String {
    r#"You are a senior developer on the project.

**Project Identification**: First, run `git remote get-url origin` to get the git remote URL, then use the project lookup MCP tool with that URL to find your project context.

**Core Responsibilities:**
Write high-quality code that follows project conventions. Fix bugs, implement features, and ensure all code compiles and tests pass. Use MCP comment tools to document your progress and communicate with the team.

**Testing Requirements (CRITICAL):**
Every ticket must include automated tests unless the ticket explicitly states manual testing criteria. Tests are not optional extras - they ARE the acceptance criteria.

*Test Hierarchy (in order of preference):*
1. **Property-based tests** - the gold standard for pure functions and data transformations
2. **Integration tests** - for workflows involving multiple components or external systems
3. **Unit tests** - for isolated logic where property-based testing is impractical
4. **Manual testing** - ONLY when the ticket explicitly requires it (rare)

*Property-Based Testing Standards:*
When writing property-based tests, your properties must be:
- **Non-trivial**: NOT just "roundtrip works" or "doesn't crash". Express business invariants or mathematical laws.
- **Meaningful**: Should catch real bugs. "sorted list has same length" catches nothing. "sorted list contains all original elements AND is in ascending order" catches real bugs.
- **Implementation-independent**: Properties describe WHAT must be true, not HOW. Don't reimplement the function in the test.

*Good properties:*
- "Withdrawal never results in negative balance" (business invariant)
- "Encoding then decoding yields original value" (roundtrip - when both directions are non-trivial)
- "Merging two sorted lists yields a sorted list containing all elements from both" (mathematical law)

*Bad properties:*
- "Function returns a value" (trivial)
- "Output type is correct" (compiler's job)
- "Large input doesn't timeout" (benchmark, not property)

*Property Categories (Scott Wlaschin + John Hughes):*

**Foundational (Wlaschin):**
1. **Different paths, same destination** - `add(a,b) = add(b,a)`
2. **There and back again** - `decode(encode(x)) = x`
3. **Some things never change** - `sort(list).length = list.length`
4. **The more things change, the more they stay the same** - `sort(sort(x)) = sort(x)`
5. **Solve a smaller problem first** - `sum(list) = head + sum(tail)`
6. **Hard to prove, easy to verify** - factorization is hard, multiplication is easy
7. **Test oracle** - compare against known-good alternate implementation

**Metamorphic (T.Y. Chen) - when you have no oracle:**
8. **Perturbation invariance** - small input change → proportional output change
9. **Monotonicity** - more input → more (or not less) output
10. **Subset/superset** - filtering results ⊆ unfiltered results

**Stateful (John Hughes) - for APIs and systems:**
11. **Model conformance** - real system matches simplified model (e.g., DB ops match dictionary)
12. **State invariants** - property holds in ALL reachable states (e.g., balance ≥ 0)
13. **Valid transitions** - only legal state changes occur (e.g., can't withdraw when overdrawn)

**Algebraic - for data structures:**
14. **Monoid laws** - `empty + a = a`, `a + empty = a`, `(a+b)+c = a+(b+c)`
15. **Relational** - reflexive (`a=a`), symmetric, transitive, antisymmetric as appropriate

**Concurrent (Jepsen/Hughes):**
16. **Linearizability** - concurrent ops appear atomic at some point between call and return
17. **No lost updates** - parallel writes don't silently drop data

If a ticket has manual testing criteria, follow the checklist exactly and report results in your completion comment.

**Definition of Done (DoD):**
A task is complete ONLY when ALL are true:
1. **Compiles** - Build passes with zero errors
2. **Tests pass** - All existing + new tests per acceptance criteria
3. **Acceptance criteria met** - Every criterion verifiably satisfied
4. **No regressions** - What worked before still works
5. **Clean commit** - Descriptive message referencing task ID

$ARGUMENTS"#
        .to_string()
}

pub fn default_harden_role_prompt() -> String {
    r#"You are a senior code hardener in an automated SDLC loop.

**Project Identification**: Run `git remote get-url origin`, then use the project lookup MCP tool to find your project.

---

# THE HARDEN FLOW

```
┌─────────────────────────────────────────────────────────────┐
│  1. GET TASK         →  Pick up review task                 │
│  2. REVIEW           →  Analyze ALL code changes            │
│  3. COMMENT #1       →  Document findings in MCP            │
│  4. FIX              →  Apply fixes (if needed)             │
│  5. COMMENT #2       →  Document what was fixed in MCP      │
│  6. LEAVE IN REVIEW  →  Fresh reviewer verifies next loop   │
└─────────────────────────────────────────────────────────────┘
```

---

## STEP 1: GET TASK

Read task description + all existing comments.

---

## STEP 2: REVIEW CODE

**Read EVERYTHING before writing anything:**

1. Check what files were modified for this task
2. Read all relevant code
3. Compare against ticket requirements
4. Look for:
   - Bugs, logic errors
   - Security issues
   - Missing error handling
   - Style violations
   - Missing/inadequate tests
   - Incomplete implementation

---

## STEP 3: COMMENT ON FINDINGS

**MANDATORY - DO NOT SKIP**

Post your review using the comment MCP tool:

```
## Review Findings

**Reviewer:** Harden Bot (automated)
**Verdict:** NEEDS_FIXES | APPROVED

### Issues Found:

1. **[HIGH]** `src/Foo.elm:42` - description
   - Problem: ...
   - Fix needed: ...

2. **[MEDIUM]** `src/Bar.elm:87` - description
   - Problem: ...
   - Fix needed: ...

### What's Good:
- ...
```

**If no issues found**, still comment with APPROVED verdict, then skip to Step 6 and mark as done.

---

## STEP 4: FIX ISSUES

**ONLY after Step 3 comment is posted.**

For each issue you documented:
1. Make the code fix
2. Verify it compiles/works
3. Track what you fixed

---

## STEP 5: COMMENT ON FIXES

**MANDATORY - DO NOT SKIP**

Post another comment documenting what you fixed:

```
## Fixes Applied

1. `src/Foo.elm:42` - Added Maybe.withDefault handling
2. `src/Bar.elm:87` - Added Result.mapError with toast notification

**Build status:** Compiles successfully
```

---

## STEP 6: SET STATUS

**If you made ANY fixes:**
- **KEEP as `ready_for_review`** (do NOT mark done)
- A fresh reviewer will verify your fixes in the next iteration
- Output: `VERDICT: NEEDS_REVIEW`

**If task was PERFECT (zero fixes):**
- Mark as `done`
- Output: `VERDICT: DONE`

**If task is blocked or has unfixable issues:**
- Leave status as-is
- Output: `VERDICT: BLOCKED`

**IMPORTANT:** Always output one of these exact VERDICT lines at the end!

---

## RULES

1. **Always comment BEFORE fixing** - creates audit trail
2. **Always comment AFTER fixing** - documents what changed
3. **Never mark done if you made fixes** - needs fresh eyes
4. **Be thorough** - catch issues now, not in production
5. **Run compile/build** - verify fixes don't break things

$ARGUMENTS"#
        .to_string()
}

pub fn default_pm_role_prompt() -> String {
    r#"You are a technical project manager on the project. Your goal is to verify and look at code and tickets using MCP tools, create tickets, and keep the project on target. Main goal is to ensure we don't break Claude rules, follow rules to the T, and maintain world-class quality. Use MCP comment tools to provide clear feedback on tasks, track progress discussions, and maintain communication threads with your team.

**Project Identification**: If you're unsure which project you're working in, run `git remote -v` to get the git remote URL, then use the project list MCP tool with the `git_remote_url` parameter to quickly find your project. You can also use create/update project tools with the `git_remote_url` parameter to associate projects with their git repositories.

**Epic Management Rules (CRITICAL):**
- NEVER create epics for Testing or Documentation - these are part of every story
- ONLY create an epic AFTER you already have 3+ related stories that justify grouping
- Epic names should be SHORT and concise (2-4 words): "User Authentication", "Payment System"
- Epics should represent major user-facing capabilities or architectural systems
- If you're creating epics for single components that could be one story, you're over-decomposing

**Story Creation Philosophy:**
When breaking down requirements into tickets, prefer creating stories that deliver ONE discrete piece of client-valued functionality. OAuth integration should be ONE story, not decomposed further. "Admin page with user grid and role assignment" should be ONE story. Start with user value, decompose ONLY if technically necessary.

**Story Content Guidelines:**
Stories should contain what needs to be built and where to look, not detailed implementation. Include relevant file paths, API examples, or DB schema if they help the dev, but avoid code examples unless absolutely necessary. Stories should NOT be more work to write than to execute.

**Acceptance Criteria & Testing (CRITICAL):**
Every story MUST include verifiable acceptance criteria. The default expectation is that acceptance = automated tests.

*Test Hierarchy (in order of preference):*
1. **Property-based tests** - the gold standard for pure functions and data transformations
2. **Integration tests** - for workflows involving multiple components or external systems
3. **Unit tests** - for isolated logic where property-based testing is impractical
4. **Manual testing criteria** - ONLY when automation is genuinely impractical

*Property-Based Testing Requirements:*
When specifying property-based tests, define properties that are:
- **Non-trivial**: NOT just "roundtrip works" or "doesn't crash"
- **Meaningful**: Properties should catch real bugs
- **Independent of implementation**: Properties describe WHAT must be true, not HOW

**Developer Success Focus:**
Your primary role is to help developers succeed. They need concise context - exactly as much as needed to complete a ticket. Don't give too little context, don't overwhelm with context, and definitely don't misdirect them.

**Technical Guidelines:**
Descriptions should contain direct files the ticket needs to address (if known), examples of curl requests if interaction with external systems is needed, Mermaid diagrams if helpful - but remain concise. Avoid external libraries unless obviously the best choice. Think simpler, focus on working software, route to the goal in fewest steps.

**Definition of Done (DoD):**
A task is complete ONLY when ALL are true:
1. **Compiles** - Build passes with zero errors
2. **Tests pass** - All existing + new tests per acceptance criteria
3. **Acceptance criteria met** - Every criterion verifiably satisfied
4. **No regressions** - What worked before still works
5. **Clean commit** - Descriptive message referencing task ID

$ARGUMENTS"#
        .to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_tool_messages: false,
            show_thinking_messages: false,
            show_tool_results: false,
            semantic_paused: false,
            omp_num_threads: default_omp_num_threads(),
            allowed_username: None,
            threading_enabled: false,
            chat_harness: default_chat_harness(),
            claude_model: default_claude_model(),
            dev_role_prompt: default_dev_role_prompt(),
            harden_role_prompt: default_harden_role_prompt(),
            pm_role_prompt: default_pm_role_prompt(),
        }
    }
}

/// Thread-safe settings store (SQLite-backed in runtime DB)
pub struct SettingsStore {
    db_path: std::path::PathBuf,
    settings: Arc<RwLock<Settings>>,
}

impl SettingsStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let store = Self {
            db_path,
            settings: Arc::new(RwLock::new(Settings::default())),
        };
        store.init_schema()?;
        let loaded = store.load_from_db()?;
        {
            let mut guard = match store.settings.write() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            *guard = loaded;
        }
        Ok(store)
    }

    fn conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| TwolebotError::storage(format!("open settings db: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TwolebotError::storage(format!("set WAL mode: {e}")))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TwolebotError::storage(format!("set synchronous: {e}")))?;
        Ok(conn)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS runtime_settings (
                key        TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            );",
        )
        .map_err(|e| TwolebotError::storage(format!("init settings schema: {e}")))?;
        Ok(())
    }

    fn load_from_db(&self) -> Result<Settings> {
        let conn = self.conn()?;
        let value: Option<String> = conn
            .query_row(
                "SELECT value_json FROM runtime_settings WHERE key = 'app'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| TwolebotError::storage(format!("load settings: {e}")))?;

        match value {
            Some(json) => {
                let parsed: Settings = serde_json::from_str(&json).map_err(TwolebotError::from)?;
                Ok(Self::normalize(parsed))
            }
            None => Ok(Self::normalize(Settings::default())),
        }
    }

    fn persist_to_db(&self, settings: &Settings) -> Result<()> {
        let conn = self.conn()?;
        let json = serde_json::to_string(settings)?;
        conn.execute(
            "INSERT INTO runtime_settings (key, value_json, updated_at)
             VALUES ('app', ?1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at = excluded.updated_at",
            params![json],
        )
        .map_err(|e| TwolebotError::storage(format!("persist settings: {e}")))?;
        Ok(())
    }

    /// Get current settings
    pub fn get(&self) -> Settings {
        match self.settings.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                tracing::warn!("Settings lock poisoned; continuing with inner value");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Update settings and persist to DB
    pub fn update(&self, settings: Settings) -> Result<()> {
        let settings = Self::normalize(settings);
        {
            match self.settings.write() {
                Ok(mut guard) => {
                    *guard = settings.clone();
                }
                Err(poisoned) => {
                    tracing::warn!("Settings lock poisoned; continuing with inner value");
                    let mut guard = poisoned.into_inner();
                    *guard = settings.clone();
                }
            }
        }

        self.persist_to_db(&settings)
    }

    /// Update a single setting
    pub fn set_show_tool_messages(&self, value: bool) -> Result<()> {
        let mut settings = self.get();
        settings.show_tool_messages = value;
        self.update(settings)
    }

    pub fn set_semantic_paused(&self, value: bool) -> Result<()> {
        let mut settings = self.get();
        settings.semantic_paused = value;
        self.update(settings)
    }

    pub fn set_omp_num_threads(&self, value: u16) -> Result<()> {
        let mut settings = self.get();
        settings.omp_num_threads = value;
        self.update(settings)
    }

    pub fn set_allowed_username(&self, value: Option<String>) -> Result<()> {
        let mut settings = self.get();
        settings.allowed_username = value;
        self.update(settings)
    }

    pub fn set_threading_enabled(&self, value: bool) -> Result<()> {
        let mut settings = self.get();
        settings.threading_enabled = value;
        self.update(settings)
    }

    pub fn set_chat_harness(&self, value: impl Into<String>) -> Result<()> {
        let mut settings = self.get();
        settings.chat_harness = value.into();
        self.update(settings)
    }

    fn normalize(mut settings: Settings) -> Settings {
        if settings.omp_num_threads < MIN_OMP_NUM_THREADS
            || settings.omp_num_threads > MAX_OMP_NUM_THREADS
        {
            settings.omp_num_threads = settings
                .omp_num_threads
                .clamp(MIN_OMP_NUM_THREADS, MAX_OMP_NUM_THREADS);
        }
        let normalized_harness = settings.chat_harness.trim().to_ascii_lowercase();
        settings.chat_harness = if normalized_harness.is_empty() {
            default_chat_harness()
        } else {
            normalized_harness
        };
        let trimmed_model = settings.claude_model.trim().to_string();
        settings.claude_model = if trimmed_model.is_empty() {
            default_claude_model()
        } else {
            trimmed_model
        };
        if settings.dev_role_prompt.trim().is_empty() {
            settings.dev_role_prompt = default_dev_role_prompt();
        }
        if settings.harden_role_prompt.trim().is_empty() {
            settings.harden_role_prompt = default_harden_role_prompt();
        }
        if settings.pm_role_prompt.trim().is_empty() {
            settings.pm_role_prompt = default_pm_role_prompt();
        }
        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(!settings.show_tool_messages);
        assert!(!settings.show_thinking_messages);
        assert!(!settings.show_tool_results);
        assert_eq!(settings.omp_num_threads, DEFAULT_OMP_NUM_THREADS);
        assert_eq!(settings.chat_harness, "claude");
        assert_eq!(settings.claude_model, DEFAULT_CLAUDE_MODEL);
        assert!(!settings.dev_role_prompt.is_empty());
        assert!(!settings.harden_role_prompt.is_empty());
        assert!(!settings.pm_role_prompt.is_empty());
        assert!(settings.dev_role_prompt.contains("$ARGUMENTS"));
        assert!(settings.harden_role_prompt.contains("$ARGUMENTS"));
        assert!(settings.pm_role_prompt.contains("$ARGUMENTS"));
    }

    #[test]
    fn test_settings_store_new() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap();
        let settings = store.get();
        assert!(!settings.show_tool_messages);
    }

    #[test]
    fn test_settings_store_update() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("runtime.sqlite3");
        let store = SettingsStore::new(&db_path).unwrap();

        store.set_show_tool_messages(true).unwrap();
        assert!(store.get().show_tool_messages);

        // Verify persistence
        let store2 = SettingsStore::new(&db_path).unwrap();
        assert!(store2.get().show_tool_messages);
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let settings = Settings {
            show_tool_messages: true,
            show_thinking_messages: false,
            show_tool_results: true,
            semantic_paused: true,
            omp_num_threads: 4,
            allowed_username: Some("schalk".to_string()),
            threading_enabled: false,
            chat_harness: "echo".to_string(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            dev_role_prompt: "custom dev prompt $ARGUMENTS".to_string(),
            harden_role_prompt: "custom harden prompt $ARGUMENTS".to_string(),
            pm_role_prompt: "custom pm prompt $ARGUMENTS".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.show_tool_messages, parsed.show_tool_messages);
        assert_eq!(
            settings.show_thinking_messages,
            parsed.show_thinking_messages
        );
        assert_eq!(settings.show_tool_results, parsed.show_tool_results);
        assert_eq!(settings.semantic_paused, parsed.semantic_paused);
        assert_eq!(settings.omp_num_threads, parsed.omp_num_threads);
        assert_eq!(settings.allowed_username, parsed.allowed_username);
        assert_eq!(settings.chat_harness, parsed.chat_harness);
        assert_eq!(settings.claude_model, parsed.claude_model);
        assert_eq!(settings.dev_role_prompt, parsed.dev_role_prompt);
        assert_eq!(settings.harden_role_prompt, parsed.harden_role_prompt);
        assert_eq!(settings.pm_role_prompt, parsed.pm_role_prompt);
    }

    #[test]
    fn test_allowed_username_backwards_compat() {
        // Old JSON without allowed_username should deserialize to None
        let json = r#"{"show_tool_messages":false,"show_thinking_messages":false,"show_tool_results":false,"semantic_paused":false,"omp_num_threads":2}"#;
        let parsed: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.allowed_username, None);
        assert_eq!(parsed.chat_harness, "claude");
        assert_eq!(parsed.claude_model, DEFAULT_CLAUDE_MODEL);
        // Role prompts should get defaults from serde
        assert!(!parsed.dev_role_prompt.is_empty());
        assert!(!parsed.harden_role_prompt.is_empty());
        assert!(!parsed.pm_role_prompt.is_empty());
    }

    #[test]
    fn test_allowed_username_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("runtime.sqlite3");
        let store = SettingsStore::new(&db_path).unwrap();

        assert_eq!(store.get().allowed_username, None);

        store
            .set_allowed_username(Some("testuser".to_string()))
            .unwrap();
        assert_eq!(store.get().allowed_username, Some("testuser".to_string()));

        // Verify persistence across reload
        let store2 = SettingsStore::new(&db_path).unwrap();
        assert_eq!(store2.get().allowed_username, Some("testuser".to_string()));

        // Clear it
        store2.set_allowed_username(None).unwrap();
        assert_eq!(store2.get().allowed_username, None);
    }

    #[test]
    fn test_omp_threads_normalized() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("runtime.sqlite3");
        let store = SettingsStore::new(&db_path).unwrap();

        store.set_omp_num_threads(0).unwrap();
        assert_eq!(store.get().omp_num_threads, MIN_OMP_NUM_THREADS);

        store.set_omp_num_threads(1000).unwrap();
        assert_eq!(store.get().omp_num_threads, MAX_OMP_NUM_THREADS);
    }

    #[test]
    fn test_chat_harness_normalized() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("runtime.sqlite3");
        let store = SettingsStore::new(&db_path).unwrap();

        store.set_chat_harness("  ECHO  ").unwrap();
        assert_eq!(store.get().chat_harness, "echo");

        store.set_chat_harness("   ").unwrap();
        assert_eq!(store.get().chat_harness, "claude");
    }
}
