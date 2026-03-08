use crate::error::{Result, TwolebotError};
use crate::rendering::telegram::render_telegram_html_chunks;
use crate::storage::MainTopicStore;
use crate::telegram::types::{ChatAction, ReactionType, TelegramResponse};
use std::time::Duration;

/// Telegram message sender
pub struct TelegramSender {
    client: reqwest::Client,
    token: String,
}

impl TelegramSender {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| TwolebotError::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            token: token.into(),
        })
    }

    /// Send a text message, automatically splitting if too long
    pub async fn send_message(
        &self,
        chat_id: i64,
        message_thread_id: Option<i64>,
        text: &str,
    ) -> Result<Vec<i64>> {
        // Render once into Telegram-safe HTML chunks. This avoids the legacy
        // Markdown parser's frequent "can't parse entities" failures that
        // silently downgrade messages to plain text (breaking code blocks/grids).
        let chunks = render_telegram_html_chunks(text);
        let mut message_ids = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            let message_text = if chunks.len() > 1 {
                format!("[{}/{}]\n{}", i + 1, chunks.len(), chunk)
            } else {
                chunk.clone()
            };

            let msg_id = self
                .send_single_message_html(chat_id, message_thread_id, &message_text)
                .await?;
            message_ids.push(msg_id);

            // Small delay between chunks to avoid rate limiting
            if i < chunks.len() - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(message_ids)
    }

    /// Send a single message rendered as Telegram HTML.
    async fn send_single_message_html(
        &self,
        chat_id: i64,
        message_thread_id: Option<i64>,
        text: &str,
    ) -> Result<i64> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        let mut body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML"
        });
        if let Some(tid) = message_thread_id {
            body["message_thread_id"] = serde_json::json!(tid);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::telegram(format!(
                "sendMessage failed: {} - {}",
                status, body
            )));
        }

        // Parse response to get message_id
        let telegram_response: TelegramResponse<serde_json::Value> = serde_json::from_str(&body)?;

        if !telegram_response.ok {
            return Err(TwolebotError::telegram(
                telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let message_id = telegram_response
            .result
            .and_then(|r| r.get("message_id").and_then(|v| v.as_i64()))
            .ok_or_else(|| TwolebotError::telegram("No message_id in response"))?;

        Ok(message_id)
    }

    /// Set a reaction on a message
    pub async fn set_reaction(&self, chat_id: i64, message_id: i64, emoji: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/setMessageReaction",
            self.token
        );

        let reaction = ReactionType::emoji(emoji);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "reaction": [reaction]
            }))
            .send()
            .await?;

        // Reactions can fail silently (e.g., bot doesn't have permission)
        // We don't treat this as a critical error
        if !response.status().is_success() {
            tracing::debug!("setMessageReaction failed (non-critical)");
        }

        Ok(())
    }

    /// Send a chat action (typing indicator, etc.)
    pub async fn send_chat_action(
        &self,
        chat_id: i64,
        message_thread_id: Option<i64>,
        action: ChatAction,
    ) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendChatAction", self.token);

        let mut body = serde_json::json!({
            "chat_id": chat_id,
            "action": action
        });
        if let Some(tid) = message_thread_id {
            body["message_thread_id"] = serde_json::json!(tid);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TwolebotError::telegram(format!(
                "sendChatAction failed: {}",
                body
            )));
        }

        Ok(())
    }

    /// Set bot commands (menu)
    pub async fn set_my_commands(&self, commands: &[(&str, &str)]) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/setMyCommands", self.token);

        let commands_json: Vec<serde_json::Value> = commands
            .iter()
            .map(|(cmd, desc)| {
                serde_json::json!({
                    "command": cmd,
                    "description": desc
                })
            })
            .collect();

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "commands": commands_json
            }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TwolebotError::telegram(format!(
                "setMyCommands failed: {} - {}",
                status, body
            )));
        }

        tracing::info!(
            "Bot commands registered: {:?}",
            commands.iter().map(|(c, _)| c).collect::<Vec<_>>()
        );
        Ok(())
    }

    /// Create a forum topic in a chat. Returns the new topic's message_thread_id.
    /// Used to lazily create a "Main" topic for DM threaded mode.
    pub async fn create_forum_topic(&self, chat_id: i64, name: &str) -> Result<i64> {
        let url = format!(
            "https://api.telegram.org/bot{}/createForumTopic",
            self.token
        );

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "name": name
            }))
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::telegram(format!(
                "createForumTopic failed: {} - {}",
                status, body
            )));
        }

        let telegram_response: TelegramResponse<serde_json::Value> =
            serde_json::from_str(&body)?;

        if !telegram_response.ok {
            return Err(TwolebotError::telegram(
                telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        telegram_response
            .result
            .and_then(|r| r.get("message_thread_id").and_then(|v| v.as_i64()))
            .ok_or_else(|| TwolebotError::telegram("No message_thread_id in createForumTopic response"))
    }

    /// Delete a forum topic in a Telegram chat.
    pub async fn delete_forum_topic(&self, chat_id: i64, message_thread_id: i64) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/deleteForumTopic",
            self.token
        );

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "message_thread_id": message_thread_id
            }))
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::telegram(format!(
                "deleteForumTopic failed: {} - {}",
                status, body
            )));
        }

        Ok(())
    }

    /// Send a document (file) to a chat via multipart upload.
    /// Returns the message_id of the sent document.
    pub async fn send_document(
        &self,
        chat_id: i64,
        message_thread_id: Option<i64>,
        file_data: Vec<u8>,
        filename: String,
        caption: Option<&str>,
    ) -> Result<i64> {
        let url = format!("https://api.telegram.org/bot{}/sendDocument", self.token);

        let file_part = reqwest::multipart::Part::bytes(file_data)
            .file_name(filename)
            .mime_str("application/octet-stream")
            .map_err(|e| TwolebotError::telegram(format!("multipart mime: {e}")))?;

        let mut form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", file_part);

        if let Some(tid) = message_thread_id {
            form = form.text("message_thread_id", tid.to_string());
        }
        if let Some(cap) = caption {
            form = form.text("caption", cap.to_string()).text("parse_mode", "HTML");
        }

        let response = self.client.post(&url).multipart(form).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::telegram(format!(
                "sendDocument failed: {} - {}",
                status, body
            )));
        }

        let telegram_response: TelegramResponse<serde_json::Value> = serde_json::from_str(&body)?;

        if !telegram_response.ok {
            return Err(TwolebotError::telegram(
                telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        telegram_response
            .result
            .and_then(|r| r.get("message_id").and_then(|v| v.as_i64()))
            .ok_or_else(|| TwolebotError::telegram("No message_id in sendDocument response"))
    }

    /// Send a photo to a chat via multipart upload (displays inline preview).
    /// Returns the message_id of the sent photo.
    pub async fn send_photo(
        &self,
        chat_id: i64,
        message_thread_id: Option<i64>,
        photo_data: Vec<u8>,
        filename: String,
        caption: Option<&str>,
    ) -> Result<i64> {
        let url = format!("https://api.telegram.org/bot{}/sendPhoto", self.token);

        let file_part = reqwest::multipart::Part::bytes(photo_data)
            .file_name(filename)
            .mime_str("image/png")
            .map_err(|e| TwolebotError::telegram(format!("multipart mime: {e}")))?;

        let mut form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("photo", file_part);

        if let Some(tid) = message_thread_id {
            form = form.text("message_thread_id", tid.to_string());
        }
        if let Some(cap) = caption {
            form = form
                .text("caption", cap.to_string())
                .text("parse_mode", "HTML");
        }

        let response = self.client.post(&url).multipart(form).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::telegram(format!(
                "sendPhoto failed: {} - {}",
                status, body
            )));
        }

        let telegram_response: TelegramResponse<serde_json::Value> =
            serde_json::from_str(&body)?;

        if !telegram_response.ok {
            return Err(TwolebotError::telegram(
                telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        telegram_response
            .result
            .and_then(|r| r.get("message_id").and_then(|v| v.as_i64()))
            .ok_or_else(|| TwolebotError::telegram("No message_id in sendPhoto response"))
    }

    /// Check if a Telegram API error indicates the thread/topic was not found.
    pub fn is_thread_not_found(err: &TwolebotError) -> bool {
        match err {
            TwolebotError::TelegramApi { message } => {
                message.contains("message thread not found")
            }
            _ => false,
        }
    }

    /// Ensure a valid Main topic exists for a private chat. If the stored topic
    /// is stale or missing, create a new one and update the store.
    ///
    /// Returns the valid message_thread_id.
    pub async fn ensure_main_topic(
        &self,
        chat_id: i64,
        main_topic_store: &MainTopicStore,
    ) -> Result<i64> {
        // Check if we have a stored topic
        if let Some(thread_id) = main_topic_store.get(chat_id)? {
            // Probe: try sending a chat action to verify the topic still exists
            let probe_result = self.send_chat_action(
                chat_id,
                Some(thread_id),
                ChatAction::Typing,
            ).await;

            match probe_result {
                Ok(()) => return Ok(thread_id),
                Err(ref e) if Self::is_thread_not_found(e) => {
                    tracing::warn!(
                        "Main topic {} for chat {} is stale, creating a new one",
                        thread_id,
                        chat_id
                    );
                }
                Err(e) => return Err(e),
            }
        }

        // Create a new Main topic
        let new_thread_id = self.create_forum_topic(chat_id, "Main").await?;
        main_topic_store.set(chat_id, new_thread_id)?;
        tracing::info!(
            "Created new Main topic {} for chat {}",
            new_thread_id,
            chat_id
        );
        Ok(new_thread_id)
    }

    /// Reply to a specific message
    pub async fn reply_to_message(
        &self,
        chat_id: i64,
        reply_to_message_id: i64,
        text: &str,
    ) -> Result<i64> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
                "reply_to_message_id": reply_to_message_id
            }))
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::telegram(format!(
                "sendMessage (reply) failed: {} - {}",
                status, body
            )));
        }

        let telegram_response: TelegramResponse<serde_json::Value> = serde_json::from_str(&body)?;

        let message_id = telegram_response
            .result
            .and_then(|r| r.get("message_id").and_then(|v| v.as_i64()))
            .ok_or_else(|| TwolebotError::telegram("No message_id in response"))?;

        Ok(message_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_creation() {
        let sender = TelegramSender::new("test_token").unwrap();
        assert!(!sender.token.is_empty());
    }
}
