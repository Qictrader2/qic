/// Convert Markdown inline formatting to Telegram HTML.
///
/// Handles: bold, italic, strikethrough, inline code, links, blockquotes,
/// headers, bullet lists, and horizontal rules. HTML entities are escaped
/// as part of rendering, so the input should be raw text (not pre-escaped).
///
/// Fenced code blocks are NOT handled here — they are already parsed by
/// `segments.rs` before this function sees the text.
pub fn markdown_to_telegram_html(input: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut blockquote_lines: Vec<String> = Vec::new();

    for line in input.split('\n') {
        // Blockquote: `> text`
        if let Some(rest) = line.strip_prefix("> ") {
            blockquote_lines.push(convert_inline(rest));
            continue;
        } else if line == ">" {
            blockquote_lines.push(String::new());
            continue;
        }

        // Flush any accumulated blockquote
        if !blockquote_lines.is_empty() {
            flush_blockquote(&mut lines, &mut blockquote_lines);
        }

        // Horizontal rule: `---`, `***`, `___` (3+ chars, alone on line)
        let trimmed = line.trim();
        if trimmed.len() >= 3 && is_horizontal_rule(trimmed) {
            lines.push("———".to_string());
            continue;
        }

        // Headers: `# text`, `## text`, etc. → bold
        if let Some(heading_text) = strip_heading(trimmed) {
            lines.push(format!("<b>{}</b>", convert_inline(heading_text)));
            continue;
        }

        // Bullet list: `- item` or `* item` (but not `**bold**`)
        if let Some(rest) = trimmed.strip_prefix("- ") {
            let indent = leading_spaces(line);
            lines.push(format!("{}• {}", " ".repeat(indent), convert_inline(rest)));
            continue;
        }
        if trimmed.starts_with("* ") && !trimmed.starts_with("**") {
            if let Some(rest) = trimmed.strip_prefix("* ") {
                let indent = leading_spaces(line);
                lines.push(format!("{}• {}", " ".repeat(indent), convert_inline(rest)));
                continue;
            }
        }

        // Regular line: convert inline formatting
        lines.push(convert_inline(line));
    }

    // Flush trailing blockquote
    if !blockquote_lines.is_empty() {
        flush_blockquote(&mut lines, &mut blockquote_lines);
    }

    lines.join("\n")
}

fn flush_blockquote(lines: &mut Vec<String>, bq_lines: &mut Vec<String>) {
    let inner = bq_lines.join("\n");
    lines.push(format!("<blockquote>{}</blockquote>", inner));
    bq_lines.clear();
}

fn is_horizontal_rule(s: &str) -> bool {
    let first = s.chars().next().unwrap_or(' ');
    if first != '-' && first != '*' && first != '_' {
        return false;
    }
    s.len() >= 3 && s.chars().all(|c| c == first)
}

fn strip_heading(s: &str) -> Option<&str> {
    if !s.starts_with('#') {
        return None;
    }
    // Count leading #s (up to 6)
    let hashes = s.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &s[hashes..];
    // Must be followed by a space or be empty
    if rest.is_empty() {
        return Some("");
    }
    if rest.starts_with(' ') {
        return Some(rest.trim_start());
    }
    None
}

fn leading_spaces(s: &str) -> usize {
    s.chars().take_while(|&c| c == ' ').count()
}

