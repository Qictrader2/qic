use crate::error::{Result, TwolebotError};
use crate::storage::PromptSource;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Pending,
    Sent,
    Failed,
}

impl ResponseStatus {
    pub fn directory(&self) -> &'static str {
        match self {
            ResponseStatus::Pending => "pending",
            ResponseStatus::Sent => "sent",
            ResponseStatus::Failed => "failed",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseStatus::Pending => "pending",
            ResponseStatus::Sent => "sent",
            ResponseStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(ResponseStatus::Pending),
            "sent" => Some(ResponseStatus::Sent),
            "failed" => Some(ResponseStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseItem {
    pub id: String,
    pub prompt_id: String,
    pub source: PromptSource,
    /// User who triggered this response (0 = system/cron)
    #[serde(default)]
    pub user_id: i64,
    pub content: String,
    pub is_partial: bool,
    pub is_final: bool,
    pub sequence: u32,
    pub status: ResponseStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<DateTime<Utc>>,
    /// If set, do not attempt delivery before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_attempt_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
}

impl ResponseItem {
    pub fn new(
        prompt_id: impl Into<String>,
        source: PromptSource,
        user_id: i64,
        content: impl Into<String>,
        is_final: bool,
        sequence: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            prompt_id: prompt_id.into(),
            source,
            user_id,
            content: content.into(),
            is_partial: !is_final,
            is_final,
            sequence,
            status: ResponseStatus::Pending,
            created_at: Utc::now(),
            sent_at: None,
            next_attempt_at: None,
            error: None,
            retry_count: None,
        }
    }

}

/// Manages the response feed queue in SQLite.
pub struct ResponseFeed {
    conn: Mutex<Connection>,
}

impl ResponseFeed {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open response queue db: {}", e)))?;
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
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS responses (
                id TEXT PRIMARY KEY,
                prompt_id TEXT NOT NULL,
                source_json TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                is_partial INTEGER NOT NULL,
                is_final INTEGER NOT NULL,
                sequence INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                sent_at TEXT,
                next_attempt_at TEXT,
                error TEXT,
                retry_count INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_responses_pending
                ON responses(status, prompt_id, sequence, created_at);
            CREATE INDEX IF NOT EXISTS idx_responses_prompt
                ON responses(prompt_id, status, sequence);
            COMMIT;",
        )
        .map_err(|e| TwolebotError::storage(format!("init response schema: {}", e)))?;
        Ok(())
    }

    fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResponseItem> {
        let source_json: String = row.get("source_json")?;
        let source: PromptSource = serde_json::from_str(&source_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                source_json.len(),
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })?;
        let status_text: String = row.get("status")?;
        let status = ResponseStatus::from_str(&status_text).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                status_text.len(),
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid response status '{}'", status_text),
                )),
            )
        })?;

        let created_at: String = row.get("created_at")?;
        let sent_at: Option<String> = row.get("sent_at")?;
        let next_attempt_at: Option<String> = row.get("next_attempt_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);
        let sent_at = sent_at
            .map(|ts| DateTime::parse_from_rfc3339(&ts).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
        let next_attempt_at = next_attempt_at
            .map(|ts| DateTime::parse_from_rfc3339(&ts).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        Ok(ResponseItem {
            id: row.get("id")?,
            prompt_id: row.get("prompt_id")?,
            source,
            user_id: row.get("user_id")?,
            content: row.get("content")?,
            is_partial: row.get::<_, i64>("is_partial")? != 0,
            is_final: row.get::<_, i64>("is_final")? != 0,
            sequence: row.get::<_, i64>("sequence")? as u32,
            status,
            created_at,
            sent_at,
            next_attempt_at,
            error: row.get("error")?,
            retry_count: row.get::<_, Option<i64>>("retry_count")?.map(|n| n as u32),
        })
    }

    fn upsert_item(conn: &Connection, item: &ResponseItem) -> Result<()> {
        conn.execute(
            "INSERT INTO responses (
                id, prompt_id, source_json, user_id, content, is_partial, is_final,
                sequence, status, created_at, sent_at, next_attempt_at, error, retry_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(id) DO UPDATE SET
                prompt_id=excluded.prompt_id,
                source_json=excluded.source_json,
                user_id=excluded.user_id,
                content=excluded.content,
                is_partial=excluded.is_partial,
                is_final=excluded.is_final,
                sequence=excluded.sequence,
                status=excluded.status,
                created_at=excluded.created_at,
                sent_at=excluded.sent_at,
                next_attempt_at=excluded.next_attempt_at,
                error=excluded.error,
                retry_count=excluded.retry_count",
            params![
                item.id,
                item.prompt_id,
                serde_json::to_string(&item.source)?,
                item.user_id,
                item.content,
                if item.is_partial { 1 } else { 0 },
                if item.is_final { 1 } else { 0 },
                item.sequence as i64,
                item.status.as_str(),
                item.created_at.to_rfc3339(),
                item.sent_at.map(|d| d.to_rfc3339()),
                item.next_attempt_at.map(|d| d.to_rfc3339()),
                item.error,
                item.retry_count.map(|n| n as i64),
            ],
        )
        .map_err(|e| TwolebotError::storage(format!("upsert response item: {}", e)))?;
        Ok(())
    }

    /// Add a new response to the pending queue.
    pub fn enqueue(&self, item: ResponseItem) -> Result<ResponseItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        Self::upsert_item(&conn, &item)?;
        Ok(item)
    }

    /// Get the next pending response (ordered and eligible by next_attempt_at).
    pub fn next_pending(&self) -> Result<Option<ResponseItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let now = Utc::now().to_rfc3339();
        conn.query_row(
            "SELECT * FROM responses
             WHERE status = 'pending'
               AND (next_attempt_at IS NULL OR next_attempt_at <= ?1)
             ORDER BY prompt_id ASC, sequence ASC, created_at ASC, id ASC
             LIMIT 1",
            params![now],
            Self::row_to_item,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("next pending response: {}", e)))
    }

    /// Get all pending responses (eligible only).
    pub fn all_pending(&self) -> Result<Vec<ResponseItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let now = Utc::now().to_rfc3339();
        let mut stmt = conn
            .prepare(
                "SELECT * FROM responses
                 WHERE status = 'pending'
                   AND (next_attempt_at IS NULL OR next_attempt_at <= ?1)
                 ORDER BY prompt_id ASC, sequence ASC, created_at ASC, id ASC",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare all_pending responses: {}", e)))?;
        let rows = stmt
            .query_map(params![now], Self::row_to_item)
            .map_err(|e| TwolebotError::storage(format!("query all_pending responses: {}", e)))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| {
                TwolebotError::storage(format!("row all_pending responses: {}", e))
            })?);
        }
        Ok(items)
    }

    /// Returns true if there is at least one eligible pending system-origin response.
    pub fn has_pending_system_responses(&self) -> Result<bool> {
        Ok(self
            .all_pending()?
            .into_iter()
            .any(|item| item.user_id == 0 && matches!(item.source, PromptSource::Cron { .. })))
    }

    /// Mark a response as sent.
    pub fn mark_sent(&self, id: &str, prompt_id: &str, sequence: u32) -> Result<ResponseItem> {
        let now = Utc::now().to_rfc3339();
        self.transition_pending_response(
            id,
            prompt_id,
            sequence,
            "sent",
            Some(now),
            None,
            None,
            false,
        )
    }

    /// Mark a response as failed.
    pub fn mark_failed(
        &self,
        id: &str,
        prompt_id: &str,
        sequence: u32,
        error: impl Into<String>,
    ) -> Result<ResponseItem> {
        self.transition_pending_response(
            id,
            prompt_id,
            sequence,
            "failed",
            None,
            None,
            Some(error.into()),
            true,
        )
    }

    fn transition_pending_response(
        &self,
        id: &str,
        prompt_id: &str,
        sequence: u32,
        new_status: &str,
        sent_at: Option<String>,
        next_attempt_at: Option<String>,
        error: Option<String>,
        increment_retry: bool,
    ) -> Result<ResponseItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| TwolebotError::storage(format!("begin response transition tx: {}", e)))?;

        let retry_sql = if increment_retry {
            "COALESCE(retry_count, 0) + 1"
        } else {
            "retry_count"
        };
        let sql = format!(
            "UPDATE responses
             SET status = ?1,
                 sent_at = ?2,
                 next_attempt_at = ?3,
                 error = ?4,
                 retry_count = {}
             WHERE id = ?5 AND prompt_id = ?6 AND sequence = ?7 AND status = 'pending'",
            retry_sql
        );

        let affected = tx
            .execute(
                &sql,
                params![
                    new_status,
                    sent_at,
                    next_attempt_at,
                    error,
                    id,
                    prompt_id,
                    sequence as i64
                ],
            )
            .map_err(|e| TwolebotError::storage(format!("transition response: {}", e)))?;

        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "pending response {} for prompt {} seq {} not found",
                id, prompt_id, sequence
            )));
        }

        let item = tx
            .query_row(
                "SELECT * FROM responses WHERE id = ?1",
                params![id],
                Self::row_to_item,
            )
            .map_err(|e| TwolebotError::storage(format!("load transitioned response: {}", e)))?;

        tx.commit()
            .map_err(|e| TwolebotError::storage(format!("commit response transition tx: {}", e)))?;
        Ok(item)
    }

    /// Defer a pending response until a specific time (keeps it pending).
    pub fn defer_until(
        &self,
        id: &str,
        prompt_id: &str,
        sequence: u32,
        next_attempt_at: DateTime<Utc>,
        reason: Option<String>,
    ) -> Result<ResponseItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| TwolebotError::storage(format!("begin defer_until tx: {}", e)))?;

        let affected = tx
            .execute(
                "UPDATE responses
                 SET next_attempt_at = ?1,
                     error = COALESCE(?2, error)
                 WHERE id = ?3 AND prompt_id = ?4 AND sequence = ?5 AND status = 'pending'",
                params![
                    next_attempt_at.to_rfc3339(),
                    reason,
                    id,
                    prompt_id,
                    sequence as i64
                ],
            )
            .map_err(|e| TwolebotError::storage(format!("defer response: {}", e)))?;
        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "pending response {} for prompt {} seq {} not found",
                id, prompt_id, sequence
            )));
        }

        let item = tx
            .query_row(
                "SELECT * FROM responses WHERE id = ?1",
                params![id],
                Self::row_to_item,
            )
            .map_err(|e| TwolebotError::storage(format!("load deferred response: {}", e)))?;

        tx.commit()
            .map_err(|e| TwolebotError::storage(format!("commit defer_until tx: {}", e)))?;
        Ok(item)
    }

    /// Increment retry count and keep pending (for automatic retry).
    pub fn increment_retry(
        &self,
        id: &str,
        prompt_id: &str,
        sequence: u32,
        error: impl Into<String>,
        next_attempt_at: Option<DateTime<Utc>>,
    ) -> Result<ResponseItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| TwolebotError::storage(format!("begin increment_retry tx: {}", e)))?;

        let affected = tx
            .execute(
                "UPDATE responses
                 SET retry_count = COALESCE(retry_count, 0) + 1,
                     error = ?1,
                     next_attempt_at = ?2
                 WHERE id = ?3 AND prompt_id = ?4 AND sequence = ?5 AND status = 'pending'",
                params![
                    error.into(),
                    next_attempt_at.map(|d| d.to_rfc3339()),
                    id,
                    prompt_id,
                    sequence as i64
                ],
            )
            .map_err(|e| TwolebotError::storage(format!("increment retry: {}", e)))?;
        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "pending response {} for prompt {} seq {} not found",
                id, prompt_id, sequence
            )));
        }

        let item = tx
            .query_row(
                "SELECT * FROM responses WHERE id = ?1",
                params![id],
                Self::row_to_item,
            )
            .map_err(|e| TwolebotError::storage(format!("load incremented response: {}", e)))?;

        tx.commit()
            .map_err(|e| TwolebotError::storage(format!("commit increment_retry tx: {}", e)))?;
        Ok(item)
    }

    /// Move a failed response back to pending for retry.
    pub fn retry(&self, id: &str, prompt_id: &str, sequence: u32) -> Result<ResponseItem> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| TwolebotError::storage(format!("begin retry tx: {}", e)))?;

        let affected = tx
            .execute(
                "UPDATE responses
                 SET status = 'pending', next_attempt_at = NULL, error = NULL
                 WHERE id = ?1 AND prompt_id = ?2 AND sequence = ?3 AND status = 'failed'",
                params![id, prompt_id, sequence as i64],
            )
            .map_err(|e| TwolebotError::storage(format!("retry response: {}", e)))?;
        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "failed response {} for prompt {} seq {} not found",
                id, prompt_id, sequence
            )));
        }

        let item = tx
            .query_row(
                "SELECT * FROM responses WHERE id = ?1",
                params![id],
                Self::row_to_item,
            )
            .map_err(|e| TwolebotError::storage(format!("load retried response: {}", e)))?;

        tx.commit()
            .map_err(|e| TwolebotError::storage(format!("commit retry tx: {}", e)))?;
        Ok(item)
    }

    /// Get a response by ID.
    pub fn get(&self, id: &str) -> Result<Option<ResponseItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        conn.query_row(
            "SELECT * FROM responses WHERE id = ?1",
            params![id],
            Self::row_to_item,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get response by id: {}", e)))
    }

    /// List recent sent responses.
    pub fn recent_sent(&self, limit: usize) -> Result<Vec<ResponseItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let mut stmt = conn
            .prepare(
                "SELECT * FROM responses
                 WHERE status = 'sent'
                 ORDER BY sent_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare recent_sent: {}", e)))?;
        let rows = stmt
            .query_map(params![limit as i64], Self::row_to_item)
            .map_err(|e| TwolebotError::storage(format!("query recent_sent: {}", e)))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| TwolebotError::storage(format!("row recent_sent: {}", e)))?);
        }
        Ok(items)
    }

    /// List recent failed responses.
    pub fn recent_failed(&self, limit: usize) -> Result<Vec<ResponseItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let mut stmt = conn
            .prepare(
                "SELECT * FROM responses
                 WHERE status = 'failed'
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare recent_failed: {}", e)))?;
        let rows = stmt
            .query_map(params![limit as i64], Self::row_to_item)
            .map_err(|e| TwolebotError::storage(format!("query recent_failed: {}", e)))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(
                row.map_err(|e| TwolebotError::storage(format!("row recent_failed: {}", e)))?,
            );
        }
        Ok(items)
    }

    /// Cancel all pending responses for a prompt (when interrupted).
    pub fn cancel_for_prompt(&self, prompt_id: &str) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        let count = conn
            .execute(
                "DELETE FROM responses WHERE prompt_id = ?1 AND status = 'pending'",
                params![prompt_id],
            )
            .map_err(|e| TwolebotError::storage(format!("cancel responses for prompt: {}", e)))?;
        Ok(count)
    }

    /// Find the final response for a given prompt ID (searches sent first, then pending).
    pub fn find_final_for_prompt(&self, prompt_id: &str) -> Result<Option<ResponseItem>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("response db mutex poisoned"))?;
        conn.query_row(
            "SELECT * FROM responses
             WHERE prompt_id = ?1 AND is_final = 1 AND status IN ('sent', 'pending')
             ORDER BY CASE status WHEN 'sent' THEN 0 ELSE 1 END, sequence DESC
             LIMIT 1",
            params![prompt_id],
            Self::row_to_item,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("find final for prompt: {}", e)))
    }

    /// Count pending responses.
    pub fn pending_count(&self) -> usize {
        self.count_by_status("pending")
    }

    /// Count sent responses.
    pub fn sent_count(&self) -> usize {
        self.count_by_status("sent")
    }

    /// Count failed responses.
    pub fn failed_count(&self) -> usize {
        self.count_by_status("failed")
    }

    fn count_by_status(&self, status: &str) -> usize {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        conn.query_row(
            "SELECT COUNT(*) FROM responses WHERE status = ?1",
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
    fn test_response_feed_basic() {
        let dir = tempdir().unwrap();
        let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        // Enqueue a response
        let item = ResponseItem::new(
            "prompt-123",
            PromptSource::Telegram {
                update_id: 1,
                message_id: 100,
                chat_id: 123,
                message_thread_id: None,
            },
            456,
            "Hello from Claude!",
            true,
            1,
        );
        let enqueued = feed.enqueue(item).unwrap();
        assert_eq!(enqueued.status, ResponseStatus::Pending);

        // Get next pending
        let next = feed.next_pending().unwrap().unwrap();
        assert_eq!(next.id, enqueued.id);

        // Mark sent
        let sent = feed
            .mark_sent(&next.id, &next.prompt_id, next.sequence)
            .unwrap();
        assert_eq!(sent.status, ResponseStatus::Sent);
        assert!(sent.sent_at.is_some());

        // Verify no more pending
        assert!(feed.next_pending().unwrap().is_none());
    }

    #[test]
    fn test_response_feed_ordering() {
        let dir = tempdir().unwrap();
        let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        // Enqueue multiple responses with different sequences
        for i in (0..3).rev() {
            let item = ResponseItem::new(
                "prompt-123",
                PromptSource::Telegram {
                    update_id: 1,
                    message_id: 100,
                    chat_id: 123,
                    message_thread_id: None,
                },
                456,
                format!("Response part {}", i),
                i == 2,
                i as u32,
            );
            feed.enqueue(item).unwrap();
        }

        // Should get lowest sequence first
        let first = feed.next_pending().unwrap().unwrap();
        assert_eq!(first.sequence, 0);
    }

    #[test]
    fn test_response_feed_retry() {
        let dir = tempdir().unwrap();
        let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        let item = ResponseItem::new(
            "prompt-123",
            PromptSource::Telegram {
                update_id: 1,
                message_id: 100,
                chat_id: 123,
                message_thread_id: None,
            },
            456,
            "Hello!",
            true,
            1,
        );
        let enqueued = feed.enqueue(item).unwrap();

        // Mark failed
        let failed = feed
            .mark_failed(
                &enqueued.id,
                &enqueued.prompt_id,
                enqueued.sequence,
                "Network error",
            )
            .unwrap();
        assert_eq!(failed.status, ResponseStatus::Failed);
        assert_eq!(failed.retry_count, Some(1));

        // Retry
        let retried = feed
            .retry(&failed.id, &failed.prompt_id, failed.sequence)
            .unwrap();
        assert_eq!(retried.status, ResponseStatus::Pending);
        assert!(retried.error.is_none());

        // Should be back in pending
        let next = feed.next_pending().unwrap().unwrap();
        assert_eq!(next.id, enqueued.id);
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_response_content() -> impl Strategy<Value = String> {
            prop::collection::vec(
                prop::sample::select(vec![
                    "Here's", "the", "answer", "to", "your", "question", "The code", "works by",
                    "first", "checking", "if", "\n", " ", ".", "!", "?", ",", "```rust", "```",
                    "let", "fn", "impl", "struct", "pub", "async",
                ]),
                1..100,
            )
            .prop_map(|parts| parts.join(" "))
        }

        fn arb_response_item() -> impl Strategy<Value = ResponseItem> {
            (
                prop::string::string_regex("[a-f0-9-]{36}").unwrap(), // prompt_id
                any::<i64>(),                                         // update_id
                any::<i64>(),                                         // message_id
                any::<i64>(),                                         // user_id
                arb_response_content(),
                any::<bool>(), // is_final
                0u32..1000u32, // sequence
            )
                .prop_map(
                    |(prompt_id, update_id, message_id, user_id, content, is_final, sequence)| {
                        ResponseItem::new(
                            prompt_id,
                            PromptSource::Telegram {
                                update_id,
                                message_id,
                                chat_id: 123,
                                message_thread_id: None,
                            },
                            user_id,
                            content,
                            is_final,
                            sequence,
                        )
                    },
                )
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            #[test]
            fn prop_json_roundtrip(item in arb_response_item()) {
                let json = serde_json::to_string(&item).unwrap();
                let deserialized: ResponseItem = serde_json::from_str(&json).unwrap();

                assert_eq!(item.id, deserialized.id);
                assert_eq!(item.prompt_id, deserialized.prompt_id);
                assert_eq!(item.content, deserialized.content);
                assert_eq!(item.is_final, deserialized.is_final);
                assert_eq!(item.sequence, deserialized.sequence);
                assert_eq!(item.status, deserialized.status);
            }

            #[test]
            fn prop_enqueue_preserves_data(item in arb_response_item()) {
                let dir = tempdir().unwrap();
                let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                let enqueued = feed.enqueue(item.clone()).unwrap();

                assert_eq!(item.id, enqueued.id);
                assert_eq!(item.prompt_id, enqueued.prompt_id);
                assert_eq!(item.content, enqueued.content);
                assert_eq!(item.is_final, enqueued.is_final);
                assert_eq!(item.sequence, enqueued.sequence);
            }

            #[test]
            fn prop_full_lifecycle_preserves_data(item in arb_response_item()) {
                let dir = tempdir().unwrap();
                let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                // Enqueue
                let enqueued = feed.enqueue(item.clone()).unwrap();
                assert_eq!(enqueued.status, ResponseStatus::Pending);

                // Mark sent
                let sent = feed.mark_sent(&enqueued.id, &enqueued.prompt_id, enqueued.sequence).unwrap();
                assert_eq!(sent.status, ResponseStatus::Sent);
                assert!(sent.sent_at.is_some());

                // Original data preserved
                assert_eq!(sent.content, item.content);
                assert_eq!(sent.is_final, item.is_final);
            }

            #[test]
            fn prop_ordering_by_sequence(
                prompt_id in prop::string::string_regex("[a-f0-9-]{36}").unwrap(),
                sequences in prop::collection::vec(0u32..1000u32, 2..10)
            ) {
                let dir = tempdir().unwrap();
                let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                // Enqueue responses with various sequences (out of order)
                let mut shuffled = sequences.clone();
                shuffled.sort();
                shuffled.reverse(); // Enqueue in reverse order

                for seq in &shuffled {
                    let item = ResponseItem::new(
                        &prompt_id,
                        PromptSource::Telegram { update_id: 1, message_id: 100, chat_id: 123, message_thread_id: None },
                        456,
                        format!("Part {}", seq),
                        *seq == *shuffled.last().unwrap(),
                        *seq,
                    );
                    feed.enqueue(item).unwrap();
                }

                // Get them one by one - should be in sequence order (lowest first)
                let mut prev_seq: Option<u32> = None;
                while let Some(next) = feed.next_pending().unwrap() {
                    if let Some(prev) = prev_seq {
                        assert!(next.sequence >= prev, "Items should be returned in sequence order");
                    }
                    prev_seq = Some(next.sequence);
                    feed.mark_sent(&next.id, &next.prompt_id, next.sequence).unwrap();
                }
            }

            #[test]
            fn prop_retry_increments_count(item in arb_response_item()) {
                let dir = tempdir().unwrap();
                let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                let enqueued = feed.enqueue(item).unwrap();

                // Fail it multiple times
                let failed1 = feed.mark_failed(&enqueued.id, &enqueued.prompt_id, enqueued.sequence, "Error 1").unwrap();
                assert_eq!(failed1.retry_count, Some(1));

                let retried1 = feed.retry(&failed1.id, &failed1.prompt_id, failed1.sequence).unwrap();
                let failed2 = feed.mark_failed(&retried1.id, &retried1.prompt_id, retried1.sequence, "Error 2").unwrap();
                assert_eq!(failed2.retry_count, Some(2));

                // Content still preserved after multiple retries
                assert_eq!(failed2.content, enqueued.content);
            }
        }
    }
}
