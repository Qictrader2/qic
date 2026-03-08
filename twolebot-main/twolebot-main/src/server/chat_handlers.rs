use crate::server::chat_ws::{ChatEvent, ChatEventHub};
use crate::storage::media::MediaStore;
use crate::storage::{
    ChatMetadataStore, MessageStore, PromptFeed, PromptItem, PromptSource, StoredMessage,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Shared state for web chat endpoints
#[derive(Clone)]
pub struct ChatState {
    pub prompt_feed: Arc<PromptFeed>,
    pub message_store: Arc<MessageStore>,
    pub media_store: Arc<MediaStore>,
    pub chat_metadata_store: Arc<ChatMetadataStore>,
    pub chat_event_hub: Arc<ChatEventHub>,
    pub data_dir: PathBuf,
    /// Default user_id for web messages (from config or first Telegram user)
    pub default_user_id: i64,
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: String,
    pub name: String,
    pub custom_name: Option<String>,
    pub auto_name: Option<String>,
    pub display_name: Option<String>,
    pub protocol: Option<String>,
    pub last_message_preview: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessagesQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub id: String,
    pub direction: String,
    pub content: String,
    pub timestamp: String,
    pub media_type: Option<String>,
    pub media_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadMediaRequest {
    pub conversation_id: String,
    /// Base64-encoded media data
    pub data: String,
    pub filename: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct UploadMediaResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub transcription: Option<String>,
    pub media_type: Option<String>,
    pub media_path: Option<String>,
    pub error: Option<String>,
}

// ============ Handlers ============

/// POST /api/chat/conversations — Create a new web conversation
pub async fn create_conversation(State(state): State<ChatState>) -> impl IntoResponse {
    let conversation_id = Uuid::new_v4().to_string();

    if let Err(e) = state.chat_metadata_store.upsert_full(
        &conversation_id,
        None,
        None,
        Some("New conversation"),
        Some("web"),
        None,
    ) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to create conversation: {e}") })),
        );
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "conversation_id": conversation_id,
            "created": true,
        })),
    )
}

/// GET /api/chat/conversations — List all conversations across all protocols
pub async fn list_conversations(State(state): State<ChatState>) -> impl IntoResponse {
    let metadata_result = {
        let store = state.chat_metadata_store.clone();
        tokio::task::spawn_blocking(move || store.list_all()).await
    };

    let metadata = match metadata_result {
        Ok(Ok(meta)) => meta,
        _ => Vec::new(),
    };

    let conversations: Vec<ConversationResponse> = metadata
        .into_iter()
        .map(|meta| {
            let is_web = meta.protocol.as_deref() == Some("web");
            let id = if is_web {
                meta.chat_id.clone()
            } else {
                match meta.topic_id {
                    Some(tid) => format!("{}_{}", meta.chat_id, tid),
                    None => meta.chat_id.clone(),
                }
            };

            // For Telegram topics with no distinct name, use topic ID or preview
            let name = if meta.effective_name() != "Untitled"
                && (is_web || meta.custom_name.is_some() || meta.auto_name.is_some())
            {
                meta.effective_name().to_string()
            } else if let Some(ref preview) = meta.last_message_preview {
                let truncated = if preview.len() > 40 {
                    format!("{}...", &preview[..37])
                } else {
                    preview.clone()
                };
                truncated
            } else if let Some(tid) = meta.topic_id {
                format!("Topic #{}", tid)
            } else {
                meta.effective_name().to_string()
            };

            ConversationResponse {
                id,
                name,
                custom_name: meta.custom_name,
                auto_name: meta.auto_name,
                display_name: meta.display_name,
                protocol: meta.protocol,
                last_message_preview: meta.last_message_preview,
                updated_at: meta.updated_at.to_rfc3339(),
            }
        })
        .collect();

    Json(serde_json::json!({ "conversations": conversations }))
}

