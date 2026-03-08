//! Hybrid search combining semantic and keyword search.
//!
//! Uses Reciprocal Rank Fusion (RRF) to combine results from both search methods.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Search mode for hybrid search.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// Hybrid search using RRF (default)
    #[default]
    Auto,
    /// Semantic similarity only
    Semantic,
    /// Keyword/regex only
    Keyword,
}

/// A unified search result that can come from either search method.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Unique identifier for deduplication
    pub id: String,
    /// The path (for memory) or session_id (for conversations)
    pub source: String,
    /// Index within the source (chunk_index or message_index)
    pub index: usize,
    /// The matching text content
    pub text: String,
    /// Combined relevance score (higher = more relevant)
    pub score: f32,
    /// Additional metadata
    pub metadata: SearchMetadata,
}

/// Additional metadata for search results.
#[derive(Debug, Clone, Default)]
pub struct SearchMetadata {
    /// For conversations: the project name
    pub project: Option<String>,
    /// For conversations: the role (user/assistant)
    pub role: Option<String>,
    /// For conversations: the timestamp
    pub timestamp: Option<String>,
    /// Semantic distance (if from semantic search)
    pub semantic_distance: Option<f32>,
    /// Keyword match count (if from keyword search)
    pub keyword_matches: Option<usize>,
}

/// Hybrid searcher that combines semantic and keyword results.
pub struct HybridSearcher {
    /// RRF constant (typically 60)
    k: f32,
}

impl Default for HybridSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridSearcher {
    /// Create a new hybrid searcher with default RRF constant.
    pub fn new() -> Self {
        Self { k: 60.0 }
    }

    /// Create a new hybrid searcher with custom RRF constant.
    pub fn with_k(k: f32) -> Self {
        Self { k }
    }

    /// Combine semantic and keyword search results using Reciprocal Rank Fusion.
    ///
    /// RRF formula: score(d) = Σ 1/(k + rank_i(d))
    /// where k is a constant (typically 60) and rank_i is the rank in result set i.
    pub fn fuse(
        &self,
        semantic_results: Vec<SearchResult>,
        keyword_results: Vec<SearchResult>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut results_map: HashMap<String, SearchResult> = HashMap::new();

        // Score semantic results
        for (rank, result) in semantic_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (self.k + rank as f32);
            *scores.entry(result.id.clone()).or_default() += rrf_score;
            results_map.entry(result.id.clone()).or_insert(result);
        }

