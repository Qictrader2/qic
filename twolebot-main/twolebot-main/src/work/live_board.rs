use rusqlite::params;

use crate::TwolebotError;

use super::models::*;
use super::queries::*;
use super::service::WorkService;

impl WorkService {
    // ── Live board queries ──────────────────────────────────────────────────

    /// Get the live board: backlog (todo tasks not selected) + selected queue.
    pub fn get_live_board(&self, backlog_limit: Option<i32>) -> Result<LiveBoard, TwolebotError> {
        let conn = self.db().conn()?;
        let limit = backlog_limit.unwrap_or(100);

        // Backlog: todo tasks NOT in the active selection queue (queued/active/paused)
        let mut backlog_stmt = conn
            .prepare(
                "SELECT t.*, COALESCE(
                    (SELECT json_group_array(tag) FROM task_tags WHERE task_id = t.id),
                    '[]'
                ) AS tags_json
                FROM tasks t
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
                  t.sort_order ASC
                LIMIT ?1",
            )
            .map_err(|e| TwolebotError::work(format!("prepare backlog query: {e}")))?;

        let backlog: Vec<TaskModel> = backlog_stmt
            .query_map(params![limit], row_to_task)
            .map_err(|e| TwolebotError::work(format!("query backlog: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read backlog: {e}")))?;

        // Selections with their tasks and comments
        let mut sel_stmt = conn
            .prepare(
                "SELECT s.* FROM live_board_selections s
                ORDER BY
                  CASE s.status
                    WHEN 'active' THEN 0
                    WHEN 'paused' THEN 1
                    WHEN 'queued' THEN 2
                    WHEN 'done' THEN 3
                    WHEN 'failed' THEN 4
                    ELSE 5
                  END,
                  s.sort_order ASC",
            )
            .map_err(|e| TwolebotError::work(format!("prepare selections query: {e}")))?;

        let selections: Vec<LiveBoardSelection> = sel_stmt
            .query_map([], row_to_live_board_selection)
            .map_err(|e| TwolebotError::work(format!("query selections: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read selections: {e}")))?;

        // Build SelectedTask entries with task + comments
        let mut selected = Vec::new();
        for sel in &selections {
            let task = conn
                .query_row(
                    "SELECT *, COALESCE(
                        (SELECT json_group_array(tag) FROM task_tags WHERE task_id = tasks.id),
                        '[]'
                    ) AS tags_json
                    FROM tasks WHERE id = ?1",
                    params![sel.task_id],
                    row_to_task,
                )
                .map_err(|e| {
                    TwolebotError::work(format!("load task {} for selection: {e}", sel.task_id))
                })?;

            let mut comment_stmt = conn
                .prepare("SELECT * FROM comments WHERE task_id = ?1 ORDER BY created_at ASC")
                .map_err(|e| TwolebotError::work(format!("prepare comments query: {e}")))?;

            let comments: Vec<Comment> = comment_stmt
                .query_map(params![sel.task_id], row_to_comment)
                .map_err(|e| TwolebotError::work(format!("query comments: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("read comments: {e}")))?;

            selected.push(SelectedTask {
                selection: sel.clone(),
                task,
                comments,
            });
        }

        // Compute stats
        let total_backlog =
            conn.query_row(
                "SELECT COUNT(*) FROM tasks
                WHERE status = 'todo'
                  AND id NOT IN (
                    SELECT task_id FROM live_board_selections
                    WHERE status IN ('queued', 'active', 'paused')
                  )",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| TwolebotError::work(format!("count backlog: {e}")))? as usize;

        let queued = selections
            .iter()
            .filter(|s| s.status == SelectionStatus::Queued)
            .count();
        let completed = selections
            .iter()
            .filter(|s| s.status == SelectionStatus::Done)
            .count();
        let failed = selections
            .iter()
            .filter(|s| s.status == SelectionStatus::Failed)
            .count();
        let active_task_id = selections
            .iter()
            .find(|s| s.status == SelectionStatus::Active)
            .map(|s| s.task_id);

        // Get agent loop state (blocking read of the mutex isn't ideal, but
        // we're already on a blocking thread via spawn_blocking)
        let agent_state = {
            let state = self.agent_loop_state().clone();
            // Use try_lock since we're in sync context
            let result = match state.try_lock() {
                Ok(guard) => guard.clone(),
                Err(_) => AgentLoopState::Idle,
            };
            result
        };

        let stats = LiveBoardStats {
            total_backlog,
            total_selected: selections.len(),
            queued,
            completed,
            failed,
            active: active_task_id,
            agent_loop_state: agent_state,
        };

        Ok(LiveBoard {
            backlog,
            selected,
            stats,
        })
    }

    // ── Selection management ────────────────────────────────────────────────

    /// Add tasks to the agent selection queue.
    /// Validates: tasks exist, are in 'todo' status, not already selected.
    pub fn select_tasks(&self, task_ids: &[i64]) -> Result<Vec<LiveBoardSelection>, TwolebotError> {
        let conn = self.db().conn()?;

        // Get current max sort_order
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) FROM live_board_selections
                 WHERE status IN ('queued', 'active', 'paused')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("get max sort_order: {e}")))?;

