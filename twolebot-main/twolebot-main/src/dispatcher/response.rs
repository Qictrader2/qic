use crate::cron::ActivityTracker;
use crate::error::Result;
use crate::server::chat_ws::{ChatEvent, ChatEventHub};
use crate::storage::{
    ActiveChatRegistry, ChatMetadataStore, CronTopicStore, MainTopicStore, MessageStore, Protocol,
    ResponseFeed, ResponseItem, StoredMessage,
};
use crate::telegram::send::TelegramSender;
use chrono::{Duration as ChronoDuration, Utc};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 1000;
const NO_TARGETS_RETRY_DELAY_SECS: i64 = 5;

/// Broadcasts responses to ALL connected consumers.
///
/// This is true broadcast architecture: every response goes to every
/// active chat across all protocols. The ActiveChatRegistry tracks
/// which chats have connected to each protocol.
pub struct ResponseBroadcaster {
    response_feed: Arc<ResponseFeed>,
    message_store: Arc<MessageStore>,
    active_chats: Arc<ActiveChatRegistry>,
    telegram_sender: Option<Arc<TelegramSender>>,
    activity_tracker: Option<ActivityTracker>,
    main_topic_store: Option<Arc<MainTopicStore>>,
    cron_topic_store: Option<Arc<CronTopicStore>>,
    chat_event_hub: Option<Arc<ChatEventHub>>,
    chat_metadata_store: Option<Arc<ChatMetadataStore>>,
}

impl ResponseBroadcaster {
    pub fn new(
        response_feed: Arc<ResponseFeed>,
        message_store: Arc<MessageStore>,
        active_chats: Arc<ActiveChatRegistry>,
        telegram_sender: Option<Arc<TelegramSender>>,
    ) -> Self {
        Self {
            response_feed,
            message_store,
            active_chats,
            telegram_sender,
            activity_tracker: None,
            main_topic_store: None,
            cron_topic_store: None,
            chat_event_hub: None,
            chat_metadata_store: None,
        }
    }

    /// Get Telegram sender ref (panics if not configured — only call from Telegram dispatch paths)
    fn telegram(&self) -> &TelegramSender {
        self.telegram_sender
            .as_ref()
            .expect("Telegram sender required for Telegram dispatch")
    }

    /// Set the activity tracker for resetting idle time on response dispatch
    pub fn with_activity_tracker(mut self, tracker: ActivityTracker) -> Self {
        self.activity_tracker = Some(tracker);
        self
    }

    /// Enable self-healing topic routing (recreate Main topic if stale/deleted)
    pub fn with_main_topic_store(mut self, store: Arc<MainTopicStore>) -> Self {
        self.main_topic_store = Some(store);
        self
    }

    /// Enable lazy per-job topic creation and self-healing for cron responses
    pub fn with_cron_topic_store(mut self, store: Arc<CronTopicStore>) -> Self {
        self.cron_topic_store = Some(store);
        self
    }

    /// Enable web chat response delivery via WebSocket
    pub fn with_chat_event_hub(mut self, hub: Arc<ChatEventHub>, metadata: Arc<ChatMetadataStore>) -> Self {
        self.chat_event_hub = Some(hub);
        self.chat_metadata_store = Some(metadata);
        self
    }

