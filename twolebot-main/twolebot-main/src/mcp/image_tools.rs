use crate::server::chat_ws::{ChatEvent, ChatEventHub};
use crate::storage::media::{mime_for_extension, MediaStore};
use crate::storage::messages::{MessageStore, StoredMessage};
use crate::telegram::send::TelegramSender;
use crate::types::image::GenerateImageRequest;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const MODEL_PREMIUM: &str = "gemini-3-pro-image-preview";
const MODEL_FAST: &str = "gemini-2.5-flash-image";
const DEFAULT_IMAGE_SIZE: &str = "1K";
const DEFAULT_ASPECT_RATIO: &str = "1:1";
const MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY_SECS: u64 = 5;
const RETRY_BUFFER_SECS: u64 = 3;

const VALID_QUALITIES: &[&str] = &["premium", "fast"];
const VALID_IMAGE_SIZES: &[&str] = &["1K", "2K", "4K"];
const VALID_ASPECT_RATIOS: &[&str] = &[
    "1:1", "2:3", "3:2", "3:4", "4:3", "4:5", "5:4", "9:16", "16:9", "21:9",
];

// -- Gemini API request types (camelCase to match API) --

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiImageRequest {
    contents: Vec<RequestContent>,
    generation_config: ImageGenerationConfig,
}

#[derive(Debug, Serialize)]
struct RequestContent {
    parts: Vec<RequestPart>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum RequestPart {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inline_data")]
        inline_data: InlineData,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageGenerationConfig {
    response_modalities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_config: Option<ImageConfig>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_size: Option<String>,
}

// -- Gemini API response types --

#[derive(Debug, Deserialize)]
struct GeminiImageResponse {
    candidates: Option<Vec<ImageCandidate>>,
}

#[derive(Debug, Deserialize)]
struct ImageCandidate {
    content: Option<ImageResponseContent>,
}

#[derive(Debug, Deserialize)]
struct ImageResponseContent {
    parts: Option<Vec<ImageResponsePart>>,
}

// InlineData variant must be listed first for correct serde untagged dispatch.
// serde tries variants in order; if Text were first, any object with a `text`
// field would match before InlineData gets a chance.
// Gemini returns camelCase in responses: `inlineData`, `mimeType`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ImageResponsePart {
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: ResponseInlineData,
    },
    Text {
        text: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseInlineData {
    #[allow(dead_code)]
    mime_type: String,
    data: String,
}

// -- Tool response --

#[derive(Debug, Serialize)]
struct GenerateImageResponse {
    file_path: String,
    filename: String,
    model: String,
    quality: String,
    aspect_ratio: String,
    image_size: String,
    delivered_to: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

// -- ImageTools --

#[derive(Clone)]
pub struct ImageTools {
    client: reqwest::Client,
    api_key: String,
    output_dir: PathBuf,
    media_store: Arc<MediaStore>,
    message_store: Arc<MessageStore>,
    telegram_sender: Option<Arc<TelegramSender>>,
    chat_event_hub: Option<Arc<ChatEventHub>>,
    tool_router: ToolRouter<Self>,
}

impl ImageTools {
    pub fn new(
        api_key: impl Into<String>,
        output_dir: PathBuf,
        media_store: Arc<MediaStore>,
        message_store: Arc<MessageStore>,
    ) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create output dir {:?}: {}", output_dir, e))?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            output_dir,
            media_store,
            message_store,
            telegram_sender: None,
            chat_event_hub: None,
            tool_router: Self::create_tool_router(),
        })
    }

    pub fn with_telegram(mut self, sender: Arc<TelegramSender>) -> Self {
        self.telegram_sender = Some(sender);
        self
    }

    pub fn with_chat_event_hub(mut self, hub: Arc<ChatEventHub>) -> Self {
        self.chat_event_hub = Some(hub);
        self
    }

    pub fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    fn create_tool_router() -> ToolRouter<Self> {
        Self::tool_router()
    }
}

