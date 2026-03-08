use crate::cron::{service as cron_service, CronFeed, CronJob};
use crate::logging::SharedLogger;
use crate::semantic::{IndexerActivity, IndexerStatus, SharedStatus};
use crate::storage::media::{mime_for_extension, MediaStore};
use crate::storage::{
    ChatMetadataStore, MessageStore, PromptFeed, ResponseFeed, Settings, SettingsStore,
};
use crate::types::api::{
    ChatSummary, ChatsResponse, CronJobSummary, CronJobsQuery, CronJobsResponse,
    CronStatusResponse, FeedResponse, LogsQuery, LogsResponse, MessagesQuery, MessagesResponse,
    ResponseFeedResponse, SnoozeRequest, StatusResponse,
};
use crate::types::cron::ScheduleJobRequest;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::io::ReaderStream;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub prompt_feed: Arc<PromptFeed>,
    pub response_feed: Arc<ResponseFeed>,
    pub message_store: Arc<MessageStore>,
    pub media_store: Arc<MediaStore>,
    pub cron_feed: Arc<CronFeed>,
    pub settings_store: Arc<SettingsStore>,
    pub chat_metadata_store: Arc<ChatMetadataStore>,
    pub logger: SharedLogger,
    pub data_dir: std::path::PathBuf,
}

/// State for semantic indexer endpoints
#[derive(Clone)]
pub struct SemanticState {
    pub status: SharedStatus,
    /// None when semantic was disabled at CLI level (toggle not available)
    pub paused: Option<Arc<AtomicBool>>,
    pub settings_store: Arc<SettingsStore>,
    /// Notification handle to trigger immediate conversation reindex
    pub conversation_notify: Option<Arc<Notify>>,
}

// ============ Feed Handlers ============

/// GET /api/feed - Get prompt feed status
pub async fn get_feed(State(state): State<AppState>) -> impl IntoResponse {
    let pending: Vec<serde_json::Value> = state
        .prompt_feed
        .all_pending()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| serde_json::to_value(p).ok())
        .collect();

    let pending_count = state.prompt_feed.pending_count();

    let running = state
        .prompt_feed
        .get_running()
        .ok()
        .flatten()
        .and_then(|p| serde_json::to_value(p).ok());

    let recent_completed = state
        .prompt_feed
        .recent_completed(20)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| serde_json::to_value(p).ok())
        .collect();

    let completed_count = state.prompt_feed.completed_count();

    Json(FeedResponse {
        pending,
        pending_count,
        running,
        recent_completed,
        completed_count,
    })
}

/// GET /api/responses - Get response feed status
pub async fn get_responses(State(state): State<AppState>) -> impl IntoResponse {
    let pending = state
        .response_feed
        .all_pending()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| serde_json::to_value(r).ok())
        .collect();

    let pending_count = state.response_feed.pending_count();

    let recent_sent = state
        .response_feed
        .recent_sent(20)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| serde_json::to_value(r).ok())
        .collect();

    let sent_count = state.response_feed.sent_count();

    let recent_failed = state
        .response_feed
        .recent_failed(20)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| serde_json::to_value(r).ok())
        .collect();

    let failed_count = state.response_feed.failed_count();

    Json(ResponseFeedResponse {
        pending,
        pending_count,
        recent_sent,
        sent_count,
        recent_failed,
        failed_count,
    })
}

// ============ Message Handlers ============

/// GET /api/chats - List all chats with metadata
pub async fn list_chats(State(state): State<AppState>) -> impl IntoResponse {
    let chats_result = {
        let store = state.message_store.clone();
        tokio::task::spawn_blocking(move || store.list_chats()).await
    };

    let chats = match chats_result {
        Ok(Ok(chats)) => chats,
        _ => Vec::new(),
    };

    let mut chats_summary = Vec::with_capacity(chats.len());
    
    for (chat_id, topic_id, message_count) in chats {
        let meta_result = {
            let store = state.chat_metadata_store.clone();
            let cid = chat_id.clone();
            tokio::task::spawn_blocking(move || store.get(&cid, topic_id)).await
        };
        let meta = match meta_result {
            Ok(Ok(Some(m))) => Some(m),
            _ => None,
        };

        chats_summary.push(ChatSummary {
            chat_id,
            topic_id,
            username: meta.as_ref().and_then(|m| m.username.clone()),
            display_name: meta.as_ref().and_then(|m| m.display_name.clone()),
            message_count,
        });
    }

    Json(ChatsResponse { chats: chats_summary })
}

