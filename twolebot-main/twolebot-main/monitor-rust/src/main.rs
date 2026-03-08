// use rxrust::prelude::*; // Will use when implementing full rxrust pattern
use monitor_rust::{Job, StreamEvent, extract_text_from_event, message_splitter::split_message_smart};
use serde::Deserialize;
use std::{sync::{Arc, Mutex}, time::Duration, process::Stdio, path::PathBuf, fs, io::Write};
use tokio::{process::{Command, Child, ChildStdin, ChildStdout}, time::{interval, timeout}, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};
use chrono::Local;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Claude Monitor - Process Claude jobs via Telegram", long_about = None)]
struct Args {
    /// Telegram username (with or without @)
    #[arg(short, long)]
    user: String,

    /// Agent ID (0-9, a-f) - identifies which agent this monitor runs as
    #[arg(short, long, default_value = "default")]
    agent: String,

    /// Working directory for Claude (defaults to /tmp/telebot-agent-{agent})
    #[arg(short, long)]
    dir: Option<String>,

    /// Remote Lamdera server URL
    #[arg(short, long, default_value = "https://telebot.lamdera.app")]
    server: String,

    /// Enable voice message responses
    #[arg(long, default_value_t = false)]
    enable_voice: bool,
}

#[derive(Clone)]
struct Config {
    lamdera_url: String,
    poll_interval: u64,
    bot_token: String,
    aggregation_delay: u64,
    process_timeout: u64,
    claude_work_dir: String,
    telegram_username: String,
    agent_id: String,
    session_id: Option<String>,
    enable_voice: bool,
    model_key: String,
}

impl Config {
    fn from_args(args: Args) -> Self {
        // Clean username (remove @ if present)
        let telegram_username = if args.user.starts_with('@') {
            args.user[1..].to_string()
        } else {
            args.user
        };

        // Normalize agent ID to lowercase
        let agent_id = args.agent.to_lowercase();

        // Determine working directory: use provided dir or default to /tmp/telebot-agent-{agent}
        let claude_work_dir = args.dir.unwrap_or_else(|| {
            let default_dir = format!("/tmp/telebot-agent-{}", agent_id);
            // Create the directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&default_dir) {
                eprintln!("Warning: Could not create default directory {}: {}", default_dir, e);
            }
            default_dir
        });

        Self {
            lamdera_url: args.server,
            poll_interval: std::env::var("POLL_INTERVAL").unwrap_or("2000".into()).parse().unwrap_or(2000),
            bot_token: String::new(),  // Will be fetched from backend after auth
            aggregation_delay: std::env::var("AGGREGATION_DELAY").unwrap_or("15000".into()).parse().unwrap_or(15000),
            process_timeout: 21_600_000, // 6 hours
            claude_work_dir,
            telegram_username,
            agent_id,
            session_id: None,
            enable_voice: args.enable_voice,
            model_key: std::env::var("MODEL_KEY").unwrap_or("94af27ab8475db6dcb923dfcb091bd99".into()),
        }
    }
}

fn timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn info(msg: &str) {
    println!("[{}] {}", timestamp(), msg);
}

fn error(msg: &str) {
    eprintln!("[{}] {}", timestamp(), msg);
}

/// Lock file management - prevents multiple monitors running with same agent in same directory
struct LockFile {
    path: PathBuf,
}

impl LockFile {
    /// Try to acquire a lock in the given directory for a specific agent
    /// Returns Err if another monitor with the same agent is already running there
    fn acquire(dir: &str, agent_id: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs::OpenOptions;

        let lock_path = PathBuf::from(dir).join(format!(".telebot-monitor-{}.lock", agent_id));
        let pid = std::process::id();

        // Retry loop to handle race conditions when removing stale locks
        for attempt in 0..3 {
            // Try atomic creation first (fails if file exists)
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut file) => {
                    // Successfully created new lock file atomically
                    writeln!(file, "{}", pid)?;
                    info(&format!("Lock file created: {} (PID: {})", lock_path.display(), pid));
                    return Ok(Self { path: lock_path });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // File exists, check if it's stale
                }
                Err(e) => return Err(e.into()),
            }

            // Lock file exists - check if it's stale
            let existing_content = match fs::read_to_string(&lock_path) {
                Ok(content) => content,
                Err(_) => {
                    // File might have been deleted between checks, retry
                    std::thread::sleep(std::time::Duration::from_millis(50 * (attempt as u64 + 1)));
                    continue;
                }
            };
            let existing_pid: u32 = existing_content.trim().parse().unwrap_or(0);

