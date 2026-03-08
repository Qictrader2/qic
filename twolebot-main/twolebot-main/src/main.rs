#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tokio_util::sync::CancellationToken;
use twolebot::{
    claude::ClaudeManager,
    config::{Args, Command, Config, SetupStatus},
    cron::{ActivityTracker, CronFeed, CronGatekeeper, CronScheduler},
    dispatcher::ResponseBroadcaster,
    logging::SharedLogger,
    mcp::{ImageTools, McpHttpState, SendTools, TwolebotMcpServer},
    semantic::{disabled_status, IndexerConfig, SemanticIndexer},
    server::{
        handlers::{AppState, SemanticState, TunnelState},
        AuthState, ChatEventHub, ChatWsState, ChatState, RouterBuilder, RouterConfig, SetupState,
        SseState, VoiceState, WorkState,
    },
    storage::{
        ActiveChatRegistry, ChatMetadataStore, CronTopicStore, MainTopicStore, MediaStore,
        MessageStore, PromptFeed, ResponseFeed, SecretsStore, SessionStore, SettingsStore,
    },
    telegram::{process_update, TelegramPoller, TelegramSender, TypingIndicator, Update},
    transcription::GeminiTranscriber,
    tunnel,
    work::{WorkApp, WorkDb},
};

const POLL_INTERVAL_MS: u64 = 100;
const CRON_POLL_INTERVAL_MS: u64 = 10_000; // 10 seconds for cron scheduler/gatekeeper

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Strip Claude Code env vars so spawned `claude -p` processes don't
    // think they're nested sessions and refuse to run.
    std::env::remove_var("CLAUDECODE");
    std::env::remove_var("CLAUDE_CODE_ENTRYPOINT");

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse CLI args
    let args = Args::parse();

    // Handle subcommands
    match args.command {
        Some(Command::Status) => {
            return run_status_command(&args);
        }
        Some(Command::McpStdio) => {
            return run_mcp_stdio(&args).await;
        }
        Some(Command::Run) | None => {
            // Check if setup is needed
            if Config::needs_setup(&args) {
                return run_setup_mode(&args).await;
            }
            // Otherwise run normally
        }
    }

    // Migrate databases from old layout to new flat layout
    migrate_db_layout(&args);

    let config = Config::from_args(&args)?;

    // Ensure auth token exists (generate on first run, show token only on first generation)
    {
        use twolebot::storage::SecretsStore;
        let secrets = SecretsStore::new(&config.general_db_path)?;
        let (auth_token, newly_generated) = secrets.ensure_auth_token()?;
        if newly_generated {
            println!();
            println!("  Dashboard auth token: {}", auth_token);
            println!("  (use this to log in from external devices)");
            println!("  Run `twolebot status` to see it again.");
            println!();
        }
    }

    tracing::info!("Starting twolebot v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Data: {:?}", config.data_dir);
    tracing::info!("HTTP server: {}:{}", config.host, config.port);

    // Create root cancellation token for graceful shutdown
    let shutdown_token = CancellationToken::new();

    // Initialize components
    let logger = SharedLogger::new(&config.logs_file)?;
    logger.info("main", "Twolebot starting up");

    let prompt_feed = Arc::new(PromptFeed::new(&config.general_db_path)?);

    // Recover any orphaned prompts from previous crash/restart
    match prompt_feed.recover_orphaned_running() {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Recovered {} orphaned prompts from previous run", count);
                logger.info("main", format!("Recovered {} orphaned prompts", count));
            }
        }
        Err(e) => tracing::warn!("Failed to recover orphaned prompts: {}", e),
    }

    let response_feed = Arc::new(ResponseFeed::new(&config.general_db_path)?);
    let message_store = Arc::new(MessageStore::new(&config.general_db_path)?);
    let media_store = Arc::new(MediaStore::new(&config.media_dir)?);
    let settings_store = Arc::new(SettingsStore::new(&config.general_db_path)?);
    let active_chats = Arc::new(ActiveChatRegistry::new(&config.general_db_path)?);
    let main_topic_store = Arc::new(MainTopicStore::new(&config.general_db_path)?);
    let cron_topic_store = Arc::new(CronTopicStore::new(&config.general_db_path)?);
    let chat_metadata_store = Arc::new(ChatMetadataStore::new(&config.general_db_path)?);

    let telegram_sender: Option<Arc<TelegramSender>> = match &config.telegram_token {
        Some(token) => Some(Arc::new(TelegramSender::new(token)?)),
        None => {
            tracing::info!("No Telegram token configured — running in web-only mode");
            None
        }
    };
    let telegram_poller: Option<Arc<TelegramPoller>> = match &config.telegram_token {
        Some(token) => Some(Arc::new(TelegramPoller::new(token)?)),
        None => None,
    };

    // Gemini transcriber is optional - only created if API key is provided
    let gemini: Option<Arc<GeminiTranscriber>> = match &config.gemini_key {
        Some(key) if !key.is_empty() => Some(Arc::new(GeminiTranscriber::new(key)?)),
        _ => {
            tracing::info!("Gemini API key not configured - voice/media transcription disabled");
            None
        }
    };

    // Initialize cron components
    let cron_feed = Arc::new(CronFeed::new(&config.general_db_path)?);
    let activity_tracker = ActivityTracker::new();
    let cron_scheduler = Arc::new(CronScheduler::new(cron_feed.clone()));
    let mut cron_gatekeeper_builder = CronGatekeeper::new(
        cron_feed.clone(),
        prompt_feed.clone(),
    )
    .with_response_feed(response_feed.clone())
    .with_topic_routing(cron_topic_store.clone(), active_chats.clone());
    if let Some(ref sender) = telegram_sender {
        cron_gatekeeper_builder = cron_gatekeeper_builder.with_telegram_sender(sender.clone());
    }
    let cron_gatekeeper = Arc::new(cron_gatekeeper_builder);

    // Initialize cron job schedules
    match cron_scheduler.initialize_job_schedules() {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Initialized {} cron job schedules", count);
            }
        }
        Err(e) => tracing::warn!("Failed to initialize cron schedules: {}", e),
    }

    // Register bot commands (only if Telegram is configured)
    if let Some(ref sender) = telegram_sender {
        if let Err(e) = sender
            .set_my_commands(&[("clear", "Clear conversation context and start fresh")])
            .await
        {
            tracing::warn!("Failed to register bot commands: {}", e);
        }
    }

    // Create typing indicator (only if Telegram is configured)
    let typing_indicator = telegram_sender.as_ref().map(|sender| {
        TypingIndicator::new(prompt_feed.clone(), sender.clone())
    });

    // Create Claude manager
    let mut claude_manager_builder = ClaudeManager::new(
        prompt_feed.clone(),
        response_feed.clone(),
        settings_store.clone(),
        &config.claude_model,
        &config.data_dir,
        config.process_timeout_ms,
    );
    if let Some(ref sender) = telegram_sender {
        claude_manager_builder = claude_manager_builder.with_telegram_sender(sender.clone());
    }
    let claude_manager = Arc::new(claude_manager_builder);

    // Create web chat WebSocket hub
    let chat_event_hub = Arc::new(ChatEventHub::new());

    // Create response broadcaster with activity tracker
    let response_broadcaster = Arc::new(
        ResponseBroadcaster::new(
            response_feed.clone(),
            message_store.clone(),
            active_chats.clone(),
            telegram_sender.clone(), // Option<Arc<TelegramSender>>
        )
        .with_activity_tracker(activity_tracker.clone())
        .with_main_topic_store(main_topic_store.clone())
        .with_cron_topic_store(cron_topic_store.clone())
        .with_chat_event_hub(chat_event_hub.clone(), chat_metadata_store.clone()),
    );

    // Create HTTP server state
    let app_state = AppState {
        prompt_feed: prompt_feed.clone(),
        response_feed: response_feed.clone(),
        message_store: message_store.clone(),
        media_store: media_store.clone(),
        cron_feed: cron_feed.clone(),
        settings_store: settings_store.clone(),
        chat_metadata_store: chat_metadata_store.clone(),
        logger: logger.clone(),
        data_dir: config.data_dir.clone(),
    };

    // Initialize work service (SQLite-backed project management)
    // Must be created before MCP state since MCP exposes work tools
    let (work_state, work_app_arc): (Option<WorkState>, Option<Arc<WorkApp>>) =
        match WorkDb::open(&config.data_dir) {
            Ok(db) => {
                let app = Arc::new(WorkApp::new(db));
                let agent_loop = Arc::new(
                    app.new_agent_loop(prompt_feed.clone(), response_feed.clone(), settings_store.clone())
                        .with_topic_routing(main_topic_store.clone(), active_chats.clone()),
                );
                tracing::info!("Work service initialized");
                logger.info("main", "Work service initialized");
                (
                    Some(WorkState {
                        app: app.clone(),
                        agent_loop,
                        activity_tracker: activity_tracker.clone(),
                        response_feed: response_feed.clone(),
                        idle_threshold_secs: config.cron_idle_threshold_secs,
                    }),
                    Some(app),
                )
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize work service: {} (continuing without)",
                    e
                );
                logger.warn(
                    "main",
                    format!("Work service failed: {} (continuing without)", e),
                );
                (None, None)
            }
        };

    // Conversation log directories for CLI harnesses.
    let claude_conversations_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .unwrap_or_else(|| config.data_dir.join("conversations"));
    let codex_conversations_dir = dirs::home_dir()
        .map(|h| h.join(".codex").join("sessions"))
        .unwrap_or_else(|| config.data_dir.join("codex-sessions"));

    // Start semantic indexer (if enabled) — must happen before MCP state
    // so we can share the VectorDb and Embedder with the MCP tools.
    let settings = settings_store.get();
    let initial_paused = settings.semantic_paused;
    let omp_num_threads = settings.omp_num_threads;
    let (semantic_state, vector_db, embedder) = if config.semantic_enabled {
        let indexer_config = IndexerConfig::new(
            &config.data_dir,
            &claude_conversations_dir,
            &codex_conversations_dir,
        );
        match SemanticIndexer::new(indexer_config, initial_paused, omp_num_threads).await {
            Ok(indexer) => {
                let paused_flag = indexer.paused();
                let conv_notify = indexer.conversation_notify();
                let db = indexer.db();
                let emb = indexer.embedder();
                tracing::info!(paused = initial_paused, "Starting semantic indexer");
                logger.info("main", "Semantic indexer started");
                let (status, _handles) = indexer.start();
                (
                    SemanticState {
                        status,
                        paused: Some(paused_flag),
                        settings_store: settings_store.clone(),
                        conversation_notify: Some(conv_notify),
                    },
                    Some(db),
                    Some(emb),
                )
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to start semantic indexer: {} (continuing without)",
                    e
                );
                logger.warn(
                    "main",
                    format!("Semantic indexer failed: {} (continuing without)", e),
                );
                (
                    SemanticState {
                        status: disabled_status(),
                        paused: None,
                        settings_store: settings_store.clone(),
                        conversation_notify: None,
                    },
                    None,
                    None,
                )
            }
        }
    } else {
        tracing::info!("Semantic search disabled via --disable-semantic");
        (
            SemanticState {
                status: disabled_status(),
                paused: None,
                settings_store: settings_store.clone(),
                conversation_notify: None,
            },
            None,
            None,
        )
    };

    // Initialize PM semantic search if both work service and embedder are available
    if let (Some(ref app), Some(ref emb)) = (&work_app_arc, &embedder) {
        app.search.init(emb.clone()).await;
        tracing::info!("PM semantic search initialized");
        // Background index — don't block startup
        let search = app.search.clone();
        tokio::spawn(async move {
            match search.reindex().await {
                Ok(stats) => tracing::info!(
                    tasks = stats.tasks_indexed,
                    docs = stats.documents_indexed,
                    comments = stats.comments_indexed,
                    chunks = stats.chunks_created,
                    skipped = stats.skipped_unchanged,
                    "PM semantic index built"
                ),
                Err(e) => tracing::warn!("PM semantic indexing failed: {e}"),
            }
        });
    }

    // Build SendTools for MCP file delivery
    let send_tools = {
        let mut tools = SendTools::new(media_store.clone(), message_store.clone());
        if let Some(ref sender) = telegram_sender {
            tools = tools.with_telegram(sender.clone());
        }
        tools = tools.with_chat_event_hub(chat_event_hub.clone());
        Some(tools)
    };

    // Build ImageTools for MCP image generation (requires Gemini API key)
    let image_tools: Option<ImageTools> = match &config.gemini_key {
        Some(key) if !key.is_empty() => {
            let output_dir = config.data_dir.join("generated_images");
            match ImageTools::new(
                key.clone(),
                output_dir,
                media_store.clone(),
                message_store.clone(),
            ) {
                Ok(tools) => {
                    let mut tools = tools
                        .with_chat_event_hub(chat_event_hub.clone());
                    if let Some(ref sender) = telegram_sender {
                        tools = tools.with_telegram(sender.clone());
                    }
                    tracing::info!("Image generation tools initialized");
                    Some(tools)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize image tools: {}", e);
                    None
                }
            }
        }
        _ => {
            tracing::info!("Gemini API key not configured - image generation disabled");
            None
        }
    };

    // Create unified MCP HTTP state (cron + memory + conversations + work + semantic)
    let mcp_cancellation = CancellationToken::new();
    let mcp_state = McpHttpState::new(
        cron_feed.clone(),
        config.memory_dir.clone(),
        claude_conversations_dir.clone(),
        codex_conversations_dir.clone(),
        mcp_cancellation.clone(),
        work_app_arc.clone(),
        vector_db,
        embedder,
        Some(cron_topic_store.clone()),
        telegram_sender.clone(),
        send_tools,
        image_tools,
    );

    // Create setup state for API key management
    let setup_state = SetupState::new(args.clone());

    // Create router config
    let router_config = RouterConfig {
        cors_allow_all: config.cors_allow_all,
        host: config.host.clone(),
        port: config.port,
    };

    // Create SSE state for work live updates
    let sse_state = work_app_arc.map(|app| SseState { app });

    // Create voice state for voice-to-MD endpoints
    let voice_state = VoiceState {
        data_dir: config.data_dir.clone(),
    };

    // Create auth state for dashboard login (localhost exempt, external needs session)
    let nonce_store = twolebot::server::auth::NonceStore::new();
    let auth_state = AuthState {
        secrets: Arc::new(SecretsStore::new(&config.general_db_path)?),
        sessions: Arc::new(SessionStore::new(&config.general_db_path)?),
        rate_limiter: twolebot::server::auth::RateLimiter::new(),
        nonce_store: nonce_store.clone(),
    };

    // Web chat state
    let chat_state = ChatState {
        prompt_feed: prompt_feed.clone(),
        message_store: message_store.clone(),
        media_store: media_store.clone(),
        chat_metadata_store: chat_metadata_store.clone(),
        chat_event_hub: chat_event_hub.clone(),
        data_dir: config.data_dir.clone(),
        default_user_id: 0,
    };
    let chat_ws_state = ChatWsState {
        hub: chat_event_hub.clone(),
    };

    // Tunnel: create a watch channel for the tunnel URL (populated after spawn)
    let tunnel_enabled = !args.no_tunnel;
    let (tunnel_url_tx, tunnel_url_rx) = tokio::sync::watch::channel::<Option<String>>(None);
    let tunnel_state = if tunnel_enabled {
        Some(TunnelState {
            url_rx: tunnel_url_rx,
            nonce_store: nonce_store.clone(),
        })
    } else {
        None
    };

    let mut builder = RouterBuilder::new(app_state)
        .config(router_config)
        .static_dir(config.frontend_dir.clone())
        .mcp(mcp_state)
        .setup(setup_state)
        .semantic(semantic_state)
        .voice(voice_state)
        .auth(auth_state)
        .chat(chat_state, chat_ws_state);
    if let Some(ws) = work_state.clone() {
        builder = builder.work(ws);
    }
    if let Some(ss) = sse_state {
        builder = builder.sse(ss);
    }
    if let Some(ts) = tunnel_state {
        builder = builder.tunnel(ts);
    }
    let router = builder.build();

    // Background work autowork loop:
    // - keeps the agent loop running when idle and work exists
    // - pulls next todo from live-board backlog when selection queue is empty
    // - respects explicit pause state
    let work_autowork = work_state
        .as_ref()
        .map(|state| (state.app.clone(), state.agent_loop.clone()));
    if let Some((work_app, agent_loop)) = work_autowork {
        let shutdown_for_autowork = shutdown_token.clone();
        let activity_for_autowork = activity_tracker.clone();
        let response_feed_for_autowork = response_feed.clone();
        let idle_threshold_secs = config.cron_idle_threshold_secs;
        tokio::spawn(async move {
            let poll_interval = Duration::from_secs(2);
            loop {
                if shutdown_for_autowork.is_cancelled() {
                    break;
                }

                let state = agent_loop.state().await;
                if state == twolebot::work::models::AgentLoopState::Paused
                    || state == twolebot::work::models::AgentLoopState::Running
                {
                    tokio::time::sleep(poll_interval).await;
                    continue;
                }

                // Only auto-start machine work after the user-facing quiet window.
                // User interactions and user-driven responses reset this timer.
                let is_quiet = activity_for_autowork
                    .is_idle_for(chrono::Duration::seconds(idle_threshold_secs))
                    .await;
                if !is_quiet {
                    tokio::time::sleep(poll_interval).await;
                    continue;
                }

                // Avoid overlapping a new machine prompt with in-flight machine responses.
                let machine_response_in_flight = response_feed_for_autowork
                    .has_pending_system_responses()
                    .unwrap_or(false);
                if machine_response_in_flight {
                    tokio::time::sleep(poll_interval).await;
                    continue;
                }

                // Auto-select live-tagged todo tasks onto the board
                match work_app.live.find_live_tagged_unselected_tasks().await {
                    Ok(live_ids) if !live_ids.is_empty() => {
                        tracing::info!(
                            "Autowork: auto-selecting {} live-tagged task(s): {:?}",
                            live_ids.len(),
                            live_ids
                        );
                        if let Err(e) = work_app.live.select_tasks(live_ids).await {
                            tracing::warn!("Autowork: failed to auto-select live tasks: {e}");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Autowork: failed to find live-tagged tasks: {e}");
                    }
                    _ => {}
                }

                let board = match work_app.live.get_live_board(Some(1)).await {
                    Ok(board) => board,
                    Err(e) => {
                        tracing::warn!("Autowork: failed to read live board: {e}");
                        tokio::time::sleep(poll_interval).await;
                        continue;
                    }
                };

                let has_selection_work = board.stats.queued > 0
                    || board.stats.active.is_some()
                    || board.selected.iter().any(|s| {
                        s.selection.status == twolebot::work::models::SelectionStatus::Paused
                    });

                if !has_selection_work && board.stats.total_backlog > 0 {
                    if let Err(e) = work_app.live.select_next_todo_for_agent().await {
                        tracing::warn!("Autowork: failed to auto-select next todo: {e}");
                    }
                }

                if let Err(e) = agent_loop.start().await {
                    if !e.to_string().contains("already running")
                        && !e.to_string().contains("no tasks in selection queue")
                    {
                        tracing::warn!("Autowork: failed to start agent loop: {e}");
                    }
                }

                tokio::time::sleep(poll_interval).await;
            }
        });
    }

    // Start HTTP server with ConnectInfo
    let http_addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&http_addr).await?;
    tracing::info!("HTTP server listening on {}", http_addr);

    // Register MCP server with Claude Code at the actual port
    register_mcp_with_claude(&config.data_dir, &logger);

    // Spawn Cloudflare quick tunnel (if enabled) with automatic restart on failure
    if tunnel_enabled {
        let tunnel_auth_token = {
            let secrets = SecretsStore::new(&config.general_db_path).ok();
            secrets
                .and_then(|s| s.get_auth_token().ok().flatten())
                .unwrap_or_default()
        };
        tokio::spawn(tunnel::run_resilient_tunnel(
            config.data_dir.clone(),
            config.port,
            tunnel_url_tx,
            shutdown_token.clone(),
            tunnel_auth_token,
            logger.clone(),
        ));
    }

    let shutdown_for_http = shutdown_token.clone();
    let http_handle = tokio::spawn(async move {
        let router = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        if let Err(e) = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                shutdown_for_http.cancelled().await;
            })
            .await
        {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    // Start typing indicator (only if Telegram is configured)
    if let Some(indicator) = typing_indicator {
        let interval = config.typing_interval_secs;
        tokio::spawn(async move {
            indicator.start(interval).await;
        });
    }

    // Start Claude manager
    let claude_manager_clone = claude_manager.clone();
    let claude_handle = tokio::spawn(async move {
        claude_manager_clone.start(POLL_INTERVAL_MS).await;
    });

    // Start response broadcaster
    let broadcaster_clone = response_broadcaster.clone();
    let broadcaster_handle = tokio::spawn(async move {
        broadcaster_clone.start(POLL_INTERVAL_MS).await;
    });

    // Start cron scheduler
    let scheduler_clone = cron_scheduler.clone();
    let scheduler_handle = tokio::spawn(async move {
        scheduler_clone.start(CRON_POLL_INTERVAL_MS).await;
    });

    // Start cron gatekeeper
    let gatekeeper_clone = cron_gatekeeper.clone();
    let gatekeeper_handle = tokio::spawn(async move {
        gatekeeper_clone.start(CRON_POLL_INTERVAL_MS).await;
    });

    // Start Telegram polling (only if configured)
    let poller_handle = if let Some(ref poller) = telegram_poller {
        let poller_clone = poller.clone();
        let (update_tx, mut update_rx) = mpsc::channel::<Update>(100);
        let handle = tokio::spawn(async move {
            if let Err(e) = poller_clone.start_polling(update_tx).await {
                tracing::error!("Telegram polling error: {}", e);
            }
        });

        // Process incoming Telegram updates
        let shutdown_for_updates = shutdown_token.clone();
        let telegram_sender_inner = telegram_sender.clone().unwrap();
        let telegram_poller_inner = telegram_poller.clone().unwrap();
        let prompt_feed = prompt_feed.clone();
        let message_store = message_store.clone();
        let media_store = media_store.clone();
        let active_chats = active_chats.clone();
        let gemini = gemini.clone();
        let main_topic_store = main_topic_store.clone();
        let logger = logger.clone();
        let activity_tracker = activity_tracker.clone();
        let settings_store = settings_store.clone();
        let chat_metadata_store = chat_metadata_store.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_for_updates.cancelled() => {
                        tracing::debug!("Update processor shutting down");
                        break;
                    }
                    update = update_rx.recv() => {
                        let Some(update) = update else { break };

                        // Record activity for cron gatekeeper
                        activity_tracker.record_activity().await;

                        if let Err(e) = process_update(
                            &update,
                            &prompt_feed,
                            &message_store,
                            &media_store,
                            &active_chats,
                            &telegram_sender_inner,
                            &telegram_poller_inner,
                            gemini.as_ref(),
                            &main_topic_store,
                            &logger,
                            &settings_store,
                            &chat_metadata_store,
                        )
                        .await
                        {
                            tracing::error!("Error processing update: {}", e);
                            logger.error("update_processor", format!("Error: {}", e));
                        }
                    }
                }
            }
        });

        Some(handle)
    } else {
        tracing::info!("Telegram polling disabled — web-only mode");
        None
    };

    logger.info("main", "All components started");
    tracing::info!("Twolebot is running. Press Ctrl+C to stop.");

    // Wait for shutdown signal (Ctrl+C or SIGTERM)
    let shutdown_signal = async {
        let ctrl_c = tokio::signal::ctrl_c();

        #[cfg(unix)]
        {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut sigterm) => {
                    tokio::select! {
                        _ = ctrl_c => {},
                        _ = sigterm.recv() => {},
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to install SIGTERM handler: {} (Ctrl+C still works)",
                        e
                    );
                    logger.warn(
                        "main",
                        format!(
                            "Failed to install SIGTERM handler: {} (Ctrl+C still works)",
                            e
                        ),
                    );
                    let _ = ctrl_c.await;
                }
            }
        }

        #[cfg(not(unix))]
        {
            ctrl_c.await.ok();
        }
    };

    shutdown_signal.await;
    tracing::info!("Shutting down gracefully...");
    logger.info("main", "Shutting down gracefully");

    // Signal all tasks to stop
    shutdown_token.cancel();

    // Give tasks up to 5 seconds to finish gracefully
    let shutdown_timeout = Duration::from_secs(5);
    let _ = tokio::time::timeout(shutdown_timeout, async {
        let _ = tokio::join!(
            http_handle,
            claude_handle,
            broadcaster_handle,
            scheduler_handle,
            gatekeeper_handle,
        );
        if let Some(handle) = poller_handle {
            let _ = handle.await;
        }
    })
    .await;

    tracing::info!("Shutdown complete");
    logger.info("main", "Shutdown complete");

    Ok(())
}