/// PUT /api/chat/conversations/:id/name — Rename a conversation (sets custom_name)
pub async fn rename_conversation(
    State(state): State<ChatState>,
    Path(conversation_id): Path<String>,
    Json(request): Json<RenameRequest>,
) -> impl IntoResponse {
    let (chat_id, topic_id) = parse_conversation_id(&conversation_id);
    match state
        .chat_metadata_store
        .set_custom_name(chat_id, topic_id, &request.name)
    {
        Ok(()) => {
            // Broadcast rename event via SSE
            state
                .chat_event_hub
                .send(
                    &conversation_id,
                    ChatEvent::ConversationRenamed {
                        conversation_id: conversation_id.clone(),
                        name: request.name.clone(),
                    },
                )
                .await;
            (
                StatusCode::OK,
                Json(serde_json::json!({ "success": true, "name": request.name })),
            )
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Conversation not found: {e}") })),
        ),
    }
}

/// DELETE /api/chat/conversations/:id — Delete a conversation and all its data
pub async fn delete_conversation(
    State(state): State<ChatState>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    let (chat_id, topic_id) = parse_conversation_id(&conversation_id);

    // Delete media files FIRST (hardest to rollback if DB deletes succeed first)
    let _ = state.media_store.delete_chat(&conversation_id);

    // Delete messages
    let msgs_deleted = state
        .message_store
        .delete_by_chat(&conversation_id)
        .unwrap_or(0);

    // Delete metadata LAST
    let meta_deleted = state
        .chat_metadata_store
        .delete(chat_id, topic_id)
        .unwrap_or(false);

    if msgs_deleted > 0 || meta_deleted {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "messages_deleted": msgs_deleted,
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Conversation not found",
            })),
        )
    }
}

/// POST /api/chat/send — Send a message in a web conversation
pub async fn send_message(
    State(state): State<ChatState>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let message_id = Uuid::new_v4().to_string();

    // Store inbound message
    let stored = StoredMessage::inbound(
        &message_id,
        &request.conversation_id,
        state.default_user_id,
        &request.content,
    );
    if let Err(e) = state.message_store.store(stored) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SendMessageResponse {
                message_id: String::new(),
                status: format!("Failed to store message: {e}"),
            }),
        );
    }

    // Update conversation metadata
    let preview = if request.content.len() > 100 {
        format!("{}...", &request.content[..97])
    } else {
        request.content.clone()
    };
    let _ = state.chat_metadata_store.upsert_full(
        &request.conversation_id,
        None,
        None,
        None,
        Some("web"),
        Some(&preview),
    );

    // Auto-name from first message if still "New conversation"
    let auto_name = {
        let content = request.content.trim();
        let name = if content.len() > 50 {
            format!("{}...", &content[..47])
        } else {
            content.to_string()
        };
        name
    };
    let (chat_id, topic_id) = parse_conversation_id(&request.conversation_id);
    if state.chat_metadata_store.set_auto_name(chat_id, topic_id, &auto_name).is_ok() {
        // Check if name was actually set (it only sets if auto_name was NULL)
        if let Ok(Some(meta)) = state.chat_metadata_store.get(chat_id, topic_id) {
            if meta.auto_name.as_deref() == Some(&auto_name) && meta.custom_name.is_none() {
                // Broadcast rename via SSE so sidebar updates live
                state
                    .chat_event_hub
                    .send(
                        &request.conversation_id,
                        ChatEvent::ConversationRenamed {
                            conversation_id: request.conversation_id.clone(),
                            name: auto_name,
                        },
                    )
                    .await;
            }
        }
    }

    // Enqueue to PromptFeed for Claude processing
    let source = PromptSource::web(&request.conversation_id);
    let prompt_item = PromptItem::new(source, state.default_user_id, &request.content);
    if let Err(e) = state.prompt_feed.enqueue(prompt_item) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SendMessageResponse {
                message_id,
                status: format!("Failed to enqueue prompt: {e}"),
            }),
        );
    }

    // Send typing indicator
    state
        .chat_event_hub
        .send(
            &request.conversation_id,
            ChatEvent::TypingIndicator {
                conversation_id: request.conversation_id.clone(),
                is_typing: true,
            },
        )
        .await;

    (
        StatusCode::OK,
        Json(SendMessageResponse {
            message_id,
            status: "queued".to_string(),
        }),
    )
}

