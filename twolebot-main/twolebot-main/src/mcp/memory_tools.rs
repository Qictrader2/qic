use crate::semantic::{Embedder, HybridSearcher, VectorDb};
use crate::types::mcp::memory::{
    MemoryReadRequest, MemoryReadResult, MemorySearchRequest, MemorySearchResult,
    MemoryWriteRequest, WriteMode,
};
use regex::Regex;
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

fn json_result(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Normalize a path lexically (without filesystem access)
/// Removes `.` and resolves `..` where possible
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(component),
        }
    }
    result
}

/// MCP tool handler for memory file management
#[derive(Clone)]
pub struct MemoryTools {
    memory_dir: PathBuf,
    vector_db: Option<Arc<VectorDb>>,
    embedder: Option<Arc<Embedder>>,
    tool_router: ToolRouter<Self>,
}

impl MemoryTools {
    pub fn new(
        memory_dir: PathBuf,
        vector_db: Option<Arc<VectorDb>>,
        embedder: Option<Arc<Embedder>>,
    ) -> Self {
        Self {
            memory_dir,
            vector_db,
            embedder,
            tool_router: Self::create_tool_router(),
        }
    }

    pub fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    fn create_tool_router() -> ToolRouter<Self> {
        Self::tool_router()
    }

    /// Validate that a path is safe (no .. traversal)
    fn validate_path(&self, path: &str) -> Result<PathBuf, McpError> {
        // Reject any path with .. for security
        if path.contains("..") {
            tracing::warn!("Rejected path with '..': {}", path);
            return Err(McpError::invalid_params(
                "Path cannot contain '..' for security reasons",
                None,
            ));
        }

        // Reject absolute paths
        if path.starts_with('/') || path.starts_with('\\') {
            tracing::warn!("Rejected absolute path: {}", path);
            return Err(McpError::invalid_params(
                "Path must be relative to memory directory",
                None,
            ));
        }

        let full_path = self.memory_dir.join(path);

        // Normalize and verify the path stays within memory_dir
        // Use lexical normalization (no filesystem access) for non-existent paths
        let normalized = normalize_path(&full_path);
        let normalized_memory = normalize_path(&self.memory_dir);

        if !normalized.starts_with(&normalized_memory) {
            tracing::warn!(
                "Path escapes memory directory: {} -> {}",
                path,
                normalized.display()
            );
            return Err(McpError::invalid_params(
                "Path must be within memory directory",
                None,
            ));
        }

        // If the file exists, also verify with canonicalization (follows symlinks)
        if full_path.exists() {
            if let (Ok(canonical_full), Ok(canonical_memory)) =
                (full_path.canonicalize(), self.memory_dir.canonicalize())
            {
                if !canonical_full.starts_with(&canonical_memory) {
                    tracing::warn!(
                        "Symlink escapes memory directory: {} -> {}",
                        path,
                        canonical_full.display()
                    );
                    return Err(McpError::invalid_params(
                        "Path must be within memory directory (symlink escape detected)",
                        None,
                    ));
                }
            }
        }

        Ok(full_path)
    }

    /// Get snippet with context lines around a match
    fn get_snippet(content: &str, match_start: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();

        // Find the line containing the match
        let mut current_pos = 0;
        let mut match_line_idx = 0;
        for (idx, line) in lines.iter().enumerate() {
            let line_end = current_pos + line.len() + 1; // +1 for newline
            if match_start < line_end {
                match_line_idx = idx;
                break;
            }
            current_pos = line_end;
        }

        // Calculate context range
        let start_line = match_line_idx.saturating_sub(context_lines);
        let end_line = (match_line_idx + context_lines + 1).min(lines.len());

        lines[start_line..end_line].join("\n")
    }
}