            if existing_pid > 0 && Self::is_process_running(existing_pid) {
                return Err(format!(
                    "Another monitor for agent '{}' is already running in this directory (PID: {})\n\
                     Lock file: {}\n\
                     If this is incorrect, delete the lock file and try again.",
                    agent_id,
                    existing_pid,
                    lock_path.display()
                ).into());
            }

            // Stale lock file - process is not running
            info(&format!("Removing stale lock file (old PID {} is not running)", existing_pid));

            // Remove stale file - if this fails, another process may have already done it
            let _ = fs::remove_file(&lock_path);

            // Small delay before retry to reduce contention
            std::thread::sleep(std::time::Duration::from_millis(50 * (attempt as u64 + 1)));
        }

        // Final attempt after retries
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    "Failed to acquire lock after multiple attempts - another process may be starting".to_string()
                } else {
                    e.to_string()
                }
            })?;

        writeln!(file, "{}", pid)?;
        info(&format!("Lock file created: {} (PID: {})", lock_path.display(), pid));

        Ok(Self { path: lock_path })
    }

    fn is_process_running(pid: u32) -> bool {
        // On Unix, check /proc/{pid} exists
        #[cfg(unix)]
        {
            PathBuf::from(format!("/proc/{}", pid)).exists()
        }

        #[cfg(not(unix))]
        {
            // On other platforms, assume it's running to be safe
            true
        }
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        // Remove lock file on exit
        if let Err(e) = fs::remove_file(&self.path) {
            error(&format!("Failed to remove lock file: {}", e));
        } else {
            info(&format!("Lock file removed: {}", self.path.display()));
        }
    }
}


async fn make_request<T: for<'de> Deserialize<'de>>(
    config: &Config,
    endpoint: &str,
    method: &str,
    body: Option<serde_json::Value>,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("{}/_r/{}", config.lamdera_url, urlencoding::encode(endpoint));

    let mut req = match method {
        "POST" => client.post(&url),
        _ => client.get(&url),
    };

    // Add authentication header if we have a session ID
    if let Some(ref session_id) = config.session_id {
        req = req.header("Authorization", format!("Bearer {}", session_id));
    }

    if let Some(b) = body {
        req = req.json(&b);
    }

    let resp = req.send().await?;

    if resp.status().is_client_error() || resp.status().is_server_error() {
        return Err(format!("HTTP {}", resp.status()).into());
    }

    let text = resp.text().await?;
    if text.trim().is_empty() {
        return serde_json::from_str("[]").map_err(|e| e.into());
    }

    serde_json::from_str(&text).map_err(|e| {
        error(&format!("Failed to parse JSON from {}: {}", endpoint, &text[..100.min(text.len())]));
        e.into()
    })
}

async fn send_to_telegram(config: &Config, chat_id: &str, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", config.bot_token);

    // DEBUG
    info(&format!("DEBUG - Sending to Telegram:"));
    info(&format!("  URL: {}", url));
    info(&format!("  Chat ID: {}", chat_id));
    info(&format!("  Text length: {} chars", text.len()));

    // Split message if needed
    let chunks = split_message_smart(text);

    for (i, chunk) in chunks.iter().enumerate() {
        // Add chunk indicator if multiple chunks
        let message_text = if chunks.len() > 1 {
            format!("[{}/{}]\n{}", i + 1, chunks.len(), chunk)
        } else {
            chunk.clone()
        };

        // Try with Markdown first
        let response = client.post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": message_text,
                "parse_mode": "Markdown"
            }))
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await?;

            // Check if it's a Markdown parsing error
            if body.contains("can't parse entities") || body.contains("can't parse markdown") {
                if i == 0 {
                    info("Markdown parsing failed, retrying as plain text");
                }

                // Retry without parse_mode
                let retry_response = client.post(&url)
                    .json(&serde_json::json!({
                        "chat_id": chat_id,
                        "text": message_text
                    }))
                    .send()
                    .await?;

                let retry_status = retry_response.status();

                if !retry_status.is_success() {
                    let retry_body = retry_response.text().await?;
                    error(&format!("Telegram API error even with plain text - Status: {}, Body: {}", retry_status, retry_body));
                    return Err(format!("Telegram rejected message even as plain text: {}", retry_body).into());
                } else if i == 0 {
                    info("Successfully sent as plain text after Markdown failure");
                }
            } else {
                error(&format!("Telegram API error - Status: {}, Body: {}", status, body));
                return Err(format!("Telegram rejected message: {}", body).into());
            }
        } else {
            // Check if response says ok=false even with 200 status
            let body = response.text().await?;
            if body.contains("\"ok\":false") {
                error(&format!("Telegram returned ok=false: {}", body));

                // Try plain text
                if i == 0 {
                    info("Retrying as plain text due to ok=false");
                }
                let retry_response = client.post(&url)
                    .json(&serde_json::json!({
                        "chat_id": chat_id,
                        "text": message_text
                    }))
                    .send()
                    .await?;

                let retry_body = retry_response.text().await?;
                if retry_body.contains("\"ok\":false") {
                    return Err(format!("Telegram rejected even plain text: {}", retry_body).into());
                }
            }
        }

        // Small delay between chunks to avoid rate limiting
        if i < chunks.len() - 1 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    Ok(())
}

