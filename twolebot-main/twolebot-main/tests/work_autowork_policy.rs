//! Integration tests for live-board autowork idle policy.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tempfile::tempdir;
use tower::util::ServiceExt;

use twolebot::{
    cron::{ActivityTracker, CronFeed},
    logging::SharedLogger,
    server::{
        handlers::AppState, work_handlers::WorkState, RouterBuilder,
    },
    storage::{ChatMetadataStore, MediaStore, MessageStore, PromptFeed, ResponseFeed, SettingsStore},
    work::{app::CreateTaskInput, models::TaskPriority, WorkApp, WorkDb},
};

fn create_base_state(dir: &tempfile::TempDir) -> AppState {
    AppState {
        prompt_feed: Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        response_feed: Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        message_store: Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        media_store: Arc::new(MediaStore::new(dir.path().join("media")).unwrap()),
        cron_feed: Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        settings_store: Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        chat_metadata_store: Arc::new(ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        logger: SharedLogger::new(dir.path().join("logs.jsonl")).unwrap(),
        data_dir: dir.path().to_path_buf(),
    }
}

#[tokio::test]
async fn test_ensure_agent_loop_defers_then_starts_after_idle_window() {
    let dir = tempdir().unwrap();
    let base_state = create_base_state(&dir);

    let work_db = WorkDb::open(dir.path()).unwrap();
    let app = Arc::new(WorkApp::new(work_db));
    let agent_loop = Arc::new(app.new_agent_loop(
        base_state.prompt_feed.clone(),
        base_state.response_feed.clone(),
        base_state.settings_store.clone(),
    ));
    let activity_tracker = ActivityTracker::new();

    // Seed one todo task so ensure_agent_loop can auto-select and start once idle.
    let project = app
        .projects
        .create(
            "autowork-policy".to_string(),
            Some("test".to_string()),
            Some(vec!["test".to_string()]),
            None,
        )
        .await
        .unwrap();
    app.tasks
        .create(CreateTaskInput {
            project_id: project.id,
            title: "idle-threshold-task".to_string(),
            description: Some("policy test".to_string()),
            status: None,
            priority: Some(TaskPriority::Medium),
            tags: Some(vec!["autowork".to_string()]),
        })
        .await
        .unwrap();

    let work_state = WorkState {
        app,
        agent_loop,
        activity_tracker: activity_tracker.clone(),
        response_feed: base_state.response_feed.clone(),
        idle_threshold_secs: 1,
    };

    let router = RouterBuilder::new(base_state)
        .work(work_state)
        .build();

    // Activity is "just now", so ensure should defer.
    activity_tracker.record_activity().await;
    let deferred_req = Request::builder()
        .method("POST")
        .uri("/api/work/live-board/agent/ensure")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"auto_select_from_todo":true}"#))
        .unwrap();
    let deferred_resp = router.clone().oneshot(deferred_req).await.unwrap();
    assert_eq!(deferred_resp.status(), StatusCode::OK);
    let deferred_body = axum::body::to_bytes(deferred_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let deferred_json: Value = serde_json::from_slice(&deferred_body).unwrap();
    assert_eq!(
        deferred_json
            .get("data")
            .and_then(|d| d.get("started"))
            .and_then(|s| s.as_bool()),
        Some(false)
    );

    // Wait for idle window and retry; now it should start.
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let start_req = Request::builder()
        .method("POST")
        .uri("/api/work/live-board/agent/ensure")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"auto_select_from_todo":true}"#))
        .unwrap();
    let start_resp = router.oneshot(start_req).await.unwrap();
    assert_eq!(start_resp.status(), StatusCode::OK);
    let start_body = axum::body::to_bytes(start_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let start_json: Value = serde_json::from_slice(&start_body).unwrap();
    assert_eq!(
        start_json
            .get("data")
            .and_then(|d| d.get("started"))
            .and_then(|s| s.as_bool()),
        Some(true)
    );
}
