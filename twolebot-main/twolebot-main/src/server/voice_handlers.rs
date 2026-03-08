use crate::storage::SecretsStore;
use crate::transcription::gemini::GeminiTranscriber;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Shared state for voice endpoints
#[derive(Clone)]
pub struct VoiceState {
    pub data_dir: PathBuf,
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct TranscribeRequest {
    /// Base64-encoded audio data
    pub audio_data: String,
    /// MIME type of the audio (e.g., "audio/ogg", "audio/webm")
    #[serde(default = "default_mime_type")]
    pub mime_type: String,
}

fn default_mime_type() -> String {
    "audio/webm".to_string()
}

#[derive(Debug, Serialize)]
pub struct TranscribeResponse {
    pub success: bool,
    pub transcription: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FormatRequest {
    /// Raw transcription text
    pub transcription: String,
    /// Format mode: "ticket", "edit", or "comment"
    pub mode: String,
    /// Existing content (for "edit" mode)
    #[serde(default)]
    pub existing_content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FormatResponse {
    pub success: bool,
    pub formatted: Option<String>,
    /// Extracted title (for ticket mode)
    pub title: Option<String>,
    pub error: Option<String>,
}

// ============ Handlers ============

/// POST /api/voice/transcribe — accepts base64 audio, returns transcription
pub async fn transcribe_audio(
    State(state): State<VoiceState>,
    Json(request): Json<TranscribeRequest>,
) -> impl IntoResponse {
    let gemini_key = match get_gemini_key(&state.data_dir) {
        Ok(key) => key,
        Err(msg) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TranscribeResponse {
                    success: false,
                    transcription: None,
                    error: Some(msg),
                }),
            )
        }
    };

    // Reject obviously oversized payloads before decoding (base64 is ~33% larger than raw)
    const MAX_AUDIO_BYTES: usize = 25 * 1024 * 1024; // 25 MB
    if request.audio_data.len() > MAX_AUDIO_BYTES * 4 / 3 + 4 {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(TranscribeResponse {
                success: false,
                transcription: None,
                error: Some(format!(
                    "Audio data too large. Maximum size is {} MB.",
                    MAX_AUDIO_BYTES / 1024 / 1024
                )),
            }),
        );
    }

    // Decode base64 audio
    let audio_bytes = match BASE64.decode(&request.audio_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(TranscribeResponse {
                    success: false,
                    transcription: None,
                    error: Some(format!("Invalid base64 audio data: {}", e)),
                }),
            )
        }
    };

    if audio_bytes.len() > MAX_AUDIO_BYTES {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(TranscribeResponse {
                success: false,
                transcription: None,
                error: Some(format!(
                    "Audio data too large ({:.1} MB). Maximum size is {} MB.",
                    audio_bytes.len() as f64 / 1024.0 / 1024.0,
                    MAX_AUDIO_BYTES / 1024 / 1024
                )),
            }),
        );
    }

    let transcriber = match GeminiTranscriber::new(gemini_key) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TranscribeResponse {
                    success: false,
                    transcription: None,
                    error: Some(format!("Failed to create transcriber: {}", e)),
                }),
            )
        }
    };

    match transcriber
        .transcribe_with_mime(&audio_bytes, "voice", &request.mime_type)
        .await
    {
        Ok(text) => (
            StatusCode::OK,
            Json(TranscribeResponse {
                success: true,
                transcription: Some(text),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TranscribeResponse {
                success: false,
                transcription: None,
                error: Some(format!("Transcription failed: {}", e)),
            }),
        ),
    }
}

const VALID_FORMAT_MODES: &[&str] = &["ticket", "edit", "comment"];

/// POST /api/voice/format — accepts transcription + context, returns formatted MD
pub async fn format_transcription(
    State(state): State<VoiceState>,
    Json(request): Json<FormatRequest>,
) -> impl IntoResponse {
    if !VALID_FORMAT_MODES.contains(&request.mode.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(FormatResponse {
                success: false,
                formatted: None,
                title: None,
                error: Some(format!(
                    "Invalid format mode '{}'. Must be one of: {}",
                    request.mode,
                    VALID_FORMAT_MODES.join(", ")
                )),
            }),
        );
    }

    let gemini_key = match get_gemini_key(&state.data_dir) {
        Ok(key) => key,
        Err(msg) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FormatResponse {
                    success: false,
                    formatted: None,
                    title: None,
                    error: Some(msg),
                }),
            )
        }
    };

    let prompt = build_format_prompt(&request);

    let transcriber = match GeminiTranscriber::new(gemini_key) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FormatResponse {
                    success: false,
                    formatted: None,
                    title: None,
                    error: Some(format!("Failed to create transcriber: {}", e)),
                }),
            )
        }
    };

    match transcriber.generate_text(&prompt).await {
        Ok(formatted) => {
            let title = if request.mode == "ticket" {
                extract_title(&formatted)
            } else {
                None
            };
            (
                StatusCode::OK,
                Json(FormatResponse {
                    success: true,
                    formatted: Some(formatted),
                    title,
                    error: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FormatResponse {
                success: false,
                formatted: None,
                title: None,
                error: Some(format!("Format failed: {}", e)),
            }),
        ),
    }
}

// ============ Helpers ============

fn get_gemini_key(data_dir: &Path) -> Result<String, String> {
    let secrets_store = SecretsStore::new(data_dir.join("runtime.sqlite3"))
        .map_err(|e| format!("Failed to open secrets store: {}", e))?;

    secrets_store
        .get_gemini_key()
        .map_err(|e| format!("Failed to read Gemini key: {}", e))?
        .ok_or_else(|| "Gemini API key not configured. Set it in Settings > API Keys.".to_string())
}

fn build_format_prompt(request: &FormatRequest) -> String {
    match request.mode.as_str() {
        "ticket" => format!(
            "Convert the following voice transcription into a well-structured task/ticket in markdown format.\n\
             Include these sections:\n\
             - **Title** (first line, as a # heading)\n\
             - **Description** (clear explanation of what needs to be done)\n\
             - **Acceptance Criteria** (bullet list of measurable criteria)\n\
             - **Definition of Done** (checklist)\n\n\
             If appropriate, include Mermaid diagrams for workflows or architecture.\n\n\
             <voice_transcription>\n{}\n</voice_transcription>", request.transcription
        ),
        "edit" => {
            let existing = request.existing_content.as_deref().unwrap_or("");
            format!(
                "You are editing an existing document based on voice instructions.\n\n\
                 <existing_content>\n{}\n</existing_content>\n\n\
                 <voice_instructions>\n{}\n</voice_instructions>\n\n\
                 Produce the updated markdown document. Preserve the overall structure \
                 but apply the requested changes. If appropriate, include Mermaid diagrams.",
                existing, request.transcription
            )
        }
        // Mode is validated before this function is called
        _ => format!(
            "Format the following voice transcription as a clean, well-structured markdown comment. \
             Fix grammar, organize thoughts, and use appropriate formatting (bold, lists, etc.) \
             while preserving the original intent and tone.\n\n\
             <voice_transcription>\n{}\n</voice_transcription>",
            request.transcription
        ),
    }
}

fn extract_title(formatted: &str) -> Option<String> {
    for line in formatted.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            return Some(trimmed.trim_start_matches('#').trim().to_string());
        }
    }
    formatted
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
}
