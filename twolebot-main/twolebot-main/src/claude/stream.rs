use serde::Deserialize;

/// Stream event from Claude CLI output
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

/// Options for text extraction
#[derive(Clone, Debug, Default)]
pub struct ExtractOptions {
    /// Include "Using tool: X" messages (default: false)
    pub show_tool_messages: bool,
    /// Include "Thinking:" messages (default: false)
    pub show_thinking_messages: bool,
    /// Include tool result messages (default: false)
    pub show_tool_results: bool,
}

impl ExtractOptions {
    pub fn all() -> Self {
        Self {
            show_tool_messages: true,
            show_thinking_messages: true,
            show_tool_results: true,
        }
    }
}

/// Extract text from a Claude stream event (with default options - hides tool/thinking messages)
pub fn extract_text_from_event(event: &StreamEvent) -> String {
    extract_text_from_event_with_options(event, &ExtractOptions::default())
}

/// Extract text from a Claude stream event with configurable options
pub fn extract_text_from_event_with_options(
    event: &StreamEvent,
    options: &ExtractOptions,
) -> String {
    match event.event_type.as_str() {
        "assistant" => extract_assistant_text_with_options(event, options),
        "tool" => {
            if options.show_tool_results {
                extract_tool_text(event)
            } else {
                String::new()
            }
        }
        "system" => extract_system_text(event),
        _ => String::new(),
    }
}

