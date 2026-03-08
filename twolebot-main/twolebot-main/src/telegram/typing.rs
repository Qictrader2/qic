use crate::storage::PromptFeed;
use crate::telegram::send::TelegramSender;
use crate::telegram::types::ChatAction;
use std::sync::Arc;
use std::time::Duration;

/// Sends typing indicators for all currently processing prompts.
///
/// Uses the return-address pattern: each running prompt carries its own
/// chat_id and message_thread_id in its PromptSource, so typing goes
/// directly to the correct chat/topic without any registry lookup.
pub struct TypingIndicator {
    prompt_feed: Arc<PromptFeed>,
    sender: Arc<TelegramSender>,
}

impl TypingIndicator {
    pub fn new(prompt_feed: Arc<PromptFeed>, sender: Arc<TelegramSender>) -> Self {
        Self {
            prompt_feed,
            sender,
        }
    }

    /// Start the typing indicator loop
    pub async fn start(self, interval_secs: u64) {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            // Send typing for ALL running prompts (multi-worker: one per topic)
            if let Ok(running) = self.prompt_feed.get_all_running() {
                for prompt in running {
                    // Only send typing for user-driven prompts, not cron/system
                    if let crate::storage::PromptSource::Telegram {
                        chat_id,
                        message_thread_id,
                        ..
                    } = &prompt.source
                    {
                        let result = self
                            .sender
                            .send_chat_action(*chat_id, *message_thread_id, ChatAction::Typing)
                            .await;
                        match result {
                            Ok(()) => tracing::debug!(
                                "Typing sent: chat={} thread={:?}",
                                chat_id, message_thread_id
                            ),
                            Err(e) => tracing::warn!(
                                "Typing failed: chat={} thread={:?}: {}",
                                chat_id, message_thread_id, e
                            ),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::prompt_feed::{PromptItem, PromptSource};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_typing_indicator_creation() {
        let dir = tempdir().unwrap();
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let sender = Arc::new(TelegramSender::new("test_token").unwrap());

        let indicator = TypingIndicator::new(prompt_feed, sender);
        drop(indicator);
    }

    #[tokio::test]
    async fn test_typing_derives_from_feed_state() {
        let dir = tempdir().unwrap();
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());

        // Initially nothing running
        assert!(prompt_feed.get_all_running().unwrap().is_empty());

        // Enqueue and mark running
        let source = PromptSource::telegram(1, 1, 12345, None);
        let item = PromptItem::new(source, 1, "test prompt");
        let item = prompt_feed.enqueue(item).unwrap();
        let _ = prompt_feed.mark_running(&item.id);

        // Now something is running
        let running = prompt_feed.get_all_running().unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].user_id, 1);

        // The running prompt carries its own return address
        if let PromptSource::Telegram { chat_id, .. } = &running[0].source {
            assert_eq!(*chat_id, 12345);
        } else {
            panic!("Expected Telegram source");
        }

        // Mark completed
        let _ = prompt_feed.mark_completed(&item.id);

        // Nothing running again
        assert!(prompt_feed.get_all_running().unwrap().is_empty());
    }
}
