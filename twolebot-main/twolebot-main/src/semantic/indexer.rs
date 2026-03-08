//! Background indexer for semantic search.
//!
//! - Memory files: indexed via file watching (immediate updates)
//! - Conversations: indexed via 5-minute polling (to avoid interference with Claude Code)
//!
//! Resource limits:
//! - Batch size limited to 16 chunks at a time (prevents memory spikes)
//! - 100ms delay between batches (prevents CPU saturation)

use anyhow::{Context, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Notify, RwLock};
use tokio::task::JoinHandle;
use walkdir::WalkDir;

use super::chunker::Chunker;
use super::embedder::Embedder;
use super::vectordb::{hash_content, VectorDb};

/// Maximum chunks to embed in a single batch (prevents memory spikes)
const EMBEDDING_BATCH_SIZE: usize = 16;

/// Delay between embedding batches (prevents CPU saturation)
const BATCH_DELAY: Duration = Duration::from_millis(100);
pub const CODEX_SESSION_PREFIX: &str = "codex::";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConversationSource {
    Claude,
    Codex,
}

impl ConversationSource {
    fn as_str(self) -> &'static str {
        match self {
            ConversationSource::Claude => "claude",
            ConversationSource::Codex => "codex",
        }
    }

    fn normalized_session_id(self, raw_session_id: &str) -> String {
        match self {
            ConversationSource::Claude => raw_session_id.to_string(),
            ConversationSource::Codex => format!("{CODEX_SESSION_PREFIX}{raw_session_id}"),
        }
    }
}

// ==================== Status Types ====================

/// Current activity of an indexer task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexerActivity {
    /// Waiting for changes
    Idle,
    /// Performing initial full index
    InitialIndex,
    /// Processing file changes
    Indexing,
    /// Polling for updates
    Polling,
    /// Paused by user
    Paused,
}

/// Status of a single indexer task (memory or conversations).
#[derive(Debug, Clone, Serialize)]
pub struct TaskStatus {
    /// What the task is currently doing
    pub activity: IndexerActivity,
    /// Current file being processed (if any)
    pub current_file: Option<String>,
    /// Files indexed in current batch
    pub files_indexed: usize,
    /// Files skipped (unchanged) in current batch
    pub files_skipped: usize,
    /// Total files to process (if known)
    pub files_total: Option<usize>,
    /// Chunks processed in current file
    pub chunks_processed: usize,
    /// Total chunks in current file (if known)
    pub chunks_total: Option<usize>,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self {
            activity: IndexerActivity::Idle,
            current_file: None,
            files_indexed: 0,
            files_skipped: 0,
            files_total: None,
            chunks_processed: 0,
            chunks_total: None,
        }
    }
}

/// Complete status of the semantic indexer.
#[derive(Debug, Clone, Serialize)]
pub struct IndexerStatus {
    /// Whether semantic search is enabled
    pub enabled: bool,
    /// Memory indexer status
    pub memory: TaskStatus,
    /// Conversation indexer status
    pub conversations: TaskStatus,
    /// Total memory chunks in database
    pub total_memory_chunks: i64,
    /// Total memory files in database
    pub total_memory_files: i64,
    /// Total conversation chunks in database
    pub total_conversation_chunks: i64,
    /// Total conversation sessions in database
    pub total_conversation_sessions: i64,
    /// Total .md files available in memory directory
    pub total_memory_files_available: usize,
    /// Total .jsonl files available in conversations directory
    pub total_conversation_files_available: usize,
    /// Memory files in DB whose hash no longer matches filesystem
    pub memory_files_stale: usize,
    /// Conversation files in DB whose hash no longer matches filesystem
    pub conversation_files_stale: usize,
    /// Unix timestamp (seconds) of last conversation poll
    pub last_conversation_poll_at: Option<u64>,
    /// Conversation poll interval in seconds
    pub conversation_poll_interval_secs: u64,
}

impl Default for IndexerStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            memory: TaskStatus::default(),
            conversations: TaskStatus::default(),
            total_memory_chunks: 0,
            total_memory_files: 0,
            total_conversation_chunks: 0,
            total_conversation_sessions: 0,
            total_memory_files_available: 0,
            total_conversation_files_available: 0,
            memory_files_stale: 0,
            conversation_files_stale: 0,
            last_conversation_poll_at: None,
            conversation_poll_interval_secs: 300,
        }
    }
}

/// Shared status that can be read from the API.
pub type SharedStatus = Arc<RwLock<IndexerStatus>>;

/// Create a new shared status with semantic disabled.
pub fn disabled_status() -> SharedStatus {
    Arc::new(RwLock::new(IndexerStatus::default()))
}

