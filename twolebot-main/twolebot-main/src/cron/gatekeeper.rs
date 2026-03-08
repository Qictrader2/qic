use crate::cron::feed::CronFeed;
use crate::cron::types::CronExecution;
use crate::storage::{
    ActiveChatRegistry, CronTopicStore, PromptFeed, PromptItem, PromptSource, Protocol,
    ResponseFeed, ResponseItem,
};
use crate::telegram::send::TelegramSender;
use std::sync::Arc;

/// Promotes waiting cron executions to the main PromptFeed.
/// Each cron job fires in its own dedicated Telegram forum topic,
/// so all waiting executions are promoted immediately without guards.
pub struct CronGatekeeper {
    cron_feed: Arc<CronFeed>,
    prompt_feed: Arc<PromptFeed>,
    response_feed: Option<Arc<ResponseFeed>>,
    cron_topic_store: Option<Arc<CronTopicStore>>,
    active_chats: Option<Arc<ActiveChatRegistry>>,
    telegram_sender: Option<Arc<TelegramSender>>,
}

impl CronGatekeeper {
    pub fn new(
        cron_feed: Arc<CronFeed>,
        prompt_feed: Arc<PromptFeed>,
    ) -> Self {
        Self {
            cron_feed,
            prompt_feed,
            response_feed: None,
            cron_topic_store: None,
            active_chats: None,
            telegram_sender: None,
        }
    }

    /// Enable acknowledgment messages when jobs are promoted
    pub fn with_response_feed(mut self, response_feed: Arc<ResponseFeed>) -> Self {
        self.response_feed = Some(response_feed);
        self
    }

    /// Enable topic-aware routing for cron prompts/responses
    pub fn with_topic_routing(
        mut self,
        cron_topic_store: Arc<CronTopicStore>,
        active_chats: Arc<ActiveChatRegistry>,
    ) -> Self {
        self.cron_topic_store = Some(cron_topic_store);
        self.active_chats = Some(active_chats);
        self
    }

    /// Enable eager topic creation (creates Telegram forum topics at promotion time)
    pub fn with_telegram_sender(mut self, sender: Arc<TelegramSender>) -> Self {
        self.telegram_sender = Some(sender);
        self
    }

