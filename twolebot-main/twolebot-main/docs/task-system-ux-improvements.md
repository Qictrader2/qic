# Task System & Live Board UX Improvements

Actionable items from UX evaluation (2026-02-11).

**IMPORTANT FOR IMPLEMENTATION AGENT: Ticking checkboxes as you complete each item is NOT optional. Update this file after every completed sub-task. Run `./compile.sh` from the repo root after every item — it must pass before moving on.**

---

## Agent/MCP Improvements

### 1. Add Live Board MCP Tools

The live board has full HTTP coverage but only `selection_move` in MCP. Agents cannot see, populate, or clean the board.

**Add these MCP tools** (minimal set, mirroring existing HTTP handlers + `LiveApp` methods):

| Tool | Description | Params | Returns |
|------|-------------|--------|---------|
| `live_board_get` | Get board state (backlog + selected + stats) | `backlog_limit?: i32` | `LiveBoard` JSON |
| `live_board_select` | Select todo tasks onto the queue | `task_ids: Vec<i64>` | `Vec<LiveBoardSelection>` |
| `live_board_deselect` | Remove a task from the queue | `task_id: i64` | success text |
| `live_board_clear_completed` | Clear done/failed selections | (none) | `{ cleared: i32 }` |

`selection_move` already exists and stays. No new app/service methods needed — all four map directly to existing `LiveApp` methods.

**Pattern to follow:** Look at `selection_move` in `src/mcp/work_tools.rs:593-609`. Each new tool follows the same pattern: take `Parameters<RequestType>`, call `self.app.live.<method>()`, return via `Self::json_result()` or `CallToolResult::success()`.

**Request types already exist** in `src/types.rs` (lines 649-669): `GetLiveBoardRequest`, `SelectTasksRequest`, `DeselectTaskRequest`. No new types needed — reuse those.

**The `LiveApp` methods already exist** in `src/work/app.rs` (lines 373-415):
- `self.app.live.get_live_board(backlog_limit)` → `Result<LiveBoard, TwolebotError>`
- `self.app.live.select_tasks(task_ids)` → `Result<Vec<LiveBoardSelection>, TwolebotError>`
- `self.app.live.deselect_task(task_id)` → `Result<(), TwolebotError>`
- `self.app.live.clear_completed_selections()` → `Result<i32, TwolebotError>`

**Files to change:**
- `src/mcp/work_tools.rs` — add 4 tool methods inside the `#[tool_router] impl WorkTools` block

- [x] `live_board_get` tool added
- [x] `live_board_select` tool added
- [x] `live_board_deselect` tool added
- [x] `live_board_clear_completed` tool added
- [x] `./compile.sh` passes

### 2. Compact Task List for MCP

Add a `compact` boolean param to `task_list`. When true, return a leaner struct instead of the full 19-field `TaskModel`.

**Compact fields:** `id`, `status`, `sort_order`, `title`, `tags`, `comment_count`

Currently `task_list` returns `PaginatedResponse<TaskModel>`. When `compact=true`, it should return `PaginatedResponse<TaskSummary>` instead. Since these are different types, the MCP tool needs to serialize the appropriate one. Use `serde_json::Value` or an enum wrapper to handle both.

