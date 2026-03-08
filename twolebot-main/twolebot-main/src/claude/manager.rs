use crate::claude::harness::{normalize_harness_name, HarnessRegistry, HarnessRequest};
use crate::claude::process::ClaudeOutput;
use crate::claude::stream::ExtractOptions;
use crate::error::Result;
use crate::storage::{
    PromptFeed, PromptItem, PromptSource, ResponseFeed, ResponseItem, SettingsStore,
};
use crate::telegram::send::TelegramSender;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// Manages Claude job processing across multiple topic workspaces.
///
/// Each topic (identified by `topic_key`) gets its own independent worker slot
/// and work directory, allowing parallel Claude processes for different topics.
/// The main thread (topic_key = None) uses base_dir directly.
pub struct ClaudeManager {
    prompt_feed: Arc<PromptFeed>,
    response_feed: Arc<ResponseFeed>,
    settings_store: Arc<SettingsStore>,
    telegram_sender: Option<Arc<TelegramSender>>,
    harnesses: HarnessRegistry,
    base_dir: PathBuf,
    /// Per-topic running jobs: key = topic_key (None = main thread)
    workers: Arc<Mutex<HashMap<Option<String>, RunningJob>>>,
    /// Per-topic clear flags: tracks which topics need a fresh conversation
    clear_pending: Arc<Mutex<HashSet<Option<String>>>>,
    /// Topics that have had their work_dir + MCP registration initialized
    initialized_topics: Arc<Mutex<HashSet<String>>>,
    /// Per-topic last harness used; if harness changes, start a fresh conversation.
    last_harness_by_topic: Arc<Mutex<HashMap<Option<String>, String>>>,
}

