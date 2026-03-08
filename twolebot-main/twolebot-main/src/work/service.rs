use rusqlite::params;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

use crate::TwolebotError;

use super::db::WorkDb;
use super::models::*;
use super::queries::*;

/// Query tasks by status for a project, returning collected Vec (avoids lifetime issues).
fn query_tasks_by_status(
    conn: &rusqlite::Connection,
    project_id: i64,
    status: &str,
) -> Result<Vec<TaskModel>, TwolebotError> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM tasks WHERE project_id = ?1 AND status = ?2
             ORDER BY sort_order ASC",
        )
        .map_err(|e| TwolebotError::work(format!("prepare {status} query: {e}")))?;
    let results: Vec<TaskModel> = stmt
        .query_map(params![project_id, status], row_to_task)
        .map_err(|e| TwolebotError::work(format!("query {status}: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TwolebotError::work(format!("collect {status}: {e}")))?;
    Ok(results)
}

/// Core business logic for the work module.
/// All methods are synchronous (SQLite is sync); callers use spawn_blocking.
pub struct WorkService {
    db: WorkDb,
    event_tx: broadcast::Sender<WorkEvent>,
    agent_loop_state: Arc<Mutex<AgentLoopState>>,
}

impl WorkService {
    pub fn new(db: WorkDb) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            db,
            event_tx,
            agent_loop_state: Arc::new(Mutex::new(AgentLoopState::Idle)),
        }
    }

    /// Get a reference to the event broadcast sender (for SSE subscribers).
    pub fn event_tx(&self) -> &broadcast::Sender<WorkEvent> {
        &self.event_tx
    }

    /// Get a reference to the agent loop state mutex.
    pub fn agent_loop_state(&self) -> &Arc<Mutex<AgentLoopState>> {
        &self.agent_loop_state
    }

    /// Get a reference to the database (for live_board use).
    pub fn db(&self) -> &WorkDb {
        &self.db
    }

    /// Broadcast an event to all SSE subscribers (ignores send errors when no receivers).
    pub(crate) fn broadcast(&self, event: WorkEvent) {
        let _ = self.event_tx.send(event);
    }

    // ── Projects ────────────────────────────────────────────────────────────

    pub fn list_projects(
        &self,
        active_only: bool,
        limit: i32,
        git_remote_url: Option<&str>,
    ) -> Result<PaginatedResponse<Project>, TwolebotError> {
        let conn = self.db.conn()?;

        let (sql, count_sql) = match (git_remote_url.is_some(), active_only) {
            (true, true) => (
                "SELECT p.*, (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count FROM projects p WHERE p.is_active = 1 AND p.git_remote_url = ?1 ORDER BY p.id DESC LIMIT ?2",
                "SELECT COUNT(*) FROM projects WHERE is_active = 1 AND git_remote_url = ?1",
            ),
            (true, false) => (
                "SELECT p.*, (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count FROM projects p WHERE p.git_remote_url = ?1 ORDER BY p.id DESC LIMIT ?2",
                "SELECT COUNT(*) FROM projects WHERE git_remote_url = ?1",
            ),
            (false, true) => (
                "SELECT p.*, (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count FROM projects p WHERE p.is_active = 1 ORDER BY p.id DESC LIMIT ?1",
                "SELECT COUNT(*) FROM projects WHERE is_active = 1",
            ),
            (false, false) => (
                "SELECT p.*, (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count FROM projects p ORDER BY p.id DESC LIMIT ?1",
                "SELECT COUNT(*) FROM projects",
            ),
        };

        let total: i64 = if let Some(url) = git_remote_url {
            conn.query_row(count_sql, params![url], |row| row.get(0))
        } else {
            conn.query_row(count_sql, [], |row| row.get(0))
        }
        .map_err(|e| TwolebotError::work(format!("count projects: {e}")))?;

        let items: Vec<Project> = if let Some(url) = git_remote_url {
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| TwolebotError::work(format!("prepare list projects: {e}")))?;
            let results: Vec<Project> = stmt
                .query_map(params![url, limit], row_to_project)
                .map_err(|e| TwolebotError::work(format!("query projects: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("read projects: {e}")))?;
            results
        } else {
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| TwolebotError::work(format!("prepare list projects: {e}")))?;
            let results: Vec<Project> = stmt
                .query_map(params![limit], row_to_project)
                .map_err(|e| TwolebotError::work(format!("query projects: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("read projects: {e}")))?;
            results
        };

        Ok(PaginatedResponse {
            total,
            page: 1,
            limit,
            items,
        })
    }

    pub fn get_project(&self, id: i64) -> Result<Project, TwolebotError> {
        let conn = self.db.conn()?;
        conn.query_row("SELECT p.*, (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count FROM projects p WHERE p.id = ?1", params![id], |row| {
            row_to_project(row)
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                TwolebotError::not_found(format!("project {id}"))
            }
            _ => TwolebotError::work(format!("get project {id}: {e}")),
        })
    }

    pub fn create_project(
        &self,
        name: &str,
        description: &str,
        tags: &[String],
        git_remote_url: Option<&str>,
    ) -> Result<Project, TwolebotError> {
        let conn = self.db.conn()?;
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO projects (name, description, git_remote_url, tags_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, description, git_remote_url, tags_json],
        )
        .map_err(|e| TwolebotError::work(format!("insert project: {e}")))?;

        let id = conn.last_insert_rowid();

        self.log_activity(
            &conn,
            Some(id),
            None,
            None,
            &ActivityAction::Created,
            &format!("Project created: {name}"),
        )?;

        self.get_project(id)
    }

    pub fn update_project(
        &self,
        id: i64,
        name: &str,
        description: Option<&str>,
        tags: Option<&[String]>,
        git_remote_url: Option<Option<&str>>,
    ) -> Result<Project, TwolebotError> {
        let conn = self.db.conn()?;

        // Verify exists
        let _existing = self.get_project(id)?;

        let tags_json = tags.map(|t| serde_json::to_string(t).unwrap_or_else(|_| "[]".to_string()));
        let git_url = git_remote_url.flatten();

        conn.execute(
            "UPDATE projects SET name = ?1, description = COALESCE(?2, description),
             tags_json = COALESCE(?3, tags_json), git_remote_url = CASE WHEN ?4 = 1 THEN ?5 ELSE git_remote_url END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE id = ?6",
            params![
                name,
                description,
                tags_json,
                git_remote_url.is_some() as i32,
                git_url,
                id,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("update project {id}: {e}")))?;

        self.get_project(id)
    }

    // ── Tasks ───────────────────────────────────────────────────────────────

    pub fn list_tasks(
        &self,
        project_id: Option<i64>,
        status_filter: Option<&[String]>,
        limit: i32,
    ) -> Result<PaginatedResponse<TaskModel>, TwolebotError> {
        let conn = self.db.conn()?;
        let total = self.count_tasks_simple(&conn, project_id, status_filter)?;
        let items = self.query_tasks_simple(&conn, project_id, status_filter, limit)?;

        Ok(PaginatedResponse {
            total,
            page: 1,
            limit,
            items,
        })
    }

    /// Build a WHERE clause from optional project_id and status filter.
    /// `col_prefix` is the table alias prefix (e.g. "" or "t.").
    /// Returns (where_clause, has_project_param).
    fn build_task_where(
        project_id: Option<i64>,
        status_filter: Option<&[String]>,
        col_prefix: &str,
    ) -> String {
        let mut conditions = Vec::new();
        if project_id.is_some() {
            conditions.push(format!("{}project_id = ?1", col_prefix));
        }
        if let Some(statuses) = status_filter {
            let quoted: Vec<String> = statuses
                .iter()
                .filter_map(|s| s.parse::<TaskStatus>().ok().map(|ts| format!("'{ts}'")))
                .collect();
            if !quoted.is_empty() {
                conditions.push(format!("{}status IN ({})", col_prefix, quoted.join(",")));
            }
        } else {
            conditions.push(format!("{}status != 'archived'", col_prefix));
        }
        if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        }
    }

    /// Append a LIMIT clause with the correct param index depending on whether
    /// project_id occupies ?1.
    fn append_limit(sql: &mut String, project_id: Option<i64>) {
        if project_id.is_some() {
            sql.push_str(" LIMIT ?2");
        } else {
            sql.push_str(" LIMIT ?1");
        }
    }

    /// Execute a query that takes optional project_id + limit params.
    fn query_with_opt_project<T, F>(
        conn: &rusqlite::Connection,
        sql: &str,
        project_id: Option<i64>,
        limit: i32,
        mapper: F,
        ctx: &str,
    ) -> Result<Vec<T>, TwolebotError>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| TwolebotError::work(format!("prepare {ctx}: {e}")))?;
        let rows = if let Some(pid) = project_id {
            stmt.query_map(params![pid, limit], mapper)
        } else {
            stmt.query_map(params![limit], mapper)
        }
        .map_err(|e| TwolebotError::work(format!("query {ctx}: {e}")))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read {ctx}: {e}")))
    }

    fn count_tasks_simple(
        &self,
        conn: &rusqlite::Connection,
        project_id: Option<i64>,
        status_filter: Option<&[String]>,
    ) -> Result<i64, TwolebotError> {
        let mut sql = "SELECT COUNT(*) FROM tasks".to_string();
        sql.push_str(&Self::build_task_where(project_id, status_filter, ""));
        if let Some(pid) = project_id {
            conn.query_row(&sql, params![pid], |row| row.get(0))
        } else {
            conn.query_row(&sql, [], |row| row.get(0))
        }
        .map_err(|e| TwolebotError::work(format!("count tasks: {e}")))
    }

    fn query_tasks_simple(
        &self,
        conn: &rusqlite::Connection,
        project_id: Option<i64>,
        status_filter: Option<&[String]>,
        limit: i32,
    ) -> Result<Vec<TaskModel>, TwolebotError> {
        let mut sql = "SELECT * FROM tasks".to_string();
        sql.push_str(&Self::build_task_where(project_id, status_filter, ""));
        sql.push_str(" ORDER BY sort_order ASC");
        Self::append_limit(&mut sql, project_id);

        let mut tasks =
            Self::query_with_opt_project(conn, &sql, project_id, limit, row_to_task, "tasks")?;

        for task in &mut tasks {
            let (blocked_by, blocks) = load_task_dependencies(conn, task.id)?;
            task.blocked_by = blocked_by;
            task.blocks = blocks;
        }

        Ok(tasks)
    }

    pub fn list_tasks_compact(
        &self,
        project_id: Option<i64>,
        status_filter: Option<&[String]>,
        limit: i32,
    ) -> Result<PaginatedResponse<TaskSummary>, TwolebotError> {
        let conn = self.db.conn()?;
        let total = self.count_tasks_simple(&conn, project_id, status_filter)?;

        let mut sql = String::from(
            "SELECT t.id, t.status, t.sort_order, t.title,
                    COALESCE((SELECT json_group_array(tag) FROM task_tags WHERE task_id = t.id), '[]') AS tags_json,
                    (SELECT COUNT(*) FROM comments WHERE task_id = t.id) AS comment_count
             FROM tasks t",
        );
        sql.push_str(&Self::build_task_where(project_id, status_filter, "t."));
        sql.push_str(" ORDER BY t.sort_order ASC");
        Self::append_limit(&mut sql, project_id);

        let row_mapper = |row: &rusqlite::Row| {
            let tags_json: String = row.get(4)?;
            let tags: Vec<String> =
                serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(TaskSummary {
                id: row.get(0)?,
                status: row.get::<_, String>(1)?
                    .parse()
                    .unwrap_or(TaskStatus::Todo),
                sort_order: row.get(2)?,
                title: row.get(3)?,
                tags,
                comment_count: row.get(5)?,
            })
        };

        let items = Self::query_with_opt_project(
            &conn, &sql, project_id, limit, row_mapper, "compact",
        )?;

        Ok(PaginatedResponse {
            total,
            page: 1,
            limit,
            items,
        })
    }

    pub fn get_task(&self, id: i64) -> Result<TaskModel, TwolebotError> {
        let conn = self.db.conn()?;
        let mut task = conn
            .query_row("SELECT * FROM tasks WHERE id = ?1", params![id], |row| {
                row_to_task(row)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    TwolebotError::not_found(format!("task {id}"))
                }
                _ => TwolebotError::work(format!("get task {id}: {e}")),
            })?;

        let (blocked_by, blocks) = load_task_dependencies(&conn, id)?;
        task.blocked_by = blocked_by;
        task.blocks = blocks;
        Ok(task)
    }

    pub fn create_task(
        &self,
        project_id: i64,
        title: &str,
        description: &str,
        status: Option<&TaskStatus>,
        priority: Option<&TaskPriority>,
        tags: &[String],
    ) -> Result<TaskModel, TwolebotError> {
        let conn = self.db.conn()?;
        let title = title.trim();
        if title.is_empty() {
            return Err(TwolebotError::work("task title cannot be empty"));
        }

        // Verify project exists
        let _proj = self.get_project(project_id)?;

        // Duplicate title check (case-insensitive within project)
        let dup_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND LOWER(title) = LOWER(?2)",
                params![project_id, title],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("dup check: {e}")))?;
        if dup_count > 0 {
            return Err(TwolebotError::work(format!(
                "a task with title '{title}' already exists in this project"
            )));
        }

        // Calculate next sort_order = max + 10
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) FROM tasks WHERE project_id = ?1",
                params![project_id],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("max order: {e}")))?;
        let sort_order = max_order + 10;

        let status = status.cloned().unwrap_or(TaskStatus::Todo);
        let priority = priority.cloned().unwrap_or(TaskPriority::Medium);
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO tasks (project_id, title, description, task_type, status, priority,
             sort_order, tags_json)
             VALUES (?1, ?2, ?3, 'task', ?4, ?5, ?6, ?7)",
            params![
                project_id,
                title,
                description,
                status.to_string(),
                priority.to_string(),
                sort_order,
                tags_json,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("insert task: {e}")))?;

        let id = conn.last_insert_rowid();

        // Sync tags to normalized table
        sync_task_tags(&conn, id, tags)?;

        // Normalize task order for the project
        self.normalize_task_order(&conn, project_id)?;

        self.log_activity(
            &conn,
            Some(project_id),
            Some(id),
            None,
            &ActivityAction::Created,
            &format!("Task created: {title}"),
        )?;

        drop(conn);
        self.get_task(id)
    }

    pub fn update_task(
        &self,
        id: i64,
        update: &TaskUpdate,
    ) -> Result<TaskModel, TwolebotError> {
        let conn = self.db.conn()?;

        // Verify exists
        let existing = conn
            .query_row("SELECT * FROM tasks WHERE id = ?1", params![id], |row| {
                row_to_task(row)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    TwolebotError::not_found(format!("task {id}"))
                }
                _ => TwolebotError::work(format!("get task for update {id}: {e}")),
            })?;

        let title = update
            .title
            .as_deref()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or(&existing.title);
        let description = update
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .unwrap_or(&existing.description);
        let status = update.status.as_ref().unwrap_or(&existing.status);
        let priority = update.priority.as_ref().unwrap_or(&existing.priority);

        // Require comment when transitioning to ReadyForReview
        if *status == TaskStatus::ReadyForReview && existing.status != TaskStatus::ReadyForReview {
            let comment_text = update
                .comment
                .as_deref()
                .map(|c| c.trim())
                .unwrap_or("");
            if comment_text.is_empty() {
                return Err(TwolebotError::work(
                    "comment required when moving to ready_for_review",
                ));
            }
        }

        let set_completed = *status == TaskStatus::Done && existing.status != TaskStatus::Done;

        conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, status = ?3, priority = ?4,
             completed_at = CASE WHEN ?5 = 1 THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ELSE completed_at END,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE id = ?6",
            params![
                title,
                description,
                status.to_string(),
                priority.to_string(),
                set_completed as i32,
                id,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("update task {id}: {e}")))?;

        // Sync tags if provided
        if let Some(ref tags) = update.tags {
            let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
            conn.execute(
                "UPDATE tasks SET tags_json = ?1 WHERE id = ?2",
                params![tags_json, id],
            )
            .map_err(|e| TwolebotError::work(format!("update tags: {e}")))?;
            sync_task_tags(&conn, id, tags)?;
        }

        // Auto-insert comment when transitioning to ReadyForReview
        if *status == TaskStatus::ReadyForReview && existing.status != TaskStatus::ReadyForReview {
            if let Some(ref comment_text) = update.comment {
                let trimmed = comment_text.trim();
                if !trimmed.is_empty() {
                    conn.execute(
                        "INSERT INTO comments (task_id, content) VALUES (?1, ?2)",
                        params![id, trimmed],
                    )
                    .map_err(|e| {
                        TwolebotError::work(format!("insert ready_for_review comment: {e}"))
                    })?;
                }
            }
        }

        // Log status change specifically
        if *status != existing.status {
            self.log_activity(
                &conn,
                Some(existing.project_id),
                Some(id),
                None,
                &ActivityAction::StatusChanged,
                &format!("Status: {} -> {}", existing.status, status),
            )?;
        } else {
            self.log_activity(
                &conn,
                Some(existing.project_id),
                Some(id),
                None,
                &ActivityAction::Updated,
                &format!("Task updated: {title}"),
            )?;
        }

        drop(conn);
        self.get_task(id)
    }

    /// Find todo tasks tagged `live` that are not already on the live board.
    /// Returns task IDs suitable for passing to `select_tasks()`.
    pub fn find_live_tagged_unselected_tasks(&self) -> Result<Vec<i64>, TwolebotError> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id FROM tasks t
                 INNER JOIN task_tags tt ON tt.task_id = t.id AND tt.tag = 'live'
                 WHERE t.status = 'todo'
                   AND t.id NOT IN (
                     SELECT task_id FROM live_board_selections
                     WHERE status IN ('queued', 'active', 'paused')
                   )
                 ORDER BY
                   CASE t.priority
                     WHEN 'critical' THEN 0
                     WHEN 'high' THEN 1
                     WHEN 'medium' THEN 2
                     WHEN 'low' THEN 3
                     ELSE 4
                   END,
                   t.sort_order ASC",
            )
            .map_err(|e| TwolebotError::work(format!("prepare live tag query: {e}")))?;

        let ids: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| TwolebotError::work(format!("query live tags: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("collect live tags: {e}")))?;

        Ok(ids)
    }

    /// Re-queue a task for a new SDLC phase (e.g., harden after dev).
    /// Resets the task to todo, clears any done/failed selection, and adds a new queued selection.
    pub fn requeue_for_sdlc_phase(
        &self,
        task_id: i64,
        phase_comment: &str,
    ) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;

        // Clean up existing selection (done/failed) for this task
        conn.execute(
            "DELETE FROM live_board_selections WHERE task_id = ?1",
            params![task_id],
        )
        .map_err(|e| TwolebotError::work(format!("clear selection for requeue: {e}")))?;

        // Reset task to todo status (for take_next to pick it up via review path)
        conn.execute(
            "UPDATE tasks SET status = 'todo', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE id = ?1",
            params![task_id],
        )
        .map_err(|e| TwolebotError::work(format!("reset task for requeue: {e}")))?;

        // Add phase comment
        conn.execute(
            "INSERT INTO comments (task_id, content, created_at, updated_at)
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![task_id, phase_comment],
        )
        .map_err(|e| TwolebotError::work(format!("insert phase comment: {e}")))?;

        // Log activity
        let project_id: Option<i64> = conn
            .query_row(
                "SELECT project_id FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .ok();

        conn.execute(
            "INSERT INTO activity_logs (project_id, task_id, action, actor, details, created_at)
             VALUES (?1, ?2, 'status_changed', 'heartbeat', ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![project_id, task_id, phase_comment],
        )
        .map_err(|e| TwolebotError::work(format!("record requeue activity: {e}")))?;

        drop(conn);

        // Re-select onto the board at the top (will be picked up next)
        self.select_tasks(&[task_id])?;

        Ok(())
    }

    // ── Task workflow ───────────────────────────────────────────────────────

    pub fn take_next_task(
        &self,
        project_id: i64,
        force: bool,
    ) -> Result<Option<TakeNextResult>, TwolebotError> {
        let conn = self.db.conn()?;

        // Check for in-progress tasks
        let in_progress = query_tasks_by_status(&conn, project_id, "in_progress")?;

        if !in_progress.is_empty() && !force {
            let task = &in_progress[0];
            let comments = self.get_comments_for_task_conn(&conn, task.id)?;
            let hours = hours_since(&task.updated_at);
            return Ok(Some(TakeNextResult {
                task: task.clone(),
                comments,
                warning: Some(format!(
                    "Task #{} is already in progress for {:.1} hours. Use force=true to override.",
                    task.id, hours
                )),
            }));
        }

        // Check for under-review tasks
        let under_review = query_tasks_by_status(&conn, project_id, "under_review")?;

        if !under_review.is_empty() && !force {
            let task = &under_review[0];
            let comments = self.get_comments_for_task_conn(&conn, task.id)?;
            return Ok(Some(TakeNextResult {
                task: task.clone(),
                comments,
                warning: Some(format!(
                    "Task #{} is under review. Complete or reject first. Use force=true to override.",
                    task.id
                )),
            }));
        }

        // Check for ready-for-review tasks (warning only)
        let ready_for_review = query_tasks_by_status(&conn, project_id, "ready_for_review")?;

        if !ready_for_review.is_empty() && !force {
            let oldest = &ready_for_review[0];
            let hours = hours_since(&oldest.updated_at);
            let count = ready_for_review.len();
            // Still proceed to take a todo task, but add a warning
            // Actually per work2 logic: this returns without taking a task
            let comments = self.get_comments_for_task_conn(&conn, oldest.id)?;
            return Ok(Some(TakeNextResult {
                task: oldest.clone(),
                comments,
                warning: Some(format!(
                    "There are {count} task(s) ready for review. Oldest waiting {hours:.1} hours. Consider reviewing first or use force=true."
                )),
            }));
        }

        // Find next todo task and update atomically in a transaction
        conn.execute("BEGIN IMMEDIATE", [])
            .map_err(|e| TwolebotError::work(format!("begin transaction: {e}")))?;

        let transaction_result = (|| {
            let todo_tasks = query_tasks_by_status(&conn, project_id, "todo")?;
            let todo_task = todo_tasks.into_iter().next();

            match todo_task {
                Some(task) => {
                    // Transition to InProgress
                    conn.execute(
                        "UPDATE tasks SET status = 'in_progress',
                         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                         WHERE id = ?1",
                        params![task.id],
                    )
                    .map_err(|e| TwolebotError::work(format!("set in_progress: {e}")))?;

                    self.log_activity(
                        &conn,
                        Some(project_id),
                        Some(task.id),
                        None,
                        &ActivityAction::StatusChanged,
                        &"Status: todo -> in_progress (take_next_task)".to_string(),
                    )?;

                    let comments = self.get_comments_for_task_conn(&conn, task.id)?;
                    Ok(Some((task.id, comments)))
                }
                None => Ok(None),
            }
        })();

        match transaction_result {
            Ok(Some((task_id, comments))) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| TwolebotError::work(format!("commit: {e}")))?;
                drop(conn);
                let updated_task = self.get_task(task_id)?;
                Ok(Some(TakeNextResult {
                    task: updated_task,
                    comments,
                    warning: None,
                }))
            }
            Ok(None) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| TwolebotError::work(format!("commit: {e}")))?;
                Ok(None)
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    pub fn take_next_review_task(
        &self,
        project_id: i64,
        force: bool,
    ) -> Result<Option<TakeNextResult>, TwolebotError> {
        let conn = self.db.conn()?;

        // Check for already under-review tasks
        let under_review = query_tasks_by_status(&conn, project_id, "under_review")?;

        if !under_review.is_empty() && !force {
            let task = &under_review[0];
            let comments = self.get_comments_for_task_conn(&conn, task.id)?;
            let hours = hours_since(&task.updated_at);
            return Ok(Some(TakeNextResult {
                task: task.clone(),
                comments,
                warning: Some(format!(
                    "Task #{} is already under review for {hours:.1} hours. Complete it first or use force=true.",
                    task.id
                )),
            }));
        }

        // Find next ready-for-review task and update atomically in a transaction
        conn.execute("BEGIN IMMEDIATE", [])
            .map_err(|e| TwolebotError::work(format!("begin transaction: {e}")))?;

        let transaction_result = (|| {
            let review_tasks = query_tasks_by_status(&conn, project_id, "ready_for_review")?;
            let review_task = review_tasks.into_iter().next();

            match review_task {
                Some(task) => {
                    conn.execute(
                        "UPDATE tasks SET status = 'under_review',
                         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                         WHERE id = ?1",
                        params![task.id],
                    )
                    .map_err(|e| TwolebotError::work(format!("set under_review: {e}")))?;

                    self.log_activity(
                        &conn,
                        Some(project_id),
                        Some(task.id),
                        None,
                        &ActivityAction::StatusChanged,
                        &"Status: ready_for_review -> under_review (take_next_review_task)".to_string(),
                    )?;

                    let comments = self.get_comments_for_task_conn(&conn, task.id)?;
                    Ok(Some((task.id, comments)))
                }
                None => Ok(None),
            }
        })();

        match transaction_result {
            Ok(Some((task_id, comments))) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| TwolebotError::work(format!("commit: {e}")))?;
                drop(conn);
                let updated_task = self.get_task(task_id)?;
                Ok(Some(TakeNextResult {
                    task: updated_task,
                    comments,
                    warning: None,
                }))
            }
            Ok(None) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| TwolebotError::work(format!("commit: {e}")))?;
                Ok(None)
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    pub fn move_task_to_top_or_bottom(
        &self,
        task_id: i64,
        position: &str,
    ) -> Result<TaskModel, TwolebotError> {
        let conn = self.db.conn()?;

        let task = conn
            .query_row(
                "SELECT * FROM tasks WHERE id = ?1",
                params![task_id],
                row_to_task,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    TwolebotError::not_found(format!("task {task_id}"))
                }
                _ => TwolebotError::work(format!("get task for move {task_id}: {e}")),
            })?;

        // Get all other tasks in project sorted by order
        let other_tasks: Vec<(i64, i32)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT id, sort_order FROM tasks WHERE project_id = ?1 AND id != ?2
                     ORDER BY sort_order ASC",
                )
                .map_err(|e| TwolebotError::work(format!("query other tasks: {e}")))?;
            let results: Vec<(i64, i32)> = stmt
                .query_map(params![task.project_id, task_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
                })
                .map_err(|e| TwolebotError::work(format!("read other tasks: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("collect other tasks: {e}")))?;
            results
        };

        // Build reordered list
        let ordered_ids: Vec<i64> = match position {
            "top" => {
                let mut ids = vec![task_id];
                ids.extend(other_tasks.iter().map(|(id, _)| *id));
                ids
            }
            "bottom" => {
                let mut ids: Vec<i64> = other_tasks.iter().map(|(id, _)| *id).collect();
                ids.push(task_id);
                ids
            }
            _ => return Err(TwolebotError::work(format!("invalid position: {position}"))),
        };

        // Re-index with increments of 10
        for (idx, tid) in ordered_ids.iter().enumerate() {
            let new_order = ((idx + 1) * 10) as i32;
            conn.execute(
                "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
                params![new_order, tid],
            )
            .map_err(|e| TwolebotError::work(format!("reorder task: {e}")))?;
        }

        drop(conn);
        self.get_task(task_id)
    }

    pub fn task_reject_review(
        &self,
        task_id: i64,
        reviewer_comment: &str,
    ) -> Result<TaskModel, TwolebotError> {
        let conn = self.db.conn()?;

        let task = conn
            .query_row(
                "SELECT * FROM tasks WHERE id = ?1",
                params![task_id],
                row_to_task,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    TwolebotError::not_found(format!("task {task_id}"))
                }
                _ => TwolebotError::work(format!("get task for reject: {e}")),
            })?;

        if task.status != TaskStatus::UnderReview {
            return Err(TwolebotError::work(format!(
                "task #{task_id} is not under review (status: {})",
                task.status
            )));
        }

        // Get minimum order in project to place this at top
        let min_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MIN(sort_order), 10) FROM tasks WHERE project_id = ?1 AND id != ?2",
                params![task.project_id, task_id],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("min order: {e}")))?;

        // Set status back to todo with order before all others
        conn.execute(
            "UPDATE tasks SET status = 'todo', sort_order = ?1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE id = ?2",
            params![min_order - 1, task_id],
        )
        .map_err(|e| TwolebotError::work(format!("reject task: {e}")))?;

        // Add automatic comment
        conn.execute(
            "INSERT INTO comments (task_id, content)
             VALUES (?1, ?2)",
            params![
                task_id,
                format!("Task rejected from review: {reviewer_comment}"),
            ],
        )
        .map_err(|e| TwolebotError::work(format!("insert reject comment: {e}")))?;

        self.log_activity(
            &conn,
            Some(task.project_id),
            Some(task_id),
            None,
            &ActivityAction::ReviewRejected,
            &format!("Review rejected: {reviewer_comment}"),
        )?;

        // Normalize order
        self.normalize_task_order(&conn, task.project_id)?;

        drop(conn);
        self.get_task(task_id)
    }

    // ── Documents ───────────────────────────────────────────────────────────

    pub fn search_documents(
        &self,
        query: &str,
        project_id: Option<i64>,
        limit: i32,
    ) -> Result<PaginatedResponse<Document>, TwolebotError> {
        let conn = self.db.conn()?;

        // Use FTS5 for search
        let (sql, count_sql) = if project_id.is_some() {
            (
                "SELECT d.* FROM documents d
                 JOIN documents_fts fts ON d.id = fts.rowid
                 WHERE fts.documents_fts MATCH ?1 AND d.project_id = ?2 AND d.deleted = 0
                 ORDER BY rank LIMIT ?3",
                "SELECT COUNT(*) FROM documents d
                 JOIN documents_fts fts ON d.id = fts.rowid
                 WHERE fts.documents_fts MATCH ?1 AND d.project_id = ?2 AND d.deleted = 0",
            )
        } else {
            (
                "SELECT d.* FROM documents d
                 JOIN documents_fts fts ON d.id = fts.rowid
                 WHERE fts.documents_fts MATCH ?1 AND d.deleted = 0
                 ORDER BY rank LIMIT ?2",
                "SELECT COUNT(*) FROM documents d
                 JOIN documents_fts fts ON d.id = fts.rowid
                 WHERE fts.documents_fts MATCH ?1 AND d.deleted = 0",
            )
        };

        // FTS5 query: escape special characters and add wildcard
        let fts_query = sanitize_fts_query(query);

        let (total, items) = if let Some(pid) = project_id {
            let total: i64 = conn
                .query_row(count_sql, params![fts_query, pid], |row| row.get(0))
                .unwrap_or(0); // FTS match failures return 0 results

            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| TwolebotError::work(format!("prepare doc search: {e}")))?;
            let items: Vec<Document> = stmt
                .query_map(params![fts_query, pid, limit], row_to_document)
                .map_err(|e| TwolebotError::work(format!("query doc search: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default();
            (total, items)
        } else {
            let total: i64 = conn
                .query_row(count_sql, params![fts_query], |row| row.get(0))
                .unwrap_or(0);

            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| TwolebotError::work(format!("prepare doc search: {e}")))?;
            let items: Vec<Document> = stmt
                .query_map(params![fts_query, limit], row_to_document)
                .map_err(|e| TwolebotError::work(format!("query doc search: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default();
            (total, items)
        };

        // Fallback: if FTS returns nothing, try LIKE search
        if items.is_empty() {
            return self.search_documents_like(&conn, query, project_id, limit);
        }

        Ok(PaginatedResponse {
            total,
            page: 1,
            limit,
            items,
        })
    }

    fn search_documents_like(
        &self,
        conn: &rusqlite::Connection,
        query: &str,
        project_id: Option<i64>,
        limit: i32,
    ) -> Result<PaginatedResponse<Document>, TwolebotError> {
        let like_pattern = format!("%{query}%");

        let (sql, count_sql) = if project_id.is_some() {
            (
                "SELECT * FROM documents WHERE project_id = ?1 AND deleted = 0
                 AND (title LIKE ?2 OR content LIKE ?2) ORDER BY updated_at DESC LIMIT ?3",
                "SELECT COUNT(*) FROM documents WHERE project_id = ?1 AND deleted = 0
                 AND (title LIKE ?2 OR content LIKE ?2)",
            )
        } else {
            (
                "SELECT * FROM documents WHERE deleted = 0
                 AND (title LIKE ?1 OR content LIKE ?1) ORDER BY updated_at DESC LIMIT ?2",
                "SELECT COUNT(*) FROM documents WHERE deleted = 0
                 AND (title LIKE ?1 OR content LIKE ?1)",
            )
        };

        let (total, items) = if let Some(pid) = project_id {
            let total: i64 = conn
                .query_row(count_sql, params![pid, like_pattern], |row| row.get(0))
                .unwrap_or(0);
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| TwolebotError::work(format!("prepare like search: {e}")))?;
            let items: Vec<Document> = stmt
                .query_map(params![pid, like_pattern, limit], |row| {
                    row_to_document(row)
                })
                .map_err(|e| TwolebotError::work(format!("query like search: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default();
            (total, items)
        } else {
            let total: i64 = conn
                .query_row(count_sql, params![like_pattern], |row| row.get(0))
                .unwrap_or(0);
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| TwolebotError::work(format!("prepare like search: {e}")))?;
            let items: Vec<Document> = stmt
                .query_map(params![like_pattern, limit], row_to_document)
                .map_err(|e| TwolebotError::work(format!("query like search: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default();
            (total, items)
        };

        Ok(PaginatedResponse {
            total,
            page: 1,
            limit,
            items,
        })
    }

    pub fn get_document(&self, id: i64) -> Result<Document, TwolebotError> {
        let conn = self.db.conn()?;
        conn.query_row(
            "SELECT * FROM documents WHERE id = ?1 AND deleted = 0",
            params![id],
            row_to_document,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                TwolebotError::not_found(format!("document {id}"))
            }
            _ => TwolebotError::work(format!("get document {id}: {e}")),
        })
    }

    pub fn create_document(
        &self,
        project_id: i64,
        document_type: &DocumentType,
        title: &str,
        content: &str,
    ) -> Result<Document, TwolebotError> {
        let conn = self.db.conn()?;

        let _proj = self.get_project(project_id)?;

        conn.execute(
            "INSERT INTO documents (project_id, document_type, title, content)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                project_id,
                document_type.to_string(),
                title,
                content,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("insert document: {e}")))?;

        let id = conn.last_insert_rowid();

        self.log_activity(
            &conn,
            Some(project_id),
            None,
            Some(id),
            &ActivityAction::Created,
            &format!("Document created: {title}"),
        )?;

        self.get_document(id)
    }

    pub fn update_document(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
        document_type: Option<&DocumentType>,
    ) -> Result<Document, TwolebotError> {
        let conn = self.db.conn()?;

        // Verify exists
        let existing = conn
            .query_row(
                "SELECT * FROM documents WHERE id = ?1 AND deleted = 0",
                params![id],
                row_to_document,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    TwolebotError::not_found(format!("document {id}"))
                }
                _ => TwolebotError::work(format!("get document for update: {e}")),
            })?;

        let title = title.filter(|t| !t.is_empty()).unwrap_or(&existing.title);
        let content = content.unwrap_or(&existing.content);
        let doc_type = document_type.unwrap_or(&existing.document_type);

        conn.execute(
            "UPDATE documents SET title = ?1, content = ?2, document_type = ?3,
             version = version + 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE id = ?4",
            params![
                title,
                content,
                doc_type.to_string(),
                id,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("update document {id}: {e}")))?;

        self.log_activity(
            &conn,
            Some(existing.project_id),
            None,
            Some(id),
            &ActivityAction::Updated,
            &format!("Document updated: {title}"),
        )?;

        self.get_document(id)
    }

    /// Read document content with optional line range. Returns line-numbered text
    /// like `cat -n`. offset is 1-based, limit is number of lines.
    pub fn read_document_content(
        &self,
        id: i64,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<String, TwolebotError> {
        let doc = self.get_document(id)?;
        let lines: Vec<&str> = doc.content.lines().collect();
        let total = lines.len();

        let start = offset.unwrap_or(1).max(1) as usize;
        let end = match limit {
            Some(n) => (start + n as usize - 1).min(total),
            None => total,
        };

        if start > total {
            return Ok(format!("(empty — document has {total} lines)"));
        }

        // Format like `cat -n`: right-aligned line numbers + tab + content
        let width = end.to_string().len();
        let mut out = String::new();
        for i in (start - 1)..end {
            out.push_str(&format!("{:>width$}\t{}\n", i + 1, lines[i], width = width));
        }

        // Append a footer so the caller knows the total size
        out.push_str(&format!(
            "\n--- lines {start}-{end} of {total} ---"
        ));

        Ok(out)
    }

    /// Exact string replacement in document content (like the Edit file tool).
    /// Returns the updated Document. Increments version.
    pub fn edit_document_content(
        &self,
        id: i64,
        old_string: &str,
        new_string: &str,
        replace_all: bool,
    ) -> Result<Document, TwolebotError> {
        let doc = self.get_document(id)?;

        if old_string == new_string {
            return Err(TwolebotError::work(
                "old_string and new_string are identical".to_string(),
            ));
        }

        let count = doc.content.matches(old_string).count();
        if count == 0 {
            return Err(TwolebotError::work(format!(
                "old_string not found in document {}",
                id
            )));
        }
        if count > 1 && !replace_all {
            return Err(TwolebotError::work(format!(
                "old_string found {count} times in document {id}. \
                 Provide more context to make it unique, or set replace_all to true."
            )));
        }

        let new_content = if replace_all {
            doc.content.replace(old_string, new_string)
        } else {
            doc.content.replacen(old_string, new_string, 1)
        };

        self.update_document(id, None, Some(&new_content), None)
    }

    // ── Comments ────────────────────────────────────────────────────────────

    pub fn list_task_comments(
        &self,
        task_id: i64,
        limit: i32,
        page: i32,
    ) -> Result<PaginatedResponse<Comment>, TwolebotError> {
        let conn = self.db.conn()?;

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE task_id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("count comments: {e}")))?;

        let offset = (page.max(1) - 1) * limit;

        let mut stmt = conn
            .prepare(
                "SELECT * FROM comments WHERE task_id = ?1
                 ORDER BY created_at ASC LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| TwolebotError::work(format!("prepare comments: {e}")))?;
        let items: Vec<Comment> = stmt
            .query_map(params![task_id, limit, offset], row_to_comment)
            .map_err(|e| TwolebotError::work(format!("query comments: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read comments: {e}")))?;

        Ok(PaginatedResponse {
            total,
            page,
            limit,
            items,
        })
    }

    pub fn upsert_comment(
        &self,
        comment_id: Option<i64>,
        task_id: Option<i64>,
        document_id: Option<i64>,
        content: &str,
        parent_comment_id: Option<i64>,
    ) -> Result<Comment, TwolebotError> {
        let conn = self.db.conn()?;

        if let Some(cid) = comment_id {
            // Update existing
            conn.execute(
                "UPDATE comments SET content = ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?2",
                params![content, cid],
            )
            .map_err(|e| TwolebotError::work(format!("update comment {cid}: {e}")))?;

            return conn
                .query_row(
                    "SELECT * FROM comments WHERE id = ?1",
                    params![cid],
                    row_to_comment,
                )
                .map_err(|e| TwolebotError::work(format!("get updated comment: {e}")));
        }

        // Create new
        if task_id.is_none() && document_id.is_none() {
            return Err(TwolebotError::work(
                "comment must have either task_id or document_id",
            ));
        }

        conn.execute(
            "INSERT INTO comments (task_id, document_id, parent_comment_id, content)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                task_id,
                document_id,
                parent_comment_id,
                content,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("insert comment: {e}")))?;

        let id = conn.last_insert_rowid();

        // Log activity
        let project_id = if let Some(tid) = task_id {
            conn.query_row(
                "SELECT project_id FROM tasks WHERE id = ?1",
                params![tid],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        } else if let Some(did) = document_id {
            conn.query_row(
                "SELECT project_id FROM documents WHERE id = ?1",
                params![did],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        } else {
            None
        };

        self.log_activity(
            &conn,
            project_id,
            task_id,
            document_id,
            &ActivityAction::Commented,
            "Comment added",
        )?;

        conn.query_row("SELECT * FROM comments WHERE id = ?1", params![id], |row| {
            row_to_comment(row)
        })
        .map_err(|e| TwolebotError::work(format!("get new comment: {e}")))
    }

    // ── Activity ────────────────────────────────────────────────────────────

    pub fn get_recent_activity(
        &self,
        limit: i32,
        project_id: Option<i64>,
    ) -> Result<Vec<ActivityLog>, TwolebotError> {
        let conn = self.db.conn()?;
        if let Some(pid) = project_id {
            let mut stmt = conn
                .prepare(
                    "SELECT * FROM activity_logs WHERE project_id = ?1
                     ORDER BY created_at DESC LIMIT ?2",
                )
                .map_err(|e| TwolebotError::work(format!("prepare activity: {e}")))?;
            let items: Vec<ActivityLog> = stmt
                .query_map(params![pid, limit], row_to_activity_log)
                .map_err(|e| TwolebotError::work(format!("query activity: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("read activity: {e}")))?;
            Ok(items)
        } else {
            let mut stmt = conn
                .prepare("SELECT * FROM activity_logs ORDER BY created_at DESC LIMIT ?1")
                .map_err(|e| TwolebotError::work(format!("prepare activity: {e}")))?;
            let items: Vec<ActivityLog> = stmt
                .query_map(params![limit], row_to_activity_log)
                .map_err(|e| TwolebotError::work(format!("query activity: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("read activity: {e}")))?;
            Ok(items)
        }
    }

    pub fn get_task_status_analytics(
        &self,
        project_id: Option<i64>,
    ) -> Result<TaskAnalytics, TwolebotError> {
        let conn = self.db.conn()?;

        // Status counts
        let raw_counts: Vec<(String, i64)> = if let Some(pid) = project_id {
            let mut stmt = conn
                .prepare("SELECT status, COUNT(*) as count FROM tasks WHERE project_id = ?1 GROUP BY status")
                .map_err(|e| TwolebotError::work(format!("prepare analytics: {e}")))?;
            let rows: Vec<(String, i64)> = stmt
                .query_map(params![pid], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })
                .map_err(|e| TwolebotError::work(format!("query analytics: {e}")))?
                .filter_map(|r| r.ok())
                .collect();
            rows
        } else {
            let mut stmt = conn
                .prepare("SELECT status, COUNT(*) as count FROM tasks GROUP BY status")
                .map_err(|e| TwolebotError::work(format!("prepare analytics: {e}")))?;
            let rows: Vec<(String, i64)> = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })
                .map_err(|e| TwolebotError::work(format!("query analytics: {e}")))?
                .filter_map(|r| r.ok())
                .collect();
            rows
        };

        let status_counts: Vec<StatusCount> = raw_counts
            .into_iter()
            .filter_map(|(status_str, count)| {
                status_str
                    .parse::<TaskStatus>()
                    .ok()
                    .map(|status| StatusCount { status, count })
            })
            .collect();

        // Calculate avg_completion_hours from completed tasks
        let avg_sql = if let Some(pid) = project_id {
            format!(
                "SELECT AVG(
                    (julianday(completed_at) - julianday(created_at)) * 24.0
                ) FROM tasks WHERE completed_at IS NOT NULL AND project_id = {pid}"
            )
        } else {
            "SELECT AVG(
                (julianday(completed_at) - julianday(created_at)) * 24.0
            ) FROM tasks WHERE completed_at IS NOT NULL"
                .to_string()
        };
        let avg_completion_hours: Option<f64> = conn
            .query_row(&avg_sql, [], |row| row.get(0))
            .unwrap_or(None);

        // Calculate throughput_30d (daily completion counts for last 30 days)
        let throughput_sql = if let Some(pid) = project_id {
            format!(
                "SELECT date(completed_at) as d, COUNT(*) as c FROM tasks
                 WHERE completed_at IS NOT NULL
                   AND completed_at >= datetime('now', '-30 days')
                   AND project_id = {pid}
                 GROUP BY d ORDER BY d"
            )
        } else {
            "SELECT date(completed_at) as d, COUNT(*) as c FROM tasks
             WHERE completed_at IS NOT NULL
               AND completed_at >= datetime('now', '-30 days')
             GROUP BY d ORDER BY d"
                .to_string()
        };
        let throughput_30d: Vec<DayCount> = {
            let mut stmt = conn
                .prepare(&throughput_sql)
                .map_err(|e| TwolebotError::work(format!("prepare throughput: {e}")))?;
            let rows: Vec<DayCount> = stmt
                .query_map([], |row| {
                    Ok(DayCount {
                        date: row.get(0)?,
                        count: row.get(1)?,
                    })
                })
                .map_err(|e| TwolebotError::work(format!("query throughput: {e}")))?
                .filter_map(|r| r.ok())
                .collect();
            rows
        };

        Ok(TaskAnalytics {
            status_counts,
            avg_completion_hours,
            throughput_30d,
        })
    }

    // ── Missing service methods ─────────────────────────────────────────────

    pub fn archive_project(&self, id: i64) -> Result<Project, TwolebotError> {
        let conn = self.db.conn()?;
        let _existing = self.get_project(id)?;
        conn.execute(
            "UPDATE projects SET is_active = 0, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| TwolebotError::work(format!("archive project {id}: {e}")))?;
        self.log_activity(
            &conn,
            Some(id),
            None,
            None,
            &ActivityAction::Updated,
            "Project archived",
        )?;
        self.get_project(id)
    }

    pub fn delete_document(&self, id: i64) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        let existing = self.get_document(id)?;
        conn.execute(
            "UPDATE documents SET deleted = 1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| TwolebotError::work(format!("soft-delete document {id}: {e}")))?;
        self.log_activity(
            &conn,
            Some(existing.project_id),
            None,
            Some(id),
            &ActivityAction::Deleted,
            &format!("Document deleted: {}", existing.title),
        )?;
        Ok(())
    }

    pub fn delete_comment(&self, id: i64) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        // Verify exists
        let _comment = conn
            .query_row(
                "SELECT id FROM comments WHERE id = ?1",
                params![id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    TwolebotError::not_found(format!("comment {id}"))
                }
                _ => TwolebotError::work(format!("get comment {id}: {e}")),
            })?;
        conn.execute("DELETE FROM comments WHERE id = ?1", params![id])
            .map_err(|e| TwolebotError::work(format!("delete comment {id}: {e}")))?;
        Ok(())
    }

    pub fn add_dependency(
        &self,
        task_id: i64,
        depends_on_task_id: i64,
    ) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        // Verify both tasks exist
        let _t1 = self.get_task(task_id)?;
        let _t2 = self.get_task(depends_on_task_id)?;

        // Check for cycle
        if would_create_cycle(&conn, task_id, depends_on_task_id)? {
            return Err(TwolebotError::work(format!(
                "adding dependency {task_id} -> {depends_on_task_id} would create a cycle"
            )));
        }

        conn.execute(
            "INSERT OR IGNORE INTO task_dependencies (task_id, depends_on_task_id) VALUES (?1, ?2)",
            params![task_id, depends_on_task_id],
        )
        .map_err(|e| TwolebotError::work(format!("add dependency: {e}")))?;
        Ok(())
    }

    pub fn remove_dependency(
        &self,
        task_id: i64,
        depends_on_task_id: i64,
    ) -> Result<(), TwolebotError> {
        let conn = self.db.conn()?;
        conn.execute(
            "DELETE FROM task_dependencies WHERE task_id = ?1 AND depends_on_task_id = ?2",
            params![task_id, depends_on_task_id],
        )
        .map_err(|e| TwolebotError::work(format!("remove dependency: {e}")))?;
        Ok(())
    }

    pub fn list_document_comments(
        &self,
        document_id: i64,
        limit: i32,
        page: i32,
    ) -> Result<PaginatedResponse<Comment>, TwolebotError> {
        let conn = self.db.conn()?;
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE document_id = ?1",
                params![document_id],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("count doc comments: {e}")))?;

        let offset = (page.max(1) - 1) * limit;
        let mut stmt = conn
            .prepare(
                "SELECT * FROM comments WHERE document_id = ?1
                 ORDER BY created_at ASC LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| TwolebotError::work(format!("prepare doc comments: {e}")))?;
        let items: Vec<Comment> = stmt
            .query_map(params![document_id, limit, offset], |row| {
                row_to_comment(row)
            })
            .map_err(|e| TwolebotError::work(format!("query doc comments: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read doc comments: {e}")))?;

        Ok(PaginatedResponse {
            total,
            page,
            limit,
            items,
        })
    }

    pub fn get_project_by_git_remote(&self, url: &str) -> Result<Option<Project>, TwolebotError> {
        let conn = self.db.conn()?;
        match conn.query_row(
            "SELECT p.*, (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count FROM projects p WHERE p.git_remote_url = ?1 AND p.is_active = 1",
            params![url],
            row_to_project,
        ) {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(TwolebotError::work(format!(
                "get project by git remote: {e}"
            ))),
        }
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    fn normalize_task_order(
        &self,
        conn: &rusqlite::Connection,
        project_id: i64,
    ) -> Result<(), TwolebotError> {
        let mut stmt = conn
            .prepare("SELECT id FROM tasks WHERE project_id = ?1 ORDER BY sort_order ASC")
            .map_err(|e| TwolebotError::work(format!("prepare normalize: {e}")))?;
        let task_ids: Vec<i64> = stmt
            .query_map(params![project_id], |row| row.get(0))
            .map_err(|e| TwolebotError::work(format!("query normalize: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read normalize: {e}")))?;

        for (idx, tid) in task_ids.iter().enumerate() {
            let new_order = ((idx + 1) * 10) as i32;
            conn.execute(
                "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
                params![new_order, tid],
            )
            .map_err(|e| TwolebotError::work(format!("normalize order: {e}")))?;
        }

        Ok(())
    }

    fn get_comments_for_task_conn(
        &self,
        conn: &rusqlite::Connection,
        task_id: i64,
    ) -> Result<Vec<Comment>, TwolebotError> {
        let mut stmt = conn
            .prepare("SELECT * FROM comments WHERE task_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| TwolebotError::work(format!("prepare comments: {e}")))?;
        let results: Vec<Comment> = stmt
            .query_map(params![task_id], row_to_comment)
            .map_err(|e| TwolebotError::work(format!("query comments: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read comments: {e}")))?;
        Ok(results)
    }

    fn log_activity(
        &self,
        conn: &rusqlite::Connection,
        project_id: Option<i64>,
        task_id: Option<i64>,
        document_id: Option<i64>,
        action: &ActivityAction,
        details: &str,
    ) -> Result<(), TwolebotError> {
        conn.execute(
            "INSERT INTO activity_logs (project_id, task_id, document_id, action, actor, details)
             VALUES (?1, ?2, ?3, ?4, 'system', ?5)",
            params![
                project_id,
                task_id,
                document_id,
                action.to_string(),
                details,
            ],
        )
        .map_err(|e| TwolebotError::work(format!("insert activity: {e}")))?;
        Ok(())
    }
}

/// Calculate hours since an ISO timestamp
fn hours_since(timestamp: &str) -> f64 {
    use chrono::{DateTime, Utc};
    let parsed = timestamp
        .parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now());
    let duration = Utc::now() - parsed;
    duration.num_minutes() as f64 / 60.0
}

