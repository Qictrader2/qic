/// A simple, platform-agnostic representation of a model response that contains
/// fenced code blocks.
///
/// We intentionally only model the pieces we need for reliable rendering across
/// chat platforms (text vs code blocks). Other styling (bold/italics/links) can
/// be added later if/when needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentSegment {
    Text(String),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
}

/// Parse Markdown-ish fenced code blocks (triple backticks) into segments.
///
/// This parser is intentionally conservative:
/// - A fence start is a line that (after optional leading spaces) begins with
///   "```" and may include a language identifier.
/// - A fence end is a line that (after optional leading spaces) begins with
///   "```" and contains nothing else but whitespace.
/// - If the input ends while inside a code block, we treat the remainder as a
///   code block (unclosed fence).
pub fn parse_fenced_code_blocks(input: &str) -> Vec<ContentSegment> {
    // Normalize Windows newlines so splitting logic is consistent.
    let input = input.replace("\r\n", "\n");

    let mut segments: Vec<ContentSegment> = Vec::new();
    let mut text_buf = String::new();
    let mut code_buf = String::new();
    let mut in_code = false;
    let mut code_lang: Option<String> = None;

    for line in input.split_inclusive('\n') {
        if !in_code {
            if let Some(lang) = fence_start_language(line) {
                if !text_buf.is_empty() {
                    segments.push(ContentSegment::Text(std::mem::take(&mut text_buf)));
                }
                in_code = true;
                code_lang = lang;
                continue;
            }

            text_buf.push_str(line);
            continue;
        }

        // in_code == true
        if is_fence_end(line) {
            segments.push(ContentSegment::CodeBlock {
                language: code_lang.take(),
                code: std::mem::take(&mut code_buf),
            });
            in_code = false;
            continue;
        }

        code_buf.push_str(line);
    }

    if in_code {
        segments.push(ContentSegment::CodeBlock {
            language: code_lang.take(),
            code: code_buf,
        });
    } else if !text_buf.is_empty() {
        segments.push(ContentSegment::Text(text_buf));
    }

    segments
}

fn fence_start_language(line: &str) -> Option<Option<String>> {
    let trimmed = line.trim_start_matches(' ');
    if !trimmed.starts_with("```") {
        return None;
    }

    // Everything after the fence marker on the same line is treated as the
    // "info string". We take the first whitespace-separated token as language.
    let after = &trimmed[3..];
    let info = after.trim();
    if info.is_empty() {
        return Some(None);
    }

    let lang = info
        .split_whitespace()
        .next()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    Some(lang)
}

fn is_fence_end(line: &str) -> bool {
    let trimmed = line.trim_start_matches(' ');
    if !trimmed.starts_with("```") {
        return false;
    }

    let after = &trimmed[3..];
    after.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_code_block() {
        let input = "hello\n```rust\nfn main() {}\n```\nworld\n";
        let segs = parse_fenced_code_blocks(input);

        assert_eq!(
            segs,
            vec![
                ContentSegment::Text("hello\n".to_string()),
                ContentSegment::CodeBlock {
                    language: Some("rust".to_string()),
                    code: "fn main() {}\n".to_string(),
                },
                ContentSegment::Text("world\n".to_string()),
            ]
        );
    }

    #[test]
    fn treats_unclosed_fence_as_code_block() {
        let input = "hello\n```\nline1\nline2\n";
        let segs = parse_fenced_code_blocks(input);
        assert_eq!(
            segs,
            vec![
                ContentSegment::Text("hello\n".to_string()),
                ContentSegment::CodeBlock {
                    language: None,
                    code: "line1\nline2\n".to_string(),
                }
            ]
        );
    }

    #[test]
    fn does_not_end_on_fence_with_info_string() {
        let input = "```\n```not_end\n```\n";
        let segs = parse_fenced_code_blocks(input);
        assert_eq!(
            segs,
            vec![ContentSegment::CodeBlock {
                language: None,
                code: "```not_end\n".to_string(),
            }]
        );
    }
}
