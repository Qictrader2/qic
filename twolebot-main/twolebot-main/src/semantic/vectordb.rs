//! Vector database using SQLite for storing embeddings.
//!
//! Stores document chunks and their embeddings, supports similarity search.
//! Uses monotonic auto-increment integers as primary keys.
//! Uses brute-force cosine similarity search (fast enough for <100k vectors).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

/// Hash content to detect changes.
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// A search result with similarity score.
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub chunk_id: i64,
    pub path: String,
    pub chunk_index: i64,
    pub chunk_text: String,
    pub distance: f32,
}

/// Search result for conversations.
#[derive(Debug, Clone)]
pub struct ConversationSearchResult {
    pub chunk_id: i64,
    pub session_id: String,
    pub project: String,
    pub message_index: i64,
    pub chunk_index: i64,
    pub role: String,
    pub chunk_text: String,
    pub timestamp: String,
    pub distance: f32,
}

/// Vector database for semantic search.
pub struct VectorDb {
    conn: Arc<Mutex<Connection>>,
}

impl VectorDb {
    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| anyhow::anyhow!("SQLite connection mutex poisoned: {e}"))
    }

    /// Open or create the vector database at the given path.
    pub fn open(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn = Connection::open(db_path).context("Failed to open SQLite database")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;

        Ok(db)
    }

    /// Initialize database schema.
    fn init_schema(&self) -> Result<()> {
        let conn = self.lock_conn()?;

        // Memory chunks table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS memory_chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                chunk_text TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(path, chunk_index)
            )
            "#,
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memory_chunks_path ON memory_chunks(path)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memory_chunks_hash ON memory_chunks(file_hash)",
            [],
        )?;

        // Memory embeddings table (standard table, not vec0 for compatibility)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS memory_vectors (
                chunk_id INTEGER PRIMARY KEY,
                embedding BLOB NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES memory_chunks(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Conversation chunks table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS conversation_chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                project TEXT NOT NULL,
                message_index INTEGER NOT NULL,
                chunk_index INTEGER NOT NULL,
                role TEXT NOT NULL,
                chunk_text TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(session_id, message_index, chunk_index)
            )
            "#,
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversation_chunks_session ON conversation_chunks(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversation_chunks_project ON conversation_chunks(project)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversation_chunks_hash ON conversation_chunks(file_hash)",
            [],
        )?;

        // Conversation embeddings table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS conversation_vectors (
                chunk_id INTEGER PRIMARY KEY,
                embedding BLOB NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES conversation_chunks(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Metadata table for tracking indexing state
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS indexing_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
            [],
        )?;

        Ok(())
    }

    /// Convert embedding vector to bytes for storage.
    fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
        embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    /// Convert bytes back to embedding vector.
    fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }

    /// Calculate cosine distance between two embeddings.
    fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0; // Maximum distance for zero vectors
        }

        1.0 - (dot / (norm_a * norm_b))
    }

    // ==================== Memory Operations ====================

    /// Get the stored hash for a memory file, if any.
    pub fn get_memory_file_hash(&self, path: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt =
            conn.prepare("SELECT file_hash FROM memory_chunks WHERE path = ? LIMIT 1")?;

        let hash: Option<String> = stmt.query_row([path], |row| row.get(0)).ok();

        Ok(hash)
    }

    /// Delete all chunks for a memory file.
    pub fn delete_memory_file(&self, path: &str) -> Result<()> {
        let conn = self.lock_conn()?;

        // Delete vectors first (foreign key)
        conn.execute(
            "DELETE FROM memory_vectors WHERE chunk_id IN (SELECT id FROM memory_chunks WHERE path = ?)",
            [path],
        )?;

        // Delete chunks
        conn.execute("DELETE FROM memory_chunks WHERE path = ?", [path])?;

        Ok(())
    }

    /// Insert a memory chunk with its embedding.
    pub fn insert_memory_chunk(
        &self,
        path: &str,
        chunk_index: usize,
        chunk_text: &str,
        file_hash: &str,
        embedding: &[f32],
    ) -> Result<i64> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO memory_chunks (path, chunk_index, chunk_text, file_hash, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            params![path, chunk_index as i64, chunk_text, file_hash, now],
        )?;

        let chunk_id = conn.last_insert_rowid();

        // Insert embedding
        let embedding_bytes = Self::embedding_to_bytes(embedding);
        conn.execute(
            "INSERT OR REPLACE INTO memory_vectors (chunk_id, embedding) VALUES (?, ?)",
            params![chunk_id, embedding_bytes],
        )?;

        Ok(chunk_id)
    }

    /// Search memory chunks by semantic similarity.
    pub fn search_memory_semantic(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        let conn = self.lock_conn()?;

        // Brute-force search (works without sqlite-vec extension)
        let mut stmt = conn.prepare(
            r#"
            SELECT c.id, c.path, c.chunk_index, c.chunk_text, v.embedding
            FROM memory_chunks c
            JOIN memory_vectors v ON v.chunk_id = c.id
            "#,
        )?;

        let mut results: Vec<VectorSearchResult> = stmt
            .query_map([], |row| {
                let chunk_id: i64 = row.get(0)?;
                let path: String = row.get(1)?;
                let chunk_index: i64 = row.get(2)?;
                let chunk_text: String = row.get(3)?;
                let embedding_bytes: Vec<u8> = row.get(4)?;

                Ok((chunk_id, path, chunk_index, chunk_text, embedding_bytes))
            })?
            .filter_map(|r| r.ok())
            .map(
                |(chunk_id, path, chunk_index, chunk_text, embedding_bytes)| {
                    let embedding = Self::bytes_to_embedding(&embedding_bytes);
                    let distance = Self::cosine_distance(query_embedding, &embedding);

                    VectorSearchResult {
                        chunk_id,
                        path,
                        chunk_index,
                        chunk_text,
                        distance,
                    }
                },
            )
            .collect();

        // Sort by distance (ascending = most similar first)
        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    /// Get all memory file paths and their stored hashes.
    pub fn get_all_memory_hashes(&self) -> Result<Vec<(String, String)>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT DISTINCT path, file_hash FROM memory_chunks")?;

        let pairs: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(pairs)
    }

    /// Get all conversation session IDs and their stored hashes.
    pub fn get_all_conversation_hashes(&self) -> Result<Vec<(String, String)>> {
        let conn = self.lock_conn()?;
        let mut stmt =
            conn.prepare("SELECT DISTINCT session_id, file_hash FROM conversation_chunks")?;

        let pairs: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(pairs)
    }

    // ==================== Conversation Operations ====================

    /// Get the stored hash for a conversation session, if any.
    pub fn get_conversation_file_hash(&self, session_id: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt =
            conn.prepare("SELECT file_hash FROM conversation_chunks WHERE session_id = ? LIMIT 1")?;

        let hash: Option<String> = stmt.query_row([session_id], |row| row.get(0)).ok();

        Ok(hash)
    }

    /// Delete all chunks for a conversation session.
    pub fn delete_conversation_session(&self, session_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;

        // Delete vectors first
        conn.execute(
            "DELETE FROM conversation_vectors WHERE chunk_id IN (SELECT id FROM conversation_chunks WHERE session_id = ?)",
            [session_id],
        )?;

        // Delete chunks
        conn.execute(
            "DELETE FROM conversation_chunks WHERE session_id = ?",
            [session_id],
        )?;

        Ok(())
    }

    /// Insert a conversation chunk with its embedding.
    pub fn insert_conversation_chunk(
        &self,
        session_id: &str,
        project: &str,
        message_index: usize,
        chunk_index: usize,
        role: &str,
        chunk_text: &str,
        timestamp: &str,
        file_hash: &str,
        embedding: &[f32],
    ) -> Result<i64> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO conversation_chunks
            (session_id, project, message_index, chunk_index, role, chunk_text, timestamp, file_hash, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                session_id,
                project,
                message_index as i64,
                chunk_index as i64,
                role,
                chunk_text,
                timestamp,
                file_hash,
                now
            ],
        )?;

        let chunk_id = conn.last_insert_rowid();

        // Insert embedding
        let embedding_bytes = Self::embedding_to_bytes(embedding);
        conn.execute(
            "INSERT OR REPLACE INTO conversation_vectors (chunk_id, embedding) VALUES (?, ?)",
            params![chunk_id, embedding_bytes],
        )?;

        Ok(chunk_id)
    }

    /// Search conversation chunks by semantic similarity.
    pub fn search_conversation_semantic(
        &self,
        query_embedding: &[f32],
        limit: usize,
        project_filter: Option<&str>,
    ) -> Result<Vec<ConversationSearchResult>> {
        let conn = self.lock_conn()?;

        let query = if project_filter.is_some() {
            r#"
            SELECT c.id, c.session_id, c.project, c.message_index, c.chunk_index,
                   c.role, c.chunk_text, c.timestamp, v.embedding
            FROM conversation_chunks c
            JOIN conversation_vectors v ON v.chunk_id = c.id
            WHERE c.project = ?
            "#
        } else {
            r#"
            SELECT c.id, c.session_id, c.project, c.message_index, c.chunk_index,
                   c.role, c.chunk_text, c.timestamp, v.embedding
            FROM conversation_chunks c
            JOIN conversation_vectors v ON v.chunk_id = c.id
            "#
        };

        let mut stmt = conn.prepare(query)?;

        let rows: Vec<_> = if let Some(project) = project_filter {
            stmt.query_map([project], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Vec<u8>>(8)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect()
        } else {
            stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Vec<u8>>(8)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };

        let mut results: Vec<ConversationSearchResult> = rows
            .into_iter()
            .map(
                |(
                    chunk_id,
                    session_id,
                    project,
                    message_index,
                    chunk_index,
                    role,
                    chunk_text,
                    timestamp,
                    embedding_bytes,
                )| {
                    let embedding = Self::bytes_to_embedding(&embedding_bytes);
                    let distance = Self::cosine_distance(query_embedding, &embedding);

                    ConversationSearchResult {
                        chunk_id,
                        session_id,
                        project,
                        message_index,
                        chunk_index,
                        role,
                        chunk_text,
                        timestamp,
                        distance,
                    }
                },
            )
            .collect();

        results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    /// Get all unique session IDs.
    pub fn get_all_session_ids(&self) -> Result<Vec<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT DISTINCT session_id FROM conversation_chunks")?;

        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ids)
    }

    // ==================== Metadata Operations ====================

    /// Get metadata value.
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM indexing_metadata WHERE key = ?")?;

        let value: Option<String> = stmt.query_row([key], |row| row.get(0)).ok();

        Ok(value)
    }

    /// Set metadata value.
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO indexing_metadata (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get statistics about the database.
    pub fn get_stats(&self) -> Result<DbStats> {
        let conn = self.lock_conn()?;

        let memory_chunks: i64 =
            conn.query_row("SELECT COUNT(*) FROM memory_chunks", [], |row| row.get(0))?;

        let memory_files: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT path) FROM memory_chunks",
            [],
            |row| row.get(0),
        )?;

        let conversation_chunks: i64 =
            conn.query_row("SELECT COUNT(*) FROM conversation_chunks", [], |row| {
                row.get(0)
            })?;

        let conversation_sessions: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT session_id) FROM conversation_chunks",
            [],
            |row| row.get(0),
        )?;

        Ok(DbStats {
            memory_chunks,
            memory_files,
            conversation_chunks,
            conversation_sessions,
        })
    }
}

