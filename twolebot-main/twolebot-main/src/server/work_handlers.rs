use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Duration;

use crate::types::work as wt;
use crate::work::{
    adapters::{paginated_payload, to_err_response, work_error, ErrorResponse},
    app::{CreateDocumentInput, CreateTaskInput},
    models::{AgentLoopState, DocumentType, TaskPriority, TaskStatus, TaskUpdate},
    AgentLoop, WorkApp,
};
use crate::{cron::ActivityTracker, storage::ResponseFeed};

/// Shared state for work endpoints
#[derive(Clone)]
pub struct WorkState {
    pub app: std::sync::Arc<WorkApp>,
    pub agent_loop: std::sync::Arc<AgentLoop>,
    pub activity_tracker: ActivityTracker,
    pub response_feed: std::sync::Arc<ResponseFeed>,
    pub idle_threshold_secs: i64,
}

pub async fn list_projects(
    State(state): State<WorkState>,
    Json(req): Json<wt::ListProjectsRequest>,
) -> impl IntoResponse {
    match state
        .app
        .projects
        .list(
            req.active_only.unwrap_or(true),
            req.limit.unwrap_or(20),
            req.git_remote_url,
        )
        .await
    {
        Ok(paginated) => Json(paginated_payload(paginated)).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn get_project(
    State(state): State<WorkState>,
    Json(req): Json<wt::GetProjectRequest>,
) -> impl IntoResponse {
    match state.app.projects.get(req.project_id).await {
        Ok(project) => Json(serde_json::json!({ "data": project })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn create_project(
    State(state): State<WorkState>,
    Json(req): Json<wt::CreateProjectRequest>,
) -> impl IntoResponse {
    match state
        .app
        .projects
        .create(
            req.name,
            req.description,
            req.tags,
            req.git_remote_url,
        )
        .await
    {
        Ok(project) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": project })),
        )
            .into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn update_project(
    State(state): State<WorkState>,
    Json(req): Json<wt::UpdateProjectRequest>,
) -> impl IntoResponse {
    match state
        .app
        .projects
        .update(
            req.project_id,
            req.name,
            req.description,
            req.tags,
            req.git_remote_url,
        )
        .await
    {
        Ok(project) => Json(serde_json::json!({ "data": project })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn list_tasks(
    State(state): State<WorkState>,
    Json(req): Json<wt::ListTasksRequest>,
) -> impl IntoResponse {
    let limit = req.limit.unwrap_or(50);
    if req.compact.unwrap_or(false) {
        match state
            .app
            .tasks
            .list_compact(req.project_id, req.status, limit)
            .await
        {
            Ok(paginated) => Json(paginated_payload(paginated)).into_response(),
            Err(e) => to_err_response(e).into_response(),
        }
    } else {
        match state
            .app
            .tasks
            .list(req.project_id, req.status, limit)
            .await
        {
            Ok(paginated) => Json(paginated_payload(paginated)).into_response(),
            Err(e) => to_err_response(e).into_response(),
        }
    }
}

pub async fn get_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::GetTaskRequest>,
) -> impl IntoResponse {
    match state.app.tasks.get(req.task_id).await {
        Ok(task) => Json(serde_json::json!({ "data": task })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn create_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::CreateTaskRequest>,
) -> impl IntoResponse {
    let status = match req
        .status
        .as_ref()
        .map(|s| s.parse::<TaskStatus>())
        .transpose()
    {
        Ok(s) => s,
        Err(e) => return work_error(StatusCode::BAD_REQUEST, e).into_response(),
    };
    let priority = match req
        .priority
        .as_ref()
        .map(|p| p.parse::<TaskPriority>())
        .transpose()
    {
        Ok(p) => p,
        Err(e) => return work_error(StatusCode::BAD_REQUEST, e).into_response(),
    };

    let input = CreateTaskInput {
        project_id: req.project_id,
        title: req.title,
        description: req.description,
        status,
        priority,
        tags: req.tags,
    };

    match state.app.tasks.create(input).await {
        Ok(task) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": task })),
        )
            .into_response(),
        Err(e) => {
            let status_code = if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else if matches!(&e, crate::TwolebotError::NotFound { .. }) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status_code,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn update_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::UpdateTaskRequest>,
) -> impl IntoResponse {
    let status = match req
        .status
        .as_ref()
        .map(|s| s.parse::<TaskStatus>())
        .transpose()
    {
        Ok(s) => s,
        Err(e) => return work_error(StatusCode::BAD_REQUEST, e).into_response(),
    };
    let priority = match req
        .priority
        .as_ref()
        .map(|p| p.parse::<TaskPriority>())
        .transpose()
    {
        Ok(p) => p,
        Err(e) => return work_error(StatusCode::BAD_REQUEST, e).into_response(),
    };

    let update = TaskUpdate {
        title: req.title,
        description: req.description,
        status,
        priority,
        tags: req.tags,
        comment: req.comment,
        ..Default::default()
    };

    match state.app.tasks.update(req.task_id, update).await {
        Ok(task) => Json(serde_json::json!({ "data": task })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn take_next_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::TakeNextRequest>,
) -> impl IntoResponse {
    match state
        .app
        .tasks
        .take_next(req.project_id, req.force.unwrap_or(false))
        .await
    {
        Ok(Some(take_result)) => Json(serde_json::json!({ "data": take_result })).into_response(),
        Ok(None) => Json(serde_json::json!({
            "data": null,
            "message": "No Todo tasks available in this project"
        }))
        .into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn take_next_review_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::TakeNextRequest>,
) -> impl IntoResponse {
    match state
        .app
        .tasks
        .take_next_review(req.project_id, req.force.unwrap_or(false))
        .await
    {
        Ok(Some(take_result)) => Json(serde_json::json!({ "data": take_result })).into_response(),
        Ok(None) => Json(serde_json::json!({
            "data": null,
            "message": "No tasks ready for review in this project"
        }))
        .into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn move_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::MoveTaskRequest>,
) -> impl IntoResponse {
    match state
        .app
        .tasks
        .move_to_top_or_bottom(req.task_id, req.position)
        .await
    {
        Ok(task) => Json(serde_json::json!({ "data": task })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn reject_review(
    State(state): State<WorkState>,
    Json(req): Json<wt::RejectReviewRequest>,
) -> impl IntoResponse {
    match state
        .app
        .tasks
        .reject_review(req.task_id, req.reviewer_comment)
        .await
    {
        Ok(task) => Json(serde_json::json!({ "data": task })).into_response(),
        Err(e) => {
            let status_code = if e.to_string().contains("not under review") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status_code,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn task_analytics(
    State(state): State<WorkState>,
    Json(req): Json<wt::AnalyticsRequest>,
) -> impl IntoResponse {
    match state.app.tasks.analytics(req.project_id).await {
        Ok(analytics) => Json(serde_json::json!({ "data": analytics })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn search_documents(
    State(state): State<WorkState>,
    Json(req): Json<wt::SearchDocumentsRequest>,
) -> impl IntoResponse {
    match state
        .app
        .documents
        .search(req.query, req.project_id, req.limit.unwrap_or(20))
        .await
    {
        Ok(paginated) => Json(paginated_payload(paginated)).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn get_document(
    State(state): State<WorkState>,
    Json(req): Json<wt::GetDocumentRequest>,
) -> impl IntoResponse {
    match state.app.documents.get(req.document_id).await {
        Ok(doc) => Json(serde_json::json!({ "data": doc })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn create_document(
    State(state): State<WorkState>,
    Json(req): Json<wt::CreateDocumentRequest>,
) -> impl IntoResponse {
    let document_type = match req.document_type.parse::<DocumentType>() {
        Ok(dt) => dt,
        Err(e) => return work_error(StatusCode::BAD_REQUEST, e).into_response(),
    };

    let input = CreateDocumentInput {
        project_id: req.project_id,
        document_type,
        title: req.title,
        content: req.content,
    };

    match state.app.documents.create(input).await {
        Ok(doc) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": doc })),
        )
            .into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn update_document(
    State(state): State<WorkState>,
    Json(req): Json<wt::UpdateDocumentRequest>,
) -> impl IntoResponse {
    let document_type = match req.document_type.parse::<DocumentType>() {
        Ok(dt) => dt,
        Err(e) => return work_error(StatusCode::BAD_REQUEST, e).into_response(),
    };

    match state
        .app
        .documents
        .update(
            req.document_id,
            req.title,
            req.content,
            document_type,
        )
        .await
    {
        Ok(doc) => Json(serde_json::json!({ "data": doc })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn list_comments(
    State(state): State<WorkState>,
    Json(req): Json<wt::ListCommentsRequest>,
) -> impl IntoResponse {
    if req.task_id.is_none() && req.document_id.is_none() {
        return work_error(
            StatusCode::BAD_REQUEST,
            "task_id or document_id is required",
        )
        .into_response();
    }

    let limit = req.limit.unwrap_or(20);
    let page = req.page.unwrap_or(1);
    let result = if let Some(task_id) = req.task_id {
        state.app.comments.list_task(task_id, limit, page).await
    } else if let Some(document_id) = req.document_id {
        state
            .app
            .comments
            .list_document(document_id, limit, page)
            .await
    } else {
        Err(crate::TwolebotError::work(
            "task_id or document_id is required",
        ))
    };

    match result {
        Ok(paginated) => Json(paginated_payload(paginated)).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn upsert_comment(
    State(state): State<WorkState>,
    Json(req): Json<wt::UpsertCommentRequest>,
) -> impl IntoResponse {
    let is_update = req.comment_id.is_some();

    match state
        .app
        .comments
        .upsert(
            req.comment_id,
            req.task_id,
            req.document_id,
            req.content,
            req.parent_comment_id,
        )
        .await
    {
        Ok(comment) => {
            let status = if is_update {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            (status, Json(serde_json::json!({ "data": comment }))).into_response()
        }
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn recent_activity(
    State(state): State<WorkState>,
    Json(req): Json<wt::RecentActivityRequest>,
) -> impl IntoResponse {
    match state
        .app
        .activity
        .recent(req.limit.unwrap_or(50), req.project_id)
        .await
    {
        Ok(logs) => Json(serde_json::json!({ "data": logs })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn start_agent_loop(State(state): State<WorkState>) -> impl IntoResponse {
    match state.agent_loop.start().await {
        Ok(()) => Json(serde_json::json!({ "message": "Agent loop started" })).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already running") {
                Json(serde_json::json!({ "message": "Agent loop already running" })).into_response()
            } else {
                work_error(StatusCode::BAD_REQUEST, msg).into_response()
            }
        }
    }
}

pub async fn stop_agent_loop(State(state): State<WorkState>) -> impl IntoResponse {
    match state.agent_loop.stop().await {
        Ok(()) => Json(serde_json::json!({ "message": "Agent loop stopping" })).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not running") {
                Json(serde_json::json!({ "message": "Agent loop already stopped" })).into_response()
            } else {
                work_error(StatusCode::BAD_REQUEST, msg).into_response()
            }
        }
    }
}

pub async fn ensure_agent_loop(
    State(state): State<WorkState>,
    Json(req): Json<wt::EnsureAgentRequest>,
) -> impl IntoResponse {
    let is_quiet = state
        .activity_tracker
        .is_idle_for(Duration::seconds(state.idle_threshold_secs))
        .await;
    if !is_quiet {
        return Json(serde_json::json!({
            "message": format!(
                "Deferring autowork until quiet window elapses ({}s)",
                state.idle_threshold_secs
            ),
            "data": { "selected_from_backlog": false, "started": false }
        }))
        .into_response();
    }
    if state
        .response_feed
        .has_pending_system_responses()
        .unwrap_or(false)
    {
        return Json(serde_json::json!({
            "message": "Waiting for previous machine response delivery",
            "data": { "selected_from_backlog": false, "started": false }
        }))
        .into_response();
    }

    let auto_select = req.auto_select_from_todo.unwrap_or(true);

    let board = match state.app.live.get_live_board(Some(1)).await {
        Ok(board) => board,
        Err(e) => return to_err_response(e).into_response(),
    };

    let has_selection_work = board.stats.active.is_some()
        || board.stats.queued > 0
        || board
            .selected
            .iter()
            .any(|s| s.selection.status == crate::work::models::SelectionStatus::Paused);

    let mut selected_from_backlog = false;
    if !has_selection_work && auto_select && board.stats.total_backlog > 0 {
        match state.app.live.select_next_todo_for_agent().await {
            Ok(Some(_)) => selected_from_backlog = true,
            Ok(None) => {}
            Err(e) => return to_err_response(e).into_response(),
        }
    }

    if state.agent_loop.state().await == AgentLoopState::Running {
        return Json(serde_json::json!({
            "message": "Agent loop already running",
            "data": { "selected_from_backlog": selected_from_backlog, "started": false }
        }))
        .into_response();
    }

    match state.agent_loop.start().await {
        Ok(()) => Json(serde_json::json!({
            "message": if selected_from_backlog {
                "Queued next todo and started agent loop"
            } else {
                "Agent loop started"
            },
            "data": { "selected_from_backlog": selected_from_backlog, "started": true }
        }))
        .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already running") {
                Json(serde_json::json!({
                    "message": "Agent loop already running",
                    "data": { "selected_from_backlog": selected_from_backlog, "started": false }
                }))
                .into_response()
            } else if msg.contains("no tasks in selection queue") {
                Json(serde_json::json!({
                    "message": "No queued tasks available to start",
                    "data": { "selected_from_backlog": selected_from_backlog, "started": false }
                }))
                .into_response()
            } else {
                work_error(StatusCode::BAD_REQUEST, msg).into_response()
            }
        }
    }
}

// ── Live board handlers ─────────────────────────────────────────────────────

pub async fn get_live_board(
    State(state): State<WorkState>,
    Json(req): Json<wt::GetLiveBoardRequest>,
) -> impl IntoResponse {
    match state.app.live.get_live_board(req.backlog_limit).await {
        Ok(board) => Json(serde_json::json!({ "data": board })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn select_tasks(
    State(state): State<WorkState>,
    Json(req): Json<wt::SelectTasksRequest>,
) -> impl IntoResponse {
    match state.app.live.select_tasks(req.task_ids).await {
        Ok(selections) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": selections })),
        )
            .into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn deselect_task(
    State(state): State<WorkState>,
    Json(req): Json<wt::DeselectTaskRequest>,
) -> impl IntoResponse {
    match state.app.live.deselect_task(req.task_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn move_selection(
    State(state): State<WorkState>,
    Json(req): Json<wt::MoveSelectionRequest>,
) -> impl IntoResponse {
    match state
        .app
        .live
        .move_selection(req.task_id, req.position)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}

pub async fn clear_completed(State(state): State<WorkState>) -> impl IntoResponse {
    match state.app.live.clear_completed_selections().await {
        Ok(count) => Json(serde_json::json!({ "data": { "cleared": count } })).into_response(),
        Err(e) => to_err_response(e).into_response(),
    }
}
