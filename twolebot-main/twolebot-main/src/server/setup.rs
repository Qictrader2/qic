use crate::config::{default_data_dir, Args, SetupStatus};
use crate::storage::{MessageStore, SecretsStore, SettingsStore};
use crate::types::api::setup::{
    ApiKeyStatus, ApiKeysResponse, ClaudeAuthCheckResponse, ClaudeCodeStatus,
    ClaudeInstallResponse, ClaudeTestResponse, GeminiSetupRequest, GeminiSetupResponse,
    SetupCompleteResponse, TelegramSetupRequest, TelegramSetupResponse, ThreadingCheckResponse,
    UpdateApiKeysRequest, UpdateApiKeysResponse,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// Shared setup state
#[derive(Clone)]
pub struct SetupState {
    pub args: Arc<Args>,
    pub runtime_db_path: std::path::PathBuf,
}

impl SetupState {
    pub fn new(args: Args) -> Self {
        let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
        let runtime_db_path = data_dir.join("runtime.sqlite3");
        Self {
            args: Arc::new(args),
            runtime_db_path,
        }
    }
}

// ============ GET /api/setup/status ============

/// GET /api/setup/status - Get current setup status
pub async fn get_setup_status(State(state): State<SetupState>) -> impl IntoResponse {
    let status = SetupStatus::check(&state.args);
    Json(status)
}

// ============ POST /api/setup/telegram ============

/// POST /api/setup/telegram - Test and save Telegram token
pub async fn setup_telegram(
    State(state): State<SetupState>,
    Json(request): Json<TelegramSetupRequest>,
) -> impl IntoResponse {
    // Test the token by calling getMe
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/getMe", request.token);

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                // Parse response to get bot name
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let bot_name = data["result"]["username"].as_str().map(|s| s.to_string());

                        // Save to runtime DB
                        let store = match SecretsStore::new(&state.runtime_db_path) {
                            Ok(s) => s,
                            Err(e) => {
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(TelegramSetupResponse {
                                        success: false,
                                        bot_name: None,
                                        error: Some(format!("Failed to open secrets store: {}", e)),
                                    }),
                                );
                            }
                        };
                        if let Some(ref name) = bot_name {
                            let _ = store.set_telegram_bot_name(name.clone());
                        }
                        if let Err(e) = store.set_telegram_token(request.token) {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(TelegramSetupResponse {
                                    success: false,
                                    bot_name: None,
                                    error: Some(format!("Failed to save token: {}", e)),
                                }),
                            );
                        }

                        (
                            StatusCode::OK,
                            Json(TelegramSetupResponse {
                                success: true,
                                bot_name,
                                error: None,
                            }),
                        )
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(TelegramSetupResponse {
                            success: false,
                            bot_name: None,
                            error: Some(format!("Failed to parse Telegram response: {}", e)),
                        }),
                    ),
                }
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(TelegramSetupResponse {
                        success: false,
                        bot_name: None,
                        error: Some(format!("Invalid token (HTTP {})", status.as_u16())),
                    }),
                )
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(TelegramSetupResponse {
                success: false,
                bot_name: None,
                error: Some(format!("Connection error: {}", e)),
            }),
        ),
    }
}

// ============ POST /api/setup/gemini ============

/// POST /api/setup/gemini - Test and save Gemini API key
pub async fn setup_gemini(
    State(state): State<SetupState>,
    Json(request): Json<GeminiSetupRequest>,
) -> impl IntoResponse {
    // Test the key with a minimal request
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        request.key
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                // Save to runtime DB
                let store = match SecretsStore::new(&state.runtime_db_path) {
                    Ok(s) => s,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(GeminiSetupResponse {
                                success: false,
                                error: Some(format!("Failed to open secrets store: {}", e)),
                            }),
                        );
                    }
                };
                if let Err(e) = store.set_gemini_key(request.key) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(GeminiSetupResponse {
                            success: false,
                            error: Some(format!("Failed to save key: {}", e)),
                        }),
                    );
                }

                return (
                    StatusCode::OK,
                    Json(GeminiSetupResponse {
                        success: true,
                        error: None,
                    }),
                );
            }

            (
                StatusCode::BAD_REQUEST,
                Json(GeminiSetupResponse {
                    success: false,
                    error: Some("Invalid API key".to_string()),
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(GeminiSetupResponse {
                success: false,
                error: Some(format!("Connection error: {}", e)),
            }),
        ),
    }
}

// ============ POST /api/setup/install-claude ============

