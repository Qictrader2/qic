use crate::claude::stream::{extract_text_from_event_with_options, ExtractOptions, StreamEvent};
use crate::error::{Result, TwolebotError};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::timeout;

const DEFAULT_TIMEOUT_MS: u64 = 21_600_000; // 6 hours

/// Output event from Claude process
#[derive(Debug, Clone)]
pub enum ClaudeOutput {
    /// Text chunk from Claude
    Text(String),
    /// Process completed successfully
    Complete(String),
    /// Process failed with error
    Error(String),
    /// Process timed out
    Timeout { partial_output: String },
}

/// Claude CLI process wrapper
pub struct ClaudeProcess {
    model: String,
    timeout_ms: u64,
    extract_options: ExtractOptions,
}

impl ClaudeProcess {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            extract_options: ExtractOptions::default(),
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_extract_options(mut self, options: ExtractOptions) -> Self {
        self.extract_options = options;
        self
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    pub fn extract_options(&self) -> &ExtractOptions {
        &self.extract_options
    }

    /// Run Claude with a prompt, streaming output to a channel
    pub async fn run_streaming(
        &self,
        prompt: &str,
        work_dir: &Path,
        continue_conversation: bool,
        tx: mpsc::Sender<ClaudeOutput>,
    ) -> Result<()> {
        let mut args = vec![
            "-p".to_string(),
            prompt.to_string(),
            "--model".to_string(),
            self.model.clone(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ];

        if continue_conversation {
            args.push("-c".to_string());
        }

        let mut child = Command::new("claude")
            .args(&args)
            .current_dir(work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| TwolebotError::claude(format!("Failed to spawn Claude: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TwolebotError::claude("Failed to capture stdout"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TwolebotError::claude("Failed to capture stderr"))?;

        // Spawn stderr handler
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if !line.trim().is_empty() {
                    tracing::debug!("Claude stderr: {}", line);
                }
            }
        });

        let mut reader = BufReader::new(stdout).lines();
        let mut accumulated_text = String::new();

        // Process with timeout
        let process_result = timeout(
            Duration::from_millis(self.timeout_ms),
            self.process_output(&mut reader, &mut child, &mut accumulated_text, &tx),
        )
        .await;

        // Clean up
        let _ = child.kill().await;
        let _ = stderr_handle.await;

        match process_result {
            Ok(Ok(())) => {
                let _ = tx.send(ClaudeOutput::Complete(accumulated_text)).await;
                Ok(())
            }
            Ok(Err(e)) => {
                let _ = tx.send(ClaudeOutput::Error(e.to_string())).await;
                Err(e)
            }
            Err(_) => {
                let _ = tx
                    .send(ClaudeOutput::Timeout {
                        partial_output: accumulated_text,
                    })
                    .await;
                Err(TwolebotError::timeout(format!(
                    "Process timeout after {}ms",
                    self.timeout_ms
                )))
            }
        }
    }

    async fn process_output(
        &self,
        reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
        child: &mut Child,
        accumulated_text: &mut String,
        tx: &mpsc::Sender<ClaudeOutput>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                line_result = reader.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            if let Ok(event) = serde_json::from_str::<StreamEvent>(&line) {
                                let text = extract_text_from_event_with_options(&event, &self.extract_options);
                                if !text.is_empty() {
                                    accumulated_text.push_str(&text);
                                    let _ = tx.send(ClaudeOutput::Text(text)).await;
                                }
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            tracing::error!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                status = child.wait() => {
                    match status {
                        Ok(exit_status) => {
                            tracing::debug!("Claude process exited with: {}", exit_status);
                            // Drain remaining output
                            while let Ok(Some(line)) = reader.next_line().await {
                                if let Ok(event) = serde_json::from_str::<StreamEvent>(&line) {
                                    let text = extract_text_from_event_with_options(&event, &self.extract_options);
                                    if !text.is_empty() {
                                        accumulated_text.push_str(&text);
                                        let _ = tx.send(ClaudeOutput::Text(text)).await;
                                    }
                                }
                            }
                            break;
                        }
                        Err(e) => {
                            return Err(TwolebotError::claude(format!("Process wait error: {}", e)));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Run Claude and collect all output (non-streaming)
    pub async fn run(
        &self,
        prompt: &str,
        work_dir: &Path,
        continue_conversation: bool,
    ) -> Result<String> {
        let (tx, mut rx) = mpsc::channel(100);

        let prompt = prompt.to_string();
        let work_dir = work_dir.to_path_buf();
        let process = Self {
            model: self.model.clone(),
            timeout_ms: self.timeout_ms,
            extract_options: self.extract_options.clone(),
        };

        tokio::spawn(async move {
            let _ = process
                .run_streaming(&prompt, &work_dir, continue_conversation, tx)
                .await;
        });

        let mut result = String::new();

        while let Some(output) = rx.recv().await {
            match output {
                ClaudeOutput::Text(text) => result.push_str(&text),
                ClaudeOutput::Complete(text) => {
                    result = text;
                    break;
                }
                ClaudeOutput::Error(e) => return Err(TwolebotError::claude(e)),
                ClaudeOutput::Timeout { partial_output } => {
                    return Err(TwolebotError::timeout(format!(
                        "Timeout with partial output: {} chars",
                        partial_output.len()
                    )));
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_process_creation() {
        let process = ClaudeProcess::new("claude-opus-4-6");
        assert_eq!(process.model, "claude-opus-4-6");
        assert_eq!(process.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn test_claude_process_with_timeout() {
        let process = ClaudeProcess::new("claude-opus-4-6").with_timeout(30_000);
        assert_eq!(process.timeout_ms, 30_000);
    }
}
