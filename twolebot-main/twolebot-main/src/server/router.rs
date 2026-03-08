use crate::mcp::server::{mcp_handler, McpHttpState};
use crate::server::auth::{self, AuthState};
use crate::server::chat_handlers::{self as web_chat, ChatState};
use crate::server::chat_ws::{self as chat_ws, ChatWsState};
use crate::server::handlers::{self, AppState, SemanticState, TunnelState};
use crate::server::setup::{self, SetupState};
use crate::server::sse::{self, SseState};
use crate::server::voice_handlers::{self, VoiceState};
use crate::server::work_handlers::{self, WorkState};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    middleware,
    routing::{any, get, post, put},
    Router,
};
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

/// Router configuration options
#[derive(Clone)]
pub struct RouterConfig {
    /// Allow CORS from any origin
    pub cors_allow_all: bool,
    /// Host for same-origin CORS (e.g., "127.0.0.1")
    pub host: String,
    /// Port for same-origin CORS
    pub port: u16,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            cors_allow_all: true,
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

/// Builder for constructing the Axum router with optional subsystems.
pub struct RouterBuilder {
    state: AppState,
    config: RouterConfig,
    static_dir: Option<PathBuf>,
    mcp_state: Option<McpHttpState>,
    setup_state: Option<SetupState>,
    semantic_state: Option<SemanticState>,
    work_state: Option<WorkState>,
    sse_state: Option<SseState>,
    voice_state: Option<VoiceState>,
    auth_state: Option<AuthState>,
    chat_state: Option<ChatState>,
    chat_ws_state: Option<ChatWsState>,
    tunnel_state: Option<TunnelState>,
}

impl RouterBuilder {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            config: RouterConfig::default(),
            static_dir: None,
            mcp_state: None,
            setup_state: None,
            semantic_state: None,
            work_state: None,
            sse_state: None,
            voice_state: None,
            auth_state: None,
            chat_state: None,
            chat_ws_state: None,
            tunnel_state: None,
        }
    }

    pub fn config(mut self, config: RouterConfig) -> Self {
        self.config = config;
        self
    }

    pub fn static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }

    pub fn mcp(mut self, state: McpHttpState) -> Self {
        self.mcp_state = Some(state);
        self
    }

    pub fn setup(mut self, state: SetupState) -> Self {
        self.setup_state = Some(state);
        self
    }

    pub fn semantic(mut self, state: SemanticState) -> Self {
        self.semantic_state = Some(state);
        self
    }

    pub fn work(mut self, state: WorkState) -> Self {
        self.work_state = Some(state);
        self
    }

    pub fn sse(mut self, state: SseState) -> Self {
        self.sse_state = Some(state);
        self
    }

    pub fn voice(mut self, state: VoiceState) -> Self {
        self.voice_state = Some(state);
        self
    }

    pub fn auth(mut self, state: AuthState) -> Self {
        self.auth_state = Some(state);
        self
    }

    pub fn chat(mut self, state: ChatState, ws_state: ChatWsState) -> Self {
        self.chat_state = Some(state);
        self.chat_ws_state = Some(ws_state);
        self
    }

    pub fn tunnel(mut self, state: TunnelState) -> Self {
        self.tunnel_state = Some(state);
        self
    }

    pub fn build(self) -> Router {
        // Build CORS layer based on config
        let cors = if self.config.cors_allow_all {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            let origin = format!("http://{}:{}", self.config.host, self.config.port);
            let origin_value = origin.parse::<header::HeaderValue>().unwrap_or_else(|e| {
                tracing::warn!(
                    "Invalid CORS origin '{}': {}. Falling back to http://127.0.0.1:8080",
                    origin,
                    e
                );
                header::HeaderValue::from_static("http://127.0.0.1:8080")
            });
            CorsLayer::new()
                .allow_origin(origin_value)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE])
        };

        // Cron management routes
        let cron_router = Router::new()
            .route("/status", get(handlers::get_cron_status))
            .route(
                "/jobs",
                get(handlers::list_cron_jobs).post(handlers::create_cron_job),
            )
            .route(
                "/jobs/:job_id",
                get(handlers::get_cron_job).delete(handlers::cancel_cron_job),
            )
            .route("/jobs/:job_id/pause", post(handlers::pause_cron_job))
            .route("/jobs/:job_id/resume", post(handlers::resume_cron_job))
            .route("/jobs/:job_id/snooze", post(handlers::snooze_cron_job));

        // API routes
        let api_router = Router::new()
            .route("/feed", get(handlers::get_feed))
            .route("/responses", get(handlers::get_responses))
            .route("/chats", get(handlers::list_chats))
            .route("/messages/:chat_id", get(handlers::get_messages))
            .route("/media/:chat_id/:filename", get(handlers::get_media))
            .route("/logs", get(handlers::get_logs))
            .route(
                "/settings",
                get(handlers::get_settings).put(handlers::update_settings),
            )
            .nest("/cron", cron_router)
            .route("/status", get(handlers::get_status));

        let mut router = Router::new()
            .nest("/api", api_router)
            .with_state(self.state);

        // Add setup routes if setup state is provided (protected by CORS)
        if let Some(setup_state) = self.setup_state {
            let setup_router = Router::new()
                .route("/status", get(setup::get_setup_status))
                .route("/telegram", post(setup::setup_telegram))
                .route("/gemini", post(setup::setup_gemini))
                .route("/install-claude", post(setup::setup_install_claude))
                .route("/update-claude", post(setup::setup_update_claude))
                .route("/claude-auth", get(setup::check_claude_auth))
                .route("/test-claude", post(setup::setup_test_claude))
                .route("/check-threading", post(setup::check_threading))
                .route("/complete", post(setup::setup_complete))
                .route(
                    "/api-keys",
                    get(setup::get_api_keys).put(setup::update_api_keys),
                )
                .with_state(setup_state);

            router = router.nest("/api/setup", setup_router);
        }

        // Add unified MCP endpoint (all tools: cron, memory, conversations)
        if let Some(mcp_state) = self.mcp_state {
            let mcp_router = Router::new()
                .route("/", any(mcp_handler))
                // Back-compat aliases for existing docs/CLI configs.
                .route("/memory", any(mcp_handler))
                .route("/conversations", any(mcp_handler))
                .with_state(mcp_state);
            router = router.nest("/mcp", mcp_router);
            tracing::info!("MCP server enabled at /mcp");
        }

        // Add semantic indexer endpoints if available
        if let Some(semantic_state) = self.semantic_state {
            let semantic_router = Router::new()
                .route("/status", get(handlers::get_semantic_status))
                .route("/toggle", post(handlers::toggle_semantic))
                .route("/reindex", post(handlers::trigger_semantic_reindex))
                .with_state(semantic_state);
            router = router.nest("/api/semantic", semantic_router);
        }

        // Add work endpoints if available
        if let Some(work_state) = self.work_state {
            let work_router = Router::new()
                .route("/projects/list", post(work_handlers::list_projects))
                .route("/projects/get", post(work_handlers::get_project))
                .route("/projects/create", post(work_handlers::create_project))
                .route("/projects/update", post(work_handlers::update_project))
                .route("/tasks/list", post(work_handlers::list_tasks))
                .route("/tasks/get", post(work_handlers::get_task))
                .route("/tasks/create", post(work_handlers::create_task))
                .route("/tasks/update", post(work_handlers::update_task))
                .route("/tasks/take-next", post(work_handlers::take_next_task))
                .route(
                    "/tasks/take-next-review",
                    post(work_handlers::take_next_review_task),
                )
                .route("/tasks/move", post(work_handlers::move_task))
                .route("/tasks/reject-review", post(work_handlers::reject_review))
                .route("/tasks/analytics", post(work_handlers::task_analytics))
                .route("/documents/search", post(work_handlers::search_documents))
                .route("/documents/get", post(work_handlers::get_document))
                .route("/documents/create", post(work_handlers::create_document))
                .route("/documents/update", post(work_handlers::update_document))
                .route("/comments/list", post(work_handlers::list_comments))
                .route("/comments/upsert", post(work_handlers::upsert_comment))
                .route("/activity/recent", post(work_handlers::recent_activity))
                // Live board routes
                .route("/live-board/get", post(work_handlers::get_live_board))
                .route("/live-board/select", post(work_handlers::select_tasks))
                .route("/live-board/deselect", post(work_handlers::deselect_task))
                .route(
                    "/live-board/move",
                    post(work_handlers::move_selection),
                )
                .route(
                    "/live-board/clear-completed",
                    post(work_handlers::clear_completed),
                )
                .route(
                    "/live-board/agent/start",
                    post(work_handlers::start_agent_loop),
                )
                .route(
                    "/live-board/agent/stop",
                    post(work_handlers::stop_agent_loop),
                )
                .route(
                    "/live-board/agent/ensure",
                    post(work_handlers::ensure_agent_loop),
                )
                .with_state(work_state);
            router = router.nest("/api/work", work_router);
            tracing::info!("Work endpoints enabled at /api/work (with live-board)");
        }

        // Add voice endpoints if available
        if let Some(voice_state) = self.voice_state {
            let voice_router = Router::new()
                .route("/transcribe", post(voice_handlers::transcribe_audio)
                    .layer(DefaultBodyLimit::max(50 * 1024 * 1024)))
                .route("/format", post(voice_handlers::format_transcription))
                .with_state(voice_state);
            router = router.nest("/api/voice", voice_router);
            tracing::info!("Voice endpoints enabled at /api/voice");
        }

        // Add web chat endpoints
        if let Some(chat_state) = self.chat_state {
            let chat_router = Router::new()
                .route(
                    "/conversations",
                    get(web_chat::list_conversations).post(web_chat::create_conversation),
                )
                .route(
                    "/conversations/:id",
                    axum::routing::delete(web_chat::delete_conversation),
                )
                .route(
                    "/conversations/:id/name",
                    put(web_chat::rename_conversation),
                )
                .route("/send", post(web_chat::send_message))
                .route("/upload", post(web_chat::upload_media)
                    .layer(DefaultBodyLimit::max(50 * 1024 * 1024)))
                .route("/messages/:conversation_id", get(web_chat::get_messages))
                .with_state(chat_state);
            router = router.nest("/api/chat", chat_router);
            tracing::info!("Web chat endpoints enabled at /api/chat");
        }

        // Add chat WebSocket endpoint
        if let Some(chat_ws_state) = self.chat_ws_state {
            let chat_ws_router = Router::new()
                .route("/ws/:conversation_id", get(chat_ws::chat_ws))
                .with_state(chat_ws_state);
            router = router.nest("/api/chat", chat_ws_router);
            tracing::info!("Chat WebSocket endpoint enabled at /api/chat/ws");
        }

        // Add SSE endpoint for work events
        if let Some(sse_state) = self.sse_state {
            let sse_router = Router::new()
                .route("/events", get(sse::work_events))
                .with_state(sse_state);
            router = router.nest("/api/work", sse_router);
            tracing::info!("SSE endpoint enabled at /api/work/events");
        }

        // Add tunnel status endpoint (behind auth middleware)
        if let Some(tunnel_state) = self.tunnel_state {
            let tunnel_router = Router::new()
                .route("/status", get(handlers::get_tunnel_status))
                .with_state(tunnel_state);
            router = router.nest("/api/tunnel", tunnel_router);
        }

        // Apply auth middleware to all API/MCP routes (localhost exempt, external needs session)
        if let Some(auth_state) = self.auth_state {
            router = router.layer(middleware::from_fn_with_state(
                auth_state.clone(),
                auth::auth_middleware,
            ));

            // Auth routes sit outside the middleware — always accessible
            let auth_router = Router::new()
                .route("/login", get(auth::login_page).post(auth::login_submit))
                .route("/qr", get(auth::qr_login))
                .route("/token", get(auth::token_login))
                .with_state(auth_state);
            router = router.nest("/auth", auth_router);
        }

        router = router.layer(cors);

        // Serve static files if directory provided, with SPA fallback to index.html
        if let Some(static_dir) = self.static_dir {
            if static_dir.exists() {
                let index_path = static_dir.join("index.html");
                let serve_dir = ServeDir::new(&static_dir).fallback(ServeFile::new(index_path));
                router = router.fallback_service(serve_dir);
            }
        }

        router
    }
}

