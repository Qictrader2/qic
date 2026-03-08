use crate::error::{Result, TwolebotError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptSource {
    Telegram {
        update_id: i64,
        message_id: i64,
        chat_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_thread_id: Option<i64>,
    },
    Cron {
        job_id: String,
        execution_id: String,
        /// Human-readable name for the job
        job_name: String,
        /// Target Telegram chat for response routing (looked up at promotion time)
        #[serde(skip_serializing_if = "Option::is_none", default)]
        chat_id: Option<i64>,
        /// Target thread for response routing
        #[serde(skip_serializing_if = "Option::is_none", default)]
        message_thread_id: Option<i64>,
    },
    Web {
        /// Unique conversation identifier (UUID)
        conversation_id: String,
    },
}

impl PromptSource {
    pub fn telegram(
        update_id: i64,
        message_id: i64,
        chat_id: i64,
        message_thread_id: Option<i64>,
    ) -> Self {
        PromptSource::Telegram {
            update_id,
            message_id,
            chat_id,
            message_thread_id,
        }
    }

    pub fn cron(
        job_id: impl Into<String>,
        execution_id: impl Into<String>,
        job_name: impl Into<String>,
    ) -> Self {
        PromptSource::Cron {
            job_id: job_id.into(),
            execution_id: execution_id.into(),
            job_name: job_name.into(),
            chat_id: None,
            message_thread_id: None,
        }
    }

    /// Create a Cron source with Telegram routing info for response delivery.
    pub fn cron_routed(
        job_id: impl Into<String>,
        execution_id: impl Into<String>,
        job_name: impl Into<String>,
        chat_id: i64,
        message_thread_id: Option<i64>,
    ) -> Self {
        PromptSource::Cron {
            job_id: job_id.into(),
            execution_id: execution_id.into(),
            job_name: job_name.into(),
            chat_id: Some(chat_id),
            message_thread_id,
        }
    }

    pub fn web(conversation_id: impl Into<String>) -> Self {
        PromptSource::Web {
            conversation_id: conversation_id.into(),
        }
    }

    /// Get the message ID for reaction setting
    pub fn message_id(&self) -> Option<i64> {
        match self {
            PromptSource::Telegram { message_id, .. } => Some(*message_id),
            PromptSource::Cron { .. } | PromptSource::Web { .. } => None,
        }
    }

    /// Get the chat ID (Telegram source or routed Cron)
    pub fn chat_id(&self) -> Option<i64> {
        match self {
            PromptSource::Telegram { chat_id, .. } => Some(*chat_id),
            PromptSource::Cron { chat_id, .. } => *chat_id,
            PromptSource::Web { .. } => None,
        }
    }

    /// Get the message thread ID (Telegram topic or routed Cron)
    pub fn message_thread_id(&self) -> Option<i64> {
        match self {
            PromptSource::Telegram {
                message_thread_id, ..
            } => *message_thread_id,
            PromptSource::Cron {
                message_thread_id, ..
            } => *message_thread_id,
            PromptSource::Web { .. } => None,
        }
    }

    /// Derive a topic routing key: None = main thread, Some = topic workspace
    pub fn topic_key(&self) -> Option<String> {
        match self {
            PromptSource::Telegram {
                chat_id,
                message_thread_id: Some(tid),
                ..
            } => Some(format!("{}_{}", chat_id, tid)),
            PromptSource::Cron {
                chat_id: Some(cid),
                message_thread_id: Some(tid),
                ..
            } => Some(format!("{}_{}", cid, tid)),
            PromptSource::Web { conversation_id } => {
                Some(format!("web_{}", conversation_id))
            }
            _ => None,
        }
    }

    /// Get a display name for the source (for acknowledgments)
    pub fn display_name(&self) -> String {
        match self {
            PromptSource::Telegram { message_id, .. } => format!("message #{}", message_id),
            PromptSource::Cron { job_name, .. } => job_name.clone(),
            PromptSource::Web { .. } => "web chat".to_string(),
        }
    }

    /// Get the web conversation ID, if this is a Web source
    pub fn conversation_id(&self) -> Option<&str> {
        match self {
            PromptSource::Web { conversation_id } => Some(conversation_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl PromptStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PromptStatus::Pending => "pending",
            PromptStatus::Running => "running",
            PromptStatus::Completed => "completed",
            PromptStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(PromptStatus::Pending),
            "running" => Some(PromptStatus::Running),
            "completed" => Some(PromptStatus::Completed),
            "failed" => Some(PromptStatus::Failed),
            _ => None,
        }
    }

    pub fn directory(&self) -> &'static str {
        match self {
            PromptStatus::Pending => "pending",
            PromptStatus::Running => "running",
            PromptStatus::Completed | PromptStatus::Failed => "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptItem {
    pub id: String,
    pub source: PromptSource,
    pub user_id: i64,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_path: Option<String>,
    /// Topic routing key: None = main thread, Some("{chat_id}_{thread_id}") = topic workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_key: Option<String>,
    pub status: PromptStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PromptItem {
    pub fn new(source: PromptSource, user_id: i64, prompt: impl Into<String>) -> Self {
        let topic_key = source.topic_key();
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            user_id,
            prompt: prompt.into(),
            media_path: None,
            topic_key,
            status: PromptStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    pub fn with_media(mut self, media_path: impl Into<String>) -> Self {
        self.media_path = Some(media_path.into());
        self
    }

}

/// Manages the prompt feed queue in SQLite (transactional, crash-safe).
pub struct PromptFeed {
    conn: Mutex<Connection>,
}

impl PromptFeed {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open prompt queue db: {}", e)))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TwolebotError::storage(format!("set WAL mode: {}", e)))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TwolebotError::storage(format!("set synchronous: {}", e)))?;

        let feed = Self { conn: Mutex::new(conn) };
        feed.init_schema()?;
        Ok(feed)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                source_json TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                prompt_text TEXT NOT NULL,
                media_path TEXT,
                topic_key TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                error TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_prompts_status_created
                ON prompts(status, created_at);
            COMMIT;",
        )
        .map_err(|e| TwolebotError::storage(format!("init prompt schema: {}", e)))?;

        // Migration: add topic_key column to existing tables
        let has_topic_key = conn
            .prepare("SELECT topic_key FROM prompts LIMIT 0")
            .is_ok();
        if !has_topic_key {
            conn.execute_batch("ALTER TABLE prompts ADD COLUMN topic_key TEXT;")
                .map_err(|e| {
                    TwolebotError::storage(format!("migrate prompts add topic_key: {}", e))
                })?;
            tracing::info!("Migrated prompts table: added topic_key column");
        }

        // Create index after migration ensures column exists
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_prompts_topic_key
                ON prompts(topic_key);",
        )
        .map_err(|e| TwolebotError::storage(format!("create topic_key index: {}", e)))?;

        Ok(())
    }

    fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptItem> {
        let source_json: String = row.get("source_json")?;
        let source: PromptSource = serde_json::from_str(&source_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                source_json.len(),
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })?;
        let status_text: String = row.get("status")?;
        let status = PromptStatus::from_str(&status_text).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                status_text.len(),
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid prompt status '{}'", status_text),
                )),
            )
        })?;

        let created_at: String = row.get("created_at")?;
        let started_at: Option<String> = row.get("started_at")?;
        let completed_at: Option<String> = row.get("completed_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);
        let started_at = started_at
            .map(|ts| DateTime::parse_from_rfc3339(&ts).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
        let completed_at = completed_at
            .map(|ts| DateTime::parse_from_rfc3339(&ts).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        Ok(PromptItem {
            id: row.get("id")?,
            source,
            user_id: row.get("user_id")?,
            prompt: row.get("prompt_text")?,
            media_path: row.get("media_path")?,
            topic_key: row.get("topic_key")?,
            status,
            created_at,
            started_at,
            completed_at,
            error: row.get("error")?,
        })
    }

    fn upsert_item(conn: &Connection, item: &PromptItem) -> Result<()> {
        conn.execute(
            "INSERT INTO prompts (
                id, source_json, user_id, prompt_text, media_path, topic_key, status,
                created_at, started_at, completed_at, error
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                source_json=excluded.source_json,
                user_id=excluded.user_id,
                prompt_text=excluded.prompt_text,
                media_path=excluded.media_path,
                topic_key=excluded.topic_key,
                status=excluded.status,
                created_at=excluded.created_at,
                started_at=excluded.started_at,
                completed_at=excluded.completed_at,
                error=excluded.error",
            params![
                item.id,
                serde_json::to_string(&item.source)?,
                item.user_id,
                item.prompt,
                item.media_path,
                item.topic_key,
                item.status.as_str(),
                item.created_at.to_rfc3339(),
                item.started_at.map(|d| d.to_rfc3339()),
                item.completed_at.map(|d| d.to_rfc3339()),
                item.error,
            ],
        )
        .map_err(|e| TwolebotError::storage(format!("upsert prompt item: {}", e)))?;
        Ok(())
    }

    /// Add a new prompt to the pending queue.
    pub fn enqueue(&self, item: PromptItem) -> Result<PromptItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        Self::upsert_item(&conn, &item)?;
        Ok(item)
    }

    /// Get the next pending prompt (oldest first).
    pub fn next_pending(&self) -> Result<Option<PromptItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        conn.query_row(
            "SELECT * FROM prompts WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1",
            [],
            Self::row_to_item,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("next pending prompt: {}", e)))
    }

    /// Get all pending prompts (oldest first).
    pub fn all_pending(&self) -> Result<Vec<PromptItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let mut stmt = conn
            .prepare("SELECT * FROM prompts WHERE status = 'pending' ORDER BY created_at ASC")
            .map_err(|e| TwolebotError::storage(format!("prepare all_pending: {}", e)))?;
        let rows = stmt
            .query_map([], Self::row_to_item)
            .map_err(|e| TwolebotError::storage(format!("query all_pending: {}", e)))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| TwolebotError::storage(format!("row all_pending: {}", e)))?);
        }
        Ok(items)
    }

    /// Returns true if there is at least one pending user-driven prompt.
    pub fn has_pending_user_prompts(&self) -> Result<bool> {
        Ok(self
            .all_pending()?
            .into_iter()
            .any(|item| matches!(item.source, PromptSource::Telegram { .. } | PromptSource::Web { .. })))
    }

    /// Mark a prompt as running using an atomic status transition.
    pub fn mark_running(&self, id: &str) -> Result<PromptItem> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| TwolebotError::storage(format!("begin mark_running tx: {}", e)))?;

        let affected = tx
            .execute(
                "UPDATE prompts
                 SET status = 'running', started_at = ?1
                 WHERE id = ?2 AND status = 'pending'",
                params![now, id],
            )
            .map_err(|e| TwolebotError::storage(format!("mark running: {}", e)))?;
        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "pending prompt {} not found",
                id
            )));
        }

        let item = tx
            .query_row(
                "SELECT * FROM prompts WHERE id = ?1",
                params![id],
                Self::row_to_item,
            )
            .map_err(|e| TwolebotError::storage(format!("load running prompt: {}", e)))?;

        tx.commit()
            .map_err(|e| TwolebotError::storage(format!("commit mark_running tx: {}", e)))?;
        Ok(item)
    }

    /// Mark a prompt as completed.
    pub fn mark_completed(&self, id: &str) -> Result<PromptItem> {
        self.set_terminal_status(id, PromptStatus::Completed, None)
    }

    /// Mark a prompt as failed.
    pub fn mark_failed(&self, id: &str, error: impl Into<String>) -> Result<PromptItem> {
        self.set_terminal_status(id, PromptStatus::Failed, Some(error.into()))
    }

    fn set_terminal_status(
        &self,
        id: &str,
        status: PromptStatus,
        error: Option<String>,
    ) -> Result<PromptItem> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| TwolebotError::storage(format!("begin set_terminal_status tx: {}", e)))?;

        let affected = tx
            .execute(
                "UPDATE prompts
                 SET status = ?1, completed_at = ?2, error = ?3
                 WHERE id = ?4 AND status = 'running'",
                params![status.as_str(), now, error, id],
            )
            .map_err(|e| TwolebotError::storage(format!("set terminal status: {}", e)))?;
        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "running prompt {} not found",
                id
            )));
        }

        let item = tx
            .query_row(
                "SELECT * FROM prompts WHERE id = ?1",
                params![id],
                Self::row_to_item,
            )
            .map_err(|e| TwolebotError::storage(format!("load terminal prompt: {}", e)))?;

        tx.commit()
            .map_err(|e| TwolebotError::storage(format!("commit set_terminal_status tx: {}", e)))?;
        Ok(item)
    }

    /// Check if there is a running prompt for a specific topic.
    pub fn has_running_for_topic(&self, topic_key: &Option<String>) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let count: i64 = match topic_key {
            Some(key) => conn
                .query_row(
                    "SELECT COUNT(*) FROM prompts WHERE status = 'running' AND topic_key = ?1",
                    params![key],
                    |r| r.get(0),
                )
                .map_err(|e| {
                    TwolebotError::storage(format!("has_running_for_topic: {}", e))
                })?,
            None => conn
                .query_row(
                    "SELECT COUNT(*) FROM prompts WHERE status = 'running' AND topic_key IS NULL",
                    [],
                    |r| r.get(0),
                )
                .map_err(|e| {
                    TwolebotError::storage(format!("has_running_for_topic (null): {}", e))
                })?,
        };
        Ok(count > 0)
    }

    /// Check if there are pending user prompts for a specific topic.
    pub fn has_pending_user_prompts_for_topic(&self, topic_key: &Option<String>) -> Result<bool> {
        Ok(self
            .all_pending()?
            .into_iter()
            .any(|item| {
                matches!(item.source, PromptSource::Telegram { .. } | PromptSource::Web { .. })
                    && item.topic_key == *topic_key
            }))
    }

    /// Get the currently running prompt (if any). Returns the oldest one for backwards compat.
    pub fn get_running(&self) -> Result<Option<PromptItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        conn.query_row(
            "SELECT * FROM prompts WHERE status = 'running' ORDER BY started_at ASC LIMIT 1",
            [],
            Self::row_to_item,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get running prompt: {}", e)))
    }

    /// Get all currently running prompts (for multi-worker typing indicators).
    pub fn get_all_running(&self) -> Result<Vec<PromptItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let mut stmt = conn
            .prepare("SELECT * FROM prompts WHERE status = 'running' ORDER BY started_at ASC")
            .map_err(|e| TwolebotError::storage(format!("prepare get_all_running: {}", e)))?;
        let rows = stmt
            .query_map([], Self::row_to_item)
            .map_err(|e| TwolebotError::storage(format!("query get_all_running: {}", e)))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(
                row.map_err(|e| TwolebotError::storage(format!("row get_all_running: {}", e)))?,
            );
        }
        Ok(items)
    }

    /// Get a prompt by ID.
    pub fn get(&self, id: &str) -> Result<Option<PromptItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        conn.query_row(
            "SELECT * FROM prompts WHERE id = ?1",
            params![id],
            Self::row_to_item,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get prompt by id: {}", e)))
    }

    /// Recover orphaned running prompts (service restart safety).
    pub fn recover_orphaned_running(&self) -> Result<usize> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let count = conn
            .execute(
                "UPDATE prompts
                 SET status = 'failed', completed_at = ?1, error = COALESCE(error, 'Orphaned after service restart')
                 WHERE status = 'running'",
                params![now],
            )
            .map_err(|e| TwolebotError::storage(format!("recover orphaned prompts: {}", e)))?;
        Ok(count)
    }

    /// Cancel a pending prompt.
    pub fn cancel(&self, id: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let count = conn
            .execute(
                "DELETE FROM prompts WHERE id = ?1 AND status = 'pending'",
                params![id],
            )
            .map_err(|e| TwolebotError::storage(format!("cancel prompt: {}", e)))?;
        Ok(count > 0)
    }

    /// List recent completed prompts.
    pub fn recent_completed(&self, limit: usize) -> Result<Vec<PromptItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("prompt db mutex poisoned"))?;
        let mut stmt = conn
            .prepare(
                "SELECT * FROM prompts
                 WHERE status IN ('completed', 'failed')
                 ORDER BY completed_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare recent_completed: {}", e)))?;
        let rows = stmt
            .query_map(params![limit as i64], Self::row_to_item)
            .map_err(|e| TwolebotError::storage(format!("query recent_completed: {}", e)))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(
                row.map_err(|e| TwolebotError::storage(format!("row recent_completed: {}", e)))?,
            );
        }
        Ok(items)
    }

    /// Count pending prompts.
    pub fn pending_count(&self) -> usize {
        self.count_by_status("pending")
    }

    /// Count completed/failed prompts.
    pub fn completed_count(&self) -> usize {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        conn.query_row(
            "SELECT COUNT(*) FROM prompts WHERE status IN ('completed', 'failed')",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n as usize)
        .unwrap_or(0)
    }

    fn count_by_status(&self, status: &str) -> usize {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        conn.query_row(
            "SELECT COUNT(*) FROM prompts WHERE status = ?1",
            params![status],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n as usize)
        .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_prompt_feed_basic() {
        let dir = tempdir().unwrap();
        let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        // Enqueue a prompt
        let item = PromptItem::new(PromptSource::telegram(1, 100, 123, None), 456, "Hello, Claude!");
        let enqueued = feed.enqueue(item.clone()).unwrap();
        assert_eq!(enqueued.status, PromptStatus::Pending);

        // Get next pending
        let next = feed.next_pending().unwrap().unwrap();
        assert_eq!(next.id, enqueued.id);

        // Mark running
        let running = feed.mark_running(&next.id).unwrap();
        assert_eq!(running.status, PromptStatus::Running);
        assert!(running.started_at.is_some());

        // Verify no more pending
        assert!(feed.next_pending().unwrap().is_none());

        // Mark completed
        let completed = feed.mark_completed(&running.id).unwrap();
        assert_eq!(completed.status, PromptStatus::Completed);
        assert!(completed.completed_at.is_some());
    }

    #[test]
    fn test_prompt_feed_ordering() {
        let dir = tempdir().unwrap();
        let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        // Enqueue multiple prompts
        for i in 0..3 {
            let item = PromptItem::new(
                PromptSource::telegram(i, i * 10, 123, None),
                456,
                format!("Message {}", i),
            );
            feed.enqueue(item).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Should get oldest first
        let first = feed.next_pending().unwrap().unwrap();
        assert!(first.prompt.contains("0"));
    }

    #[test]
    fn test_prompt_feed_with_media() {
        let dir = tempdir().unwrap();
        let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        let item = PromptItem::new(PromptSource::telegram(1, 100, 123, None), 456, "What is this?")
            .with_media("/data/media/123/photo.jpg");

        let enqueued = feed.enqueue(item).unwrap();
        assert_eq!(
            enqueued.media_path,
            Some("/data/media/123/photo.jpg".to_string())
        );
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_prompt_text() -> impl Strategy<Value = String> {
            prop::collection::vec(
                prop::sample::select(vec![
                    "Hello",
                    "Claude",
                    "help me",
                    "write",
                    "code",
                    "for",
                    "a function",
                    "that",
                    "calculates",
                    "the sum",
                    "of",
                    "\n",
                    " ",
                    ".",
                    "!",
                    "?",
                    ",",
                    "123",
                    "test",
                ]),
                1..50,
            )
            .prop_map(|parts| parts.join(" "))
        }

        fn arb_prompt_item() -> impl Strategy<Value = PromptItem> {
            (
                any::<i64>(), // update_id
                any::<i64>(), // message_id
                any::<i64>(), // user_id
                arb_prompt_text(),
                proptest::option::of(
                    prop::string::string_regex("/data/media/[a-z0-9]+/[a-z]+\\.[a-z]{3}").unwrap(),
                ),
            )
                .prop_map(|(update_id, message_id, user_id, prompt, media_path)| {
                    let mut item = PromptItem::new(
                        PromptSource::telegram(update_id, message_id, 12345, None),
                        user_id,
                        prompt,
                    );
                    if let Some(path) = media_path {
                        item = item.with_media(path);
                    }
                    item
                })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            #[test]
            fn prop_json_roundtrip(item in arb_prompt_item()) {
                // Serialize to JSON and back
                let json = serde_json::to_string(&item).unwrap();
                let deserialized: PromptItem = serde_json::from_str(&json).unwrap();

                assert_eq!(item.id, deserialized.id);
                assert_eq!(item.user_id, deserialized.user_id);
                assert_eq!(item.prompt, deserialized.prompt);
                assert_eq!(item.media_path, deserialized.media_path);
                assert_eq!(item.status, deserialized.status);
            }

            #[test]
            fn prop_enqueue_preserves_data(item in arb_prompt_item()) {
                let dir = tempdir().unwrap();
                let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                let enqueued = feed.enqueue(item.clone()).unwrap();

                // The feed preserves all data
                assert_eq!(item.id, enqueued.id);
                assert_eq!(item.user_id, enqueued.user_id);
                assert_eq!(item.prompt, enqueued.prompt);
                assert_eq!(item.media_path, enqueued.media_path);
            }

            #[test]
            fn prop_enqueue_then_get_preserves_data(item in arb_prompt_item()) {
                let dir = tempdir().unwrap();
                let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                let enqueued = feed.enqueue(item.clone()).unwrap();
                let retrieved = feed.get(&enqueued.id).unwrap().unwrap();

                // Data integrity through storage
                assert_eq!(enqueued.id, retrieved.id);
                assert_eq!(enqueued.user_id, retrieved.user_id);
                assert_eq!(enqueued.prompt, retrieved.prompt);
                assert_eq!(enqueued.media_path, retrieved.media_path);
                assert_eq!(enqueued.status, retrieved.status);
            }

            #[test]
            fn prop_full_lifecycle_preserves_data(item in arb_prompt_item()) {
                let dir = tempdir().unwrap();
                let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                // Enqueue
                let enqueued = feed.enqueue(item.clone()).unwrap();
                assert_eq!(enqueued.status, PromptStatus::Pending);

                // Mark running
                let running = feed.mark_running(&enqueued.id).unwrap();
                assert_eq!(running.status, PromptStatus::Running);
                assert_eq!(running.prompt, enqueued.prompt);
                assert!(running.started_at.is_some());

                // Mark completed
                let completed = feed.mark_completed(&running.id).unwrap();
                assert_eq!(completed.status, PromptStatus::Completed);
                assert_eq!(completed.prompt, enqueued.prompt);
                assert!(completed.completed_at.is_some());

                // Original data preserved through all state transitions
                assert_eq!(completed.user_id, item.user_id);
                assert_eq!(completed.prompt, item.prompt);
            }

            #[test]
            fn prop_ordering_by_created_at(items in prop::collection::vec(arb_prompt_item(), 2..10)) {
                let dir = tempdir().unwrap();
                let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                // Enqueue all items with small delays to ensure different timestamps
                for item in items {
                    feed.enqueue(item).unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }

                // Get them one by one - should be in order
                let mut prev_created_at: Option<DateTime<Utc>> = None;
                while let Some(next) = feed.next_pending().unwrap() {
                    if let Some(prev) = prev_created_at {
                        assert!(next.created_at >= prev, "Items should be returned in creation order");
                    }
                    prev_created_at = Some(next.created_at);
                    feed.mark_running(&next.id).unwrap();
                }
            }
        }
    }
}
