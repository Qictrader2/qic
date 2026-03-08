//! Embedding service using fastembed.
//!
//! Wraps fastembed's TextEmbedding to provide a simple interface for generating
//! text embeddings using the BGE-small-en model.
//!
//! Resource limits: set OMP_NUM_THREADS before process start to control ONNX threads.

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Embedding dimension for BGE-small-en-v1.5
#[allow(dead_code)]
pub const EMBEDDING_DIM: usize = 384;

/// Default OMP thread count used for ONNX inference.
const DEFAULT_EMBEDDING_THREADS: u16 = 2;

/// Singleton embedder instance (model is ~100MB, only load once)
static EMBEDDER: OnceCell<Arc<Embedder>> = OnceCell::const_new();

/// Wrapper around fastembed's TextEmbedding.
pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    /// Get or initialize the global embedder instance.
    ///
    /// The model is loaded lazily on first call and cached for subsequent use.
    /// This avoids loading the ~100MB model multiple times.
    pub async fn global(omp_num_threads: u16) -> Result<Arc<Embedder>> {
        EMBEDDER
            .get_or_try_init(|| async {
                tracing::info!("Initializing embedding model (BGE-small-en-v1.5)...");
                let embedder = Self::new(omp_num_threads).context("Failed to initialize embedder")?;
                tracing::info!("Embedding model initialized");
                Ok(Arc::new(embedder))
            })
            .await
            .cloned()
    }

    /// Create a new embedder instance.
    ///
    /// Prefer using `global()` to avoid loading the model multiple times.
    fn new(omp_num_threads: u16) -> Result<Self> {
        let threads = if omp_num_threads == 0 {
            DEFAULT_EMBEDDING_THREADS
        } else {
            omp_num_threads
        };
        std::env::set_var("OMP_NUM_THREADS", threads.to_string());
        tracing::info!("Semantic embedder thread limit set to {}", threads);

        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true),
        )
        .context("Failed to load embedding model")?;

        Ok(Self { model })
    }

    /// Generate embeddings for a batch of texts.
    ///
    /// Returns a vector of embeddings, one per input text.
    /// Each embedding is a Vec<f32> of length EMBEDDING_DIM (384).
    pub fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        self.model
            .embed(texts, None)
            .context("Failed to generate embeddings")
    }

    /// Generate embedding for a single text.
    pub fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed(vec![text.to_string()])?;
        embeddings
            .into_iter()
            .next()
            .context("Expected at least one embedding")
    }
}

// Embedding tests are ignored by default because they require downloading ~100MB model
// and the test binary becomes too large (linker OOM). Run with: cargo test --ignored
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_embedder_initialization() {
        let embedder = Embedder::global(DEFAULT_EMBEDDING_THREADS).await;
        assert!(embedder.is_ok(), "Embedder should initialize successfully");
    }

    #[tokio::test]
    #[ignore]
    async fn test_embed_single_text() {
        let embedder = Embedder::global(DEFAULT_EMBEDDING_THREADS).await.unwrap();
        let embedding = embedder.embed_one("Hello, world!");
        assert!(embedding.is_ok());

        let vec = embedding.unwrap();
        assert_eq!(vec.len(), EMBEDDING_DIM);
    }

    #[tokio::test]
    #[ignore]
    async fn test_embed_batch() {
        let embedder = Embedder::global(DEFAULT_EMBEDDING_THREADS).await.unwrap();
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = embedder.embed(texts);
        assert!(embeddings.is_ok());

        let vecs = embeddings.unwrap();
        assert_eq!(vecs.len(), 3);
        for vec in vecs {
            assert_eq!(vec.len(), EMBEDDING_DIM);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_embed_empty_batch() {
        let embedder = Embedder::global(DEFAULT_EMBEDDING_THREADS).await.unwrap();
        let embeddings = embedder.embed(Vec::new());
        assert!(embeddings.is_ok());
        assert!(embeddings.unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_embedding_determinism() {
        let embedder = Embedder::global(DEFAULT_EMBEDDING_THREADS).await.unwrap();
        let text = "The quick brown fox jumps over the lazy dog";

        let embedding1 = embedder.embed_one(text).unwrap();
        let embedding2 = embedder.embed_one(text).unwrap();

        // Embeddings should be identical for the same input
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_similar_texts_have_similar_embeddings() {
        let embedder = Embedder::global(DEFAULT_EMBEDDING_THREADS).await.unwrap();

        let text1 = "I love programming in Rust";
        let text2 = "I enjoy coding with Rust";
        let text3 = "The weather is nice today";

        let emb1 = embedder.embed_one(text1).unwrap();
        let emb2 = embedder.embed_one(text2).unwrap();
        let emb3 = embedder.embed_one(text3).unwrap();

        // Cosine similarity helper
        fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            dot / (norm_a * norm_b)
        }

        let sim_1_2 = cosine_similarity(&emb1, &emb2);
        let sim_1_3 = cosine_similarity(&emb1, &emb3);

        // Similar texts should have higher similarity
        assert!(
            sim_1_2 > sim_1_3,
            "Similar texts should have higher cosine similarity: {} vs {}",
            sim_1_2,
            sim_1_3
        );
    }
}