/// GET /api/messages/:chat_id - Get messages for a chat with pagination and search
pub async fn get_messages(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> impl IntoResponse {
    // Get all messages for this chat (already newest first)
    let all_messages_result = {
        let store = state.message_store.clone();
        let cid = chat_id.clone();
        tokio::task::spawn_blocking(move || store.list(&cid, 10000)).await
    };
    
    let all_messages = match all_messages_result {
        Ok(Ok(msgs)) => msgs,
        _ => Vec::new(),
    };

    // Filter by topic_id if provided
    let topic_filtered: Vec<_> = match &query.topic_id {
        Some(tid) if tid == "none" => {
            all_messages.into_iter().filter(|m| m.topic_id.is_none()).collect()
        }
        Some(tid) => {
            if let Ok(tid_num) = tid.parse::<i64>() {
                all_messages.into_iter().filter(|m| m.topic_id == Some(tid_num)).collect()
            } else {
                all_messages
            }
        }
        None => all_messages,
    };

    // Filter by search term if provided
    let filtered: Vec<_> = if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        topic_filtered
            .into_iter()
            .filter(|m| m.content.to_lowercase().contains(&search_lower))
            .collect()
    } else {
        topic_filtered
    };

    let total = filtered.len();
    let total_pages = if total == 0 {
        1
    } else {
        (total + query.page_size - 1) / query.page_size.max(1)
    };

    // Paginate
    let start = query.page * query.page_size;
    let messages: Vec<serde_json::Value> = filtered
        .into_iter()
        .skip(start)
        .take(query.page_size)
        .filter_map(|m| serde_json::to_value(m).ok())
        .collect();

    Json(MessagesResponse {
        messages,
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    })
}

// ============ Media Handler ============

/// GET /api/media/:chat_id/:filename - Serve media files
pub async fn get_media(
    State(state): State<AppState>,
    Path((chat_id, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    // Validate path to prevent directory traversal attacks
    let media_path = match state.media_store.safe_media_path(&chat_id, &filename) {
        Ok(p) => p,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid path")),
    };

    if !media_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Media not found"));
    }

    // Get MIME type from extension
    let extension = media_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let mime_type = mime_for_extension(extension);

    // Open file for streaming
    let file = match tokio::fs::File::open(&media_path).await {
        Ok(f) => f,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file")),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CACHE_CONTROL, "public, max-age=31536000"),
        ],
        body,
    ))
}

// ============ Log Handlers ============

/// GET /api/logs - Get log entries with search and pagination
pub async fn get_logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    // Get all logs (already newest first)
    let all_entries = state.logger.all();

    // Filter by search term if provided
    let filtered: Vec<_> = if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        all_entries
            .into_iter()
            .filter(|e| {
                e.message.to_lowercase().contains(&search_lower)
                    || e.component.to_lowercase().contains(&search_lower)
            })
            .collect()
    } else {
        all_entries
    };

    // Filter by level if provided
    let filtered: Vec<_> = if let Some(ref level) = query.level {
        let level_lower = level.to_lowercase();
        filtered
            .into_iter()
            .filter(|e| {
                let entry_level = match e.level {
                    crate::logging::LogLevel::Debug => "debug",
                    crate::logging::LogLevel::Info => "info",
                    crate::logging::LogLevel::Warn => "warn",
                    crate::logging::LogLevel::Error => "error",
                };
                entry_level == level_lower
            })
            .collect()
    } else {
        filtered
    };

    let total = filtered.len();
    let total_pages = (total + query.page_size - 1) / query.page_size.max(1);

    // Paginate
    let start = query.page * query.page_size;
    let entries: Vec<serde_json::Value> = filtered
        .into_iter()
        .skip(start)
        .take(query.page_size)
        .filter_map(|e| serde_json::to_value(e).ok())
        .collect();

    Json(LogsResponse {
        entries,
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    })
}

