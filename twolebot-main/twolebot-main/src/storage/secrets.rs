use crate::error::{Result, TwolebotError};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

const TELEGRAM_TOKEN_KEY: &str = "telegram_token";
const TELEGRAM_BOT_NAME_KEY: &str = "telegram_bot_name";
const GEMINI_KEY: &str = "gemini_key";
const AUTH_TOKEN_KEY: &str = "auth_token";

/// Stores runtime secrets in the shared runtime SQLite database.
pub struct SecretsStore {
    conn: Mutex<Connection>,
}

impl SecretsStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open secrets db: {}", e)))?;
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
            .map_err(|_| TwolebotError::storage("secrets db mutex poisoned"))?;
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS runtime_secrets (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );
             COMMIT;",
        )
        .map_err(|e| TwolebotError::storage(format!("init secrets schema: {}", e)))?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("secrets db mutex poisoned"))?;
        conn.query_row(
            "SELECT value FROM runtime_secrets WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get secret '{}': {}", key, e)))
    }

    pub fn set(&self, key: &str, value: impl Into<String>) -> Result<()> {
        let value = value.into();
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("secrets db mutex poisoned"))?;
        conn.execute(
            "INSERT INTO runtime_secrets (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
               value = excluded.value,
               updated_at = excluded.updated_at",
            params![key, value, now],
        )
        .map_err(|e| TwolebotError::storage(format!("set secret '{}': {}", key, e)))?;
        Ok(())
    }

    pub fn get_telegram_token(&self) -> Result<Option<String>> {
        self.get(TELEGRAM_TOKEN_KEY)
    }

    pub fn get_gemini_key(&self) -> Result<Option<String>> {
        self.get(GEMINI_KEY)
    }

    pub fn set_telegram_token(&self, token: impl Into<String>) -> Result<()> {
        self.set(TELEGRAM_TOKEN_KEY, token)
    }

    pub fn get_telegram_bot_name(&self) -> Result<Option<String>> {
        self.get(TELEGRAM_BOT_NAME_KEY)
    }

    pub fn set_telegram_bot_name(&self, name: impl Into<String>) -> Result<()> {
        self.set(TELEGRAM_BOT_NAME_KEY, name)
    }

    pub fn set_gemini_key(&self, key: impl Into<String>) -> Result<()> {
        self.set(GEMINI_KEY, key)
    }

    /// Get the dashboard auth token, if one exists.
    pub fn get_auth_token(&self) -> Result<Option<String>> {
        self.get(AUTH_TOKEN_KEY)
    }

    /// Ensure an auth token exists — generates one if absent.
    /// Returns (token, was_newly_generated).
    pub fn ensure_auth_token(&self) -> Result<(String, bool)> {
        if let Some(token) = self.get_auth_token()? {
            return Ok((token, false));
        }
        let token = uuid::Uuid::new_v4().to_string();
        self.set(AUTH_TOKEN_KEY, &token)?;
        Ok((token, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn test_set_and_get_tokens() {
        let dir = tempdir().unwrap();
        let store = SecretsStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        store.set_telegram_token("abc123").unwrap();
        store.set_gemini_key("gem-key").unwrap();

        assert_eq!(
            store.get_telegram_token().unwrap().as_deref(),
            Some("abc123")
        );
        assert_eq!(store.get_gemini_key().unwrap().as_deref(), Some("gem-key"));
    }

    fn arb_secret() -> impl Strategy<Value = String> {
        prop::string::string_regex("[A-Za-z0-9_\\-\\.]{1,128}").unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn prop_secret_roundtrip_preserved(telegram in arb_secret(), gemini in arb_secret()) {
            let dir = tempdir().unwrap();
            let store = SecretsStore::new(dir.path().join("runtime.sqlite3")).unwrap();

            store.set_telegram_token(telegram.clone()).unwrap();
            store.set_gemini_key(gemini.clone()).unwrap();

            prop_assert_eq!(store.get_telegram_token().unwrap(), Some(telegram));
            prop_assert_eq!(store.get_gemini_key().unwrap(), Some(gemini));
        }
    }
}
