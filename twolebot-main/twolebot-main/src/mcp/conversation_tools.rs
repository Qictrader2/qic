use crate::semantic::{Embedder, VectorDb, CODEX_SESSION_PREFIX};
use crate::types::mcp::conversations::{
    ConversationMatch, ConversationMessage as Message, ConversationSearchRequest,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;

fn json_result(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

#[derive(Clone, Copy)]
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
}

/// MCP tool handler for conversation search
#[derive(Clone)]
pub struct ConversationTools {
    claude_conversations_dir: PathBuf,
    codex_conversations_dir: PathBuf,
    vector_db: Option<Arc<VectorDb>>,
    embedder: Option<Arc<Embedder>>,
    tool_router: ToolRouter<Self>,
}

fn decode_session_key(session_id: &str) -> (ConversationSource, String) {
    if let Some(stripped) = session_id.strip_prefix(CODEX_SESSION_PREFIX) {
        (ConversationSource::Codex, stripped.to_string())
    } else {
        (ConversationSource::Claude, session_id.to_string())
    }
}

impl ConversationTools {
    pub fn new(
        claude_conversations_dir: PathBuf,
        codex_conversations_dir: PathBuf,
        vector_db: Option<Arc<VectorDb>>,
        embedder: Option<Arc<Embedder>>,
    ) -> Self {
        Self {
            claude_conversations_dir,
            codex_conversations_dir,
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
}

/// Extract text content from JSONL message content field
fn extract_text(content: &serde_json::Value) -> String {
    match content {
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
        _ => String::new(),
    }
}

fn extract_project_from_cwd(cwd: &str) -> Option<String> {
    let components: Vec<String> = std::path::Path::new(cwd)
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

/// Calculate recency score based on timestamp
fn recency_score(timestamp: &DateTime<Utc>, weight: f32) -> f32 {
    let age_hours = (Utc::now() - *timestamp).num_hours() as f32;
    let decay = (-age_hours / 168.0).exp(); // 1 week half-life
    decay * weight + (1.0 - weight)
}

/// Extract project name from directory path
fn extract_project_name(path: &str) -> String {
    // Path format: -home-schalk-git-twolebot-data -> twolebot
    // Take the second-to-last segment if it looks like a project
    let parts: Vec<&str> = path.split('-').collect();
    if parts.len() >= 2 {
        // Find the "git" segment and take what follows
        if let Some(git_idx) = parts.iter().position(|&s| s == "git") {
            if git_idx + 1 < parts.len() {
                return parts[git_idx + 1].to_string();
            }
        }
    }
    // Fallback: use the whole path
    path.to_string()
}

/// Parse a Claude JSONL line into a Message if it's a user/assistant message.
fn parse_claude_message_line(line: &str) -> Option<Message> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    let msg_type = json.get("type")?.as_str()?;
    if msg_type != "user" && msg_type != "assistant" {
        return None;
    }

    let message = json.get("message")?;
    let role = message.get("role")?.as_str()?.to_string();
    let content = message.get("content")?;
    let text = extract_text(content);

    if text.is_empty() {
        return None;
    }

    let timestamp = json
        .get("timestamp")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    Some(Message {
        role,
        content: text,
        timestamp,
    })
}

/// Parse a Codex JSONL line into a Message if it's a user/assistant response_item message.
fn parse_codex_message_line(line: &str) -> Option<Message> {
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
    let text = extract_text(payload.get("content")?);
    if text.is_empty() {
        return None;
    }
    let timestamp = json
        .get("timestamp")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    Some(Message {
        role,
        content: text,
        timestamp,
    })
}

/// Parse Codex file content and return (project_name, parsed messages).
fn parse_codex_file(content: &str) -> (String, Vec<Message>) {
    let mut project: Option<String> = None;
    let mut messages: Vec<Message> = Vec::new();

    for line in content.lines() {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        match json.get("type").and_then(|v| v.as_str()) {
            Some("session_meta") => {
                if project.is_none() {
                    project = json
                        .get("payload")
                        .and_then(|p| p.get("cwd"))
                        .and_then(|c| c.as_str())
                        .and_then(extract_project_from_cwd);
                }
            }
            Some("response_item") => {
                if let Some(msg) = parse_codex_message_line(line) {
                    messages.push(msg);
                }
            }
            _ => {}
        }
    }

    (project.unwrap_or_else(|| "codex".to_string()), messages)
}

#[tool_router]
impl ConversationTools {
    #[tool(
        name = "conversation_search",
        description = "Search conversation history across Claude Code and Codex sessions. Returns matching messages with surrounding context."
    )]
    async fn conversation_search(
        &self,
        request: Parameters<ConversationSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;
        let limit = request.limit.unwrap_or(10);
        let context_before = request.context_before.unwrap_or(3);
        let context_after = request.context_after.unwrap_or(3);
        let recency_weight = request.recency_weight.unwrap_or(0.5).clamp(0.0, 1.0);

        // Parse optional date range filters
        let after_dt = request.after.as_deref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
                .or_else(|| {
                    // Also accept plain dates like "2026-02-23"
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|ndt| ndt.and_utc())
                })
        });
        let before_dt = request.before.as_deref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
                .or_else(|| {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(23, 59, 59))
                        .map(|ndt| ndt.and_utc())
                })
        });

        tracing::info!(
            query = %request.query,
            project = ?request.project,
            after = ?after_dt,
            before = ?before_dt,
            limit = limit,
            "Conversation search started"
        );

        // Compile regex
        let regex = Regex::new(&request.query).map_err(|e| {
            tracing::warn!(error = %e, "Invalid regex pattern");
            McpError::invalid_params(format!("Invalid regex pattern: {}", e), None)
        })?;

        // Check if at least one conversations directory exists
        if !self.claude_conversations_dir.exists() && !self.codex_conversations_dir.exists() {
            tracing::warn!(
                claude_dir = ?self.claude_conversations_dir,
                codex_dir = ?self.codex_conversations_dir,
                "Conversation directories not found"
            );
            return Ok(CallToolResult::success(vec![Content::text("[]")]));
        }

        // Collect all JSONL files with their modification times
        let mut files_with_mtime: Vec<(
            ConversationSource,
            PathBuf,
            std::time::SystemTime,
            Option<String>,
        )> = Vec::new();

        for entry in WalkDir::new(&self.claude_conversations_dir)
            .follow_links(false)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file()
                || path.extension().and_then(|e| e.to_str()) != Some("jsonl")
            {
                continue;
            }

            let project_dir = path
                .parent()
                .and_then(|p| {
                    if p.file_name().and_then(|n| n.to_str()) == Some("subagents") {
                        p.parent()
                    } else {
                        Some(p)
                    }
                })
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let project_name = extract_project_name(project_dir);

            if let Some(ref filter_project) = request.project {
                if !project_name
                    .to_lowercase()
                    .contains(&filter_project.to_lowercase())
                {
                    continue;
                }
            }

            let mtime = match std::fs::metadata(path).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(e) => {
                    tracing::debug!(path = ?path, error = %e, "Could not get mtime, skipping");
                    continue;
                }
            };
            files_with_mtime.push((
                ConversationSource::Claude,
                path.to_path_buf(),
                mtime,
                Some(project_name),
            ));
        }

        for entry in WalkDir::new(&self.codex_conversations_dir)
            .follow_links(false)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file()
                || path.extension().and_then(|e| e.to_str()) != Some("jsonl")
            {
                continue;
            }

            let mtime = match std::fs::metadata(path).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(e) => {
                    tracing::debug!(path = ?path, error = %e, "Could not get mtime, skipping");
                    continue;
                }
            };
            files_with_mtime.push((
                ConversationSource::Codex,
                path.to_path_buf(),
                mtime,
                None,
            ));
        }

        // Sort by modification time (newest first) for deterministic results
        files_with_mtime.sort_by(|a, b| b.1.cmp(&a.1));

        let mut all_matches: Vec<ConversationMatch> = Vec::new();

        // Early exit threshold: stop if we have 2x the limit (newer files processed first)
        let early_exit_threshold = limit * 2;

        for (source, path, _mtime, known_project) in files_with_mtime {
            // Early exit if we have enough matches (files are sorted by mtime, newest first)
            if all_matches.len() >= early_exit_threshold {
                tracing::debug!(
                    matches = all_matches.len(),
                    "Early exit: collected enough matches"
                );
                break;
            }

            // Read and parse the file
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::debug!(path = ?path, error = %e, "Could not read file, skipping");
                    continue;
                }
            };

            let (project_name, messages): (String, Vec<Message>) = match source {
                ConversationSource::Claude => (
                    known_project.unwrap_or_else(|| "unknown".to_string()),
                    content.lines().filter_map(parse_claude_message_line).collect(),
                ),
                ConversationSource::Codex => parse_codex_file(&content),
            };

            if let Some(ref filter_project) = request.project {
                if !project_name
                    .to_lowercase()
                    .contains(&filter_project.to_lowercase())
                {
                    continue;
                }
            }

            if messages.is_empty() {
                continue;
            }

            // Extract session ID from filename
            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Search for matches
            for (idx, msg) in messages.iter().enumerate() {
                if !regex.is_match(&msg.content) {
                    continue;
                }

                // Calculate relevance score
                let timestamp = DateTime::parse_from_rfc3339(&msg.timestamp)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                // Apply date range filters
                if let Some(after) = after_dt {
                    if timestamp < after {
                        continue;
                    }
                }
                if let Some(before) = before_dt {
                    if timestamp > before {
                        continue;
                    }
                }
                let score = recency_score(&timestamp, recency_weight);

                // Get context
                let start = idx.saturating_sub(context_before);
                let end = (idx + 1 + context_after).min(messages.len());

                let ctx_before: Vec<Message> = messages[start..idx].to_vec();
                let ctx_after: Vec<Message> = if idx + 1 < end {
                    messages[idx + 1..end].to_vec()
                } else {
                    Vec::new()
                };

                all_matches.push(ConversationMatch {
                    source: source.as_str().to_string(),
                    project: project_name.clone(),
                    session_id: session_id.clone(),
                    timestamp: msg.timestamp.clone(),
                    match_message: msg.clone(),
                    context_before: ctx_before,
                    context_after: ctx_after,
                    relevance_score: score,
                });
            }
        }

        // --- Semantic search (if available) ---
        if let (Some(db), Some(embedder)) = (&self.vector_db, &self.embedder) {
            if let Ok(query_embedding) = embedder.embed_one(&request.query) {
                let project_filter = request.project.as_deref();
                match db.search_conversation_semantic(
                    &query_embedding,
                    limit * 3,
                    project_filter,
                ) {
                    Ok(semantic_hits) => {
                        tracing::debug!(
                            count = semantic_hits.len(),
                            "Semantic conversation search returned results"
                        );
                        // Deduplicate: only add semantic results not already found by keyword
                        let existing_keys: std::collections::HashSet<String> = all_matches
                            .iter()
                            .map(|m| format!("{}:{}:{}", m.source, m.session_id, m.timestamp))
                            .collect();

                        for hit in semantic_hits {
                            let (source, display_session_id) = decode_session_key(&hit.session_id);
                            let key = format!(
                                "{}:{}:{}",
                                source.as_str(),
                                display_session_id,
                                hit.timestamp
                            );
                            if existing_keys.contains(&key) {
                                continue;
                            }

                            // Apply date range filters to semantic results
                            if let Ok(hit_ts) = DateTime::parse_from_rfc3339(&hit.timestamp) {
                                let hit_utc = hit_ts.with_timezone(&Utc);
                                if let Some(after) = after_dt {
                                    if hit_utc < after {
                                        continue;
                                    }
                                }
                                if let Some(before) = before_dt {
                                    if hit_utc > before {
                                        continue;
                                    }
                                }
                            }

                            // Semantic-only result: use chunk text as the message
                            let similarity = 1.0 - hit.distance;
                            all_matches.push(ConversationMatch {
                                source: source.as_str().to_string(),
                                project: hit.project,
                                session_id: display_session_id,
                                timestamp: hit.timestamp.clone(),
                                match_message: Message {
                                    role: hit.role,
                                    content: hit.chunk_text,
                                    timestamp: hit.timestamp,
                                },
                                context_before: Vec::new(),
                                context_after: Vec::new(),
                                relevance_score: similarity,
                            });
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Semantic conversation search failed");
                    }
                }
            }
        }

        // Sort by relevance score (highest first)
        all_matches.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        all_matches.truncate(limit);

        tracing::info!(
            results = all_matches.len(),
            semantic = self.vector_db.is_some(),
            "Conversation search completed"
        );
        json_result(&all_matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_tools() -> (ConversationTools, TempDir) {
        let dir = TempDir::new().unwrap();
        let claude_dir = dir.path().join("conversations");
        let codex_dir = dir.path().join("codex-sessions");
        std::fs::create_dir_all(&claude_dir).unwrap();
        std::fs::create_dir_all(&codex_dir).unwrap();
        (ConversationTools::new(claude_dir, codex_dir, None, None), dir)
    }

    fn write_test_jsonl(dir: &TempDir, project: &str, session: &str, messages: &[(&str, &str)]) {
        let project_dir = dir.path().join("conversations").join(project);
        std::fs::create_dir_all(&project_dir).unwrap();

        let file_path = project_dir.join(format!("{}.jsonl", session));
        let content: Vec<String> = messages
            .iter()
            .enumerate()
            .map(|(i, (role, text))| {
                serde_json::json!({
                    "type": role,
                    "message": {
                        "role": role,
                        "content": text
                    },
                    "timestamp": format!("2026-01-30T10:{:02}:00Z", i)
                })
                .to_string()
            })
            .collect();

        std::fs::write(file_path, content.join("\n")).unwrap();
    }

    #[tokio::test]
    async fn test_conversation_search_basic() {
        let (tools, dir) = create_test_tools();

        write_test_jsonl(
            &dir,
            "-home-user-git-myproject",
            "session1",
            &[
                ("user", "Hello, how are you?"),
                ("assistant", "I'm doing well, thank you!"),
                ("user", "Can you help me with Rust code?"),
                ("assistant", "Of course! What do you need help with?"),
            ],
        );

        let request = Parameters(ConversationSearchRequest {
            query: "Rust".to_string(),
            project: None,
            context_before: Some(1),
            context_after: Some(1),
            limit: Some(10),
            recency_weight: Some(0.5),
            after: None,
            before: None,
        });

        let result = tools.conversation_search(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        let results: Vec<ConversationMatch> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].match_message.content.contains("Rust"));
        assert_eq!(results[0].source, "claude");
        assert_eq!(results[0].context_before.len(), 1);
        assert_eq!(results[0].context_after.len(), 1);
    }

    #[tokio::test]
    async fn test_conversation_search_project_filter() {
        let (tools, dir) = create_test_tools();

        write_test_jsonl(
            &dir,
            "-home-user-git-project1",
            "session1",
            &[("user", "Test message for project1")],
        );

        write_test_jsonl(
            &dir,
            "-home-user-git-project2",
            "session2",
            &[("user", "Test message for project2")],
        );

        // Search only project1
        let request = Parameters(ConversationSearchRequest {
            query: "Test".to_string(),
            project: Some("project1".to_string()),
            context_before: None,
            context_after: None,
            limit: None,
            recency_weight: None,
            after: None,
            before: None,
        });

        let result = tools.conversation_search(request).await.unwrap();
        let results: Vec<ConversationMatch> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project, "project1");
    }

    #[tokio::test]
    async fn test_conversation_search_codex_source() {
        let (tools, dir) = create_test_tools();
        let codex_dir = dir.path().join("codex-sessions").join("2026").join("02").join("18");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let file_path = codex_dir.join("rollout-abc123.jsonl");
        let content = vec![
            serde_json::json!({
                "timestamp": "2026-02-18T17:10:53.828Z",
                "type": "session_meta",
                "payload": {
                    "cwd": "/home/schalk/git/twolebot/data/topics/abc"
                }
            })
            .to_string(),
            serde_json::json!({
                "timestamp": "2026-02-18T17:10:54.000Z",
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "user",
                    "content": [
                        {"type": "input_text", "text": "Please explain codex indexing"}
                    ]
                }
            })
            .to_string(),
        ];
        std::fs::write(file_path, content.join("\n")).unwrap();

        let request = Parameters(ConversationSearchRequest {
            query: "codex indexing".to_string(),
            project: None,
            context_before: Some(0),
            context_after: Some(0),
            limit: Some(5),
            recency_weight: Some(0.5),
            after: None,
            before: None,
        });

        let result = tools.conversation_search(request).await.unwrap();
        let results: Vec<ConversationMatch> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "codex");
    }

    #[tokio::test]
    async fn test_conversation_search_empty_results() {
        let (tools, _dir) = create_test_tools();

        let request = Parameters(ConversationSearchRequest {
            query: "nonexistent".to_string(),
            project: None,
            context_before: None,
            context_after: None,
            limit: None,
            recency_weight: None,
            after: None,
            before: None,
        });

        let result = tools.conversation_search(request).await.unwrap();
        let results: Vec<ConversationMatch> =
            serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_conversation_search_invalid_regex() {
        let (tools, _dir) = create_test_tools();

        let request = Parameters(ConversationSearchRequest {
            query: "[invalid".to_string(),
            project: None,
            context_before: None,
            context_after: None,
            limit: None,
            recency_weight: None,
            after: None,
            before: None,
        });

        let result = tools.conversation_search(request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(
            extract_project_name("-home-schalk-git-twolebot-data"),
            "twolebot"
        );
        assert_eq!(
            extract_project_name("-home-user-git-myproject"),
            "myproject"
        );
        assert_eq!(extract_project_name("simple"), "simple");
    }

    #[test]
    fn test_extract_text() {
        // String content
        let string_content = serde_json::json!("Hello world");
        assert_eq!(extract_text(&string_content), "Hello world");

        // Array content with text blocks
        let array_content = serde_json::json!([
            {"type": "text", "text": "First"},
            {"type": "text", "text": "Second"}
        ]);
        assert_eq!(extract_text(&array_content), "First\nSecond");

        // Empty content
        let empty = serde_json::json!(null);
        assert_eq!(extract_text(&empty), "");
    }

    /// Integration tests with the real embedding model.
    /// Run with: cargo test --ignored
    mod semantic_tests {
        use super::*;
        use crate::semantic::{hash_content, Embedder, VectorDb};

        async fn create_semantic_tools() -> (ConversationTools, TempDir) {
            let dir = TempDir::new().unwrap();
            let db_path = dir.path().join("vectors.sqlite3");
            let db = Arc::new(VectorDb::open(&db_path).unwrap());
            let embedder = Embedder::global(2).await.unwrap();

            let convos_dir = dir.path().join("conversations");
            std::fs::create_dir_all(&convos_dir).unwrap();
            let codex_dir = dir.path().join("codex-sessions");
            std::fs::create_dir_all(&codex_dir).unwrap();

            let tools =
                ConversationTools::new(convos_dir, codex_dir, Some(db.clone()), Some(embedder.clone()));
            (tools, dir)
        }

        /// Write a JSONL conversation file AND index messages into the vector DB.
        fn write_and_index_conversation(
            dir: &TempDir,
            db: &VectorDb,
            embedder: &Embedder,
            project: &str,
            session: &str,
            messages: &[(&str, &str)], // (role, content)
        ) {
            let convos_dir = dir.path().join("conversations");
            let project_dir = convos_dir.join(project);
            std::fs::create_dir_all(&project_dir).unwrap();

            // Write JSONL file
            let file_path = project_dir.join(format!("{}.jsonl", session));
            let content: Vec<String> = messages
                .iter()
                .enumerate()
                .map(|(i, (role, text))| {
                    serde_json::json!({
                        "type": role,
                        "message": {
                            "role": role,
                            "content": text
                        },
                        "timestamp": format!("2026-01-15T10:{:02}:00Z", i)
                    })
                    .to_string()
                })
                .collect();
            std::fs::write(&file_path, content.join("\n")).unwrap();

            // Index into vector DB
            let file_hash = hash_content(&content.join("\n"));
            for (i, (role, text)) in messages.iter().enumerate() {
                let embedding = embedder.embed_one(text).unwrap();
                db.insert_conversation_chunk(
                    session,
                    project,
                    i,
                    0,
                    role,
                    text,
                    &format!("2026-01-15T10:{:02}:00Z", i),
                    &file_hash,
                    &embedding,
                )
                .unwrap();
            }
        }

        #[tokio::test]
        #[ignore]
        async fn test_semantic_conversation_search_no_keyword_overlap() {
            // Proves semantic search surfaces conversations by meaning,
            // not just keyword matching.
            let (tools, dir) = create_semantic_tools().await;
            let db = tools.vector_db.as_ref().unwrap();
            let embedder = tools.embedder.as_ref().unwrap();

            write_and_index_conversation(
                &dir, db, embedder,
                "-home-user-git-twolebot",
                "session-deploy",
                &[
                    ("user", "Can you help me set up CI/CD pipelines for automated deployment?"),
                    ("assistant", "Sure! I recommend using GitHub Actions with Docker containers."),
                ],
            );

            write_and_index_conversation(
                &dir, db, embedder,
                "-home-user-git-myproject",
                "session-recipe",
                &[
                    ("user", "What ingredients do I need for a chocolate soufflé?"),
                    ("assistant", "You'll need dark chocolate, eggs, sugar, and butter."),
                ],
            );

            // Search for "shipping code to production servers" — semantically
            // close to the CI/CD conversation but shares NO keywords.
            let request = Parameters(ConversationSearchRequest {
                query: "shipping code to production servers".to_string(),
                project: None,
                context_before: None,
                context_after: None,
                limit: Some(10),
                recency_weight: Some(0.0), // Pure relevance, no recency bias
                after: None,
                before: None,
            });
            let result = tools.conversation_search(request).await.unwrap();
            let results: Vec<ConversationMatch> =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

            assert!(
                !results.is_empty(),
                "Semantic search should find the CI/CD conversation"
            );
            // The deployment-related conversation should appear
            let has_deploy = results
                .iter()
                .any(|m| m.match_message.content.contains("CI/CD") || m.match_message.content.contains("deployment"));
            assert!(
                has_deploy,
                "Should find the deployment conversation via semantic similarity"
            );
        }

        #[tokio::test]
        #[ignore]
        async fn test_semantic_conversation_search_ranks_relevant_higher() {
            let (tools, dir) = create_semantic_tools().await;
            let db = tools.vector_db.as_ref().unwrap();
            let embedder = tools.embedder.as_ref().unwrap();

            write_and_index_conversation(
                &dir, db, embedder,
                "-home-user-git-twolebot",
                "session-memory",
                &[
                    ("user", "How does the memory allocation work in this system?"),
                    ("assistant", "The allocator uses a slab-based approach for small objects."),
                ],
            );

            write_and_index_conversation(
                &dir, db, embedder,
                "-home-user-git-twolebot",
                "session-weather",
                &[
                    ("user", "What's the forecast for tomorrow?"),
                    ("assistant", "It should be sunny with temperatures around 25 degrees."),
                ],
            );

            // Search for "heap and stack usage" — related to memory allocation
            let request = Parameters(ConversationSearchRequest {
                query: "heap and stack usage".to_string(),
                project: None,
                context_before: None,
                context_after: None,
                limit: Some(10),
                recency_weight: Some(0.0),
                after: None,
                before: None,
            });
            let result = tools.conversation_search(request).await.unwrap();
            let results: Vec<ConversationMatch> =
                serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();

            assert!(!results.is_empty());
            // Memory-related conversation should rank above weather
            let mem_pos = results
                .iter()
                .position(|m| {
                    m.match_message.content.contains("memory")
                        || m.match_message.content.contains("allocat")
                });
            let weather_pos = results
                .iter()
                .position(|m| m.match_message.content.contains("forecast") || m.match_message.content.contains("sunny"));

            if let (Some(mp), Some(wp)) = (mem_pos, weather_pos) {
                assert!(
                    mp < wp,
                    "Memory conversation should rank above weather for allocation query"
                );
            }
        }
    }
}
