use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use tracing::{info, warn};

use crate::TwolebotError;

pub type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// SQLite connection pool for the work module.
#[derive(Clone)]
pub struct WorkDb {
    pool: Pool<SqliteConnectionManager>,
}

/// Configures each SQLite connection with WAL mode, foreign keys, and busy_timeout.
#[derive(Debug)]
struct WorkConnectionCustomizer;

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for WorkConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(())
    }
}

impl WorkDb {
    /// Open (or create) the work SQLite database in the unified runtime DB
    /// and run migrations.
    pub fn open(data_dir: &Path) -> Result<Self, TwolebotError> {
        let db_path = data_dir.join("runtime.sqlite3");
        info!(path = %db_path.display(), "opening work database");

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder()
            .max_size(4)
            .connection_customizer(Box::new(WorkConnectionCustomizer))
            .build(manager)
            .map_err(|e| TwolebotError::storage(format!("failed to create work pool: {e}")))?;

        let db = WorkDb { pool };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database (for tests).
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, TwolebotError> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1)
            .connection_customizer(Box::new(WorkConnectionCustomizer))
            .build(manager)
            .map_err(|e| TwolebotError::storage(format!("failed to create in-memory pool: {e}")))?;

        let db = WorkDb { pool };
        db.migrate()?;
        Ok(db)
    }

    /// Get a connection from the pool.
    pub fn conn(&self) -> Result<PooledConnection, TwolebotError> {
        self.pool
            .get()
            .map_err(|e| TwolebotError::storage(format!("work pool error: {e}")))
    }

    /// Run all pending migrations.
    fn migrate(&self) -> Result<(), TwolebotError> {
        let conn = self.conn()?;

        // Ensure schema_version table exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version    INTEGER NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );",
        )
        .map_err(|e| TwolebotError::storage(format!("failed to create schema_version: {e}")))?;

        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::storage(format!("failed to read schema version: {e}")))?;

        info!(current_version, "work schema version");

        let migrations: Vec<(i64, &str, fn(&Connection) -> Result<(), TwolebotError>)> = vec![
            (1, "initial schema", migrate_v1),
            (2, "drop board tables", migrate_v2),
            (3, "drop worker_type columns", migrate_v3),
            (4, "pm semantic search tables", migrate_v4),
        ];

        for (version, name, migrate_fn) in &migrations {
            if current_version < *version {
                info!(version, name, "applying work migration");
                migrate_fn(&conn)?;
                conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    [version],
                )
                .map_err(|e| {
                    TwolebotError::storage(format!("failed to record migration v{version}: {e}"))
                })?;
                info!(version, name, "work migration applied");
            }
        }

        Ok(())
    }
}

// ── V1 Migration: Initial schema ────────────────────────────────────────────