**Files to change:**
- `src/types.rs` — add `compact: Option<bool>` to `ListTasksRequest` (line ~483)
- `src/work/models.rs` — add new struct:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TaskSummary {
      pub id: i64,
      pub status: TaskStatus,
      pub sort_order: i32,
      pub title: String,
      pub tags: Vec<String>,
      pub comment_count: i64,
  }
  ```
- `src/work/service.rs` — add `list_tasks_compact()` method using SQL:
  ```sql
  SELECT t.id, t.status, t.sort_order, t.title,
         COALESCE((SELECT json_group_array(tag) FROM task_tags WHERE task_id = t.id), '[]') AS tags_json,
         (SELECT COUNT(*) FROM comments WHERE task_id = t.id) AS comment_count
  FROM tasks t WHERE ... ORDER BY t.sort_order ASC
  ```
- `src/work/app.rs` — add `list_compact()` method on `TasksApp`
- `src/mcp/work_tools.rs` — in `list_tasks`, check `req.compact`, call the right method
- `src/server/work_handlers.rs` — same check in `list_tasks` handler

- [x] `TaskSummary` struct added to `models.rs`
- [x] `list_tasks_compact()` query implemented in service
- [x] `list_compact()` added to `TasksApp` in `app.rs`
- [x] MCP tool branches on compact flag
- [x] HTTP handler branches on compact flag
- [x] `./compile.sh` passes

### 3. Make `activity_recent` Consistent

**Current issues:**
- Results sorted by `id DESC` but timestamps have inconsistent formats (some `2026-02-10T21:04:29Z`, some `2026-02-10 21:37:36` without T or Z)
- No project filter

**Fix:**
1. Audit all `INSERT INTO activity_logs` across the codebase. Search for `INSERT INTO activity_logs` — they exist in `src/work/service.rs` and `src/work/live_board.rs`. Ensure every one uses `strftime('%Y-%m-%dT%H:%M:%SZ', 'now')` for `created_at`.
2. Change the `get_recent_activity` query in `service.rs` to `ORDER BY created_at DESC` instead of the current ordering.
3. Add optional `project_id` filter.

**Files to change:**
- `src/types.rs` — add `project_id: Option<i64>` to `RecentActivityRequest` (line ~644)
- `src/work/service.rs` — update `get_recent_activity()` to accept `project_id`, add WHERE clause, fix ORDER BY
- `src/work/app.rs` — update `ActivityApp::recent()` to pass project_id
- `src/mcp/work_tools.rs` — pass `req.0.project_id` to `self.app.activity.recent()`
- `src/server/work_handlers.rs` — pass `req.project_id` to `state.app.activity.recent()`
- `src/work/live_board.rs` — audit timestamp format in INSERT statements

- [x] All `INSERT INTO activity_logs` use ISO 8601 timestamps
- [x] Query uses ORDER BY created_at DESC
- [x] project_id filter wired through types → app → service → MCP → HTTP
- [x] `./compile.sh` passes

### 4. Make `dependency_add`/`dependency_remove` Return Consistent Responses

Currently returns plain string `"Dependency added"` / `"Dependency removed"` while every other mutating tool returns the full object.

**Fix:** After add/remove, fetch and return the updated task.

**Files to change:**
- `src/mcp/work_tools.rs` — in `add_task_dependency` (line ~357) and `remove_task_dependency` (line ~371):
  Replace the `Ok(CallToolResult::success(vec![Content::text(...)]))` with:
  ```rust
  let task = self.app.tasks.get(req.0.task_id).await.map_err(Self::to_mcp_err)?;
  Self::json_result(&task)
  ```

- [x] `dependency_add` returns full task with populated `blocked_by`/`blocks`
- [x] `dependency_remove` returns full task with populated `blocked_by`/`blocks`
- [x] `./compile.sh` passes

### 5. Remove `worker_type` From System

`worker_type` exists in 4 DB tables and throughout the codebase. It was supposed to be removed already. `worker_name` stays.

**Migration strategy:** The project uses versioned migrations in `src/work/db.rs`. Current version is 2. Add `migrate_v3` that does `ALTER TABLE <table> DROP COLUMN worker_type` for each of the 4 tables. SQLite 3.35+ supports DROP COLUMN and the system uses 3.45.1.

Also update the v1 CREATE TABLE statements to remove `worker_type` (for fresh DBs).

**Tables with worker_type:** `tasks` (line 161), `documents` (line 199), `comments` (line 214), `activity_logs` (line 239)

**Full file list to change:**
- `src/work/db.rs` — add `migrate_v3` function, add to migrations vec, remove `worker_type` from all CREATE TABLE statements
- `src/work/queries.rs` — remove from any row mapper that reads `worker_type`
- `src/work/service.rs` — remove from INSERT/UPDATE statements that write `worker_type`
- `src/types.rs` — remove `worker_type` field from request structs in the `work` module (search for `worker_type`)
- `src/mcp/work_tools.rs` — remove any parameter handling for worker_type
- `src/server/work_handlers.rs` — remove any parsing of worker_type
- `src/work/models.rs` — remove any worker_type fields from structs

**Also check:** `src/work/adapters.rs` and the Elm frontend `frontend/src/Types.elm`, `frontend/src/Api.elm` for worker_type references.

- [x] `migrate_v3` added to `db.rs` (DROP COLUMN on all 4 tables)
- [x] v1 CREATE TABLE statements updated (for fresh DBs)
- [x] Removed from `queries.rs` row mappers
- [x] Removed from `service.rs` INSERT/UPDATE statements
- [x] Removed from `types.rs` request structs
- [x] Removed from MCP tools and HTTP handlers
- [x] Removed from Elm frontend (Types.elm, Api.elm) if present
- [x] `./compile.sh` passes

### 6. Task Archival

Add `archived` as a valid `TaskStatus`. Archived tasks excluded from default list queries.

**Files to change:**
- `src/work/models.rs` — add `Archived` variant to `TaskStatus` enum, add `Display`/`FromStr` arms
- `src/work/db.rs` — update the CHECK constraint in v1 CREATE TABLE (add `'archived'`), AND add it to `migrate_v3` (or a new v4) via: `ALTER TABLE tasks DROP CONSTRAINT ...` — actually SQLite doesn't support dropping constraints. Instead, the CHECK constraint in the CREATE TABLE is only enforced on fresh DBs. For existing DBs, SQLite CHECK constraints are evaluated on INSERT/UPDATE, so just adding `'archived'` to the v1 schema is sufficient for fresh DBs. For existing DBs, a migration can recreate the table or simply rely on the fact that SQLite's CHECK can be bypassed if not re-created. **Simplest approach:** just update the v1 CREATE TABLE for fresh DBs. For existing DBs, SQLite won't enforce the CHECK if the column already exists — the value `'archived'` will be accepted because the constraint is only checked against the schema at table creation time. Verify this works with a test.
- `src/work/service.rs` — in `list_tasks()`, when no status filter is provided, add `WHERE status != 'archived'`
- Frontend: `frontend/src/Components/TaskFilters.elm` — add Archived chip (covered in item 16)

- [x] `Archived` added to `TaskStatus` enum with Display/FromStr
- [x] CHECK constraint updated in v1 schema for fresh DBs
- [x] Default list queries exclude `archived` when no status filter
- [x] `./compile.sh` passes

### 7. Require Comment on Review Transitions

When `task_update` transitions to `ready_for_review`, require a comment explaining what was done. `task_reject_review` already requires `reviewer_comment` — no change needed there.

**Files to change:**
- `src/types.rs` — add `comment: Option<String>` to `UpdateTaskRequest` (line ~511)
- `src/work/service.rs` — in `update_task()`, after status change is applied: if new status is `ReadyForReview` and comment is None/empty, return error `"comment required when moving to ready_for_review"`. If comment is provided, insert it into `comments` table for the task.
- `src/work/models.rs` — add `comment: Option<String>` to `TaskUpdate` struct (line ~342)
- `src/mcp/work_tools.rs` — in `update_task` (line ~237), read `req.comment` and pass to `TaskUpdate`
- `src/server/work_handlers.rs` — in `update_task` handler (line ~179), read `req.comment` and pass to `TaskUpdate`
- `frontend/src/Pages/TaskDetail.elm` — when user clicks "Ready for Review" status button, show a comment input prompt before submitting
- `frontend/src/Api.elm` — add `comment` field to `updateTask` function (line ~838)

- [x] `comment` field added to `UpdateTaskRequest` and `TaskUpdate`
- [x] Service rejects ready_for_review without comment
- [x] Comment auto-inserted into comments table
- [x] MCP tool passes comment through
- [x] HTTP handler passes comment through
- [x] Frontend prompts for comment on ready_for_review transition
- [x] `Api.elm` updated to send comment field
- [x] `./compile.sh` passes

### 8. Analytics: Cycle Time Tracking

The `avg_completion_hours` field exists in `TaskAnalytics` (models.rs line ~368) but is always `None`.

**Files to change:**
- `src/work/service.rs` — in `get_task_status_analytics()`, add query:
  ```sql
  SELECT AVG((julianday(completed_at) - julianday(created_at)) * 24)
  FROM tasks WHERE completed_at IS NOT NULL AND project_id = ?
  ```
  Assign result to `avg_completion_hours`.

- [x] SQL query added to `get_task_status_analytics()`
- [x] `avg_completion_hours` populated in response
- [x] `./compile.sh` passes

---

## Human/Frontend UX Improvements

**Context for frontend work:**
- Elm frontend lives in `frontend/src/`. Build with `./elm-build.sh` (quick) or `./compile.sh` (full).
- No Elm ports currently — the app is pure Elm, no JS interop yet.
- API calls go through `frontend/src/Api.elm` which uses `workPost "/path" body decoder toMsg` pattern.
- `Api.updateTask` already exists (Api.elm line 838) and calls `POST /api/work/tasks/update`.
- Pages: `frontend/src/Pages/TaskDetail.elm`, `Pages/LiveBoard.elm`, `Pages/ProjectDetail.elm`, `Pages/Projects.elm`.
- Shared components: `Components/TaskCard.elm`, `Components/TaskFilters.elm`, `Components/Comments.elm`.
- Types: `frontend/src/Types.elm`.
- Styling is inline Elm `Html.Attributes.style` — no external CSS.

### 9. Inline Editing on Task Detail Page (HIGH)

`frontend/src/Pages/TaskDetail.elm` is read-only except for status buttons and comments. Users cannot edit title, description, priority, or tags.

**Implementation approach:**
- Add `EditingField` type to the page model: `EditingTitle String | EditingDescription String | EditingPriority | EditingTags String | NotEditing`
- On click of title text → switch to input field pre-filled with current value
- On Enter or blur → call `Api.updateTask taskId { title = Just newTitle, ... }` and reset to NotEditing
- On Escape → cancel edit, reset to NotEditing
- Same pattern for description (use textarea), priority (chip selector), tags (comma-separated input)

**Files to change:**
- `frontend/src/Pages/TaskDetail.elm` — add edit states to Model, view functions for edit mode, Msg variants for edit actions

- [x] Title click-to-edit implemented
- [x] Description click-to-edit implemented
- [x] Priority selector implemented
- [x] Tags editor implemented
- [x] All edits call `Api.updateTask`
- [x] `./compile.sh` passes

### 10. Project Context on Live Board (HIGH)

`frontend/src/Pages/LiveBoard.elm` shows task cards with no project name.

**Implementation approach:** The `LiveBoard` response already includes full `TaskModel` for each selected task, and `TaskModel` has `project_id`. Options:
1. **Backend approach (preferred):** Add `project_name` to `TaskModel` by joining with projects table in the live_board query. This requires changes in `src/work/live_board.rs` and `src/work/queries.rs`.
2. **Frontend approach:** On live board load, also call `Api.listProjects` and build a `Dict Int String` of id→name, then look up per task.

Go with option 2 (simpler, no backend change): load projects alongside the board data and display project name on each card.

**Files to change:**
- `frontend/src/Pages/LiveBoard.elm` — add `projects : Dict Int String` to Model, fetch projects on init, show project name on task cards

- [x] Project names loaded on live board init
- [x] Project name shown on each task card in selected queue and backlog
- [x] `./compile.sh` passes

### 11. Task Title Clickable on Live Board (HIGH)

In `frontend/src/Pages/LiveBoard.elm`, task titles in the selected queue are rendered as plain `text` nodes. They should be links.

**Fix:** Wrap the task title in `Html.a [ href ("/tasks/" ++ String.fromInt task.id) ] [ text task.title ]` with appropriate styling (underline on hover, inherit color).

- [x] Task titles are `<a>` links to `/tasks/{id}` on live board
- [x] `./compile.sh` passes

### 12. Drag-and-Drop Status Transitions (HIGH)

The project task list (`frontend/src/Pages/ProjectDetail.elm`) shows tasks as a flat list. Users must click into task detail to change status.

**Implementation approach:**
- Group tasks by status into columns/sections (kanban-style) in the Tasks tab
- Use HTML5 drag-and-drop via Elm's `Html.Events` — Elm supports `on "dragstart"`, `on "dragover"`, `on "drop"` etc. through `Html.Events.on` with custom decoders. No JS ports needed.
- On drop: call `Api.updateTask taskId { status = Just newStatus, ... }`
- For transitions that require a comment (ready_for_review per item 7), show a modal/prompt on drop before submitting
- Keep the existing flat list + filter as an alternative view mode (toggle between "list" and "board" view)

**Key Elm drag-and-drop pattern:**
```elm
-- On drag source
Html.Attributes.attribute "draggable" "true"
Html.Events.on "dragstart" (D.succeed (DragStart taskId))