async fn set_message_reaction(config: &Config, chat_id: &str, message_id: &str, emoji: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/setMessageReaction", config.bot_token);

    let _ = client.post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "reaction": [{"type": "emoji", "emoji": emoji}]
        }))
        .send()
        .await;

    Ok(())
}

fn sanitize_text_for_voice(text: &str) -> String {
    // Filter out lines starting with 🔧 (tool/debug lines)
    let filtered: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("🔧"))
        .collect::<Vec<_>>()
        .join("\n");

    // Remove markdown characters
    let sanitized = filtered
        .replace('*', "")
        .replace('_', "")
        .replace('`', "")
        .replace('[', "")
        .replace(']', "");

    // Replace newlines with periods
    let sanitized = sanitized.replace('\n', ". ");

    // Convert ALL CAPS words to Title Case (F5-TTS spells out acronyms)
    convert_caps_to_title_case(&sanitized)
}

/// Convert ALL CAPS words (3+ chars) to Title Case
fn convert_caps_to_title_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_alphabetic() {
            let word_start = i;
            while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '\'') {
                i += 1;
            }
            let word: String = chars[word_start..i].iter().collect();
            if word.len() >= 3 && word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
                let mut title_case = String::new();
                for (j, c) in word.chars().enumerate() {
                    if j == 0 { title_case.push(c); } else { title_case.extend(c.to_lowercase()); }
                }
                result.push_str(&title_case);
            } else {
                result.push_str(&word);
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Persistent TTS server that keeps the model loaded in memory
struct TtsServer {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl TtsServer {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let home = std::env::var("HOME").unwrap_or("/home/schalk".to_string());
        let tts_script = format!("{}/git/telebot/scripts/tts-server.py", home);

        info("Starting persistent TTS server (loading model ~11s)...");

        // Set PYTHONPATH explicitly with all required paths
        let python_path = format!(
            "{}/.local/lib/python3.12/site-packages:{}/git/rick-voice-f5/src:/usr/local/lib/python3.12/dist-packages:/usr/lib/python3/dist-packages",
            home, home
        );

        let mut child = Command::new("python3")
            .arg(&tts_script)
            .env("PYTHONPATH", &python_path)
            .env("HOME", &home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to capture TTS stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture TTS stdout")?;
        let mut stdout = BufReader::new(stdout);

        // Wait for READY signal with timeout
        let mut ready_line = String::new();
        let ready_result = timeout(Duration::from_secs(120), stdout.read_line(&mut ready_line)).await;

        match ready_result {
            Ok(Ok(_)) => {
                let response: serde_json::Value = serde_json::from_str(&ready_line)?;
                if response.get("ready").and_then(|v| v.as_bool()).unwrap_or(false) {
                    info("TTS server ready!");
                    Ok(Self { child, stdin, stdout })
                } else {
                    let err = response.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                    Err(format!("TTS server failed to start: {}", err).into())
                }
            }
            Ok(Err(e)) => Err(format!("Failed to read from TTS server: {}", e).into()),
            Err(_) => Err("TTS server startup timed out (120s)".into()),
        }
    }

    async fn generate(&mut self, text: &str, output_path: &str, speed: f64, nfe_steps: u32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request = serde_json::json!({
            "text": text,
            "output": output_path,
            "speed": speed,
            "nfe_steps": nfe_steps
        });

        // Send request
        let request_line = format!("{}\n", serde_json::to_string(&request)?);
        self.stdin.write_all(request_line.as_bytes()).await?;
        self.stdin.flush().await?;

        // Read response with timeout
        let mut response_line = String::new();
        let read_result = timeout(Duration::from_secs(300), self.stdout.read_line(&mut response_line)).await;

        match read_result {
            Ok(Ok(0)) => Err("TTS server closed unexpectedly".into()),
            Ok(Ok(_)) => {
                let response: serde_json::Value = serde_json::from_str(&response_line)?;
                if response.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                    Ok(response.get("file").and_then(|v| v.as_str()).unwrap_or(output_path).to_string())
                } else {
                    let err = response.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                    Err(format!("TTS generation failed: {}", err).into())
                }
            }
            Ok(Err(e)) => Err(format!("Failed to read TTS response: {}", e).into()),
            Err(_) => Err("TTS generation timed out (300s)".into()),
        }
    }

    fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,  // Still running
            _ => false,        // Exited or error
        }
    }
}

