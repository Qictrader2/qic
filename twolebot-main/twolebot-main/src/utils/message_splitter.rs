/// Split a message into chunks that fit within Telegram's message size limit.
/// Tries to split at natural boundaries (code blocks, paragraphs, sentences, words).
pub fn split_message_smart(text: &str) -> Vec<String> {
    const MAX_SIZE: usize = 3900; // Leave buffer for Telegram overhead
    const MIN_SIZE: usize = 2000;

    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut remaining = text.to_string();

    while !remaining.is_empty() {
        if remaining.len() <= MAX_SIZE {
            chunks.push(remaining);
            break;
        }

        let (split_at, found_delimiter) = match find_best_split(&remaining, MIN_SIZE, MAX_SIZE) {
            Some(pos) => (pos, true),
            None => {
                let byte_limit = MAX_SIZE.min(remaining.len());
                let split_pos = find_char_boundary(&remaining, byte_limit);
                let split_pos = if split_pos == 0 && byte_limit > 0 {
                    remaining
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| i)
                        .unwrap_or(remaining.len())
                } else {
                    split_pos
                };
                (split_pos, false)
            }
        };

        if split_at == 0 {
            let first_char_boundary = remaining
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());
            chunks.push(remaining[..first_char_boundary].to_string());
            remaining = remaining[first_char_boundary..].to_string();
            continue;
        }

        let (chunk, rest) = remaining.split_at(split_at);
        chunks.push(chunk.to_string());

        remaining = if !found_delimiter && !rest.is_empty() {
            format!("...{}", rest)
        } else {
            rest.to_string()
        };
    }

    chunks
}

fn find_char_boundary(text: &str, byte_limit: usize) -> usize {
    if text.is_char_boundary(byte_limit) {
        return byte_limit;
    }

    let mut boundary = byte_limit;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

fn find_best_split(text: &str, min: usize, max: usize) -> Option<usize> {
    let search_max = find_char_boundary(text, max.min(text.len()));
    if min > search_max {
        return None;
    }

    // Priority: code block end > paragraph > sentence > line > word
    let patterns = [
        "```\n", // After code block
        "\n\n",  // After paragraph
        ". ",    // After sentence
        ".\n",   // After sentence with newline
        "\n",    // After line
        " ",     // After word
    ];

    for delimiter in patterns {
        if search_max <= text.len() && text.is_char_boundary(search_max) {
            let search_range = &text[..search_max];
            if let Some(pos) = search_range.rfind(delimiter) {
                let split_pos = pos + delimiter.len();
                if split_pos >= min && split_pos <= text.len() {
                    return Some(split_pos);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_message_not_split() {
        let text = "This is a short message.";
        let chunks = split_message_smart(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_exact_max_size_not_split() {
        let text = "a".repeat(3900);
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_long_message_split_at_newline() {
        let part1 = "a".repeat(3000);
        let part2 = "b".repeat(2000);
        let text = format!("{}\n{}", part1, part2);
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], format!("{}\n", part1));
        assert_eq!(chunks[1], part2);
    }

    #[test]
    fn test_empty_string() {
        let chunks = split_message_smart("");
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_all_chunks_within_bounds() {
        let text = "Lorem ipsum dolor sit amet. ".repeat(500);
        let chunks = split_message_smart(&text);

        for (i, chunk) in chunks.iter().enumerate() {
            assert!(
                chunk.len() <= 3900,
                "Chunk {} is too long: {} chars",
                i,
                chunk.len()
            );
        }
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_text_with_patterns(min_len: usize, max_len: usize) -> impl Strategy<Value = String> {
            prop::collection::vec(
                prop::sample::select(vec![
                    "hello", "world", "test", "message", "split", "chunk", "telegram", "bot",
                    "text", "data", "info", "content", "\n", "\n\n", ". ", ", ", "; ", ": ", "! ",
                    "? ", "a", "b", "c", "the", "and", "or", "but", "for",
                ]),
                (min_len / 5)..=(max_len / 5),
            )
            .prop_map(|parts| parts.join(""))
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_no_text_lost(text in arb_text_with_patterns(0, 10000)) {
                let chunks = split_message_smart(&text);

                if text.is_empty() {
                    assert!(chunks.is_empty() || chunks == vec![""]);
                    return Ok(());
                }

                let reconstructed = chunks.join("").replace("...", "");
                let original_chars: Vec<char> = text.chars().collect();
                let reconstructed_chars: Vec<char> = reconstructed.chars().collect();

                assert_eq!(
                    original_chars, reconstructed_chars,
                    "Characters were lost during splitting"
                );
            }

            #[test]
            fn prop_all_chunks_within_max_size(text in arb_text_with_patterns(0, 10000)) {
                let chunks = split_message_smart(&text);
                for (i, chunk) in chunks.iter().enumerate() {
                    assert!(
                        chunk.len() <= 3900,
                        "Chunk {} exceeds maximum size: {} > 3900",
                        i,
                        chunk.len()
                    );
                }
            }

            #[test]
            fn prop_short_text_not_split(text in arb_text_with_patterns(0, 3900)) {
                let chunks = split_message_smart(&text);
                if text.len() <= 3900 {
                    assert!(
                        chunks.len() <= 1,
                        "Short text of {} bytes was split unnecessarily",
                        text.len()
                    );
                }
            }
        }
    }
}
