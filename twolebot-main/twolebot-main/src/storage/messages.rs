use crate::error::{Result, TwolebotError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

impl MessageDirection {
    fn as_str(&self) -> &'static str {
        match self {
            MessageDirection::Inbound => "inbound",
            MessageDirection::Outbound => "outbound",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "inbound" => Some(MessageDirection::Inbound),
            "outbound" => Some(MessageDirection::Outbound),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub chat_id: String,
    pub user_id: Option<i64>,
    pub direction: MessageDirection,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_message_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i64>,
}

impl StoredMessage {
    pub fn inbound(
        id: impl Into<String>,
        chat_id: impl Into<String>,
        user_id: i64,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            chat_id: chat_id.into(),
            user_id: Some(user_id),
            direction: MessageDirection::Inbound,
            content: content.into(),
            media_type: None,
            media_path: None,
            reply_to: None,
            timestamp: Utc::now(),
            telegram_message_id: None,
            topic_id: None,
        }
    }

    pub fn outbound(
        id: impl Into<String>,
        chat_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            chat_id: chat_id.into(),
            user_id: None,
            direction: MessageDirection::Outbound,
            content: content.into(),
            media_type: None,
            media_path: None,
            reply_to: None,
            timestamp: Utc::now(),
            telegram_message_id: None,
            topic_id: None,
        }
    }

    pub fn outbound_with_user(
        id: impl Into<String>,
        chat_id: impl Into<String>,
        user_id: i64,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            chat_id: chat_id.into(),
            user_id: if user_id == 0 { None } else { Some(user_id) },
            direction: MessageDirection::Outbound,
            content: content.into(),
            media_type: None,
            media_path: None,
            reply_to: None,
            timestamp: Utc::now(),
            telegram_message_id: None,
            topic_id: None,
        }
    }

    pub fn with_media(
        mut self,
        media_type: impl Into<String>,
        media_path: impl Into<String>,
    ) -> Self {
        self.media_type = Some(media_type.into());
        self.media_path = Some(media_path.into());
        self
    }

    pub fn with_telegram_id(mut self, telegram_message_id: i64) -> Self {
        self.telegram_message_id = Some(telegram_message_id);
        self
    }

    pub fn with_topic_id(mut self, topic_id: Option<i64>) -> Self {
        self.topic_id = topic_id;
        self
    }
}

/// SQLite-backed message store in the unified runtime DB.
pub struct MessageStore {
    conn: Mutex<Connection>,
}