/// Global TTS server manager with auto-respawn
struct TtsManager {
    server: Option<TtsServer>,
}

impl TtsManager {
    fn new() -> Self {
        Self { server: None }
    }

    async fn ensure_server(&mut self) -> Result<&mut TtsServer, Box<dyn std::error::Error + Send + Sync>> {
        // Check if server is alive
        let needs_restart = match &mut self.server {
            Some(server) => !server.is_alive(),
            None => true,
        };

        if needs_restart {
            if self.server.is_some() {
                info("TTS server died, respawning...");
            }
            self.server = Some(TtsServer::new().await?);
        }

        Ok(self.server.as_mut().unwrap())
    }

    async fn generate(&mut self, text: &str, output_path: &str, speed: f64, nfe_steps: u32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let server = self.ensure_server().await?;
        server.generate(text, output_path, speed, nfe_steps).await
    }
}

async fn generate_voice_file(tts_manager: &Arc<tokio::sync::Mutex<TtsManager>>, text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let sanitized = sanitize_text_for_voice(text);

    if sanitized.trim().is_empty() {
        return Err("No text to generate voice for".into());
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();

    let temp_file = format!("/tmp/voice-{}.ogg", timestamp);

    info(&format!("Generating voice with F5-TTS ({} chars, 64 NFE steps)", sanitized.len()));

    let mut manager = tts_manager.lock().await;
    manager.generate(&sanitized, &temp_file, 1.33, 64).await?;

    Ok(temp_file)
}

async fn upload_to_r2(file_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    info("Voice generated, uploading to R2...");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();

    let upload_url = format!("https://telegram-media-r2.ohmyseoulww.workers.dev/upload/voice-{}.ogg", timestamp);

    // Read file
    let mut file = File::open(file_path).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;

    // Upload
    let client = reqwest::Client::new();
    let response = client.post(&upload_url)
        .header("Content-Type", "audio/ogg")
        .body(contents)
        .send()
        .await?;

    #[derive(serde::Deserialize)]
    struct R2Response {
        success: bool,
        url: String,
    }

    let r2_response: R2Response = response.json().await?;

    if !r2_response.success {
        return Err("R2 upload failed".into());
    }

    info(&format!("Voice uploaded: {}", r2_response.url));
    Ok(r2_response.url)
}

async fn send_voice_message(config: &Config, chat_id: &str, voice_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let voice_endpoint = format!("{}/_r/sendvoice", config.lamdera_url);

    let client = reqwest::Client::new();
    let response = client.post(&voice_endpoint)
        .header("Content-Type", "application/json")
        .header("x-model-key", &config.model_key)
        .json(&serde_json::json!({
            "chatId": chat_id,
            "voiceUrl": voice_url,
            "agentId": config.agent_id
        }))
        .send()
        .await?;

    #[derive(serde::Deserialize)]
    struct VoiceResponse {
        ok: bool,
    }

    let voice_response: VoiceResponse = response.json().await?;

    if !voice_response.ok {
        return Err("Voice endpoint returned error".into());
    }

    info("Voice message sent successfully");
    Ok(())
}

async fn generate_and_send_voice(config: &Config, chat_id: &str, text: &str, tts_manager: &Arc<tokio::sync::Mutex<TtsManager>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Generate voice file
    let temp_file = generate_voice_file(tts_manager, text).await?;

    // Upload to R2
    let voice_url = match upload_to_r2(&temp_file).await {
        Ok(url) => url,
        Err(e) => {
            // Cleanup temp file on error
            let _ = tokio::fs::remove_file(&temp_file).await;
            return Err(e);
        }
    };

    // Send voice message
    let result = send_voice_message(config, chat_id, &voice_url).await;

    // Cleanup temp file
    let _ = tokio::fs::remove_file(&temp_file).await;

    result
}

async fn update_job_status(
    config: &Config,
    job_id: &str,
    output: &str,
    is_complete: bool,
    error_msg: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    make_request::<serde_json::Value>(
        config,
        "claude/jobs/update",
        "POST",
        Some(serde_json::json!({
            "jobId": job_id,
            "output": output,
            "isComplete": is_complete,
            "error": error_msg
        })),
    ).await?;
    Ok(())
}

async fn create_claude_stream(
    config: Arc<Config>,
    job: Job,
    text_accumulator: Arc<Mutex<String>>
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info(&format!("Starting Claude for job {} in {}", job.id, config.claude_work_dir));

    let work_dir = config.claude_work_dir.clone();

    // Check if this is a /clear command
    let is_clear_command = job.prompt.trim().starts_with("/clear");
    let actual_prompt = if is_clear_command {
        // Send feedback to user
        let _ = send_to_telegram(&config, &job.chat_id, "🔄 *Context cleared!* Starting fresh conversation...").await;

        // Use remaining text after /clear, or default message
        let remaining = job.prompt.trim_start_matches("/clear").trim();
        if remaining.is_empty() {
            "Starting fresh! How can I help you today?".to_string()
        } else {
            remaining.to_string()
        }
    } else {
        job.prompt.clone()
    };

    // Build args conditionally
    let mut args = vec![
        "-p", &actual_prompt,
        "--model", "claude-opus-4-6",
        "--output-format", "stream-json",
        "--verbose",
        "--dangerously-skip-permissions"
    ];

    // Only add -c flag if NOT a clear command
    if !is_clear_command {
        args.insert(2, "-c");
    }

    let child = Command::new("claude")
        .args(&args)
        .current_dir(&work_dir)
        .env("CLAUDE_CONFIG_DIR", format!("{}/.claude", std::env::var("HOME").unwrap_or_else(|_| "/home/schalk".to_string())))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let mut child = child;
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let mut reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut accumulated_text = String::new();

    // Spawn stderr handler
    let stderr_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            if !line.trim().is_empty() {
                error(&format!("Claude stderr: {}", line));
            }
        }
    });

    // Handle process exit separately
    let mut child_handle = tokio::spawn(async move {
        child.wait().await
    });

    // Log if we're starting fresh
    if is_clear_command {
        info("Starting Claude with fresh context (no -c flag)");
    }

    // Process stdout with timeout
    let process_result = timeout(Duration::from_millis(config.process_timeout), async {
        loop {
            tokio::select! {
                line_result = reader.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            if let Ok(event) = serde_json::from_str::<StreamEvent>(&line) {
                                let text = extract_text_from_event(&event);
                                if !text.is_empty() {
                                    accumulated_text.push_str(&text);
                                    // Update shared accumulator for periodic updates
                                    if let Ok(mut guard) = text_accumulator.lock() {
                                        *guard = accumulated_text.clone();
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            // EOF reached
                            break;
                        }
                        Err(_) => {
                            // Error reading
                            break;
                        }
                    }
                }
                _exit_status = &mut child_handle => {
                    // Process exited
                    info("Claude process exited");
                    // Read any remaining output
                    while let Ok(Some(line)) = reader.next_line().await {
                        if let Ok(event) = serde_json::from_str::<StreamEvent>(&line) {
                            let text = extract_text_from_event(&event);
                            if !text.is_empty() {
                                accumulated_text.push_str(&text);
                                if let Ok(mut guard) = text_accumulator.lock() {
                                    *guard = accumulated_text.clone();
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        accumulated_text.clone()
    }).await;

    // Clean up
    child_handle.abort();
    let _ = stderr_handle.await;

    match process_result {
        Ok(text) => Ok(text),
        Err(_) => {
            error(&format!("Timeout: killing Claude process for job {}", job.id));
            Err(format!("Process timeout after {}ms", config.process_timeout).into())
        }
    }
}

async fn process_job(config: Arc<Config>, job: Job, tts_manager: Arc<tokio::sync::Mutex<TtsManager>>) {
    let mut current_prompt = job.prompt.clone();
    let mut retry_count = 0u32;

    loop {
        info(&format!("Processing job {} (attempt: {})", job.id, retry_count + 1));
        info(&format!("  Chat ID: {}", job.chat_id));
        info(&format!("  Prompt: \"{}...\"", &current_prompt[..50.min(current_prompt.len())]));

        // Extract message ID from job ID (format: msg-{chatId}-{messageId})
        let message_id = job.id.split('-').nth(2).unwrap_or("");

        // Add ⏳ reaction when starting (only on first attempt)
        if retry_count == 0 {
            let _ = set_message_reaction(&config, &job.chat_id, message_id, "⏳").await;
        }

        let text_accumulator = Arc::new(Mutex::new(String::new()));
        let last_sent = Arc::new(Mutex::new(String::new()));

        let text_accumulator_clone = text_accumulator.clone();
        let last_sent_clone = last_sent.clone();
        let config_clone = config.clone();
        let job_id = job.id.clone();
        let chat_id = job.chat_id.clone();

        // Create a job for this attempt
        let mut attempt_job = job.clone();
        attempt_job.prompt = current_prompt.clone();

        // Start 5-minute ping task
        let config_ping = config.clone();
        let chat_id_ping = chat_id.clone();
        let ping_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300000)).await; // 5 minutes
            let _ = send_to_telegram(&config_ping, &chat_id_ping, "⏱️ *Still running...* The process is taking longer than usual. I'll report back soon.").await;
        });

        // Start periodic update task (sends aggregated delta updates every 15 seconds)
        let update_handle = tokio::spawn(async move {
        let mut update_interval = interval(Duration::from_millis(config_clone.aggregation_delay));
        let mut last_checked_len = 0;
        let mut pending_delta = String::new();

        loop {
            update_interval.tick().await;

            let current_total = text_accumulator_clone.lock().map(|g| g.clone()).unwrap_or_default();

            // Get new text since last check
            if current_total.len() > last_checked_len {
                let new_text = current_total[last_checked_len..].to_string();
                pending_delta.push_str(&new_text);
                last_checked_len = current_total.len();
            }

            // Send aggregated delta if we have any pending text
            if !pending_delta.is_empty() {
                info(&format!("Sending aggregated update: {} new chars", pending_delta.len()));
                match send_to_telegram(&config_clone, &chat_id, &pending_delta).await {
                    Ok(_) => {},
                    Err(e) => error(&format!("FAILED to send periodic update to Telegram: {}", e))
                }
                let _ = update_job_status(&config_clone, &job_id, &current_total, false, None).await;

                // Update last_sent to track what's been sent
                if let Ok(mut guard) = last_sent_clone.lock() {
                    *guard = current_total;
                }

                // Clear pending delta after sending
                pending_delta.clear();
            }
            }
        });

        // Process Claude stream
        let stream_result = create_claude_stream(config.clone(), attempt_job.clone(), text_accumulator.clone()).await;

        match stream_result {
            Ok(final_text) => {
            ping_handle.abort();
            update_handle.abort();

            let last_sent_text = last_sent.lock().map(|g| g.clone()).unwrap_or_default();
            let delta = if final_text.len() > last_sent_text.len() {
                final_text[last_sent_text.len()..].to_string()
            } else {
                String::new()
            };

            if !delta.is_empty() {
                info(&format!("Job complete, sending final update: {} chars", delta.len()));

                // DEBUG: Show ENTIRE final message
                info(&format!("DEBUG - FULL FINAL MESSAGE:\n{}", delta));
                info(&format!("DEBUG - Message length: {} chars", delta.len()));

                match send_to_telegram(&config, &job.chat_id, &delta).await {
                    Ok(_) => info("Final update sent successfully to Telegram"),
                    Err(e) => {
                        error(&format!("FAILED to send final update to Telegram: {}", e));
                        error(&format!("FAILED MESSAGE CONTENT WAS:\n{}", delta));
                    }
                }
                let _ = update_job_status(&config, &job.id, &final_text, true, None).await;

                // Send voice message if enabled
                info(&format!("Voice enabled: {}", config.enable_voice));
                if config.enable_voice {
                    info("Starting voice generation...");
                    match generate_and_send_voice(&config, &job.chat_id, &final_text, &tts_manager).await {
                        Ok(_) => info("Voice message sent successfully"),
                        Err(e) => error(&format!("Voice generation failed (continuing): {}", e)),
                    }
                }

                let _ = set_message_reaction(&config, &job.chat_id, message_id, "✅").await;
            } else if !final_text.is_empty() {
                info("Job complete, all text already sent");
                let _ = update_job_status(&config, &job.id, &final_text, true, None).await;

                // Send voice message if enabled
                info(&format!("Voice enabled: {}", config.enable_voice));
                if config.enable_voice {
                    info("Starting voice generation...");
                    match generate_and_send_voice(&config, &job.chat_id, &final_text, &tts_manager).await {
                        Ok(_) => info("Voice message sent successfully"),
                        Err(e) => error(&format!("Voice generation failed (continuing): {}", e)),
                    }
                }

                let _ = set_message_reaction(&config, &job.chat_id, message_id, "✅").await;
            } else {
                let _ = update_job_status(&config, &job.id, "", true, None).await;
                let _ = set_message_reaction(&config, &job.chat_id, message_id, "⚠️").await;
            }

                info(&format!("Job {} completed successfully", job.id));
                break; // Exit the retry loop on success
            }
            Err(e) => {
                ping_handle.abort();
                update_handle.abort();
                let error_msg = e.to_string();

                // Check if this is a timeout error
                if error_msg.contains("timeout") {
                    error(&format!("Timeout on job {}: {}", job.id, error_msg));

                    retry_count += 1;
                    info(&format!("Notifying chat and retrying job {} (attempt {})", job.id, retry_count + 1));

                    // Send timeout notification to chat
                    let _ = send_to_telegram(
                        &config,
                        &job.chat_id,
                        "⏰ *Process timed out.* Restarting with continuation prompt..."
                    ).await;

                    // Update prompt for next iteration
                    current_prompt = "you timed out you absolute muppet, pls continue, inform me if there's an issue that we need to address, if not just update me and continue".to_string();

                    // Continue to next iteration of the loop
                    continue;
                } else {
                    error(&format!("Error processing job {}: {}", job.id, error_msg));
                }

                let _ = set_message_reaction(&config, &job.chat_id, message_id, "❌").await;
                let _ = update_job_status(&config, &job.id, "", true, Some(&error_msg)).await;
                break; // Exit the retry loop on non-timeout errors
            }
        }
    }
}

async fn fetch_pending_jobs(config: &Config) -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>> {
    make_request(config, "claude/jobs/pending", "POST", Some(serde_json::json!({}))).await
}

// Authentication functions
async fn register_monitor(config: &Config) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let hostname = hostname::get()?.to_string_lossy().to_string();

    // Try to get IP address using a simple method
    let ip_address = std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let system_info = serde_json::json!({
        "hostname": hostname,
        "ipAddress": ip_address,
        "location": None::<String>,
        "osType": std::env::consts::OS,
        "username": std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
    });

    let body = serde_json::json!({
        "telegramUsername": config.telegram_username,
        "agentId": config.agent_id,
        "systemInfo": system_info,
    });

    #[derive(Deserialize)]
    struct RegisterResponse {
        #[serde(rename = "requestId")]
        request_id: String,
        #[allow(dead_code)]
        status: String,
    }

    let response: RegisterResponse = make_request(config, "monitor/register", "POST", Some(body)).await?;
    Ok(response.request_id)
}

async fn check_auth_status(config: &Config, request_id: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct AuthResponse {
        status: String,
        #[serde(rename = "sessionId")]
        session_id: Option<String>,
    }

    let body = serde_json::json!({
        "requestId": request_id,
    });

    let response: AuthResponse = make_request(config, "monitor/check-auth", "POST", Some(body)).await?;

    match response.status.as_str() {
        "approved" => Ok(response.session_id),
        "rejected" => Err("Authentication rejected by user".into()),
        _ => Ok(None), // Still pending
    }
}

async fn fetch_bot_token(config: &Config) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct BotTokenResponse {
        #[serde(rename = "botToken")]
        bot_token: String,
        #[serde(rename = "agentId")]
        #[allow(dead_code)]
        agent_id: String,
    }

    let response: BotTokenResponse = make_request(config, "monitor/bot-token", "POST", Some(serde_json::json!({}))).await?;
    Ok(response.bot_token)
}