fn parse_write_mode(s: Option<&str>) -> Result<WriteMode, McpError> {
    match s {
        Some("append") => Ok(WriteMode::Append),
        Some("replace") | None => Ok(WriteMode::Replace),
        Some(other) => Err(McpError::invalid_params(
            format!(
                "Invalid write mode '{}', must be 'replace' or 'append'",
                other
            ),
            None,
        )),
    }
}

#[tool_router]
impl MemoryTools {
    #[tool(
        name = "memory_search",
        description = "Search for text across all markdown files in the memory directory. Returns matching files with snippets containing 2 lines of context. Results are sorted by modification time (newest first)."
    )]
    async fn memory_search(
        &self,
        request: Parameters<MemorySearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;
        let limit = request.limit.unwrap_or(10);
        let offset = request.offset.unwrap_or(0);

        tracing::info!(query = %request.query, limit = limit, offset = offset, "Memory search started");

        // Compile regex (may fail for semantic-only queries, that's OK)
        let regex = Regex::new(&request.query).map_err(|e| {
            tracing::warn!(error = %e, "Invalid regex pattern");
            McpError::invalid_params(format!("Invalid regex pattern: {}", e), None)
        })?;

        // Ensure memory directory exists
        if !self.memory_dir.exists() {
            tracing::debug!(dir = ?self.memory_dir, "Creating memory directory");
            std::fs::create_dir_all(&self.memory_dir).map_err(|e| {
                tracing::error!(error = %e, "Failed to create memory directory");
                McpError::internal_error(format!("Failed to create memory directory: {}", e), None)
            })?;
        }

        // --- Semantic search (if available) ---
        let semantic_results = if let (Some(db), Some(embedder)) =
            (&self.vector_db, &self.embedder)
        {
            match embedder.embed_one(&request.query) {
                Ok(query_embedding) => {
                    // Fetch more than limit to give RRF good ranking material
                    match db.search_memory_semantic(&query_embedding, limit * 3) {
                        Ok(results) => {
                            tracing::debug!(
                                count = results.len(),
                                "Semantic memory search returned results"
                            );
                            HybridSearcher::from_semantic_memory(results)
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Semantic memory search failed, falling back to keyword");
                            Vec::new()
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to embed query, falling back to keyword");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // --- Keyword search (always runs) ---
        let mut keyword_results = Vec::new();
        let mut files_with_mtime: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        for entry in WalkDir::new(&self.memory_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let mtime = match std::fs::metadata(path).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(_) => continue,
            };
            files_with_mtime.push((path.to_path_buf(), mtime));
        }

        files_with_mtime.sort_by(|a, b| b.1.cmp(&a.1));

        for (path, _mtime) in &files_with_mtime {
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let matches: Vec<_> = regex.find_iter(&content).collect();
            if matches.is_empty() {
                continue;
            }
            let relative_path = path
                .strip_prefix(&self.memory_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            let snippet = Self::get_snippet(&content, matches[0].start(), 2);

            keyword_results.push((
                HybridSearcher::keyword_memory_result(
                    relative_path.clone(),
                    0,
                    snippet.clone(),
                    matches.len(),
                ),
                MemorySearchResult {
                    path: relative_path,
                    snippet,
                    match_count: matches.len(),
                },
            ));
        }

        // --- Fuse or fall back ---
        let results: Vec<MemorySearchResult> = if !semantic_results.is_empty() {
            let searcher = HybridSearcher::new();
            let keyword_search_results: Vec<_> =
                keyword_results.iter().map(|(sr, _)| sr.clone()).collect();
            let fused = searcher.fuse(semantic_results, keyword_search_results, limit + offset);

            // Convert fused results back to MemorySearchResult
            // Build a lookup from keyword results for snippets
            let keyword_map: std::collections::HashMap<String, MemorySearchResult> =
                keyword_results
                    .into_iter()
                    .map(|(sr, mr)| (sr.id, mr))
                    .collect();

            fused
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|sr| {
                    // If this result came from keyword search, use its snippet
                    if let Some(mr) = keyword_map.get(&sr.id) {
                        mr.clone()
                    } else {
                        // Semantic-only result: use the chunk text as snippet
                        MemorySearchResult {
                            path: sr.source,
                            snippet: sr.text,
                            match_count: 0, // No keyword matches
                        }
                    }
                })
                .collect()
        } else {
            // Pure keyword fallback
            keyword_results
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|(_, mr)| mr)
                .collect()
        };

        tracing::info!(
            results = results.len(),
            semantic = self.vector_db.is_some(),
            "Memory search completed"
        );
        json_result(&results)
    }

    #[tool(
        name = "memory_read",
        description = "Read a specific memory file. Path is relative to the memory directory."
    )]
    async fn memory_read(
        &self,
        request: Parameters<MemoryReadRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;

        tracing::info!(path = %request.path, "Memory read started");

        // Validate path
        let full_path = self.validate_path(&request.path)?;

        // Check file exists
        if !full_path.exists() {
            tracing::warn!(path = %request.path, "File not found");
            return Err(McpError::invalid_params(
                format!("File not found: {}", request.path),
                None,
            ));
        }

        if !full_path.is_file() {
            tracing::warn!(path = %request.path, "Path is not a file");
            return Err(McpError::invalid_params(
                format!("Path is not a file: {}", request.path),
                None,
            ));
        }

        // Read content
        let content = std::fs::read_to_string(&full_path).map_err(|e| {
            tracing::error!(path = %request.path, error = %e, "Failed to read file");
            McpError::internal_error(format!("Failed to read file: {}", e), None)
        })?;

        // Check for frontmatter (starts with ---)
        let has_frontmatter = content.starts_with("---");
        let content_len = content.len();

        let result = MemoryReadResult {
            content,
            has_frontmatter,
        };

        tracing::info!(path = %request.path, bytes = content_len, has_frontmatter = has_frontmatter, "Memory read completed");
        json_result(&result)
    }

    #[tool(
        name = "memory_write",
        description = "Write or append to a memory file. Path must be relative to the memory directory and end in .md. Creates parent directories as needed."
    )]
    async fn memory_write(
        &self,
        request: Parameters<MemoryWriteRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;

        tracing::info!(path = %request.path, mode = ?request.mode, "Memory write started");

        // Validate .md extension
        if !request.path.ends_with(".md") {
            tracing::warn!(path = %request.path, "Rejected path without .md extension");
            return Err(McpError::invalid_params(
                "Path must end with .md extension",
                None,
            ));
        }

        // Validate path
        let full_path = self.validate_path(&request.path)?;

        // Parse write mode
        let mode = parse_write_mode(request.mode.as_deref())?;

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                tracing::debug!(parent = ?parent, "Creating parent directories");
            }
            std::fs::create_dir_all(parent).map_err(|e| {
                tracing::error!(parent = ?parent, error = %e, "Failed to create directories");
                McpError::internal_error(format!("Failed to create directories: {}", e), None)
            })?;
        }

        // Write content based on mode
        let content_len = request.content.len();
        let path_str = request.path.clone();

        match mode {
            WriteMode::Replace => {
                std::fs::write(&full_path, &request.content).map_err(|e| {
                    tracing::error!(path = %path_str, error = %e, "Failed to write file");
                    McpError::internal_error(format!("Failed to write file: {}", e), None)
                })?;
            }
            WriteMode::Append => {
                // Read existing content if file exists
                let existing = if full_path.exists() {
                    std::fs::read_to_string(&full_path).unwrap_or_default()
                } else {
                    String::new()
                };

                // Append with separator
                let new_content = if existing.is_empty() {
                    request.content
                } else {
                    format!("{}\n\n{}", existing, request.content)
                };

                std::fs::write(&full_path, &new_content).map_err(|e| {
                    tracing::error!(path = %path_str, error = %e, "Failed to append to file");
                    McpError::internal_error(format!("Failed to write file: {}", e), None)
                })?;
            }
        }

        let message = match mode {
            WriteMode::Replace => {
                tracing::info!(path = %path_str, bytes = content_len, "Memory write completed (replace)");
                format!("Wrote {} bytes to {}", content_len, path_str)
            }
            WriteMode::Append => {
                tracing::info!(path = %path_str, bytes = content_len, "Memory write completed (append)");
                format!("Appended {} bytes to {}", content_len, path_str)
            }
        };

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_tools() -> (MemoryTools, TempDir) {
        let dir = TempDir::new().unwrap();
        (MemoryTools::new(dir.path().to_path_buf(), None, None), dir)
    }

    #[tokio::test]
    async fn test_memory_write_and_read() {
        let (tools, _tmpdir) = create_test_tools();

        // Write a file
        let write_request = Parameters(MemoryWriteRequest {
            path: "test.md".to_string(),
            content: "# Test\n\nHello world".to_string(),
            mode: None,
        });
        let result = tools.memory_write(write_request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Read it back
        let read_request = Parameters(MemoryReadRequest {
            path: "test.md".to_string(),
        });
        let result = tools.memory_read(read_request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify content
        let response: MemoryReadResult =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert_eq!(response.content, "# Test\n\nHello world");
        assert!(!response.has_frontmatter);
    }

    #[tokio::test]
    async fn test_memory_write_with_frontmatter() {
        let (tools, _tmpdir) = create_test_tools();

        let content = "---\ntags: [test]\n---\n\n# Test";
        let write_request = Parameters(MemoryWriteRequest {
            path: "frontmatter.md".to_string(),
            content: content.to_string(),
            mode: None,
        });
        tools.memory_write(write_request).await.unwrap();

        let read_request = Parameters(MemoryReadRequest {
            path: "frontmatter.md".to_string(),
        });
        let result = tools.memory_read(read_request).await.unwrap();

        let response: MemoryReadResult =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert!(response.has_frontmatter);
    }

    #[tokio::test]
    async fn test_memory_write_append() {
        let (tools, _tmpdir) = create_test_tools();

        // Write initial content
        let write1 = Parameters(MemoryWriteRequest {
            path: "append.md".to_string(),
            content: "First".to_string(),
            mode: None,
        });
        tools.memory_write(write1).await.unwrap();

        // Append more content
        let write2 = Parameters(MemoryWriteRequest {
            path: "append.md".to_string(),
            content: "Second".to_string(),
            mode: Some("append".to_string()),
        });
        tools.memory_write(write2).await.unwrap();

        // Read and verify
        let read_request = Parameters(MemoryReadRequest {
            path: "append.md".to_string(),
        });
        let result = tools.memory_read(read_request).await.unwrap();

        let response: MemoryReadResult =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert_eq!(response.content, "First\n\nSecond");
    }

    #[tokio::test]
    async fn test_memory_write_creates_directories() {
        let (tools, _tmpdir) = create_test_tools();

        let write_request = Parameters(MemoryWriteRequest {
            path: "nested/dir/file.md".to_string(),
            content: "Nested content".to_string(),
            mode: None,
        });
        let result = tools.memory_write(write_request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify we can read it
        let read_request = Parameters(MemoryReadRequest {
            path: "nested/dir/file.md".to_string(),
        });
        let result = tools.memory_read(read_request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_memory_search() {
        let (tools, _tmpdir) = create_test_tools();

        // Write some files
        tools
            .memory_write(Parameters(MemoryWriteRequest {
                path: "file1.md".to_string(),
                content: "Hello world from file1".to_string(),
                mode: None,
            }))
            .await
            .unwrap();

        tools
            .memory_write(Parameters(MemoryWriteRequest {
                path: "file2.md".to_string(),
                content: "Goodbye world from file2".to_string(),
                mode: None,
            }))
            .await
            .unwrap();

        tools
            .memory_write(Parameters(MemoryWriteRequest {
                path: "file3.md".to_string(),
                content: "No match here".to_string(),
                mode: None,
            }))
            .await
            .unwrap();

        // Search for "world"
        let search_request = Parameters(MemorySearchRequest {
            query: "world".to_string(),
            limit: None,
            offset: None,
        });
        let result = tools.memory_search(search_request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        let results: Vec<MemorySearchResult> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_search_regex() {
        let (tools, _tmpdir) = create_test_tools();

        tools
            .memory_write(Parameters(MemoryWriteRequest {
                path: "regex.md".to_string(),
                content: "Line 1\nLine 2 with pattern123\nLine 3".to_string(),
                mode: None,
            }))
            .await
            .unwrap();

        let search_request = Parameters(MemorySearchRequest {
            query: r"pattern\d+".to_string(),
            limit: None,
            offset: None,
        });
        let result = tools.memory_search(search_request).await.unwrap();

        let results: Vec<MemorySearchResult> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].snippet.contains("pattern123"));
    }

    #[tokio::test]
    async fn test_path_traversal_rejected() {
        let (tools, _tmpdir) = create_test_tools();

        let write_request = Parameters(MemoryWriteRequest {
            path: "../escape.md".to_string(),
            content: "Bad content".to_string(),
            mode: None,
        });
        let result = tools.memory_write(write_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_non_md_extension_rejected() {
        let (tools, _tmpdir) = create_test_tools();

        let write_request = Parameters(MemoryWriteRequest {
            path: "test.txt".to_string(),
            content: "Content".to_string(),
            mode: None,
        });
        let result = tools.memory_write(write_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let (tools, _tmpdir) = create_test_tools();

        let read_request = Parameters(MemoryReadRequest {
            path: "nonexistent.md".to_string(),
        });
        let result = tools.memory_read(read_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_write_mode() {
        let (tools, _tmpdir) = create_test_tools();

        let write_request = Parameters(MemoryWriteRequest {
            path: "test.md".to_string(),
            content: "Content".to_string(),
            mode: Some("invalid".to_string()),
        });
        let result = tools.memory_write(write_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_limit() {
        let (tools, _tmpdir) = create_test_tools();

        // Write many files
        for i in 0..20 {
            tools
                .memory_write(Parameters(MemoryWriteRequest {
                    path: format!("file{}.md", i),
                    content: format!("Match content {}", i),
                    mode: None,
                }))
                .await
                .unwrap();
        }

        // Search with limit
        let search_request = Parameters(MemorySearchRequest {
            query: "Match".to_string(),
            limit: Some(5),
            offset: None,
        });
        let result = tools.memory_search(search_request).await.unwrap();

        let results: Vec<MemorySearchResult> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_safe_filename() -> impl Strategy<Value = String> {
            prop::string::string_regex("[a-zA-Z0-9_-]{1,20}")
                .unwrap()
                .prop_map(|s| format!("{}.md", s))
        }

        fn arb_content() -> impl Strategy<Value = String> {
            prop::string::string_regex(".{0,500}").unwrap()
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(20))]

            #[test]
            fn prop_write_read_roundtrip(filename in arb_safe_filename(), content in arb_content()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let dir = TempDir::new().unwrap();
                    let tools = MemoryTools::new(dir.path().to_path_buf(), None, None);

                    // Write
                    let write_req = Parameters(MemoryWriteRequest {
                        path: filename.clone(),
                        content: content.clone(),
                        mode: None,
                    });
                    let write_result = tools.memory_write(write_req).await.unwrap();
                    assert!(!write_result.is_error.unwrap_or(false));

                    // Read
                    let read_req = Parameters(MemoryReadRequest {
                        path: filename,
                    });
                    let read_result = tools.memory_read(read_req).await.unwrap();
                    assert!(!read_result.is_error.unwrap_or(false));

                    let response: MemoryReadResult = serde_json::from_str(
                        &read_result.content[0].as_text().unwrap().text
                    ).unwrap();
                    assert_eq!(response.content, content);
                });
            }

            #[test]
            fn prop_append_grows_content(filename in arb_safe_filename(), content1 in arb_content(), content2 in arb_content()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let dir = TempDir::new().unwrap();
                    let tools = MemoryTools::new(dir.path().to_path_buf(), None, None);

                    // Write first
                    tools.memory_write(Parameters(MemoryWriteRequest {
                        path: filename.clone(),
                        content: content1.clone(),
                        mode: None,
                    })).await.unwrap();

                    // Append second
                    tools.memory_write(Parameters(MemoryWriteRequest {
                        path: filename.clone(),
                        content: content2.clone(),
                        mode: Some("append".to_string()),
                    })).await.unwrap();

                    // Read and verify it contains both
                    let read_result = tools.memory_read(Parameters(MemoryReadRequest {
                        path: filename,
                    })).await.unwrap();

                    let response: MemoryReadResult = serde_json::from_str(
                        &read_result.content[0].as_text().unwrap().text
                    ).unwrap();
                    assert!(response.content.contains(&content1));
                    assert!(response.content.contains(&content2));
                });
            }
        }
    }

    /// Integration tests with the real embedding model.
    /// These require downloading the ~100MB BGE-small-en-v1.5 model.
    /// Run with: cargo test --ignored
    #[cfg(test)]
    mod semantic_tests {
        use super::*;
        use crate::semantic::VectorDb;

        /// Create MemoryTools backed by a real VectorDb + Embedder.
        /// Writes memory files AND indexes them into the vector DB.
        async fn create_semantic_tools() -> (MemoryTools, tempfile::TempDir) {
            let dir = tempfile::TempDir::new().unwrap();
            let db_path = dir.path().join("vectors.sqlite3");
            let db = Arc::new(VectorDb::open(&db_path).unwrap());
            let embedder = Embedder::global(2).await.unwrap();

            let tools = MemoryTools::new(
                dir.path().join("memory"),
                Some(db.clone()),
                Some(embedder.clone()),
            );

            // Create memory directory
            std::fs::create_dir_all(dir.path().join("memory")).unwrap();

            (tools, dir)
        }

        /// Write a memory file AND index it into the vector DB.
        async fn write_and_index(
            tools: &MemoryTools,
            db: &VectorDb,
            embedder: &Embedder,
            path: &str,
            content: &str,
        ) {
            // Write the file via MCP tool
            tools
                .memory_write(Parameters(MemoryWriteRequest {
                    path: path.to_string(),
                    content: content.to_string(),
                    mode: None,
                }))
                .await
                .unwrap();

            // Index into vector DB (normally the SemanticIndexer does this)
            let embedding = embedder.embed_one(content).unwrap();
            let hash = crate::semantic::hash_content(content);
            db.insert_memory_chunk(path, 0, content, &hash, &embedding)
                .unwrap();
        }

        #[tokio::test]
        #[ignore]
        async fn test_semantic_search_finds_conceptually_similar_no_keyword_overlap() {
            // This test proves semantic search works: we search for a concept
            // using words that have ZERO overlap with the indexed document.
            let (tools, _dir) = create_semantic_tools().await;
            let db = tools.vector_db.as_ref().unwrap();
            let embedder = tools.embedder.as_ref().unwrap();

            // Index documents about distinct topics
            write_and_index(
                &tools, db, embedder,
                "cooking.md",
                "The best way to prepare Italian pasta is to use fresh ingredients \
                 and cook the noodles al dente in salted boiling water.",
            ).await;

            write_and_index(
                &tools, db, embedder,
                "astronomy.md",
                "The Hubble Space Telescope has captured stunning images of distant \
                 galaxies and nebulae in deep space observations.",
            ).await;

            write_and_index(
                &tools, db, embedder,
                "programming.md",
                "Rust ownership model prevents data races at compile time through \
                 its borrow checker and lifetime annotations.",
            ).await;

            // Search for "making food at home" — conceptually close to cooking.md
            // but shares NO keywords with it (no "pasta", "Italian", "noodles", etc.)
            let request = Parameters(MemorySearchRequest {
                query: "making food at home".to_string(),
                limit: Some(10),
                offset: None,
            });
            let result = tools.memory_search(request).await.unwrap();
            let results: Vec<MemorySearchResult> =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

            // Should find cooking.md via semantic similarity even with zero keyword match
            assert!(
                !results.is_empty(),
                "Semantic search should return results even with no keyword overlap"
            );
            assert_eq!(
                results[0].path, "cooking.md",
                "Cooking document should rank first for food-related query"
            );
        }

        #[tokio::test]
        #[ignore]
        async fn test_semantic_search_distinguishes_unrelated_topics() {
            let (tools, _dir) = create_semantic_tools().await;
            let db = tools.vector_db.as_ref().unwrap();
            let embedder = tools.embedder.as_ref().unwrap();

            write_and_index(
                &tools, db, embedder,
                "machine-learning.md",
                "Neural networks use gradient descent and backpropagation to learn \
                 patterns from training data in supervised learning tasks.",
            ).await;

            write_and_index(
                &tools, db, embedder,
                "gardening.md",
                "Tomatoes grow best in well-drained soil with full sunlight exposure \
                 and regular watering during the growing season.",
            ).await;

            // Search for "artificial intelligence algorithms"
            let request = Parameters(MemorySearchRequest {
                query: "artificial intelligence algorithms".to_string(),
                limit: Some(10),
                offset: None,
            });
            let result = tools.memory_search(request).await.unwrap();
            let results: Vec<MemorySearchResult> =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

            assert!(!results.is_empty());
            // ML doc should rank higher than gardening for an AI query
            let ml_pos = results.iter().position(|r| r.path == "machine-learning.md");
            let garden_pos = results.iter().position(|r| r.path == "gardening.md");
            assert!(
                ml_pos.unwrap() < garden_pos.unwrap(),
                "ML document should rank above gardening for AI query"
            );
        }

        #[tokio::test]
        #[ignore]
        async fn test_hybrid_search_boosts_results_in_both_semantic_and_keyword() {
            let (tools, _dir) = create_semantic_tools().await;
            let db = tools.vector_db.as_ref().unwrap();
            let embedder = tools.embedder.as_ref().unwrap();

            write_and_index(
                &tools, db, embedder,
                "rust-guide.md",
                "Rust programming language provides memory safety without garbage \
                 collection through its ownership system.",
            ).await;

            write_and_index(
                &tools, db, embedder,
                "go-guide.md",
                "Go programming language uses goroutines for concurrent execution \
                 and channels for communication between them.",
            ).await;

            write_and_index(
                &tools, db, embedder,
                "cooking-rust.md",
                "Removing rust from cast iron pans requires vinegar and scrubbing.",
            ).await;

            // Search for "Rust" — keyword matches rust-guide.md AND cooking-rust.md,
            // but semantic should also boost rust-guide.md (it's about programming).
            // Hybrid search (RRF) should rank rust-guide.md first.
            let request = Parameters(MemorySearchRequest {
                query: "Rust".to_string(),
                limit: Some(10),
                offset: None,
            });
            let result = tools.memory_search(request).await.unwrap();
            let results: Vec<MemorySearchResult> =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

            assert!(results.len() >= 2, "Should find at least 2 results");
            // Both rust-guide.md and cooking-rust.md should be present
            let paths: Vec<&str> = results.iter().map(|r| r.path.as_str()).collect();
            assert!(paths.contains(&"rust-guide.md"));
            assert!(paths.contains(&"cooking-rust.md"));
        }
    }
}