-- On drop target (status column)
Html.Events.preventDefaultOn "dragover" (D.succeed ( DragOver, True ))
Html.Events.on "drop" (D.succeed (DropOnStatus targetStatus))
```

**Files to change:**
- `frontend/src/Pages/ProjectDetail.elm` — add board view with status columns, drag event handlers, view mode toggle
- May need `frontend/src/Components/Modal.elm` for the comment prompt (already exists)

- [x] Board/kanban view added to project detail (toggle between list and board)
- [x] Tasks grouped by status into columns
- [x] Drag-and-drop between columns triggers status change via API
- [x] Comment-required transitions (ready_for_review) show prompt on drop
- [x] `./compile.sh` passes

### 13. Label the "R" Button (MEDIUM)

Top-right of project detail and task detail pages shows an unlabeled "R" that appears to be refresh.

**Files to change:**
- `frontend/src/Pages/ProjectDetail.elm` — find the "R" button, change text to "REFRESH" (match the pattern used on the Dashboard and Live Board pages)
- `frontend/src/Pages/TaskDetail.elm` — same

- [x] "R" button shows "REFRESH" text on project detail
- [x] "R" button shows "REFRESH" text on task detail
- [x] `./compile.sh` passes

### 14. Fix Priority Color Semantics (MEDIUM)

Priority badges currently: LOW = green (#4ade80), MEDIUM = yellow (#fbbf24), HIGH = red (#f87171). Green = "low priority" is confusing because green usually means "good/done."

**Suggested palette:**
- LOW = muted blue (#6b8aad) or gray (#8b949e)
- MEDIUM = neutral slate (#a0aec0)
- HIGH = amber/orange (#f59e0b)
- CRITICAL = red (#f87171)

**Files to change:** Search for priority color definitions. They're likely in `frontend/src/Components/TaskCard.elm` or `frontend/src/UI.elm` — look for functions that map priority strings to colors.

- [x] Priority color mapping updated
- [x] Consistent across TaskCard, TaskDetail, LiveBoard, ProjectDetail
- [x] `./compile.sh` passes

### 15. Task Count on Project Cards (MEDIUM)

`frontend/src/Pages/Projects.elm` shows project cards with name, description, status, date — but no task count.

**Implementation approach:** The `listProjects` API doesn't return task counts. Two options:
1. **Backend:** Add task count fields to the project list response (add a subquery in `list_projects`). This is the proper approach.
2. **Frontend:** After loading projects, fire off `listTasks` for each project — expensive.

Go with option 1.

**Backend change:**
- `src/work/service.rs` — in `list_projects()`, add a subquery or JOIN:
  ```sql
  (SELECT COUNT(*) FROM tasks WHERE project_id = p.id AND status != 'archived') AS task_count
  ```
- `src/work/models.rs` — add `task_count: i64` to `Project` struct
- `src/work/queries.rs` — update `row_to_project` to read the new column

**Frontend change:**
- `frontend/src/Types.elm` — add `taskCount : Int` to `WorkProject`
- `frontend/src/Api.elm` — update `workProjectDecoder` to decode `task_count`
- `frontend/src/Pages/Projects.elm` — show count on card, e.g. `"12 tasks"`

- [x] Backend returns task_count in project list response
- [x] Frontend decodes and displays task count on project cards
- [x] `./compile.sh` passes

### 16. Add "Abandoned" to Filter Chips (LOW)

`frontend/src/Components/TaskFilters.elm` has status filter chips but is missing Abandoned (and Archived once item 6 lands).

**Files to change:**
- `frontend/src/Components/TaskFilters.elm` — add "Abandoned" to the status options list. After item 6, also add "Archived".

- [x] Abandoned chip added to status filter
- [x] Archived chip added (after item 6 is done)
- [x] `./compile.sh` passes

### 17. Show Task ID Instead of Sort Order on Live Board (LOW)

`frontend/src/Pages/LiveBoard.elm` shows `#10` (the sort_order) on selected task cards. This looks like a task ID but isn't.

**Fix:** Change the display to show the task ID. Search for `sort_order` or `#` in LiveBoard.elm — it's likely rendered as `"#" ++ String.fromInt selection.sort_order`. Change to `"#" ++ String.fromInt task.id`.

- [x] Live board cards show task ID (e.g. `#2`) not sort_order (e.g. `#10`)
- [x] `./compile.sh` passes

---

## Priority Order

**Agent/MCP (do first):**
1. Live board MCP tools (unblocks agent automation)
2. Remove worker_type (cleanup debt)
3. Compact task list (token efficiency)
4. Consistent dependency responses (quick fix)
5. Activity recent consistency + project filter (quick fix)
6. Task archival (new feature)
7. Require comment on review transitions (workflow improvement)
8. Analytics cycle time (nice-to-have)

**Human/Frontend (do after agent items):**
9. Inline editing on task detail (highest impact)
10. Project context on live board
11. Task title clickable on live board
12. Drag-and-drop status transitions
13. Label the "R" button
14. Fix priority color semantics
15. Task count on project cards
16. Add Abandoned to filter chips
17. Show task ID on live board cards
