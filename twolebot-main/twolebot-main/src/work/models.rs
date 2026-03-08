use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ── Task status ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    ReadyForReview,
    UnderReview,
    Done,
    Blocked,
    Abandoned,
    Archived,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "todo"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::ReadyForReview => write!(f, "ready_for_review"),
            TaskStatus::UnderReview => write!(f, "under_review"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Blocked => write!(f, "blocked"),
            TaskStatus::Abandoned => write!(f, "abandoned"),
            TaskStatus::Archived => write!(f, "archived"),
        }
    }
}

impl FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "ready_for_review" => Ok(TaskStatus::ReadyForReview),
            "under_review" => Ok(TaskStatus::UnderReview),
            "done" => Ok(TaskStatus::Done),
            "blocked" => Ok(TaskStatus::Blocked),
            "abandoned" => Ok(TaskStatus::Abandoned),
            "archived" => Ok(TaskStatus::Archived),
            _ => Err(format!("invalid task status: {s}")),
        }
    }
}

// ── Task priority ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "low"),
            TaskPriority::Medium => write!(f, "medium"),
            TaskPriority::High => write!(f, "high"),
            TaskPriority::Critical => write!(f, "critical"),
        }
    }
}

impl FromStr for TaskPriority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            "critical" => Ok(TaskPriority::Critical),
            _ => Err(format!("invalid task priority: {s}")),
        }
    }
}

// ── Document types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Plan,
    Specification,
    Notes,
    Code,
    Other,
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentType::Plan => write!(f, "plan"),
            DocumentType::Specification => write!(f, "specification"),
            DocumentType::Notes => write!(f, "notes"),
            DocumentType::Code => write!(f, "code"),
            DocumentType::Other => write!(f, "other"),
        }
    }
}

impl FromStr for DocumentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plan" => Ok(DocumentType::Plan),
            "specification" => Ok(DocumentType::Specification),
            "notes" => Ok(DocumentType::Notes),
            "code" => Ok(DocumentType::Code),
            "other" => Ok(DocumentType::Other),
            _ => Err(format!("invalid document type: {s}")),
        }
    }
}

// ── Activity actions ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityAction {
    Created,
    Updated,
    Deleted,
    StatusChanged,
    Commented,
    ReviewRejected,
}

impl fmt::Display for ActivityAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityAction::Created => write!(f, "created"),
            ActivityAction::Updated => write!(f, "updated"),
            ActivityAction::Deleted => write!(f, "deleted"),
            ActivityAction::StatusChanged => write!(f, "status_changed"),
            ActivityAction::Commented => write!(f, "commented"),
            ActivityAction::ReviewRejected => write!(f, "review_rejected"),
        }
    }
}

impl FromStr for ActivityAction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(ActivityAction::Created),
            "updated" => Ok(ActivityAction::Updated),
            "deleted" => Ok(ActivityAction::Deleted),
            "status_changed" => Ok(ActivityAction::StatusChanged),
            "commented" => Ok(ActivityAction::Commented),
            "review_rejected" => Ok(ActivityAction::ReviewRejected),
            _ => Err(format!("invalid activity action: {s}")),
        }
    }
}

// ── Selection status (live board) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SelectionStatus {
    Queued,
    Active,
    Paused,
    Done,
    Failed,
}

impl fmt::Display for SelectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionStatus::Queued => write!(f, "queued"),
            SelectionStatus::Active => write!(f, "active"),
            SelectionStatus::Paused => write!(f, "paused"),
            SelectionStatus::Done => write!(f, "done"),
            SelectionStatus::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for SelectionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(SelectionStatus::Queued),
            "active" => Ok(SelectionStatus::Active),
            "paused" => Ok(SelectionStatus::Paused),
            "done" => Ok(SelectionStatus::Done),
            "failed" => Ok(SelectionStatus::Failed),
            _ => Err(format!("invalid selection status: {s}")),
        }
    }
}

// ── Agent loop state ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentLoopState {
    Idle,
    Running,
    Paused,
}

impl fmt::Display for AgentLoopState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentLoopState::Idle => write!(f, "idle"),
            AgentLoopState::Running => write!(f, "running"),
            AgentLoopState::Paused => write!(f, "paused"),
        }
    }
}

impl FromStr for AgentLoopState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "idle" => Ok(AgentLoopState::Idle),
            "running" => Ok(AgentLoopState::Running),
            "paused" => Ok(AgentLoopState::Paused),
            _ => Err(format!("invalid agent loop state: {s}")),
        }
    }
}

// ── Core entities ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub git_remote_url: Option<String>,
    pub tags: Vec<String>,
    pub is_active: bool,
    pub task_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskModel {
    pub id: i64,
    pub project_id: i64,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub sort_order: i32,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub blocked_by: Vec<i64>,
    #[serde(default)]
    pub blocks: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: i64,
    pub project_id: i64,
    pub document_type: DocumentType,
    pub title: String,
    pub content: String,
    pub version: i32,
    pub deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub task_id: Option<i64>,
    pub document_id: Option<i64>,
    pub parent_comment_id: Option<i64>,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: i64,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub document_id: Option<i64>,
    pub action: ActivityAction,
    pub actor: String,
    pub details: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBoardSelection {
    pub id: i64,
    pub task_id: i64,
    pub sort_order: i32,
    pub selected_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub status: SelectionStatus,
}

// ── Compact summary (for token-efficient task listings) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: i64,
    pub status: TaskStatus,
    pub sort_order: i32,
    pub title: String,
    pub tags: Vec<String>,
    pub comment_count: i64,
}

// ── Partial update types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub tags: Option<Vec<String>>,
    pub comment: Option<String>,
}

// ── Workflow result types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeNextResult {
    pub task: TaskModel,
    pub comments: Vec<Comment>,
    pub warning: Option<String>,
}

// ── Analytics types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalytics {
    pub status_counts: Vec<StatusCount>,
    pub avg_completion_hours: Option<f64>,
    pub throughput_30d: Vec<DayCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCount {
    pub status: TaskStatus,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCount {
    pub date: String,
    pub count: i64,
}

// ── Paginated response ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

// ── Live board composite types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBoard {
    pub backlog: Vec<TaskModel>,
    pub selected: Vec<SelectedTask>,
    pub stats: LiveBoardStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedTask {
    pub selection: LiveBoardSelection,
    pub task: TaskModel,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBoardStats {
    pub total_backlog: usize,
    pub total_selected: usize,
    pub queued: usize,
    pub completed: usize,
    pub failed: usize,
    pub active: Option<i64>,
    pub agent_loop_state: AgentLoopState,
}

// ── SSE event types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WorkEvent {
    SelectionUpdated(LiveBoardStats),
    AgentProgress {
        task_id: i64,
        message: String,
    },
}
