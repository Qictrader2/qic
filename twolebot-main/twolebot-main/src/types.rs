//! Centralized request/response types (HTTP + MCP).
//!
//! This module is intentionally "boring": pure data structs/enums with serde/JsonSchema derives.
//! Business logic should live in dedicated service modules (e.g. `cron::service`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod api {
    use super::*;

    // ============ Shared / Pagination ============

    #[derive(Debug, Deserialize)]
    pub struct PaginationQuery {
        #[serde(default = "default_page")]
        pub page: usize,
        #[serde(default = "default_page_size")]
        pub page_size: usize,
    }

    fn default_page() -> usize {
        0
    }

    fn default_page_size() -> usize {
        50
    }

    // ============ Feed ============

    #[derive(Debug, Serialize)]
    pub struct FeedResponse {
        pub pending: Vec<serde_json::Value>,
        pub pending_count: usize,
        pub running: Option<serde_json::Value>,
        pub recent_completed: Vec<serde_json::Value>,
        pub completed_count: usize,
    }

    #[derive(Debug, Serialize)]
    pub struct ResponseFeedResponse {
        pub pending: Vec<serde_json::Value>,
        pub pending_count: usize,
        pub recent_sent: Vec<serde_json::Value>,
        pub sent_count: usize,
        pub recent_failed: Vec<serde_json::Value>,
        pub failed_count: usize,
    }

    // ============ Messages ============

    #[derive(Debug, Serialize)]
    pub struct ChatsResponse {
        pub chats: Vec<ChatSummary>,
    }

    #[derive(Debug, Serialize)]
    pub struct ChatSummary {
        pub chat_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic_id: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<String>,
        pub message_count: usize,
    }

    #[derive(Debug, Deserialize)]
    pub struct MessagesQuery {
        #[serde(default = "default_page")]
        pub page: usize,
        #[serde(default = "default_page_size")]
        pub page_size: usize,
        #[serde(default)]
        pub search: Option<String>,
        /// Filter by topic_id. Use "none" for messages with no topic.
        #[serde(default)]
        pub topic_id: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct MessagesResponse {
        pub messages: Vec<serde_json::Value>,
        pub total: usize,
        pub page: usize,
        pub page_size: usize,
        pub total_pages: usize,
    }

    // ============ Logs ============

    #[derive(Debug, Deserialize)]
    pub struct LogsQuery {
        #[serde(default = "default_page")]
        pub page: usize,
        #[serde(default = "default_log_page_size")]
        pub page_size: usize,
        #[serde(default)]
        pub search: Option<String>,
        #[serde(default)]
        pub level: Option<String>,
    }

    fn default_log_page_size() -> usize {
        100
    }

    #[derive(Debug, Serialize)]
    pub struct LogsResponse {
        pub entries: Vec<serde_json::Value>,
        pub total: usize,
        pub page: usize,
        pub page_size: usize,
        pub total_pages: usize,
    }

    // ============ Status ============

    #[derive(Debug, Serialize)]
    pub struct StatusResponse {
        pub status: String,
        pub version: String,
    }

    // ============ Cron (HTTP) ============

    #[derive(Debug, Serialize)]
    pub struct CronJobsResponse {
        pub jobs: Vec<CronJobSummary>,
    }

    #[derive(Debug, Serialize)]
    pub struct CronJobSummary {
        pub id: String,
        pub name: String,
        pub schedule: String,
        pub status: String,
        pub next_run: Option<String>,
        pub last_run: Option<String>,
        pub created_at: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct CronJobsQuery {
        /// Filter by status: "active", "paused", "all" (default: "all")
        #[serde(default = "default_cron_status")]
        pub status: String,
    }

    fn default_cron_status() -> String {
        "all".to_string()
    }

    #[derive(Debug, Deserialize)]
    pub struct SnoozeRequest {
        /// Minutes to delay the next execution
        pub minutes: i64,
    }

    #[derive(Debug, Serialize)]
    pub struct CronStatusResponse {
        pub active_jobs: usize,
        pub paused_jobs: usize,
        pub waiting_executions: usize,
    }

    // ============ Setup (HTTP) ============

    pub mod setup {
        use super::*;

        #[derive(Debug, Deserialize)]
        pub struct TelegramSetupRequest {
            pub token: String,
        }

        #[derive(Debug, Serialize)]
        pub struct TelegramSetupResponse {
            pub success: bool,
            pub bot_name: Option<String>,
            pub error: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        pub struct GeminiSetupRequest {
            pub key: String,
        }

        #[derive(Debug, Serialize)]
        pub struct GeminiSetupResponse {
            pub success: bool,
            pub error: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct ClaudeInstallResponse {
            pub success: bool,
            pub version: Option<String>,
            pub error: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct ClaudeAuthCheckResponse {
            pub installed: bool,
            pub version: Option<String>,
            pub authenticated: bool,
            pub auth_mode: Option<String>,
            pub account_email: Option<String>,
            pub account_name: Option<String>,
            pub needs_update: bool,
            pub latest_version: Option<String>,
            pub error: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct ClaudeTestResponse {
            pub success: bool,
            pub output: Option<String>,
            pub error: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct SetupCompleteResponse {
            pub success: bool,
            pub message: String,
        }

        #[derive(Debug, Serialize)]
        pub struct ThreadingCheckResponse {
            pub success: bool,
            pub enabled: bool,
            pub error: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct ApiKeysResponse {
            pub has_telegram_token: bool,
            pub telegram_token_masked: Option<String>,
            pub telegram_status: Option<ApiKeyStatus>,
            pub has_gemini_key: bool,
            pub gemini_key_masked: Option<String>,
            pub gemini_status: Option<ApiKeyStatus>,
            pub claude_code_status: Option<ClaudeCodeStatus>,
            pub has_user_contacted: Option<bool>,
        }

        #[derive(Debug, Serialize)]
        pub struct ClaudeCodeStatus {
            pub auth_mode: String, // "oauth" or "api_key"
            pub account_email: Option<String>,
            pub account_name: Option<String>,
            pub organization: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct ApiKeyStatus {
            pub valid: bool,
            pub error: Option<String>,
            pub info: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        pub struct UpdateApiKeysRequest {
            #[serde(default)]
            pub telegram_token: Option<String>,
            #[serde(default)]
            pub gemini_key: Option<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct UpdateApiKeysResponse {
            pub success: bool,
            pub telegram_updated: bool,
            pub gemini_updated: bool,
            pub telegram_error: Option<String>,
            pub gemini_error: Option<String>,
        }
    }
}

pub mod cron {
    use super::*;

    // Shared between HTTP (REST) and MCP tools.
    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct ScheduleJobRequest {
        /// The prompt to execute (required, can be very long)
        pub prompt: String,
        /// Human-readable name for the job (required)
        pub name: String,
        /// Schedule as minutes from now (for one-shot jobs)
        #[serde(default)]
        pub in_minutes: Option<i64>,
        /// Schedule as cron expression (for recurring jobs)
        #[serde(default)]
        pub cron: Option<String>,
        /// Schedule as multiple cron expressions (for jobs with multiple trigger times sharing one topic)
        #[serde(default)]
        pub crons: Option<Vec<String>>,
        /// Origin Telegram chat ID — route executions to this chat (optional, resolved automatically if omitted)
        #[serde(default)]
        pub origin_chat_id: Option<i64>,
    }

    #[derive(Debug, Serialize, JsonSchema)]
    pub struct ScheduleJobResponse {
        pub job_id: String,
        pub next_run: Option<String>,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct ListJobsRequest {
        /// Filter by status: "active", "paused", or "all" (default: "active")
        #[serde(default)]
        pub status: Option<String>,
        /// Maximum number of jobs to return (default: 50)
        #[serde(default)]
        pub limit: Option<usize>,
        /// Number of jobs to skip (for pagination)
        #[serde(default)]
        pub offset: Option<usize>,
    }

    #[derive(Debug, Serialize, JsonSchema)]
    pub struct JobSummary {
        pub job_id: String,
        pub name: String,
        pub schedule: String,
        pub next_run: Option<String>,
        pub last_run: Option<String>,
        pub status: String,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct CancelJobRequest {
        /// The job ID to cancel
        pub job_id: String,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct SnoozeJobRequest {
        /// The job ID to snooze
        pub job_id: String,
        /// Minutes to delay the next execution
        pub minutes: i64,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct CloseTopicRequest {
        /// The job ID whose topic should be closed
        pub job_id: String,
        /// The Telegram chat ID containing the topic
        pub chat_id: i64,
    }
}

pub mod mcp {
    use super::*;

    pub mod memory {
        use super::*;

        #[derive(Debug, Deserialize, JsonSchema)]
        pub struct MemorySearchRequest {
            /// Search query (regex pattern)
            pub query: String,
            /// Maximum number of results (default: 10)
            #[serde(default)]
            pub limit: Option<usize>,
            /// Number of results to skip (for pagination)
            #[serde(default)]
            pub offset: Option<usize>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
        pub struct MemorySearchResult {
            /// Relative path from memory directory
            pub path: String,
            /// Matching text with context
            pub snippet: String,
            /// Number of matches in this file
            pub match_count: usize,
        }

        #[derive(Debug, Deserialize, JsonSchema)]
        pub struct MemoryReadRequest {
            /// Relative path from memory directory
            pub path: String,
        }

        #[derive(Debug, Serialize, Deserialize, JsonSchema)]
        pub struct MemoryReadResult {
            /// Full file content
            pub content: String,
            /// Whether the file had YAML frontmatter
            pub has_frontmatter: bool,
        }

        #[derive(Debug, Deserialize, JsonSchema)]
        pub struct MemoryWriteRequest {
            /// Relative path from memory directory (must end in .md)
            pub path: String,
            /// Content to write
            pub content: String,
            /// Write mode: "replace" (default) or "append"
            #[serde(default)]
            pub mode: Option<String>,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum WriteMode {
            Replace,
            Append,
        }
    }

    pub mod conversations {
        use super::*;

        #[derive(Debug, Deserialize, JsonSchema)]
        pub struct ConversationSearchRequest {
            /// Search query (regex pattern)
            pub query: String,
            /// Filter to specific project (e.g., "twolebot")
            #[serde(default)]
            pub project: Option<String>,
            /// Messages before match (default: 3)
            #[serde(default)]
            pub context_before: Option<usize>,
            /// Messages after match (default: 3)
            #[serde(default)]
            pub context_after: Option<usize>,
            /// Max results (default: 10)
            #[serde(default)]
            pub limit: Option<usize>,
            /// 0.0-1.0, higher = prefer recent (default: 0.5)
            #[serde(default)]
            pub recency_weight: Option<f32>,
            /// Only include messages after this ISO datetime (e.g., "2026-02-23T00:00:00Z")
            #[serde(default)]
            pub after: Option<String>,
            /// Only include messages before this ISO datetime (e.g., "2026-02-24T00:00:00Z")
            #[serde(default)]
            pub before: Option<String>,
        }

        #[derive(Debug, Serialize, Deserialize, JsonSchema)]
        pub struct ConversationMatch {
            /// Source harness for this conversation ("claude" or "codex")
            pub source: String,
            /// Project name extracted from path
            pub project: String,
            /// Session UUID
            pub session_id: String,
            /// ISO timestamp of matching message
            pub timestamp: String,
            /// The message that matched
            pub match_message: ConversationMessage,
            /// N messages before
            pub context_before: Vec<ConversationMessage>,
            /// N messages after
            pub context_after: Vec<ConversationMessage>,
            /// Combined text + recency score
            pub relevance_score: f32,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
        pub struct ConversationMessage {
            /// "user" or "assistant"
            pub role: String,
            /// Text content (extracted from various formats)
            pub content: String,
            /// ISO timestamp
            pub timestamp: String,
        }
    }
}

pub mod work {
    use super::*;

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct ListProjectsRequest {
        #[serde(default)]
        pub active_only: Option<bool>,
        #[serde(default)]
        pub limit: Option<i32>,
        #[serde(default)]
        pub git_remote_url: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct GetProjectRequest {
        pub project_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct GetProjectByGitRemoteRequest {
        pub git_remote_url: String,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct CreateProjectRequest {
        pub name: String,
        #[serde(default)]
        pub description: Option<String>,
        #[serde(default)]
        pub tags: Option<Vec<String>>,
        #[serde(default)]
        pub git_remote_url: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct UpdateProjectRequest {
        pub project_id: i64,
        pub name: String,
        #[serde(default)]
        pub description: Option<String>,
        #[serde(default)]
        pub tags: Option<Vec<String>>,
        #[serde(default)]
        pub git_remote_url: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct ArchiveProjectRequest {
        pub project_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct ListTasksRequest {
        #[serde(default)]
        pub project_id: Option<i64>,
        #[serde(default)]
        pub status: Option<Vec<String>>,
        #[serde(default)]
        pub limit: Option<i32>,
        #[serde(default)]
        pub compact: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct GetTaskRequest {
        pub task_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct CreateTaskRequest {
        pub project_id: i64,
        pub title: String,
        #[serde(default)]
        pub description: Option<String>,
        #[serde(default)]
        pub status: Option<String>,
        #[serde(default)]
        pub priority: Option<String>,
        #[serde(default)]
        pub tags: Option<Vec<String>>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct UpdateTaskRequest {
        pub task_id: i64,
        #[serde(default)]
        pub title: Option<String>,
        #[serde(default)]
        pub description: Option<String>,
        #[serde(default)]
        pub status: Option<String>,
        #[serde(default)]
        pub priority: Option<String>,
        #[serde(default)]
        pub tags: Option<Vec<String>>,
        /// Comment to attach (required when transitioning to ready_for_review)
        #[serde(default)]
        pub comment: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct TakeNextRequest {
        pub project_id: i64,
        #[serde(default)]
        pub force: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct MoveTaskRequest {
        pub task_id: i64,
        pub position: String,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct RejectReviewRequest {
        pub task_id: i64,
        pub reviewer_comment: String,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct DependencyRequest {
        pub task_id: i64,
        pub depends_on_task_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct AnalyticsRequest {
        #[serde(default)]
        pub project_id: Option<i64>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct SearchDocumentsRequest {
        pub query: String,
        #[serde(default)]
        pub project_id: Option<i64>,
        #[serde(default)]
        pub limit: Option<i32>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct GetDocumentRequest {
        pub document_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct CreateDocumentRequest {
        pub project_id: i64,
        #[serde(rename = "type")]
        pub document_type: String,
        pub title: String,
        #[serde(default)]
        pub content: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct UpdateDocumentRequest {
        pub document_id: i64,
        pub title: String,
        pub content: String,
        #[serde(rename = "type")]
        pub document_type: String,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct DeleteDocumentRequest {
        pub document_id: i64,
    }

    /// Read document content with optional line range (like the Read file tool).
    /// Returns line-numbered content. Use offset/limit for large documents.
    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct DocReadRequest {
        pub document_id: i64,
        /// Line number to start reading from (1-based). Defaults to 1.
        #[serde(default)]
        pub offset: Option<i64>,
        /// Number of lines to read. Defaults to all remaining lines.
        #[serde(default)]
        pub limit: Option<i64>,
    }

    /// Exact string replacement in a document (like the Edit file tool).
    /// old_string must be unique in the document unless replace_all is true.
    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct DocEditRequest {
        pub document_id: i64,
        /// The exact text to find and replace. Must be unique in the document
        /// unless replace_all is true.
        pub old_string: String,
        /// The replacement text.
        pub new_string: String,
        /// Replace all occurrences of old_string (default false).
        #[serde(default)]
        pub replace_all: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct ListCommentsRequest {
        #[serde(default)]
        pub task_id: Option<i64>,
        #[serde(default)]
        pub document_id: Option<i64>,
        #[serde(default)]
        pub limit: Option<i32>,
        #[serde(default)]
        pub page: Option<i32>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct ListTaskCommentsRequest {
        pub task_id: i64,
        #[serde(default)]
        pub limit: Option<i32>,
        #[serde(default)]
        pub page: Option<i32>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct ListDocumentCommentsRequest {
        pub document_id: i64,
        #[serde(default)]
        pub limit: Option<i32>,
        #[serde(default)]
        pub page: Option<i32>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct UpsertCommentRequest {
        #[serde(default)]
        pub comment_id: Option<i64>,
        #[serde(default)]
        pub task_id: Option<i64>,
        #[serde(default)]
        pub document_id: Option<i64>,
        pub content: String,
        #[serde(default)]
        pub parent_comment_id: Option<i64>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct DeleteCommentRequest {
        pub comment_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct RecentActivityRequest {
        #[serde(default)]
        pub limit: Option<i32>,
        #[serde(default)]
        pub project_id: Option<i64>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct GetLiveBoardRequest {
        #[serde(default)]
        pub backlog_limit: Option<i32>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct SelectTasksRequest {
        pub task_ids: Vec<i64>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct DeselectTaskRequest {
        pub task_id: i64,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct MoveSelectionRequest {
        pub task_id: i64,
        pub position: String,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct EnsureAgentRequest {
        #[serde(default)]
        pub auto_select_from_todo: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct PmSearchRequest {
        pub query: String,
        #[serde(default)]
        pub project_id: Option<i64>,
        #[serde(default)]
        pub limit: Option<i64>,
    }
}

pub mod send {
    use super::*;

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct SendFileRequest {
        /// Absolute path to the file on the server filesystem
        pub file_path: String,
        /// Optional caption / description shown alongside the file
        #[serde(default)]
        pub caption: Option<String>,
        /// Telegram chat ID (required for Telegram delivery)
        #[serde(default)]
        pub chat_id: Option<i64>,
        /// Telegram message thread ID (optional, for forum topics)
        #[serde(default)]
        pub message_thread_id: Option<i64>,
        /// Web conversation ID (required for web delivery)
        #[serde(default)]
        pub conversation_id: Option<String>,
    }
}

pub mod image {
    use super::*;

    /// Deserialize an Option<i64> that accepts both JSON numbers and strings.
    /// MCP tool calls sometimes pass integers as strings.
    fn deserialize_optional_i64_lenient<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct OptionalI64Visitor;

        impl<'de> de::Visitor<'de> for OptionalI64Visitor {
            type Value = Option<i64>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an integer, a string containing an integer, or null")
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(Some(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Some(v as i64))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                if v.is_empty() {
                    return Ok(None);
                }
                v.parse::<i64>().map(Some).map_err(de::Error::custom)
            }

            fn visit_some<D: serde::Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                deserializer.deserialize_any(Self)
            }
        }

        deserializer.deserialize_any(OptionalI64Visitor)
    }

    #[derive(Debug, Clone, Deserialize, JsonSchema)]
    pub struct GenerateImageRequest {
        /// Text prompt describing the image to generate or the edit to make
        pub prompt: String,
        /// Optional path to an input image for editing (absolute path on server filesystem)
        #[serde(default)]
        pub input_image_path: Option<String>,
        /// Optional list of input image paths for multi-image editing (up to 14 for Gemini)
        #[serde(default)]
        pub input_image_paths: Option<Vec<String>>,
        /// Quality tier: "premium" (default, Gemini 3 Pro) or "fast" (Gemini 2.5 Flash, cheaper)
        #[serde(default)]
        pub quality: Option<String>,
        /// Image resolution: "1K" (default, 720p), "2K", or "4K"
        #[serde(default)]
        pub image_size: Option<String>,
        /// Aspect ratio: "1:1" (default), "2:3", "3:2", "3:4", "4:3", "4:5", "5:4", "9:16", "16:9", "21:9"
        #[serde(default)]
        pub aspect_ratio: Option<String>,
        /// Telegram chat ID (required for Telegram delivery)
        #[serde(default, deserialize_with = "deserialize_optional_i64_lenient")]
        #[schemars(with = "Option<i64>")]
        pub chat_id: Option<i64>,
        /// Telegram message thread ID (optional, for forum topics)
        #[serde(default, deserialize_with = "deserialize_optional_i64_lenient")]
        #[schemars(with = "Option<i64>")]
        pub message_thread_id: Option<i64>,
        /// Web conversation ID (required for web delivery)
        #[serde(default)]
        pub conversation_id: Option<String>,
    }
}