fn run_command_with_local_fallback(
    binary: &str,
    args: &[&str],
) -> std::io::Result<std::process::Output> {
    use std::io::ErrorKind;
    use std::process::Command;

    match Command::new(binary).args(args).output() {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => return Err(e),
            };
            let local_bin = home.join(".local").join("bin").join(binary);
            if !local_bin.exists() {
                return Err(e);
            }
            Command::new(local_bin).args(args).output()
        }
        result => result,
    }
}

fn claude_version_string() -> Option<String> {
    let version_check = run_command_with_local_fallback("claude", &["--version"]).ok()?;
    if !version_check.status.success() {
        return None;
    }
    String::from_utf8(version_check.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

fn install_claude_with_npm(package: &str) -> Result<(), String> {
    let mut errors = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let prefix = home.join(".local");
        let prefix_str = prefix.to_string_lossy().into_owned();
        match run_command_with_local_fallback(
            "npm",
            &["install", "-g", "--prefix", &prefix_str, package],
        ) {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => errors.push(format!(
                "user-local install failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )),
            Err(e) => errors.push(format!("user-local install failed to start: {}", e)),
        }
    }

    match run_command_with_local_fallback("npm", &["install", "-g", package]) {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            errors.push(format!(
                "global install failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
            Err(errors.join(" | "))
        }
        Err(e) => {
            errors.push(format!("global install failed to start: {}", e));
            Err(errors.join(" | "))
        }
    }
}

/// POST /api/setup/install-claude - Install Claude CLI via npm
pub async fn setup_install_claude() -> impl IntoResponse {
    // First check if npm is available
    let npm_check = run_command_with_local_fallback("npm", &["--version"]);

    if npm_check.is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ClaudeInstallResponse {
                success: false,
                version: None,
                error: Some(
                    "npm not found. Please install Node.js first: https://nodejs.org/".to_string(),
                ),
            }),
        );
    }

    // Install Claude CLI (prefer user-local prefix, then fallback)
    tracing::info!("Installing Claude CLI via npm...");
    let install_result = install_claude_with_npm("@anthropic-ai/claude-code");

    match install_result {
        Ok(()) => {
            let version = claude_version_string();
            tracing::info!("Claude CLI installed successfully: {:?}", version);
            (
                StatusCode::OK,
                Json(ClaudeInstallResponse {
                    success: true,
                    version,
                    error: None,
                }),
            )
        }
        Err(err) => {
            tracing::error!("Failed to install Claude CLI: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClaudeInstallResponse {
                    success: false,
                    version: None,
                    error: Some(format!("npm install failed: {}", err)),
                }),
            )
        }
    }
}

// ============ GET /api/setup/claude-auth ============

/// GET /api/setup/claude-auth - Check Claude CLI install, auth, and update status
pub async fn check_claude_auth() -> impl IntoResponse {
    // Check if claude is installed
    let version_output = run_command_with_local_fallback("claude", &["--version"]);
    let (installed, version) = match version_output {
        Ok(output) if output.status.success() => {
            let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(v))
        }
        _ => {
            return Json(ClaudeAuthCheckResponse {
                installed: false,
                version: None,
                authenticated: false,
                auth_mode: None,
                account_email: None,
                account_name: None,
                needs_update: false,
                latest_version: None,
                error: None,
            });
        }
    };

    // Check authentication via ~/.claude.json
    let (authenticated, auth_mode, account_email, account_name) = check_claude_auth_status();

    // Check for updates via npm
    let (needs_update, latest_version) = check_claude_update(&version);

    Json(ClaudeAuthCheckResponse {
        installed,
        version,
        authenticated,
        auth_mode,
        account_email,
        account_name,
        needs_update,
        latest_version,
        error: None,
    })
}

fn check_claude_auth_status() -> (bool, Option<String>, Option<String>, Option<String>) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return (false, None, None, None),
    };
    let claude_config_path = home.join(".claude.json");

    let content = match std::fs::read_to_string(&claude_config_path) {
        Ok(c) => c,
        Err(_) => return (false, None, None, None),
    };
    let config: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (false, None, None, None),
    };

    // Check OAuth
    if let Some(oauth) = config.get("oauthAccount") {
        let email = oauth
            .get("emailAddress")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let name = oauth
            .get("displayName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        return (true, Some("oauth".to_string()), email, name);
    }

    // Check API key
    if let Some(api_keys) = config.get("customApiKeyResponses") {
        if let Some(obj) = api_keys.as_object() {
            if !obj.is_empty() {
                return (true, Some("api_key".to_string()), None, None);
            }
        }
    }

    (false, None, None, None)
}

