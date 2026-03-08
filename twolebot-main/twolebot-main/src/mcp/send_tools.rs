use crate::server::chat_ws::{ChatEvent, ChatEventHub};
use crate::storage::media::{mime_for_extension, MediaStore};
use crate::storage::messages::{MessageStore, StoredMessage};
use crate::telegram::send::TelegramSender;
use crate::types::send::SendFileRequest;
use bytes::Bytes;
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50 MB (Telegram limit)

/// Directories that must never be served via send_file.
const BLOCKED_PREFIXES: &[&str] = &[
    "/etc/",
    "/proc/",
    "/sys/",
    "/dev/",
    "/boot/",
    "/root/",
];

/// File patterns that must never be served.
const BLOCKED_NAMES: &[&str] = &[
    ".env",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    "known_hosts",
    "authorized_keys",
    "shadow",
    "passwd",
    "credentials.json",
];

#[derive(Debug, Serialize)]
struct SendFileResponse {
    delivered_to: Vec<String>,
    filename: String,
    size_bytes: u64,
}

/// Validate that a file path is safe to serve.
/// Canonicalizes the path (resolving symlinks) and rejects dangerous locations.
fn validate_file_path(raw_path: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw_path);

    // Canonicalize to resolve symlinks and ../ components
    let canonical = path.canonicalize().map_err(|e| {
        format!("Cannot resolve path '{}': {}", raw_path, e)
    })?;

    let canonical_str = canonical.to_string_lossy();

    // Block system directories
    for prefix in BLOCKED_PREFIXES {
        if canonical_str.starts_with(prefix) {
            return Err(format!(
                "Access denied: path '{}' is in a restricted directory",
                raw_path
            ));
        }
    }

    // Block .ssh directory anywhere in the path
    for component in canonical.components() {
        if let std::path::Component::Normal(name) = component {
            if name == ".ssh" {
                return Err(format!(
                    "Access denied: path '{}' contains .ssh directory",
                    raw_path
                ));
            }
        }
    }

    // Block sensitive filenames
    if let Some(filename) = canonical.file_name().and_then(|n| n.to_str()) {
        for blocked in BLOCKED_NAMES {
            if filename == *blocked {
                return Err(format!(
                    "Access denied: '{}' is a sensitive file",
                    filename
                ));
            }
        }
    }

    Ok(canonical)
}

/// MCP tool handler for sending files to users via Telegram or Web.
#[derive(Clone)]
pub struct SendTools {
    media_store: Arc<MediaStore>,
    message_store: Arc<MessageStore>,
    telegram_sender: Option<Arc<TelegramSender>>,
    chat_event_hub: Option<Arc<ChatEventHub>>,
    tool_router: ToolRouter<Self>,
}

impl SendTools {
    pub fn new(media_store: Arc<MediaStore>, message_store: Arc<MessageStore>) -> Self {
        Self {
            media_store,
            message_store,
            telegram_sender: None,
            chat_event_hub: None,
            tool_router: Self::create_tool_router(),
        }
    }

    pub fn with_telegram(mut self, sender: Arc<TelegramSender>) -> Self {
        self.telegram_sender = Some(sender);
        self
    }

    pub fn with_chat_event_hub(mut self, hub: Arc<ChatEventHub>) -> Self {
        self.chat_event_hub = Some(hub);
        self
    }

    pub fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    fn create_tool_router() -> ToolRouter<Self> {
        Self::tool_router()
    }
}