/// Parse a conversation ID into (chat_id, topic_id).
/// Telegram topics: "266340517_212854" → ("266340517", Some(212854))
/// Web/plain: "some-uuid" → ("some-uuid", None)
fn parse_conversation_id(conversation_id: &str) -> (&str, Option<i64>) {
    // Try splitting on underscore — if the right part parses as i64, it's a topic
    if let Some(pos) = conversation_id.rfind('_') {
        let (left, right) = conversation_id.split_at(pos);
        let right = &right[1..]; // skip the underscore
        if let Ok(topic_id) = right.parse::<i64>() {
            return (left, Some(topic_id));
        }
    }
    (conversation_id, None)
}

/// GET /api/chat/messages/:conversation_id — Get messages for a conversation (oldest first)
pub async fn get_messages(
    State(state): State<ChatState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
) -> impl IntoResponse {
    let (chat_id, topic_id) = parse_conversation_id(&conversation_id);
    let chat_id_str = chat_id.to_string();
    
    let messages_result = {
        let store = state.message_store.clone();
        let limit = query.limit;
        tokio::task::spawn_blocking(move || store.history_by_topic(&chat_id_str, topic_id, limit)).await
    };

    match messages_result {
        Ok(Ok(messages)) => {
            let responses: Vec<ChatMessageResponse> = messages
                .into_iter()
                .map(|msg| ChatMessageResponse {
                    id: msg.id,
                    direction: match msg.direction {
                        crate::storage::messages::MessageDirection::Inbound => "inbound".to_string(),
                        crate::storage::messages::MessageDirection::Outbound => {
                            "outbound".to_string()
                        }
                    },
                    content: msg.content,
                    timestamp: msg.timestamp.to_rfc3339(),
                    media_type: msg.media_type,
                    media_path: msg.media_path,
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({ "messages": responses })))
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to get messages: {e}") })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Spawn blocking failed: {e}") })),
        ),
    }
}

    /// POST /api/chat/upload — Upload media (voice/image/video/file) to a conversation
    pub async fn upload_media(
        State(state): State<ChatState>,
        Json(request): Json<UploadMediaRequest>,
    ) -> impl IntoResponse {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

        let conversation_id = request.conversation_id;
        let filename = request.filename;
        let mime_type = request.mime_type;
        
        let data = match BASE64.decode(&request.data) {
            Ok(bytes) => bytes,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(UploadMediaResponse {
                        success: false,
                        message_id: None,
                        transcription: None,
                        media_type: None,
                        media_path: None,
                        error: Some(format!("Invalid base64 data: {e}")),
                    }),
                )
            }
        };

    // Store the media file
    let media_filename = format!(
        "{}_{}",
        Uuid::new_v4().to_string().split('-').next().unwrap_or("media"),
        filename
    );
    if let Err(e) = state
        .media_store
        .store(&conversation_id, &media_filename, &data)
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadMediaResponse {
                success: false,
                message_id: None,
                transcription: None,
                media_type: None,
                media_path: None,
                error: Some(format!("Failed to store media: {e}")),
            }),
        );
    }

    let media_path = format!("{}/{}", conversation_id, media_filename);

    // Determine media type from mime
    let media_type = if mime_type.starts_with("audio/") {
        "voice"
    } else if mime_type.starts_with("video/") {
        "video"
    } else if mime_type.starts_with("image/") {
        "photo"
    } else {
        "document"
    };

    // Placeholder text — shown until transcription completes
    let placeholder_text = format!("[Attached {media_type}: {}]", filename);

    let message_id = Uuid::new_v4().to_string();

    // Store inbound message with media (content updated after transcription)
    let stored = StoredMessage::inbound(
        &message_id,
        &conversation_id,
        state.default_user_id,
        &placeholder_text,
    )
    .with_media(media_type, &media_path);

    if let Err(e) = state.message_store.store(stored) {
        tracing::warn!("Failed to store media message: {e}");
    }

    let media_path_for_response = media_path.clone();

    // Update conversation metadata with placeholder
    let _ = state.chat_metadata_store.upsert_full(
        &conversation_id,
        None,
        None,
        None,
        Some("web"),
        Some(&format!("[{media_type}]")),
    );

    // Spawn background: transcribe, update DB, notify frontend, THEN enqueue to LLM
    {
        let prompt_feed = state.prompt_feed.clone();
        let data_dir = state.data_dir.clone();
        let message_store = state.message_store.clone();
        let chat_metadata_store = state.chat_metadata_store.clone();
        let chat_event_hub = state.chat_event_hub.clone();
        let default_user_id = state.default_user_id;
        let bg_media_type = media_type.to_string();
        let bg_mime_type = mime_type.clone();
        let bg_message_id = message_id.clone();
        let bg_conversation_id = conversation_id.clone();
        let bg_media_path = media_path;
        let bg_placeholder = placeholder_text;
        tokio::spawn(async move {
            // 0. Notify frontend that transcription is starting
            chat_event_hub
                .send(
                    &bg_conversation_id,
                    ChatEvent::Transcribing {
                        conversation_id: bg_conversation_id.clone(),
                    },
                )
                .await;

            // 1. Transcribe (may return None if no Gemini key / unsupported type)
            let transcription =
                transcribe_media(&data_dir, &data, &bg_media_type, &bg_mime_type).await;

            // 2. Build the final prompt text the LLM will see
            let prompt_text = match &transcription {
                Some(text) => format!("[{bg_media_type}] {text}"),
                None => bg_placeholder,
            };

            // 3. Update stored message + metadata if we got a transcription
            if let Some(ref text) = transcription {
                let new_content = format!("[{}] {}", bg_media_type, text);
                if let Err(e) = message_store.update_content(&bg_message_id, &new_content) {
                    tracing::warn!("Background transcription DB update failed: {e}");
                }
                let preview = if text.len() > 100 {
                    format!("[{}] {}...", bg_media_type, &text[..97])
                } else {
                    format!("[{}] {}", bg_media_type, text)
                };
                let _ = chat_metadata_store.upsert_full(
                    &bg_conversation_id,
                    None,
                    None,
                    None,
                    Some("web"),
                    Some(&preview),
                );

                // 4. Notify frontend so it can replace the placeholder live
                chat_event_hub
                    .send(
                        &bg_conversation_id,
                        ChatEvent::MessageUpdated {
                            conversation_id: bg_conversation_id.clone(),
                            message_id: bg_message_id.clone(),
                            content: new_content,
                        },
                    )
                    .await;

                tracing::info!(
                    "Background transcription done for message {}",
                    bg_message_id
                );
            }

            // 5. Enqueue to LLM with real transcription (or placeholder if none)
            let source = PromptSource::web(&bg_conversation_id);
            let mut prompt_item = PromptItem::new(source, default_user_id, &prompt_text);
            prompt_item.media_path = Some(bg_media_path);
            if let Err(e) = prompt_feed.enqueue(prompt_item) {
                tracing::error!("Failed to enqueue transcribed prompt: {e}");
                return;
            }

            // 6. Typing indicator after enqueue so frontend knows agent is working
            chat_event_hub
                .send(
                    &bg_conversation_id,
                    ChatEvent::TypingIndicator {
                        conversation_id: bg_conversation_id.clone(),
                        is_typing: true,
                    },
                )
                .await;
        });
    }

    (
        StatusCode::OK,
        Json(UploadMediaResponse {
            success: true,
            message_id: Some(message_id),
            transcription: None,
            media_type: Some(media_type.to_string()),
            media_path: Some(media_path_for_response),
            error: None,
        }),
    )
}

/// Transcribe media via Gemini (if available). Returns None on failure.
async fn transcribe_media(
    data_dir: &std::path::Path,
    data: &[u8],
    media_type: &str,
    mime_type: &str,
) -> Option<String> {
    use crate::storage::SecretsStore;
    use crate::transcription::gemini::GeminiTranscriber;

    let secrets = SecretsStore::new(data_dir.join("runtime.sqlite3")).ok()?;
    let key = secrets.get_gemini_key().ok()??;
    let transcriber = GeminiTranscriber::new(key).ok()?;

    match transcriber
        .transcribe_with_mime(data, media_type, mime_type)
        .await
    {
        Ok(text) => Some(text),
        Err(e) => {
            tracing::warn!("Media transcription failed: {e}");
            None
        }
    }
}
