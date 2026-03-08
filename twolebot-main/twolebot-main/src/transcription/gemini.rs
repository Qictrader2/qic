use crate::error::{Result, TwolebotError};
use crate::storage::media::mime_for_telegram_media;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GEMINI_MODEL: &str = "gemini-3-flash-preview";
const MAX_RETRIES: u32 = 5;
const DEFAULT_RETRY_DELAY_SECS: u64 = 5;
const RETRY_BUFFER_SECS: u64 = 3; // Extra buffer added to retry delays

const VIDEO_PROMPT: &str = "\
You are a seeing LLM describing video to a blind LLM that cannot process video. \
Your job is to be the eyes. Be extremely descriptive — include subtle details, \
body language, facial expressions, background elements, lighting, text on screen, \
and anything a sighted person would notice.\n\n\
Provide TWO timestamped tracks:\n\n\
**[AUDIO]** — Timestamped transcription of everything said or heard. \
Include speaker identification where possible, tone of voice, and non-speech sounds \
(footsteps, wind, music, laughter, etc.).\n\
Format: [MM:SS] Speaker/Sound: \"content\"\n\n\
**[VISUAL]** — Timestamped description of what is seen at each moment. \
Describe scene changes, camera movement, people's actions, objects, text overlays, \
environment details, and anything visually notable — even small or subtle things.\n\
Format: [MM:SS] Description\n\n\
Be thorough. The receiving LLM has ZERO visual context and depends entirely on \
your description to understand what happened in this video.";

const VIDEO_NOTE_PROMPT: &str = "\
You are a seeing LLM describing a video note to a blind LLM that cannot process video. \
Your job is to be the eyes. Be extremely descriptive.\n\n\
Provide TWO timestamped tracks:\n\n\
**[AUDIO]** — Timestamped transcription of speech and sounds.\n\
Format: [MM:SS] Speaker/Sound: \"content\"\n\n\
**[VISUAL]** — Timestamped description of what is shown (face, expressions, \
gestures, background, lighting, anything visible).\n\
Format: [MM:SS] Description\n\n\
Be thorough — the receiving LLM depends entirely on your description.";

/// Gemini API transcriber for voice/video messages
pub struct GeminiTranscriber {
    client: reqwest::Client,
    api_key: String,
}