/// Database statistics.
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    pub memory_chunks: i64,
    pub memory_files: i64,
    pub conversation_chunks: i64,
    pub conversation_sessions: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::embedder::EMBEDDING_DIM;
    use tempfile::TempDir;

    fn create_test_db() -> (VectorDb, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_vectors.sqlite3");
        let db = VectorDb::open(&db_path).unwrap();
        (db, dir)
    }

    fn dummy_embedding() -> Vec<f32> {
        vec![0.1; EMBEDDING_DIM]
    }

    #[test]
    fn test_create_database() {
        let (db, _dir) = create_test_db();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.memory_chunks, 0);
        assert_eq!(stats.conversation_chunks, 0);
    }

    #[test]
    fn test_insert_and_search_memory() {
        let (db, _dir) = create_test_db();

        // Insert a chunk
        let embedding = dummy_embedding();
        db.insert_memory_chunk("test.md", 0, "Hello world", "hash123", &embedding)
            .unwrap();

        // Search
        let results = db.search_memory_semantic(&embedding, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "test.md");
        assert_eq!(results[0].chunk_text, "Hello world");
        assert!(results[0].distance < 0.001); // Same vector should have ~0 distance
    }

    #[test]
    fn test_delete_memory_file() {
        let (db, _dir) = create_test_db();

        let embedding = dummy_embedding();
        db.insert_memory_chunk("test.md", 0, "Chunk 1", "hash1", &embedding)
            .unwrap();
        db.insert_memory_chunk("test.md", 1, "Chunk 2", "hash1", &embedding)
            .unwrap();
        db.insert_memory_chunk("other.md", 0, "Other", "hash2", &embedding)
            .unwrap();

        // Delete test.md
        db.delete_memory_file("test.md").unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.memory_chunks, 1);
        assert_eq!(stats.memory_files, 1);
    }

    #[test]
    fn test_get_memory_file_hash() {
        let (db, _dir) = create_test_db();

        assert!(db.get_memory_file_hash("test.md").unwrap().is_none());

        let embedding = dummy_embedding();
        db.insert_memory_chunk("test.md", 0, "Content", "myhash", &embedding)
            .unwrap();

        let hash = db.get_memory_file_hash("test.md").unwrap();
        assert_eq!(hash, Some("myhash".to_string()));
    }

    #[test]
    fn test_insert_and_search_conversation() {
        let (db, _dir) = create_test_db();

        let embedding = dummy_embedding();
        db.insert_conversation_chunk(
            "session1",
            "twolebot",
            0,
            0,
            "user",
            "How do I fix this?",
            "2026-01-01T00:00:00Z",
            "hash1",
            &embedding,
        )
        .unwrap();

        let results = db
            .search_conversation_semantic(&embedding, 10, None)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "session1");
        assert_eq!(results[0].project, "twolebot");
    }

    #[test]
    fn test_conversation_project_filter() {
        let (db, _dir) = create_test_db();

        let embedding = dummy_embedding();
        db.insert_conversation_chunk(
            "s1",
            "project1",
            0,
            0,
            "user",
            "Text 1",
            "2026-01-01T00:00:00Z",
            "h1",
            &embedding,
        )
        .unwrap();
        db.insert_conversation_chunk(
            "s2",
            "project2",
            0,
            0,
            "user",
            "Text 2",
            "2026-01-01T00:00:00Z",
            "h2",
            &embedding,
        )
        .unwrap();

        let all = db
            .search_conversation_semantic(&embedding, 10, None)
            .unwrap();
        assert_eq!(all.len(), 2);

        let filtered = db
            .search_conversation_semantic(&embedding, 10, Some("project1"))
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].project, "project1");
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        // Same vector: distance ~0
        assert!(VectorDb::cosine_distance(&a, &b) < 0.001);

        // Orthogonal vectors: distance ~1
        assert!((VectorDb::cosine_distance(&a, &c) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_serialization_roundtrip() {
        let original: Vec<f32> = (0..EMBEDDING_DIM).map(|i| i as f32 * 0.01).collect();
        let bytes = VectorDb::embedding_to_bytes(&original);
        let restored = VectorDb::bytes_to_embedding(&bytes);

        assert_eq!(original.len(), restored.len());
        for (a, b) in original.iter().zip(restored.iter()) {
            assert!((a - b).abs() < 0.0001);
        }
    }

    #[test]
    fn test_metadata() {
        let (db, _dir) = create_test_db();

        assert!(db.get_metadata("test_key").unwrap().is_none());

        db.set_metadata("test_key", "test_value").unwrap();
        assert_eq!(
            db.get_metadata("test_key").unwrap(),
            Some("test_value".to_string())
        );

        // Update
        db.set_metadata("test_key", "new_value").unwrap();
        assert_eq!(
            db.get_metadata("test_key").unwrap(),
            Some("new_value".to_string())
        );
    }

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("hello");
        let hash2 = hash_content("hello");
        let hash3 = hash_content("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex
    }

    /// Helper: create a unit vector pointing in dimension `dim` (one-hot).
    /// Useful for creating vectors with known cosine distances.
    fn one_hot_embedding(dim: usize) -> Vec<f32> {
        let mut v = vec![0.0; EMBEDDING_DIM];
        v[dim % EMBEDDING_DIM] = 1.0;
        v
    }

    /// Helper: create a vector that's a blend of two one-hot vectors.
    /// `blend(0, 1, 0.8)` → 80% dim-0, 20% dim-1 (normalized).
    fn blended_embedding(dim_a: usize, dim_b: usize, weight_a: f32) -> Vec<f32> {
        let mut v = vec![0.0; EMBEDDING_DIM];
        v[dim_a % EMBEDDING_DIM] = weight_a;
        v[dim_b % EMBEDDING_DIM] = 1.0 - weight_a;
        // Normalize
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= norm);
        v
    }

    #[test]
    fn test_memory_semantic_search_ranks_by_similarity() {
        let (db, _dir) = create_test_db();

        // Insert 3 chunks with distinct embedding directions.
        // "cooking" → dim 0, "programming" → dim 1, "music" → dim 2
        let cooking_emb = one_hot_embedding(0);
        let programming_emb = one_hot_embedding(1);
        let music_emb = one_hot_embedding(2);

        db.insert_memory_chunk("cooking.md", 0, "Best pasta recipe", "h1", &cooking_emb)
            .unwrap();
        db.insert_memory_chunk("code.md", 0, "Rust borrow checker tips", "h2", &programming_emb)
            .unwrap();
        db.insert_memory_chunk("music.md", 0, "Jazz guitar improvisation", "h3", &music_emb)
            .unwrap();

        // Query with a vector close to "cooking" (dim 0 + slight dim 1)
        let query = blended_embedding(0, 1, 0.95);
        let results = db.search_memory_semantic(&query, 10).unwrap();

        assert_eq!(results.len(), 3);
        // cooking.md should be first (closest to query)
        assert_eq!(results[0].path, "cooking.md");
        // programming should be second (slight overlap in dim 1)
        assert_eq!(results[1].path, "code.md");
        // music should be last (orthogonal to query)
        assert_eq!(results[2].path, "music.md");
        // Verify distances are properly ordered
        assert!(results[0].distance < results[1].distance);
        assert!(results[1].distance < results[2].distance);
    }

    #[test]
    fn test_memory_semantic_search_finds_similar_not_identical() {
        let (db, _dir) = create_test_db();

        // Insert chunks with overlapping semantic spaces
        let emb_a = blended_embedding(0, 1, 0.7); // "web development"
        let emb_b = blended_embedding(0, 1, 0.3); // "backend systems"
        let emb_c = one_hot_embedding(5); // "gardening" (completely different)

        db.insert_memory_chunk("webdev.md", 0, "React hooks patterns", "h1", &emb_a)
            .unwrap();
        db.insert_memory_chunk("backend.md", 0, "Database optimization", "h2", &emb_b)
            .unwrap();
        db.insert_memory_chunk("garden.md", 0, "Tomato growing guide", "h3", &emb_c)
            .unwrap();

        // Query for "web development" direction
        let query = blended_embedding(0, 1, 0.7);
        let results = db.search_memory_semantic(&query, 3).unwrap();

        // webdev.md should be nearest (exact match), backend.md second (related)
        assert_eq!(results[0].path, "webdev.md");
        assert_eq!(results[1].path, "backend.md");
        assert_eq!(results[2].path, "garden.md");

        // The gap between garden and backend should be much larger than webdev-backend
        let gap_related = results[1].distance - results[0].distance;
        let gap_unrelated = results[2].distance - results[1].distance;
        assert!(
            gap_unrelated > gap_related,
            "Unrelated result should be much further away: gap_related={}, gap_unrelated={}",
            gap_related,
            gap_unrelated
        );
    }

    #[test]
    fn test_conversation_semantic_search_ranks_by_similarity() {
        let (db, _dir) = create_test_db();

        let deploy_emb = one_hot_embedding(10);
        let debug_emb = one_hot_embedding(11);
        let recipe_emb = one_hot_embedding(50);

        db.insert_conversation_chunk(
            "s1", "twolebot", 0, 0, "user",
            "How do I deploy to production?",
            "2026-01-01T00:00:00Z", "h1", &deploy_emb,
        ).unwrap();
        db.insert_conversation_chunk(
            "s2", "twolebot", 0, 0, "user",
            "My debugger keeps crashing",
            "2026-01-02T00:00:00Z", "h2", &debug_emb,
        ).unwrap();
        db.insert_conversation_chunk(
            "s3", "personal", 0, 0, "user",
            "Best chocolate cake recipe",
            "2026-01-03T00:00:00Z", "h3", &recipe_emb,
        ).unwrap();

        // Query for "deployment" (close to deploy_emb direction)
        let query = blended_embedding(10, 11, 0.9);
        let results = db.search_conversation_semantic(&query, 10, None).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].chunk_text, "How do I deploy to production?");
        assert_eq!(results[1].chunk_text, "My debugger keeps crashing");
        assert_eq!(results[2].chunk_text, "Best chocolate cake recipe");
    }

    #[test]
    fn test_conversation_semantic_search_with_project_filter_and_ranking() {
        let (db, _dir) = create_test_db();

        let similar_emb = one_hot_embedding(20);
        let different_emb = one_hot_embedding(100);

        db.insert_conversation_chunk(
            "s1", "project_a", 0, 0, "user",
            "Relevant to query",
            "2026-01-01T00:00:00Z", "h1", &similar_emb,
        ).unwrap();
        db.insert_conversation_chunk(
            "s2", "project_b", 0, 0, "user",
            "Also relevant but wrong project",
            "2026-01-02T00:00:00Z", "h2", &similar_emb,
        ).unwrap();
        db.insert_conversation_chunk(
            "s3", "project_a", 1, 0, "assistant",
            "Unrelated answer in same project",
            "2026-01-03T00:00:00Z", "h3", &different_emb,
        ).unwrap();

        // Filter to project_a only
        let results = db
            .search_conversation_semantic(&similar_emb, 10, Some("project_a"))
            .unwrap();

        assert_eq!(results.len(), 2); // Only project_a chunks
        // The semantically closer one should rank first
        assert_eq!(results[0].chunk_text, "Relevant to query");
        assert!(results[0].distance < results[1].distance);
    }
}