fn migrate_v1(conn: &Connection) -> Result<(), TwolebotError> {
    conn.execute_batch(
        "
        -- Core entities
        CREATE TABLE IF NOT EXISTS projects (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT    NOT NULL,
            description     TEXT    NOT NULL DEFAULT '',
            git_remote_url  TEXT,
            tags_json       TEXT    NOT NULL DEFAULT '[]',
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id       INTEGER NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
            task_type        TEXT    NOT NULL CHECK (task_type IN ('epic','story','task','bug','component')),
            status           TEXT    NOT NULL DEFAULT 'todo'
                                     CHECK (status IN ('todo','in_progress','ready_for_review',
                                                       'under_review','done','blocked','abandoned','archived')),
            priority         TEXT    NOT NULL DEFAULT 'medium'
                                     CHECK (priority IN ('low','medium','high','critical')),
            sort_order       INTEGER NOT NULL DEFAULT 0,
            title            TEXT    NOT NULL,
            description      TEXT    NOT NULL DEFAULT '',
            tags_json        TEXT    NOT NULL DEFAULT '[]',
            worker_name      TEXT,
            completed_at     TEXT,
            created_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks(project_id, status);
        CREATE INDEX IF NOT EXISTS idx_tasks_status_order   ON tasks(status, sort_order);
        CREATE INDEX IF NOT EXISTS idx_tasks_project_order  ON tasks(project_id, sort_order);
        CREATE TABLE IF NOT EXISTS task_dependencies (
            task_id            INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            depends_on_task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            PRIMARY KEY (task_id, depends_on_task_id),
            CHECK (task_id != depends_on_task_id)
        );

        CREATE INDEX IF NOT EXISTS idx_task_deps_reverse ON task_dependencies(depends_on_task_id);

        CREATE TABLE IF NOT EXISTS task_tags (
            task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            tag     TEXT    NOT NULL,
            PRIMARY KEY (task_id, tag)
        );

        CREATE TABLE IF NOT EXISTS documents (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id          INTEGER NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
            document_type       TEXT    NOT NULL CHECK (document_type IN ('plan','specification','notes','code','other')),
            title               TEXT    NOT NULL,
            content             TEXT    NOT NULL DEFAULT '',
            version             INTEGER NOT NULL DEFAULT 1,
            deleted             INTEGER NOT NULL DEFAULT 0,
            worker_name         TEXT,
            created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_documents_project ON documents(project_id);

        CREATE TABLE IF NOT EXISTS comments (
            id                 INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id            INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
            document_id        INTEGER REFERENCES documents(id) ON DELETE CASCADE,
            parent_comment_id  INTEGER REFERENCES comments(id) ON DELETE CASCADE,
            content            TEXT    NOT NULL,
            worker_name        TEXT,
            created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            CHECK (
                (task_id IS NOT NULL AND document_id IS NULL) OR
                (task_id IS NULL AND document_id IS NOT NULL)
            )
        );

        CREATE INDEX IF NOT EXISTS idx_comments_task     ON comments(task_id);
        CREATE INDEX IF NOT EXISTS idx_comments_document ON comments(document_id);

        CREATE TABLE IF NOT EXISTS activity_logs (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id  INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            task_id     INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
            document_id INTEGER REFERENCES documents(id) ON DELETE SET NULL,
            action      TEXT    NOT NULL CHECK (action IN (
                            'created','updated','deleted','status_changed',
                            'assigned','commented','priority_changed',
                            'selected','deselected','review_rejected',
                            'agent_loop_started','agent_loop_stopped','agent_task_failed'
                        )),
            actor       TEXT    NOT NULL DEFAULT 'system',
            worker_name TEXT,
            details     TEXT    NOT NULL DEFAULT '',
            created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_activity_project ON activity_logs(project_id);
        CREATE INDEX IF NOT EXISTS idx_activity_time    ON activity_logs(created_at DESC);

        -- Full-text search for documents
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            title, content,
            content='documents',
            content_rowid='id'
        );

        CREATE TRIGGER IF NOT EXISTS documents_fts_insert AFTER INSERT ON documents BEGIN
            INSERT INTO documents_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS documents_fts_update AFTER UPDATE OF title, content ON documents BEGIN
            INSERT INTO documents_fts(documents_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
            INSERT INTO documents_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS documents_fts_delete AFTER DELETE ON documents BEGIN
            INSERT INTO documents_fts(documents_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
        END;

        -- Trello-style boards
        CREATE TABLE IF NOT EXISTS boards (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL,
            board_type  TEXT    NOT NULL DEFAULT 'kanban'
                                CHECK (board_type IN ('kanban','scrum','custom')),
            created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS board_columns (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id          INTEGER NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
            name              TEXT    NOT NULL,
            sort_order        INTEGER NOT NULL DEFAULT 0,
            column_type       TEXT    NOT NULL DEFAULT 'status'
                                      CHECK (column_type IN ('status','backlog','selected','custom')),
            wip_limit         INTEGER,
            filter_status     TEXT    CHECK (filter_status IS NULL OR filter_status IN
                                      ('todo','in_progress','ready_for_review',
                                       'under_review','done','blocked','abandoned')),
            filter_project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_board_columns_board ON board_columns(board_id, sort_order);

        CREATE TABLE IF NOT EXISTS board_cards (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            column_id   INTEGER NOT NULL REFERENCES board_columns(id) ON DELETE CASCADE,
            task_id     INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            UNIQUE(column_id, task_id)
        );

        CREATE INDEX IF NOT EXISTS idx_board_cards_column ON board_cards(column_id, sort_order);
        CREATE INDEX IF NOT EXISTS idx_board_cards_task   ON board_cards(task_id);

        -- Agent live-board
        CREATE TABLE IF NOT EXISTS live_board_selections (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id       INTEGER NOT NULL UNIQUE REFERENCES tasks(id) ON DELETE CASCADE,
            sort_order    INTEGER NOT NULL DEFAULT 0,
            selected_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            started_at    TEXT,
            completed_at  TEXT,
            status        TEXT    NOT NULL DEFAULT 'queued'
                                  CHECK (status IN ('queued','active','paused','done','failed'))
        );

        CREATE INDEX IF NOT EXISTS idx_live_board_status_order ON live_board_selections(status, sort_order);
        ",
    )
    .map_err(|e| TwolebotError::storage(format!("v1 migration failed: {e}")))?;

    warn!("work v1 migration: note that FTS5 triggers require SQLite compiled with FTS5 support");
    Ok(())
}

// ── V2 Migration: Drop board tables ─────────────────────────────────────────

fn migrate_v2(conn: &Connection) -> Result<(), TwolebotError> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS board_cards;
        DROP TABLE IF EXISTS board_columns;
        DROP TABLE IF EXISTS boards;
        ",
    )
    .map_err(|e| TwolebotError::storage(format!("v2 migration failed: {e}")))?;

    info!("dropped board_cards, board_columns, boards tables");
    Ok(())
}

// ── V3 Migration: Drop worker_type columns ──────────────────────────────────

fn migrate_v3(conn: &Connection) -> Result<(), TwolebotError> {
    let tables = ["tasks", "documents", "comments", "activity_logs"];
    for table in &tables {
        let has_column: bool = conn
            .prepare(&format!("SELECT worker_type FROM {table} LIMIT 0"))
            .is_ok();
        if has_column {
            conn.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN worker_type;"))
                .map_err(|e| {
                    TwolebotError::storage(format!(
                        "v3 migration: drop worker_type from {table}: {e}"
                    ))
                })?;
            info!(table, "dropped worker_type column");
        }
    }
    Ok(())
}

// ── V4 Migration: PM semantic search tables ─────────────────────────────────

fn migrate_v4(conn: &Connection) -> Result<(), TwolebotError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS pm_chunks (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type   TEXT    NOT NULL CHECK (entity_type IN ('task', 'document', 'comment')),
            entity_id     INTEGER NOT NULL,
            project_id    INTEGER NOT NULL,
            chunk_index   INTEGER NOT NULL DEFAULT 0,
            chunk_text    TEXT    NOT NULL,
            content_hash  TEXT    NOT NULL,
            updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            UNIQUE(entity_type, entity_id, chunk_index)
        );

        CREATE INDEX IF NOT EXISTS idx_pm_chunks_entity
            ON pm_chunks(entity_type, entity_id);
        CREATE INDEX IF NOT EXISTS idx_pm_chunks_project
            ON pm_chunks(project_id);
        CREATE INDEX IF NOT EXISTS idx_pm_chunks_hash
            ON pm_chunks(content_hash);

        CREATE TABLE IF NOT EXISTS pm_vectors (
            chunk_id  INTEGER PRIMARY KEY REFERENCES pm_chunks(id) ON DELETE CASCADE,
            embedding BLOB NOT NULL
        );
        ",
    )
    .map_err(|e| TwolebotError::storage(format!("v4 migration failed: {e}")))?;

    info!("created pm_chunks and pm_vectors tables for PM semantic search");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_and_migrate() {
        let dir = TempDir::new().unwrap();
        let db = WorkDb::open(dir.path()).unwrap();
        let conn = db.conn().unwrap();

        // Verify schema version
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 4);

        // Verify WAL mode
        let journal: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal, "wal");

        // Verify foreign keys enabled
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn test_idempotent_migration() {
        let dir = TempDir::new().unwrap();
        let _db1 = WorkDb::open(dir.path()).unwrap();
        // Opening a second time should not fail (idempotent)
        let db2 = WorkDb::open(dir.path()).unwrap();
        let conn = db2.conn().unwrap();
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 4);
    }

    #[test]
    fn test_tables_created() {
        let dir = TempDir::new().unwrap();
        let db = WorkDb::open(dir.path()).unwrap();
        let conn = db.conn().unwrap();

        let expected_tables = vec![
            "projects",
            "tasks",
            "task_dependencies",
            "task_tags",
            "documents",
            "comments",
            "activity_logs",
            "live_board_selections",
            "schema_version",
            "pm_chunks",
            "pm_vectors",
        ];

        for table in expected_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "table {table} should exist");
        }
    }
}