        let mut results = Vec::new();
        let mut order = max_order;

        for &task_id in task_ids {
            // Validate task exists and is in todo status
            let status: String = conn
                .query_row(
                    "SELECT status FROM tasks WHERE id = ?1",
                    params![task_id],
                    |row| row.get(0),
                )
                .map_err(|_| TwolebotError::not_found(format!("task {task_id} not found")))?;

            if status != "todo" {
                return Err(TwolebotError::work(format!(
                    "task {task_id} has status '{status}', must be 'todo' to select"
                )));
            }

            // Check not already in active selection
            let already_selected: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM live_board_selections
                     WHERE task_id = ?1 AND status IN ('queued', 'active', 'paused')",
                    params![task_id],
                    |row| row.get(0),
                )
                .map_err(|e| TwolebotError::work(format!("check existing selection: {e}")))?;

            if already_selected {
                return Err(TwolebotError::work(format!(
                    "task {task_id} is already in the selection queue"
                )));
            }

            order += 10;
            conn.execute(
                "INSERT INTO live_board_selections (task_id, sort_order, selected_at, status)
                 VALUES (?1, ?2, datetime('now'), 'queued')",
                params![task_id, order],
            )
            .map_err(|e| TwolebotError::work(format!("insert selection: {e}")))?;

            let sel = conn
                .query_row(
                    "SELECT * FROM live_board_selections WHERE task_id = ?1 AND status = 'queued'
                     ORDER BY id DESC LIMIT 1",
                    params![task_id],
                    row_to_live_board_selection,
                )
                .map_err(|e| TwolebotError::work(format!("read new selection: {e}")))?;

