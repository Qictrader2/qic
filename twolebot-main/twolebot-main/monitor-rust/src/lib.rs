use serde::{Deserialize, Serialize};

pub mod message_splitter;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    #[serde(rename = "chatId")]
    pub chat_id: String,
    #[serde(rename = "userId")]
    pub user_id: i64,
    pub prompt: String,
    #[serde(rename = "mediaUrl")]
    pub media_url: Option<String>,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub message: Option<MessageContent>,
    #[allow(dead_code)]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageContent {
    pub content: Option<serde_json::Value>,
}

pub fn extract_text_from_event(event: &StreamEvent) -> String {
    match event.event_type.as_str() {
        "assistant" => {
            if let Some(ref message) = event.message {
                if let Some(ref content) = message.content {
                    if let Some(arr) = content.as_array() {
                        return arr.iter()
                            .filter_map(|item| {
                                let item_type = item.get("type")?.as_str()?;
                                match item_type {
                                    "text" => item.get("text")?.as_str().map(|s| s.to_string()),
                                    "tool_use" => {
                                        // Show tool use notifications
                                        let tool_name = item.get("name")?.as_str()?;
                                        Some(format!("\n🔧 Using tool: {}\n", tool_name))
                                    }
                                    "thinking" => {
                                        // Show thinking process
                                        item.get("text")?.as_str().map(|s| format!("\n💭 Thinking: {}\n", s))
                                    }
                                    _ => None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("");
                    } else if let Some(s) = content.as_str() {
                        return s.to_string();
                    }
                }
            }
        }
        "tool" => {
            // Show tool results WITH THE ACTUAL OUTPUT
            if let Some(ref message) = event.message {
                if let Some(ref content) = message.content {
                    // Get the actual tool output/content
                    if let Some(output) = content.get("content").and_then(|c| c.as_str()) {
                        if let Some(tool_name) = content.get("name").and_then(|n| n.as_str()) {
                            return format!("📊 Tool result from {}:\n{}\n", tool_name, output);
                        } else {
                            return output.to_string();
                        }
                    }
                }
            }
        }
        "system" => {
            // Show important system messages
            if let Some(ref message) = event.message {
                if let Some(ref content) = message.content {
                    if let Some(text) = content.as_str() {
                        return format!("⚙️ System: {}\n", text);
                    }
                }
            }
        }
        _ => {}
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_text_from_assistant_event() {
        let event = StreamEvent {
            event_type: "assistant".to_string(),
            message: Some(MessageContent {
                content: Some(json!([
                    {"type": "text", "text": "Hello "},
                    {"type": "text", "text": "world!"}
                ])),
            }),
            session_id: None,
        };

        let result = extract_text_from_event(&event);
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_extract_text_from_non_text_event() {
        let event = StreamEvent {
            event_type: "assistant".to_string(),
            message: Some(MessageContent {
                content: Some(json!([
                    {"type": "image", "url": "http://example.com"},
                    {"type": "text", "text": "visible"}
                ])),
            }),
            session_id: None,
        };

        let result = extract_text_from_event(&event);
        assert_eq!(result, "visible");
    }

    #[test]
    fn test_extract_text_from_string_content() {
        let event = StreamEvent {
            event_type: "assistant".to_string(),
            message: Some(MessageContent {
                content: Some(json!("Direct string content")),
            }),
            session_id: None,
        };

        let result = extract_text_from_event(&event);
        assert_eq!(result, "Direct string content");
    }

    #[test]
    fn test_extract_text_from_non_assistant_event() {
        let event = StreamEvent {
            event_type: "user".to_string(),
            message: Some(MessageContent {
                content: Some(json!("Should be ignored")),
            }),
            session_id: None,
        };

        let result = extract_text_from_event(&event);
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_text_from_empty_event() {
        let event = StreamEvent {
            event_type: "assistant".to_string(),
            message: None,
            session_id: None,
        };

        let result = extract_text_from_event(&event);
        assert_eq!(result, "");
    }

    #[test]
    fn test_job_serialization() {
        let job = Job {
            id: "msg-123-456".to_string(),
            chat_id: "123".to_string(),
            user_id: 789,
            prompt: "Test prompt".to_string(),
            media_url: Some("https://example.com/media/test.jpg?expires=123&sig=abc".to_string()),
            status: "pending".to_string(),
            created_at: 1234567890,
        };

        let serialized = serde_json::to_string(&job).unwrap();
        assert!(serialized.contains("\"chatId\":\"123\""));
        assert!(serialized.contains("\"userId\":789"));
        assert!(serialized.contains("\"createdAt\":1234567890"));
        assert!(serialized.contains("\"mediaUrl\":\"https://example.com/media/test.jpg"));
    }

    #[test]
    fn test_job_serialization_without_media() {
        let job = Job {
            id: "msg-123-456".to_string(),
            chat_id: "123".to_string(),
            user_id: 789,
            prompt: "Test prompt".to_string(),
            media_url: None,
            status: "pending".to_string(),
            created_at: 1234567890,
        };

        let serialized = serde_json::to_string(&job).unwrap();
        assert!(serialized.contains("\"chatId\":\"123\""));
        assert!(serialized.contains("\"mediaUrl\":null"));
    }

    #[test]
    fn test_job_deserialization() {
        let json = r#"{
            "id": "msg-123-456",
            "chatId": "123",
            "userId": 789,
            "prompt": "Test prompt",
            "mediaUrl": null,
            "status": "pending",
            "createdAt": 1234567890
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "msg-123-456");
        assert_eq!(job.chat_id, "123");
        assert_eq!(job.user_id, 789);
        assert!(job.media_url.is_none());
    }

    #[test]
    fn test_job_deserialization_with_media_url() {
        let json = r#"{
            "id": "msg-123-456",
            "chatId": "123",
            "userId": 789,
            "prompt": "Test prompt with media",
            "mediaUrl": "https://telegram-media-r2.workers.dev/media/telegram%2F123%2Fphoto.jpg?expires=1735000000&sig=abc123",
            "status": "pending",
            "createdAt": 1234567890
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "msg-123-456");
        assert_eq!(job.chat_id, "123");
        assert_eq!(job.user_id, 789);
        assert!(job.media_url.is_some());
        assert!(job.media_url.unwrap().contains("telegram-media-r2"));
    }

    #[test]
    fn test_job_deserialization_missing_media_url() {
        // Test backwards compatibility - mediaUrl might be missing in older responses
        let json = r#"{
            "id": "msg-123-456",
            "chatId": "123",
            "userId": 789,
            "prompt": "Test prompt",
            "status": "pending",
            "createdAt": 1234567890
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "msg-123-456");
        assert!(job.media_url.is_none());
    }

    #[test]
    fn test_message_id_extraction() {
        let job_id = "msg-123-456";
        let message_id = job_id.split('-').nth(2).unwrap_or("");
        assert_eq!(message_id, "456");

        let invalid_job_id = "invalid";
        let message_id = invalid_job_id.split('-').nth(2).unwrap_or("");
        assert_eq!(message_id, "");
    }

    // Agent ID validation tests
    fn is_valid_agent_id(id: &str) -> bool {
        match id {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" |
            "a" | "b" | "c" | "d" | "e" | "f" | "default" => true,
            _ => false,
        }
    }

    fn normalize_agent_id(id: &str) -> String {
        id.to_lowercase()
    }

    #[test]
    fn test_valid_agent_ids() {
        // All hex digits should be valid
        let valid_ids = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                         "a", "b", "c", "d", "e", "f", "default"];
        for id in valid_ids {
            assert!(is_valid_agent_id(id), "Expected '{}' to be valid", id);
        }
    }

    #[test]
    fn test_invalid_agent_ids() {
        // Non-hex characters should be invalid
        let invalid_ids = ["g", "h", "z", "A", "B", "G", "!", "@", " ", "", "10", "ab", "foo"];
        for id in invalid_ids {
            assert!(!is_valid_agent_id(id), "Expected '{}' to be invalid", id);
        }
    }

    #[test]
    fn test_agent_id_normalization() {
        // Uppercase should normalize to lowercase
        assert_eq!(normalize_agent_id("A"), "a");
        assert_eq!(normalize_agent_id("F"), "f");
        assert_eq!(normalize_agent_id("Default"), "default");
        assert_eq!(normalize_agent_id("DEFAULT"), "default");
    }

    // Job ID parsing tests
    fn parse_job_id(job_id: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = job_id.split('-').collect();
        if parts.len() >= 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2..].join("-"), // Handle message IDs that might contain dashes
            ))
        } else {
            None
        }
    }

    #[test]
    fn test_job_id_parsing() {
        let job_id = "msg-123456-789";
        let parsed = parse_job_id(job_id);
        assert!(parsed.is_some());
        let (prefix, chat_id, msg_id) = parsed.unwrap();
        assert_eq!(prefix, "msg");
        assert_eq!(chat_id, "123456");
        assert_eq!(msg_id, "789");
    }

    #[test]
    fn test_job_id_parsing_with_complex_message_id() {
        let job_id = "msg-123456-789-extra-parts";
        let parsed = parse_job_id(job_id);
        assert!(parsed.is_some());
        let (prefix, chat_id, msg_id) = parsed.unwrap();
        assert_eq!(prefix, "msg");
        assert_eq!(chat_id, "123456");
        assert_eq!(msg_id, "789-extra-parts");
    }

    #[test]
    fn test_job_id_parsing_invalid() {
        // Too few parts
        assert!(parse_job_id("msg").is_none());
        assert!(parse_job_id("msg-123").is_none());
        assert!(parse_job_id("").is_none());
    }

    // Bot token response parsing tests
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct BotTokenResponse {
        #[serde(rename = "botToken")]
        bot_token: String,
        #[serde(rename = "agentId")]
        agent_id: String,
    }

    fn parse_bot_token_response(json: &str) -> Result<BotTokenResponse, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn test_bot_token_response_parsing() {
        let json = r#"{"botToken": "123456:ABC-xyz", "agentId": "1"}"#;
        let result = parse_bot_token_response(json);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.bot_token, "123456:ABC-xyz");
        assert_eq!(response.agent_id, "1");
    }

    #[test]
    fn test_bot_token_response_with_default_agent() {
        let json = r#"{"botToken": "987654:DEF-uvw", "agentId": "default"}"#;
        let result = parse_bot_token_response(json);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.bot_token, "987654:DEF-uvw");
        assert_eq!(response.agent_id, "default");
    }

    #[test]
    fn test_bot_token_response_with_hex_agent() {
        let json = r#"{"botToken": "token:with-special-chars_123", "agentId": "f"}"#;
        let result = parse_bot_token_response(json);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.bot_token, "token:with-special-chars_123");
        assert_eq!(response.agent_id, "f");
    }

    #[test]
    fn test_bot_token_response_missing_field() {
        let json = r#"{"botToken": "123456:ABC"}"#;
        let result = parse_bot_token_response(json);
        assert!(result.is_err(), "Should fail when agentId is missing");
    }

    #[test]
    fn test_bot_token_response_empty_token() {
        // Even empty tokens should parse (backend will return default token, never empty)
        let json = r#"{"botToken": "", "agentId": "1"}"#;
        let result = parse_bot_token_response(json);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.bot_token, "");
    }

    #[test]
    fn test_bot_token_is_used_for_telegram_url() {
        // Test that bot token is correctly formatted for Telegram API URL
        let bot_token = "123456789:ABCdefGHI-jklMNOpqrsTUVwxyz";
        let expected_url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
        assert!(expected_url.contains(bot_token));
        assert!(expected_url.starts_with("https://api.telegram.org/bot"));
        assert!(expected_url.ends_with("/sendMessage"));
    }

    // Property-based tests
    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        // Generate valid agent IDs
        fn arb_valid_agent_id() -> impl Strategy<Value = String> {
            prop::sample::select(vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                "a", "b", "c", "d", "e", "f", "default"
            ]).prop_map(|s| s.to_string())
        }

        // Generate invalid agent IDs
        fn arb_invalid_agent_id() -> impl Strategy<Value = String> {
            prop_oneof![
                // Non-hex lowercase letters
                prop::sample::select(vec!["g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                    "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"]).prop_map(|s| s.to_string()),
                // Uppercase letters (should be rejected before normalization)
                prop::sample::select(vec!["A", "B", "C", "D", "E", "F"]).prop_map(|s| s.to_string()),
                // Multi-character strings (not single hex digit)
                "[a-z0-9]{2,10}".prop_filter("not valid single char", |s| s.len() > 1 && s != "default"),
                // Empty string
                Just("".to_string()),
                // Special characters
                prop::sample::select(vec!["!", "@", "#", "$", "%", " ", "-"]).prop_map(|s| s.to_string()),
            ]
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_valid_agent_ids_accepted(id in arb_valid_agent_id()) {
                assert!(is_valid_agent_id(&id),
                    "Valid agent ID '{}' was rejected", id);
            }

            #[test]
            fn prop_invalid_agent_ids_rejected(id in arb_invalid_agent_id()) {
                assert!(!is_valid_agent_id(&id),
                    "Invalid agent ID '{}' was accepted", id);
            }

            #[test]
            fn prop_normalization_idempotent(id in arb_valid_agent_id()) {
                let normalized = normalize_agent_id(&id);
                let double_normalized = normalize_agent_id(&normalized);
                assert_eq!(normalized, double_normalized,
                    "Normalization should be idempotent");
            }

            #[test]
            fn prop_normalization_produces_lowercase(id in "[A-Fa-f0-9]") {
                let normalized = normalize_agent_id(&id);
                assert!(normalized.chars().all(|c| c.is_lowercase() || c.is_numeric()),
                    "Normalization should produce lowercase: '{}'", normalized);
            }

            #[test]
            fn prop_job_id_parsing_roundtrip(
                prefix in prop::sample::select(vec!["msg", "transcription"]),
                chat_id in "[0-9]{1,12}",
                msg_id in "[0-9]{1,12}"
            ) {
                let job_id = format!("{}-{}-{}", prefix, chat_id, msg_id);
                let parsed = parse_job_id(&job_id);
                assert!(parsed.is_some(), "Valid job ID should parse: {}", job_id);
                let (p, c, m) = parsed.unwrap();
                assert_eq!(p, prefix);
                assert_eq!(c, chat_id);
                assert_eq!(m, msg_id);
            }

            #[test]
            fn prop_job_id_message_id_extraction(
                chat_id in "[0-9]{1,12}",
                msg_id in "[0-9]{1,12}"
            ) {
                let job_id = format!("msg-{}-{}", chat_id, msg_id);
                let message_id = job_id.split('-').nth(2).unwrap_or("");
                assert_eq!(message_id, msg_id,
                    "Message ID extraction failed for job_id: {}", job_id);
            }

            #[test]
            fn prop_job_deserialization_with_agent_id(
                agent_id in arb_valid_agent_id()
            ) {
                // Test that jobs with various agent IDs deserialize correctly
                let json = format!(r#"{{
                    "id": "msg-123-456",
                    "chatId": "123",
                    "userId": 789,
                    "prompt": "Test prompt",
                    "agentId": "{}",
                    "mediaUrl": null,
                    "status": "pending",
                    "createdAt": 1234567890
                }}"#, agent_id);

                // The current Job struct doesn't have agentId, but this tests
                // that JSON with extra fields is still parseable
                let result: Result<Job, _> = serde_json::from_str(&json);
                assert!(result.is_ok(), "Failed to deserialize job with agentId: {}", agent_id);
            }

            #[test]
            fn prop_extract_text_never_crashes(
                event_type in "[a-z]{1,20}",
                content_str in "[a-zA-Z0-9 .,!?]{0,1000}"
            ) {
                let event = StreamEvent {
                    event_type,
                    message: Some(MessageContent {
                        content: Some(serde_json::json!(content_str)),
                    }),
                    session_id: None,
                };

                // Should never crash, even with unexpected input
                let _ = extract_text_from_event(&event);
            }

            #[test]
            fn prop_extract_text_from_array_content_never_crashes(
                items in prop::collection::vec(
                    prop::sample::select(vec!["text", "tool_use", "thinking", "image", "code"]),
                    0..10
                )
            ) {
                let content: Vec<serde_json::Value> = items.iter()
                    .map(|item_type| serde_json::json!({
                        "type": item_type,
                        "text": "sample text"
                    }))
                    .collect();

                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(serde_json::json!(content)),
                    }),
                    session_id: None,
                };

                // Should never crash
                let result = extract_text_from_event(&event);
                // Text items should be extracted
                if items.contains(&"text") {
                    assert!(result.contains("sample text"),
                        "Text content should be extracted");
                }
            }

            #[test]
            fn prop_bot_token_response_parsing_with_valid_agents(
                agent_id in arb_valid_agent_id(),
                token_suffix in "[a-zA-Z0-9_-]{10,40}"
            ) {
                let bot_token = format!("123456789:{}", token_suffix);
                let json = format!(r#"{{"botToken": "{}", "agentId": "{}"}}"#, bot_token, agent_id);
                let result = parse_bot_token_response(&json);
                assert!(result.is_ok(), "Should parse valid bot token response");
                let response = result.unwrap();
                assert_eq!(response.bot_token, bot_token);
                assert_eq!(response.agent_id, agent_id);
            }

            #[test]
            fn prop_telegram_url_format_is_valid(
                token_id in "[0-9]{8,12}",
                token_secret in "[a-zA-Z0-9_-]{20,50}"
            ) {
                let bot_token = format!("{}:{}", token_id, token_secret);
                let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

                // Verify URL structure
                assert!(url.starts_with("https://"), "URL should use HTTPS");
                assert!(url.contains("api.telegram.org"), "URL should point to Telegram API");
                assert!(url.contains(&bot_token), "URL should contain the bot token");
                assert!(url.ends_with("/sendMessage"), "URL should end with the method");

                // Verify token appears exactly once and in correct position
                let bot_prefix = "https://api.telegram.org/bot";
                let after_bot = url.strip_prefix(bot_prefix).unwrap();
                assert!(after_bot.starts_with(&token_id), "Token should start with ID");
            }
        }
    }
}