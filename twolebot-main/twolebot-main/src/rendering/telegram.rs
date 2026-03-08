use crate::rendering::markdown::markdown_to_telegram_html;
use crate::rendering::segments::{parse_fenced_code_blocks, ContentSegment};

/// Telegram has a hard limit of 4096 characters for `sendMessage.text`.
///
/// We stay below that with a conservative ceiling, because:
/// - we add small chunk prefixes for multi-part messages
/// - HTML escaping can expand content
pub const TELEGRAM_MAX_MESSAGE_SIZE: usize = 3900;

/// Reserve space for the `[i/n]\n` prefix that `TelegramSender` adds when splitting.
const CHUNK_PREFIX_SLACK: usize = 32;

/// Extra headroom for closing tags that `balance_html_tags` may append
/// when repairing a chunk split mid-tag-pair (e.g. `</blockquote></b></i>`).
const TAG_REPAIR_HEADROOM: usize = 80;

/// Render Markdown-ish content into Telegram HTML chunks.
///
/// The main goal is *reliability*: code blocks (fenced by ``` in the model output)
/// become `<pre><code>...</code></pre>` and will not randomly fail Telegram's
/// Markdown parser (because we no longer use it).
pub fn render_telegram_html_chunks(markdown: &str) -> Vec<String> {
    let segments = parse_fenced_code_blocks(markdown);
    render_segments_to_html_chunks(&segments)
}

fn render_segments_to_html_chunks(segments: &[ContentSegment]) -> Vec<String> {
    let max_len = TELEGRAM_MAX_MESSAGE_SIZE.saturating_sub(CHUNK_PREFIX_SLACK + TAG_REPAIR_HEADROOM);
    let mut chunker = HtmlChunker::new(max_len);

    for seg in segments {
        match seg {
            ContentSegment::Text(t) => {
                let html = markdown_to_telegram_html(t);
                chunker.append_html(&html);
            }
            ContentSegment::CodeBlock { language, code } => {
                chunker.append_code_block(language.as_deref(), code)
            }
        }
    }

    chunker.finish()
}

struct HtmlChunker {
    max_len: usize,
    chunks: Vec<String>,
    current: String,
}

impl HtmlChunker {
    fn new(max_len: usize) -> Self {
        Self {
            max_len,
            chunks: Vec::new(),
            current: String::new(),
        }
    }

    fn finish(mut self) -> Vec<String> {
        if !self.current.is_empty() {
            let (repaired, _) = balance_html_tags(&self.current);
            self.chunks.push(repaired);
        }
        self.chunks
    }

    fn remaining(&self) -> usize {
        self.max_len.saturating_sub(self.current.len())
    }

    fn start_new_chunk(&mut self) {
        if !self.current.is_empty() {
            let (repaired, reopen) = balance_html_tags(&self.current);
            self.chunks.push(repaired);
            self.current = String::new();
            for tag in &reopen {
                self.current.push_str(tag);
            }
        }
    }

    /// Append raw text with HTML escaping. Used only for the defensive fallback
    /// in code block splitting when tags alone exceed max_len.
    fn append_escaped_text(&mut self, text: &str) {
        let mut remaining = text;
        while !remaining.is_empty() {
            let available = self.remaining();
            if available == 0 {
                self.start_new_chunk();
                continue;
            }

            let end = choose_split_point(remaining, available, &["\n\n", "\n", " "]);
            if end == 0 {
                self.start_new_chunk();
                continue;
            }

            let (part, rest) = remaining.split_at(end);
            escape_html_push(&mut self.current, part);
            remaining = rest;
        }
    }

    /// Append pre-rendered HTML (already escaped). Splits at natural boundaries
    /// but does NOT escape — the caller is responsible for correct HTML.
    fn append_html(&mut self, html: &str) {
        let mut remaining = html;
        while !remaining.is_empty() {
            let available = self.remaining();
            if available == 0 {
                self.start_new_chunk();
                continue;
            }

            let end = choose_html_split_point(remaining, available);
            if end == 0 {
                self.start_new_chunk();
                continue;
            }

            let (part, rest) = remaining.split_at(end);
            self.current.push_str(part);
            remaining = rest;
        }
    }

