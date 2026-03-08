pub mod codex_stream;
pub mod harness;
pub mod manager;
pub mod process;
pub mod stream;
pub mod topics_claude_md;

pub use harness::{normalize_harness_name, HarnessRegistry, DEFAULT_HARNESS};
pub use manager::ClaudeManager;
pub use process::{ClaudeOutput, ClaudeProcess};
pub use stream::{
    extract_text_from_event, extract_text_from_event_with_options, ExtractOptions, StreamEvent,
};
pub use topics_claude_md::TOPICS_CLAUDE_MD;