impl MessageStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open messages db: {e}")))?;
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
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS messages (
                id                  TEXT NOT NULL,
                chat_id             TEXT NOT NULL,
                user_id             INTEGER,
                direction           TEXT NOT NULL,
                content             TEXT NOT NULL,
                media_type          TEXT,
                media_path          TEXT,
                reply_to            TEXT,
                timestamp           TEXT NOT NULL,
                telegram_message_id INTEGER,
                topic_id            INTEGER,
                PRIMARY KEY (chat_id, id)
            );
            CREATE INDEX IF NOT EXISTS idx_messages_chat_timestamp
                ON messages(chat_id, timestamp DESC, id DESC);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp
                ON messages(timestamp DESC);
            COMMIT;",
        )
        .map_err(|e| TwolebotError::storage(format!("init messages schema: {e}")))?;

        // Migration: add topic_id column to existing tables that lack it
        Self::migrate_add_column(&conn, "messages", "topic_id", "INTEGER")?;

        Ok(())
    }

    fn migrate_add_column(
        conn: &Connection,
        table: &str,
        column: &str,
        col_type: &str,
    ) -> Result<()> {
        let has_column: bool = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .and_then(|mut stmt| {
                let names: Vec<String> = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .filter_map(|r| r.ok())
                    .collect();
                Ok(names.iter().any(|n| n == column))
            })
            .map_err(|e| TwolebotError::storage(format!("check column {column}: {e}")))?;

        if !has_column {
            conn.execute_batch(&format!(
                "ALTER TABLE {table} ADD COLUMN {column} {col_type};"
            ))
            .map_err(|e| TwolebotError::storage(format!("migrate add {column}: {e}")))?;
            tracing::info!("Migrated: added {column} column to {table}");
        }

        Ok(())
    }

    fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredMessage> {
        let direction_text: String = row.get("direction")?;
        let direction = MessageDirection::from_str(&direction_text).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                direction_text.len(),
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid message direction '{direction_text}'"),
                )),
            )
        })?;

        let timestamp_text: String = row.get("timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_text)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        Ok(StoredMessage {
            id: row.get("id")?,
            chat_id: row.get("chat_id")?,
            user_id: row.get("user_id")?,
            direction,
            content: row.get("content")?,
            media_type: row.get("media_type")?,
            media_path: row.get("media_path")?,
            reply_to: row.get("reply_to")?,
            timestamp,
            telegram_message_id: row.get("telegram_message_id")?,
            topic_id: row.get("topic_id")?,
        })
    }

    /// Delete all messages for a chat_id (used when deleting a web conversation).
    pub fn delete_by_chat(&self, chat_id: &str) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;
        let count = conn
            .execute(
                "DELETE FROM messages WHERE chat_id = ?1",
                params![chat_id],
            )
            .map_err(|e| TwolebotError::storage(format!("delete messages by chat: {e}")))?;
        Ok(count)
    }

    /// Store a message.
    pub fn store(&self, message: StoredMessage) -> Result<StoredMessage> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        conn.execute(
            "INSERT OR REPLACE INTO messages
                (id, chat_id, user_id, direction, content, media_type, media_path, reply_to, timestamp, telegram_message_id, topic_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                message.id,
                message.chat_id,
                message.user_id,
                message.direction.as_str(),
                message.content,
                message.media_type,
                message.media_path,
                message.reply_to,
                message.timestamp.to_rfc3339(),
                message.telegram_message_id,
                message.topic_id,
            ],
        )
        .map_err(|e| TwolebotError::storage(format!("store message: {e}")))?;

        Ok(message)
    }

    /// Update the content of an existing message (used for background transcription).
    pub fn update_content(&self, message_id: &str, new_content: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        let rows = conn
            .execute(
                "UPDATE messages SET content = ?1 WHERE id = ?2",
                params![new_content, message_id],
            )
            .map_err(|e| TwolebotError::storage(format!("update message content: {e}")))?;

        Ok(rows > 0)
    }

    /// Get a message by ID.
    pub fn get(&self, chat_id: &str, message_id: &str) -> Result<Option<StoredMessage>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        conn.query_row(
            "SELECT * FROM messages WHERE chat_id = ?1 AND id = ?2",
            params![chat_id, message_id],
            Self::row_to_message,
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get message: {e}")))
    }

    /// List recent messages for a chat (newest first).
    pub fn list(&self, chat_id: &str, limit: usize) -> Result<Vec<StoredMessage>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        let mut stmt = conn
            .prepare(
                "SELECT * FROM messages
                 WHERE chat_id = ?1
                 ORDER BY timestamp DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare list messages: {e}")))?;

        let rows = stmt
            .query_map(params![chat_id, limit as i64], Self::row_to_message)
            .map_err(|e| TwolebotError::storage(format!("query list messages: {e}")))?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(
                row.map_err(|e| TwolebotError::storage(format!("row list messages: {e}")))?,
            );
        }

        Ok(messages)
    }

    /// List all chat+topic combinations with message counts.
    pub fn list_chats(&self) -> Result<Vec<(String, Option<i64>, usize)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        let mut stmt = conn
            .prepare(
                "SELECT chat_id, topic_id, COUNT(*) AS message_count
                 FROM messages
                 GROUP BY chat_id, topic_id
                 ORDER BY MAX(timestamp) DESC",
            )
            .map_err(|e| TwolebotError::storage(format!("prepare list chats: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let chat_id: String = row.get(0)?;
                let topic_id: Option<i64> = row.get(1)?;
                let count: i64 = row.get(2)?;
                Ok((chat_id, topic_id, count as usize))
            })
            .map_err(|e| TwolebotError::storage(format!("query list chats: {e}")))?;

        let mut chats = Vec::new();
        for row in rows {
            chats.push(row.map_err(|e| TwolebotError::storage(format!("row list chats: {e}")))?);
        }

        Ok(chats)
    }

    /// List recent messages for a chat + topic (newest first).
    pub fn list_by_topic(
        &self,
        chat_id: &str,
        topic_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<StoredMessage>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;

        let (sql, params_vec): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match topic_id {
            Some(tid) => (
                "SELECT * FROM messages WHERE chat_id = ?1 AND topic_id = ?2 ORDER BY timestamp DESC, id DESC LIMIT ?3",
                vec![
                    Box::new(chat_id.to_string()),
                    Box::new(tid),
                    Box::new(limit as i64),
                ],
            ),
            None => (
                "SELECT * FROM messages WHERE chat_id = ?1 AND topic_id IS NULL ORDER BY timestamp DESC, id DESC LIMIT ?2",
                vec![
                    Box::new(chat_id.to_string()),
                    Box::new(limit as i64),
                ],
            ),
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| TwolebotError::storage(format!("prepare list_by_topic: {e}")))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(params_refs.as_slice(), Self::row_to_message)
            .map_err(|e| TwolebotError::storage(format!("query list_by_topic: {e}")))?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(
                row.map_err(|e| TwolebotError::storage(format!("row list_by_topic: {e}")))?,
            );
        }

        Ok(messages)
    }

    /// Get conversation history for a chat (oldest first, for context).
    pub fn history(&self, chat_id: &str, limit: usize) -> Result<Vec<StoredMessage>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut newest_first = self.list(chat_id, limit)?;
        newest_first.reverse();
        Ok(newest_first)
    }

    /// Get conversation history for a chat + topic (oldest first, for context).
    pub fn history_by_topic(
        &self,
        chat_id: &str,
        topic_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<StoredMessage>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut newest_first = self.list_by_topic(chat_id, topic_id, limit)?;
        newest_first.reverse();
        Ok(newest_first)
    }

    /// Check if any inbound messages exist (i.e. a user has contacted the bot).
    pub fn has_inbound_messages(&self) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("messages db mutex poisoned"))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE direction = 'inbound' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::storage(format!("query inbound messages: {e}")))?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use proptest::prelude::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_message_store_basic() {
        let dir = tempdir().unwrap();
        let store = MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        let msg = StoredMessage::inbound("msg-1", "chat-123", 456, "Hello!");
        store.store(msg.clone()).unwrap();

        let retrieved = store.get("chat-123", "msg-1").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.content, "Hello!");
        assert_eq!(retrieved.direction, MessageDirection::Inbound);
        assert_eq!(retrieved.user_id, Some(456));
    }

    #[test]
    fn test_message_store_ordering() {
        let dir = tempdir().unwrap();
        let store = MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        let base = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        for i in 0..5 {
            let mut msg = StoredMessage::inbound(
                format!("msg-{}", i),
                "chat-123",
                456,
                format!("Message {}", i),
            );
            msg.timestamp = base + Duration::seconds(i as i64);
            store.store(msg).unwrap();
        }

        let messages = store.list("chat-123", 3).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, "Message 4");
        assert_eq!(messages[1].content, "Message 3");
        assert_eq!(messages[2].content, "Message 2");

        let history = store.history("chat-123", 3).unwrap();
        assert_eq!(history[0].content, "Message 2");
        assert_eq!(history[1].content, "Message 3");
        assert_eq!(history[2].content, "Message 4");
    }

    #[test]
    fn test_message_store_with_media() {
        let dir = tempdir().unwrap();
        let store = MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        let msg = StoredMessage::inbound("msg-1", "chat-123", 456, "Check this out!")
            .with_media("photo", "/media/chat-123/img.jpg");

        store.store(msg).unwrap();

        let retrieved = store.get("chat-123", "msg-1").unwrap().unwrap();
        assert_eq!(retrieved.media_type, Some("photo".to_string()));
        assert_eq!(retrieved.media_path, Some("/media/chat-123/img.jpg".to_string()));
    }

    #[test]
    fn test_message_store_list_chats() {
        let dir = tempdir().unwrap();
        let store = MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store
            .store(StoredMessage::inbound("a-1", "chat-a", 1, "hello"))
            .unwrap();
        store
            .store(StoredMessage::inbound("a-2", "chat-a", 1, "world"))
            .unwrap();
        store
            .store(StoredMessage::inbound("b-1", "chat-b", 2, "other"))
            .unwrap();

        let chats = store.list_chats().unwrap();
        assert_eq!(chats.len(), 2);

        let mut counts = HashMap::new();
        for (chat_id, _topic_id, count) in chats {
            counts.insert(chat_id, count);
        }

        assert_eq!(counts.get("chat-a"), Some(&2usize));
        assert_eq!(counts.get("chat-b"), Some(&1usize));
    }

    #[test]
    fn test_message_store_list_chats_with_topics() {
        let dir = tempdir().unwrap();
        let store = MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        // Same chat, different topics
        store
            .store(StoredMessage::inbound("a-1", "chat-a", 1, "hello").with_topic_id(Some(100)))
            .unwrap();
        store
            .store(StoredMessage::inbound("a-2", "chat-a", 1, "world").with_topic_id(Some(100)))
            .unwrap();
        store
            .store(StoredMessage::inbound("a-3", "chat-a", 1, "topic2").with_topic_id(Some(200)))
            .unwrap();
        // No topic
        store
            .store(StoredMessage::inbound("a-4", "chat-a", 1, "no topic"))
            .unwrap();

        let chats = store.list_chats().unwrap();
        // 3 groups: (chat-a, Some(100)), (chat-a, Some(200)), (chat-a, None)
        assert_eq!(chats.len(), 3);

        let mut counts: HashMap<(String, Option<i64>), usize> = HashMap::new();
        for (chat_id, topic_id, count) in chats {
            counts.insert((chat_id, topic_id), count);
        }

        assert_eq!(counts.get(&("chat-a".to_string(), Some(100))), Some(&2));
        assert_eq!(counts.get(&("chat-a".to_string(), Some(200))), Some(&1));
        assert_eq!(counts.get(&("chat-a".to_string(), None)), Some(&1));
    }

    fn arb_messages() -> impl Strategy<Value = Vec<(u8, Option<u8>, String)>> {
        prop::collection::vec(
            (0u8..4, prop::option::of(0u8..3), ".{1,80}"),
            1..80,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(40))]

        #[test]
        fn prop_chat_counts_match_inserted_data(items in arb_messages()) {
            let dir = tempdir().unwrap();
            let store = MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap();

            let mut expected: HashMap<(String, Option<i64>), usize> = HashMap::new();
            let base = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

            for (idx, (chat_idx, topic_idx, text)) in items.iter().enumerate() {
                let chat_id = format!("chat-{}", chat_idx);
                let topic_id = topic_idx.map(|t| t as i64);
                let mut msg = StoredMessage::inbound(
                    format!("msg-{idx}"),
                    chat_id.clone(),
                    idx as i64,
                    text,
                ).with_topic_id(topic_id);
                msg.timestamp = base + Duration::seconds(idx as i64);
                store.store(msg).unwrap();

                let entry = expected.entry((chat_id, topic_id)).or_insert(0);
                *entry += 1;
            }

            let chats = store.list_chats().unwrap();
            let mut actual: HashMap<(String, Option<i64>), usize> = HashMap::new();
            for (chat_id, topic_id, count) in chats {
                actual.insert((chat_id, topic_id), count);
            }

            prop_assert_eq!(&actual, &expected);
        }
    }
}