    fn append_code_block(&mut self, language: Option<&str>, code: &str) {
        let start_tag = code_start_tag(language);
        let end_tag = "</code></pre>";
        let overhead = start_tag.len() + end_tag.len();

        let total_needed = overhead.saturating_add(escaped_html_len(code));

        // If we have some text already in the current chunk and the full code
        // block won't fit, start a fresh chunk.
        if !self.current.is_empty() && total_needed > self.remaining() {
            self.start_new_chunk();
        }

        if total_needed <= self.remaining() {
            self.current.push_str(&start_tag);
            escape_html_push(&mut self.current, code);
            self.current.push_str(end_tag);
            return;
        }

        // Code block is too big to fit in a single chunk. Split it into multiple
        // independent `<pre><code>...</code></pre>` blocks.
        if overhead >= self.max_len {
            // Defensive fallback: if tags alone exceed max_len, just emit escaped text.
            self.append_escaped_text(code);
            return;
        }

        let capacity_for_code = self.max_len - overhead;
        let mut remaining = code;
        while !remaining.is_empty() {
            // Keep code pieces self-contained; don't share chunk with previous content.
            if !self.current.is_empty() {
                self.start_new_chunk();
            }

            let mut end = choose_split_point(remaining, capacity_for_code, &["\n"]);
            if end == 0 {
                end = prefix_end_by_escaped_len(remaining, capacity_for_code);
            }

            if end == 0 {
                // Last resort: take at least one char boundary to avoid infinite loops.
                end = remaining
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| i)
                    .unwrap_or(remaining.len());
            }

            let (part, rest) = remaining.split_at(end);
            self.current.push_str(&start_tag);
            escape_html_push(&mut self.current, part);
            self.current.push_str(end_tag);
            remaining = rest;
        }
    }
}

fn code_start_tag(language: Option<&str>) -> String {
    match language.and_then(sanitize_language) {
        Some(lang) => format!("<pre><code class=\"language-{}\">", lang),
        None => "<pre><code>".to_string(),
    }
}

