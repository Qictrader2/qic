use crate::error::{Result, TwolebotError};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

/// Maps cron job_id → Telegram forum topic thread_id per chat.
///
/// Each recurring cron job gets its own dedicated forum topic.
/// On first trigger, the delivery layer creates the topic and stores
/// the mapping here. Subsequent triggers reuse the stored thread_id.
/// If the user deletes a topic, the delivery layer self-heals by
/// creating a new one and updating this mapping.
pub struct CronTopicStore {
    conn: Mutex<Connection>,
}

impl CronTopicStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open cron_topic db: {}", e)))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TwolebotError::storage(format!("set WAL mode: {}", e)))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TwolebotError::storage(format!("set synchronous: {}", e)))?;

        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("cron_topic db mutex poisoned"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cron_topics (
                job_id  TEXT NOT NULL,
                chat_id INTEGER NOT NULL,
                thread_id INTEGER NOT NULL,
                PRIMARY KEY (job_id, chat_id)
            );",
        )
        .map_err(|e| TwolebotError::storage(format!("init cron_topics schema: {}", e)))?;
        Ok(())
    }

    /// Get the thread_id for a cron job in a specific chat.
    pub fn get(&self, job_id: &str, chat_id: i64) -> Result<Option<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("cron_topic db mutex poisoned"))?;
        conn.query_row(
            "SELECT thread_id FROM cron_topics WHERE job_id = ?1 AND chat_id = ?2",
            params![job_id, chat_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get cron topic: {}", e)))
    }

    /// Store the thread_id for a cron job in a chat (upsert).
    pub fn set(&self, job_id: &str, chat_id: i64, thread_id: i64) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("cron_topic db mutex poisoned"))?;
        conn.execute(
            "INSERT INTO cron_topics (job_id, chat_id, thread_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(job_id, chat_id) DO UPDATE SET thread_id = excluded.thread_id",
            params![job_id, chat_id, thread_id],
        )
        .map_err(|e| TwolebotError::storage(format!("set cron topic: {}", e)))?;
        Ok(())
    }

    /// Remove the topic mapping for a job (e.g. when the job is cancelled).
    pub fn remove(&self, job_id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("cron_topic db mutex poisoned"))?;
        conn.execute(
            "DELETE FROM cron_topics WHERE job_id = ?1",
            params![job_id],
        )
        .map_err(|e| TwolebotError::storage(format!("remove cron topic: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_set_roundtrip() {
        let dir = tempdir().unwrap();
        let store = CronTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        assert!(store.get("job-1", 12345).unwrap().is_none());

        store.set("job-1", 12345, 99).unwrap();
        assert_eq!(store.get("job-1", 12345).unwrap(), Some(99));

        // Upsert overwrites
        store.set("job-1", 12345, 200).unwrap();
        assert_eq!(store.get("job-1", 12345).unwrap(), Some(200));
    }

    #[test]
    fn test_different_jobs_same_chat() {
        let dir = tempdir().unwrap();
        let store = CronTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.set("job-1", 12345, 100).unwrap();
        store.set("job-2", 12345, 200).unwrap();

        assert_eq!(store.get("job-1", 12345).unwrap(), Some(100));
        assert_eq!(store.get("job-2", 12345).unwrap(), Some(200));
    }

    #[test]
    fn test_same_job_different_chats() {
        let dir = tempdir().unwrap();
        let store = CronTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.set("job-1", 111, 10).unwrap();
        store.set("job-1", 222, 20).unwrap();

        assert_eq!(store.get("job-1", 111).unwrap(), Some(10));
        assert_eq!(store.get("job-1", 222).unwrap(), Some(20));
    }

    #[test]
    fn test_remove() {
        let dir = tempdir().unwrap();
        let store = CronTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.set("job-1", 12345, 99).unwrap();
        store.set("job-1", 67890, 88).unwrap();
        assert!(store.get("job-1", 12345).unwrap().is_some());

        store.remove("job-1").unwrap();
        assert!(store.get("job-1", 12345).unwrap().is_none());
        assert!(store.get("job-1", 67890).unwrap().is_none());
    }
}
