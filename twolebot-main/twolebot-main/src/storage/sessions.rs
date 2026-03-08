use crate::error::{Result, TwolebotError};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

const SESSION_LIFETIME_DAYS: i64 = 90;

/// Stores dashboard login sessions in the shared runtime SQLite database.
pub struct SessionStore {
    conn: Mutex<Connection>,
}

impl SessionStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open sessions db: {}", e)))?;
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
            .map_err(|_| TwolebotError::storage("sessions db mutex poisoned"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );",
        )
        .map_err(|e| TwolebotError::storage(format!("init sessions schema: {}", e)))?;
        Ok(())
    }

    /// Create a new session, returning its ID.
    pub fn create(&self) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires = now + Duration::days(SESSION_LIFETIME_DAYS);
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("sessions db mutex poisoned"))?;
        conn.execute(
            "INSERT INTO sessions (session_id, created_at, expires_at) VALUES (?1, ?2, ?3)",
            params![session_id, now.to_rfc3339(), expires.to_rfc3339()],
        )
        .map_err(|e| TwolebotError::storage(format!("create session: {}", e)))?;
        Ok(session_id)
    }

    /// Validate a session ID. Returns true if the session exists and hasn't expired.
    pub fn validate(&self, session_id: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("sessions db mutex poisoned"))?;
        let expires_at: Option<String> = conn
            .query_row(
                "SELECT expires_at FROM sessions WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| TwolebotError::storage(format!("validate session: {}", e)))?;

        match expires_at {
            Some(exp) => {
                let expires = chrono::DateTime::parse_from_rfc3339(&exp)
                    .map_err(|e| TwolebotError::storage(format!("parse expires_at: {}", e)))?;
                Ok(Utc::now() < expires)
            }
            None => Ok(false),
        }
    }

    /// Delete expired sessions (garbage collection).
    pub fn cleanup_expired(&self) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| TwolebotError::storage("sessions db mutex poisoned"))?;
        let now = Utc::now().to_rfc3339();
        let deleted = conn
            .execute(
                "DELETE FROM sessions WHERE expires_at < ?1",
                params![now],
            )
            .map_err(|e| TwolebotError::storage(format!("cleanup sessions: {}", e)))?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_validate() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        let session_id = store.create().unwrap();
        assert!(store.validate(&session_id).unwrap());
    }

    #[test]
    fn test_invalid_session() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        assert!(!store.validate("nonexistent").unwrap());
    }

    #[test]
    fn test_cleanup_expired() {
        let dir = tempdir().unwrap();
        let store = SessionStore::new(dir.path().join("runtime.sqlite3")).unwrap();

        // Create a session
        let _session_id = store.create().unwrap();

        // Cleanup should not remove a fresh session
        let deleted = store.cleanup_expired().unwrap();
        assert_eq!(deleted, 0);
    }
}