    /// Start the broadcaster loop
    pub async fn start(self: Arc<Self>, poll_interval_ms: u64) {
        let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

        loop {
            interval.tick().await;

            // Process pending responses
            match self.response_feed.all_pending() {
                Ok(responses) => {
                    for response in responses {
                        if let Err(e) = self.broadcast_response(response).await {
                            tracing::error!("Error broadcasting response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error fetching pending responses: {}", e);
                }
            }
        }
    }

    /// Broadcast a single response to the appropriate consumers.
    ///
    /// For Telegram-sourced responses: route directly using the embedded chat_id and
    /// message_thread_id (return address pattern — no ActiveChatRegistry lookup needed).
    /// For routed cron responses: same direct routing using embedded chat_id/thread_id.
    /// For unrouted cron/system responses: broadcast to all active chats.
    async fn broadcast_response(&self, response: ResponseItem) -> Result<()> {
        // Direct routing for Telegram-sourced responses
        if let crate::storage::PromptSource::Telegram {
            chat_id,
            message_thread_id,
            ..
        } = &response.source
        {
            // Skip empty final-only chunks (used as stream terminators)
            if response.content.is_empty() && response.is_final {
                self.response_feed.mark_sent(
                    &response.id,
                    &response.prompt_id,
                    response.sequence,
                )?;
                return Ok(());
            }
            return self
                .deliver_to_telegram(&response, *chat_id, *message_thread_id)
                .await;
        }

        // Web chat responses: deliver via WebSocket, store message, mark sent
        if let crate::storage::PromptSource::Web { conversation_id } = &response.source {
            return self.deliver_to_web(&response, conversation_id).await;
        }

        // Direct routing for cron responses that carry a target chat
        if let crate::storage::PromptSource::Cron {
            chat_id: Some(chat_id),
            message_thread_id,
            job_id,
            job_name,
            ..
        } = &response.source
        {
            // Agent loop tasks: only deliver the final response to Telegram.
            // Partial responses (tool calls, intermediate text) are silently consumed
            // to avoid flooding the chat with dozens of messages per task.
            if job_id.starts_with("agent-task-") && !response.is_final {
                self.response_feed.mark_sent(
                    &response.id,
                    &response.prompt_id,
                    response.sequence,
                )?;
                return Ok(());
            }

            // Resolve thread: use embedded thread_id, check cron store, or create lazily
            let thread_id = match message_thread_id {
                Some(tid) => {
                    // Check if the stored thread is still valid by checking cron topic store
                    // (another response may have self-healed to a new thread)
                    if let Some(ref store) = self.cron_topic_store {
                        if let Ok(Some(healed_tid)) = store.get(job_id, *chat_id) {
                            if healed_tid != *tid {
                                Some(healed_tid)
                            } else {
                                Some(*tid)
                            }
                        } else {
                            Some(*tid)
                        }
                    } else {
                        Some(*tid)
                    }
                }
                None => {
                    // No topic yet — create one lazily
                    self.ensure_cron_topic(job_id, job_name, *chat_id).await
                }
            };
            return self
                .deliver_cron_to_telegram(&response, *chat_id, thread_id)
                .await;
        }

        // Unrouted cron/system: broadcast to all active chats
        let targets = self.compute_broadcast_targets(response.user_id);

        if targets.is_empty() {
            // No place to deliver yet - defer and try again later (avoid tight 100ms loop)
            let next_attempt_at = Utc::now() + ChronoDuration::seconds(NO_TARGETS_RETRY_DELAY_SECS);
            if let Err(e) = self.response_feed.defer_until(
                &response.id,
                &response.prompt_id,
                response.sequence,
                next_attempt_at,
                None,
            ) {
                tracing::error!("Failed to defer response {} (no targets): {}", response.id, e);
            }
            tracing::debug!(
                "Deferring response {} (user {}) - no active targets",
                response.id,
                response.user_id
            );
            return Ok(());
        }

        // Broadcast to all targets
        let mut any_success = false;
        let mut last_error: Option<String> = None;

        for (protocol, chat_id) in &targets {
            let result = match protocol {
                Protocol::Telegram => {
                    self.send_to_telegram(chat_id, None, &response.content)
                        .await
                }
                Protocol::WhatsApp | Protocol::Slack | Protocol::Web => {
                    // Future: implement other protocols (Web uses WebSocket, not broadcast)
                    tracing::debug!("Skipping {} (not broadcast-routable)", protocol.as_str());
                    continue;
                }
            };

            match result {
                Ok(()) => {
                    any_success = true;
                    tracing::debug!(
                        "Response {} sent to {} chat {}",
                        response.id,
                        protocol.as_str(),
                        chat_id
                    );

                    // Store outbound message for each successful send
                    let stored_msg = StoredMessage::outbound_with_user(
                        format!("resp-{}-{}", response.id, chat_id),
                        chat_id,
                        response.user_id,
                        &response.content,
                    );
                    if let Err(e) = self.message_store.store(stored_msg) {
                        tracing::warn!("Failed to store outbound message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to send response {} to {} chat {}: {}",
                        response.id,
                        protocol.as_str(),
                        chat_id,
                        e
                    );
                    last_error = Some(e.to_string());
                }
            }
        }

        // Handle result based on whether any sends succeeded
        if any_success {
            self.response_feed
                .mark_sent(&response.id, &response.prompt_id, response.sequence)?;

            // Record human-facing interaction activity only for user-driven prompts.
            // Cron/agent-loop outputs should not reset idle windows for deferred work.
            if Self::should_record_human_activity(&response) {
                if let Some(ref tracker) = self.activity_tracker {
                    tracker.record_activity().await;
                }
            }
        } else if let Some(error) = last_error {
            // All sends failed
            let retry_count = response.retry_count.unwrap_or(0);
            if retry_count < MAX_RETRIES {
                let delay_ms = INITIAL_RETRY_DELAY_MS * 2u64.pow(retry_count);
                let next_attempt_at = Utc::now()
                    + ChronoDuration::milliseconds(delay_ms.try_into().unwrap_or(i64::MAX));
                tracing::warn!(
                    "Response {} failed all targets (attempt {}/{}), will retry in {}ms",
                    response.id,
                    retry_count + 1,
                    MAX_RETRIES,
                    delay_ms
                );
                self.response_feed.increment_retry(
                    &response.id,
                    &response.prompt_id,
                    response.sequence,
                    error,
                    Some(next_attempt_at),
                )?;
            } else {
                tracing::error!(
                    "Response {} permanently failed after {} retries",
                    response.id,
                    MAX_RETRIES
                );
                self.response_feed.mark_failed(
                    &response.id,
                    &response.prompt_id,
                    response.sequence,
                    format!("Max retries exceeded: {}", error),
                )?;
            }
        }

        Ok(())
    }

    fn compute_broadcast_targets(&self, user_id: i64) -> Vec<(Protocol, String)> {
        // Determine broadcast targets.
        //
        // - user_id != 0: route to the active chats for that user
        // - user_id == 0: system/cron -> broadcast to all active chats across all users
        //
        // IMPORTANT: De-dupe by (protocol, chat_id). A single Telegram group chat can
        // appear multiple times (one per user who interacted), and legacy migrations
        // can leave a user_id=0 entry that points at the same chat. Without de-dupe,
        // system/cron messages show up multiple times in the same chat.
        if user_id != 0 {
            return self.active_chats.get_broadcast_targets_for_user(user_id);
        }

        let mut seen: HashSet<(Protocol, String)> = HashSet::new();
        let mut unique: Vec<(Protocol, String)> = Vec::new();

        for (_uid, protocol, chat_id) in self.active_chats.get_broadcast_targets_all_users() {
            let key = (protocol, chat_id);
            if seen.insert(key.clone()) {
                unique.push(key);
            }
        }

        unique
    }

    fn should_record_human_activity(response: &ResponseItem) -> bool {
        matches!(
            response.source,
            crate::storage::PromptSource::Telegram { .. } | crate::storage::PromptSource::Web { .. }
        )
    }

    /// Deliver a Telegram-sourced response directly using its embedded return address.
    /// Self-heals stale topic IDs: if send fails with "thread not found", creates
    /// a new Main topic and retries.
    async fn deliver_to_telegram(
        &self,
        response: &ResponseItem,
        chat_id: i64,
        message_thread_id: Option<i64>,
    ) -> Result<()> {
        let result = self
            .telegram()
            .send_message(chat_id, message_thread_id, &response.content)
            .await;

        // Self-heal: if thread not found and we have a topic store, recreate and retry
        let (result, actual_thread_id) = match (&result, message_thread_id, &self.main_topic_store)
        {
            (Err(e), Some(tid), Some(store)) if TelegramSender::is_thread_not_found(e) => {
                tracing::warn!(
                    "Thread {} not found for chat {} (response {}), self-healing Main topic",
                    tid, chat_id, response.id
                );
                match self
                    .telegram()
                    .ensure_main_topic(chat_id, store)
                    .await
                {
                    Ok(new_tid) => {
                        let retry = self
                            .telegram()
                            .send_message(chat_id, Some(new_tid), &response.content)
                            .await;
                        (retry, Some(new_tid))
                    }
                    Err(heal_err) => {
                        tracing::error!("Failed to self-heal Main topic: {}", heal_err);
                        (result, message_thread_id)
                    }
                }
            }
            _ => (result, message_thread_id),
        };

        self.finalize_delivery(response, chat_id, actual_thread_id, result).await
    }

    /// Deliver a Web-sourced response via WebSocket and store in message history.
    async fn deliver_to_web(&self, response: &ResponseItem, conversation_id: &str) -> Result<()> {
        tracing::info!(
            "deliver_to_web: conv={}, seq={}, is_final={}, content_len={}",
            conversation_id, response.sequence, response.is_final, response.content.len()
        );
        // Store outbound message (skip empty final-only chunks)
        if !response.content.is_empty() {
            let stored_msg = StoredMessage::outbound_with_user(
                format!("resp-{}-web", response.id),
                conversation_id,
                response.user_id,
                &response.content,
            );
            if let Err(e) = self.message_store.store(stored_msg) {
                tracing::warn!("Failed to store web outbound message: {}", e);
            }
        }

        // Send via WebSocket if hub is available
        if let Some(ref hub) = self.chat_event_hub {
            hub.send(
                conversation_id,
                ChatEvent::MessageChunk {
                    conversation_id: conversation_id.to_string(),
                    content: response.content.clone(),
                    sequence: response.sequence,
                    is_final: response.is_final,
                },
            )
            .await;
        }

        // Mark as sent
        self.response_feed.mark_sent(
            &response.id,
            &response.prompt_id,
            response.sequence,
        )?;

        // Update last message preview (skip empty final-only chunks)
        if !response.content.is_empty() {
            if let Some(ref meta_store) = self.chat_metadata_store {
                let preview = if response.content.len() > 100 {
                    format!("{}...", &response.content[..97])
                } else {
                    response.content.clone()
                };
                let _ = meta_store.upsert_full(
                    conversation_id,
                    None,
                    None,
                    None,
                    Some("web"),
                    Some(&preview),
                );
            }
        }

        // Record activity for user-driven prompts
        if Self::should_record_human_activity(response) {
            if let Some(ref tracker) = self.activity_tracker {
                tracker.record_activity().await;
            }
        }

        tracing::debug!(
            "Response {} sent to web chat {}",
            response.id,
            conversation_id,
        );

        Ok(())
    }

    /// Ensure a forum topic exists for a cron job, creating one lazily if needed.
    /// Returns the thread_id or None if creation fails.
    async fn ensure_cron_topic(&self, job_id: &str, job_name: &str, chat_id: i64) -> Option<i64> {
        let store = self.cron_topic_store.as_ref()?;

        // Check store first (another response may have created it)
        if let Ok(Some(tid)) = store.get(job_id, chat_id) {
            return Some(tid);
        }

        // Create a new forum topic named after the job
        let topic_name = format!("\u{1F552} {}", job_name);
        match self
            .telegram()
            .create_forum_topic(chat_id, &topic_name)
            .await
        {
            Ok(thread_id) => {
                if let Err(e) = store.set(job_id, chat_id, thread_id) {
                    tracing::warn!("Failed to persist cron topic mapping: {}", e);
                }
                tracing::info!(
                    "Created forum topic '{}' (thread {}) for cron job {}",
                    job_name,
                    thread_id,
                    job_id
                );
                Some(thread_id)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create forum topic '{}' for cron job {}: {}",
                    job_name,
                    job_id,
                    e
                );
                None
            }
        }
    }

    /// Deliver a cron response to Telegram with self-healing topic routing.
    /// If "thread not found", creates a new topic and retries.
    async fn deliver_cron_to_telegram(
        &self,
        response: &ResponseItem,
        chat_id: i64,
        message_thread_id: Option<i64>,
    ) -> Result<()> {
        let result = self
            .telegram()
            .send_message(chat_id, message_thread_id, &response.content)
            .await;

        // Extract job info for self-healing
        let (job_id, job_name) = match &response.source {
            crate::storage::PromptSource::Cron {
                job_id, job_name, ..
            } => (job_id.as_str(), job_name.as_str()),
            _ => return self.finalize_delivery(response, chat_id, message_thread_id, result).await,
        };

        // Self-heal: if thread not found, check cron store for an already-healed
        // mapping before creating a new topic (prevents duplicate topic creation
        // when multiple responses fail in quick succession).
        let (result, actual_thread_id) = match (&result, message_thread_id, &self.cron_topic_store)
        {
            (Err(e), Some(tid), Some(store)) if TelegramSender::is_thread_not_found(e) => {
                tracing::warn!(
                    "Thread {} not found for cron job '{}' in chat {} (response {})",
                    tid, job_name, chat_id, response.id
                );
                // First: check if another response already self-healed this job
                if let Ok(Some(existing_tid)) = store.get(job_id, chat_id) {
                    if Some(existing_tid) != message_thread_id {
                        tracing::info!(
                            "Using already-healed cron topic for '{}': thread {} in chat {}",
                            job_name, existing_tid, chat_id
                        );
                        let retry = self
                            .telegram()
                            .send_message(chat_id, Some(existing_tid), &response.content)
                            .await;
                        (retry, Some(existing_tid))
                    } else {
                        // Store has the same stale tid — fall through to create new
                        Self::create_healed_topic(
                            self.telegram(), store, job_id, job_name, chat_id,
                            &response.content, result, message_thread_id,
                        ).await
                    }
                } else {
                    Self::create_healed_topic(
                        self.telegram(), store, job_id, job_name, chat_id,
                        &response.content, result, message_thread_id,
                    ).await
                }
            }
            _ => (result, message_thread_id),
        };

        self.finalize_delivery(response, chat_id, actual_thread_id, result).await
    }

    /// Create a new forum topic to self-heal a stale/deleted cron topic, retry sending.
    async fn create_healed_topic(
        sender: &TelegramSender,
        store: &CronTopicStore,
        job_id: &str,
        job_name: &str,
        chat_id: i64,
        content: &str,
        original_result: Result<Vec<i64>>,
        original_thread_id: Option<i64>,
    ) -> (Result<Vec<i64>>, Option<i64>) {
        tracing::warn!(
            "Thread not found for cron job '{}' in chat {}, creating new topic",
            job_name, chat_id
        );
        let topic_name = format!("\u{1F552} {}", job_name);
        match sender.create_forum_topic(chat_id, &topic_name).await {
            Ok(new_tid) => {
                if let Err(e) = store.set(job_id, chat_id, new_tid) {
                    tracing::warn!("Failed to update cron topic mapping: {}", e);
                }
                tracing::info!(
                    "Self-healed cron topic '{}' → thread {} in chat {}",
                    job_name, new_tid, chat_id
                );
                let retry = sender
                    .send_message(chat_id, Some(new_tid), content)
                    .await;
                (retry, Some(new_tid))
            }
            Err(heal_err) => {
                tracing::error!("Failed to self-heal cron topic '{}': {}", job_name, heal_err);
                (original_result, original_thread_id)
            }
        }
    }

    /// Common delivery finalization: mark sent/retry/failed, store message, record activity.
    async fn finalize_delivery(
        &self,
        response: &ResponseItem,
        chat_id: i64,
        thread_id: Option<i64>,
        result: Result<Vec<i64>>,
    ) -> Result<()> {
        match result {
            Ok(_) => {
                self.response_feed.mark_sent(
                    &response.id,
                    &response.prompt_id,
                    response.sequence,
                )?;

                let stored_msg = StoredMessage::outbound_with_user(
                    format!("resp-{}-{}", response.id, chat_id),
                    chat_id.to_string(),
                    response.user_id,
                    &response.content,
                )
                .with_topic_id(thread_id);
                if let Err(e) = self.message_store.store(stored_msg) {
                    tracing::warn!("Failed to store outbound message: {}", e);
                }

                if Self::should_record_human_activity(response) {
                    if let Some(ref tracker) = self.activity_tracker {
                        tracker.record_activity().await;
                    }
                }

                tracing::debug!(
                    "Response {} sent to Telegram chat {} thread {:?}",
                    response.id,
                    chat_id,
                    thread_id
                );
            }
            Err(e) => {
                let retry_count = response.retry_count.unwrap_or(0);
                if retry_count < MAX_RETRIES {
                    let delay_ms = INITIAL_RETRY_DELAY_MS * 2u64.pow(retry_count);
                    let next_attempt_at = Utc::now()
                        + ChronoDuration::milliseconds(delay_ms.try_into().unwrap_or(i64::MAX));
                    tracing::warn!(
                        "Response {} failed (chat {} thread {:?}, attempt {}/{}), will retry in {}ms: {}",
                        response.id,
                        chat_id,
                        thread_id,
                        retry_count + 1,
                        MAX_RETRIES,
                        delay_ms,
                        e
                    );
                    self.response_feed.increment_retry(
                        &response.id,
                        &response.prompt_id,
                        response.sequence,
                        e.to_string(),
                        Some(next_attempt_at),
                    )?;
                } else {
                    tracing::error!(
                        "Response {} permanently failed after {} retries (chat {} thread {:?}): {}",
                        response.id,
                        MAX_RETRIES,
                        chat_id,
                        thread_id,
                        e
                    );
                    self.response_feed.mark_failed(
                        &response.id,
                        &response.prompt_id,
                        response.sequence,
                        format!("Max retries exceeded: {}", e),
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Send to a Telegram chat (for cron/system broadcast)
    async fn send_to_telegram(
        &self,
        chat_id: &str,
        message_thread_id: Option<i64>,
        content: &str,
    ) -> Result<()> {
        let chat_id: i64 = chat_id
            .parse()
            .map_err(|_| crate::error::TwolebotError::other("Invalid chat_id"))?;

        self.telegram()
            .send_message(chat_id, message_thread_id, content)
            .await?;

        Ok(())
    }

    /// Get stats about response processing
    pub fn stats(&self) -> Result<BroadcasterStats> {
        let pending = self.response_feed.all_pending()?.len();
        let recent_sent = self.response_feed.recent_sent(100)?.len();
        let recent_failed = self.response_feed.recent_failed(100)?.len();
        let active_targets = self.active_chats.get_broadcast_targets_all_users().len();

        Ok(BroadcasterStats {
            pending,
            recent_sent,
            recent_failed,
            active_targets,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BroadcasterStats {
    pub pending: usize,
    pub recent_sent: usize,
    pub recent_failed: usize,
    pub active_targets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_broadcaster_creation() {
        let dir = tempdir().unwrap();
        let response_feed = Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let message_store = Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap());
        let active_chats = Arc::new(ActiveChatRegistry::new(dir.path().join("runtime.sqlite3")).unwrap());
        let telegram_sender = Some(Arc::new(TelegramSender::new("test_token").unwrap()));

        let broadcaster = ResponseBroadcaster::new(
            response_feed,
            message_store,
            active_chats.clone(),
            telegram_sender,
        );

        let stats = broadcaster.stats().unwrap();
        assert_eq!(stats.active_targets, 0);

        // Register a chat
        active_chats
            .set_active(42, Protocol::Telegram, "123456")
            .unwrap();
        let stats = broadcaster.stats().unwrap();
        assert_eq!(stats.active_targets, 1);
    }

    #[test]
    fn test_system_broadcast_targets_are_deduped_by_chat() {
        let dir = tempdir().unwrap();
        let response_feed = Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let message_store = Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap());
        let active_chats = Arc::new(ActiveChatRegistry::new(dir.path().join("runtime.sqlite3")).unwrap());
        let telegram_sender = Some(Arc::new(TelegramSender::new("test_token").unwrap()));

        // Same Telegram chat can show up multiple times (e.g., legacy user_id=0 mapping
        // plus a real user, or multiple users in the same group chat).
        active_chats
            .set_active(0, Protocol::Telegram, "123456")
            .unwrap();
        active_chats
            .set_active(42, Protocol::Telegram, "123456")
            .unwrap();

        let broadcaster =
            ResponseBroadcaster::new(response_feed, message_store, active_chats, telegram_sender);

        let targets = broadcaster.compute_broadcast_targets(0);
        assert_eq!(targets, vec![(Protocol::Telegram, "123456".to_string())]);
    }

    #[test]
    fn test_human_activity_only_for_user_driven_sources() {
        let telegram = ResponseItem::new(
            "p1",
            crate::storage::PromptSource::Telegram {
                update_id: 1,
                message_id: 1,
                chat_id: 12345,
                message_thread_id: None,
            },
            42,
            "hi",
            true,
            0,
        );
        let cron = ResponseItem::new(
            "p2",
            crate::storage::PromptSource::cron("job", "exec", "Job"),
            0,
            "tick",
            true,
            0,
        );

        assert!(ResponseBroadcaster::should_record_human_activity(&telegram));
        assert!(!ResponseBroadcaster::should_record_human_activity(&cron));
    }
}