#[tool_router]
impl ImageTools {
    #[tool(
        name = "generate_image",
        description = "Generate an image using Gemini's Nano Banana image model, or edit an existing image. Provide a text prompt for generation, or a prompt plus an input image path for editing. The generated image is saved to disk and automatically delivered to the user via Telegram and/or web chat. Requires at least one of chat_id or conversation_id for delivery."
    )]
    async fn generate_image(
        &self,
        request: Parameters<GenerateImageRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = request.0;

        // Validate delivery target
        if req.chat_id.is_none() && req.conversation_id.is_none() {
            return Err(McpError::invalid_params(
                "Either chat_id (Telegram) or conversation_id (web) is required for delivery",
                None,
            ));
        }

        // Validate quality tier and resolve model
        let quality = req.quality.as_deref().unwrap_or("premium");
        if !VALID_QUALITIES.contains(&quality) {
            return Err(McpError::invalid_params(
                format!(
                    "Invalid quality '{}'. Valid: {:?}",
                    quality, VALID_QUALITIES
                ),
                None,
            ));
        }
        let model = match quality {
            "fast" => MODEL_FAST,
            _ => MODEL_PREMIUM,
        };

        let image_size = req.image_size.as_deref().unwrap_or(DEFAULT_IMAGE_SIZE);
        if !VALID_IMAGE_SIZES.contains(&image_size) {
            return Err(McpError::invalid_params(
                format!(
                    "Invalid image_size '{}'. Valid: {:?}",
                    image_size, VALID_IMAGE_SIZES
                ),
                None,
            ));
        }

        let aspect_ratio = req.aspect_ratio.as_deref().unwrap_or(DEFAULT_ASPECT_RATIO);
        if !VALID_ASPECT_RATIOS.contains(&aspect_ratio) {
            return Err(McpError::invalid_params(
                format!(
                    "Invalid aspect_ratio '{}'. Valid: {:?}",
                    aspect_ratio, VALID_ASPECT_RATIOS
                ),
                None,
            ));
        }

        // Build request parts
        let mut parts = vec![RequestPart::Text {
            text: req.prompt.clone(),
        }];

        // Collect all input image paths (singular + plural fields merged)
        let mut all_image_paths: Vec<String> = Vec::new();
        if let Some(ref path) = req.input_image_path {
            all_image_paths.push(path.clone());
        }
        if let Some(ref paths) = req.input_image_paths {
            all_image_paths.extend(paths.iter().cloned());
        }

        if all_image_paths.len() > 14 {
            return Err(McpError::invalid_params(
                format!(
                    "Too many input images ({}). Gemini supports up to 14.",
                    all_image_paths.len()
                ),
                None,
            ));
        }

        // Read and base64-encode each input image
        for input_path in &all_image_paths {
            let image_data = tokio::fs::read(input_path).await.map_err(|e| {
                McpError::invalid_params(
                    format!("Cannot read input image '{}': {}", input_path, e),
                    None,
                )
            })?;

            let ext = std::path::Path::new(input_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let mime_type = mime_for_extension(ext).to_string();

            parts.push(RequestPart::InlineData {
                inline_data: InlineData {
                    mime_type,
                    data: BASE64.encode(&image_data),
                },
            });
        }

        // Build Gemini request
        let gemini_request = GeminiImageRequest {
            contents: vec![RequestContent { parts }],
            generation_config: ImageGenerationConfig {
                response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
                image_config: Some(ImageConfig {
                    aspect_ratio: Some(aspect_ratio.to_string()),
                    image_size: Some(image_size.to_string()),
                }),
            },
        };

        // Call Gemini API with retry
        let (image_bytes, description) = self
            .call_gemini_image(model, &gemini_request)
            .await
            .map_err(|e| McpError::internal_error(e, None))?;

        // Generate filename with human-readable timestamp
        let now = chrono::Local::now();
        let filename = format!("{}.png", now.format("%Y-%m-%d_%H-%M-%S"));
        let file_path = self.output_dir.join(&filename);

        // Save to disk
        tokio::fs::write(&file_path, &image_bytes)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to write image: {}", e), None)
            })?;

        let file_path_str = file_path.to_string_lossy().to_string();
        tracing::info!(
            path = %file_path_str,
            model = model,
            quality = quality,
            size_bytes = image_bytes.len(),
            "Generated image saved"
        );

        // Deliver to user
        let mut delivered_to = Vec::new();

        // Telegram delivery (as photo for inline preview)
        if let Some(chat_id) = req.chat_id {
            if let Some(ref sender) = self.telegram_sender {
                let caption = truncate_caption(&req.prompt, 1024);

                match sender
                    .send_photo(
                        chat_id,
                        req.message_thread_id,
                        image_bytes.clone(),
                        filename.clone(),
                        Some(&caption),
                    )
                    .await
                {
                    Ok(msg_id) => {
                        let stored = StoredMessage::outbound(
                            format!("img-tg-{}", msg_id),
                            chat_id.to_string(),
                            &caption,
                        )
                        .with_media("photo", format!("{}/{}", chat_id, filename))
                        .with_telegram_id(msg_id)
                        .with_topic_id(req.message_thread_id);

                        if let Err(e) = self.message_store.store(stored) {
                            tracing::warn!("Failed to store outbound image message: {}", e);
                        }
                        delivered_to.push(format!("telegram:{}", chat_id));
                    }
                    Err(e) => {
                        tracing::warn!("Telegram send_photo failed: {}", e);
                    }
                }
            }
        }

        // Web delivery (store in media + SSE notification)
        if let Some(ref conversation_id) = req.conversation_id {
            let stored_filename = format!(
                "{}-{}",
                chrono::Utc::now().timestamp_millis(),
                filename
            );
            if let Err(e) = self
                .media_store
                .store(conversation_id, &stored_filename, &image_bytes)
            {
                tracing::warn!("Failed to store image in media_store: {}", e);
            } else {
                let media_path = format!("{}/{}", conversation_id, stored_filename);
                let message_id = format!("img-web-{}", uuid::Uuid::new_v4());

                let stored = StoredMessage::outbound(
                    &message_id,
                    conversation_id,
                    &req.prompt,
                )
                .with_media("photo", &media_path);

                if let Err(e) = self.message_store.store(stored) {
                    tracing::warn!("Failed to store web image message: {}", e);
                }

                if let Some(ref hub) = self.chat_event_hub {
                    hub.send(
                        conversation_id,
                        ChatEvent::FileMessage {
                            conversation_id: conversation_id.clone(),
                            message_id,
                            filename: filename.clone(),
                            media_path,
                            mime_type: "image/png".to_string(),
                            caption: req.prompt.clone(),
                        },
                    )
                    .await;
                }

                delivered_to.push(format!("web:{}", conversation_id));
            }
        }

        let response = GenerateImageResponse {
            file_path: file_path_str,
            filename,
            model: model.to_string(),
            quality: quality.to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            image_size: image_size.to_string(),
            delivered_to,
            description,
        };

        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