impl GeminiTranscriber {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| TwolebotError::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key: api_key.into(),
        })
    }

    /// Transcribe media data
    pub async fn transcribe(&self, media_data: &[u8], media_type: &str) -> Result<String> {
        let mime_type = mime_for_telegram_media(media_type);
        let base64_data = BASE64.encode(media_data);

        let prompt = match media_type {
            "voice" | "audio" => {
                "Please transcribe this voice message accurately. Include any important context or emotion conveyed."
            }
            "video" => {
                VIDEO_PROMPT
            }
            "video_note" => {
                VIDEO_NOTE_PROMPT
            }
            "photo" => {
                "Please describe this image in detail, including any text visible in it."
            }
            "animation" => {
                "Please describe what's happening in this animated image/video. Include any text visible, describe the action or movement, and note any important visual elements."
            }
            "document" => {
                "Please extract and summarize the text content from this document."
            }
            _ => {
                "Please transcribe or describe this media file."
            }
        };

        self.call_gemini_with_retry(&base64_data, mime_type, prompt)
            .await
    }

    /// Generate text from a text-only prompt (no media).
    /// Useful for formatting, summarizing, or transforming text via Gemini.
    pub async fn generate_text(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            GEMINI_MODEL, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part::Text {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.2,
                max_output_tokens: 4096,
            }),
        };

        let response = self.client.post(&url).json(&request).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::gemini(format!(
                "Gemini API error {}: {}",
                status, body
            )));
        }

        let gemini_response: GeminiResponse = serde_json::from_str(&body).map_err(|e| {
            TwolebotError::gemini(format!("Failed to parse response: {} - {}", e, body))
        })?;

        gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .map(|p| match p {
                ResponsePart::Text { text } => text,
            })
            .ok_or_else(|| TwolebotError::gemini("No text in response"))
    }

    /// Transcribe media data with explicit MIME type
    /// Use this when you have the actual MIME type from Telegram
    pub async fn transcribe_with_mime(
        &self,
        media_data: &[u8],
        media_type: &str,
        mime_type: &str,
    ) -> Result<String> {
        let base64_data = BASE64.encode(media_data);

        let prompt = match media_type {
            "voice" | "audio" => {
                "Please transcribe this voice message accurately. Include any important context or emotion conveyed."
            }
            "video" => {
                VIDEO_PROMPT
            }
            "video_note" => {
                VIDEO_NOTE_PROMPT
            }
            "photo" => {
                "Please describe this image in detail, including any text visible in it."
            }
            "animation" => {
                "Please describe what's happening in this animated image/video (GIF). Include any text visible, describe the action or movement, and note any important visual elements. If it's a meme or reaction GIF, identify the reference if possible."
            }
            "sticker" => {
                "Please describe this sticker, including any text, emoji, or character shown. Note the emotion or message it conveys."
            }
            "document" => {
                "Please extract and summarize the text content from this document."
            }
            _ => {
                "Please transcribe or describe this media file."
            }
        };

        self.call_gemini_with_retry(&base64_data, mime_type, prompt)
            .await
    }



    /// Call Gemini API with retry logic
    async fn call_gemini_with_retry(
        &self,
        base64_data: &str,
        mime_type: &str,
        prompt: &str,
    ) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match self.call_gemini(base64_data, mime_type, prompt).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_msg = e.to_string();
                    tracing::warn!("Gemini API attempt {} failed: {}", attempt + 1, error_msg);

                    // Check for rate limit with retry delay
                    // Add buffer to the delay to avoid hitting the limit again immediately
                    let base_delay =
                        extract_retry_delay(&error_msg).unwrap_or(DEFAULT_RETRY_DELAY_SECS);
                    let retry_delay = base_delay + RETRY_BUFFER_SECS;

                    if attempt < MAX_RETRIES - 1 {
                        tracing::info!(
                            "Gemini rate limited. Retrying in {} seconds ({}s delay + {}s buffer)...",
                            retry_delay, base_delay, RETRY_BUFFER_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(retry_delay)).await;
                    }

                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| TwolebotError::gemini("Max retries exceeded")))
    }

    /// Single Gemini API call
    async fn call_gemini(
        &self,
        base64_data: &str,
        mime_type: &str,
        prompt: &str,
    ) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            GEMINI_MODEL, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: base64_data.to_string(),
                        },
                    },
                    Part::Text {
                        text: prompt.to_string(),
                    },
                ],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.1,
                max_output_tokens: 8192,
            }),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TwolebotError::gemini(format!(
                "Gemini API error {}: {}",
                status, body
            )));
        }

        let gemini_response: GeminiResponse = serde_json::from_str(&body).map_err(|e| {
            TwolebotError::gemini(format!("Failed to parse response: {} - {}", e, body))
        })?;

        // Extract text from response
        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .map(|p| match p {
                ResponsePart::Text { text } => text,
            })
            .ok_or_else(|| TwolebotError::gemini("No text in response"))?;

        Ok(text)
    }
}

/// Extract retry delay from error message (Gemini returns retryDelay in error)
/// Handles multiple patterns:
/// - `"retryDelay": "59s"` in JSON
/// - `Please retry in 59.965134946s.` in message text
fn extract_retry_delay(error_msg: &str) -> Option<u64> {
    // Try pattern 1: "retryDelay": "59s"
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

    // Try pattern 2: "Please retry in 59.965134946s"
    if let Some(pos) = error_msg.find("retry in ") {
        let after = &error_msg[pos + 9..]; // Skip "retry in "
                                           // Extract the number (may have decimal)
        let num_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        // Parse as float and round up to be safe
        if let Ok(secs) = num_str.parse::<f64>() {
            return Some(secs.ceil() as u64);
        }
    }

    // Try pattern 3: Just look for any number followed by 's' near "retry" or "wait"
    for keyword in ["retry", "wait", "delay"] {
        if let Some(pos) = error_msg.to_lowercase().find(keyword) {
            let after = &error_msg[pos..];
            // Find first number
            let mut chars = after.chars().peekable();
            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    let mut num_str = String::from(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() || next == '.' {
                            // Safe: we just confirmed peek() returned Some
                            if let Some(ch) = chars.next() {
                                num_str.push(ch);
                            }
                        } else {
                            break;
                        }
                    }
                    // Check if followed by 's' (seconds)
                    if chars.peek() == Some(&'s') || chars.peek() == Some(&'S') {
                        if let Ok(secs) = num_str.parse::<f64>() {
                            return Some(secs.ceil() as u64);
                        }
                    }
                }
            }
        }
    }

    None
}

