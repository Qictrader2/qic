/// Agent instructions compiled into the binary.
/// Written to data/topics/CLAUDE.md on startup so that every topic
/// inherits these instructions via Claude's ancestor directory loading.
pub const TOPICS_CLAUDE_MD: &str = include_str!("topics_claude_md.txt");