            results.push(sel);
        }

        // Broadcast selection update
        self.broadcast_selection_stats(&conn);

        Ok(results)
    }

    /// Remove a task from the selection queue.
    /// Cannot deselect an 'active' task (must /stop first).
    pub fn deselect_task(&self, task_id: i64) -> Result<(), TwolebotError> {
        let conn = self.db().conn()?;

        let status: String = conn
            .query_row(
                "SELECT status FROM live_board_selections
                 WHERE task_id = ?1 AND status IN ('queued', 'active', 'paused')
                 ORDER BY id DESC LIMIT 1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|_| {
                TwolebotError::not_found(format!("task {task_id} is not in the selection queue"))
            })?;

        if status == "active" {
            return Err(TwolebotError::work(
                "cannot deselect an active task; use /stop first to pause the agent loop"
                    .to_string(),
            ));
        }

        conn.execute(
            "DELETE FROM live_board_selections
             WHERE task_id = ?1 AND status IN ('queued', 'paused')",
            params![task_id],
        )
        .map_err(|e| TwolebotError::work(format!("delete selection: {e}")))?;

        self.broadcast_selection_stats(&conn);

        Ok(())
    }

    /// Move a selection to the top or bottom of the queue.
    pub fn move_selection(
        &self,
        task_id: i64,
        position: &str,
    ) -> Result<(), TwolebotError> {
        let conn = self.db().conn()?;

        // Verify the selection exists and is in a moveable state
        let sel_id: i64 = conn
            .query_row(
                "SELECT id FROM live_board_selections
                 WHERE task_id = ?1 AND status IN ('queued', 'paused')
                 ORDER BY id DESC LIMIT 1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|_| {
                TwolebotError::not_found(format!(
                    "task {task_id} is not in a moveable selection state"
                ))
            })?;

        // Get all other reorderable selections
        let other_ids: Vec<i64> = {
            let mut stmt = conn
                .prepare(
                    "SELECT id FROM live_board_selections
                     WHERE status IN ('queued', 'paused') AND id != ?1
                     ORDER BY sort_order ASC",
                )
                .map_err(|e| TwolebotError::work(format!("query other selections: {e}")))?;
            let results = stmt
                .query_map(params![sel_id], |row| row.get(0))
                .map_err(|e| TwolebotError::work(format!("read other selections: {e}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TwolebotError::work(format!("collect other selections: {e}")))?;
            results
        };

        let ordered_ids: Vec<i64> = match position {
            "top" => {
                let mut ids = vec![sel_id];
                ids.extend(&other_ids);
                ids
            }
            "bottom" => {
                let mut ids = other_ids;
                ids.push(sel_id);
                ids
            }
            _ => return Err(TwolebotError::work(format!("invalid position: {position}"))),
        };

        for (idx, id) in ordered_ids.iter().enumerate() {
            let new_order = ((idx + 1) * 10) as i32;
            conn.execute(
                "UPDATE live_board_selections SET sort_order = ?1 WHERE id = ?2",
                params![new_order, id],
            )
            .map_err(|e| TwolebotError::work(format!("reorder selection: {e}")))?;
        }

        Ok(())
    }

    /// Clear completed/failed selections. Returns count removed.
    pub fn clear_completed_selections(&self) -> Result<i32, TwolebotError> {
        let conn = self.db().conn()?;

        let count = conn
            .execute(
                "DELETE FROM live_board_selections WHERE status IN ('done', 'failed')",
                [],
            )
            .map_err(|e| TwolebotError::work(format!("clear completed selections: {e}")))?;

        self.broadcast_selection_stats(&conn);

        Ok(count as i32)
    }

    /// Select the highest-priority todo task from the live-board backlog.
    /// Returns the created selection, or None when no backlog task is available.
    pub fn select_next_todo_for_agent(&self) -> Result<Option<LiveBoardSelection>, TwolebotError> {
        let conn = self.db().conn()?;

        let next_task_id: Option<i64> = conn
            .query_row(
                "SELECT t.id
                 FROM tasks t
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
                   t.sort_order ASC
                 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        let Some(task_id) = next_task_id else {
            return Ok(None);
        };

        let mut selections = self.select_tasks(&[task_id])?;
        Ok(selections.pop())
    }

    // ── Agent execution interface ───────────────────────────────────────────

    /// Get the next task to work on.
    /// Returns first 'paused' selection (resume priority), or first 'queued' by sort_order.
    pub fn get_next_selected_task(&self) -> Result<Option<SelectedTask>, TwolebotError> {
        let conn = self.db().conn()?;

        // Paused tasks get resume priority, then queued by sort_order
        let sel = conn.query_row(
            "SELECT * FROM live_board_selections
                 WHERE status IN ('paused', 'queued')
                 ORDER BY
                   CASE status WHEN 'paused' THEN 0 WHEN 'queued' THEN 1 ELSE 2 END,
                   sort_order ASC
                 LIMIT 1",
            [],
            row_to_live_board_selection,
        );

        let sel = match sel {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(TwolebotError::work(format!("get next selection: {e}"))),
        };

        // Load the task
        let task = conn
            .query_row(
                "SELECT *, COALESCE(
                    (SELECT json_group_array(tag) FROM task_tags WHERE task_id = tasks.id),
                    '[]'
                ) AS tags_json
                FROM tasks WHERE id = ?1",
                params![sel.task_id],
                row_to_task,
            )
            .map_err(|e| TwolebotError::work(format!("load task for selection: {e}")))?;

        // Load comments
        let mut comment_stmt = conn
            .prepare("SELECT * FROM comments WHERE task_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| TwolebotError::work(format!("prepare comments: {e}")))?;

        let comments: Vec<Comment> = comment_stmt
            .query_map(params![sel.task_id], row_to_comment)
            .map_err(|e| TwolebotError::work(format!("query comments: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read comments: {e}")))?;

        Ok(Some(SelectedTask {
            selection: sel,
            task,
            comments,
        }))
    }

    /// Start working on a selected task.
    /// Sets selection status to 'active', started_at to now, task status to 'in_progress'.
    /// Fails if another selection is already 'active'.
    pub fn start_selected_task(&self, task_id: i64) -> Result<SelectedTask, TwolebotError> {
        let conn = self.db().conn()?;

        // Check no other selection is active
        let active_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM live_board_selections WHERE status = 'active'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| TwolebotError::work(format!("check active count: {e}")))?;

        if active_count > 0 {
            return Err(TwolebotError::work(
                "another task is already active in the selection queue".to_string(),
            ));
        }

        // Update selection
        let updated = conn
            .execute(
                "UPDATE live_board_selections
                 SET status = 'active', started_at = datetime('now')
                 WHERE task_id = ?1 AND status IN ('queued', 'paused')",
                params![task_id],
            )
            .map_err(|e| TwolebotError::work(format!("start selection: {e}")))?;

        if updated == 0 {
            return Err(TwolebotError::not_found(format!(
                "task {task_id} is not in the selection queue"
            )));
        }

        // Update task status to in_progress
        conn.execute(
            "UPDATE tasks SET status = 'in_progress', updated_at = datetime('now')
             WHERE id = ?1",
            params![task_id],
        )
        .map_err(|e| TwolebotError::work(format!("update task status: {e}")))?;

        // Record activity
        let project_id: Option<i64> = conn
            .query_row(
                "SELECT project_id FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .ok();

        conn.execute(
            "INSERT INTO activity_logs (project_id, task_id, action, actor, details, created_at)
             VALUES (?1, ?2, 'status_changed', 'agent', 'Agent started working on task', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![project_id, task_id],
        )
        .map_err(|e| TwolebotError::work(format!("record activity: {e}")))?;

        self.broadcast_selection_stats(&conn);

        // Return the selected task
        let sel = conn
            .query_row(
                "SELECT * FROM live_board_selections
                 WHERE task_id = ?1 AND status = 'active'
                 ORDER BY id DESC LIMIT 1",
                params![task_id],
                row_to_live_board_selection,
            )
            .map_err(|e| TwolebotError::work(format!("read updated selection: {e}")))?;

        let task = conn
            .query_row(
                "SELECT *, COALESCE(
                    (SELECT json_group_array(tag) FROM task_tags WHERE task_id = tasks.id),
                    '[]'
                ) AS tags_json
                FROM tasks WHERE id = ?1",
                params![task_id],
                row_to_task,
            )
            .map_err(|e| TwolebotError::work(format!("load task: {e}")))?;

        let mut comment_stmt = conn
            .prepare("SELECT * FROM comments WHERE task_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| TwolebotError::work(format!("prepare comments: {e}")))?;

        let comments: Vec<Comment> = comment_stmt
            .query_map(params![task_id], row_to_comment)
            .map_err(|e| TwolebotError::work(format!("query comments: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TwolebotError::work(format!("read comments: {e}")))?;

        Ok(SelectedTask {
            selection: sel,
            task,
            comments,
        })
    }

    /// Mark a selected task as completed.
    /// Sets selection status to 'done', completed_at to now, task status to given status.
    pub fn complete_selected_task(
        &self,
        task_id: i64,
        set_task_status: &TaskStatus,
    ) -> Result<(), TwolebotError> {
        let conn = self.db().conn()?;

        let updated = conn
            .execute(
                "UPDATE live_board_selections
                 SET status = 'done', completed_at = datetime('now')
                 WHERE task_id = ?1 AND status = 'active'",
                params![task_id],
            )
            .map_err(|e| TwolebotError::work(format!("complete selection: {e}")))?;

        if updated == 0 {
            return Err(TwolebotError::not_found(format!(
                "task {task_id} has no active selection"
            )));
        }

        // Update task status
        let completed_at = if *set_task_status == TaskStatus::Done {
            "datetime('now')"
        } else {
            "NULL"
        };

        conn.execute(
            &format!(
                "UPDATE tasks SET status = ?1, completed_at = {completed_at}, updated_at = datetime('now')
                 WHERE id = ?2"
            ),
            params![set_task_status.to_string(), task_id],
        )
        .map_err(|e| TwolebotError::work(format!("update task status: {e}")))?;

        // Record activity
        let project_id: Option<i64> = conn
            .query_row(
                "SELECT project_id FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .ok();

        conn.execute(
            "INSERT INTO activity_logs (project_id, task_id, action, actor, details, created_at)
             VALUES (?1, ?2, 'status_changed', 'agent', ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![
                project_id,
                task_id,
                format!("Agent completed task, status set to {set_task_status}")
            ],
        )
        .map_err(|e| TwolebotError::work(format!("record activity: {e}")))?;

        self.broadcast_selection_stats(&conn);

        Ok(())
    }

    /// Mark a selected task as failed.
    /// Sets selection status to 'failed', adds comment with reason.
    /// Does NOT change task status (leaves as in_progress for manual review).
    pub fn fail_selected_task(&self, task_id: i64, reason: &str) -> Result<(), TwolebotError> {
        let conn = self.db().conn()?;

        let updated = conn
            .execute(
                "UPDATE live_board_selections
                 SET status = 'failed', completed_at = datetime('now')
                 WHERE task_id = ?1 AND status = 'active'",
                params![task_id],
            )
            .map_err(|e| TwolebotError::work(format!("fail selection: {e}")))?;

        if updated == 0 {
            return Err(TwolebotError::not_found(format!(
                "task {task_id} has no active selection"
            )));
        }

        // Add failure comment
        conn.execute(
            "INSERT INTO comments (task_id, content, created_at, updated_at)
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            params![task_id, format!("Agent task failed: {reason}")],
        )
        .map_err(|e| TwolebotError::work(format!("add failure comment: {e}")))?;

        // Record activity
        let project_id: Option<i64> = conn
            .query_row(
                "SELECT project_id FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .ok();

        conn.execute(
            "INSERT INTO activity_logs (project_id, task_id, action, actor, details, created_at)
             VALUES (?1, ?2, 'agent_task_failed', 'agent', ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![project_id, task_id, format!("Task failed: {reason}")],
        )
        .map_err(|e| TwolebotError::work(format!("record activity: {e}")))?;

        self.broadcast_selection_stats(&conn);

        Ok(())
    }

    /// Pause a selected task.
    /// Sets selection status to 'paused', task status stays in_progress.
    /// Called when owner sends /stop during active execution.
    pub fn pause_selected_task(&self, task_id: i64) -> Result<(), TwolebotError> {
        let conn = self.db().conn()?;

        let updated = conn
            .execute(
                "UPDATE live_board_selections
                 SET status = 'paused'
                 WHERE task_id = ?1 AND status = 'active'",
                params![task_id],
            )
            .map_err(|e| TwolebotError::work(format!("pause selection: {e}")))?;

        if updated == 0 {
            return Err(TwolebotError::not_found(format!(
                "task {task_id} has no active selection"
            )));
        }

        self.broadcast_selection_stats(&conn);

        Ok(())
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    /// Broadcast current selection stats to SSE subscribers.
    fn broadcast_selection_stats(&self, conn: &rusqlite::Connection) {
        let stats = self.compute_selection_stats(conn);
        if let Ok(stats) = stats {
            self.broadcast(WorkEvent::SelectionUpdated(stats));
        }
    }

    /// Compute current selection stats from the database.
    fn compute_selection_stats(
        &self,
        conn: &rusqlite::Connection,
    ) -> Result<LiveBoardStats, TwolebotError> {
        let total_backlog: usize =
            conn.query_row(
                "SELECT COUNT(*) FROM tasks
                 WHERE status = 'todo'
                   AND id NOT IN (
                     SELECT task_id FROM live_board_selections
                     WHERE status IN ('queued', 'active', 'paused')
                   )",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| TwolebotError::work(format!("count backlog: {e}")))? as usize;

        let total_selected: usize =
            conn.query_row("SELECT COUNT(*) FROM live_board_selections", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|e| TwolebotError::work(format!("count selected: {e}")))? as usize;

        let queued: usize =
            conn.query_row(
                "SELECT COUNT(*) FROM live_board_selections WHERE status = 'queued'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| TwolebotError::work(format!("count queued: {e}")))? as usize;

        let completed: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM live_board_selections WHERE status = 'done'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| TwolebotError::work(format!("count completed: {e}")))?
            as usize;

        let failed: usize =
            conn.query_row(
                "SELECT COUNT(*) FROM live_board_selections WHERE status = 'failed'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| TwolebotError::work(format!("count failed: {e}")))? as usize;

        let active: Option<i64> = conn
            .query_row(
                "SELECT task_id FROM live_board_selections WHERE status = 'active' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        let agent_state = match self.agent_loop_state().try_lock() {
            Ok(guard) => guard.clone(),
            Err(_) => AgentLoopState::Idle,
        };

        Ok(LiveBoardStats {
            total_backlog,
            total_selected,
            queued,
            completed,
            failed,
            active,
            agent_loop_state: agent_state,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::work::db::WorkDb;
    use proptest::prelude::*;
    use tempfile::TempDir;

    fn test_service() -> (WorkService, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = WorkDb::open(dir.path()).unwrap();
        (WorkService::new(db), dir)
    }

    /// Create n tasks, select them all onto the live board.
    /// Returns task ids in creation order.
    fn setup_selected_tasks(svc: &WorkService, n: usize) -> Vec<i64> {
        let project = svc.create_project("Test", "", &[], None).unwrap();
        let task_ids: Vec<i64> = (0..n)
            .map(|i| {
                svc.create_task(project.id, &format!("T{i}"), "", None, None, &[])
                    .unwrap()
                    .id
            })
            .collect();
        svc.select_tasks(&task_ids).unwrap();
        task_ids
    }

    /// Read back selected task ids in their current sort order.
    fn selected_task_ids_in_order(svc: &WorkService) -> Vec<i64> {
        let board = svc.get_live_board(None).unwrap();
        let mut selected = board.selected;
        selected.sort_by_key(|s| s.selection.sort_order);
        selected.iter().map(|s| s.task.id).collect()
    }

    /// Read back selection sort_orders in order.
    fn selection_sort_orders(svc: &WorkService) -> Vec<i32> {
        let board = svc.get_live_board(None).unwrap();
        let mut selected = board.selected;
        selected.sort_by_key(|s| s.selection.sort_order);
        selected.iter().map(|s| s.selection.sort_order).collect()
    }

    proptest! {
        /// After any sequence of move operations on live board selections,
        /// sort_orders are contiguous multiples of 10 and the moved item
        /// ends up at the correct position.
        #[test]
        fn prop_selection_ordering_invariants(
            n in 2..8usize,
            ops in proptest::collection::vec(
                (0..8usize, prop::bool::ANY),  // (index % n, is_top)
                1..10,
            ),
        ) {
            let (svc, _dir) = test_service();
            let task_ids = setup_selected_tasks(&svc, n);

            for (idx, is_top) in &ops {
                let task_id = task_ids[idx % n];
                let pos = if *is_top { "top" } else { "bottom" };
                svc.move_selection(task_id, pos).unwrap();
            }

            let orders = selection_sort_orders(&svc);
            let result_ids = selected_task_ids_in_order(&svc);

            // Invariant 1: sort_orders are contiguous 10, 20, 30, ...
            let expected_orders: Vec<i32> = (1..=orders.len())
                .map(|i| (i * 10) as i32)
                .collect();
            prop_assert_eq!(&orders, &expected_orders,
                "sort_orders must be contiguous multiples of 10");

            // Invariant 2: all task ids present
            let mut sorted_ids = result_ids.clone();
            sorted_ids.sort();
            let mut orig_ids = task_ids.clone();
            orig_ids.sort();
            prop_assert_eq!(&sorted_ids, &orig_ids,
                "all original task ids must be present");

            // Invariant 3: last operation's target is at correct position
            if let Some((idx, is_top)) = ops.last() {
                let moved_id = task_ids[idx % n];
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
    fn test_live_board_empty() {
        let (svc, _dir) = test_service();
        let board = svc.get_live_board(None).unwrap();
        assert!(board.backlog.is_empty());
        assert!(board.selected.is_empty());
        assert_eq!(board.stats.total_backlog, 0);
        assert_eq!(board.stats.total_selected, 0);
    }

    #[test]
    fn test_select_and_deselect() {
        let (svc, _dir) = test_service();

        // Create a project and todo task
        let project = svc.create_project("Test", "", &[], None).unwrap();
        let task = svc
            .create_task(project.id, "Task 1", "desc", None, None, &[])
            .unwrap();

        // Select the task
        let selections = svc.select_tasks(&[task.id]).unwrap();
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].status, SelectionStatus::Queued);

        // Verify live board
        let board = svc.get_live_board(None).unwrap();
        assert!(board.backlog.is_empty()); // task is now selected, not in backlog
        assert_eq!(board.selected.len(), 1);
        assert_eq!(board.stats.queued, 1);

        // Deselect
        svc.deselect_task(task.id).unwrap();
        let board = svc.get_live_board(None).unwrap();
        assert_eq!(board.backlog.len(), 1); // back in backlog
        assert!(board.selected.is_empty());
    }

    #[test]
    fn test_start_and_complete_task() {
        let (svc, _dir) = test_service();

        let project = svc.create_project("Test", "", &[], None).unwrap();
        let task = svc
            .create_task(project.id, "Task 1", "desc", None, None, &[])
            .unwrap();

        svc.select_tasks(&[task.id]).unwrap();

        // Start task
        let selected = svc.start_selected_task(task.id).unwrap();
        assert_eq!(selected.selection.status, SelectionStatus::Active);
        assert_eq!(selected.task.status, TaskStatus::InProgress);

        // Complete task
        svc.complete_selected_task(task.id, &TaskStatus::ReadyForReview)
            .unwrap();

        let board = svc.get_live_board(None).unwrap();
        assert_eq!(board.stats.completed, 1);
        assert!(board.stats.active.is_none());
    }

    #[test]
    fn test_fail_and_pause_task() {
        let (svc, _dir) = test_service();

        let project = svc.create_project("Test", "", &[], None).unwrap();
        let t1 = svc
            .create_task(project.id, "Task 1", "desc", None, None, &[])
            .unwrap();
        let t2 = svc
            .create_task(project.id, "Task 2", "desc", None, None, &[])
            .unwrap();

        svc.select_tasks(&[t1.id, t2.id]).unwrap();

        // Start and fail t1
        svc.start_selected_task(t1.id).unwrap();
        svc.fail_selected_task(t1.id, "compilation error").unwrap();

        let board = svc.get_live_board(None).unwrap();
        assert_eq!(board.stats.failed, 1);

        // Start and pause t2
        svc.start_selected_task(t2.id).unwrap();
        svc.pause_selected_task(t2.id).unwrap();

        let board = svc.get_live_board(None).unwrap();
        assert!(board.stats.active.is_none());

        // Get next should return paused t2 (resume priority)
        let next = svc.get_next_selected_task().unwrap();
        assert!(next.is_some());
        assert_eq!(next.as_ref().map(|n| n.task.id), Some(t2.id));
    }

    #[test]
    fn test_clear_completed() {
        let (svc, _dir) = test_service();

        let project = svc.create_project("Test", "", &[], None).unwrap();
        let task = svc
            .create_task(project.id, "Task 1", "desc", None, None, &[])
            .unwrap();

        svc.select_tasks(&[task.id]).unwrap();
        svc.start_selected_task(task.id).unwrap();
        svc.complete_selected_task(task.id, &TaskStatus::ReadyForReview)
            .unwrap();

        let count = svc.clear_completed_selections().unwrap();
        assert_eq!(count, 1);

        let board = svc.get_live_board(None).unwrap();
        assert_eq!(board.stats.total_selected, 0);
    }

    #[test]
    fn test_select_next_todo_for_agent_picks_highest_priority() {
        let (svc, _dir) = test_service();

        let project = svc.create_project("Test", "", &[], None).unwrap();

        let low = svc
            .create_task(
                project.id,
                "Low",
                "desc",
                Some(&TaskStatus::Todo),
                Some(&TaskPriority::Low),
                &[],
            )
            .unwrap();
        let high = svc
            .create_task(
                project.id,
                "High",
                "desc",
                Some(&TaskStatus::Todo),
                Some(&TaskPriority::High),
                &[],
            )
            .unwrap();

        let first = svc.select_next_todo_for_agent().unwrap().unwrap();
        assert_eq!(first.task_id, high.id);

        let second = svc.select_next_todo_for_agent().unwrap().unwrap();
        assert_eq!(second.task_id, low.id);

        let none_left = svc.select_next_todo_for_agent().unwrap();
        assert!(none_left.is_none());
    }
}
