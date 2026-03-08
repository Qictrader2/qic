use crate::error::{Result, TwolebotError};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

/// Stores the "Main" forum topic ID for each DM chat.
///
/// When a bot has Threaded Mode enabled, threadless messages need to be
/// routed to a designated "Main" topic (created lazily via createForumTopic).
/// This store remembers the topic_id per chat_id so we only create it once.
pub struct MainTopicStore {
    conn: Mutex<Connection>,
}

impl MainTopicStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open main_topic db: {}", e)))?;
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
            .map_err(|_| TwolebotError::storage("main_topic db mutex poisoned"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS main_topics (
                chat_id INTEGER PRIMARY KEY,
                topic_id INTEGER NOT NULL
            );",
        )
        .map_err(|e| TwolebotError::storage(format!("init main_topics schema: {}", e)))?;
        Ok(())
    }

    /// Get the Main topic_id for a chat, if one has been created.
    pub fn get(&self, chat_id: i64) -> Result<Option<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("main_topic db mutex poisoned"))?;
        conn.query_row(
            "SELECT topic_id FROM main_topics WHERE chat_id = ?1",
            params![chat_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get main topic: {}", e)))
    }

    /// Store the Main topic_id for a chat (upsert).
    pub fn set(&self, chat_id: i64, topic_id: i64) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("main_topic db mutex poisoned"))?;
        conn.execute(
            "INSERT INTO main_topics (chat_id, topic_id) VALUES (?1, ?2)
             ON CONFLICT(chat_id) DO UPDATE SET topic_id = excluded.topic_id",
            params![chat_id, topic_id],
        )
        .map_err(|e| TwolebotError::storage(format!("set main topic: {}", e)))?;
        Ok(())
    }

    /// Store the topic_id only if no mapping exists yet for this chat.
    /// Returns Ok(true) if inserted, Ok(false) if already present.
    pub fn set_if_absent(&self, chat_id: i64, topic_id: i64) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("main_topic db mutex poisoned"))?;
        let rows = conn
            .execute(
                "INSERT OR IGNORE INTO main_topics (chat_id, topic_id) VALUES (?1, ?2)",
                params![chat_id, topic_id],
            )
            .map_err(|e| TwolebotError::storage(format!("set_if_absent main topic: {}", e)))?;
        Ok(rows > 0)
    }

    /// Get the topic routing key for the Main topic of a chat.
    /// Returns Some("{chat_id}_{topic_id}") if a Main topic exists, None otherwise.
    pub fn get_topic_key(&self, chat_id: i64) -> Result<Option<String>> {
        Ok(self.get(chat_id)?.map(|tid| format!("{}_{}", chat_id, tid)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_set_roundtrip() {
        let dir = tempdir().unwrap();
        let store = MainTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        assert!(store.get(12345).unwrap().is_none());

        store.set(12345, 99).unwrap();
        assert_eq!(store.get(12345).unwrap(), Some(99));

        // Upsert overwrites
        store.set(12345, 200).unwrap();
        assert_eq!(store.get(12345).unwrap(), Some(200));
    }

    #[test]
    fn test_get_topic_key() {
        let dir = tempdir().unwrap();
        let store = MainTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        assert!(store.get_topic_key(12345).unwrap().is_none());

        store.set(12345, 99).unwrap();
        assert_eq!(
            store.get_topic_key(12345).unwrap(),
            Some("12345_99".to_string())
        );
    }

    #[test]
    fn test_multiple_chats() {
        let dir = tempdir().unwrap();
        let store = MainTopicStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.set(111, 10).unwrap();
        store.set(222, 20).unwrap();

        assert_eq!(store.get(111).unwrap(), Some(10));
        assert_eq!(store.get(222).unwrap(), Some(20));
        assert!(store.get(333).unwrap().is_none());
    }
}
