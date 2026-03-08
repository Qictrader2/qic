use crate::claude::stream::ExtractOptions;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CodexStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub item: Option<CodexItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CodexItem {
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub aggregated_output: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
}

pub fn extract_text_from_event_with_options(
    event: &CodexStreamEvent,
    options: &ExtractOptions,
) -> String {
    match event.event_type.as_str() {
        "item.started" => extract_started_item_text(event, options),
        "item.completed" => extract_completed_item_text(event, options),
        _ => String::new(),
    }
}

fn extract_started_item_text(event: &CodexStreamEvent, options: &ExtractOptions) -> String {
    if !options.show_tool_messages {
        return String::new();
    }

    let Some(item) = event.item.as_ref() else {
        return String::new();
    };

    if !is_tool_item_type(&item.item_type) {
        return String::new();
    }

    Some(item)
        .and_then(item_tool_name)
        .map(|tool_name| format!("\n🔧 Using tool: {}\n", tool_name))
        .unwrap_or_default()
}

fn extract_completed_item_text(event: &CodexStreamEvent, options: &ExtractOptions) -> String {
    let Some(item) = event.item.as_ref() else {
        return String::new();
    };

    match item.item_type.as_str() {
        "agent_message" => item.text.clone().unwrap_or_default(),
        "reasoning" => {
            if options.show_thinking_messages {
                item.text
                    .as_deref()
                    .map(|text| format!("\n💭 Thinking: {}\n", text))
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
        _ => {
            if options.show_tool_results && is_tool_item_type(&item.item_type) {
                extract_tool_result_text(item)
            } else {
                String::new()
            }
        }
    }
}

fn extract_tool_result_text(item: &CodexItem) -> String {
    let output = item
        .aggregated_output
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();

    let exit_code_suffix = item
        .exit_code
        .map(|code| format!(" (exit {})", code))
        .unwrap_or_default();

    if !output.is_empty() {
        return format!(
            "📊 Tool result from {}{}:\n{}\n",
            item_tool_name(item).unwrap_or_else(|| item.item_type.clone()),
            exit_code_suffix,
            output
        );
    }

    if exit_code_suffix.is_empty() {
        String::new()
    } else {
        format!(
            "📊 Tool result from {}{}\n",
            item_tool_name(item).unwrap_or_else(|| item.item_type.clone()),
            exit_code_suffix
        )
    }
}

fn item_tool_name(item: &CodexItem) -> Option<String> {
    if item.item_type == "command_execution" {
        return item
            .command
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| Some("command_execution".to_string()));
    }

    Some(item.item_type.clone())
}

fn is_tool_item_type(item_type: &str) -> bool {
    matches!(
        item_type,
        "command_execution" | "mcp_tool_call" | "tool_call" | "function_call"
    ) || item_type.contains("tool")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_message_is_visible_by_default() {
        let line =
            r#"{"type":"item.completed","item":{"id":"a","type":"agent_message","text":"hello"}}"#;
        let event: CodexStreamEvent = serde_json::from_str(line).unwrap();

        let result = extract_text_from_event_with_options(&event, &ExtractOptions::default());
        assert_eq!(result, "hello");
    }

    #[test]
    fn tool_messages_are_hidden_by_default() {
        let line = r#"{"type":"item.started","item":{"id":"a","type":"command_execution","command":"ls"}}"#;
        let event: CodexStreamEvent = serde_json::from_str(line).unwrap();

        let result = extract_text_from_event_with_options(&event, &ExtractOptions::default());
        assert_eq!(result, "");
    }

    #[test]
    fn tool_messages_are_visible_when_enabled() {
        let line = r#"{"type":"item.started","item":{"id":"a","type":"command_execution","command":"ls"}}"#;
        let event: CodexStreamEvent = serde_json::from_str(line).unwrap();

        let options = ExtractOptions {
            show_tool_messages: true,
            show_thinking_messages: false,
            show_tool_results: false,
        };
        let result = extract_text_from_event_with_options(&event, &options);
        assert!(result.contains("Using tool"));
    }

    #[test]
    fn tool_results_are_visible_when_enabled() {
        let line = r#"{"type":"item.completed","item":{"id":"a","type":"command_execution","command":"ls","aggregated_output":"a\nb\n","exit_code":0}}"#;
        let event: CodexStreamEvent = serde_json::from_str(line).unwrap();

        let options = ExtractOptions {
            show_tool_messages: false,
            show_thinking_messages: false,
            show_tool_results: true,
        };
        let result = extract_text_from_event_with_options(&event, &options);
        assert!(result.contains("Tool result"));
        assert!(result.contains("a"));
    }

    #[test]
    fn thinking_is_filtered_by_option() {
        let line = r#"{"type":"item.completed","item":{"id":"a","type":"reasoning","text":"internal thought"}}"#;
        let event: CodexStreamEvent = serde_json::from_str(line).unwrap();

        let hidden = extract_text_from_event_with_options(&event, &ExtractOptions::default());
        assert_eq!(hidden, "");

        let shown = extract_text_from_event_with_options(
            &event,
            &ExtractOptions {
                show_tool_messages: false,
                show_thinking_messages: true,
                show_tool_results: false,
            },
        );
        assert!(shown.contains("Thinking:"));
    }
}