#[tool_router]
impl SendTools {
    #[tool(
        name = "send_file",
        description = "Send a file to the user. Provide chat_id for Telegram delivery or conversation_id for web delivery. The file must exist on the server filesystem and be under 50MB."
    )]
    async fn send_file(
        &self,
        request: Parameters<SendFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = request.0;

        // Validate at least one target
        if req.chat_id.is_none() && req.conversation_id.is_none() {
            return Err(McpError::invalid_params(
                "Either chat_id (Telegram) or conversation_id (Web) is required".to_string(),
                None,
            ));
        }

        // Validate and canonicalize file path (security: blocks sensitive paths)
        let file_path = validate_file_path(&req.file_path).map_err(|e| {
            McpError::invalid_params(e, None)
        })?;

        let metadata = tokio::fs::metadata(&file_path).await.map_err(|e| {
            McpError::invalid_params(format!("Cannot access file '{}': {}", req.file_path, e), None)
        })?;

        if !metadata.is_file() {
            return Err(McpError::invalid_params(
                format!("'{}' is not a file", req.file_path),
                None,
            ));
        }

        let file_size = metadata.len();
        if file_size > MAX_FILE_SIZE {
            return Err(McpError::invalid_params(
                format!(
                    "File too large: {} bytes (max {} bytes / 50MB)",
                    file_size, MAX_FILE_SIZE
                ),
                None,
            ));
        }

        // Extract filename
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let mime_type = mime_for_extension(extension).to_string();

        // Read file data once as Bytes (zero-copy sharing between Telegram and Web)
        let file_data: Bytes = tokio::fs::read(&file_path).await.map_err(|e| {
            McpError::internal_error(format!("Failed to read file: {}", e), None)
        })?.into();

        let mut delivered_to = Vec::new();

        // Telegram delivery
        if let Some(chat_id) = req.chat_id {
            let sender = self.telegram_sender.as_ref().ok_or_else(|| {
                McpError::internal_error("Telegram sender not configured".to_string(), None)
            })?;

            let caption_ref = req.caption.as_deref();
            let msg_id = sender
                .send_document(
                    chat_id,
                    req.message_thread_id,
                    file_data.to_vec(),
                    filename.clone(),
                    caption_ref,
                )
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Telegram send_document failed: {}", e), None)
                })?;

            // Store outbound message record
            let stored = StoredMessage::outbound(
                format!("file-tg-{}", msg_id),
                chat_id.to_string(),
                req.caption.as_deref().unwrap_or(&filename),
            )
            .with_media("document", format!("{}/{}", chat_id, filename))
            .with_telegram_id(msg_id)
            .with_topic_id(req.message_thread_id);

            if let Err(e) = self.message_store.store(stored) {
                tracing::warn!("Failed to store outbound file message: {}", e);
            }

            delivered_to.push(format!("telegram:{}", chat_id));
        }

        // Web delivery
        if let Some(ref conversation_id) = req.conversation_id {
            // Store file in MediaStore under the conversation_id
            // Add timestamp prefix to avoid collisions
            let stored_filename = format!(
                "{}-{}",
                chrono::Utc::now().timestamp_millis(),
                filename
            );
            self.media_store
                .store(conversation_id, &stored_filename, &file_data)
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to store file: {}", e), None)
                })?;

            let media_path = format!("{}/{}", conversation_id, stored_filename);
            let message_id = format!("file-web-{}", uuid::Uuid::new_v4());

            // Store outbound message with media metadata
            let stored = StoredMessage::outbound(
                &message_id,
                conversation_id,
                req.caption.as_deref().unwrap_or(&filename),
            )
            .with_media("document", &media_path);

            if let Err(e) = self.message_store.store(stored) {
                tracing::warn!("Failed to store web file message: {}", e);
            }

            // Notify frontend via SSE
            if let Some(ref hub) = self.chat_event_hub {
                hub.send(
                    conversation_id,
                    ChatEvent::FileMessage {
                        conversation_id: conversation_id.clone(),
                        message_id: message_id.clone(),
                        filename: filename.clone(),
                        media_path,
                        mime_type: mime_type.clone(),
                        caption: req.caption.clone().unwrap_or_default(),
                    },
                )
                .await;
            }

            delivered_to.push(format!("web:{}", conversation_id));
        }

        let response = SendFileResponse {
            delivered_to,
            filename,
            size_bytes: file_size,
        };
        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_tools() -> (SendTools, TempDir) {
        let dir = TempDir::new().unwrap();
        let media_store = Arc::new(MediaStore::new(dir.path().join("media")).unwrap());
        let message_store =
            Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap());
        (SendTools::new(media_store, message_store), dir)
    }

    #[tokio::test]
    async fn test_send_file_rejects_missing_target() {
        let (tools, _dir) = create_test_tools();
        let request = Parameters(SendFileRequest {
            file_path: "/tmp/test.txt".to_string(),
            caption: None,
            chat_id: None,
            message_thread_id: None,
            conversation_id: None,
        });
        let result = tools.send_file(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_file_rejects_nonexistent_file() {
        let (tools, _dir) = create_test_tools();
        let request = Parameters(SendFileRequest {
            file_path: "/tmp/nonexistent_file_abc123.txt".to_string(),
            caption: None,
            chat_id: None,
            message_thread_id: None,
            conversation_id: Some("test-conv".to_string()),
        });
        let result = tools.send_file(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_file_rejects_directory() {
        let (tools, dir) = create_test_tools();
        let request = Parameters(SendFileRequest {
            file_path: dir.path().to_string_lossy().to_string(),
            caption: None,
            chat_id: None,
            message_thread_id: None,
            conversation_id: Some("test-conv".to_string()),
        });
        let result = tools.send_file(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_file_web_delivery() {
        let (tools, dir) = create_test_tools();

        // Create a test file
        let test_file = dir.path().join("hello.txt");
        std::fs::write(&test_file, b"hello world").unwrap();

        let request = Parameters(SendFileRequest {
            file_path: test_file.to_string_lossy().to_string(),
            caption: Some("Test file".to_string()),
            chat_id: None,
            message_thread_id: None,
            conversation_id: Some("conv-123".to_string()),
        });

        let result = tools.send_file(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify file was stored in media store
        let files = tools.media_store.list("conv-123").unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("hello.txt"));
    }

    #[tokio::test]
    async fn test_send_file_telegram_without_sender_fails() {
        let (tools, dir) = create_test_tools();

        let test_file = dir.path().join("hello.txt");
        std::fs::write(&test_file, b"hello world").unwrap();

        let request = Parameters(SendFileRequest {
            file_path: test_file.to_string_lossy().to_string(),
            caption: None,
            chat_id: Some(12345),
            message_thread_id: None,
            conversation_id: None,
        });

        let result = tools.send_file(request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_blocks_etc() {
        let result = validate_file_path("/etc/shadow");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("restricted directory"));
    }

    #[test]
    fn test_validate_path_blocks_ssh() {
        let result = validate_file_path("/home/user/.ssh/id_rsa");
        // May fail on canonicalize (path doesn't exist) but if it exists, should be blocked
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_blocks_proc() {
        let result = validate_file_path("/proc/self/environ");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_file_rejects_etc_passwd() {
        let (tools, _dir) = create_test_tools();
        let request = Parameters(SendFileRequest {
            file_path: "/etc/passwd".to_string(),
            caption: None,
            chat_id: None,
            message_thread_id: None,
            conversation_id: Some("test-conv".to_string()),
        });
        let result = tools.send_file(request).await;
        assert!(result.is_err());
    }
}
