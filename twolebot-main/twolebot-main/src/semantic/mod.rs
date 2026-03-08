//! Semantic search module for memory and conversation indexing.
//!
//! Provides embedding-based similarity search complementing regex/keyword search.
//! Uses fastembed for local embeddings and SQLite with sqlite-vec for vector storage.

mod chunker;
mod embedder;
mod indexer;
mod search;
mod vectordb;

pub use chunker::{Chunk, Chunker};
pub use embedder::Embedder;
pub use indexer::{
    disabled_status, IndexerActivity, IndexerConfig, IndexerStatus, SemanticIndexer, SharedStatus,
    TaskStatus, CODEX_SESSION_PREFIX,
};
pub use search::{HybridSearcher, SearchMode, SearchResult};
pub use vectordb::{hash_content, VectorDb};
