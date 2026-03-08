//! Text chunking strategies for semantic search.
//!
//! Splits documents into appropriately-sized chunks for embedding.
//! Different strategies for markdown files vs conversation messages.

/// A chunk of text with its position in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The text content of this chunk
    pub text: String,
    /// Zero-based index of this chunk within the document
    pub index: usize,
}

/// Text chunker with configurable parameters.
pub struct Chunker {
    /// Target chunk size in characters
    target_size: usize,
    /// Overlap between adjacent chunks
    overlap: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            target_size: 400,
            overlap: 50,
        }
    }
}

impl Chunker {
    /// Create a new chunker with custom parameters.
    pub fn new(target_size: usize, overlap: usize) -> Self {
        Self {
            target_size,
            overlap,
        }
    }

    /// Chunk a markdown document.
    ///
    /// Strategy:
    /// 1. Split on paragraph boundaries (double newlines)
    /// 2. Merge small paragraphs up to target_size
    /// 3. Split large paragraphs on sentence boundaries
    /// 4. Prepend relevant heading context to chunks
    pub fn chunk_markdown(&self, content: &str) -> Vec<Chunk> {
        if content.trim().is_empty() {
            return Vec::new();
        }

        let paragraphs = self.split_paragraphs(content);
        let mut chunks = Vec::new();
        let mut current_heading = String::new();
        let mut current_text = String::new();
        let mut chunk_index = 0;

        for para in paragraphs {
            let para = para.trim();
            if para.is_empty() {
                continue;
            }

            // Track headings for context
            if para.starts_with('#') {
                current_heading = para.to_string();
                // Don't add heading-only chunks, wait for content
                continue;
            }

            // Build chunk text with heading context
            let text_with_context = if current_heading.is_empty() {
                para.to_string()
            } else {
                format!("{}\n\n{}", current_heading, para)
            };

            // Check if adding this would exceed target
            let combined_len = if current_text.is_empty() {
                text_with_context.len()
            } else {
                current_text.len() + 2 + text_with_context.len() // +2 for "\n\n"
            };

            if combined_len <= self.target_size {
                // Merge into current chunk
                if current_text.is_empty() {
                    current_text = text_with_context;
                } else {
                    current_text.push_str("\n\n");
                    current_text.push_str(&text_with_context);
                }
            } else {
                // Flush current chunk if non-empty
                if !current_text.is_empty() {
                    chunks.push(Chunk {
                        text: current_text.clone(),
                        index: chunk_index,
                    });
                    chunk_index += 1;
                    current_text.clear();
                }

                // Handle the new paragraph
                if text_with_context.len() <= self.target_size {
                    current_text = text_with_context;
                } else {
                    // Split large paragraph into smaller chunks
                    let sub_chunks = self.split_large_text(&text_with_context);
                    for sub in sub_chunks {
                        chunks.push(Chunk {
                            text: sub,
                            index: chunk_index,
                        });
                        chunk_index += 1;
                    }
                }
            }
        }

        // Flush remaining text
        if !current_text.is_empty() {
            chunks.push(Chunk {
                text: current_text,
                index: chunk_index,
            });
        }

        chunks
    }

    /// Chunk a conversation message.
    ///
    /// Strategy:
    /// - Short messages (< target_size): single chunk
    /// - Long messages: split on sentence boundaries
    pub fn chunk_message(&self, content: &str) -> Vec<Chunk> {
        if content.trim().is_empty() {
            return Vec::new();
        }

        let content = content.trim();

        if content.len() <= self.target_size {
            return vec![Chunk {
                text: content.to_string(),
                index: 0,
            }];
        }

        self.split_large_text(content)
            .into_iter()
            .enumerate()
            .map(|(index, text)| Chunk { text, index })
            .collect()
    }

