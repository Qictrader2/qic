use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

const MAX_LOG_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            component: component.into(),
            message: message.into(),
            context: None,
        }
    }

}

/// Ring buffer logger that persists to JSONL file
pub struct Logger {
    entries: VecDeque<LogEntry>,
    file_path: std::path::PathBuf,
    writer: Option<BufWriter<File>>,
}

impl Logger {
    /// Create a new logger, loading existing entries from file
    pub fn new(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file_path = file_path.as_ref().to_path_buf();

        // Load existing entries (up to MAX_LOG_ENTRIES from the end)
        let mut entries = VecDeque::with_capacity(MAX_LOG_ENTRIES);

        if file_path.exists() {
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);

            for line in reader.lines().flatten() {
                if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                    if entries.len() >= MAX_LOG_ENTRIES {
                        entries.pop_front();
                    }
                    entries.push_back(entry);
                }
            }
        }

        // Open file for appending
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        let writer = BufWriter::new(file);

        Ok(Self {
            entries,
            file_path,
            writer: Some(writer),
        })
    }

    /// Log an entry
    pub fn log(&mut self, entry: LogEntry) {
        // Write to file
        if let Some(ref mut writer) = self.writer {
            if let Ok(json) = serde_json::to_string(&entry) {
                let _ = writeln!(writer, "{}", json);
                let _ = writer.flush();
            }
        }

        // Add to ring buffer
        if self.entries.len() >= MAX_LOG_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Get recent entries (newest first)
    pub fn recent(&self, limit: usize) -> Vec<LogEntry> {
        self.entries.iter().rev().take(limit).cloned().collect()
    }

    /// Get all entries (newest first)
    pub fn all(&self) -> Vec<LogEntry> {
        self.entries.iter().rev().cloned().collect()
    }

    /// Compact the log file (rewrite with only ring buffer contents)
    pub fn compact(&mut self) -> std::io::Result<()> {
        // Close existing writer
        self.writer = None;

        // Rewrite file with current entries
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)?;
        let mut writer = BufWriter::new(file);

        for entry in &self.entries {
            if let Ok(json) = serde_json::to_string(entry) {
                writeln!(writer, "{}", json)?;
            }
        }
        writer.flush()?;

        // Reopen for appending
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        self.writer = Some(BufWriter::new(file));

        Ok(())
    }
}

/// Thread-safe logger wrapper
#[derive(Clone)]
pub struct SharedLogger(Arc<Mutex<Logger>>);

impl SharedLogger {
    pub fn new(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self(Arc::new(Mutex::new(Logger::new(file_path)?))))
    }

    pub fn log(&self, entry: LogEntry) {
        if let Ok(mut logger) = self.0.lock() {
            logger.log(entry);
        }
    }

    pub fn debug(&self, component: impl Into<String>, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Debug, component, message));
    }

    pub fn info(&self, component: impl Into<String>, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Info, component, message));
    }

    pub fn warn(&self, component: impl Into<String>, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Warn, component, message));
    }

    pub fn error(&self, component: impl Into<String>, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Error, component, message));
    }

    pub fn recent(&self, limit: usize) -> Vec<LogEntry> {
        self.0.lock().map(|l| l.recent(limit)).unwrap_or_default()
    }

    pub fn all(&self) -> Vec<LogEntry> {
        self.0.lock().map(|l| l.all()).unwrap_or_default()
    }

    pub fn compact(&self) -> std::io::Result<()> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("Lock poisoned"))?
            .compact()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_logger_creation() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.jsonl");

        let logger = Logger::new(&log_path).unwrap();
        assert!(logger.entries.is_empty());
        assert!(log_path.exists());
    }

    #[test]
    fn test_log_entry() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.jsonl");

        let mut logger = Logger::new(&log_path).unwrap();
        logger.log(LogEntry::new(LogLevel::Info, "test", "Hello world"));

        assert_eq!(logger.entries.len(), 1);

        // Verify file content
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Hello world"));
    }

    #[test]
    fn test_ring_buffer_limit() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.jsonl");

        let mut logger = Logger::new(&log_path).unwrap();

        // Log more than MAX_LOG_ENTRIES
        for i in 0..MAX_LOG_ENTRIES + 100 {
            logger.log(LogEntry::new(
                LogLevel::Info,
                "test",
                format!("Message {}", i),
            ));
        }

        assert_eq!(logger.entries.len(), MAX_LOG_ENTRIES);

        // First entry should be entry #100 (0-99 were evicted)
        let first = logger.entries.front().unwrap();
        assert!(first.message.contains("100"));
    }

    #[test]
    fn test_recent_entries() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.jsonl");

        let mut logger = Logger::new(&log_path).unwrap();

        for i in 0..10 {
            logger.log(LogEntry::new(
                LogLevel::Info,
                "test",
                format!("Message {}", i),
            ));
        }

        let recent = logger.recent(3);
        assert_eq!(recent.len(), 3);

        // Should be newest first
        assert!(recent[0].message.contains("9"));
        assert!(recent[1].message.contains("8"));
        assert!(recent[2].message.contains("7"));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.jsonl");

        // Create logger and add entries
        {
            let mut logger = Logger::new(&log_path).unwrap();
            logger.log(LogEntry::new(LogLevel::Info, "test", "Persistent message"));
        }

        // Create new logger and verify entries loaded
        let logger = Logger::new(&log_path).unwrap();
        assert_eq!(logger.entries.len(), 1);
        assert!(logger.entries[0].message.contains("Persistent message"));
    }

    #[test]
    fn test_compact() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.jsonl");

        let mut logger = Logger::new(&log_path).unwrap();

        // Add some entries
        for i in 0..5 {
            logger.log(LogEntry::new(
                LogLevel::Info,
                "test",
                format!("Message {}", i),
            ));
        }

        // Compact
        logger.compact().unwrap();

        // File should still have all 5 entries
        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 5);
    }
}
