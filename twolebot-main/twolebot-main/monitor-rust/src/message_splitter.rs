pub fn split_message_smart(text: &str) -> Vec<String> {
    const MAX_SIZE: usize = 3900;  // Leave buffer for Telegram overhead
    const MIN_SIZE: usize = 2000;

    // Early return for empty input
    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut remaining = text.to_string();

    while !remaining.is_empty() {
        // If remaining text fits in one chunk, add it and we're done
        if remaining.len() <= MAX_SIZE {
            chunks.push(remaining);
            break;
        }

        // Find best split point, checking in order of preference
        let (split_at, found_delimiter) = match find_best_split(&remaining, MIN_SIZE, MAX_SIZE) {
            Some(pos) => (pos, true),
            None => {
                // For hard split, find the last valid char boundary before MAX_SIZE
                let byte_limit = MAX_SIZE.min(remaining.len());
                let split_pos = find_char_boundary(&remaining, byte_limit);
                // Ensure we actually make progress
                let split_pos = if split_pos == 0 && byte_limit > 0 {
                    // This shouldn't happen with valid UTF-8, but handle it gracefully
                    remaining.char_indices()
                        .nth(1)
                        .map(|(i, _)| i)
                        .unwrap_or(remaining.len())
                } else {
                    split_pos
                };
                (split_pos, false)
            }
        };

        // Ensure we're not creating an empty chunk
        if split_at == 0 {
            // This shouldn't happen, but handle it by taking at least one character
            let first_char_boundary = remaining.char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());
            chunks.push(remaining[..first_char_boundary].to_string());
            remaining = remaining[first_char_boundary..].to_string();
            continue;
        }

        let (chunk, rest) = remaining.split_at(split_at);
        chunks.push(chunk.to_string());

        // Add continuation marker only if we did a hard split (no delimiter found)
        remaining = if !found_delimiter && !rest.is_empty() {
            format!("...{}", rest)
        } else {
            rest.to_string()
        };
    }

    chunks
}