    /// Split text into paragraphs (double newline separated).
    fn split_paragraphs<'a>(&self, text: &'a str) -> Vec<&'a str> {
        text.split("\n\n").collect()
    }

    /// Split a large text into smaller chunks on sentence boundaries.
    fn split_large_text(&self, text: &str) -> Vec<String> {
        let sentences = self.split_sentences(text);
        let num_sentences = sentences.len();
        let mut chunks = Vec::new();
        let mut current = String::new();

        for sentence in sentences {
            let combined_len = if current.is_empty() {
                sentence.len()
            } else {
                current.len() + 1 + sentence.len() // +1 for space
            };

            if combined_len <= self.target_size {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            } else {
                // Flush current chunk
                if !current.is_empty() {
                    // Add overlap from end of previous chunk to start of next
                    let overlap_text = self.get_overlap_suffix(&current);
                    chunks.push(current);
                    current = overlap_text;
                }

                // Add the sentence
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);

                // If single sentence exceeds target, just keep it as is
                if current.len() > self.target_size && num_sentences == 1 {
                    chunks.push(current);
                    current = String::new();
                }
            }
        }

        // Flush remaining
        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    /// Split text into sentences.
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            current.push(c);

            // Sentence endings: . ! ? followed by space or end
            if matches!(c, '.' | '!' | '?') {
                // Peek ahead would be nice, but we'll handle in next iteration
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_string());
                }
                current.clear();
            }
        }

        // Remaining text (no sentence ending)
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_string());
        }

        sentences
    }

    /// Get the last `overlap` characters as prefix for next chunk.
    /// Uses char boundaries for proper UTF-8 handling.
    fn get_overlap_suffix(&self, text: &str) -> String {
        let char_count = text.chars().count();
        if char_count <= self.overlap {
            return text.to_string();
        }

        // Get the last `overlap` characters (not bytes)
        let suffix: String = text.chars().skip(char_count - self.overlap).collect();

        // Try to break at word boundary
        if let Some(space_idx) = suffix.find(' ') {
            suffix[space_idx + 1..].to_string()
        } else {
            suffix
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_empty_document() {
        let chunker = Chunker::default();
        assert!(chunker.chunk_markdown("").is_empty());
        assert!(chunker.chunk_markdown("   ").is_empty());
        assert!(chunker.chunk_message("").is_empty());
    }

    #[test]
    fn test_chunk_small_document() {
        let chunker = Chunker::default();
        let content = "This is a small document.";
        let chunks = chunker.chunk_markdown(content);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, content);
        assert_eq!(chunks[0].index, 0);
    }

    #[test]
    fn test_chunk_with_heading() {
        let chunker = Chunker::default();
        let content = "# Title\n\nSome content here.";
        let chunks = chunker.chunk_markdown(content);

        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text.contains("# Title"));
        assert!(chunks[0].text.contains("Some content here."));
    }

    #[test]
    fn test_chunk_multiple_paragraphs() {
        let chunker = Chunker::new(100, 20);
        let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunker.chunk_markdown(content);

        // All should fit in one chunk with size 100
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text.contains("First"));
        assert!(chunks[0].text.contains("Third"));
    }

    #[test]
    fn test_chunk_large_document() {
        let chunker = Chunker::new(50, 10);
        // Multiple sentences allow splitting while preserving sentence boundaries
        let content = "First sentence here. Second sentence continues. Third sentence adds more. Fourth finishes.";
        let chunks = chunker.chunk_markdown(content);

        assert!(chunks.len() > 1, "Should split into multiple chunks");

        // Verify all content is covered (accounting for overlap)
        let all_text: String = chunks
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(all_text.contains("First"));
        assert!(all_text.contains("Fourth"));
    }

    #[test]
    fn test_chunk_message_short() {
        let chunker = Chunker::default();
        let content = "Short message";
        let chunks = chunker.chunk_message(content);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, content);
    }

    #[test]
    fn test_chunk_message_long() {
        let chunker = Chunker::new(50, 10);
        let content = "This is a much longer message that should definitely be split into multiple chunks. It contains several sentences. Each sentence adds more length.";
        let chunks = chunker.chunk_message(content);

        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_chunk_indices_sequential() {
        let chunker = Chunker::new(30, 5);
        let content = "First part. Second part. Third part. Fourth part.";
        let chunks = chunker.chunk_message(content);

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_heading_context_preserved() {
        let chunker = Chunker::new(200, 20);
        let content =
            "# Main Heading\n\nFirst section content.\n\n## Sub Heading\n\nSecond section content.";
        let chunks = chunker.chunk_markdown(content);

        // The sub heading should become context for its content
        let has_sub_heading_context = chunks
            .iter()
            .any(|c| c.text.contains("## Sub Heading") && c.text.contains("Second section"));
        assert!(
            has_sub_heading_context,
            "Heading should be included as context"
        );
    }

    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            #[test]
            fn prop_chunk_indices_are_sequential(content in ".{0,1000}") {
                let chunker = Chunker::default();
                let chunks = chunker.chunk_markdown(&content);

                for (i, chunk) in chunks.iter().enumerate() {
                    prop_assert_eq!(chunk.index, i);
                }
            }

            #[test]
            fn prop_no_empty_chunks(content in ".{1,500}") {
                let chunker = Chunker::default();
                let chunks = chunker.chunk_markdown(&content);

                for chunk in chunks {
                    prop_assert!(!chunk.text.trim().is_empty(), "Chunk should not be empty");
                }
            }

            #[test]
            fn prop_message_chunking_covers_content(content in "[a-zA-Z ]{10,200}") {
                let chunker = Chunker::new(50, 10);
                let chunks = chunker.chunk_message(&content);

                // All words from original should appear in some chunk
                for word in content.split_whitespace() {
                    let found = chunks.iter().any(|c| c.text.contains(word));
                    prop_assert!(found, "Word '{}' should appear in some chunk", word);
                }
            }
        }
    }
}
