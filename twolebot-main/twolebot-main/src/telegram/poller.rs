use crate::error::{Result, TwolebotError};
use crate::telegram::types::{TelegramResponse, Update};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

const LONG_POLL_TIMEOUT: u64 = 30;

/// Telegram long-polling update fetcher
pub struct TelegramPoller {
    client: reqwest::Client,
    token: String,
    offset: Arc<AtomicI64>,
}

impl TelegramPoller {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(LONG_POLL_TIMEOUT + 10))
            .build()
            .map_err(|e| TwolebotError::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            token: token.into(),
            offset: Arc::new(AtomicI64::new(0)),
        })
    }

    /// Set the initial offset (useful for resuming)
    pub fn set_offset(&self, offset: i64) {
        self.offset.store(offset, Ordering::SeqCst);
    }

    /// Get the current offset
    pub fn get_offset(&self) -> i64 {
        self.offset.load(Ordering::SeqCst)
    }

    /// Fetch updates once (blocking)
    pub async fn get_updates(&self) -> Result<Vec<Update>> {
        let offset = self.offset.load(Ordering::SeqCst);
        let url = format!("https://api.telegram.org/bot{}/getUpdates", self.token);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "offset": offset,
                "timeout": LONG_POLL_TIMEOUT,
                "allowed_updates": ["message", "edited_message", "callback_query"]
            }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TwolebotError::telegram(format!(
                "getUpdates failed: {} - {}",
                status, body
            )));
        }

        let telegram_response: TelegramResponse<Vec<Update>> = response.json().await?;

        if !telegram_response.ok {
            return Err(TwolebotError::telegram(
                telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let updates = telegram_response.result.unwrap_or_default();

        // Update offset to acknowledge received updates
        if let Some(last) = updates.last() {
            self.offset.store(last.update_id + 1, Ordering::SeqCst);
        }

        Ok(updates)
    }

    /// Start polling loop, sending updates to the provided channel
    pub async fn start_polling(self: Arc<Self>, tx: mpsc::Sender<Update>) -> Result<()> {
        loop {
            match self.get_updates().await {
                Ok(updates) => {
                    for update in updates {
                        if tx.send(update).await.is_err() {
                            // Receiver dropped, stop polling
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    // Log error but continue polling
                    tracing::error!("Telegram polling error: {}", e);
                    // Brief delay before retry on error
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Get file info from Telegram
    pub async fn get_file(&self, file_id: &str) -> Result<String> {
        let url = format!("https://api.telegram.org/bot{}/getFile", self.token);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "file_id": file_id }))
            .send()
            .await?;

        let telegram_response: TelegramResponse<crate::telegram::types::File> =
            response.json().await?;

        if !telegram_response.ok {
            return Err(TwolebotError::telegram(
                telegram_response
                    .description
                    .unwrap_or_else(|| "getFile failed".to_string()),
            ));
        }

        let file = telegram_response
            .result
            .ok_or_else(|| TwolebotError::telegram("No file in response"))?;

        file.file_path
            .ok_or_else(|| TwolebotError::telegram("No file_path in response"))
    }

    /// Download a file from Telegram
    pub async fn download_file(&self, file_path: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://api.telegram.org/file/bot{}/{}",
            self.token, file_path
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(TwolebotError::telegram(format!(
                "Failed to download file: {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Get file and download it in one call
    pub async fn download_file_by_id(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.get_file(file_id).await?;
        self.download_file(&file_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_management() {
        let poller = TelegramPoller::new("test_token").unwrap();

        assert_eq!(poller.get_offset(), 0);

        poller.set_offset(12345);
        assert_eq!(poller.get_offset(), 12345);
    }
}
