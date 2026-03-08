use crate::error::{Result, TwolebotError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct ChatMetadata {
    pub chat_id: String,
    pub topic_id: Option<i64>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub custom_name: Option<String>,
    pub auto_name: Option<String>,
    pub protocol: Option<String>,
    pub last_message_preview: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl ChatMetadata {
    /// Effective display name: custom_name > auto_name > display_name > "Untitled"
    pub fn effective_name(&self) -> &str {
        self.custom_name
            .as_deref()
            .or(self.auto_name.as_deref())
            .or(self.display_name.as_deref())
            .unwrap_or("Untitled")
    }
}

/// SQLite-backed store for chat display metadata (username, display name per chat+topic).
/// Stored in the unified runtime DB alongside messages.
pub struct ChatMetadataStore {
    conn: Mutex<Connection>,
}

impl ChatMetadataStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open chat_metadata db: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TwolebotError::storage(format!("set WAL mode: {e}")))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TwolebotError::storage(format!("set synchronous: {e}")))?;

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
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;

        // Use a sentinel value (-1) for NULL topic_id in the composite PK
        // since SQLite treats NULLs as distinct in PRIMARY KEY.
        // The actual topic_id is stored in real_topic_id.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chat_metadata (
                chat_id         TEXT NOT NULL,
                topic_id_key    INTEGER NOT NULL DEFAULT -1,
                real_topic_id   INTEGER,
                username        TEXT,
                display_name    TEXT,
                updated_at      TEXT NOT NULL,
                PRIMARY KEY (chat_id, topic_id_key)
            );",
        )
        .map_err(|e| TwolebotError::storage(format!("init chat_metadata schema: {e}")))?;

        // Migration: add web chat naming columns
        let has_custom_name = conn
            .prepare("SELECT custom_name FROM chat_metadata LIMIT 0")
            .is_ok();
        if !has_custom_name {
            conn.execute_batch(
                "ALTER TABLE chat_metadata ADD COLUMN custom_name TEXT;
                 ALTER TABLE chat_metadata ADD COLUMN auto_name TEXT;
                 ALTER TABLE chat_metadata ADD COLUMN protocol TEXT;
                 ALTER TABLE chat_metadata ADD COLUMN last_message_preview TEXT;",
            )
            .map_err(|e| {
                TwolebotError::storage(format!("migrate chat_metadata add naming columns: {e}"))
            })?;
            tracing::info!("Migrated chat_metadata: added custom_name, auto_name, protocol, last_message_preview columns");
        }

        Ok(())
    }

    /// Upsert chat metadata — called on every incoming message to keep display info fresh.
    /// IMPORTANT: custom_name is NEVER overwritten by this method (user-set names are sacred).
    pub fn upsert(
        &self,
        chat_id: &str,
        topic_id: Option<i64>,
        username: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<()> {
        self.upsert_full(chat_id, topic_id, username, display_name, None, None)
    }

    /// Full upsert with protocol and preview. custom_name is never auto-overridden.
    pub fn upsert_full(
        &self,
        chat_id: &str,
        topic_id: Option<i64>,
        username: Option<&str>,
        display_name: Option<&str>,
        protocol: Option<&str>,
        last_message_preview: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;

        let topic_id_key = topic_id.unwrap_or(-1);

        conn.execute(
            "INSERT INTO chat_metadata (chat_id, topic_id_key, real_topic_id, username, display_name, protocol, last_message_preview, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(chat_id, topic_id_key) DO UPDATE SET
                real_topic_id = excluded.real_topic_id,
                username = COALESCE(excluded.username, chat_metadata.username),
                display_name = COALESCE(excluded.display_name, chat_metadata.display_name),
                protocol = COALESCE(excluded.protocol, chat_metadata.protocol),
                last_message_preview = COALESCE(excluded.last_message_preview, chat_metadata.last_message_preview),
                updated_at = excluded.updated_at",
            params![
                chat_id,
                topic_id_key,
                topic_id,
                username,
                display_name,
                protocol,
                last_message_preview,
                Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| TwolebotError::storage(format!("upsert chat_metadata: {e}")))?;

        Ok(())
    }

    /// Set user-chosen custom name for a conversation. This is never auto-overridden.
    pub fn set_custom_name(&self, chat_id: &str, topic_id: Option<i64>, name: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;
        let topic_id_key = topic_id.unwrap_or(-1);
        let affected = conn
            .execute(
                "UPDATE chat_metadata SET custom_name = ?1, updated_at = ?2
                 WHERE chat_id = ?3 AND topic_id_key = ?4",
                params![name, Utc::now().to_rfc3339(), chat_id, topic_id_key],
            )
            .map_err(|e| TwolebotError::storage(format!("set custom_name: {e}")))?;
        if affected == 0 {
            return Err(TwolebotError::not_found(format!(
                "chat_metadata for {chat_id} topic {topic_id_key}"
            )));
        }
        Ok(())
    }

    /// Set AI-generated auto name for a conversation. Only writes if auto_name is currently NULL.
    pub fn set_auto_name(&self, chat_id: &str, topic_id: Option<i64>, name: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;
        let topic_id_key = topic_id.unwrap_or(-1);
        conn.execute(
            "UPDATE chat_metadata SET auto_name = ?1, updated_at = ?2
             WHERE chat_id = ?3 AND topic_id_key = ?4 AND auto_name IS NULL",
            params![name, Utc::now().to_rfc3339(), chat_id, topic_id_key],
        )
        .map_err(|e| TwolebotError::storage(format!("set auto_name: {e}")))?;
        Ok(())
    }

    /// Delete metadata for a specific chat+topic.
    pub fn delete(&self, chat_id: &str, topic_id: Option<i64>) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;
        let topic_id_key = topic_id.unwrap_or(-1);
        let affected = conn
            .execute(
                "DELETE FROM chat_metadata WHERE chat_id = ?1 AND topic_id_key = ?2",
                params![chat_id, topic_id_key],
            )
            .map_err(|e| TwolebotError::storage(format!("delete chat_metadata: {e}")))?;
        Ok(affected > 0)
    }

    /// Get metadata for a specific chat+topic.
    pub fn get(&self, chat_id: &str, topic_id: Option<i64>) -> Result<Option<ChatMetadata>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;

        let topic_id_key = topic_id.unwrap_or(-1);

        let result = conn
            .query_row(
                "SELECT chat_id, real_topic_id, username, display_name,
                        custom_name, auto_name, protocol, last_message_preview, updated_at
                 FROM chat_metadata
                 WHERE chat_id = ?1 AND topic_id_key = ?2",
                params![chat_id, topic_id_key],
                Self::row_to_metadata,
            )
            .optional()
            .map_err(|e| TwolebotError::storage(format!("get chat_metadata: {e}")))?;

        Ok(result)
    }

    /// List all chat metadata, sorted by most recently updated.
    pub fn list_all(&self) -> Result<Vec<ChatMetadata>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("chat_metadata db mutex poisoned"))?;

        let mut stmt = conn
            .prepare(
                "SELECT chat_id, real_topic_id, username, display_name,
                        custom_name, auto_name, protocol, last_message_preview, updated_at
                 FROM chat_metadata
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare list chat_metadata: {e}")))?;

        let rows = stmt
            .query_map([], Self::row_to_metadata)
            .map_err(|e| TwolebotError::storage(format!("query list chat_metadata: {e}")))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(
                row.map_err(|e| TwolebotError::storage(format!("row chat_metadata: {e}")))?,
            );
        }

        Ok(items)
    }

    fn row_to_metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMetadata> {
        let timestamp_text: String = row.get(8)?;
        let updated_at = DateTime::parse_from_rfc3339(&timestamp_text)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        Ok(ChatMetadata {
            chat_id: row.get(0)?,
            topic_id: row.get(1)?,
            username: row.get(2)?,
            display_name: row.get(3)?,
            custom_name: row.get(4)?,
            auto_name: row.get(5)?,
            protocol: row.get(6)?,
            last_message_preview: row.get(7)?,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_upsert_and_get() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store
            .upsert("chat-123", Some(42), Some("alice"), Some("Alice Wonder"))
            .unwrap();

        let meta = store.get("chat-123", Some(42)).unwrap().unwrap();
        assert_eq!(meta.chat_id, "chat-123");
        assert_eq!(meta.topic_id, Some(42));
        assert_eq!(meta.username.as_deref(), Some("alice"));
        assert_eq!(meta.display_name.as_deref(), Some("Alice Wonder"));
    }

    #[test]
    fn test_upsert_preserves_existing_on_none() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store
            .upsert("chat-123", None, Some("bob"), Some("Bob Builder"))
            .unwrap();

        // Second upsert with None username should keep "bob"
        store
            .upsert("chat-123", None, None, Some("Bob Updated"))
            .unwrap();

        let meta = store.get("chat-123", None).unwrap().unwrap();
        assert_eq!(meta.username.as_deref(), Some("bob"));
        assert_eq!(meta.display_name.as_deref(), Some("Bob Updated"));
    }

    #[test]
    fn test_separate_topics() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store
            .upsert("chat-1", Some(10), Some("alice"), Some("Topic A"))
            .unwrap();
        store
            .upsert("chat-1", Some(20), Some("alice"), Some("Topic B"))
            .unwrap();
        store
            .upsert("chat-1", None, Some("alice"), Some("Main"))
            .unwrap();

        let all = store.list_all().unwrap();
        assert_eq!(all.len(), 3);

        assert_eq!(
            store
                .get("chat-1", Some(10))
                .unwrap()
                .unwrap()
                .display_name
                .as_deref(),
            Some("Topic A")
        );
        assert_eq!(
            store
                .get("chat-1", Some(20))
                .unwrap()
                .unwrap()
                .display_name
                .as_deref(),
            Some("Topic B")
        );
        assert_eq!(
            store
                .get("chat-1", None)
                .unwrap()
                .unwrap()
                .display_name
                .as_deref(),
            Some("Main")
        );
    }

    #[test]
    fn test_get_nonexistent() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        assert!(store.get("nope", None).unwrap().is_none());
    }

    #[test]
    fn test_custom_name_not_overridden_by_upsert() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.upsert("chat-1", None, Some("alice"), Some("General")).unwrap();
        store.set_custom_name("chat-1", None, "My Custom Name").unwrap();

        // Upsert should not overwrite custom_name
        store.upsert("chat-1", None, Some("alice"), Some("General Updated")).unwrap();

        let meta = store.get("chat-1", None).unwrap().unwrap();
        assert_eq!(meta.custom_name.as_deref(), Some("My Custom Name"));
        assert_eq!(meta.effective_name(), "My Custom Name");
    }

    #[test]
    fn test_auto_name_only_sets_once() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.upsert("chat-1", None, None, None).unwrap();
        store.set_auto_name("chat-1", None, "First auto name").unwrap();
        store.set_auto_name("chat-1", None, "Second auto name").unwrap();

        let meta = store.get("chat-1", None).unwrap().unwrap();
        assert_eq!(meta.auto_name.as_deref(), Some("First auto name"));
    }

    #[test]
    fn test_effective_name_priority() {
        let dir = tempdir().unwrap();
        let store = ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.upsert("chat-1", None, None, Some("Display")).unwrap();
        assert_eq!(store.get("chat-1", None).unwrap().unwrap().effective_name(), "Display");

        store.set_auto_name("chat-1", None, "Auto").unwrap();
        assert_eq!(store.get("chat-1", None).unwrap().unwrap().effective_name(), "Auto");

        store.set_custom_name("chat-1", None, "Custom").unwrap();
        assert_eq!(store.get("chat-1", None).unwrap().unwrap().effective_name(), "Custom");
    }
}
