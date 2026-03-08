//! PM semantic search — indexes tasks, documents, and comments for vector search.
//!
//! Reuses the existing fastembed infrastructure from `src/semantic/` but stores
//! vectors in the work DB (pm_chunks + pm_vectors tables from V4 migration).

use std::cmp::Ordering;
use std::sync::Arc;

use rusqlite::params;
use sha2::{Digest, Sha256};
use tracing::{debug, info};

use crate::semantic::{Chunk, Chunker, Embedder};
use crate::TwolebotError;

use super::db::WorkDb;

/// A PM search result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PmSearchResult {
    pub entity_type: String,
    pub entity_id: i64,
    pub project_id: i64,
    pub chunk_text: String,
    pub distance: f32,
    /// Extra context: task title, document title, or parent entity info
    pub context: String,
}

/// Stats from an indexing run.
#[derive(Debug, Default)]
pub struct PmIndexStats {
    pub tasks_indexed: usize,
    pub documents_indexed: usize,
    pub comments_indexed: usize,
    pub chunks_created: usize,
    pub skipped_unchanged: usize,
}

/// PM semantic indexer and searcher.
pub struct PmSearch {
    db: WorkDb,
    embedder: Arc<Embedder>,
    chunker: Chunker,
}

impl PmSearch {
    pub fn new(db: WorkDb, embedder: Arc<Embedder>) -> Self {
        Self {
            db,
            embedder,
            chunker: Chunker::default(),
        }
    }

    /// Index all PM content (tasks, documents, comments). Skips unchanged via hash.
    pub fn index_all(&self) -> Result<PmIndexStats, TwolebotError> {
        let mut stats = PmIndexStats::default();

        self.index_all_tasks(&mut stats)?;
        self.index_all_documents(&mut stats)?;
        self.index_all_comments(&mut stats)?;

        info!(
            tasks = stats.tasks_indexed,
            documents = stats.documents_indexed,
            comments = stats.comments_indexed,
            chunks = stats.chunks_created,
            skipped = stats.skipped_unchanged,
            "PM semantic indexing complete"
        );

        Ok(stats)
    }

