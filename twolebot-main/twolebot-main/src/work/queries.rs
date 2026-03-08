use rusqlite::{params, Row};

use crate::TwolebotError;

use super::models::*;

// ── Row mapping helpers ─────────────────────────────────────────────────────

pub fn row_to_project(row: &Row<'_>) -> Result<Project, rusqlite::Error> {
    let tags_json: String = row.get("tags_json")?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_else(|e| {
        tracing::warn!("Failed to parse tags JSON in project row: {e} (json: {tags_json})");
        Vec::new()
    });
    let is_active_int: i64 = row.get("is_active")?;
    let task_count: i64 = row.get("task_count").unwrap_or(0);
    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        git_remote_url: row.get("git_remote_url")?,
        tags,
        is_active: is_active_int != 0,
        task_count,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn row_to_task(row: &Row<'_>) -> Result<TaskModel, rusqlite::Error> {
    let tags_json: String = row.get("tags_json")?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_else(|e| {
        tracing::warn!("Failed to parse tags JSON in task row: {e} (json: {tags_json})");
        Vec::new()
    });
    let status_str: String = row.get("status")?;
    let priority_str: String = row.get("priority")?;

    Ok(TaskModel {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        status: status_str.parse().map_err(|e: String| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?,
        priority: priority_str.parse().map_err(|e: String| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?,
        sort_order: row.get("sort_order")?,
        title: row.get("title")?,
        description: row.get("description")?,
        tags,
        completed_at: row.get("completed_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        blocked_by: Vec::new(),
        blocks: Vec::new(),
    })
}

pub fn row_to_document(row: &Row<'_>) -> Result<Document, rusqlite::Error> {
    let doc_type_str: String = row.get("document_type")?;
    let deleted_int: i64 = row.get("deleted")?;

    Ok(Document {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        document_type: doc_type_str.parse().map_err(|e: String| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?,
        title: row.get("title")?,
        content: row.get("content")?,
        version: row.get("version")?,
        deleted: deleted_int != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn row_to_comment(row: &Row<'_>) -> Result<Comment, rusqlite::Error> {
    Ok(Comment {
        id: row.get("id")?,
        task_id: row.get("task_id")?,
        document_id: row.get("document_id")?,
        parent_comment_id: row.get("parent_comment_id")?,
        content: row.get("content")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn row_to_activity_log(row: &Row<'_>) -> Result<ActivityLog, rusqlite::Error> {
    let action_str: String = row.get("action")?;

    Ok(ActivityLog {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        task_id: row.get("task_id")?,
        document_id: row.get("document_id")?,
        action: action_str.parse().map_err(|e: String| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?,
        actor: row.get("actor")?,
        details: row.get("details")?,
        created_at: row.get("created_at")?,
    })
}

pub fn row_to_live_board_selection(row: &Row<'_>) -> Result<LiveBoardSelection, rusqlite::Error> {
    let status_str: String = row.get("status")?;

    Ok(LiveBoardSelection {
        id: row.get("id")?,
        task_id: row.get("task_id")?,
        sort_order: row.get("sort_order")?,
        selected_at: row.get("selected_at")?,
        started_at: row.get("started_at")?,
        completed_at: row.get("completed_at")?,
        status: status_str.parse().map_err(|e: String| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?,
    })
}

// ── Sync task_tags from tags_json ───────────────────────────────────────────

pub fn sync_task_tags(
    conn: &rusqlite::Connection,
    task_id: i64,
    tags: &[String],
) -> Result<(), TwolebotError> {
    conn.execute("DELETE FROM task_tags WHERE task_id = ?1", params![task_id])
        .map_err(|e| TwolebotError::work(format!("failed to clear task tags: {e}")))?;

    let mut stmt = conn
        .prepare("INSERT INTO task_tags (task_id, tag) VALUES (?1, ?2)")
        .map_err(|e| TwolebotError::work(format!("failed to prepare tag insert: {e}")))?;

    for tag in tags {
        stmt.execute(params![task_id, tag])
            .map_err(|e| TwolebotError::work(format!("failed to insert tag: {e}")))?;
    }
    Ok(())
}

// ── Load task dependencies ──────────────────────────────────────────────────

pub fn load_task_dependencies(
    conn: &rusqlite::Connection,
    task_id: i64,
) -> Result<(Vec<i64>, Vec<i64>), TwolebotError> {
    let mut blocked_by_stmt = conn
        .prepare("SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1")
        .map_err(|e| TwolebotError::work(format!("failed to prepare blocked_by query: {e}")))?;
    let blocked_by: Vec<i64> = blocked_by_stmt
        .query_map(params![task_id], |row| row.get(0))
        .map_err(|e| TwolebotError::work(format!("failed to query blocked_by: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TwolebotError::work(format!("failed to read blocked_by: {e}")))?;

    let mut blocks_stmt = conn
        .prepare("SELECT task_id FROM task_dependencies WHERE depends_on_task_id = ?1")
        .map_err(|e| TwolebotError::work(format!("failed to prepare blocks query: {e}")))?;
    let blocks: Vec<i64> = blocks_stmt
        .query_map(params![task_id], |row| row.get(0))
        .map_err(|e| TwolebotError::work(format!("failed to query blocks: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TwolebotError::work(format!("failed to read blocks: {e}")))?;

    Ok((blocked_by, blocks))
}

// ── Cycle detection for task dependencies ───────────────────────────────────

pub fn would_create_cycle(
    conn: &rusqlite::Connection,
    task_id: i64,
    depends_on_task_id: i64,
) -> Result<bool, TwolebotError> {
    // BFS from depends_on_task_id: if we can reach task_id, adding this edge would create a cycle
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(depends_on_task_id);

    while let Some(current) = queue.pop_front() {
        if current == task_id {
            return Ok(true);
        }
        if !visited.insert(current) {
            continue;
        }
        let mut stmt = conn
            .prepare("SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1")
            .map_err(|e| TwolebotError::work(format!("cycle check query failed: {e}")))?;
        let deps: Vec<i64> = stmt
            .query_map(params![current], |row| row.get(0))
            .map_err(|e| TwolebotError::work(format!("cycle check read failed: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("cycle check collect failed: {e}")))?;
        for dep in deps {
            if !visited.contains(&dep) {
                queue.push_back(dep);
            }
        }
    }

    Ok(false)
}
