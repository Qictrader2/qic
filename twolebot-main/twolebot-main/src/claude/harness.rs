use crate::claude::process::{ClaudeOutput, ClaudeProcess};
use crate::claude::stream::ExtractOptions;
use crate::claude::{codex_stream, codex_stream::CodexStreamEvent};
use crate::error::{Result, TwolebotError};
use crate::storage::SettingsStore;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

pub const DEFAULT_HARNESS: &str = "claude";

pub struct HarnessRequest {
    pub prompt: String,
    pub work_dir: PathBuf,
    pub continue_conversation: bool,
    pub extract_options: ExtractOptions,
}

type HarnessFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub trait ChatHarness: Send + Sync {
    fn name(&self) -> &'static str;
    fn run_streaming(
        &self,
        request: HarnessRequest,
        tx: mpsc::Sender<ClaudeOutput>,
    ) -> HarnessFuture;
}

#[derive(Clone)]
pub struct ClaudeCliHarness {
    default_model: Arc<str>,
    settings_store: Arc<SettingsStore>,
    timeout_ms: u64,
}

impl ClaudeCliHarness {
    pub fn new(
        default_model: impl Into<String>,
        timeout_ms: u64,
        settings_store: Arc<SettingsStore>,
    ) -> Self {
        Self {
            default_model: Arc::from(default_model.into()),
            settings_store,
            timeout_ms,
        }
    }

    fn resolve_model(&self) -> String {
        let settings_model = self.settings_store.get().claude_model;
        if settings_model.is_empty() {
            self.default_model.to_string()
        } else {
            settings_model
        }
    }
}

impl ChatHarness for ClaudeCliHarness {
    fn name(&self) -> &'static str {
        DEFAULT_HARNESS
    }

    fn run_streaming(
        &self,
        request: HarnessRequest,
        tx: mpsc::Sender<ClaudeOutput>,
    ) -> HarnessFuture {
        let model = self.resolve_model();
        let timeout_ms = self.timeout_ms;

        Box::pin(async move {
            ClaudeProcess::new(&model)
                .with_timeout(timeout_ms)
                .with_extract_options(request.extract_options)
                .run_streaming(
                    &request.prompt,
                    &request.work_dir,
                    request.continue_conversation,
                    tx,
                )
                .await
        })
    }
}

/// Simple built-in harness useful for diagnostics and future harness plumbing.
/// It never spawns external processes and emits a single final response.
pub struct EchoHarness;