    /// Index a single entity after mutation.
    pub fn index_entity(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<(), TwolebotError> {
        match entity_type {
            "task" => self.index_task(entity_id),
            "document" => self.index_document(entity_id),
            "comment" => self.index_comment(entity_id),
            _ => Err(TwolebotError::work(format!(
                "unknown entity type: {entity_type}"
            ))),
        }
    }

    /// Search PM content by semantic similarity.
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        project_id: Option<i64>,
    ) -> Result<Vec<PmSearchResult>, TwolebotError> {
        let query_embedding = self
            .embedder
            .embed_one(query)
            .map_err(|e| TwolebotError::work(format!("embed query: {e}")))?;

        let conn = self.db.conn()?;

        let query_row = |row: &rusqlite::Row| -> rusqlite::Result<_> {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        };

        let rows: Vec<_> = if let Some(pid) = project_id {
            let mut stmt = conn
                .prepare(
                    r#"SELECT c.entity_type, c.entity_id, c.project_id, c.chunk_text, v.embedding
                    FROM pm_chunks c
                    JOIN pm_vectors v ON v.chunk_id = c.id
                    WHERE c.project_id = ?1"#,
                )
                .map_err(|e| TwolebotError::work(format!("prepare pm search: {e}")))?;
            let rows: Vec<_> = stmt
                .query_map(params![pid], query_row)
                .map_err(|e| TwolebotError::work(format!("query pm search: {e}")))?
                .filter_map(|r| r.ok())
                .collect();
            rows
        } else {
            let mut stmt = conn
                .prepare(
                    r#"SELECT c.entity_type, c.entity_id, c.project_id, c.chunk_text, v.embedding
                    FROM pm_chunks c
                    JOIN pm_vectors v ON v.chunk_id = c.id"#,
                )
                .map_err(|e| TwolebotError::work(format!("prepare pm search: {e}")))?;
            let rows: Vec<_> = stmt
                .query_map([], query_row)
                .map_err(|e| TwolebotError::work(format!("query pm search: {e}")))?
                .filter_map(|r| r.ok())
                .collect();
            rows
        };

        let mut results: Vec<PmSearchResult> = rows
            .into_iter()
            .map(
                |(entity_type, entity_id, project_id, chunk_text, embedding_bytes)| {
                    let embedding = bytes_to_embedding(&embedding_bytes);
                    let distance = cosine_distance(&query_embedding, &embedding);

                    PmSearchResult {
                        entity_type,
                        entity_id,
                        project_id,
                        chunk_text,
                        distance,
                        context: String::new(),
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

        // Enrich with context (titles)
        for result in &mut results {
            result.context = self
                .get_entity_context(&result.entity_type, result.entity_id)
                .unwrap_or_default();
        }

        Ok(results)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn index_all_tasks(&self, stats: &mut PmIndexStats) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description FROM tasks
                 WHERE status NOT IN ('archived', 'abandoned')",
            )
            .map_err(|e| TwolebotError::work(format!("prepare tasks query: {e}")))?;

        let tasks: Vec<(i64, i64, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| TwolebotError::work(format!("query tasks: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        for (id, project_id, title, description) in tasks {
            let text = format!("{title}\n\n{description}");
            let hash = hash_content(&text);

            if self.hash_unchanged("task", id, &hash)? {
                stats.skipped_unchanged += 1;
                continue;
            }

            self.delete_entity_chunks("task", id)?;
            let chunks = self.chunker.chunk_markdown(&text);
            self.embed_and_store("task", id, project_id, &chunks, &hash)?;
            stats.tasks_indexed += 1;
            stats.chunks_created += chunks.len();
        }

        Ok(())
    }

    fn index_all_documents(&self, stats: &mut PmIndexStats) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, content FROM documents WHERE deleted = 0",
            )
            .map_err(|e| TwolebotError::work(format!("prepare docs query: {e}")))?;

        let docs: Vec<(i64, i64, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| TwolebotError::work(format!("query docs: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        for (id, project_id, title, content) in docs {
            let text = format!("# {title}\n\n{content}");
            let hash = hash_content(&text);

            if self.hash_unchanged("document", id, &hash)? {
                stats.skipped_unchanged += 1;
                continue;
            }

            self.delete_entity_chunks("document", id)?;
            let chunks = self.chunker.chunk_markdown(&text);
            self.embed_and_store("document", id, project_id, &chunks, &hash)?;
            stats.documents_indexed += 1;
            stats.chunks_created += chunks.len();
        }

        Ok(())
    }

    fn index_all_comments(&self, stats: &mut PmIndexStats) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT c.id, COALESCE(t.project_id, d.project_id, 0), c.content
                 FROM comments c
                 LEFT JOIN tasks t ON t.id = c.task_id
                 LEFT JOIN documents d ON d.id = c.document_id",
            )
            .map_err(|e| TwolebotError::work(format!("prepare comments query: {e}")))?;

        let comments: Vec<(i64, i64, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| TwolebotError::work(format!("query comments: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        for (id, project_id, content) in comments {
            let hash = hash_content(&content);

            if self.hash_unchanged("comment", id, &hash)? {
                stats.skipped_unchanged += 1;
                continue;
            }

            self.delete_entity_chunks("comment", id)?;
            // Comments are typically short — single chunk
            let chunks = if content.len() > 500 {
                self.chunker.chunk_markdown(&content)
            } else {
                vec![Chunk { text: content.clone(), index: 0 }]
            };
            self.embed_and_store("comment", id, project_id, &chunks, &hash)?;
            stats.comments_indexed += 1;
            stats.chunks_created += chunks.len();
        }

        Ok(())
    }

    fn index_task(&self, task_id: i64) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let row: (i64, String, String) = conn
            .query_row(
                "SELECT project_id, title, description FROM tasks WHERE id = ?1",
                params![task_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| TwolebotError::work(format!("get task {task_id} for indexing: {e}")))?;

        let text = format!("{}\n\n{}", row.1, row.2);
        let hash = hash_content(&text);
        self.delete_entity_chunks("task", task_id)?;
        let chunks = self.chunker.chunk_markdown(&text);
        self.embed_and_store("task", task_id, row.0, &chunks, &hash)?;
        debug!(task_id, chunks = chunks.len(), "indexed task");
        Ok(())
    }

    fn index_document(&self, doc_id: i64) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let row: (i64, String, String) = conn
            .query_row(
                "SELECT project_id, title, content FROM documents WHERE id = ?1 AND deleted = 0",
                params![doc_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| TwolebotError::work(format!("get doc {doc_id} for indexing: {e}")))?;

        let text = format!("# {}\n\n{}", row.1, row.2);
        let hash = hash_content(&text);
        self.delete_entity_chunks("document", doc_id)?;
        let chunks = self.chunker.chunk_markdown(&text);
        self.embed_and_store("document", doc_id, row.0, &chunks, &hash)?;
        debug!(doc_id, chunks = chunks.len(), "indexed document");
        Ok(())
    }

    fn index_comment(&self, comment_id: i64) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let row: (i64, String) = conn
            .query_row(
                "SELECT COALESCE(
                    (SELECT project_id FROM tasks WHERE id = c.task_id),
                    (SELECT project_id FROM documents WHERE id = c.document_id),
                    0
                ), c.content
                FROM comments c WHERE c.id = ?1",
                params![comment_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| {
                TwolebotError::work(format!("get comment {comment_id} for indexing: {e}"))
            })?;

        let hash = hash_content(&row.1);
        self.delete_entity_chunks("comment", comment_id)?;
        let chunks = if row.1.len() > 500 {
            self.chunker.chunk_markdown(&row.1)
        } else {
            vec![Chunk { text: row.1.clone(), index: 0 }]
        };
        self.embed_and_store("comment", comment_id, row.0, &chunks, &hash)?;
        debug!(comment_id, chunks = chunks.len(), "indexed comment");
        Ok(())
    }

    fn hash_unchanged(
        &self,
        entity_type: &str,
        entity_id: i64,
        new_hash: &str,
    ) -> Result<bool, TwolebotError> {
        let conn = self.db.conn()?;
        let stored_hash: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM pm_chunks WHERE entity_type = ?1 AND entity_id = ?2 LIMIT 1",
                params![entity_type, entity_id],
                |row| row.get(0),
            )
            .ok();

        Ok(stored_hash.as_deref() == Some(new_hash))
    }

    fn delete_entity_chunks(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;

        // CASCADE handles pm_vectors
        conn.execute(
            "DELETE FROM pm_chunks WHERE entity_type = ?1 AND entity_id = ?2",
            params![entity_type, entity_id],
        )
        .map_err(|e| TwolebotError::work(format!("delete pm_chunks: {e}")))?;

        Ok(())
    }

    fn embed_and_store(
        &self,
        entity_type: &str,
        entity_id: i64,
        project_id: i64,
        chunks: &[Chunk],
        content_hash: &str,
    ) -> Result<(), TwolebotError> {
        if chunks.is_empty() {
            return Ok(());
        }

        // Batch embed (max 16 per batch, matching existing pattern)
        for batch_start in (0..chunks.len()).step_by(16) {
            let batch_end = (batch_start + 16).min(chunks.len());
            let batch: Vec<String> = chunks[batch_start..batch_end]
                .iter()
                .map(|c| c.text.clone())
                .collect();

            let embeddings = self
                .embedder
                .embed(batch)
                .map_err(|e| TwolebotError::work(format!("embed batch: {e}")))?;

            let conn = self.db.conn()?;
            for (i, embedding) in embeddings.iter().enumerate() {
                let chunk = &chunks[batch_start + i];

                conn.execute(
                    "INSERT OR REPLACE INTO pm_chunks
                     (entity_type, entity_id, project_id, chunk_index, chunk_text, content_hash)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        entity_type,
                        entity_id,
                        project_id,
                        chunk.index as i64,
                        chunk.text,
                        content_hash,
                    ],
                )
                .map_err(|e| TwolebotError::work(format!("insert pm_chunk: {e}")))?;

                let chunk_id = conn.last_insert_rowid();
                let embedding_bytes = embedding_to_bytes(embedding);

                conn.execute(
                    "INSERT OR REPLACE INTO pm_vectors (chunk_id, embedding) VALUES (?1, ?2)",
                    params![chunk_id, embedding_bytes],
                )
                .map_err(|e| TwolebotError::work(format!("insert pm_vector: {e}")))?;
            }
        }

        Ok(())
    }

    fn get_entity_context(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<String, TwolebotError> {
        let conn = self.db.conn()?;
        match entity_type {
            "task" => {
                let title: String = conn
                    .query_row(
                        "SELECT title FROM tasks WHERE id = ?1",
                        params![entity_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| TwolebotError::work(format!("get task title: {e}")))?;
                Ok(format!("Task #{entity_id}: {title}"))
            }
            "document" => {
                let title: String = conn
                    .query_row(
                        "SELECT title FROM documents WHERE id = ?1",
                        params![entity_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| TwolebotError::work(format!("get doc title: {e}")))?;
                Ok(format!("Document #{entity_id}: {title}"))
            }
            "comment" => {
                let info: (Option<i64>, Option<i64>) = conn
                    .query_row(
                        "SELECT task_id, document_id FROM comments WHERE id = ?1",
                        params![entity_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(|e| TwolebotError::work(format!("get comment parent: {e}")))?;
                match info {
                    (Some(tid), _) => Ok(format!("Comment on task #{tid}")),
                    (_, Some(did)) => Ok(format!("Comment on document #{did}")),
                    _ => Ok(format!("Comment #{entity_id}")),
                }
            }
            _ => Ok(String::new()),
        }
    }
}

// ── Standalone helpers (same as vectordb.rs) ─────────────────────────────────

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    1.0 - (dot / (norm_a * norm_b))
}