fn check_claude_update(current_version: &Option<String>) -> (bool, Option<String>) {
    let current = match current_version {
        Some(v) => v,
        None => return (false, None),
    };

    // npm view @anthropic-ai/claude-code version
    let output = run_command_with_local_fallback("npm", &["view", "@anthropic-ai/claude-code", "version"]);

    match output {
        Ok(o) if o.status.success() => {
            let latest = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // Extract just version numbers for comparison (strip any prefix like "claude-code ")
            let current_clean = current
                .split_whitespace()
                .find(|s| s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                .unwrap_or(current);
            let needs_update = !latest.is_empty() && latest != current_clean;
            (needs_update, Some(latest))
        }
        _ => (false, None),
    }
}

// ============ POST /api/setup/update-claude ============

/// POST /api/setup/update-claude - Update Claude CLI to latest version
pub async fn setup_update_claude() -> impl IntoResponse {
    tracing::info!("Updating Claude CLI via npm...");
    let result = install_claude_with_npm("@anthropic-ai/claude-code@latest");

    match result {
        Ok(()) => {
            let version = claude_version_string();
            tracing::info!("Claude CLI updated: {:?}", version);
            (
                StatusCode::OK,
                Json(ClaudeInstallResponse {
                    success: true,
                    version,
                    error: None,
                }),
            )
        }
        Err(err) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClaudeInstallResponse {
                    success: false,
                    version: None,
                    error: Some(format!("Update failed: {}", err)),
                }),
            )
        }
    }
}

// ============ POST /api/setup/test-claude ============

/// POST /api/setup/test-claude - Test Claude CLI by running a simple prompt
pub async fn setup_test_claude() -> impl IntoResponse {
    let result = run_command_with_local_fallback("claude", &["-p", "Say exactly: Hello from Claude!"]);

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (
                StatusCode::OK,
                Json(ClaudeTestResponse {
                    success: true,
                    output: Some(stdout),
                    error: None,
                }),
            )
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let error_msg = if !stderr.is_empty() {
                stderr
            } else {
                stdout
            };
            (
                StatusCode::OK,
                Json(ClaudeTestResponse {
                    success: false,
                    output: None,
                    error: Some(error_msg),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ClaudeTestResponse {
                success: false,
                output: None,
                error: Some(format!("Failed to run claude: {}", e)),
            }),
        ),
    }
}

// ============ POST /api/setup/complete ============

/// POST /api/setup/complete - Mark setup as complete and restart
pub async fn setup_complete(State(state): State<SetupState>) -> impl IntoResponse {
    let status = SetupStatus::check(&state.args);

    if status.is_complete {
        Json(SetupCompleteResponse {
            success: true,
            message: "Setup complete! Restart twolebot to begin.".to_string(),
        })
    } else {
        Json(SetupCompleteResponse {
            success: false,
            message: "Setup incomplete. Please configure all required settings.".to_string(),
        })
    }
}

// ============ POST /api/setup/check-threading ============

/// POST /api/setup/check-threading - Check if threading is enabled via Telegram getMe
pub async fn check_threading(State(state): State<SetupState>) -> impl IntoResponse {
    // Get saved token
    let token = match SecretsStore::new(&state.runtime_db_path)
        .ok()
        .and_then(|s| s.get_telegram_token().ok().flatten())
    {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ThreadingCheckResponse {
                    success: false,
                    enabled: false,
                    error: Some("No Telegram token configured yet.".to_string()),
                }),
            );
        }
    };

    // Call getMe to check has_topics_enabled.
    // Note: token is embedded in the URL per Telegram's API convention.
    // Do not log this URL — it contains the bot secret token.
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/getMe", token);

    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    let enabled = data["result"]["has_topics_enabled"]
                        .as_bool()
                        .unwrap_or(false);

                    if enabled {
                        // Persist to settings
                        if let Ok(store) = SettingsStore::new(&state.runtime_db_path) {
                            if let Err(e) = store.set_threading_enabled(true) {
                                tracing::warn!("Failed to persist threading setting: {}", e);
                            }
                        }
                    }

                    (
                        StatusCode::OK,
                        Json(ThreadingCheckResponse {
                            success: true,
                            enabled,
                            error: None,
                        }),
                    )
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ThreadingCheckResponse {
                        success: false,
                        enabled: false,
                        error: Some(format!("Failed to parse Telegram response: {}", e)),
                    }),
                ),
            }
        }
        Ok(response) => (
            StatusCode::BAD_REQUEST,
            Json(ThreadingCheckResponse {
                success: false,
                enabled: false,
                error: Some(format!("Telegram API error (HTTP {})", response.status().as_u16())),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ThreadingCheckResponse {
                success: false,
                enabled: false,
                error: Some(format!("Connection error: {}", e)),
            }),
        ),
    }
}

