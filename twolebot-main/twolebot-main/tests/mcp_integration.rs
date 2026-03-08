//! Integration tests for MCP HTTP endpoint
//!
//! These tests verify that the MCP server can be started and responds to requests.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tempfile::tempdir;
use tokio_util::sync::CancellationToken;
use tower::util::ServiceExt;

// Import the necessary types from twolebot
use std::path::PathBuf;
use twolebot::{
    cron::CronFeed,
    logging::SharedLogger,
    mcp::McpHttpState,
    server::{create_router, handlers::AppState},
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

#[tokio::test]
async fn test_mcp_endpoint_exists() {
    let (state, _dir) = create_test_state();
    let cron_feed = state.cron_feed.clone();
    let cancellation = CancellationToken::new();
    let mcp_state = McpHttpState::new(
        cron_feed,
        _dir.path().join("memory"),
        PathBuf::from("/tmp/nonexistent"),
        PathBuf::from("/tmp/nonexistent"),
        cancellation,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let router = create_router(state, Some(mcp_state), None);

    // Send a request to the MCP endpoint
    // MCP uses Server-Sent Events, so we just verify it accepts POST
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        ))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // MCP endpoint should accept the request (not 404)
    // It may return various status codes depending on the request content
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cron_api_list_jobs() {
    let (state, _dir) = create_test_state();
    let router = create_router(state, None, None);

    let request = Request::builder()
        .method("GET")
        .uri("/api/cron/jobs")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cron_api_create_and_list_job() {
    let (state, _dir) = create_test_state();

    // Clone state for second request
    let cron_feed = state.cron_feed.clone();

    let router = create_router(state, None, None);

    // Create a job via the REST API
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/cron/jobs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Test","prompt":"Test job","in_minutes":10}"#))
        .unwrap();

    let response = router.oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify job was created
    let jobs = cron_feed.list_active_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].prompt, "Test job");
}

#[tokio::test]
async fn test_cron_api_create_multi_cron_job() {
    let (state, _dir) = create_test_state();
    let cron_feed = state.cron_feed.clone();
    let router = create_router(state, None, None);

    // Create a multi-cron job via REST API using the new 'crons' field
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/cron/jobs")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"name":"Study session","prompt":"Check in","crons":["0 0 6 * * *","0 30 6 * * *","0 0 7 * * *"]}"#,
        ))
        .unwrap();

    let response = router.oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify single job was created with all 3 expressions
    let jobs = cron_feed.list_active_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name, "Study session");
    assert_eq!(jobs[0].schedule.all_expressions().len(), 3);
    assert!(jobs[0].next_run.is_some());
}

#[tokio::test]
async fn test_cron_api_cancel_job() {
    let (state, _dir) = create_test_state();
    let cron_feed = state.cron_feed.clone();

    // Create a job directly
    use twolebot::cron::{CronJob, CronSchedule};
    let job = cron_feed
        .create_job(CronJob::new("Cancel test", "To cancel", CronSchedule::from_minutes(10)))
        .unwrap();

    let router = create_router(state, None, None);

    // Cancel via API
    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/cron/jobs/{}", job.id))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify cancelled
    let cancelled = cron_feed.get_job(&job.id).unwrap().unwrap();
    assert_eq!(cancelled.status, twolebot::cron::CronJobStatus::Cancelled);
}

#[tokio::test]
async fn test_cron_api_get_status() {
    let (state, _dir) = create_test_state();
    let router = create_router(state, None, None);

    let request = Request::builder()
        .method("GET")
        .uri("/api/cron/status")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_status_endpoint() {
    let (state, _dir) = create_test_state();
    let router = create_router(state, None, None);

    let request = Request::builder()
        .method("GET")
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_router_without_mcp_returns_404_on_mcp_endpoint() {
    let (state, _dir) = create_test_state();
    let router = create_router(state, None, None);

    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    // Without MCP state, the /mcp endpoint should not exist
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