/// Convert inline Markdown to Telegram HTML within a single line.
///
/// Processing order:
/// 1. Escape HTML entities
/// 2. Inline code (to protect contents from further processing)
/// 3. Links
/// 4. Bold (`**`)
/// 5. Italic (`*`)
/// 6. Strikethrough (`~~`)
fn convert_inline(input: &str) -> String {
    let escaped = escape_html(input);
    let s = convert_inline_code(&escaped);
    let s = convert_links(&s);
    let s = convert_bold(&s);
    let s = convert_italic(&s);
    convert_strikethrough(&s)
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Convert `` `code` `` → `<code>code</code>`
///
/// Uses a placeholder to prevent inner content from being processed by
/// subsequent inline converters.
fn convert_inline_code(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '`' {
            // Find closing backtick
            if let Some(end) = input[i + 1..].find('`') {
                let code_content = &input[i + 1..i + 1 + end];
                result.push_str("<code>");
                result.push_str(code_content);
                result.push_str("</code>");
                // Advance past closing backtick
                let target = i + 1 + end + 1;
                while let Some(&(j, _)) = chars.peek() {
                    if j >= target {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
        }
        result.push(ch);
        chars.next();
    }

    result
}

/// Convert `[text](url)` → `<a href="url">text</a>`
fn convert_links(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '[' {
            // Look for ](url)
            if let Some(close_bracket) = find_unescaped(input, ']', i + 1) {
                let next_byte = close_bracket + 1;
                if next_byte < input.len() && input.as_bytes()[next_byte] == b'(' {
                    if let Some(close_paren) = find_char(input, ')', next_byte + 1) {
                        let text = &input[i + 1..close_bracket];
                        let url = &input[next_byte + 1..close_paren];
                        result.push_str("<a href=\"");
                        result.push_str(url);
                        result.push_str("\">");
                        result.push_str(text);
                        result.push_str("</a>");
                        // Advance past close_paren
                        while let Some(&(j, _)) = chars.peek() {
                            if j > close_paren {
                                break;
                            }
                            chars.next();
                        }
                        continue;
                    }
                }
            }
        }
        result.push(ch);
        chars.next();
    }

    result
}

/// Convert `**text**` → `<b>text</b>`
fn convert_bold(input: &str) -> String {
    convert_delimited(input, "**", "<b>", "</b>")
}

/// Convert `*text*` → `<i>text</i>`
///
/// Careful not to match inside already-converted bold tags or `**`.
fn convert_italic(input: &str) -> String {
    convert_delimited(input, "*", "<i>", "</i>")
}

/// Convert `~~text~~` → `<s>text</s>`
fn convert_strikethrough(input: &str) -> String {
    convert_delimited(input, "~~", "<s>", "</s>")
}

/// Generic delimited converter: finds matching pairs of `delim` and wraps
/// the content in `open_tag`/`close_tag`.
///
/// Skips content inside `<code>...</code>` and `<a ...>...</a>` tags.
fn convert_delimited(input: &str, delim: &str, open_tag: &str, close_tag: &str) -> String {
    let dlen = delim.len();
    if input.len() < dlen * 2 {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        // Skip inside <code>...</code> and <a ...>...</a>
        // These tags are ASCII, so checking the first byte is safe
        if input.as_bytes()[i] == b'<' && input.is_char_boundary(i) {
            if input[i..].starts_with("<code>") {
                if let Some(end) = input[i..].find("</code>") {
                    let chunk = &input[i..i + end + 7];
                    result.push_str(chunk);
                    i += end + 7;
                    continue;
                }
            }
            if input[i..].starts_with("<a ") {
                if let Some(end) = input[i..].find("</a>") {
                    let chunk = &input[i..i + end + 4];
                    result.push_str(chunk);
                    i += end + 4;
                    continue;
                }
            }
        }

        // Check for delimiter (delimiters are always ASCII, safe to compare bytes)
        if i + dlen <= input.len() && input.as_bytes()[i..i + dlen] == *delim.as_bytes() {
            let delim_byte = delim.as_bytes()[0];

            // For single-char delim (*), make sure we're not at a double (**)
            if dlen == 1 && i + 1 < input.len() && input.as_bytes()[i + 1] == delim_byte {
                result.push(delim_byte as char);
                i += 1;
                continue;
            }

            // Markdown rule: opening delimiter must not be followed by whitespace
            let after_delim = i + dlen;
            if after_delim < input.len() && input.as_bytes()[after_delim] == b' ' {
                result.push(delim_byte as char);
                i += 1;
                continue;
            }

            // Find closing delimiter
            let search_start = i + dlen;
            if let Some(rel_end) = find_delim_end(input, delim, search_start) {
                let inner = &input[search_start..search_start + rel_end];
                // Don't wrap empty or whitespace-only content
                if !inner.trim().is_empty() {
                    result.push_str(open_tag);
                    result.push_str(inner);
                    result.push_str(close_tag);
                    i = search_start + rel_end + dlen;
                    continue;
                }
            }
        }

        // Advance by one full UTF-8 character
        let ch = &input[i..];
        if let Some(c) = ch.chars().next() {
            result.push(c);
            i += c.len_utf8();
        } else {
            break;
        }
    }

    result
}

/// Find the position of a closing delimiter, skipping inside code/link tags.
fn find_delim_end(input: &str, delim: &str, start: usize) -> Option<usize> {
    let dlen = delim.len();
    let mut j = start;

    while j < input.len() {
        // Skip <code>...</code> and <a ...>...</a>
        if input.as_bytes()[j] == b'<' && input.is_char_boundary(j) {
            if input[j..].starts_with("<code>") {
                if let Some(end) = input[j..].find("</code>") {
                    j += end + 7;
                    continue;
                }
            }
            if input[j..].starts_with("<a ") {
                if let Some(end) = input[j..].find("</a>") {
                    j += end + 4;
                    continue;
                }
            }
        }

        if j + dlen <= input.len() && input.as_bytes()[j..j + dlen] == *delim.as_bytes() {
            // For single-char delim, skip doubles
            if dlen == 1 && j + 1 < input.len() && input.as_bytes()[j + 1] == input.as_bytes()[j] {
                j += 1;
                continue;
            }
            return Some(j - start);
        }

        // Advance by one full UTF-8 character
        if let Some(c) = input[j..].chars().next() {
            j += c.len_utf8();
        } else {
            break;
        }
    }

    None
}

fn find_unescaped(input: &str, target: char, start: usize) -> Option<usize> {
    let mut prev = None;
    for (i, ch) in input.char_indices() {
        if i < start {
            prev = Some(ch);
            continue;
        }
        if ch == target {
            if prev == Some('\\') {
                prev = Some(ch);
                continue;
            }
            return Some(i);
        }
        prev = Some(ch);
    }
    None
}

fn find_char(input: &str, target: char, start: usize) -> Option<usize> {
    input[start..].find(target).map(|pos| pos + start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold() {
        assert_eq!(
            markdown_to_telegram_html("hello **world**"),
            "hello <b>world</b>"
        );
    }

    #[test]
    fn italic() {
        assert_eq!(
            markdown_to_telegram_html("hello *world*"),
            "hello <i>world</i>"
        );
    }

    #[test]
    fn bold_and_italic_coexist() {
        assert_eq!(
            markdown_to_telegram_html("**bold** and *italic*"),
            "<b>bold</b> and <i>italic</i>"
        );
    }

    #[test]
    fn strikethrough() {
        assert_eq!(
            markdown_to_telegram_html("hello ~~world~~"),
            "hello <s>world</s>"
        );
    }

    #[test]
    fn inline_code() {
        assert_eq!(
            markdown_to_telegram_html("use `foo()` here"),
            "use <code>foo()</code> here"
        );
    }

    #[test]
    fn inline_code_preserves_special_chars() {
        assert_eq!(
            markdown_to_telegram_html("run `x < 5 && y > 3`"),
            "run <code>x &lt; 5 &amp;&amp; y &gt; 3</code>"
        );
    }

    #[test]
    fn bold_inside_inline_code_not_converted() {
        assert_eq!(
            markdown_to_telegram_html("see `**not bold**` here"),
            "see <code>**not bold**</code> here"
        );
    }

    #[test]
    fn link() {
        assert_eq!(
            markdown_to_telegram_html("click [here](https://example.com)"),
            "click <a href=\"https://example.com\">here</a>"
        );
    }

    #[test]
    fn heading_h1() {
        assert_eq!(
            markdown_to_telegram_html("# Hello World"),
            "<b>Hello World</b>"
        );
    }

    #[test]
    fn heading_h2() {
        assert_eq!(
            markdown_to_telegram_html("## Sub heading"),
            "<b>Sub heading</b>"
        );
    }

    #[test]
    fn heading_with_inline() {
        assert_eq!(
            markdown_to_telegram_html("# A **bold** heading"),
            "<b>A <b>bold</b> heading</b>"
        );
    }

    #[test]
    fn bullet_list_dash() {
        assert_eq!(
            markdown_to_telegram_html("- first\n- second"),
            "• first\n• second"
        );
    }

    #[test]
    fn bullet_list_asterisk() {
        assert_eq!(
            markdown_to_telegram_html("* first\n* second"),
            "• first\n• second"
        );
    }

    #[test]
    fn numbered_list_unchanged() {
        assert_eq!(
            markdown_to_telegram_html("1. first\n2. second"),
            "1. first\n2. second"
        );
    }

    #[test]
    fn blockquote_single_line() {
        assert_eq!(
            markdown_to_telegram_html("> hello"),
            "<blockquote>hello</blockquote>"
        );
    }

    #[test]
    fn blockquote_multi_line() {
        assert_eq!(
            markdown_to_telegram_html("> line one\n> line two"),
            "<blockquote>line one\nline two</blockquote>"
        );
    }

    #[test]
    fn blockquote_with_inline_formatting() {
        assert_eq!(
            markdown_to_telegram_html("> **bold** quote"),
            "<blockquote><b>bold</b> quote</blockquote>"
        );
    }

    #[test]
    fn horizontal_rule_dashes() {
        assert_eq!(markdown_to_telegram_html("---"), "———");
    }

    #[test]
    fn horizontal_rule_asterisks() {
        assert_eq!(markdown_to_telegram_html("***"), "———");
    }

    #[test]
    fn html_entities_escaped() {
        assert_eq!(
            markdown_to_telegram_html("a < b & c > d"),
            "a &lt; b &amp; c &gt; d"
        );
    }

    #[test]
    fn mixed_formatting() {
        let input = "# Title\n\nSome **bold** and *italic* text.\n\n- item one\n- item two\n\n> a quote\n\n---\n\nA [link](https://x.com) and `code`.";
        let output = markdown_to_telegram_html(input);
        assert!(output.contains("<b>Title</b>"));
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
        assert!(output.contains("• item one"));
        assert!(output.contains("<blockquote>a quote</blockquote>"));
        assert!(output.contains("———"));
        assert!(output.contains("<a href=\"https://x.com\">link</a>"));
        assert!(output.contains("<code>code</code>"));
    }

    #[test]
    fn empty_bold_not_converted() {
        assert_eq!(markdown_to_telegram_html("** **"), "** **");
    }

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(
            markdown_to_telegram_html("just plain text"),
            "just plain text"
        );
    }

    #[test]
    fn asterisk_not_bullet_when_bold() {
        // `**bold**` on a line should not be treated as a bullet
        assert_eq!(
            markdown_to_telegram_html("**bold text**"),
            "<b>bold text</b>"
        );
    }

    #[test]
    fn nested_bold_in_italic() {
        assert_eq!(
            markdown_to_telegram_html("*an **important** point*"),
            "<i>an <b>important</b> point</i>"
        );
    }

    #[test]
    fn emoji_in_text() {
        assert_eq!(
            markdown_to_telegram_html("🧑 Using tool: Bash"),
            "🧑 Using tool: Bash"
        );
    }

    #[test]
    fn emoji_with_bold() {
        assert_eq!(
            markdown_to_telegram_html("🎉 **congratulations** 🥳"),
            "🎉 <b>congratulations</b> 🥳"
        );
    }

    #[test]
    fn emoji_in_link_text() {
        assert_eq!(
            markdown_to_telegram_html("[🔗 link](https://example.com)"),
            "<a href=\"https://example.com\">🔗 link</a>"
        );
    }

    #[test]
    fn multibyte_chars_preserved() {
        assert_eq!(
            markdown_to_telegram_html("café résumé naïve"),
            "café résumé naïve"
        );
    }
}