// ============ GET /api/settings/api-keys ============

fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        "*".repeat(token.len())
    } else {
        format!("{}...{}", &token[..4], &token[token.len() - 4..])
    }
}

/// GET /api/settings/api-keys - Get masked API keys with liveness check
pub async fn get_api_keys(State(state): State<SetupState>) -> impl IntoResponse {
    let store = SecretsStore::new(&state.runtime_db_path).ok();
    let client = reqwest::Client::new();

    // Prefer runtime DB values so UI reflects keys saved via setup/settings.
    // Fall back to CLI args for one-off runs that don't persist keys.
    let telegram_token = resolve_key(
        state.args.telegram_token.clone(),
        store
            .as_ref()
            .and_then(|s| s.get_telegram_token().ok())
            .flatten(),
    );
    let gemini_key = resolve_key(
        state.args.gemini_key.clone(),
        store.as_ref().and_then(|s| s.get_gemini_key().ok()).flatten(),
    );

    // Check Telegram token liveness
    let telegram_status = if let Some(ref token) = telegram_token {
        let url = format!("https://api.telegram.org/bot{}/getMe", token);
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let bot_name = data["result"]["username"].as_str().unwrap_or("unknown");
                        let can_read = data["result"]["can_read_all_group_messages"]
                            .as_bool()
                            .unwrap_or(false);
                        Some(ApiKeyStatus {
                            valid: true,
                            error: None,
                            info: Some(format!(
                                "@{}{}",
                                bot_name,
                                if can_read { " (group access)" } else { "" }
                            )),
                        })
                    }
                    Err(_) => Some(ApiKeyStatus {
                        valid: true,
                        error: None,
                        info: Some("Connected".to_string()),
                    }),
                }
            }
            Ok(response) => Some(ApiKeyStatus {
                valid: false,
                error: Some(format!("HTTP {}", response.status().as_u16())),
                info: None,
            }),
            Err(e) => Some(ApiKeyStatus {
                valid: false,
                error: Some(format!("Connection error: {}", e)),
                info: None,
            }),
        }
    } else {
        None
    };

    // Check Gemini key liveness
    let gemini_status = if let Some(ref key) = gemini_key {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            key
        );
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        // Count available models as a rough indicator
                        let model_count = data["models"].as_array().map(|m| m.len()).unwrap_or(0);
                        Some(ApiKeyStatus {
                            valid: true,
                            error: None,
                            info: Some(format!("{} models available", model_count)),
                        })
                    }
                    Err(_) => Some(ApiKeyStatus {
                        valid: true,
                        error: None,
                        info: Some("Connected".to_string()),
                    }),
                }
            }
            Ok(response) => {
                let status = response.status();
                let error_msg = if status.as_u16() == 400 {
                    "Invalid API key".to_string()
                } else if status.as_u16() == 403 {
                    "API key disabled or quota exceeded".to_string()
                } else {
                    format!("HTTP {}", status.as_u16())
                };
                Some(ApiKeyStatus {
                    valid: false,
                    error: Some(error_msg),
                    info: None,
                })
            }
            Err(e) => Some(ApiKeyStatus {
                valid: false,
                error: Some(format!("Connection error: {}", e)),
                info: None,
            }),
        }
    } else {
        None
    };

    // Check Claude Code status from ~/.claude.json
    let claude_code_status = get_claude_code_status();

    // Check if any user has contacted the bot (has inbound messages)
    let has_user_contacted = SettingsStore::new(&state.runtime_db_path)
        .ok()
        .and_then(|settings| {
            if settings.get().allowed_username.is_some() {
                MessageStore::new(&state.runtime_db_path)
                    .ok()
                    .map(|msg_store| msg_store.has_inbound_messages().unwrap_or(false))
            } else {
                None
            }
        });

    Json(ApiKeysResponse {
        has_telegram_token: telegram_token.is_some(),
        telegram_token_masked: telegram_token.as_ref().map(|t| mask_token(t)),
        telegram_status,
        has_gemini_key: gemini_key.is_some(),
        gemini_key_masked: gemini_key.as_ref().map(|k| mask_token(k)),
        gemini_status,
        claude_code_status,
        has_user_contacted,
    })
}