/// Configuration for the semantic indexer.
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Directory containing memory files
    pub memory_dir: PathBuf,
    /// Directory containing Claude Code conversation files
    pub claude_conversations_dir: PathBuf,
    /// Directory containing Codex conversation files
    pub codex_conversations_dir: PathBuf,
    /// Path to the vector database
    pub db_path: PathBuf,
    /// Interval for conversation polling (default: 5 minutes)
    pub conversation_poll_interval: Duration,
    /// Debounce time for file watching (default: 500ms)
    pub debounce_duration: Duration,
}

impl IndexerConfig {
    pub fn new(data_dir: &Path, claude_conversations_dir: &Path, codex_conversations_dir: &Path) -> Self {
        Self {
            memory_dir: data_dir.join("memory"),
            claude_conversations_dir: claude_conversations_dir.to_path_buf(),
            codex_conversations_dir: codex_conversations_dir.to_path_buf(),
            db_path: data_dir.join("vectors.sqlite3"),
            conversation_poll_interval: Duration::from_secs(300), // 5 minutes
            debounce_duration: Duration::from_millis(500),
        }
    }
}

/// Background semantic indexer.
pub struct SemanticIndexer {
    config: IndexerConfig,
    db: Arc<VectorDb>,
    embedder: Arc<Embedder>,
    chunker: Chunker,
    status: SharedStatus,
    paused: Arc<AtomicBool>,
    conversation_notify: Arc<Notify>,
}

impl SemanticIndexer {
    /// Create a new semantic indexer.
    pub async fn new(
        config: IndexerConfig,
        initial_paused: bool,
        omp_num_threads: u16,
    ) -> Result<Self> {
        let db = VectorDb::open(&config.db_path).context("Failed to open vector database")?;
        let embedder = Embedder::global(omp_num_threads)
            .await
            .context("Failed to initialize embedder")?;

        // Initialize status with db stats
        let stats = db.get_stats().unwrap_or_default();
        let initial_activity = if initial_paused {
            IndexerActivity::Paused
        } else {
            IndexerActivity::Idle
        };
        let poll_interval_secs = config.conversation_poll_interval.as_secs();
        let status = Arc::new(RwLock::new(IndexerStatus {
            enabled: !initial_paused,
            memory: TaskStatus {
                activity: initial_activity.clone(),
                ..TaskStatus::default()
            },
            conversations: TaskStatus {
                activity: initial_activity,
                ..TaskStatus::default()
            },
            total_memory_chunks: stats.memory_chunks,
            total_memory_files: stats.memory_files,
            total_conversation_chunks: stats.conversation_chunks,
            total_conversation_sessions: stats.conversation_sessions,
            total_memory_files_available: 0,
            total_conversation_files_available: 0,
            memory_files_stale: 0,
            conversation_files_stale: 0,
            last_conversation_poll_at: None,
            conversation_poll_interval_secs: poll_interval_secs,
        }));

        Ok(Self {
            config,
            db: Arc::new(db),
            embedder,
            chunker: Chunker::default(),
            status,
            paused: Arc::new(AtomicBool::new(initial_paused)),
            conversation_notify: Arc::new(Notify::new()),
        })
    }

    /// Get a reference to the vector database.
    pub fn db(&self) -> Arc<VectorDb> {
        Arc::clone(&self.db)
    }

    /// Get a reference to the embedder.
    pub fn embedder(&self) -> Arc<Embedder> {
        Arc::clone(&self.embedder)
    }

    /// Get a reference to the shared status.
    pub fn status(&self) -> SharedStatus {
        Arc::clone(&self.status)
    }