/// Show current configuration status
fn run_status_command(args: &Args) -> anyhow::Result<()> {
    let status = SetupStatus::check(args);
    let data_dir = args
        .data_dir
        .clone()
        .unwrap_or_else(twolebot::config::default_data_dir);
    let runtime_db = data_dir.join("runtime.sqlite3");
    let vector_db = data_dir.join("vectors.sqlite3");
    let logs_file = data_dir.join("logs.jsonl");

    println!("twolebot v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Platform: {}", status.platform);
    println!();
    println!("Directories:");
    println!("  Data:   {}", status.data_dir);
    println!();
    println!("Databases:");
    println!("  General DB: {}", runtime_db.display());
    println!("  Vector DB:  {}", vector_db.display());
    println!("  Logs:       {}", logs_file.display());
    println!();
    println!("Configuration:");
    println!(
        "  Telegram token: {}",
        if status.has_telegram_token {
            "configured"
        } else {
            "missing"
        }
    );
    println!(
        "  Gemini API key: {}",
        if status.has_gemini_key {
            "configured"
        } else {
            "missing"
        }
    );
    println!(
        "  Claude CLI:     {}",
        status.claude_cli_version.as_deref().unwrap_or("not found")
    );
    println!();

    // Show auth token if it exists
    {
        use twolebot::storage::SecretsStore;
        if let Ok(secrets) = SecretsStore::new(&runtime_db) {
            if let Ok(Some(token)) = secrets.get_auth_token() {
                println!("Auth token:     {}", token);
                println!();
            }
        }
    }

    if status.is_complete {
        println!("Status: Ready to run");
    } else {
        println!("Status: Setup required");
        println!("\nRun 'twolebot' to start setup wizard.");
    }

    Ok(())
}