fn extract_assistant_text_with_options(event: &StreamEvent, options: &ExtractOptions) -> String {
    let Some(ref message) = event.message else {
        return String::new();
    };
    let Some(ref content) = message.content else {
        return String::new();
    };

    // Handle array content (most common)
    if let Some(arr) = content.as_array() {
        return arr
            .iter()
            .filter_map(|item| {
                let item_type = item.get("type")?.as_str()?;
                match item_type {
                    "text" => item.get("text")?.as_str().map(|s| s.to_string()),
                    "tool_use" => {
                        if options.show_tool_messages {
                            let tool_name = item.get("name")?.as_str()?;
                            Some(format!("\n🔧 Using tool: {}\n", tool_name))
                        } else {
                            None
                        }
                    }
                    "thinking" => {
                        if options.show_thinking_messages {
                            item.get("text")?
                                .as_str()
                                .map(|s| format!("\n💭 Thinking: {}\n", s))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>()
            .join("");
    }

    // Handle string content
    if let Some(s) = content.as_str() {
        return s.to_string();
    }

    String::new()
}

fn extract_tool_text(event: &StreamEvent) -> String {
    let Some(ref message) = event.message else {
        return String::new();
    };
    let Some(ref content) = message.content else {
        return String::new();
    };

    // Get tool output
    if let Some(output) = content.get("content").and_then(|c| c.as_str()) {
        if let Some(tool_name) = content.get("name").and_then(|n| n.as_str()) {
            return format!("📊 Tool result from {}:\n{}\n", tool_name, output);
        }
        return output.to_string();
    }

    String::new()
}

fn extract_system_text(event: &StreamEvent) -> String {
    let Some(ref message) = event.message else {
        return String::new();
    };
    let Some(ref content) = message.content else {
        return String::new();
    };

    if let Some(text) = content.as_str() {
        return format!("⚙️ System: {}\n", text);
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
    fn test_extract_tool_use() {
        let event = StreamEvent {
            event_type: "assistant".to_string(),
            message: Some(MessageContent {
                content: Some(json!([
                    {"type": "tool_use", "name": "Read", "id": "123"}
                ])),
            }),
            session_id: None,
        };

        // With show_tool_messages enabled
        let result = extract_text_from_event_with_options(&event, &ExtractOptions::all());
        assert!(result.contains("Using tool: Read"));

        // Default behavior should hide tool messages
        let result_default = extract_text_from_event(&event);
        assert!(!result_default.contains("Using tool"));
    }

    #[test]
    fn test_extract_thinking() {
        let event = StreamEvent {
            event_type: "assistant".to_string(),
            message: Some(MessageContent {
                content: Some(json!([
                    {"type": "thinking", "text": "Let me analyze this..."}
                ])),
            }),
            session_id: None,
        };

        // With show_thinking_messages enabled
        let result = extract_text_from_event_with_options(&event, &ExtractOptions::all());
        assert!(result.contains("Thinking:"));
        assert!(result.contains("Let me analyze"));

        // Default behavior should hide thinking messages
        let result_default = extract_text_from_event(&event);
        assert!(!result_default.contains("Thinking"));
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;
        use serde_json::json;

        fn arb_text_content() -> impl Strategy<Value = String> {
            prop::collection::vec(
                prop::sample::select(vec![
                    "Hello", "World", "Here's", "the", "answer", "to", "your", "question", "about",
                    "coding", "in", "Rust", "\n", " ", ".", "!", "?", ",", "```", "let", "fn",
                    "struct", "impl", "pub", "async", "await", "Ok",
                ]),
                1..50,
            )
            .prop_map(|parts| parts.join(""))
        }

        fn arb_tool_name() -> impl Strategy<Value = String> {
            prop::sample::select(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Edit".to_string(),
                "Bash".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
                "Task".to_string(),
            ])
        }

        fn arb_event_type() -> impl Strategy<Value = String> {
            prop::sample::select(vec![
                "assistant".to_string(),
                "tool".to_string(),
                "system".to_string(),
                "user".to_string(),
                "result".to_string(),
            ])
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            #[test]
            fn prop_stream_event_json_roundtrip(
                event_type in arb_event_type(),
                has_session_id in any::<bool>(),
            ) {
                let mut json_value = json!({
                    "type": event_type
                });

                if has_session_id {
                    json_value["session_id"] = json!("session-123");
                }

                let json_str = json_value.to_string();
                let parsed: Result<StreamEvent, _> = serde_json::from_str(&json_str);

                assert!(parsed.is_ok(), "Should parse basic event");
                let event = parsed.unwrap();
                assert_eq!(event.event_type, event_type);
            }

            #[test]
            fn prop_text_extraction_preserves_content(text in arb_text_content()) {
                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(json!([{"type": "text", "text": text}])),
                    }),
                    session_id: None,
                };

                let result = extract_text_from_event(&event);
                assert_eq!(result, text, "Text content should be preserved exactly");
            }

            #[test]
            fn prop_multiple_text_blocks_concatenated(
                texts in prop::collection::vec(arb_text_content(), 1..5)
            ) {
                let content_array: Vec<serde_json::Value> = texts
                    .iter()
                    .map(|t| json!({"type": "text", "text": t}))
                    .collect();

                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(json!(content_array)),
                    }),
                    session_id: None,
                };

                let result = extract_text_from_event(&event);
                let expected = texts.join("");
                assert_eq!(result, expected, "Multiple text blocks should be concatenated");
            }

            #[test]
            fn prop_tool_use_includes_tool_name(tool_name in arb_tool_name()) {
                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(json!([
                            {"type": "tool_use", "name": tool_name, "id": "tool-123"}
                        ])),
                    }),
                    session_id: None,
                };

                // With options enabled
                let result = extract_text_from_event_with_options(&event, &ExtractOptions::all());
                assert!(result.contains(&tool_name), "Tool name should be in output");
                assert!(result.contains("Using tool"), "Should indicate tool usage");

                // Default should hide tool messages
                let result_default = extract_text_from_event(&event);
                assert!(!result_default.contains("Using tool"), "Default should hide tool messages");
            }

            #[test]
            fn prop_thinking_extraction(thinking_text in arb_text_content()) {
                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(json!([
                            {"type": "thinking", "text": thinking_text}
                        ])),
                    }),
                    session_id: None,
                };

                // With options enabled
                let result = extract_text_from_event_with_options(&event, &ExtractOptions::all());
                assert!(result.contains(&thinking_text), "Thinking content should be preserved");
                assert!(result.contains("Thinking"), "Should indicate thinking block");

                // Default should hide thinking messages
                let result_default = extract_text_from_event(&event);
                assert!(!result_default.contains("Thinking"), "Default should hide thinking");
            }

            #[test]
            fn prop_non_assistant_events_return_empty(
                event_type in prop::sample::select(vec!["user", "result", "error", "unknown"]),
            ) {
                let event = StreamEvent {
                    event_type: event_type.to_string(),
                    message: Some(MessageContent {
                        content: Some(json!("Some content that should be ignored")),
                    }),
                    session_id: None,
                };

                let result = extract_text_from_event(&event);
                // Non-assistant, non-tool, non-system events should return empty
                assert!(result.is_empty() || event_type == "system",
                    "Non-matching event types should return empty (unless system)");
            }

            #[test]
            fn prop_mixed_content_types(
                text in arb_text_content(),
                tool_name in arb_tool_name(),
            ) {
                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(json!([
                            {"type": "text", "text": text},
                            {"type": "tool_use", "name": tool_name, "id": "123"},
                            {"type": "image", "url": "http://ignored"},  // Should be ignored
                        ])),
                    }),
                    session_id: None,
                };

                // With all options enabled
                let result = extract_text_from_event_with_options(&event, &ExtractOptions::all());
                assert!(result.contains(&text), "Text content should be included");
                assert!(result.contains(&tool_name), "Tool name should be included");

                // Default should only include text
                let result_default = extract_text_from_event(&event);
                assert!(result_default.contains(&text), "Text content should be included in default");
                assert!(!result_default.contains(&tool_name), "Tool name should be hidden in default");
            }

            #[test]
            fn prop_empty_event_handling(
                event_type in arb_event_type(),
            ) {
                // Event with no message
                let event1 = StreamEvent {
                    event_type: event_type.clone(),
                    message: None,
                    session_id: None,
                };
                let result1 = extract_text_from_event(&event1);
                assert!(result1.is_empty(), "No message should return empty");

                // Event with message but no content
                let event2 = StreamEvent {
                    event_type: event_type.clone(),
                    message: Some(MessageContent { content: None }),
                    session_id: None,
                };
                let result2 = extract_text_from_event(&event2);
                assert!(result2.is_empty(), "No content should return empty");

                // Event with empty array
                let event3 = StreamEvent {
                    event_type,
                    message: Some(MessageContent {
                        content: Some(json!([])),
                    }),
                    session_id: None,
                };
                let result3 = extract_text_from_event(&event3);
                assert!(result3.is_empty(), "Empty array should return empty");
            }

            #[test]
            fn prop_string_content_direct(text in arb_text_content()) {
                let event = StreamEvent {
                    event_type: "assistant".to_string(),
                    message: Some(MessageContent {
                        content: Some(json!(text)),
                    }),
                    session_id: None,
                };

                let result = extract_text_from_event(&event);
                assert_eq!(result, text, "Direct string content should be returned as-is");
            }
        }
    }
}