struct RunningJob {
    prompt_id: String,
    kind: RunningJobKind,
    cancel_tx: mpsc::Sender<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunningJobKind {
    User(i64),
    Cron,
}

impl ClaudeManager {
    pub fn new(
        prompt_feed: Arc<PromptFeed>,
        response_feed: Arc<ResponseFeed>,
        settings_store: Arc<SettingsStore>,
        model: impl Into<String>,
        base_dir: impl Into<PathBuf>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            prompt_feed,
            response_feed,
            settings_store: settings_store.clone(),
            telegram_sender: None,
            harnesses: HarnessRegistry::with_defaults(model, timeout_ms, settings_store),
            base_dir: base_dir.into(),
            workers: Arc::new(Mutex::new(HashMap::new())),
            clear_pending: Arc::new(Mutex::new(HashSet::new())),
            initialized_topics: Arc::new(Mutex::new(HashSet::new())),
            last_harness_by_topic: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Enable reaction updates on Telegram messages during processing
    pub fn with_telegram_sender(mut self, sender: Arc<TelegramSender>) -> Self {
        self.telegram_sender = Some(sender);
        self
    }

    /// Create extract options from current settings
    fn extract_options(&self) -> ExtractOptions {
        let settings = self.settings_store.get();
        ExtractOptions {
            show_tool_messages: settings.show_tool_messages,
            show_thinking_messages: settings.show_thinking_messages,
            show_tool_results: settings.show_tool_results,
        }
    }

    /// Resolve configured harness for a prompt source.
    /// All prompt sources currently use the runtime `chat_harness` setting.
    fn resolve_harness_for_source(
        &self,
        source: &PromptSource,
    ) -> (String, Arc<dyn crate::claude::harness::ChatHarness>) {
        let requested = self.settings_store.get().chat_harness;
        let requested_normalized = normalize_harness_name(&requested);
        let resolved = self.harnesses.resolve(&requested_normalized);
        if resolved.0 != requested_normalized {
            tracing::warn!(
                "Unknown harness '{}' requested for source {:?}; falling back to '{}'",
                requested_normalized,
                source,
                resolved.0
            );
        }
        resolved
    }

    /// Resolve the work directory for a given topic_key.
    /// None = base_dir (main thread), Some(key) = base_dir/topics/{key}/
    fn work_dir_for_topic(&self, topic_key: &Option<String>) -> PathBuf {
        match topic_key {
            None => self.base_dir.clone(),
            Some(key) => self.base_dir.join("topics").join(key),
        }
    }

    /// Ensure a topic's work directory exists and has MCP registered.
    /// Idempotent — skips if already initialized this session.
    async fn ensure_topic_initialized(&self, topic_key: &str, work_dir: &std::path::Path) {
        // Fast path: already initialized (no lock contention for the common case)
        {
            let initialized = self.initialized_topics.lock().await;
            if initialized.contains(topic_key) {
                return;
            }
        }

        // Create the work directory (must use tokio's async fs to avoid blocking the runtime)
        let work_dir_owned = work_dir.to_path_buf();
        if let Err(e) = tokio::fs::create_dir_all(&work_dir_owned).await {
            tracing::error!(
                "Failed to create topic work dir {}: {}",
                work_dir_owned.display(),
                e
            );
            return;
        }

        // Register MCP for this topic directory
        register_mcp_for_dir(work_dir, &self.base_dir).await;

        // Note: topic work dirs inherit CLAUDE.md from data/topics/CLAUDE.md
        // via Claude CLI's ancestor-directory loading. We do NOT symlink the
        // project-level CLAUDE.md here — that would override the agent guidance file.

        // Mark as initialized
        let mut initialized = self.initialized_topics.lock().await;
        initialized.insert(topic_key.to_string());
        tracing::info!(
            "Initialized topic workspace: {} at {}",
            topic_key,
            work_dir.display()
        );
    }

    /// Update the reaction on the original Telegram message (fire-and-forget).
    async fn set_source_reaction(&self, source: &PromptSource, emoji: &str) {
        let Some(ref sender) = self.telegram_sender else {
            return;
        };
        if let PromptSource::Telegram {
            chat_id,
            message_id,
            ..
        } = source
        {
            let _ = sender.set_reaction(*chat_id, *message_id, emoji).await;
        }
    }

    /// Start the manager loop
    pub async fn start(self: Arc<Self>, poll_interval_ms: u64) {
        let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

        loop {
            interval.tick().await;

            // Process ALL pending prompts (not just one — multiple topics can run in parallel)
            match self.prompt_feed.all_pending() {
                Ok(prompts) => {
                    for prompt in prompts {
                        if let Err(e) = self.try_schedule_prompt(prompt).await {
                            tracing::error!("Error scheduling prompt: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking pending prompts: {}", e);
                }
            }
        }
    }

    /// Try to schedule a pending prompt. Returns Ok(()) whether it was scheduled or deferred.
    async fn try_schedule_prompt(self: &Arc<Self>, prompt: PromptItem) -> Result<()> {
        let topic_key = prompt.topic_key.clone();
        let pending_kind = if matches!(prompt.source, crate::storage::PromptSource::Cron { .. }) {
            RunningJobKind::Cron
        } else {
            RunningJobKind::User(prompt.user_id)
        };

        // Check if this topic's worker slot is busy
        {
            let workers = self.workers.lock().await;
            if let Some(running) = workers.get(&topic_key) {
                if running.prompt_id == prompt.id {
                    return Ok(());
                }

                // Per-topic interruption policy (same rules, scoped to topic)
                let should_interrupt = match (pending_kind, running.kind) {
                    (RunningJobKind::User(_), RunningJobKind::Cron) => true,
                    (RunningJobKind::User(a), RunningJobKind::User(b)) => a == b,
                    _ => false,
                };

                if should_interrupt {
                    tracing::info!(
                        "Interrupting job {} in topic {:?} for new prompt {}",
                        running.prompt_id,
                        topic_key,
                        prompt.id
                    );
                    // Clone sender and drop lock before awaiting
                    let cancel_tx = running.cancel_tx.clone();
                    drop(workers);
                    let _ = cancel_tx.send(()).await;
                } else {
                    // Topic is busy — leave prompt pending for next tick
                    return Ok(());
                }
            }
        }

        // CRITICAL: Mark as running BEFORE spawning to prevent race condition
        let prompt = match self.prompt_feed.mark_running(&prompt.id) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!("Could not claim prompt {}: {}", prompt.id, e);
                return Ok(());
            }
        };

        // Ensure topic workspace is initialized (for non-main topics)
        if let Some(ref key) = topic_key {
            let work_dir = self.work_dir_for_topic(&topic_key);
            self.ensure_topic_initialized(key, &work_dir).await;
        }

        // Register worker slot BEFORE spawning to prevent duplicate workers
        let (cancel_tx, cancel_rx) = mpsc::channel(1);
        let job_kind = if matches!(prompt.source, crate::storage::PromptSource::Cron { .. }) {
            RunningJobKind::Cron
        } else {
            RunningJobKind::User(prompt.user_id)
        };
        {
            let mut workers = self.workers.lock().await;
            workers.insert(
                topic_key.clone(),
                RunningJob {
                    prompt_id: prompt.id.clone(),
                    kind: job_kind,
                    cancel_tx,
                },
            );
        }

        // Capture extract_options at schedule time (reads current settings)
        let extract_options = self.extract_options();
        let manager = Arc::clone(self);

        tokio::spawn(async move {
            if let Err(e) = manager
                .process_prompt(prompt, cancel_rx, extract_options)
                .await
            {
                tracing::error!("Error processing prompt: {}", e);
            }
        });

        Ok(())
    }

    /// Get the currently running job's prompt ID for the main topic (backwards compat)
    pub async fn current_prompt_id(&self) -> Option<String> {
        self.workers
            .lock()
            .await
            .get(&None)
            .map(|j| j.prompt_id.clone())
    }

    async fn process_prompt(
        &self,
        prompt: PromptItem,
        mut cancel_rx: mpsc::Receiver<()>,
        extract_options: ExtractOptions,
    ) -> Result<()> {
        let topic_key = prompt.topic_key.clone();

        tracing::info!(
            "Processing prompt {} for user {} in topic {:?}",
            prompt.id,
            prompt.user_id,
            topic_key
        );

        // Signal that processing has started
        self.set_source_reaction(&prompt.source, "👾").await;

        // Handle /clear as a fast-path: no Claude process, just set flag for this topic
        if prompt.prompt.trim().starts_with("/clear") {
            let confirm = ResponseItem::new(
                &prompt.id,
                prompt.source.clone(),
                prompt.user_id,
                "Context cleared!",
                true,
                0,
            );
            self.response_feed.enqueue(confirm)?;
            {
                let mut clears = self.clear_pending.lock().await;
                clears.insert(topic_key.clone());
            }
            // Clear worker slot since we're returning early
            {
                let mut workers = self.workers.lock().await;
                workers.remove(&topic_key);
            }
            self.prompt_feed.mark_completed(&prompt.id)?;
            tracing::info!("Clear command processed for topic {:?}", prompt.topic_key);
            return Ok(());
        }

        // Check for role command prefixes (/pm, /dev, /harden) and inject role prompts.
        // Extracts user text after the command, injects the role template with $ARGUMENTS replaced.
        let prompt_text = {
            let trimmed = prompt.prompt.trim();
            let role_match = [("/pm", "pm"), ("/dev", "dev"), ("/harden", "harden")]
                .iter()
                .find_map(|(prefix, role)| {
                    if let Some(rest) = trimmed.strip_prefix(prefix) {
                        // Must be followed by whitespace or end-of-string (not "/pmsomething")
                        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
                            Some((*role, rest.trim()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

            if let Some((role, user_text)) = role_match {
                let settings = self.settings_store.get();
                let template = match role {
                    "pm" => &settings.pm_role_prompt,
                    "dev" => &settings.dev_role_prompt,
                    "harden" => &settings.harden_role_prompt,
                    _ => unreachable!(),
                };
                let injected = template.replace("$ARGUMENTS", user_text);
                tracing::info!("Injecting {} role prompt for /{} command", role, role);
                injected
            } else {
                prompt.prompt.clone()
            }
        };

        let actual_prompt = if let Some(ref media_path) = prompt.media_path {
            let relative_media = if prompt.topic_key.is_some() {
                format!("../../media/{}", media_path)
            } else {
                format!("media/{}", media_path)
            };
            format!("{}\n\nMedia file: {}", prompt_text, relative_media)
        } else {
            prompt_text
        };

        // Check if we need to start fresh (after a /clear for this topic)
        let start_fresh = {
            let mut clears = self.clear_pending.lock().await;
            clears.remove(&topic_key)
        };

        // Set up output channel
        let (output_tx, mut output_rx) = mpsc::channel(100);

        let (harness_name, harness) = self.resolve_harness_for_source(&prompt.source);
        tracing::info!(
            "Using harness '{}' for prompt {} in topic {:?}",
            harness_name,
            prompt.id,
            topic_key
        );

        // If the selected harness changed for this topic, do a fresh start.
        let switched_harness = {
            let mut by_topic = self.last_harness_by_topic.lock().await;
            let switched = by_topic
                .get(&topic_key)
                .is_some_and(|prev| prev != &harness_name);
            by_topic.insert(topic_key.clone(), harness_name.clone());
            switched
        };
        if switched_harness {
            tracing::info!(
                "Harness switched for topic {:?}; forcing fresh conversation context",
                topic_key
            );
        }

        // Spawn configured harness in the topic's work directory
        let work_dir = self.work_dir_for_topic(&topic_key);
        let request = HarnessRequest {
            prompt: actual_prompt,
            work_dir,
            continue_conversation: !start_fresh && !switched_harness,
            extract_options,
        };

        let process_handle =
            tokio::spawn(async move { harness.run_streaming(request, output_tx).await });

        let mut sequence = 0u32;
        let mut accumulated_text = String::new();
        let mut cancelled = false;
        let mut failed = false;
        let mut failure_reason: Option<String> = None;

        // Periodic "still working" status messages every 10 minutes
        let status_interval = Duration::from_secs(600);
        let mut status_tick = tokio::time::interval(status_interval);
        status_tick.tick().await; // consume the immediate first tick
        let mut status_count = 0u32;

        let status_messages = [
            "📚 Still working hard like a schoolboy making an A! Please be patient...",
            "✏️ Head down, still grinding away. This one's taking some effort!",
            "📖 Nose to the grindstone — making good progress, hang tight!",
            "🎒 Still at it! Doing my homework diligently, won't be long now...",
            "🔬 Deep in concentration here. Quality takes time!",
            "📝 Still scribbling away — this is a big one but I'm on it!",
        ];

        loop {
            tokio::select! {
                output = output_rx.recv() => {
                    match output {
                        Some(ClaudeOutput::Text(text)) => {
                            accumulated_text.push_str(&text);
                            let response = ResponseItem::new(
                                &prompt.id,
                                prompt.source.clone(),
                                prompt.user_id,
                                text,
                                false,
                                sequence,
                            );
                            self.response_feed.enqueue(response)?;
                            sequence += 1;
                        }
                        Some(ClaudeOutput::Complete(final_text)) => {
                            let remaining = if accumulated_text.len() < final_text.len() {
                                final_text[accumulated_text.len()..].to_string()
                            } else {
                                String::new()
                            };
                            // Always send a final chunk (even if empty) so SSE
                            // consumers know the response is complete.
                            let response = ResponseItem::new(
                                &prompt.id,
                                prompt.source.clone(),
                                prompt.user_id,
                                remaining,
                                true,
                                sequence,
                            );
                            self.response_feed.enqueue(response)?;
                            break;
                        }
                        Some(ClaudeOutput::Error(e)) => {
                            tracing::error!("Harness '{}' error in topic {:?}: {}", harness_name, topic_key, e);
                            failed = true;
                            failure_reason = Some(e.clone());
                            self.prompt_feed.mark_failed(&prompt.id, &e)?;
                            break;
                        }
                        Some(ClaudeOutput::Timeout { partial_output }) => {
                            tracing::warn!(
                                "Harness '{}' timeout in topic {:?} with {} chars",
                                harness_name,
                                topic_key,
                                partial_output.len()
                            );
                            failed = true;
                            failure_reason = Some("Process timeout".to_string());
                            self.prompt_feed.mark_failed(&prompt.id, "Process timeout")?;
                            break;
                        }
                        None => break,
                    }
                }
                _ = status_tick.tick() => {
                    status_count += 1;
                    let msg = status_messages[status_count as usize % status_messages.len()];
                    let elapsed_mins = status_count * 10;
                    let status_text = format!("{} ({}min elapsed)", msg, elapsed_mins);
                    tracing::info!(
                        "Sending status update for prompt {} in topic {:?} ({}min)",
                        prompt.id,
                        topic_key,
                        elapsed_mins
                    );

                    // Send directly via Telegram if available
                    if let Some(ref sender) = self.telegram_sender {
                        let chat_id = prompt.source.chat_id();
                        let thread_id = prompt.source.message_thread_id();
                        if let Some(cid) = chat_id {
                            let _ = sender.send_message(cid, thread_id, &status_text).await;
                        }
                    }
                }
                _ = cancel_rx.recv() => {
                    tracing::info!("Prompt {} cancelled in topic {:?}", prompt.id, topic_key);
                    process_handle.abort();
                    cancelled = true;
                    break;
                }
            }
        }

        if !cancelled {
            match process_handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(
                        "Harness '{}' failed for prompt {}: {}",
                        harness_name,
                        prompt.id,
                        e
                    );
                    if !failed {
                        failed = true;
                        let reason = format!("Harness '{harness_name}' failed: {e}");
                        failure_reason = Some(reason.clone());
                        if let Err(mark_err) = self.prompt_feed.mark_failed(&prompt.id, &reason) {
                            tracing::error!(
                                "Failed to mark prompt {} failed after harness error: {}",
                                prompt.id,
                                mark_err
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Harness task join error for prompt {}: {}", prompt.id, e);
                    if !failed {
                        failed = true;
                        let reason = "Harness task join error".to_string();
                        failure_reason = Some(reason.clone());
                        if let Err(mark_err) = self.prompt_feed.mark_failed(&prompt.id, &reason) {
                            tracing::error!(
                                "Failed to mark prompt {} failed after join error: {}",
                                prompt.id,
                                mark_err
                            );
                        }
                    }
                }
            }
        }

        // Clear this topic's worker slot
        {
            let mut workers = self.workers.lock().await;
            if workers
                .get(&topic_key)
                .is_some_and(|j| j.prompt_id == prompt.id)
            {
                workers.remove(&topic_key);
            }
        }

        // Mark completed unless cancelled/failed
        if !cancelled && !failed {
            if let Err(e) = self.prompt_feed.mark_completed(&prompt.id) {
                tracing::error!("Failed to mark prompt {} completed: {}", prompt.id, e);
            }
            self.set_source_reaction(&prompt.source, "👌").await;
        } else if !cancelled {
            tracing::warn!(
                "Prompt {} ended in failed state ({})",
                prompt.id,
                failure_reason.unwrap_or_else(|| "unknown error".to_string())
            );
            self.set_source_reaction(&prompt.source, "👌").await;
        } else {
            if let Err(e) = self.prompt_feed.mark_failed(&prompt.id, "Interrupted") {
                tracing::error!("Failed to mark prompt {} as interrupted: {}", prompt.id, e);
            }
            if let Err(e) = self.response_feed.cancel_for_prompt(&prompt.id) {
                tracing::error!("Failed to cancel responses for prompt {}: {}", prompt.id, e);
            }
            self.set_source_reaction(&prompt.source, "👌").await;
        }

        Ok(())
    }
}

/// Register MCP for a topic work directory.
/// The MCP --data-dir points to base_dir (shared DB), but registration is local to work_dir.
async fn register_mcp_for_dir(work_dir: &std::path::Path, data_dir: &std::path::Path) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                "Cannot determine executable path, skipping MCP registration for {}: {}",
                work_dir.display(),
                e
            );
            return;
        }
    };
    // Linux appends " (deleted)" to /proc/self/exe when the binary is replaced by a rebuild.
    // Strip it so the registered MCP command points to the actual binary path.
    let exe_str = exe.display().to_string().trim_end_matches(" (deleted)").to_string();
    let data_dir_str = data_dir.display().to_string();

    // Remove stale registration (ignore errors)
    let _ = tokio::process::Command::new("claude")
        .args(["mcp", "remove", "-s", "local", "twolebot"])
        .current_dir(work_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .await;

    match tokio::process::Command::new("claude")
        .args([
            "mcp",
            "add",
            "--transport",
            "stdio",
            "-s",
            "local",
            "twolebot",
            "--",
            &exe_str,
            "mcp-stdio",
            "--data-dir",
            &data_dir_str,
        ])
        .current_dir(work_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            tracing::info!(
                "Registered MCP (stdio) at local scope in topic dir {}",
                work_dir.display()
            );
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                "Failed to register MCP for topic dir {}: {}",
                work_dir.display(),
                stderr.trim()
            );
        }
        Err(e) => {
            tracing::warn!(
                "Claude CLI not found, skipping MCP registration for {}: {}",
                work_dir.display(),
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::PromptSource;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_manager_creation() {
        let dir = tempdir().unwrap();
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let response_feed =
            Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let settings_store =
            Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap());

        let manager = ClaudeManager::new(
            prompt_feed,
            response_feed,
            settings_store,
            "claude-opus-4-6",
            dir.path(),
            60_000,
        );

        assert!(manager.current_prompt_id().await.is_none());
    }

    #[tokio::test]
    async fn test_work_dir_resolution() {
        let dir = tempdir().unwrap();
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let response_feed =
            Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let settings_store =
            Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap());

        let manager = ClaudeManager::new(
            prompt_feed,
            response_feed,
            settings_store,
            "claude-opus-4-6",
            dir.path(),
            60_000,
        );

        // Main thread uses base_dir
        assert_eq!(manager.work_dir_for_topic(&None), dir.path().to_path_buf());

        // Topic uses base_dir/topics/{key}
        let topic = Some("123_456".to_string());
        assert_eq!(
            manager.work_dir_for_topic(&topic),
            dir.path().join("topics").join("123_456")
        );
    }

    #[tokio::test]
    async fn test_chat_harness_selection_and_fallback() {
        let dir = tempdir().unwrap();
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let response_feed =
            Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let settings_store =
            Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap());

        let manager = ClaudeManager::new(
            prompt_feed,
            response_feed,
            settings_store.clone(),
            "claude-opus-4-6",
            dir.path(),
            60_000,
        );

        settings_store.set_chat_harness("echo").unwrap();
        let (web_harness, _) = manager.resolve_harness_for_source(&PromptSource::web("conv-1"));
        assert_eq!(web_harness, "echo");
        let (cron_harness_echo, _) =
            manager.resolve_harness_for_source(&PromptSource::cron("job-1", "exec-1", "Job Name"));
        assert_eq!(cron_harness_echo, "echo");

        settings_store.set_chat_harness("not-registered").unwrap();
        let (fallback_web_harness, _) =
            manager.resolve_harness_for_source(&PromptSource::web("conv-2"));
        assert_eq!(
            fallback_web_harness,
            crate::claude::harness::DEFAULT_HARNESS
        );

        let (fallback_cron_harness, _) =
            manager.resolve_harness_for_source(&PromptSource::cron("job-1", "exec-1", "Job Name"));
        assert_eq!(
            fallback_cron_harness,
            crate::claude::harness::DEFAULT_HARNESS
        );
    }
}