/// Run in setup mode - serve only the setup UI
async fn run_setup_mode(args: &Args) -> anyhow::Result<()> {
    let config = Config::for_setup(args)?;

    println!();
    println!("  twolebot v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("  Setup required. Open your browser:");
    println!();
    println!("    http://localhost:{}/setup", config.port);
    println!();
    println!("  Remote access (VPS/server):");
    println!("    http://<your-ip>:{}/setup", config.port);
    println!("    or: ssh -L {0}:localhost:{0} user@host", config.port);
    println!();

    // Create minimal app state for setup mode
    let logger = SharedLogger::new(&config.logs_file)?;

    // Create directories
    if let Some(parent) = config.general_db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir_all(&config.media_dir)?;

    let prompt_feed = Arc::new(PromptFeed::new(&config.general_db_path)?);
    let response_feed = Arc::new(ResponseFeed::new(&config.general_db_path)?);
    let message_store = Arc::new(MessageStore::new(&config.general_db_path)?);
    let media_store = Arc::new(MediaStore::new(&config.media_dir)?);
    let cron_feed = Arc::new(CronFeed::new(&config.general_db_path)?);
    let settings_store = Arc::new(SettingsStore::new(&config.general_db_path)?);

    let chat_metadata_store = Arc::new(ChatMetadataStore::new(&config.general_db_path)?);

    let app_state = AppState {
        prompt_feed,
        response_feed,
        message_store,
        media_store,
        cron_feed,
        settings_store,
        chat_metadata_store,
        logger,
        data_dir: config.data_dir.clone(),
    };

    // Create setup state for the setup API
    let setup_state = SetupState::new(args.clone());

    // Create router config (setup mode uses permissive CORS since it's local-only)
    let router_config = RouterConfig {
        cors_allow_all: true,
        host: config.host.clone(),
        port: config.port,
    };

    let router = RouterBuilder::new(app_state)
        .config(router_config)
        .static_dir(config.frontend_dir.clone())
        .setup(setup_state)
        .build();

    // Start HTTP server - setup mode always binds to 0.0.0.0 for easier access
    // Try the configured port first, then fall back to nearby ports
    let (listener, actual_port) = {
        let mut port = config.port;
        let max_attempts = 10;
        let mut listener_result = None;
        for attempt in 0..max_attempts {
            let addr = format!("0.0.0.0:{}", port);
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => {
                    if attempt > 0 {
                        println!("  Port {} was busy, using port {} instead.", config.port, port);
                        println!();
                    }
                    listener_result = Some((l, port));
                    break;
                }
                Err(_) => {
                    port = config.port + (attempt as u16) + 1;
                }
            }
        }
        match listener_result {
            Some(lr) => lr,
            None => {
                let addr = format!("0.0.0.0:{}", config.port);
                let l = tokio::net::TcpListener::bind(&addr).await?;
                (l, config.port)
            }
        }
    };

    // Try to open browser
    let url = format!("http://localhost:{}/setup", actual_port);
    let _ = open::that(&url);

    let router = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
    axum::serve(listener, router).await?;

    Ok(())
}