        // Score keyword results
        for (rank, result) in keyword_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (self.k + rank as f32);
            *scores.entry(result.id.clone()).or_default() += rrf_score;
            results_map.entry(result.id.clone()).or_insert(result);
        }

        // Combine and sort by score
        let mut combined: Vec<SearchResult> = results_map
            .into_iter()
            .map(|(id, mut result)| {
                result.score = scores.get(&id).copied().unwrap_or(0.0);
                result
            })
            .collect();

        combined.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combined.truncate(limit);

        combined
    }

    /// Convert semantic search results to unified format.
    pub fn from_semantic_memory(
        results: Vec<super::vectordb::VectorSearchResult>,
    ) -> Vec<SearchResult> {
        results
            .into_iter()
            .map(|r| SearchResult {
                id: format!("memory:{}:{}", r.path, r.chunk_index),
                source: r.path,
                index: r.chunk_index as usize,
                text: r.chunk_text,
                score: 1.0 - r.distance, // Convert distance to similarity
                metadata: SearchMetadata {
                    semantic_distance: Some(r.distance),
                    ..Default::default()
                },
            })
            .collect()
    }

    /// Convert semantic conversation results to unified format.
    pub fn from_semantic_conversation(
        results: Vec<super::vectordb::ConversationSearchResult>,
    ) -> Vec<SearchResult> {
        results
            .into_iter()
            .map(|r| SearchResult {
                id: format!(
                    "conv:{}:{}:{}",
                    r.session_id, r.message_index, r.chunk_index
                ),
                source: r.session_id,
                index: r.message_index as usize,
                text: r.chunk_text,
                score: 1.0 - r.distance,
                metadata: SearchMetadata {
                    project: Some(r.project),
                    role: Some(r.role),
                    timestamp: Some(r.timestamp),
                    semantic_distance: Some(r.distance),
                    ..Default::default()
                },
            })
            .collect()
    }

    /// Create a search result from keyword match (memory).
    pub fn keyword_memory_result(
        path: String,
        chunk_index: usize,
        text: String,
        match_count: usize,
    ) -> SearchResult {
        SearchResult {
            id: format!("memory:{}:{}", path, chunk_index),
            source: path,
            index: chunk_index,
            text,
            score: match_count as f32, // Will be normalized by RRF
            metadata: SearchMetadata {
                keyword_matches: Some(match_count),
                ..Default::default()
            },
        }
    }

    /// Create a search result from keyword match (conversation).
    pub fn keyword_conversation_result(
        session_id: String,
        message_index: usize,
        text: String,
        match_count: usize,
        project: String,
        role: String,
        timestamp: String,
    ) -> SearchResult {
        SearchResult {
            id: format!("conv:{}:{}:0", session_id, message_index),
            source: session_id,
            index: message_index,
            text,
            score: match_count as f32,
            metadata: SearchMetadata {
                project: Some(project),
                role: Some(role),
                timestamp: Some(timestamp),
                keyword_matches: Some(match_count),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(id: &str, source: &str, index: usize, score: f32) -> SearchResult {
        SearchResult {
            id: id.to_string(),
            source: source.to_string(),
            index,
            text: format!("Text for {}", id),
            score,
            metadata: SearchMetadata::default(),
        }
    }

    #[test]
    fn test_rrf_fusion_basic() {
        let searcher = HybridSearcher::new();

        let semantic = vec![
            make_result("a", "file_a", 0, 0.9),
            make_result("b", "file_b", 0, 0.8),
            make_result("c", "file_c", 0, 0.7),
        ];

        let keyword = vec![
            make_result("b", "file_b", 0, 5.0), // b is also #1 in keyword
            make_result("d", "file_d", 0, 3.0),
            make_result("a", "file_a", 0, 1.0), // a is #3 in keyword
        ];

        let fused = searcher.fuse(semantic, keyword, 10);

        // b should be ranked highest (top in keyword, #2 in semantic)
        assert_eq!(fused[0].id, "b");

        // a should be second (top in semantic, #3 in keyword)
        assert_eq!(fused[1].id, "a");

        // All 4 unique items should be present
        assert_eq!(fused.len(), 4);
    }

    #[test]
    fn test_rrf_fusion_no_overlap() {
        let searcher = HybridSearcher::new();

        let semantic = vec![make_result("a", "file_a", 0, 0.9)];
        let keyword = vec![make_result("b", "file_b", 0, 5.0)];

        let fused = searcher.fuse(semantic, keyword, 10);

        assert_eq!(fused.len(), 2);
        // Both should have equal RRF score (both rank 0 in their lists)
        assert!((fused[0].score - fused[1].score).abs() < 0.001);
    }

    #[test]
    fn test_rrf_fusion_limit() {
        let searcher = HybridSearcher::new();

        let semantic = vec![
            make_result("a", "f", 0, 0.9),
            make_result("b", "f", 1, 0.8),
            make_result("c", "f", 2, 0.7),
        ];

        let keyword = vec![make_result("d", "f", 3, 5.0), make_result("e", "f", 4, 4.0)];

        let fused = searcher.fuse(semantic, keyword, 2);
        assert_eq!(fused.len(), 2);
    }

    #[test]
    fn test_rrf_fusion_empty_inputs() {
        let searcher = HybridSearcher::new();

        let fused = searcher.fuse(Vec::new(), Vec::new(), 10);
        assert!(fused.is_empty());

        let semantic = vec![make_result("a", "f", 0, 0.9)];
        let fused = searcher.fuse(semantic, Vec::new(), 10);
        assert_eq!(fused.len(), 1);
    }

    #[test]
    fn test_search_mode_default() {
        let mode = SearchMode::default();
        assert_eq!(mode, SearchMode::Auto);
    }

    #[test]
    fn test_search_mode_serde() {
        let json = serde_json::to_string(&SearchMode::Semantic).unwrap();
        assert_eq!(json, "\"semantic\"");

        let parsed: SearchMode = serde_json::from_str("\"keyword\"").unwrap();
        assert_eq!(parsed, SearchMode::Keyword);
    }
}