// -- Private helpers --

impl ImageTools {
    async fn call_gemini_image(
        &self,
        model: &str,
        request: &GeminiImageRequest,
    ) -> Result<(Vec<u8>, Option<String>), String> {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match self.call_gemini_image_once(model, request).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!("Gemini image API attempt {} failed: {}", attempt + 1, e);

                    let base_delay =
                        extract_retry_delay(&e).unwrap_or(DEFAULT_RETRY_DELAY_SECS);
                    let retry_delay = base_delay + RETRY_BUFFER_SECS;

                    if attempt < MAX_RETRIES - 1 {
                        tracing::info!(
                            "Retrying image generation in {}s ({}s base + {}s buffer)",
                            retry_delay,
                            base_delay,
                            RETRY_BUFFER_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(retry_delay)).await;
                    }

                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Max retries exceeded".to_string()))
    }

    /// Single Gemini image generation API call.
    /// Returns (image_bytes, optional_description_text).
    async fn call_gemini_image_once(
        &self,
        model: &str,
        request: &GeminiImageRequest,
    ) -> Result<(Vec<u8>, Option<String>), String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if !status.is_success() {
            return Err(format!("Gemini API error {}: {}", status, body));
        }

        let gemini_response: GeminiImageResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {} - body: {}", e, body))?;

        let parts = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .ok_or_else(|| "No content in Gemini response".to_string())?;

        let mut image_bytes: Option<Vec<u8>> = None;
        let mut description: Option<String> = None;