// Gemini API request/response types

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<ResponseContent>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponsePart {
    Text { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_retry_delay() {
        // Pattern 1: JSON retryDelay field
        assert_eq!(extract_retry_delay(r#"{"retryDelay": "5s"}"#), Some(5));
        assert_eq!(extract_retry_delay(r#"{"retryDelay": "59s"}"#), Some(59));
        assert_eq!(extract_retry_delay(r#"retryDelay: 10"#), Some(10));

        // Pattern 2: "Please retry in Xs" message
        assert_eq!(
            extract_retry_delay("Please retry in 59.965134946s."),
            Some(60)
        ); // Rounds up
        assert_eq!(extract_retry_delay("Please retry in 30s"), Some(30));

        // Pattern 3: Generic number + s near retry/wait keywords
        assert_eq!(extract_retry_delay("wait 15s before retrying"), Some(15));

        // No match
        assert_eq!(extract_retry_delay("no delay here"), None);
        assert_eq!(extract_retry_delay("error 429"), None);

        // Real Gemini error message
        let real_error = r#"Gemini API error 429 Too Many Requests: { "error": { "code": 429, "message": "Please retry in 59.965134946s.", "details": [ { "@type": "type.googleapis.com/google.rpc.RetryInfo", "retryDelay": "59s" } ] } }"#;
        assert_eq!(extract_retry_delay(real_error), Some(59));
    }

    #[test]
    fn test_mime_type_mapping() {
        assert_eq!(mime_for_telegram_media("voice"), "audio/ogg");
        assert_eq!(mime_for_telegram_media("video"), "video/mp4");
        assert_eq!(mime_for_telegram_media("photo"), "image/jpeg");
    }

    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            /// Property: JSON retryDelay pattern always extracts correctly
            #[test]
            fn prop_json_retry_delay_extraction(secs in 1u64..3600) {
                let json = format!(r#"{{"retryDelay": "{}s"}}"#, secs);
                assert_eq!(extract_retry_delay(&json), Some(secs));
            }

            /// Property: "retry in Xs" pattern extracts correctly (integer seconds)
            #[test]
            fn prop_retry_in_pattern_integer(secs in 1u64..3600) {
                let msg = format!("Please retry in {}s.", secs);
                assert_eq!(extract_retry_delay(&msg), Some(secs));
            }

            /// Property: "retry in X.Ys" pattern rounds up correctly
            #[test]
            fn prop_retry_in_pattern_decimal(
                secs in 1u64..3600,
                frac in 0.001f64..0.999
            ) {
                let total = secs as f64 + frac;
                let msg = format!("Please retry in {}s.", total);
                let result = extract_retry_delay(&msg);
                // Should round up
                assert_eq!(result, Some((total.ceil()) as u64));
            }

            /// Property: Extracted delay is always positive when present
            #[test]
            fn prop_extracted_delay_positive(secs in 1u64..3600) {
                let patterns = vec![
                    format!(r#"{{"retryDelay": "{}s"}}"#, secs),
                    format!("Please retry in {}s.", secs),
                    format!("wait {}s before retrying", secs),
                ];

                for pattern in patterns {
                    if let Some(delay) = extract_retry_delay(&pattern) {
                        assert!(delay > 0, "Delay should be positive for: {}", pattern);
                    }
                }
            }

            /// Property: No false positives on random text without retry patterns
            #[test]
            fn prop_no_false_positives(
                text in "[a-zA-Z ]{10,50}"
            ) {
                // Random alphabetic text shouldn't match
                if !text.to_lowercase().contains("retry")
                    && !text.to_lowercase().contains("wait")
                    && !text.to_lowercase().contains("delay") {
                    assert_eq!(extract_retry_delay(&text), None);
                }
            }
        }
    }
}