fn sanitize_language(lang: &str) -> Option<String> {
    let lang = lang.trim();
    if lang.is_empty() {
        return None;
    }

    // Keep only safe identifier-ish chars so we never emit unsafe attributes.
    let mut out = String::new();
    for ch in lang.chars() {
        let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '+';
        if ok {
            out.push(ch);
        } else {
            break;
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn escape_html_push(out: &mut String, input: &str) {
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn escaped_html_len(input: &str) -> usize {
    input.chars().map(escaped_html_len_char).sum()
}

fn escaped_html_len_char(ch: char) -> usize {
    match ch {
        '&' => 5,       // &amp;
        '<' | '>' => 4, // &lt; / &gt;
        _ => ch.len_utf8(),
    }
}

/// Choose a split point for pre-rendered HTML, avoiding splits inside tags.
fn choose_html_split_point(s: &str, max_len: usize) -> usize {
    if s.len() <= max_len {
        return s.len();
    }

    // Find the largest char boundary <= max_len so we never slice mid-character.
    let mut boundary = max_len;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    if boundary == 0 {
        return 0;
    }

    let candidate = &s[..boundary];

    // Try splitting at natural boundaries, preferring paragraph > line > space
    for delim in &["\n\n", "\n", " "] {
        if let Some(pos) = candidate.rfind(delim) {
            let split = pos + delim.len();
            if split > 0 {
                return split;
            }
        }
    }

    // Fallback: split at the char boundary we found.
    boundary
}

fn choose_split_point(s: &str, max_escaped_len: usize, delims: &[&str]) -> usize {
    let end = prefix_end_by_escaped_len(s, max_escaped_len);
    if end == 0 {
        return 0;
    }

    let candidate = &s[..end];
    for delim in delims {
        if let Some(pos) = candidate.rfind(delim) {
            let split = pos + delim.len();
            if split > 0 {
                return split;
            }
        }
    }

    end
}

fn prefix_end_by_escaped_len(s: &str, max_escaped_len: usize) -> usize {
    let mut out_len = 0usize;
    let mut end = 0usize;

    for (idx, ch) in s.char_indices() {
        let add = escaped_html_len_char(ch);
        if out_len.saturating_add(add) > max_escaped_len {
            break;
        }
        out_len = out_len.saturating_add(add);
        end = idx + ch.len_utf8();
    }

    end
}

/// Scan HTML for unclosed tags. Close them at the end of the chunk and return
/// the full opening-tag strings so the next chunk can reopen them.
///
/// This prevents Telegram's "can't parse entities: Can't find end tag" errors
/// that occur when `append_html` splits pre-rendered HTML mid-tag-pair.
fn balance_html_tags(html: &str) -> (String, Vec<String>) {
    let mut stack: Vec<(String, String)> = Vec::new(); // (tag_name, full_opening_tag)
    let bytes = html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(gt_rel) = html[i..].find('>') {
                let full_tag = &html[i..i + gt_rel + 1];
                let inner = &html[i + 1..i + gt_rel];

                if let Some(close_name) = inner.strip_prefix('/') {
                    let close_name = close_name.trim();
                    if let Some(pos) = stack.iter().rposition(|(n, _)| n == close_name) {
                        stack.remove(pos);
                    }
                } else {
                    let name = inner
                        .split(|c: char| c.is_whitespace())
                        .next()
                        .unwrap_or(inner);
                    let is_self_closing = inner.ends_with('/');
                    let is_void = matches!(name, "br" | "hr" | "img" | "input" | "meta" | "area" | "col");
                    if !name.is_empty() && !is_self_closing && !is_void {
                        stack.push((name.to_string(), full_tag.to_string()));
                    }
                }

                i += gt_rel + 1;
                continue;
            }
        }
        i += 1;
    }

    if stack.is_empty() {
        return (html.to_string(), Vec::new());
    }

    let mut repaired = html.to_string();
    let mut reopen = Vec::with_capacity(stack.len());

    for (name, full_open) in stack.iter().rev() {
        repaired.push_str("</");
        repaired.push_str(name);
        repaired.push('>');
        reopen.push(full_open.clone());
    }

    reopen.reverse();
    (repaired, reopen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_fenced_code_blocks_as_pre() {
        let input = "before\n```\nA_B\n```\nafter\n";
        let chunks = render_telegram_html_chunks(input);
        let out = chunks.join("");

        assert!(out.contains("before"));
        assert!(out.contains("<pre><code>"));
        assert!(out.contains("A_B\n"));
        assert!(out.contains("</code></pre>"));
        assert!(out.contains("after"));
        assert!(!out.contains("```"));
    }

    #[test]
    fn escapes_html_in_text_and_code() {
        let input = "t: <x>&\n```\n<y>&\n```\n";
        let chunks = render_telegram_html_chunks(input);
        let out = chunks.join("");

        assert!(out.contains("t: &lt;x&gt;&amp;"));
        assert!(out.contains("<pre><code>"));
        assert!(out.contains("&lt;y&gt;&amp;"));
        assert!(!out.contains("<y>"));
    }

    #[test]
    fn chunks_never_exceed_limit() {
        let big = "```\n".to_string() + &"a".repeat(20_000) + "\n```\n";
        let chunks = render_telegram_html_chunks(&big);

        assert!(chunks.len() > 1);
        for c in chunks {
            assert!(
                c.len() <= TELEGRAM_MAX_MESSAGE_SIZE,
                "chunk too big: {}",
                c.len()
            );
            assert!(c.contains("<pre><code"));
            assert!(c.contains("</code></pre>"));
        }
    }

    #[test]
    fn balance_html_tags_noop_when_balanced() {
        let html = "<b>hello</b> <i>world</i>";
        let (repaired, reopen) = balance_html_tags(html);
        assert_eq!(repaired, html);
        assert!(reopen.is_empty());
    }

    #[test]
    fn balance_html_tags_closes_unclosed() {
        let html = "<b>hello <i>world";
        let (repaired, reopen) = balance_html_tags(html);
        assert_eq!(repaired, "<b>hello <i>world</i></b>");
        assert_eq!(reopen, vec!["<b>", "<i>"]);
    }

    #[test]
    fn balance_html_tags_preserves_attributes() {
        let html = "<a href=\"https://example.com\">click";
        let (repaired, reopen) = balance_html_tags(html);
        assert_eq!(repaired, "<a href=\"https://example.com\">click</a>");
        assert_eq!(reopen, vec!["<a href=\"https://example.com\">"]);
    }

    #[test]
    fn balance_html_tags_ignores_void_and_self_closing() {
        let html = "<b>hello<br>world<br/>end";
        let (repaired, reopen) = balance_html_tags(html);
        assert_eq!(repaired, "<b>hello<br>world<br/>end</b>");
        assert_eq!(reopen, vec!["<b>"]);
    }

    #[test]
    fn long_bold_text_produces_balanced_chunks() {
        let long_bold = format!("**{}**", "word ".repeat(1000));
        let chunks = render_telegram_html_chunks(&long_bold);

        assert!(chunks.len() > 1, "expected multiple chunks");
        for (i, chunk) in chunks.iter().enumerate() {
            let opens = chunk.matches("<b>").count();
            let closes = chunk.matches("</b>").count();
            assert_eq!(
                opens, closes,
                "chunk {} has {} <b> but {} </b>: {:?}",
                i, opens, closes, &chunk[..chunk.len().min(200)]
            );
        }
    }

    #[test]
    fn long_italic_text_produces_balanced_chunks() {
        let long_italic = format!("*{}*", "text ".repeat(1000));
        let chunks = render_telegram_html_chunks(&long_italic);

        assert!(chunks.len() > 1, "expected multiple chunks");
        for (i, chunk) in chunks.iter().enumerate() {
            let opens = chunk.matches("<i>").count();
            let closes = chunk.matches("</i>").count();
            assert_eq!(
                opens, closes,
                "chunk {} has {} <i> but {} </i>",
                i, opens, closes
            );
        }
    }

    #[test]
    fn nested_tags_balanced_across_split() {
        let nested = format!("**bold *and italic {}* end**", "x ".repeat(1000));
        let chunks = render_telegram_html_chunks(&nested);

        for (i, chunk) in chunks.iter().enumerate() {
            for tag in &["b", "i"] {
                let opens = chunk.matches(&format!("<{}>", tag)).count();
                let closes = chunk.matches(&format!("</{}>", tag)).count();
                assert_eq!(
                    opens, closes,
                    "chunk {} unbalanced <{}>: {} opens, {} closes",
                    i, tag, opens, closes
                );
            }
        }
    }
}