/// Sanitize a query for FTS5 (escape special characters)
fn sanitize_fts_query(query: &str) -> String {
    // FTS5 special characters: AND OR NOT ( ) * "
    // Wrap each token in quotes to treat as literal
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|token| {
            let cleaned = token.replace('"', "");
            if cleaned.is_empty() {
                return String::new();
            }
            format!("\"{cleaned}\"")
        })
        .filter(|s| !s.is_empty())
        .collect();
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    fn test_service() -> (WorkService, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = WorkDb::open(dir.path()).unwrap();
        (WorkService::new(db), dir)
    }

    /// Create n tasks in a project, returns their ids in creation order.
    fn create_n_tasks(svc: &WorkService, project_id: i64, n: usize) -> Vec<i64> {
        (0..n)
            .map(|i| {
                svc.create_task(project_id, &format!("Task {i}"), "", None, None, &[])
                    .unwrap()
                    .id
            })
            .collect()
    }

    /// Read back task ids in their current sort order.
    fn task_ids_in_order(svc: &WorkService, project_id: i64) -> Vec<i64> {
        svc.list_tasks(Some(project_id), None, 100)
            .unwrap()
            .items
            .iter()
            .map(|t| t.id)
            .collect()
    }

    /// Read back sort_orders in their current order.
    fn task_sort_orders(svc: &WorkService, project_id: i64) -> Vec<i32> {
        svc.list_tasks(Some(project_id), None, 100)
            .unwrap()
            .items
            .iter()
            .map(|t| t.sort_order)
            .collect()
    }

    proptest! {
        /// After any sequence of move operations, sort_orders are always
        /// contiguous multiples of 10 and the moved item ends up at the
        /// correct position.
        #[test]
        fn prop_task_ordering_invariants(
            n in 2..8usize,
            ops in proptest::collection::vec(
                (0..8usize, prop::bool::ANY),  // (index % n, is_top)
                1..10,
            ),
        ) {
            let (svc, _dir) = test_service();
            let proj = svc.create_project("P", "", &[], None).unwrap();
            let ids = create_n_tasks(&svc, proj.id, n);

            for (idx, is_top) in &ops {
                let task_id = ids[idx % n];
                let pos = if *is_top { "top" } else { "bottom" };
                svc.move_task_to_top_or_bottom(task_id, pos).unwrap();
            }

            let orders = task_sort_orders(&svc, proj.id);
            let result_ids = task_ids_in_order(&svc, proj.id);

            // Invariant 1: sort_orders are contiguous 10, 20, 30, ...
            let expected_orders: Vec<i32> = (1..=orders.len())
                .map(|i| (i * 10) as i32)
                .collect();
            prop_assert_eq!(&orders, &expected_orders,
                "sort_orders must be contiguous multiples of 10");

            // Invariant 2: no duplicates, no missing ids
            let mut sorted_ids = result_ids.clone();
            sorted_ids.sort();
            let mut orig_ids = ids.clone();
            orig_ids.sort();
            prop_assert_eq!(&sorted_ids, &orig_ids,
                "all original task ids must be present");

            // Invariant 3: last operation's target is at correct position
            if let Some((idx, is_top)) = ops.last() {
                let moved_id = ids[idx % n];
                if *is_top {
                    prop_assert_eq!(result_ids[0], moved_id,
                        "move-to-top should place item first");
                } else {
                    prop_assert_eq!(*result_ids.last().unwrap(), moved_id,
                        "move-to-bottom should place item last");
                }
            }
        }
    }

    #[test]
    fn test_project_crud() {
        let (svc, _dir) = test_service();

        // Create
        let p = svc
            .create_project("Test Project", "A test", &["rust".into()], None)
            .unwrap();
        assert_eq!(p.name, "Test Project");
        assert_eq!(p.tags, vec!["rust"]);

        // Get
        let p2 = svc.get_project(p.id).unwrap();
        assert_eq!(p2.name, "Test Project");

        // List
        let list = svc.list_projects(true, 10, None).unwrap();
        assert_eq!(list.items.len(), 1);

        // Update
        let p3 = svc
            .update_project(p.id, "Updated", Some("new desc"), None, None)
            .unwrap();
        assert_eq!(p3.name, "Updated");
    }

    #[test]
    fn test_task_crud_and_ordering() {
        let (svc, _dir) = test_service();
        let proj = svc.create_project("P1", "", &[], None).unwrap();

        // Create tasks
        let _t1 = svc
            .create_task(proj.id, "Task 1", "", None, None, &[])
            .unwrap();
        let _t2 = svc
            .create_task(proj.id, "Task 2", "", None, None, &[])
            .unwrap();
        let t3 = svc
            .create_task(proj.id, "Task 3", "", None, None, &[])
            .unwrap();

        // Verify ordering (normalized to 10, 20, 30)
        let tasks = svc.list_tasks(Some(proj.id), None, 50).unwrap();
        assert_eq!(tasks.items.len(), 3);
        assert_eq!(tasks.items[0].sort_order, 10);
        assert_eq!(tasks.items[1].sort_order, 20);
        assert_eq!(tasks.items[2].sort_order, 30);

        // Move task 3 to top
        svc.move_task_to_top_or_bottom(t3.id, "top").unwrap();
        let tasks = svc.list_tasks(Some(proj.id), None, 50).unwrap();
        assert_eq!(tasks.items[0].id, t3.id);

        // Duplicate title check
        let dup = svc.create_task(proj.id, "Task 1", "", None, None, &[]);
        assert!(dup.is_err());
    }

    #[test]
    fn test_take_next_task_workflow() {
        let (svc, _dir) = test_service();
        let proj = svc.create_project("P1", "", &[], None).unwrap();

        let _t1 = svc
            .create_task(proj.id, "First", "", None, None, &[])
            .unwrap();

        // Take next should transition to in_progress
        let result = svc.take_next_task(proj.id, false).unwrap().unwrap();
        assert_eq!(result.task.status, TaskStatus::InProgress);
        assert!(result.warning.is_none());

        // Taking again without force should warn
        let result2 = svc.take_next_task(proj.id, false).unwrap().unwrap();
        assert!(result2.warning.is_some());
    }

    #[test]
    fn test_reject_review() {
        let (svc, _dir) = test_service();
        let proj = svc.create_project("P1", "", &[], None).unwrap();

        let t1 = svc
            .create_task(proj.id, "Review Me", "", None, None, &[])
            .unwrap();

        // Set to under_review
        svc.update_task(
            t1.id,
            &TaskUpdate {
                status: Some(TaskStatus::UnderReview),
                ..Default::default()
            },
        )
        .unwrap();

        // Reject
        let rejected = svc
            .task_reject_review(t1.id, "Needs more tests")
            .unwrap();
        assert_eq!(rejected.status, TaskStatus::Todo);

        // Check comment was added
        let comments = svc.list_task_comments(t1.id, 50, 1).unwrap();
        assert!(comments
            .items
            .iter()
            .any(|c| c.content.contains("Needs more tests")));
    }

    #[test]
    fn test_document_crud() {
        let (svc, _dir) = test_service();
        let proj = svc.create_project("P1", "", &[], None).unwrap();

        let doc = svc
            .create_document(
                proj.id,
                &DocumentType::Plan,
                "My Plan",
                "Content here",
            )
            .unwrap();
        assert_eq!(doc.title, "My Plan");
        assert_eq!(doc.version, 1);

        let updated = svc
            .update_document(
                doc.id,
                Some("Updated Plan"),
                Some("New content"),
                None,
            )
            .unwrap();
        assert_eq!(updated.version, 2);
        assert_eq!(updated.title, "Updated Plan");
    }

    #[test]
    fn test_comment_crud() {
        let (svc, _dir) = test_service();
        let proj = svc.create_project("P1", "", &[], None).unwrap();
        let task = svc
            .create_task(proj.id, "T1", "", None, None, &[])
            .unwrap();

        // Create comment
        let c1 = svc
            .upsert_comment(None, Some(task.id), None, "Hello", None)
            .unwrap();
        assert_eq!(c1.content, "Hello");

        // Update comment
        let c2 = svc
            .upsert_comment(Some(c1.id), Some(task.id), None, "Updated", None)
            .unwrap();
        assert_eq!(c2.content, "Updated");
        assert_eq!(c2.id, c1.id);

        // List
        let list = svc.list_task_comments(task.id, 50, 1).unwrap();
        assert_eq!(list.items.len(), 1);
    }
}