/// Run twolebot as a stdio MCP server.
/// Claude CLI spawns this process and communicates over stdin/stdout.
async fn run_mcp_stdio(args: &Args) -> anyhow::Result<()> {
    use rmcp::ServiceExt;

    let data_dir = args
        .data_dir
        .clone()
        .unwrap_or_else(twolebot::config::default_data_dir);
    let runtime_db = data_dir.join("runtime.sqlite3");
    let memory_dir = args
        .memory_dir
        .clone()
        .unwrap_or_else(|| data_dir.join("memory"));
    let claude_conversations_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .unwrap_or_else(|| data_dir.join("conversations"));
    let codex_conversations_dir = dirs::home_dir()
        .map(|h| h.join(".codex").join("sessions"))
        .unwrap_or_else(|| data_dir.join("codex-sessions"));

    // Ensure directories exist
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(&memory_dir)?;

    let cron_feed = Arc::new(CronFeed::new(&runtime_db)?);

    // Initialize work service (optional — non-fatal if it fails)
    let work_app: Option<Arc<WorkApp>> = match WorkDb::open(&data_dir) {
        Ok(db) => Some(Arc::new(WorkApp::new(db))),
        Err(_) => None,
    };

    // Initialize semantic search for MCP tools (if not disabled)
    let (vector_db, embedder) = if !args.disable_semantic {
        use twolebot::semantic::{Embedder, IndexerConfig, VectorDb};
        let indexer_config = IndexerConfig::new(
            &data_dir,
            &claude_conversations_dir,
            &codex_conversations_dir,
        );
        match VectorDb::open(&indexer_config.db_path) {
            Ok(db) => match Embedder::global(2).await {
                Ok(emb) => (Some(std::sync::Arc::new(db)), Some(emb)),
                Err(e) => {
                    eprintln!("Warning: semantic embedder failed: {e}");
                    (None, None)
                }
            },
            Err(e) => {
                eprintln!("Warning: vector DB failed: {e}");
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    // Wire topic management for cron_close_topic MCP tool
    let cron_topic_store = Arc::new(CronTopicStore::new(&runtime_db)?);
    let telegram_sender = {
        use twolebot::storage::secrets::SecretsStore;
        let token = args
            .telegram_token
            .clone()
            .or_else(|| {
                SecretsStore::new(&runtime_db)
                    .ok()
                    .and_then(|s| s.get_telegram_token().ok().flatten())
            });
        match token {
            Some(t) => Some(Arc::new(TelegramSender::new(&t)?)),
            None => None,
        }
    };

    // Initialize PM semantic search if both work service and embedder are available
    if let (Some(ref app), Some(ref emb)) = (&work_app, &embedder) {
        app.search.init(emb.clone()).await;
    }

    // Build SendTools for file delivery (no chat_event_hub in stdio mode)
    let (send_tools, image_tools) = {
        use twolebot::storage::{MediaStore, MessageStore};
        let media_store = Arc::new(MediaStore::new(data_dir.join("media"))?);
        let message_store = Arc::new(MessageStore::new(&runtime_db)?);
        let mut tools = SendTools::new(media_store.clone(), message_store.clone());
        if let Some(ref sender) = telegram_sender {
            tools = tools.with_telegram(sender.clone());
        }

        // Build ImageTools (requires Gemini API key)
        let gemini_key = args.gemini_key.clone().or_else(|| {
            twolebot::storage::SecretsStore::new(&runtime_db)
                .ok()
                .and_then(|s| s.get_gemini_key().ok().flatten())
        });
        let img_tools = match gemini_key {
            Some(key) if !key.is_empty() => {
                let output_dir = data_dir.join("generated_images");
                match ImageTools::new(key, output_dir, media_store, message_store) {
                    Ok(t) => {
                        let mut t = t;
                        if let Some(ref sender) = telegram_sender {
                            t = t.with_telegram(sender.clone());
                        }
                        Some(t)
                    }
                    Err(_) => None,
                }
            }
            _ => None,
        };

        (Some(tools), img_tools)
    };

    let server = TwolebotMcpServer::new(
        cron_feed,
        memory_dir,
        claude_conversations_dir,
        codex_conversations_dir,
        work_app,
        vector_db,
        embedder,
        Some(cron_topic_store),
        telegram_sender,
        send_tools,
        image_tools,
    );

    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}

/// One-time migration: copy databases from old layout to new flat layout.
/// Old: data/runtime/runtime.sqlite3, data/vectors.db
/// New: data/runtime.sqlite3, data/vectors.sqlite3
/// Copies only — old files are left intact.
fn migrate_db_layout(args: &Args) {
    let data_dir = args
        .data_dir
        .clone()
        .unwrap_or_else(twolebot::config::default_data_dir);

    let migrations: &[(&str, &str)] = &[
        ("runtime/runtime.sqlite3", "runtime.sqlite3"),
        ("vectors.db", "vectors.sqlite3"),
    ];

    for (old_rel, new_rel) in migrations {
        let old_path = data_dir.join(old_rel);
        let new_path = data_dir.join(new_rel);

        if old_path.exists() && !new_path.exists() {
            match std::fs::copy(&old_path, &new_path) {
                Ok(bytes) => {
                    tracing::info!(
                        old = %old_path.display(),
                        new = %new_path.display(),
                        bytes,
                        "migrated database to new layout"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        old = %old_path.display(),
                        new = %new_path.display(),
                        error = %e,
                        "failed to migrate database"
                    );
                }
            }
        }
    }
}

/// Register twolebot's MCP server with Claude Code at local scope (stdio transport).
/// Uses `claude mcp remove` + `add` since add refuses to overwrite.
/// Local scope writes to data_dir/.claude/settings.local.json - only visible to
/// Claude processes spawned from that directory.
/// Non-fatal: logs warnings on failure so the server still starts.
fn register_mcp_with_claude(data_dir: &std::path::Path, logger: &SharedLogger) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Cannot determine executable path, skipping MCP registration: {}", e);
            logger.warn("mcp", format!("Cannot determine exe path: {}", e));
            return;
        }
    };
    // Linux appends " (deleted)" to /proc/self/exe when the binary is replaced by a rebuild.
    // Strip it so the registered MCP command points to the actual binary path.
    let exe_str = exe.display().to_string().trim_end_matches(" (deleted)").to_string();
    let data_dir_str = data_dir.display().to_string();

    // Remove stale registration (ignore errors - it may not exist)
    let _ = std::process::Command::new("claude")
        .args(["mcp", "remove", "-s", "local", "twolebot"])
        .current_dir(data_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();

    match std::process::Command::new("claude")
        .args([
            "mcp", "add", "--transport", "stdio", "-s", "local", "twolebot", "--",
            &exe_str, "mcp-stdio", "--data-dir", &data_dir_str,
        ])
        .current_dir(data_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) if output.status.success() => {
            tracing::info!("Registered MCP (stdio) at local scope in {}: {} mcp-stdio", data_dir_str, exe_str);
            logger.info(
                "mcp",
                format!("Registered MCP (stdio) at local scope in {}", data_dir_str),
            );
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to register MCP: {}", stderr.trim());
            logger.warn("mcp", format!("MCP registration failed: {}", stderr.trim()));
        }
        Err(e) => {
            tracing::warn!("Claude CLI not found, skipping MCP registration: {}", e);
            logger.warn("mcp", format!("Claude CLI not found: {}", e));
        }
    }
}
