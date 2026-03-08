use crate::error::{Result, TwolebotError};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// Supported messaging protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Telegram,
    WhatsApp,
    Slack,
    Web,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Telegram => "telegram",
            Protocol::WhatsApp => "whatsapp",
            Protocol::Slack => "slack",
            Protocol::Web => "web",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "telegram" => Some(Protocol::Telegram),
            "whatsapp" => Some(Protocol::WhatsApp),
            "slack" => Some(Protocol::Slack),
            "web" => Some(Protocol::Web),
            _ => None,
        }
    }
}

type ActiveChatsByUser = HashMap<i64, HashMap<Protocol, String>>;

/// Tracks the most recently active chat_id for each (user_id, protocol) pair.
pub struct ActiveChatRegistry {
    chats: RwLock<ActiveChatsByUser>,
    db_path: std::path::PathBuf,
}

impl ActiveChatRegistry {
    fn read_chats(&self) -> RwLockReadGuard<'_, ActiveChatsByUser> {
        match self.chats.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("ActiveChatRegistry lock poisoned on read; recovering");
                poisoned.into_inner()
            }
        }
    }

    fn write_chats(&self) -> RwLockWriteGuard<'_, ActiveChatsByUser> {
        match self.chats.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("ActiveChatRegistry lock poisoned on write; recovering");
                poisoned.into_inner()
            }
        }
    }

    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let registry = Self {
            chats: RwLock::new(HashMap::new()),
            db_path,
        };

        registry.init_schema()?;
        registry.reload_from_db()?;
        Ok(registry)
    }

    fn conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| TwolebotError::storage(format!("open active chats db: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TwolebotError::storage(format!("set WAL mode: {e}")))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TwolebotError::storage(format!("set synchronous: {e}")))?;
        Ok(conn)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS active_chats (
                user_id    INTEGER NOT NULL,
                protocol   TEXT NOT NULL,
                chat_id    TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (user_id, protocol)
            );
            CREATE INDEX IF NOT EXISTS idx_active_chats_protocol ON active_chats(protocol);
            CREATE INDEX IF NOT EXISTS idx_active_chats_updated_at ON active_chats(updated_at DESC);",
        )
        .map_err(|e| TwolebotError::storage(format!("init active chats schema: {e}")))?;
        Ok(())
    }

    fn reload_from_db(&self) -> Result<()> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT user_id, protocol, chat_id FROM active_chats")
            .map_err(|e| TwolebotError::storage(format!("prepare load active chats: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let user_id: i64 = row.get(0)?;
                let protocol: String = row.get(1)?;
                let chat_id: String = row.get(2)?;
                Ok((user_id, protocol, chat_id))
            })
            .map_err(|e| TwolebotError::storage(format!("query load active chats: {e}")))?;

        let mut in_memory: ActiveChatsByUser = HashMap::new();
        for row in rows {
            let (user_id, protocol_text, chat_id) =
                row.map_err(|e| TwolebotError::storage(format!("row load active chats: {e}")))?;
            if let Some(protocol) = Protocol::from_str(&protocol_text) {
                in_memory.entry(user_id).or_default().insert(protocol, chat_id);
            }
        }

        let mut chats = self.write_chats();
        *chats = in_memory;
        Ok(())
    }

    /// Update the active chat for a user+protocol. Called on every incoming message.
    pub fn set_active(
        &self,
        user_id: i64,
        protocol: Protocol,
        chat_id: impl Into<String>,
    ) -> Result<()> {
        let chat_id = chat_id.into();
        {
            let mut chats = self.write_chats();
            chats
                .entry(user_id)
                .or_default()
                .insert(protocol, chat_id.clone());
        }

        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO active_chats (user_id, protocol, chat_id, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![user_id, protocol.as_str(), chat_id, Utc::now().to_rfc3339()],
        )
        .map_err(|e| TwolebotError::storage(format!("set active chat: {e}")))?;
        Ok(())
    }

    /// Get the active chat_id for a user+protocol, if any.
    pub fn get_active(&self, user_id: i64, protocol: Protocol) -> Option<String> {
        let chats = self.read_chats();
        chats.get(&user_id).and_then(|m| m.get(&protocol).cloned())
    }

    /// Get all active chats for a user across all protocols.
    pub fn get_all_active_for_user(&self, user_id: i64) -> HashMap<Protocol, String> {
        let chats = self.read_chats();
        chats.get(&user_id).cloned().unwrap_or_default()
    }

    /// Get active chat_ids for a user as a list of (protocol, chat_id) pairs.
    pub fn get_broadcast_targets_for_user(&self, user_id: i64) -> Vec<(Protocol, String)> {
        let chats = self.read_chats();
        chats
            .get(&user_id)
            .map(|m| m.iter().map(|(p, c)| (*p, c.clone())).collect())
            .unwrap_or_default()
    }

    /// Get active chat_ids across all users as a list of (user_id, protocol, chat_id) triples.
    pub fn get_broadcast_targets_all_users(&self) -> Vec<(i64, Protocol, String)> {
        let chats = self.read_chats();
        chats
            .iter()
            .flat_map(|(user_id, per_protocol)| {
                per_protocol
                    .iter()
                    .map(|(p, c)| (*user_id, *p, c.clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_set_and_get_active() {
        let dir = TempDir::new().unwrap();
        let registry = ActiveChatRegistry::new(dir.path().join("runtime.sqlite3")).unwrap();

        registry
            .set_active(42, Protocol::Telegram, "123456")
            .unwrap();
        assert_eq!(
            registry.get_active(42, Protocol::Telegram),
            Some("123456".to_string())
        );
        assert_eq!(registry.get_active(42, Protocol::WhatsApp), None);
    }

    #[test]
    fn test_overwrites_previous() {
        let dir = TempDir::new().unwrap();
        let registry = ActiveChatRegistry::new(dir.path().join("runtime.sqlite3")).unwrap();

        registry.set_active(42, Protocol::Telegram, "111").unwrap();
        registry.set_active(42, Protocol::Telegram, "222").unwrap();
        assert_eq!(
            registry.get_active(42, Protocol::Telegram),
            Some("222".to_string())
        );
    }

    #[test]
    fn test_multiple_protocols() {
        let dir = TempDir::new().unwrap();
        let registry = ActiveChatRegistry::new(dir.path().join("runtime.sqlite3")).unwrap();

        registry
            .set_active(42, Protocol::Telegram, "tg_123")
            .unwrap();
        registry
            .set_active(42, Protocol::WhatsApp, "wa_456")
            .unwrap();
        registry.set_active(42, Protocol::Slack, "sl_789").unwrap();

        let all = registry.get_all_active_for_user(42);
        assert_eq!(all.len(), 3);
        assert_eq!(all.get(&Protocol::Telegram), Some(&"tg_123".to_string()));
        assert_eq!(all.get(&Protocol::WhatsApp), Some(&"wa_456".to_string()));
        assert_eq!(all.get(&Protocol::Slack), Some(&"sl_789".to_string()));
    }

    #[test]
    fn test_persistence() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("runtime.sqlite3");

        {
            let registry = ActiveChatRegistry::new(&db_path).unwrap();
            registry
                .set_active(42, Protocol::Telegram, "persistent_id")
                .unwrap();
        }

        {
            let registry = ActiveChatRegistry::new(&db_path).unwrap();
            assert_eq!(
                registry.get_active(42, Protocol::Telegram),
                Some("persistent_id".to_string())
            );
        }
    }

    #[test]
    fn test_get_broadcast_targets() {
        let dir = TempDir::new().unwrap();
        let registry = ActiveChatRegistry::new(dir.path().join("runtime.sqlite3")).unwrap();

        registry.set_active(42, Protocol::Telegram, "tg").unwrap();
        registry.set_active(42, Protocol::Slack, "sl").unwrap();

        let targets = registry.get_broadcast_targets_for_user(42);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&(Protocol::Telegram, "tg".to_string())));
        assert!(targets.contains(&(Protocol::Slack, "sl".to_string())));
    }
}