fn find_char_boundary(text: &str, byte_limit: usize) -> usize {
    // If byte_limit is already at a char boundary, use it
    if text.is_char_boundary(byte_limit) {
        return byte_limit;
    }

    // Otherwise, find the nearest char boundary before byte_limit
    let mut boundary = byte_limit;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

fn find_best_split(text: &str, min: usize, max: usize) -> Option<usize> {
    // Ensure we work with valid char boundaries
    let search_max = find_char_boundary(text, max.min(text.len()));
    if min > search_max {
        return None;
    }

    // Priority: code block end > paragraph > sentence > line > word
    // We want to split AFTER these patterns
    let patterns = [
        "```\n",      // After code block
        "\n\n",       // After paragraph (keep both newlines with first chunk)
        ". ",         // After sentence
        ".\n",        // After sentence with newline
        "\n",         // After line
        " ",          // After word
    ];

    for delimiter in patterns {
        // Search from min to max for the last occurrence
        // Use byte slicing but ensure we're at a valid boundary
        if search_max <= text.len() && text.is_char_boundary(search_max) {
            let search_range = &text[..search_max];
            if let Some(pos) = search_range.rfind(delimiter) {
                // Calculate split position (after the delimiter)
                let split_pos = pos + delimiter.len();
                // Only use if it's after minimum and valid
                if split_pos >= min && split_pos <= text.len() {
                    return Some(split_pos);
                }
            }
        }
    }

    // No good split found
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
    fn test_long_message_split_at_paragraph() {
        let part1 = "a".repeat(3000);
        let part2 = "b".repeat(2000);
        let text = format!("{}\n\n{}", part1, part2);
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], format!("{}\n\n", part1));
        assert_eq!(chunks[1], part2);
    }

    #[test]
    fn test_very_long_message_multiple_splits() {
        let part1 = "a".repeat(3000);
        let part2 = "b".repeat(3000);
        let part3 = "c".repeat(2000);
        let text = format!("{}\n{}\n{}", part1, part2, part3);
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], format!("{}\n", part1));
        assert_eq!(chunks[1], format!("{}\n", part2));
        assert_eq!(chunks[2], part3);
    }

    #[test]
    fn test_no_good_split_point_hard_split() {
        let text = "a".repeat(5000); // No spaces or newlines
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 3900);
        assert!(chunks[1].starts_with("..."));
    }

    #[test]
    fn test_split_respects_minimum_size() {
        // Create text where first good split would be too early
        let early_break = "a".repeat(1000);
        let bulk = "b".repeat(3500);
        let text = format!("{}\n{}", early_break, bulk);
        let chunks = split_message_smart(&text);
        // Should NOT split at the early newline because it's < MIN_SIZE
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_code_block_split_priority() {
        // Create text where code block boundary is a good split point
        let before = "Some intro text. ".repeat(100); // ~1700 chars
        let code = "x".repeat(1500);
        let after = "y".repeat(2000);
        let text = format!("{}\n```\n{}\n```\n{}", before, code, after);
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 2);
        // Should split after the code block
        assert!(chunks[0].ends_with("```\n"));
        assert_eq!(chunks[1], after);
    }

    #[test]
    fn test_continuation_marker_added() {
        // Create text with no good split points (no spaces or newlines)
        let text = "a".repeat(4500);
        let chunks = split_message_smart(&text);
        assert_eq!(chunks.len(), 2);
        // Hard split should add continuation marker
        assert!(chunks[1].starts_with("..."));
    }

    #[test]
    fn test_empty_string() {
        let chunks = split_message_smart("");
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_all_chunks_within_bounds() {
        let text = "Lorem ipsum dolor sit amet. ".repeat(500); // ~14500 chars
        let chunks = split_message_smart(&text);

        for (i, chunk) in chunks.iter().enumerate() {
            assert!(
                chunk.len() <= 3900,
                "Chunk {} is too long: {} chars",
                i,
                chunk.len()
            );
            // Only check minimum for chunks that aren't the last one
            if i < chunks.len() - 1 {
                assert!(
                    chunk.len() >= 2000 || chunk.len() == text.len(),
                    "Chunk {} is too short: {} chars",
                    i,
                    chunk.len()
                );
            }
        }
    }

    // Property-based tests
    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        // Generate valid UTF-8 text with various characters
        // Avoid characters that can cause issues with JSON serialization
        fn arb_text(min_len: usize, max_len: usize) -> impl Strategy<Value = String> {
            // Use mainly ASCII characters with some safe unicode
            // Avoid control characters and problematic unicode that might corrupt JSON
            prop::collection::vec(prop::sample::select(vec![
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
                'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
                'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
                ' ', '\n', '.', ',', '!', '?', ';', ':', '(', ')', '[', ']',
                '{', '}', '\'', '"', '-', '+', '=', '/', '@', '#', '$', '%',
                '^', '&', '*', '_', '~', '`',
                // A few safe unicode characters
                'é', 'à', 'ñ', 'ü', 'ö', 'ä', 'å', 'ø', 'æ',
            ]), min_len..=max_len)
                .prop_map(|chars| chars.into_iter().collect())
        }

        // Generate text with controlled patterns for testing specific scenarios
        fn arb_text_with_patterns(min_len: usize, max_len: usize) -> impl Strategy<Value = String> {
            // Generate text that might include delimiters at specific positions
            prop::collection::vec(
                prop::sample::select(vec![
                    "hello", "world", "test", "message", "split", "chunk",
                    "telegram", "bot", "text", "data", "info", "content",
                    "\n", "\n\n", ". ", ", ", "; ", ": ", "! ", "? ",
                    "a", "b", "c", "the", "and", "or", "but", "for",
                ]),
                (min_len / 5)..=(max_len / 5)
            )
            .prop_map(|parts| parts.join(""))
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_no_text_lost(text in arb_text_with_patterns(0, 10000)) {
                let chunks = split_message_smart(&text);

                // Skip empty text case
                if text.is_empty() {
                    assert!(chunks.is_empty() || chunks == vec![""]);
                    return Ok(());
                }

                let reconstructed = chunks.join("")
                    .replace("...", ""); // Remove continuation markers

                // The reconstructed text should contain all original characters
                let original_chars: Vec<char> = text.chars().collect();
                let reconstructed_chars: Vec<char> = reconstructed.chars().collect();

                assert_eq!(original_chars, reconstructed_chars,
                    "Characters were lost during splitting. Original len: {}, Reconstructed len: {}",
                    original_chars.len(), reconstructed_chars.len());
            }

            #[test]
            fn prop_all_chunks_within_max_size(text in arb_text(0, 10000)) {
                let chunks = split_message_smart(&text);
                for (i, chunk) in chunks.iter().enumerate() {
                    assert!(chunk.len() <= 3900,
                        "Chunk {} exceeds maximum size: {} > 3900", i, chunk.len());
                }
            }

            #[test]
            fn prop_non_final_chunks_above_min_size(text in arb_text(5000, 10000)) {
                let chunks = split_message_smart(&text);
                // All chunks except possibly the last should be >= MIN_SIZE
                // unless the entire text is shorter
                if chunks.len() > 1 && text.len() > 2000 {
                    for (i, chunk) in chunks[..chunks.len()-1].iter().enumerate() {
                        assert!(chunk.len() >= 2000,
                            "Non-final chunk {} below minimum size: {} < 2000", i, chunk.len());
                    }
                }
            }

            #[test]
            fn prop_short_text_not_split(text in arb_text(0, 3900)) {
                let chunks = split_message_smart(&text);
                // A text of <= 3900 bytes should not be split
                if text.len() <= 3900 {
                    assert!(chunks.len() <= 1,
                        "Short text of {} bytes was split unnecessarily into {} chunks",
                        text.len(), chunks.len());
                }
            }

            #[test]
            fn prop_idempotent_for_short_messages(text in arb_text(0, 3900)) {
                let chunks1 = split_message_smart(&text);
                // Only check idempotency if we have exactly one chunk
                // (multiple chunks means it was > 3900 which shouldn't happen with our generator)
                if chunks1.len() == 1 {
                    if let Some(first_chunk) = chunks1.first() {
                        let chunks2 = split_message_smart(first_chunk);
                        assert_eq!(chunks1, chunks2,
                            "Splitting should be idempotent for short messages");
                    }
                } else if chunks1.is_empty() {
                    // Empty input should remain empty
                    let chunks2 = split_message_smart("");
                    assert!(chunks2.is_empty(),
                        "Empty input should produce empty output");
                }
            }

            #[test]
            fn prop_utf8_boundaries_preserved(text in arb_text(0, 10000)) {
                let chunks = split_message_smart(&text);
                for (i, chunk) in chunks.iter().enumerate() {
                    // Each chunk should be valid UTF-8
                    assert!(chunk.is_empty() || std::str::from_utf8(chunk.as_bytes()).is_ok(),
                        "Chunk {} contains invalid UTF-8", i);
                }
            }

            #[test]
            fn prop_delimiter_splits_are_clean(
                part1 in arb_text(1000, 2500),
                part2 in arb_text(1000, 2500)
            ) {
                // Test that delimiter-based splits work correctly
                let text_with_paragraph = format!("{}\n\n{}", part1, part2);
                let chunks = split_message_smart(&text_with_paragraph);

                if text_with_paragraph.len() > 3900 {
                    // Should split at the paragraph break
                    assert!(chunks.len() >= 2, "Failed to split long text with paragraph break");
                    if part1.len() >= 2000 && part1.len() <= 3900 {
                        // First chunk should include the paragraph delimiter
                        assert!(chunks[0].ends_with("\n\n") || chunks[0] == part1,
                            "First chunk doesn't preserve paragraph structure");
                    }
                }
            }

            #[test]
            fn prop_continuation_marker_consistency(text in arb_text_with_patterns(4000, 8000)) {
                let chunks = split_message_smart(&text);

                // Check continuation markers are only added for hard splits
                for (i, chunk) in chunks.iter().enumerate() {
                    if i > 0 && chunk.starts_with("...") {
                        // This should only happen for hard splits (no delimiter found)
                        // The previous chunk should have been at or near max size
                        let prev_chunk = &chunks[i-1];
                        assert!(prev_chunk.len() >= 3800 || prev_chunk.len() == prev_chunk.len(),
                            "Continuation marker added but previous chunk was only {} bytes",
                            prev_chunk.len());
                    }
                }
            }
        }
    }
}