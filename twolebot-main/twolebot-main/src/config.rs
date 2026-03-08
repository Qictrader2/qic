use crate::storage::{SecretsStore, SettingsStore};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Telegram bot with Claude integration
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Telegram bot token
    #[arg(long, global = true)]
    pub telegram_token: Option<String>,

    /// Gemini API key for transcription (optional - required only for voice/media)
    #[arg(long, global = true)]
    pub gemini_key: Option<String>,

    /// Data directory override (default: XDG data dir)
    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,

    /// Memory directory for MCP memory storage (default: {data-dir}/memory)
    #[arg(long, global = true)]
    pub memory_dir: Option<PathBuf>,

    /// Host address to bind to (default: 127.0.0.1 for security)
    #[arg(long, default_value = "127.0.0.1", global = true)]
    pub host: String,

    /// HTTP server port
    #[arg(short, long, default_value = "8080", global = true)]
    pub port: u16,

    /// Allow CORS from any origin (default: true — needed for tunnels/external access)
    #[arg(long, default_value = "true", global = true)]
    pub cors_allow_all: bool,

    /// Claude model to use
    #[arg(long, default_value = "claude-opus-4-6", global = true)]
    pub claude_model: String,

    /// Process timeout in milliseconds (default: 6 hours)
    #[arg(long, default_value = "21600000", global = true)]
    pub process_timeout_ms: u64,

    /// Typing indicator interval in seconds
    #[arg(long, default_value = "4", global = true)]
    pub typing_interval_secs: u64,

    /// Cron idle threshold in seconds before promoting jobs (default: 10 minutes)
    #[arg(long, default_value = "600", global = true)]
    pub cron_idle_threshold_secs: i64,

    /// Disable semantic search (vector embeddings for memory/conversation search)
    #[arg(long, default_value = "false", global = true)]
    pub disable_semantic: bool,

    /// Disable Cloudflare quick tunnel (tunnel is on by default)
    #[arg(long, default_value = "false", global = true)]
    pub no_tunnel: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Run the main Telegram bot server (default)
    Run,
    /// Show current configuration paths and status
    Status,
    /// Run as a stdio MCP server (for Claude CLI spawning)
    McpStdio,
}

// ============ XDG Directory Functions ============

/// Get the XDG data directory for twolebot
/// macOS: ~/Library/Application Support/twolebot
/// Linux: ~/.local/share/twolebot
pub fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("twolebot")
}


// ============ Runtime Config ============

/// Runtime configuration derived from CLI args + config file
#[derive(Debug, Clone)]
pub struct Config {
    pub telegram_token: Option<String>,
    pub gemini_key: Option<String>,
    pub host: String,
    pub port: u16,
    pub cors_allow_all: bool,
    pub claude_model: String,
    pub process_timeout_ms: u64,
    pub typing_interval_secs: u64,
    pub cron_idle_threshold_secs: i64,
    pub semantic_enabled: bool,

    // Paths
    pub data_dir: PathBuf,

    // Derived data paths
    pub general_db_path: PathBuf,
    pub media_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub logs_file: PathBuf,
    pub frontend_dir: PathBuf,

}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    SetupRequired,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
            ConfigError::SetupRequired => write!(
                f,
                "Setup required. Run 'twolebot' and visit http://localhost:8080/setup"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Create config for the main server (requires telegram_token)
    /// Merges CLI args > SQLite runtime secrets for API keys.
    /// Other settings still use CLI > config file.
    pub fn from_args(args: &Args) -> Result<Self, ConfigError> {
        let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
        let general_db_path = data_dir.join("runtime.sqlite3");

        let secret_store =
            SecretsStore::new(&general_db_path).map_err(|e| ConfigError::Parse(e.to_string()))?;

        // API keys: CLI > runtime DB
        let telegram_token = args
            .telegram_token
            .clone()
            .or(secret_store
                .get_telegram_token()
                .map_err(|e| ConfigError::Parse(e.to_string()))?)
            .filter(|t| !t.is_empty() && t != "disabled");

        // Gemini key is optional - only needed for voice/media transcription
        let gemini_key = args.gemini_key.clone().or(secret_store
            .get_gemini_key()
            .map_err(|e| ConfigError::Parse(e.to_string()))?);

        // Semantic search: enabled by default, can be disabled with --disable-semantic
        let semantic_enabled = !args.disable_semantic;

        // Create directory structure
        let media_dir = data_dir.join("media");
        let memory_dir = args
            .memory_dir
            .clone()
            .unwrap_or_else(|| data_dir.join("memory"));
        let frontend_dir = data_dir.join("frontend").join("dist");
        let logs_file = data_dir.join("logs.jsonl");

        // Create all directories
        std::fs::create_dir_all(&data_dir).map_err(ConfigError::Io)?;
        std::fs::create_dir_all(&media_dir).map_err(ConfigError::Io)?;
        std::fs::create_dir_all(&memory_dir).map_err(ConfigError::Io)?;
        std::fs::create_dir_all(&frontend_dir).map_err(ConfigError::Io)?;

        // Write agent instructions to data/topics/CLAUDE.md (compiled into binary).
        // Every topic subfolder inherits these via Claude's ancestor directory loading.
        // Only writes if missing — won't overwrite user edits.
        let topics_dir = data_dir.join("topics");
        std::fs::create_dir_all(&topics_dir).map_err(ConfigError::Io)?;
        let topics_claude_md = topics_dir.join("CLAUDE.md");
        if !topics_claude_md.exists() {
            std::fs::write(&topics_claude_md, crate::claude::TOPICS_CLAUDE_MD)
                .map_err(ConfigError::Io)?;
        }

        Ok(Self {
            telegram_token,
            gemini_key,
            host: args.host.clone(),
            port: args.port,
            cors_allow_all: args.cors_allow_all,
            claude_model: args.claude_model.clone(),
            process_timeout_ms: args.process_timeout_ms,
            typing_interval_secs: args.typing_interval_secs,
            cron_idle_threshold_secs: args.cron_idle_threshold_secs,
            semantic_enabled,
            data_dir,
            general_db_path,
            media_dir,
            memory_dir,
            logs_file,
            frontend_dir,
        })
    }

    /// Create config for setup mode (partial config, no tokens required)
    pub fn for_setup(args: &Args) -> Result<Self, ConfigError> {
        let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
        let general_db_path = data_dir.join("runtime.sqlite3");
        let memory_dir = args
            .memory_dir
            .clone()
            .unwrap_or_else(|| data_dir.join("memory"));

        // Ensure directories exist
        std::fs::create_dir_all(&data_dir).map_err(ConfigError::Io)?;
        std::fs::create_dir_all(&memory_dir).map_err(ConfigError::Io)?;

        Ok(Self {
            telegram_token: None,
            gemini_key: None,
            host: args.host.clone(),
            port: args.port,
            cors_allow_all: args.cors_allow_all,
            claude_model: args.claude_model.clone(),
            process_timeout_ms: args.process_timeout_ms,
            typing_interval_secs: args.typing_interval_secs,
            cron_idle_threshold_secs: args.cron_idle_threshold_secs,
            semantic_enabled: !args.disable_semantic,
            data_dir: data_dir.clone(),
            general_db_path,
            media_dir: data_dir.join("media"),
            memory_dir,
            logs_file: data_dir.join("logs.jsonl"),
            frontend_dir: data_dir.join("frontend").join("dist"),
        })
    }

    /// Check if setup is needed (only on truly fresh installs — no secrets at all)
    pub fn needs_setup(args: &Args) -> bool {
        // If any token is provided via CLI, skip setup
        if args.telegram_token.is_some() {
            return false;
        }

        let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
        let runtime_db = data_dir.join("runtime.sqlite3");

        // Setup is needed only if the DB doesn't exist or has no secrets at all
        // (Telegram is optional — web-only instances are fine)
        match SecretsStore::new(&runtime_db) {
            Ok(store) => {
                let has_telegram = store.get_telegram_token().ok().flatten().is_some();
                let has_auth = store.get_auth_token().ok().flatten().is_some();
                // If we have either token, setup is done
                !has_telegram && !has_auth
            }
            Err(_) => true,
        }
    }
}