fn resolve_key(cli_value: Option<String>, db_value: Option<String>) -> Option<String> {
    db_value
        .filter(|v| !v.trim().is_empty())
        .or(cli_value.filter(|v| !v.trim().is_empty()))
}

fn get_claude_code_status() -> Option<ClaudeCodeStatus> {
    let home = dirs::home_dir()?;
    let claude_config_path = home.join(".claude.json");

    let content = std::fs::read_to_string(&claude_config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Check if using OAuth account
    if let Some(oauth) = config.get("oauthAccount") {
        let email = oauth
            .get("emailAddress")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let name = oauth
            .get("displayName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let org = oauth
            .get("organizationName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        return Some(ClaudeCodeStatus {
            auth_mode: "oauth".to_string(),
            account_email: email,
            account_name: name,
            organization: org,
        });
    }

    // Check if using API key (customApiKeyResponses exists and has entries)
    if let Some(api_keys) = config.get("customApiKeyResponses") {
        if let Some(obj) = api_keys.as_object() {
            if !obj.is_empty() {
                return Some(ClaudeCodeStatus {
                    auth_mode: "api_key".to_string(),
                    account_email: None,
                    account_name: None,
                    organization: None,
                });
            }
        }
    }

    None
}

// ============ PUT /api/settings/api-keys ============

/// PUT /api/settings/api-keys - Update API keys (validates before saving)
pub async fn update_api_keys(
    State(state): State<SetupState>,
    Json(request): Json<UpdateApiKeysRequest>,
) -> impl IntoResponse {
    let store = match SecretsStore::new(&state.runtime_db_path) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateApiKeysResponse {
                    success: false,
                    telegram_updated: false,
                    gemini_updated: false,
                    telegram_error: Some(format!("Failed to open secrets store: {}", e)),
                    gemini_error: None,
                }),
            );
        }
    };
    let client = reqwest::Client::new();

    let mut telegram_updated = false;
    let mut gemini_updated = false;
    let mut telegram_error: Option<String> = None;
    let mut gemini_error: Option<String> = None;

    // Validate and update Telegram token if provided
    if let Some(token) = &request.telegram_token {
        if !token.is_empty() {
            let url = format!("https://api.telegram.org/bot{}/getMe", token);
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    match store.set_telegram_token(token.clone()) {
                        Ok(()) => telegram_updated = true,
                        Err(e) => {
                            telegram_error = Some(format!("Failed to save token: {}", e));
                        }
                    }
                }
                Ok(response) => {
                    telegram_error = Some(format!(
                        "Invalid token (HTTP {})",
                        response.status().as_u16()
                    ));
                }
                Err(e) => {
                    telegram_error = Some(format!("Connection error: {}", e));
                }
            }
        }
    }

    // Validate and update Gemini key if provided
    if let Some(key) = &request.gemini_key {
        if !key.is_empty() {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                key
            );
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    match store.set_gemini_key(key.clone()) {
                        Ok(()) => gemini_updated = true,
                        Err(e) => {
                            gemini_error = Some(format!("Failed to save key: {}", e));
                        }
                    }
                }
                Ok(_) => {
                    gemini_error = Some("Invalid API key".to_string());
                }
                Err(e) => {
                    gemini_error = Some(format!("Connection error: {}", e));
                }
            }
        }
    }

    let success = (request.telegram_token.is_none() || telegram_updated)
        && (request.gemini_key.is_none() || gemini_updated);

    (
        if success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        },
        Json(UpdateApiKeysResponse {
            success,
            telegram_updated,
            gemini_updated,
            telegram_error,
            gemini_error,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::resolve_key;

    #[test]
    fn resolve_key_prefers_db_over_cli() {
        let resolved = resolve_key(Some("dummy-cli-token".to_string()), Some("real-db-token".to_string()));
        assert_eq!(resolved.as_deref(), Some("real-db-token"));
    }

    #[test]
    fn resolve_key_falls_back_to_cli_when_db_missing() {
        let resolved = resolve_key(Some("cli-token".to_string()), None);
        assert_eq!(resolved.as_deref(), Some("cli-token"));
    }

    #[test]
    fn resolve_key_ignores_empty_values() {
        let resolved = resolve_key(Some("".to_string()), Some("   ".to_string()));
        assert_eq!(resolved, None);
    }
}