// Track running job task handle for interruption
struct RunningJob {
    job_id: String,
    chat_id: String,
    handle: tokio::task::JoinHandle<()>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Claude Monitor Starting...");
    println!();

    let mut config = Config::from_args(args);

    // Validate agent ID if not "default"
    if config.agent_id != "default" {
        let valid_ids: Vec<&str> = vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];
        if !valid_ids.contains(&config.agent_id.as_str()) {
            error(&format!("Invalid agent ID: '{}'. Must be 0-9, a-f, or 'default'", config.agent_id));
            std::process::exit(1);
        }
    }

    println!("Configuration:");
    println!("  Telegram User: @{}", config.telegram_username);
    println!("  Agent ID: {}", config.agent_id);
    println!("  Lamdera URL: {}", config.lamdera_url);
    println!("  Work Directory: {}", config.claude_work_dir);
    println!("  Poll Interval: {}ms", config.poll_interval);
    println!("  Aggregation Delay: {}ms", config.aggregation_delay);
    println!();

    // Acquire lock file to prevent multiple monitors with same agent in same directory
    let _lock = match LockFile::acquire(&config.claude_work_dir, &config.agent_id) {
        Ok(lock) => lock,
        Err(e) => {
            error(&format!("Failed to acquire lock: {}", e));
            std::process::exit(1);
        }
    };

    // Authenticate with the backend
    info("🔐 Authenticating with backend...");
    let request_id = match register_monitor(&config).await {
        Ok(id) => {
            info(&format!("📨 Authentication request sent. Request ID: {}", id));
            info("⏳ Waiting for approval on Telegram...");
            info(&format!("💬 Please check your Telegram (@{}) for the approval request", config.telegram_username));
            id
        }
        Err(e) => {
            error(&format!("Failed to register with backend: {}", e));
            std::process::exit(1);
        }
    };

    // Poll for authentication approval
    let mut auth_poll_interval = interval(Duration::from_secs(2));
    loop {
        auth_poll_interval.tick().await;

        match check_auth_status(&config, &request_id).await {
            Ok(Some(session_id)) => {
                info(&format!("✅ Authentication approved! Session ID: {}", session_id));
                config.session_id = Some(session_id);
                break;
            }
            Ok(None) => {
                // Still pending, continue waiting
            }
            Err(e) => {
                error(&format!("❌ Authentication failed: {}", e));
                std::process::exit(1);
            }
        }
    }

    // Fetch bot token from backend for this agent
    info("🔑 Fetching bot token for agent...");
    match fetch_bot_token(&config).await {
        Ok(token) => {
            info(&format!("✅ Bot token retrieved for agent '{}'", config.agent_id));
            config.bot_token = token;
        }
        Err(e) => {
            error(&format!("❌ Failed to fetch bot token: {}", e));
            std::process::exit(1);
        }
    }

    let config = Arc::new(config);
    println!();

    // Create TTS manager (lazy initialization - server starts on first voice request)
    let tts_manager: Arc<tokio::sync::Mutex<TtsManager>> = Arc::new(tokio::sync::Mutex::new(TtsManager::new()));

    let current_running_job: Arc<Mutex<Option<RunningJob>>> = Arc::new(Mutex::new(None));
    let mut was_disconnected = false;

    // Set up SIGINT handler
    tokio::spawn(async {
        if let Ok(_) = tokio::signal::ctrl_c().await {
            info("Shutting down monitor...");
            std::process::exit(0);
        }
    });

    // Main polling loop
    let mut poll_interval = interval(Duration::from_millis(config.poll_interval));

    loop {
        poll_interval.tick().await;

        // Fetch and process jobs
        match fetch_pending_jobs(&config).await {
            Ok(jobs) => {
                // If we were disconnected and now we're back, show reconnection message
                if was_disconnected {
                    info("✅ Connection to Lamdera backend re-established!");
                    was_disconnected = false;
                }

                if !jobs.is_empty() {
                let job = jobs.into_iter().next().unwrap();

                // Check if we need to interrupt existing job
                let should_process = {
                    let mut guard = current_running_job.lock().unwrap_or_else(|e| e.into_inner());

                    if let Some(ref mut running) = *guard {
                        // If it's the same job ID, skip
                        if running.job_id == job.id {
                            false
                        }
                        // If it's the same chat but different job, interrupt and continue
                        else if running.chat_id == job.chat_id {
                            info(&format!("Interrupting job {} for new prompt from same chat", running.job_id));
                            running.handle.abort();
                            // Send interrupted message
                            let _ = send_to_telegram(&config, &job.chat_id, "\n⚡ *Interrupted - continuing with new prompt...*\n").await;
                            true
                        }
                        // Different chat, let current job continue
                        else {
                            false
                        }
                    } else {
                        // No job running
                        true
                    }
                };

                if should_process {
                    info(&format!("Found pending job: {}", job.id));

                    let config_clone = config.clone();
                    let current_job_clone = current_running_job.clone();
                    let tts_manager_clone = tts_manager.clone();
                    let job_id = job.id.clone();
                    let chat_id = job.chat_id.clone();

                    let handle = tokio::spawn(async move {
                        process_job(config_clone, job, tts_manager_clone).await;
                        if let Ok(mut guard) = current_job_clone.lock() {
                            *guard = None;
                        }
                    });

                    // Store the running job
                    if let Ok(mut guard) = current_running_job.lock() {
                        *guard = Some(RunningJob {
                            job_id,
                            chat_id,
                            handle,
                        });
                    }
                }
            }
            }
            Err(e) => {
                let error_msg = format!("Failed to fetch jobs: {}", e);

                // Only show the disconnection message once, not every poll interval
                if !was_disconnected {
                    error(&format!("❌ {}", error_msg));
                    was_disconnected = true;
                } else {
                    // Just log quietly while disconnected (no spam)
                    // Could optionally show a dot or something to indicate we're still trying
                }
            }
        }
    }
}