    /// Get a reference to the paused flag (shared with background tasks).
    pub fn paused(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.paused)
    }

    /// Get a reference to the conversation notify handle (for triggering immediate reindex).
    pub fn conversation_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.conversation_notify)
    }

    // ==================== Status Updates ====================

    async fn set_memory_activity(&self, activity: IndexerActivity) {
        let mut status = self.status.write().await;
        status.memory.activity = activity;
    }

    async fn set_memory_file(&self, file: Option<&str>) {
        let mut status = self.status.write().await;
        status.memory.current_file = file.map(|s| s.to_string());
    }

    async fn update_memory_progress(&self, indexed: usize, skipped: usize, total: Option<usize>) {
        let mut status = self.status.write().await;
        status.memory.files_indexed = indexed;
        status.memory.files_skipped = skipped;
        status.memory.files_total = total;
    }

    async fn set_memory_chunks(&self, processed: usize, total: Option<usize>) {
        let mut status = self.status.write().await;
        status.memory.chunks_processed = processed;
        status.memory.chunks_total = total;
    }

    async fn set_conversation_activity(&self, activity: IndexerActivity) {
        let mut status = self.status.write().await;
        status.conversations.activity = activity;
    }

    async fn set_conversation_file(&self, file: Option<&str>) {
        let mut status = self.status.write().await;
        status.conversations.current_file = file.map(|s| s.to_string());
    }

    async fn update_conversation_progress(
        &self,
        indexed: usize,
        skipped: usize,
        total: Option<usize>,
    ) {
        let mut status = self.status.write().await;
        status.conversations.files_indexed = indexed;
        status.conversations.files_skipped = skipped;
        status.conversations.files_total = total;
    }

    async fn set_conversation_chunks(&self, processed: usize, total: Option<usize>) {
        let mut status = self.status.write().await;
        status.conversations.chunks_processed = processed;
        status.conversations.chunks_total = total;
    }

    async fn refresh_db_stats(&self) {
        if let Ok(stats) = self.db.get_stats() {
            let mut status = self.status.write().await;
            status.total_memory_chunks = stats.memory_chunks;
            status.total_memory_files = stats.memory_files;
            status.total_conversation_chunks = stats.conversation_chunks;
            status.total_conversation_sessions = stats.conversation_sessions;
        }
    }

    fn count_memory_files(&self) -> usize {
        if !self.config.memory_dir.exists() {
            return 0;
        }
        WalkDir::new(&self.config.memory_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .count()
    }

    fn count_conversation_files(&self) -> usize {
        self.collect_conversation_files().len()
    }

    async fn update_file_counts(&self) {
        let mem = self.count_memory_files();
        let conv = self.count_conversation_files();
        let mem_stale = self.count_stale_memory_files();
        let conv_stale = self.count_stale_conversation_files();
        let mut status = self.status.write().await;
        status.total_memory_files_available = mem;
        status.total_conversation_files_available = conv;
        status.memory_files_stale = mem_stale;
        status.conversation_files_stale = conv_stale;
    }

    /// Count memory files whose content hash differs from what's in the DB.
    fn count_stale_memory_files(&self) -> usize {
        let db_hashes = match self.db.get_all_memory_hashes() {
            Ok(h) => h,
            Err(_) => return 0,
        };

        db_hashes
            .iter()
            .filter(|(path, stored_hash)| {
                let full_path = self.config.memory_dir.join(path);
                match std::fs::read_to_string(&full_path) {
                    Ok(content) => hash_content(&content) != *stored_hash,
                    Err(_) => true, // file deleted = stale
                }
            })
            .count()
    }

    /// Count conversation files whose content hash differs from what's in the DB.
    /// Builds a file lookup map once instead of walking the tree per session.
    fn count_stale_conversation_files(&self) -> usize {
        let db_hashes = match self.db.get_all_conversation_hashes() {
            Ok(h) => h,
            Err(_) => return 0,
        };

        if db_hashes.is_empty() {
            return 0;
        }

        // Build a map of session_id -> file_path once (avoids O(n*m) walks)
        let file_map = self.build_conversation_file_map();

        db_hashes
            .iter()
            .filter(|(session_id, stored_hash)| {
                match file_map.get(session_id.as_str()) {
                    Some(path) => match std::fs::read_to_string(path) {
                        Ok(content) => hash_content(&content) != *stored_hash,
                        Err(_) => true,
                    },
                    None => true, // file no longer exists = stale
                }
            })
            .count()
    }

    /// Build a map of session_id -> file_path for all conversation files.
    fn build_conversation_file_map(&self) -> HashMap<String, PathBuf> {
        self.collect_conversation_files()
            .into_iter()
            .filter_map(|(source, path)| {
                let raw_session_id = path.file_stem()?.to_str()?;
                Some((source.normalized_session_id(raw_session_id), path))
            })
            .collect()
    }

    fn collect_conversation_files(&self) -> Vec<(ConversationSource, PathBuf)> {
        let mut files = Vec::new();

        files.extend(
            self.collect_source_files(
                &self.config.claude_conversations_dir,
                ConversationSource::Claude,
            ),
        );
        files.extend(
            self.collect_source_files(
                &self.config.codex_conversations_dir,
                ConversationSource::Codex,
            ),
        );

        files
    }

    fn collect_source_files(
        &self,
        dir: &Path,
        source: ConversationSource,
    ) -> Vec<(ConversationSource, PathBuf)> {
        if !dir.exists() {
            return Vec::new();
        }

        WalkDir::new(dir)
            .follow_links(false)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
            .map(|e| (source, e.path().to_path_buf()))
            .collect()
    }

    async fn record_conversation_poll_time(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut status = self.status.write().await;
        status.last_conversation_poll_at = Some(now);
    }

    /// Start the background indexing tasks.
    ///
    /// Returns:
    /// - SharedStatus for monitoring
    /// - Tuple of handles: (memory watcher, conversation poller)
    pub fn start(self) -> (SharedStatus, (JoinHandle<()>, JoinHandle<()>)) {
        let status = Arc::clone(&self.status);
        let indexer = Arc::new(self);

        // Memory file watcher
        let memory_handle = {
            let indexer = Arc::clone(&indexer);
            tokio::spawn(async move {
                if let Err(e) = indexer.run_memory_watcher().await {
                    tracing::error!(error = %e, "Memory watcher failed");
                }
            })
        };

        // Conversation poller
        let conversation_handle = {
            let indexer = Arc::clone(&indexer);
            tokio::spawn(async move {
                indexer.run_conversation_poller().await;
            })
        };

        (status, (memory_handle, conversation_handle))
    }

    /// Check if the indexer is currently paused.
    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// Wait until the indexer is unpaused, checking every second.
    async fn wait_for_unpause(&self) {
        while self.is_paused() {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Run the memory file watcher.
    async fn run_memory_watcher(self: &Arc<Self>) -> Result<()> {
        // Ensure memory directory exists
        if !self.config.memory_dir.exists() {
            std::fs::create_dir_all(&self.config.memory_dir)?;
        }

        // Wait for unpause before initial index
        self.wait_for_unpause().await;

        // Initial full index
        tracing::info!(dir = ?self.config.memory_dir, "Starting initial memory index");
        self.set_memory_activity(IndexerActivity::InitialIndex)
            .await;
        self.index_all_memories().await?;
        self.set_memory_activity(IndexerActivity::Idle).await;
        self.refresh_db_stats().await;
        self.update_file_counts().await;
        tracing::info!("Initial memory index complete");

        // Set up file watcher
        let (tx, mut rx) = mpsc::channel::<Result<Event, notify::Error>>(100);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.blocking_send(res);
            },
            notify::Config::default(),
        )?;

        watcher.watch(&self.config.memory_dir, RecursiveMode::Recursive)?;

        tracing::info!(dir = ?self.config.memory_dir, "Watching for memory file changes");

        // Debounced processing
        let mut pending: HashSet<PathBuf> = HashSet::new();
        let mut last_process = Instant::now();

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    if let Ok(event) = event {
                        for path in event.paths {
                            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                                pending.insert(path);
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Check if debounce time has passed
                    if !pending.is_empty() && last_process.elapsed() >= self.config.debounce_duration {
                        // Skip processing while paused (keep pending events for later)
                        if self.is_paused() {
                            continue;
                        }
                        self.set_memory_activity(IndexerActivity::Indexing).await;
                        let total = pending.len();
                        let mut indexed = 0;
                        let mut skipped = 0;

                        for path in pending.drain() {
                            let rel_path = self.relative_memory_path(&path);
                            self.set_memory_file(Some(&rel_path)).await;
                            self.update_memory_progress(indexed, skipped, Some(total)).await;

                            match self.index_memory_file(&path).await {
                                Ok(true) => indexed += 1,
                                Ok(false) => skipped += 1,
                                Err(e) => {
                                    tracing::warn!(path = ?path, error = %e, "Failed to index memory file");
                                }
                            }
                        }

                        self.set_memory_file(None).await;
                        self.update_memory_progress(indexed, skipped, None).await;
                        self.set_memory_activity(IndexerActivity::Idle).await;
                        self.refresh_db_stats().await;
                        self.update_file_counts().await;
                        last_process = Instant::now();
                    }
                }
            }
        }
    }

    /// Run the conversation poller (every 5 minutes).
    async fn run_conversation_poller(self: &Arc<Self>) {
        // Wait for unpause before initial index
        self.wait_for_unpause().await;

        // Initial index
        tracing::info!(
            claude_dir = ?self.config.claude_conversations_dir,
            codex_dir = ?self.config.codex_conversations_dir,
            "Starting initial conversation index"
        );

        self.set_conversation_activity(IndexerActivity::InitialIndex)
            .await;
        self.record_conversation_poll_time().await;
        if let Err(e) = self.index_all_conversations().await {
            tracing::error!(error = %e, "Initial conversation index failed");
        } else {
            tracing::info!("Initial conversation index complete");
        }
        self.set_conversation_activity(IndexerActivity::Idle).await;
        self.refresh_db_stats().await;
        self.update_file_counts().await;

        // Poll loop
        loop {
            // Wait for next poll interval or manual trigger
            tokio::select! {
                _ = tokio::time::sleep(self.config.conversation_poll_interval) => {}
                _ = self.conversation_notify.notified() => {
                    tracing::info!("Conversation reindex triggered manually");
                }
            }

            // Skip poll while paused
            if self.is_paused() {
                continue;
            }

            tracing::debug!("Polling for conversation changes");
            self.record_conversation_poll_time().await;
            self.set_conversation_activity(IndexerActivity::Polling)
                .await;
            if let Err(e) = self.poll_conversations().await {
                tracing::warn!(error = %e, "Conversation poll failed");
            }
            self.set_conversation_activity(IndexerActivity::Idle).await;
            self.refresh_db_stats().await;
            self.update_file_counts().await;
        }
    }

    // ==================== Memory Indexing ====================

    /// Index all memory files.
    async fn index_all_memories(&self) -> Result<()> {
        if !self.config.memory_dir.exists() {
            return Ok(());
        }

        // Collect all files first to get total count
        let files: Vec<_> = WalkDir::new(&self.config.memory_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect();

        let total = files.len();
        let mut indexed = 0;
        let mut skipped = 0;

        for entry in files {
            let path = entry.path();
            let rel_path = self.relative_memory_path(path);
            self.set_memory_file(Some(&rel_path)).await;
            self.update_memory_progress(indexed, skipped, Some(total))
                .await;

            match self.index_memory_file(path).await {
                Ok(true) => indexed += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    tracing::warn!(path = ?path, error = %e, "Failed to index memory file");
                }
            }
        }

        self.set_memory_file(None).await;
        self.update_memory_progress(indexed, skipped, None).await;
        tracing::info!(indexed, skipped, "Memory indexing complete");
        Ok(())
    }

    /// Index a single memory file.
    ///
    /// Returns Ok(true) if indexed, Ok(false) if skipped (unchanged).
    async fn index_memory_file(&self, path: &Path) -> Result<bool> {
        // Read file content
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File was deleted - remove from index
                let rel_path = self.relative_memory_path(path);
                self.db.delete_memory_file(&rel_path)?;
                tracing::debug!(path = ?path, "Removed deleted file from index");
                return Ok(true);
            }
            Err(e) => return Err(e.into()),
        };

        let file_hash = hash_content(&content);
        let rel_path = self.relative_memory_path(path);

        // Check if file has changed
        if let Some(existing_hash) = self.db.get_memory_file_hash(&rel_path)? {
            if existing_hash == file_hash {
                return Ok(false); // Unchanged
            }
        }

        // Delete existing chunks for this file
        self.db.delete_memory_file(&rel_path)?;

        // Chunk the content
        let chunks = self.chunker.chunk_markdown(&content);
        if chunks.is_empty() {
            return Ok(true);
        }

        // Generate embeddings in small batches with delays (resource limiting)
        let total_chunks = chunks.len();
        let mut chunks_done = 0;
        self.set_memory_chunks(0, Some(total_chunks)).await;

        for batch in chunks.chunks(EMBEDDING_BATCH_SIZE) {
            let texts: Vec<String> = batch.iter().map(|c| c.text.clone()).collect();
            let embeddings = self.embedder.embed(texts)?;

            // Insert chunks with embeddings
            for (chunk, embedding) in batch.iter().zip(embeddings.iter()) {
                self.db.insert_memory_chunk(
                    &rel_path,
                    chunk.index,
                    &chunk.text,
                    &file_hash,
                    embedding,
                )?;
            }

            chunks_done += batch.len();
            self.set_memory_chunks(chunks_done, Some(total_chunks)).await;

            // Small delay between batches to prevent CPU saturation
            if total_chunks > EMBEDDING_BATCH_SIZE {
                tokio::time::sleep(BATCH_DELAY).await;
            }
        }

        self.set_memory_chunks(0, None).await;
        tracing::debug!(path = %rel_path, chunks = total_chunks, "Indexed memory file");
        Ok(true)
    }

    /// Get relative path for a memory file.
    fn relative_memory_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.config.memory_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    // ==================== Conversation Indexing ====================

    /// Index all conversation files.
    async fn index_all_conversations(&self) -> Result<()> {
        let files = self.collect_conversation_files();
        if files.is_empty() {
            tracing::debug!(
                claude_dir = ?self.config.claude_conversations_dir,
                codex_dir = ?self.config.codex_conversations_dir,
                "No conversation files found for indexing"
            );
            return Ok(());
        }

        let total = files.len();
        let mut indexed = 0;
        let mut skipped = 0;

        for (source, path) in files {
            let raw_session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let display_id = format!("{}:{raw_session_id}", source.as_str());
            self.set_conversation_file(Some(&display_id)).await;
            self.update_conversation_progress(indexed, skipped, Some(total))
                .await;

            match self.index_conversation_file(source, &path).await {
                Ok(true) => indexed += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    tracing::warn!(path = ?path, error = %e, "Failed to index conversation file");
                }
            }
        }

        self.set_conversation_file(None).await;
        self.update_conversation_progress(indexed, skipped, None)
            .await;
        tracing::info!(indexed, skipped, "Conversation indexing complete");
        Ok(())
    }

    /// Poll for changed conversation files.
    async fn poll_conversations(&self) -> Result<()> {
        let files = self.collect_conversation_files();
        if files.is_empty() {
            return Ok(());
        }

        let total = files.len();
        let mut indexed = 0;
        let mut skipped = 0;

        // Build set of session IDs on disk for orphan detection
        let mut disk_session_ids: HashSet<String> = HashSet::new();

        for (source, path) in files {
            let raw_session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let normalized_session_id = source.normalized_session_id(&raw_session_id);

            disk_session_ids.insert(normalized_session_id.clone());
            let display_id = format!("{}:{raw_session_id}", source.as_str());
            self.set_conversation_file(Some(&display_id)).await;
            self.update_conversation_progress(indexed, skipped, Some(total))
                .await;

            // Check if file has been modified by comparing content hash.
            match self.index_conversation_file(source, &path).await {
                Ok(true) => indexed += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    tracing::warn!(path = ?path, error = %e, "Failed to index conversation file");
                }
            }
        }

        self.set_conversation_file(None).await;
        self.update_conversation_progress(indexed, skipped, None)
            .await;

        // Clean up orphaned DB entries (files deleted by Claude Code, e.g. subagent sessions)
        if let Ok(db_sessions) = self.db.get_all_session_ids() {
            let mut orphans_removed = 0;
            for session_id in &db_sessions {
                if !disk_session_ids.contains(session_id) {
                    if let Err(e) = self.db.delete_conversation_session(session_id) {
                        tracing::warn!(session_id = %session_id, error = %e, "Failed to remove orphaned session");
                    } else {
                        orphans_removed += 1;
                    }
                }
            }
            if orphans_removed > 0 {
                tracing::info!(
                    orphans_removed,
                    "Removed orphaned conversation sessions from index"
                );
            }
        }

        if indexed > 0 {
            tracing::info!(indexed, "Updated conversation indices");
        }

        Ok(())
    }

    /// Index a single conversation file.
    ///
    /// Returns Ok(true) if indexed, Ok(false) if skipped (unchanged).
    async fn index_conversation_file(&self, source: ConversationSource, path: &Path) -> Result<bool> {
        let content = std::fs::read_to_string(path)?;
        let file_hash = hash_content(&content);

        // Extract session ID from filename
        let raw_session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let session_id = source.normalized_session_id(&raw_session_id);

        if raw_session_id.is_empty() {
            return Ok(false);
        }

        // Check if file has changed
        if let Some(existing_hash) = self.db.get_conversation_file_hash(&session_id)? {
            if existing_hash == file_hash {
                return Ok(false); // Unchanged
            }
        }

        // Extract project name from path/content
        let project = match source {
            ConversationSource::Claude => self.extract_claude_project_name(path),
            ConversationSource::Codex => self.extract_codex_project_name(path, &content),
        };

        // Delete existing chunks for this session
        self.db.delete_conversation_session(&session_id)?;

        // Parse messages
        let messages = match source {
            ConversationSource::Claude => self.parse_claude_conversation_messages(&content),
            ConversationSource::Codex => self.parse_codex_conversation_messages(&content),
        };
        if messages.is_empty() {
            return Ok(true);
        }

        // Batch collect all chunks for embedding
        let mut all_chunks: Vec<(usize, String, String, usize, String)> = Vec::new(); // (msg_idx, role, timestamp, chunk_idx, text)

        for (msg_idx, msg) in messages.iter().enumerate() {
            let chunks = self.chunker.chunk_message(&msg.content);
            for chunk in chunks {
                all_chunks.push((
                    msg_idx,
                    msg.role.clone(),
                    msg.timestamp.clone(),
                    chunk.index,
                    chunk.text,
                ));
            }
        }

        if all_chunks.is_empty() {
            return Ok(true);
        }

        // Generate embeddings in small batches with delays (resource limiting)
        let total_chunks = all_chunks.len();
        let mut chunks_done = 0;
        self.set_conversation_chunks(0, Some(total_chunks)).await;

        for batch_start in (0..total_chunks).step_by(EMBEDDING_BATCH_SIZE) {
            let batch_end = (batch_start + EMBEDDING_BATCH_SIZE).min(total_chunks);
            let batch = &all_chunks[batch_start..batch_end];

            let texts: Vec<String> = batch.iter().map(|(_, _, _, _, t)| t.clone()).collect();
            let embeddings = self.embedder.embed(texts)?;

            // Insert chunks with embeddings
            for ((msg_idx, role, timestamp, chunk_idx, text), embedding) in
                batch.iter().zip(embeddings.iter())
            {
                self.db.insert_conversation_chunk(
                    &session_id,
                    &project,
                    *msg_idx,
                    *chunk_idx,
                    role,
                    text,
                    timestamp,
                    &file_hash,
                    embedding,
                )?;
            }

            chunks_done += batch.len();
            self.set_conversation_chunks(chunks_done, Some(total_chunks))
                .await;

            // Small delay between batches to prevent CPU saturation
            if total_chunks > EMBEDDING_BATCH_SIZE {
                tokio::time::sleep(BATCH_DELAY).await;
            }
        }

        self.set_conversation_chunks(0, None).await;
        tracing::debug!(
            source = source.as_str(),
            session_id = %session_id,
            project = %project,
            messages = messages.len(),
            chunks = total_chunks,
            "Indexed conversation file"
        );

        Ok(true)
    }

    /// Extract project name from Claude conversation path.
    fn extract_claude_project_name(&self, path: &Path) -> String {
        // Path format: ~/.claude/projects/-home-schalk-git-twolebot-data/session.jsonl
        // or: ~/.claude/projects/-home-schalk-git-twolebot-data/subagents/session.jsonl

        let parent = path.parent();
        let project_dir = parent.and_then(|p| {
            if p.file_name().and_then(|n| n.to_str()) == Some("subagents") {
                p.parent()
            } else {
                Some(p)
            }
        });

        let dir_name = project_dir
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Extract project name from encoded path: -home-schalk-git-twolebot-data -> twolebot
        let parts: Vec<&str> = dir_name.split('-').collect();
        if let Some(git_idx) = parts.iter().position(|&s| s == "git") {
            if git_idx + 1 < parts.len() {
                return parts[git_idx + 1].to_string();
            }
        }

        dir_name.to_string()
    }

    /// Extract project name from Codex conversation content/path.
    fn extract_codex_project_name(&self, path: &Path, content: &str) -> String {
        for line in content.lines() {
            let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if json.get("type").and_then(|v| v.as_str()) != Some("session_meta") {
                continue;
            }
            if let Some(cwd) = json
                .get("payload")
                .and_then(|p| p.get("cwd"))
                .and_then(|c| c.as_str())
            {
                if let Some(project) = Self::extract_project_from_cwd(cwd) {
                    return project;
                }
            }
        }

        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("codex")
            .to_string()
    }

    fn extract_project_from_cwd(cwd: &str) -> Option<String> {
        let components: Vec<String> = Path::new(cwd)
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(ToOwned::to_owned))
            .collect();
        if let Some(git_idx) = components.iter().position(|c| c == "git") {
            if git_idx + 1 < components.len() {
                return Some(components[git_idx + 1].clone());
            }
        }
        components.last().cloned()
    }

    fn extract_text_value(content_val: &serde_json::Value) -> Option<String> {
        match content_val {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => {
                let text = arr
                    .iter()
                    .filter_map(|item| {
                        item.get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            _ => None,
        }
    }

    /// Parse Claude conversation messages from JSONL content.
    fn parse_claude_conversation_messages(&self, content: &str) -> Vec<ParsedMessage> {
        content
            .lines()
            .filter_map(|line| {
                let json: serde_json::Value = serde_json::from_str(line).ok()?;

                let msg_type = json.get("type")?.as_str()?;
                if msg_type != "user" && msg_type != "assistant" {
                    return None;
                }

                let message = json.get("message")?;
                let role = message.get("role")?.as_str()?.to_string();
                let content_val = message.get("content")?;
                let content = Self::extract_text_value(content_val)?;

                let timestamp = json
                    .get("timestamp")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(ParsedMessage {
                    role,
                    content,
                    timestamp,
                })
            })
            .collect()
    }

    /// Parse Codex conversation messages from JSONL content.
    fn parse_codex_conversation_messages(&self, content: &str) -> Vec<ParsedMessage> {
        content
            .lines()
            .filter_map(|line| {
                let json: serde_json::Value = serde_json::from_str(line).ok()?;
                if json.get("type")?.as_str()? != "response_item" {
                    return None;
                }
                let payload = json.get("payload")?;
                if payload.get("type")?.as_str()? != "message" {
                    return None;
                }
                let role = payload.get("role")?.as_str()?.to_string();
                if role != "user" && role != "assistant" {
                    return None;
                }
                let content_val = payload.get("content")?;
                let content = Self::extract_text_value(content_val)?;
                let timestamp = json
                    .get("timestamp")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(ParsedMessage {
                    role,
                    content,
                    timestamp,
                })
            })
            .collect()
    }
}

/// A parsed message from conversation JSONL.
#[derive(Debug)]
struct ParsedMessage {
    role: String,
    content: String,
    timestamp: String,
}

// Indexer tests that use embeddings are ignored by default (linker OOM with onnxruntime)
// Run with: cargo test --ignored
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[allow(dead_code)]
    async fn create_test_indexer() -> (SemanticIndexer, TempDir) {
        let dir = TempDir::new().unwrap();
        let config = IndexerConfig {
            memory_dir: dir.path().join("memory"),
            claude_conversations_dir: dir.path().join("conversations"),
            codex_conversations_dir: dir.path().join("codex"),
            db_path: dir.path().join("vectors.sqlite3"),
            conversation_poll_interval: Duration::from_secs(60),
            debounce_duration: Duration::from_millis(100),
        };

        std::fs::create_dir_all(&config.memory_dir).unwrap();
        std::fs::create_dir_all(&config.claude_conversations_dir).unwrap();
        std::fs::create_dir_all(&config.codex_conversations_dir).unwrap();

        let indexer = SemanticIndexer::new(config, false, 2).await.unwrap();
        (indexer, dir)
    }

    #[tokio::test]
    #[ignore]
    async fn test_index_memory_file() {
        let (indexer, dir) = create_test_indexer().await;

        // Create a test file
        let test_file = dir.path().join("memory/test.md");
        std::fs::write(&test_file, "# Test\n\nThis is test content.").unwrap();

        // Index it
        let indexed = indexer.index_memory_file(&test_file).await.unwrap();
        assert!(indexed);

        // Second index should skip (unchanged)
        let indexed = indexer.index_memory_file(&test_file).await.unwrap();
        assert!(!indexed);

        // Modify and re-index
        std::fs::write(&test_file, "# Test\n\nUpdated content.").unwrap();
        let indexed = indexer.index_memory_file(&test_file).await.unwrap();
        assert!(indexed);
    }

    #[tokio::test]
    #[ignore]
    async fn test_index_conversation_file() {
        let (indexer, dir) = create_test_indexer().await;

        // Create a test conversation file
        let project_dir = dir.path().join("conversations/-home-user-git-myproject");
        std::fs::create_dir_all(&project_dir).unwrap();

        let test_file = project_dir.join("session123.jsonl");
        let content = r#"{"type":"user","message":{"role":"user","content":"Hello"},"timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","message":{"role":"assistant","content":"Hi there!"},"timestamp":"2026-01-01T00:00:01Z"}"#;
        std::fs::write(&test_file, content).unwrap();

        // Index it
        let indexed = indexer
            .index_conversation_file(ConversationSource::Claude, &test_file)
            .await
            .unwrap();
        assert!(indexed);

        // Check database
        let stats = indexer.db.get_stats().unwrap();
        assert_eq!(stats.conversation_chunks, 2);
        assert_eq!(stats.conversation_sessions, 1);
    }

    #[test]
    fn test_extract_project_name() {
        // Test the project name extraction logic directly
        let parts: Vec<&str> = "-home-schalk-git-twolebot-data".split('-').collect();
        let git_idx = parts.iter().position(|&s| s == "git");
        assert_eq!(git_idx, Some(3));
        if let Some(idx) = git_idx {
            assert_eq!(parts[idx + 1], "twolebot");
        }
    }

    #[test]
    fn test_parse_conversation_messages() {
        let content = r#"{"type":"user","message":{"role":"user","content":"Question"},"timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Answer"}]},"timestamp":"2026-01-01T00:00:01Z"}
{"type":"summary","message":"ignored"}
{"type":"user","message":{"role":"user","content":""},"timestamp":"2026-01-01T00:00:02Z"}"#;

        // Manual parsing for test
        let messages: Vec<_> = content
            .lines()
            .filter_map(|line| {
                let json: serde_json::Value = serde_json::from_str(line).ok()?;
                let msg_type = json.get("type")?.as_str()?;
                if msg_type != "user" && msg_type != "assistant" {
                    return None;
                }
                let message = json.get("message")?;
                let content_val = message.get("content")?;
                let content = match content_val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => arr
                        .iter()
                        .filter_map(|item| {
                            item.get("text")
                                .and_then(|t| t.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                    _ => return None,
                };
                if content.is_empty() {
                    return None;
                }
                Some(content)
            })
            .collect();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "Question");
        assert_eq!(messages[1], "Answer");
    }
}