        for part in parts {
            match part {
                ImageResponsePart::InlineData { inline_data } => {
                    if image_bytes.is_none() {
                        image_bytes = Some(
                            BASE64
                                .decode(&inline_data.data)
                                .map_err(|e| format!("Failed to decode image data: {}", e))?,
                        );
                    }
                }
                ImageResponsePart::Text { text } => {
                    if description.is_none() && !text.is_empty() {
                        description = Some(text);
                    }
                }
            }
        }

        let bytes = image_bytes.ok_or_else(|| "No image data in Gemini response".to_string())?;
        Ok((bytes, description))
    }
}

/// Truncate a prompt for use as a Telegram caption (max 1024 chars).
fn truncate_caption(prompt: &str, max_len: usize) -> String {
    if prompt.len() <= max_len {
        prompt.to_string()
    } else {
        format!("{}...", &prompt[..max_len - 3])
    }
}

/// Extract retry delay from Gemini error messages.
fn extract_retry_delay(error_msg: &str) -> Option<u64> {
    // Pattern 1: "retryDelay": "59s" in JSON
    if let Some(pos) = error_msg.find("retryDelay") {
        let after = &error_msg[pos..];
        let digits: String = after
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(secs) = digits.parse::<u64>() {
            return Some(secs);
        }
    }
    // Pattern 2: "Please retry in 59.965s."
    if let Some(pos) = error_msg.find("retry in ") {
        let after = &error_msg[pos + 9..];
        let num_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(secs) = num_str.parse::<f64>() {
            return Some(secs.ceil() as u64);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_tools(dir: &TempDir) -> ImageTools {
        let media_store = Arc::new(MediaStore::new(dir.path().join("media")).unwrap());
        let message_store =
            Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap());
        ImageTools::new(
            "fake-api-key",
            dir.path().join("generated_images"),
            media_store,
            message_store,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_rejects_missing_delivery_target() {
        let dir = TempDir::new().unwrap();
        let tools = create_test_tools(&dir);
        let request = Parameters(GenerateImageRequest {
            prompt: "a cat".to_string(),
            input_image_path: None,
            input_image_paths: None,
            quality: None,
            image_size: None,
            aspect_ratio: None,
            chat_id: None,
            message_thread_id: None,
            conversation_id: None,
        });
        let result = tools.generate_image(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rejects_invalid_quality() {
        let dir = TempDir::new().unwrap();
        let tools = create_test_tools(&dir);
        let request = Parameters(GenerateImageRequest {
            prompt: "a cat".to_string(),
            input_image_path: None,
            input_image_paths: None,
            quality: Some("ultra".to_string()),
            image_size: None,
            aspect_ratio: None,
            chat_id: Some(12345),
            message_thread_id: None,
            conversation_id: None,
        });
        let result = tools.generate_image(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rejects_invalid_size() {
        let dir = TempDir::new().unwrap();
        let tools = create_test_tools(&dir);
        let request = Parameters(GenerateImageRequest {
            prompt: "a cat".to_string(),
            input_image_path: None,
            input_image_paths: None,
            quality: None,
            image_size: Some("8K".to_string()),
            aspect_ratio: None,
            chat_id: Some(12345),
            message_thread_id: None,
            conversation_id: None,
        });
        let result = tools.generate_image(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rejects_invalid_aspect_ratio() {
        let dir = TempDir::new().unwrap();
        let tools = create_test_tools(&dir);
        let request = Parameters(GenerateImageRequest {
            prompt: "a cat".to_string(),
            input_image_path: None,
            input_image_paths: None,
            quality: None,
            image_size: None,
            aspect_ratio: Some("7:3".to_string()),
            chat_id: Some(12345),
            message_thread_id: None,
            conversation_id: None,
        });
        let result = tools.generate_image(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rejects_nonexistent_input_image() {
        let dir = TempDir::new().unwrap();
        let tools = create_test_tools(&dir);
        let request = Parameters(GenerateImageRequest {
            prompt: "make this blue".to_string(),
            input_image_path: Some("/tmp/nonexistent_image_xyz.png".to_string()),
            input_image_paths: None,
            quality: None,
            image_size: None,
            aspect_ratio: None,
            chat_id: Some(12345),
            message_thread_id: None,
            conversation_id: None,
        });
        let result = tools.generate_image(request).await;
        // Should fail when trying to read the nonexistent file
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_retry_delay_json() {
        assert_eq!(
            extract_retry_delay(r#"{"retryDelay": "59s"}"#),
            Some(59)
        );
    }

    #[test]
    fn test_extract_retry_delay_message() {
        assert_eq!(extract_retry_delay("retry in 30.5s"), Some(31));
    }

    #[test]
    fn test_extract_retry_delay_none() {
        assert_eq!(extract_retry_delay("some other error"), None);
    }

    #[test]
    fn test_truncate_caption() {
        assert_eq!(truncate_caption("short", 1024), "short");
        let long = "x".repeat(2000);
        let truncated = truncate_caption(&long, 100);
        assert_eq!(truncated.len(), 100);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_quality_to_model_mapping() {
        // Verify the mapping constants are correct
        assert_eq!(MODEL_PREMIUM, "gemini-3-pro-image-preview");
        assert_eq!(MODEL_FAST, "gemini-2.5-flash-image");
    }

    #[test]
    fn test_request_serialization() {
        let request = GeminiImageRequest {
            contents: vec![RequestContent {
                parts: vec![RequestPart::Text {
                    text: "a cat".to_string(),
                }],
            }],
            generation_config: ImageGenerationConfig {
                response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
                image_config: Some(ImageConfig {
                    aspect_ratio: Some("16:9".to_string()),
                    image_size: Some("2K".to_string()),
                }),
            },
        };
        let json = serde_json::to_value(&request).unwrap();
        // Verify camelCase serialization
        assert!(json.get("generationConfig").is_some());
        assert_eq!(
            json["generationConfig"]["responseModalities"],
            serde_json::json!(["TEXT", "IMAGE"])
        );
        assert_eq!(
            json["generationConfig"]["imageConfig"]["aspectRatio"],
            "16:9"
        );
        assert_eq!(
            json["generationConfig"]["imageConfig"]["imageSize"],
            "2K"
        );
    }

    #[test]
    fn test_response_parsing() {
        let response_json = r#"{
            "candidates": [{
                "content": {
                    "parts": [
                        {"text": "Here is your image"},
                        {"inlineData": {"mimeType": "image/png", "data": "aGVsbG8="}}
                    ]
                }
            }]
        }"#;
        let response: GeminiImageResponse = serde_json::from_str(response_json).unwrap();
        let candidates = response.candidates.unwrap();
        let parts = candidates[0]
            .content
            .as_ref()
            .unwrap()
            .parts
            .as_ref()
            .unwrap();
        assert_eq!(parts.len(), 2);
        match &parts[0] {
            ImageResponsePart::Text { text } => assert_eq!(text, "Here is your image"),
            _ => panic!("Expected text part"),
        }
        match &parts[1] {
            ImageResponsePart::InlineData { inline_data } => {
                assert_eq!(inline_data.mime_type, "image/png");
                assert_eq!(BASE64.decode(&inline_data.data).unwrap(), b"hello");
            }
            _ => panic!("Expected inline_data part"),
        }
    }

    #[test]
    fn test_response_parsing_image_only() {
        let response_json = r#"{
            "candidates": [{
                "content": {
                    "parts": [
                        {"inlineData": {"mimeType": "image/png", "data": "dGVzdA=="}}
                    ]
                }
            }]
        }"#;
        let response: GeminiImageResponse = serde_json::from_str(response_json).unwrap();
        let candidates = response.candidates.unwrap();
        let parts = candidates[0]
            .content
            .as_ref()
            .unwrap()
            .parts
            .as_ref()
            .unwrap();
        assert_eq!(parts.len(), 1);
        match &parts[0] {
            ImageResponsePart::InlineData { inline_data } => {
                assert_eq!(BASE64.decode(&inline_data.data).unwrap(), b"test");
            }
            _ => panic!("Expected inline_data part"),
        }
    }

    #[test]
    fn test_output_dir_created() {
        let dir = TempDir::new().unwrap();
        let output_dir = dir.path().join("generated_images");
        assert!(!output_dir.exists());
        let _tools = create_test_tools(&dir);
        assert!(output_dir.exists());
    }
}