// ============ Config/Status Handlers ============

/// GET /api/status - Get server status
pub async fn get_status() -> impl IntoResponse {
    Json(StatusResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ============ Cron Handlers ============

impl From<CronJob> for CronJobSummary {
    fn from(job: CronJob) -> Self {
        Self {
            id: job.id,
            name: job.name,
            schedule: job.schedule.description(),
            status: format!("{:?}", job.status).to_lowercase(),
            next_run: job.next_run.map(|t| t.to_rfc3339()),
            last_run: job.last_run.map(|t| t.to_rfc3339()),
            created_at: job.created_at.to_rfc3339(),
        }
    }
}

/// Helper: convert a cron job Result into a JSON response.
fn cron_job_response(result: crate::error::Result<CronJob>) -> axum::response::Response {
    match result {
        Ok(job) => {
            let summary: CronJobSummary = job.into();
            (StatusCode::OK, Json(summary)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// GET /api/cron/jobs - List cron jobs
pub async fn list_cron_jobs(
    State(state): State<AppState>,
    Query(query): Query<CronJobsQuery>,
) -> impl IntoResponse {
    match cron_service::list_jobs(state.cron_feed.as_ref(), query.status.as_str()) {
        Ok(jobs) => {
            let summaries: Vec<CronJobSummary> = jobs.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(CronJobsResponse { jobs: summaries })).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// POST /api/cron/jobs - Create a new cron job
pub async fn create_cron_job(
    State(state): State<AppState>,
    Json(request): Json<ScheduleJobRequest>,
) -> impl IntoResponse {
    match cron_service::schedule_job(state.cron_feed.as_ref(), request) {
        Ok(created) => {
            let summary: CronJobSummary = created.into();
            (StatusCode::CREATED, Json(summary)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// GET /api/cron/jobs/:id - Get a specific cron job
pub async fn get_cron_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match cron_service::get_job(state.cron_feed.as_ref(), &job_id) {
        Ok(Some(job)) => {
            let summary: CronJobSummary = job.into();
            (StatusCode::OK, Json(summary)).into_response()
        }
        Ok(None) => crate::TwolebotError::not_found("Job not found").into_response(),
        Err(e) => e.into_response(),
    }
}

/// DELETE /api/cron/jobs/:id - Cancel a cron job
pub async fn cancel_cron_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    cron_job_response(
        cron_service::cancel_job(state.cron_feed.as_ref(), &job_id).map(|(job, _)| job),
    )
}

/// POST /api/cron/jobs/:id/pause - Pause a cron job
pub async fn pause_cron_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    cron_job_response(cron_service::pause_job(state.cron_feed.as_ref(), &job_id))
}

/// POST /api/cron/jobs/:id/resume - Resume a paused cron job
pub async fn resume_cron_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    cron_job_response(cron_service::resume_job(state.cron_feed.as_ref(), &job_id))
}

/// POST /api/cron/jobs/:id/snooze - Snooze a job
pub async fn snooze_cron_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<SnoozeRequest>,
) -> impl IntoResponse {
    cron_job_response(cron_service::snooze_job(
        state.cron_feed.as_ref(),
        &job_id,
        request.minutes,
    ))
}

/// GET /api/cron/status - Get cron system status
pub async fn get_cron_status(State(state): State<AppState>) -> impl IntoResponse {
    let active_jobs = state
        .cron_feed
        .list_active_jobs()
        .map(|j| j.len())
        .unwrap_or(0);
    let paused_jobs = state
        .cron_feed
        .list_paused_jobs()
        .map(|j| j.len())
        .unwrap_or(0);
    let waiting_executions = state.cron_feed.list_waiting().map(|e| e.len()).unwrap_or(0);

    Json(CronStatusResponse {
        active_jobs,
        paused_jobs,
        waiting_executions,
    })
}

// ============ Settings Handlers ============

/// GET /api/settings - Get current settings
pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let settings = state.settings_store.get();
    Json(settings)
}

/// PUT /api/settings - Update settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(mut settings): Json<Settings>,
) -> impl IntoResponse {
    // Preserve fields managed by other endpoints
    let existing = state.settings_store.get();
    settings.semantic_paused = existing.semantic_paused;
    settings.threading_enabled = existing.threading_enabled;

    match state.settings_store.update(settings) {
        Ok(_) => {
            let updated = state.settings_store.get();
            match serde_json::to_value(updated) {
                Ok(value) => (StatusCode::OK, Json(value)),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Serialization error: {}", e)})),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

// ============ Semantic Indexer Handlers ============

/// GET /api/semantic/status - Get semantic indexer status
pub async fn get_semantic_status(State(state): State<SemanticState>) -> impl IntoResponse {
    let status = state.status.read().await;
    Json::<IndexerStatus>((*status).clone())
}

#[derive(Deserialize)]
pub struct ToggleRequest {
    pub enabled: bool,
}

/// POST /api/semantic/toggle - Toggle semantic indexer on/off
pub async fn toggle_semantic(
    State(state): State<SemanticState>,
    Json(request): Json<ToggleRequest>,
) -> impl IntoResponse {
    let paused = !request.enabled;

    // Update the AtomicBool (if indexer was started)
    if let Some(ref paused_flag) = state.paused {
        paused_flag.store(paused, Ordering::Relaxed);
    } else {
        // Indexer was disabled at CLI level, can't toggle
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Semantic indexer was disabled at startup (use --disable-semantic flag)"})),
        ).into_response();
    }

    // Update the status
    {
        let mut status = state.status.write().await;
        status.enabled = request.enabled;
        if request.enabled {
            status.memory.activity = IndexerActivity::Idle;
            status.conversations.activity = IndexerActivity::Idle;
        } else {
            status.memory.activity = IndexerActivity::Paused;
            status.conversations.activity = IndexerActivity::Paused;
            status.memory.current_file = None;
            status.conversations.current_file = None;
        }
    }

    // Persist to settings
    if let Err(e) = state.settings_store.set_semantic_paused(paused) {
        tracing::warn!(error = %e, "Failed to persist semantic_paused setting");
    }

    // Return updated status
    let status = state.status.read().await;
    (
        StatusCode::OK,
        Json(serde_json::to_value(&*status).unwrap_or_default()),
    )
        .into_response()
}

/// POST /api/semantic/reindex - Trigger immediate conversation reindex
pub async fn trigger_semantic_reindex(State(state): State<SemanticState>) -> impl IntoResponse {
    // Check if paused
    if let Some(ref paused_flag) = state.paused {
        if paused_flag.load(Ordering::Relaxed) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Semantic indexer is paused"})),
            )
                .into_response();
        }
    }

    match state.conversation_notify {
        Some(ref notify) => {
            notify.notify_one();
            tracing::info!("Manual conversation reindex triggered via API");
            (StatusCode::OK, Json(serde_json::json!({"success": true}))).into_response()
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Semantic indexer not available"})),
        )
            .into_response(),
    }
}

// ============ Tunnel Handlers ============

/// State for the Cloudflare tunnel status endpoint.
#[derive(Clone)]
pub struct TunnelState {
    pub url_rx: tokio::sync::watch::Receiver<Option<String>>,
    pub nonce_store: crate::server::auth::NonceStore,
}

/// GET /api/tunnel/status - Get tunnel URL and QR code SVG
/// Generates a fresh one-time nonce for each request (nonces expire after 10 minutes).
pub async fn get_tunnel_status(State(state): State<TunnelState>) -> impl IntoResponse {
    let url = state.url_rx.borrow().clone();
    let qr_svg = match url.as_ref() {
        Some(u) => {
            let nonce = state.nonce_store.generate().await;
            let login_url = format!("{}/auth/qr?nonce={}", u, nonce);
            crate::tunnel::generate_qr_svg(&login_url).ok()
        }
        None => None,
    };

    Json(serde_json::json!({
        "active": url.is_some(),
        "url": url,
        "qr_svg": qr_svg,
    }))
}

// ============ Error Handling ============

pub async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "Not found"})),
    )
}