/// Create the Axum router with all routes (simple convenience wrapper)
pub fn create_router(
    state: AppState,
    mcp_state: Option<McpHttpState>,
    static_dir: Option<PathBuf>,
) -> Router {
    let mut builder = RouterBuilder::new(state);
    if let Some(s) = mcp_state {
        builder = builder.mcp(s);
    }
    if let Some(d) = static_dir {
        builder = builder.static_dir(d);
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::CronFeed;
    use crate::logging::SharedLogger;
    use crate::storage::{
        media::MediaStore, ChatMetadataStore, MessageStore, PromptFeed, ResponseFeed, SettingsStore,
    };
    use std::sync::Arc;
    use tempfile::tempdir;

    fn test_app_state() -> AppState {
        let dir = tempdir().unwrap();
        let db = dir.path().join("runtime.sqlite3");
        AppState {
            prompt_feed: Arc::new(PromptFeed::new(&db).unwrap()),
            response_feed: Arc::new(ResponseFeed::new(&db).unwrap()),
            message_store: Arc::new(MessageStore::new(&db).unwrap()),
            media_store: Arc::new(MediaStore::new(dir.path().join("media")).unwrap()),
            cron_feed: Arc::new(CronFeed::new(&db).unwrap()),
            settings_store: Arc::new(SettingsStore::new(&db).unwrap()),
            chat_metadata_store: Arc::new(ChatMetadataStore::new(&db).unwrap()),
            logger: SharedLogger::new(dir.path().join("logs.jsonl")).unwrap(),
            data_dir: dir.path().to_path_buf(),
        }
    }

    #[test]
    fn test_router_creation() {
        let state = test_app_state();
        let _router = create_router(state, None, None);
    }

    #[test]
    fn test_router_with_custom_config() {
        let state = test_app_state();

        let config = RouterConfig {
            cors_allow_all: false,
            host: "127.0.0.1".to_string(),
            port: 8080,
        };

        let _router = RouterBuilder::new(state)
            .config(config)
            .build();
    }
}
