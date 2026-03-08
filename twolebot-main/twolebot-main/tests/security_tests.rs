//! Security tests for twolebot
//!
//! These tests verify security measures:
//! - Path traversal protection on media endpoints
//! - Dashboard/API access without credentials
//! - CORS configuration

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use std::sync::Arc;
use tempfile::tempdir;
use tower::util::ServiceExt;

use twolebot::{
    cron::CronFeed,
    logging::SharedLogger,
    server::{handlers::AppState, RouterBuilder, RouterConfig},
    storage::{ChatMetadataStore, MediaStore, MessageStore, PromptFeed, ResponseFeed, SettingsStore},
};

/// Create a test app state with temporary directories
fn create_test_state() -> (AppState, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let state = AppState {
        prompt_feed: Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        response_feed: Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        message_store: Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        media_store: Arc::new(MediaStore::new(dir.path().join("media")).unwrap()),
        cron_feed: Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        settings_store: Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        chat_metadata_store: Arc::new(ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        logger: SharedLogger::new(dir.path().join("logs.jsonl")).unwrap(),
        data_dir: dir.path().to_path_buf(),
    };
    (state, dir)
}

// ============ Path Traversal Tests ============

#[tokio::test]
async fn test_media_path_traversal_dot_dot() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig::default();
    let router = RouterBuilder::new(state).config(config).build();

    // Attempt path traversal with ../
    let request = Request::builder()
        .method("GET")
        .uri("/api/media/123/../../../etc/passwd")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // Should be 400 Bad Request (path traversal blocked) or 404 Not Found
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Path traversal should be blocked, got: {}",
        response.status()
    );
}

#[tokio::test]
async fn test_media_path_traversal_encoded() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig::default();
    let router = RouterBuilder::new(state).config(config).build();

    // Attempt path traversal with URL-encoded ../
    let request = Request::builder()
        .method("GET")
        .uri("/api/media/123/%2e%2e%2f%2e%2e%2fetc/passwd")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // Should be blocked
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Encoded path traversal should be blocked, got: {}",
        response.status()
    );
}

#[tokio::test]
async fn test_media_path_absolute() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig::default();
    let router = RouterBuilder::new(state).config(config).build();

    // Attempt absolute path
    let request = Request::builder()
        .method("GET")
        .uri("/api/media//etc/passwd")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // Should be blocked - either 400 or 404
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Absolute path should be blocked, got: {}",
        response.status()
    );
}

// ============ CORS Tests ============

#[tokio::test]
async fn test_cors_preflight_allowed_origin() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig {
        cors_allow_all: false,
        host: "127.0.0.1".to_string(),
        port: 8080,
    };
    let router = RouterBuilder::new(state).config(config).build();

    // OPTIONS preflight request from allowed origin
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/status")
        .header(header::ORIGIN, "http://127.0.0.1:8080")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // Should allow the preflight
    assert!(
        response.status().is_success() || response.status() == StatusCode::NO_CONTENT,
        "CORS preflight from allowed origin should succeed, got: {}",
        response.status()
    );
}

#[tokio::test]
async fn test_cors_allow_all_mode() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig {
        cors_allow_all: true,
        host: "127.0.0.1".to_string(),
        port: 8080,
    };
    let router = RouterBuilder::new(state).config(config).build();

    // Request with arbitrary origin when cors_allow_all is true
    let request = Request::builder()
        .method("GET")
        .uri("/api/status")
        .header(header::ORIGIN, "http://evil.com")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // Should succeed when cors_allow_all is true
    assert_eq!(response.status(), StatusCode::OK);
}

// ============ Access Tests ============

#[tokio::test]
async fn test_status_endpoint_accessible() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig {
        cors_allow_all: true,
        host: "127.0.0.1".to_string(),
        port: 8080,
    };
    let router = RouterBuilder::new(state).config(config).build();

    // /api/status should be accessible
    let request = Request::builder()
        .method("GET")
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Status endpoint should be accessible"
    );
}

#[tokio::test]
async fn test_feed_endpoint_accessible_without_auth_header() {
    let (state, _dir) = create_test_state();
    let config = RouterConfig {
        cors_allow_all: true,
        host: "127.0.0.1".to_string(),
        port: 8080,
    };
    let router = RouterBuilder::new(state).config(config).build();

    // Endpoint should not require any auth header
    let request = Request::builder()
        .method("GET")
        .uri("/api/feed")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Feed endpoint should be accessible without auth"
    );
}

// ============ Property Tests ============

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_path_traversal_variants_blocked(
            prefix in "[a-z0-9]{1,10}",
            depth in 1usize..5,
            suffix in "[a-z]{1,10}"
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result: Result<(), TestCaseError> = rt.block_on(async {
                let (state, _dir) = create_test_state();
                let config = RouterConfig::default();
                let router = RouterBuilder::new(state).config(config).build();

                // Build path with varying depth of ../
                let traversal = "../".repeat(depth);
                let path = format!("/api/media/{}/{}{}", prefix, traversal, suffix);

                let request = Request::builder()
                    .method("GET")
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap();

                let response = router.oneshot(request).await.unwrap();

                // All traversal attempts should be blocked (400 or 404)
                prop_assert!(
                    response.status() == StatusCode::BAD_REQUEST
                        || response.status() == StatusCode::NOT_FOUND,
                    "Path traversal should be blocked for path: {}, got: {}",
                    path,
                    response.status()
                );
                Ok(())
            });
            result?;
        }

        #[test]
        fn prop_valid_chat_id_media_path_accepted(
            chat_id in "[0-9]{5,15}",
            filename in "[a-zA-Z0-9_-]{1,20}\\.[a-z]{2,4}"
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result: Result<(), TestCaseError> = rt.block_on(async {
                let (state, _dir) = create_test_state();
                let config = RouterConfig::default();
                let router = RouterBuilder::new(state).config(config).build();

                // Valid-looking media path (file won't exist, but path should be accepted)
                let path = format!("/api/media/{}/{}", chat_id, filename);

                let request = Request::builder()
                    .method("GET")
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap();

                let response = router.oneshot(request).await.unwrap();

                // Should be 404 (file not found) not 400 (bad request)
                // This confirms the path format is accepted
                prop_assert_eq!(
                    response.status(),
                    StatusCode::NOT_FOUND,
                    "Valid media path format should be accepted (404 not 400) for: {}",
                    path
                );
                Ok(())
            });
            result?;
        }
    }
}