impl ChatHarness for EchoHarness {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn run_streaming(
        &self,
        request: HarnessRequest,
        tx: mpsc::Sender<ClaudeOutput>,
    ) -> HarnessFuture {
        Box::pin(async move {
            let trimmed = request.prompt.trim();
            let response = if trimmed.is_empty() {
                "(echo harness) Empty prompt".to_string()
            } else {
                format!("(echo harness) {trimmed}")
            };
            tx.send(ClaudeOutput::Complete(response))
                .await
                .map_err(|e| TwolebotError::claude(format!("echo harness send failed: {e}")))?;
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct CodexCliHarness {
    timeout_ms: u64,
}

impl CodexCliHarness {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }
}

impl ChatHarness for CodexCliHarness {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn run_streaming(
        &self,
        request: HarnessRequest,
        tx: mpsc::Sender<ClaudeOutput>,
    ) -> HarnessFuture {
        let timeout_ms = self.timeout_ms;
        Box::pin(async move {
            let output_file =
                std::env::temp_dir().join(format!("twolebot-codex-{}.txt", uuid::Uuid::new_v4()));
            let output_file_str = output_file.to_string_lossy().to_string();

            let mut cmd = tokio::process::Command::new("codex");
            cmd.arg("exec")
                .arg("--cd")
                .arg(&request.work_dir)
                .arg("--skip-git-repo-check")
                .arg("--dangerously-bypass-approvals-and-sandbox")
                .arg("--color")
                .arg("never")
                .arg("--json")
                .arg("--output-last-message")
                .arg(&output_file_str);

            if request.continue_conversation {
                cmd.arg("resume").arg("--last");
            }

            cmd.arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            let mut child = cmd
                .spawn()
                .map_err(|e| TwolebotError::claude(format!("Failed to spawn codex exec: {e}")))?;

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| TwolebotError::claude("Failed to capture codex stdout"))?;
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| TwolebotError::claude("Failed to capture codex stderr"))?;

            let stdout_handle = tokio::spawn(stream_codex_stdout(
                stdout,
                request.extract_options,
                tx.clone(),
            ));
            let stderr_handle = tokio::spawn(read_codex_stderr(stderr));

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(request.prompt.as_bytes())
                    .await
                    .map_err(|e| {
                        TwolebotError::claude(format!("Failed to write codex prompt: {e}"))
                    })?;
            }

            let status =
                match tokio::time::timeout(Duration::from_millis(timeout_ms), child.wait()).await {
                    Ok(Ok(status)) => status,
                    Ok(Err(e)) => {
                        let _ = tokio::fs::remove_file(&output_file).await;
                        return Err(TwolebotError::claude(format!(
                            "Failed waiting for codex exec output: {e}"
                        )));
                    }
                    Err(_) => {
                        let _ = child.kill().await;
                        let stdout_result = stdout_handle
                            .await
                            .unwrap_or_else(|_| CodexStdoutResult::default());
                        let _ = stderr_handle.await;
                        let _ = tokio::fs::remove_file(&output_file).await;
                        let _ = tx
                            .send(ClaudeOutput::Timeout {
                                partial_output: stdout_result.filtered_text.clone(),
                            })
                            .await;
                        return Err(TwolebotError::timeout(format!(
                            "Codex harness timeout after {}ms",
                            timeout_ms
                        )));
                    }
                };

            let stdout_result = stdout_handle.await.map_err(|e| {
                TwolebotError::claude(format!("Codex stdout task join failure: {e}"))
            })?;
            let stderr_output = stderr_handle.await.map_err(|e| {
                TwolebotError::claude(format!("Codex stderr task join failure: {e}"))
            })?;

            if !status.success() {
                let stderr = stderr_output.trim().to_string();
                let stdout = stdout_result.raw_output.trim().to_string();
                let reason = if !stderr.is_empty() {
                    stderr
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    format!("codex exited with status {}", status)
                };
                let _ = tokio::fs::remove_file(&output_file).await;
                return Err(TwolebotError::claude(format!(
                    "Codex harness failed: {reason}"
                )));
            }

            let final_message = if !stdout_result.filtered_text.trim().is_empty() {
                stdout_result.filtered_text
            } else {
                match tokio::fs::read_to_string(&output_file).await {
                    Ok(contents) if !contents.trim().is_empty() => contents,
                    _ if !stdout_result.raw_output.trim().is_empty() => stdout_result.raw_output,
                    _ => "(codex harness) Completed with no output".to_string(),
                }
            };
            let _ = tokio::fs::remove_file(&output_file).await;

            tx.send(ClaudeOutput::Complete(final_message))
                .await
                .map_err(|e| TwolebotError::claude(format!("codex harness send failed: {e}")))?;
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct HarnessRegistry {
    default_harness: String,
    harnesses: HashMap<String, Arc<dyn ChatHarness>>,
}

impl HarnessRegistry {
    pub fn with_defaults(
        model: impl Into<String>,
        timeout_ms: u64,
        settings_store: Arc<SettingsStore>,
    ) -> Self {
        Self {
            default_harness: DEFAULT_HARNESS.to_string(),
            harnesses: HashMap::new(),
        }
        .register(ClaudeCliHarness::new(model, timeout_ms, settings_store))
        .register(CodexCliHarness::new(timeout_ms))
        .register(EchoHarness)
    }

    pub fn register<H>(mut self, harness: H) -> Self
    where
        H: ChatHarness + 'static,
    {
        self.harnesses
            .insert(normalize_harness_name(harness.name()), Arc::new(harness));
        self
    }

    pub fn resolve(&self, requested: &str) -> (String, Arc<dyn ChatHarness>) {
        let requested_normalized = normalize_harness_name(requested);
        if let Some(harness) = self.harnesses.get(&requested_normalized) {
            return (requested_normalized, harness.clone());
        }

        let fallback = normalize_harness_name(&self.default_harness);
        if let Some(harness) = self.harnesses.get(&fallback) {
            return (fallback, harness.clone());
        }

        panic!("default harness '{fallback}' is not registered");
    }
}

#[derive(Default)]
struct CodexStdoutResult {
    filtered_text: String,
    raw_output: String,
}

async fn stream_codex_stdout(
    stdout: tokio::process::ChildStdout,
    extract_options: ExtractOptions,
    tx: mpsc::Sender<ClaudeOutput>,
) -> CodexStdoutResult {
    let mut reader = BufReader::new(stdout).lines();
    let mut filtered_text = String::new();
    let mut raw_output = String::new();

    while let Ok(Some(line)) = reader.next_line().await {
        if !line.trim().is_empty() {
            raw_output.push_str(&line);
            raw_output.push('\n');
        }

        let Ok(event) = serde_json::from_str::<CodexStreamEvent>(&line) else {
            continue;
        };

        let text = codex_stream::extract_text_from_event_with_options(&event, &extract_options);
        if text.is_empty() {
            continue;
        }

        filtered_text.push_str(&text);
        if tx.send(ClaudeOutput::Text(text)).await.is_err() {
            break;
        }
    }

    CodexStdoutResult {
        filtered_text,
        raw_output,
    }
}

async fn read_codex_stderr(stderr: tokio::process::ChildStderr) -> String {
    let mut reader = BufReader::new(stderr).lines();
    let mut stderr_output = String::new();
    while let Ok(Some(line)) = reader.next_line().await {
        if !line.trim().is_empty() {
            stderr_output.push_str(&line);
            stderr_output.push('\n');
        }
    }
    stderr_output
}

pub fn normalize_harness_name(name: &str) -> String {
    let lowered = name.trim().to_ascii_lowercase();
    if lowered.is_empty() {
        DEFAULT_HARNESS.to_string()
    } else {
        lowered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_defaults_include_codex() {
        let dir = tempfile::tempdir().unwrap();
        let settings_store =
            Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap());
        let registry =
            HarnessRegistry::with_defaults("claude-opus-4-6", 60_000, settings_store);
        let (name, _) = registry.resolve("codex");
        assert_eq!(name, "codex");
    }
}
