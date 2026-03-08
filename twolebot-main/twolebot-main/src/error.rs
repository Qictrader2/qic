use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use thiserror::Error;

/// Central error type for twolebot
#[derive(Error, Debug)]
pub enum TwolebotError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Telegram API error: {message}")]
    TelegramApi { message: String },

    #[error("Gemini API error: {message}")]
    GeminiApi { message: String },

    #[error("Claude process error: {message}")]
    ClaudeProcess { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("Timeout: {message}")]
    Timeout { message: String },

    #[error("Not found: {message}")]
    NotFound { message: String },

    #[error("Cron error: {message}")]
    Cron { message: String },

    #[error("Work error: {message}")]
    Work { message: String },

    #[error("Semantic error: {0:#}")]
    Semantic(anyhow::Error),

    #[error("{0}")]
    Other(String),
}

impl TwolebotError {
    pub fn telegram(msg: impl Into<String>) -> Self {
        TwolebotError::TelegramApi {
            message: msg.into(),
        }
    }

    pub fn gemini(msg: impl Into<String>) -> Self {
        TwolebotError::GeminiApi {
            message: msg.into(),
        }
    }

    pub fn claude(msg: impl Into<String>) -> Self {
        TwolebotError::ClaudeProcess {
            message: msg.into(),
        }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        TwolebotError::Config {
            message: msg.into(),
        }
    }

    pub fn storage(msg: impl Into<String>) -> Self {
        TwolebotError::Storage {
            message: msg.into(),
        }
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        TwolebotError::Timeout {
            message: msg.into(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        TwolebotError::NotFound {
            message: msg.into(),
        }
    }

    pub fn cron(msg: impl Into<String>) -> Self {
        TwolebotError::Cron {
            message: msg.into(),
        }
    }

    pub fn work(msg: impl Into<String>) -> Self {
        TwolebotError::Work {
            message: msg.into(),
        }
    }

    pub fn semantic(err: anyhow::Error) -> Self {
        TwolebotError::Semantic(err)
    }

    pub fn other(msg: impl Into<String>) -> Self {
        TwolebotError::Other(msg.into())
    }
}

/// Result type alias using TwolebotError
pub type Result<T> = std::result::Result<T, TwolebotError>;

impl TwolebotError {
    /// Map this error to an appropriate HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            TwolebotError::NotFound { .. } => StatusCode::NOT_FOUND,
            TwolebotError::Cron { .. } => StatusCode::BAD_REQUEST,
            TwolebotError::Config { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for TwolebotError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = serde_json::json!({ "error": self.to_string() });
        (status, Json(body)).into_response()
    }
}