// ============ Setup Status ============

/// Setup status for the onboarding page
#[derive(Debug, Clone, Serialize)]
pub struct SetupStatus {
    pub data_dir: String,
    pub has_telegram_token: bool,
    pub has_gemini_key: bool,
    pub has_claude_cli: bool,
    pub claude_cli_version: Option<String>,
    pub has_allowed_username: bool,
    pub has_threading_enabled: bool,
    pub is_complete: bool,
    pub platform: String,
    pub gemini_key_preview: Option<String>,
    pub allowed_username_value: Option<String>,
    pub bot_name: Option<String>,
}

impl SetupStatus {
    pub fn check(args: &Args) -> Self {
        let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
        let runtime_db = data_dir.join("runtime.sqlite3");

        let (db_has_telegram, db_has_gemini, db_gemini_preview, db_bot_name) = match SecretsStore::new(&runtime_db) {
            Ok(store) => {
                let has_telegram = store
                    .get_telegram_token()
                    .ok()
                    .flatten()
                    .map(|t| !t.is_empty())
                    .unwrap_or(false);
                let gemini_key = store.get_gemini_key().ok().flatten().unwrap_or_default();
                let has_gemini = !gemini_key.is_empty();
                let gemini_preview = if gemini_key.len() >= 8 {
                    Some(format!(
                        "{}...{}",
                        &gemini_key[..4],
                        &gemini_key[gemini_key.len() - 4..]
                    ))
                } else if !gemini_key.is_empty() {
                    Some(gemini_key.clone())
                } else {
                    None
                };
                let bot_name = store.get_telegram_bot_name().ok().flatten();
                (has_telegram, has_gemini, gemini_preview, bot_name)
            }
            Err(_) => (false, false, None, None),
        };

        let has_telegram_token = args.telegram_token.is_some() || db_has_telegram;
        let has_gemini_key = args.gemini_key.is_some() || db_has_gemini;
        let gemini_key_preview = db_gemini_preview;
        let bot_name = db_bot_name;

        // Check Claude CLI
        let (has_claude_cli, claude_cli_version) = check_claude_cli();

        // Check settings
        let settings = SettingsStore::new(&runtime_db).ok().map(|s| s.get());
        let has_allowed_username = settings
            .as_ref()
            .map(|s| s.allowed_username.is_some())
            .unwrap_or(false);
        let allowed_username_value = settings
            .as_ref()
            .and_then(|s| s.allowed_username.clone());
        let has_threading_enabled = settings
            .as_ref()
            .map(|s| s.threading_enabled)
            .unwrap_or(false);

        // claude_cli, allowed_username, and threading are required; telegram is optional
        let is_complete =
            has_claude_cli && has_allowed_username && has_threading_enabled;

        let platform = if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        };

        Self {
            data_dir: data_dir.display().to_string(),
            has_telegram_token,
            has_gemini_key,
            has_claude_cli,
            claude_cli_version,
            has_allowed_username,
            has_threading_enabled,
            is_complete,
            platform: platform.to_string(),
            gemini_key_preview,
            allowed_username_value,
            bot_name,
        }
    }
}

/// Check if Claude CLI is installed and get version
fn check_claude_cli() -> (bool, Option<String>) {
    match std::process::Command::new("claude")
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(version))
        }
        _ => (false, None),
    }
}