    /// Start the gatekeeper loop
    pub async fn start(self: Arc<Self>, poll_interval_ms: u64) {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_millis(poll_interval_ms));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_and_promote().await {
                tracing::error!("Error in cron gatekeeper: {}", e);
            }
        }
    }

    /// Resolve the target chat_id for cron delivery.
    /// Prefers the job's stored origin_chat_id, falls back to first active Telegram chat.
    fn resolve_chat_id(&self, origin_chat_id: Option<i64>) -> Option<i64> {
        if let Some(cid) = origin_chat_id {
            return Some(cid);
        }
        let active_chats = self.active_chats.as_ref()?;
        let targets = active_chats.get_broadcast_targets_all_users();
        targets
            .iter()
            .find(|(_, proto, _)| *proto == Protocol::Telegram)
            .and_then(|(_, _, cid)| cid.parse::<i64>().ok())
    }

    /// Resolve per-job thread_id from CronTopicStore.
    fn resolve_job_thread_id(&self, job_id: &str, chat_id: i64) -> Option<i64> {
        self.cron_topic_store
            .as_ref()
            .and_then(|store| store.get(job_id, chat_id).ok().flatten())
    }

    /// Eagerly create a Telegram forum topic for a cron job.
    /// This ensures the agent runs in the correct topic directory from the start,
    /// so conversation history is preserved for user replies.
    async fn ensure_job_topic(&self, job_id: &str, job_name: &str, chat_id: i64) -> Option<i64> {
        let sender = self.telegram_sender.as_ref()?;
        let store = self.cron_topic_store.as_ref()?;

        let topic_name = format!("\u{1F552} {}", job_name);
        match sender.create_forum_topic(chat_id, &topic_name).await {
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

    /// Promote all waiting executions to the prompt feed.
    /// Each cron job fires in its own topic, so no guards are needed.
    async fn check_and_promote(&self) -> crate::error::Result<()> {
        let waiting = self.cron_feed.list_waiting()?;
        if waiting.is_empty() {
            return Ok(());
        }

        for exec in &waiting {
            // Look up the job to get routing info and display name
            let job = self.cron_feed.get_job(&exec.job_id).ok().flatten();
            let job_name = job
                .as_ref()
                .map(|j| j.display_name().to_string())
                .unwrap_or_else(|| exec.job_id.clone());
            let origin_chat_id = job.as_ref().and_then(|j| j.origin_chat_id);

            let chat_id = self.resolve_chat_id(origin_chat_id);

            // Resolve or eagerly create per-job forum topic so the agent runs
            // in the correct topic directory (enabling conversation continuity).
            let thread_id = match chat_id {
                Some(cid) => match self.resolve_job_thread_id(&exec.job_id, cid) {
                    Some(tid) => Some(tid),
                    None => self.ensure_job_topic(&exec.job_id, &job_name, cid).await,
                },
                None => None,
            };

            // Promote the execution with routing info
            self.promote_execution(exec, chat_id, thread_id)?;
        }

        Ok(())
    }

    /// Promote a cron execution to the prompt feed
    fn promote_execution(
        &self,
        exec: &CronExecution,
        chat_id: Option<i64>,
        thread_id: Option<i64>,
    ) -> crate::error::Result<()> {
        // Look up the job to get the name (fall back to job_id if job was deleted)
        let job_name = self
            .cron_feed
            .get_job(&exec.job_id)?
            .map(|j| j.name.clone())
            .unwrap_or_else(|| exec.job_id.clone());

        tracing::info!(
            "Promoting cron execution {} (job {} / {}) to prompt feed (chat {:?}, thread {:?})",
            exec.id,
            exec.job_id,
            job_name,
            chat_id,
            thread_id,
        );

        // Create a prompt item with routing info (if available)
        let source = match chat_id {
            Some(cid) => {
                PromptSource::cron_routed(&exec.job_id, &exec.id, &job_name, cid, thread_id)
            }
            None => PromptSource::cron(&exec.job_id, &exec.id, &job_name),
        };

        // Wrap the prompt so the agent knows it's executing a scheduled task
        let wrapped_prompt = format!(
            "[SCHEDULED TASK — job '{}' (id: {})]\n\
             You are executing a scheduled cron job. Do NOT schedule another \
             reminder or cron job — you ARE the task. Execute the prompt below directly.\n\n\
             {}",
            job_name, exec.job_id, exec.prompt
        );

        let prompt_item = PromptItem::new(
            source,
            0, // No user_id for cron jobs
            &wrapped_prompt,
        );

        // Enqueue to prompt feed
        self.prompt_feed.enqueue(prompt_item)?;

        // Remove from waiting queue
        self.cron_feed.remove_execution(exec)?;

        // Send promotion acknowledgment
        self.send_promoted_acknowledgment(exec, &job_name, chat_id, thread_id)?;

        Ok(())
    }

    /// Single context notice right before a scheduled task runs.
    /// Provides context for the messages that follow from the agent.
    fn send_promoted_acknowledgment(
        &self,
        exec: &CronExecution,
        job_name: &str,
        chat_id: Option<i64>,
        thread_id: Option<i64>,
    ) -> crate::error::Result<()> {
        let Some(ref response_feed) = self.response_feed else {
            return Ok(());
        };

        let message = format!("Running scheduled task '{}'", job_name);

        let source = match chat_id {
            Some(cid) => PromptSource::cron_routed(
                &exec.job_id,
                "ack-promoted",
                job_name,
                cid,
                thread_id,
            ),
            None => PromptSource::cron(&exec.job_id, "ack-promoted", job_name),
        };
        let response = ResponseItem::new(
            "ack-promoted",
            source,
            0,
            message,
            true,
            0,
        );

        response_feed.enqueue(response)?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::types::{CronJob, CronSchedule};
    use tempfile::TempDir;

    fn create_test_feeds() -> (Arc<CronFeed>, Arc<PromptFeed>, TempDir) {
        let dir = TempDir::new().unwrap();
        let cron_feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        (cron_feed, prompt_feed, dir)
    }

    #[tokio::test]
    async fn test_check_and_promote_noop_when_empty() {
        let (cron_feed, prompt_feed, _tmpdir) = create_test_feeds();
        let gatekeeper = CronGatekeeper::new(cron_feed, prompt_feed);

        gatekeeper.check_and_promote().await.unwrap();
        assert!(gatekeeper.prompt_feed.next_pending().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_promotes_despite_pending_user_prompts() {
        // Cron jobs fire in their own topics, so pending user prompts don't block them.
        let (cron_feed, prompt_feed, _tmpdir) = create_test_feeds();
        let gatekeeper = CronGatekeeper::new(cron_feed.clone(), prompt_feed.clone());

        // Create a pending user prompt
        let prompt = PromptItem::new(
            PromptSource::Telegram {
                update_id: 1,
                message_id: 1,
                chat_id: 12345,
                message_thread_id: None,
            },
            456,
            "Test",
        );
        prompt_feed.enqueue(prompt).unwrap();

        // Create a waiting execution
        let job = CronJob::new("Test cron", "Test cron", CronSchedule::from_minutes(0));
        cron_feed.create_job(job.clone()).unwrap();
        let exec = CronExecution::from_job(&job, chrono::Utc::now());
        cron_feed.enqueue_execution(exec).unwrap();

        gatekeeper.check_and_promote().await.unwrap();
        assert_eq!(cron_feed.list_waiting().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_pending_cron_prompt_does_not_block_promotion() {
        let (cron_feed, prompt_feed, _tmpdir) = create_test_feeds();
        let gatekeeper = CronGatekeeper::new(cron_feed.clone(), prompt_feed.clone());

        let existing_cron_prompt = PromptItem::new(
            PromptSource::cron(
                "job-existing",
                "exec-existing",
                "Existing",
            ),
            0,
            "existing machine prompt",
        );
        prompt_feed.enqueue(existing_cron_prompt).unwrap();

        let job = CronJob::new("New cron", "New cron", CronSchedule::from_minutes(0));
        cron_feed.create_job(job.clone()).unwrap();
        let exec = CronExecution::from_job(&job, chrono::Utc::now());
        cron_feed.enqueue_execution(exec).unwrap();

        gatekeeper.check_and_promote().await.unwrap();

        assert_eq!(prompt_feed.all_pending().unwrap().len(), 2);
        assert!(cron_feed.list_waiting().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_promote_execution() {
        let (cron_feed, prompt_feed, _tmpdir) = create_test_feeds();
        let gatekeeper = CronGatekeeper::new(cron_feed.clone(), prompt_feed.clone());

        let job = CronJob::new("Test cron", "Test cron prompt", CronSchedule::from_minutes(0));
        cron_feed.create_job(job.clone()).unwrap();

        let exec = CronExecution::from_job(&job, chrono::Utc::now());
        cron_feed.enqueue_execution(exec.clone()).unwrap();

        gatekeeper.promote_execution(&exec, None, None).unwrap();

        assert_eq!(cron_feed.list_waiting().unwrap().len(), 0);

        let pending = prompt_feed.next_pending().unwrap().unwrap();
        assert!(
            pending.prompt.contains("Test cron prompt"),
            "Promoted prompt should contain original text, got: {}",
            pending.prompt
        );

        match pending.source {
            PromptSource::Cron {
                job_id,
                execution_id,
                ..
            } => {
                assert_eq!(job_id, job.id);
                assert_eq!(execution_id, exec.id);
            }
            _ => panic!("Expected Cron source"),
        }
    }

    #[tokio::test]
    async fn test_promotes_immediately_regardless_of_activity() {
        // No deferral: cron jobs always promote immediately
        let (cron_feed, prompt_feed, _tmpdir) = create_test_feeds();
        let gatekeeper = CronGatekeeper::new(cron_feed.clone(), prompt_feed.clone());

        let job = CronJob::new("Test", "Test", CronSchedule::from_minutes(0));
        cron_feed.create_job(job.clone()).unwrap();
        let exec = CronExecution::from_job(&job, chrono::Utc::now());
        cron_feed.enqueue_execution(exec).unwrap();

        gatekeeper.check_and_promote().await.unwrap();

        assert_eq!(cron_feed.list_waiting().unwrap().len(), 0);
        assert!(prompt_feed.next_pending().unwrap().is_some());
    }

    #[tokio::test]
    async fn test_origin_chat_id_used_for_routing() {
        let (cron_feed, prompt_feed, _tmpdir) = create_test_feeds();
        let gatekeeper = CronGatekeeper::new(cron_feed.clone(), prompt_feed.clone());

        let job = CronJob::new("Routed job", "Routed prompt", CronSchedule::from_minutes(0))
            .with_origin_chat_id(99999);
        cron_feed.create_job(job.clone()).unwrap();
        let exec = CronExecution::from_job(&job, chrono::Utc::now());
        cron_feed.enqueue_execution(exec).unwrap();

        // Without topic routing configured, origin_chat_id is used but no thread created
        gatekeeper.check_and_promote().await.unwrap();

        let pending = prompt_feed.next_pending().unwrap().unwrap();
        match &pending.source {
            PromptSource::Cron { chat_id, .. } => {
                assert_eq!(*chat_id, Some(99999));
            }
            _ => panic!("Expected Cron source"),
        }
    }
}
