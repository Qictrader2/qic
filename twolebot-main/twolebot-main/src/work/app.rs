use std::sync::Arc;

use tokio::sync::{broadcast, OnceCell};

use crate::error::TwolebotError;
use crate::semantic::Embedder;
use crate::storage::{PromptFeed, ResponseFeed};

use super::db::WorkDb;
use super::models::*;
use super::pm_search::{PmIndexStats, PmSearch, PmSearchResult};
use super::service::WorkService;
use super::AgentLoop;

async fn run_blocking<T, F>(f: F) -> Result<T, TwolebotError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, TwolebotError> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| TwolebotError::work(format!("spawn_blocking: {e}")))?
}

#[derive(Clone)]
pub struct WorkApp {
    service: Arc<WorkService>,
    pub projects: ProjectsApp,
    pub tasks: TasksApp,
    pub documents: DocumentsApp,
    pub comments: CommentsApp,
    pub live: LiveApp,
    pub activity: ActivityApp,
    pub search: SearchApp,
}

impl WorkApp {
    pub fn new(db: WorkDb) -> Self {
        Self::from_service(Arc::new(WorkService::new(db)))
    }

    pub fn from_service(service: Arc<WorkService>) -> Self {
        Self {
            projects: ProjectsApp {
                service: service.clone(),
            },
            tasks: TasksApp {
                service: service.clone(),
            },
            documents: DocumentsApp {
                service: service.clone(),
            },
            comments: CommentsApp {
                service: service.clone(),
            },
            live: LiveApp {
                service: service.clone(),
            },
            activity: ActivityApp {
                service: service.clone(),
            },
            search: SearchApp {
                pm_search: Arc::new(OnceCell::new()),
                db: service.db().clone(),
            },
            service,
        }
    }

    pub fn new_agent_loop(
        &self,
        prompt_feed: Arc<PromptFeed>,
        response_feed: Arc<ResponseFeed>,
        settings_store: Arc<crate::storage::SettingsStore>,
    ) -> AgentLoop {
        AgentLoop::new(self.service.clone(), prompt_feed, response_feed, settings_store)
    }

    pub fn event_tx(&self) -> &broadcast::Sender<WorkEvent> {
        self.service.event_tx()
    }
}

#[derive(Clone)]
pub struct ProjectsApp {
    service: Arc<WorkService>,
}

impl ProjectsApp {
    pub async fn list(
        &self,
        active_only: bool,
        limit: i32,
        git_remote_url: Option<String>,
    ) -> Result<PaginatedResponse<Project>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.list_projects(active_only, limit, git_remote_url.as_deref())).await
    }

    pub async fn get(&self, project_id: i64) -> Result<Project, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_project(project_id)).await
    }

    pub async fn get_by_git_remote(
        &self,
        git_remote_url: String,
    ) -> Result<Option<Project>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_project_by_git_remote(&git_remote_url)).await
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
        git_remote_url: Option<String>,
    ) -> Result<Project, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || {
            svc.create_project(
                &name,
                description.as_deref().unwrap_or(""),
                &tags.unwrap_or_default(),
                git_remote_url.as_deref(),
            )
        })
        .await
    }

    pub async fn update(
        &self,
        project_id: i64,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
        git_remote_url: Option<String>,
    ) -> Result<Project, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || {
            svc.update_project(
                project_id,
                &name,
                description.as_deref(),
                tags.as_deref(),
                git_remote_url.as_ref().map(|u| Some(u.as_str())),
            )
        })
        .await
    }

    pub async fn archive(&self, project_id: i64) -> Result<Project, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.archive_project(project_id)).await
    }
}

#[derive(Clone)]
pub struct TasksApp {
    service: Arc<WorkService>,
}

impl TasksApp {
    pub async fn list(
        &self,
        project_id: Option<i64>,
        status: Option<Vec<String>>,
        limit: i32,
    ) -> Result<PaginatedResponse<TaskModel>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.list_tasks(project_id, status.as_deref(), limit)).await
    }

    pub async fn list_compact(
        &self,
        project_id: Option<i64>,
        status: Option<Vec<String>>,
        limit: i32,
    ) -> Result<PaginatedResponse<TaskSummary>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.list_tasks_compact(project_id, status.as_deref(), limit)).await
    }

    pub async fn get(&self, task_id: i64) -> Result<TaskModel, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_task(task_id)).await
    }

    pub async fn create(&self, req: CreateTaskInput) -> Result<TaskModel, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || {
            svc.create_task(
                req.project_id,
                &req.title,
                req.description.as_deref().unwrap_or(""),
                req.status.as_ref(),
                req.priority.as_ref(),
                &req.tags.unwrap_or_default(),
            )
        })
        .await
    }

    pub async fn update(
        &self,
        task_id: i64,
        update: TaskUpdate,
    ) -> Result<TaskModel, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.update_task(task_id, &update)).await
    }

    pub async fn take_next(
        &self,
        project_id: i64,
        force: bool,
    ) -> Result<Option<TakeNextResult>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.take_next_task(project_id, force)).await
    }

    pub async fn take_next_review(
        &self,
        project_id: i64,
        force: bool,
    ) -> Result<Option<TakeNextResult>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.take_next_review_task(project_id, force)).await
    }

    pub async fn reject_review(
        &self,
        task_id: i64,
        reviewer_comment: String,
    ) -> Result<TaskModel, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.task_reject_review(task_id, &reviewer_comment)).await
    }

    pub async fn move_to_top_or_bottom(
        &self,
        task_id: i64,
        position: String,
    ) -> Result<TaskModel, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.move_task_to_top_or_bottom(task_id, &position)).await
    }

    pub async fn add_dependency(
        &self,
        task_id: i64,
        depends_on_task_id: i64,
    ) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.add_dependency(task_id, depends_on_task_id)).await
    }

    pub async fn remove_dependency(
        &self,
        task_id: i64,
        depends_on_task_id: i64,
    ) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.remove_dependency(task_id, depends_on_task_id)).await
    }

    pub async fn analytics(&self, project_id: Option<i64>) -> Result<TaskAnalytics, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_task_status_analytics(project_id)).await
    }
}

#[derive(Clone)]
pub struct DocumentsApp {
    service: Arc<WorkService>,
}

impl DocumentsApp {
    pub async fn search(
        &self,
        query: String,
        project_id: Option<i64>,
        limit: i32,
    ) -> Result<PaginatedResponse<Document>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.search_documents(&query, project_id, limit)).await
    }

    pub async fn get(&self, document_id: i64) -> Result<Document, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_document(document_id)).await
    }

    pub async fn create(&self, req: CreateDocumentInput) -> Result<Document, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || {
            svc.create_document(
                req.project_id,
                &req.document_type,
                &req.title,
                req.content.as_deref().unwrap_or(""),
            )
        })
        .await
    }

    pub async fn update(
        &self,
        document_id: i64,
        title: String,
        content: String,
        document_type: DocumentType,
    ) -> Result<Document, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || {
            svc.update_document(
                document_id,
                Some(&title),
                Some(&content),
                Some(&document_type),
            )
        })
        .await
    }

    pub async fn read_content(
        &self,
        document_id: i64,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<String, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.read_document_content(document_id, offset, limit)).await
    }

    pub async fn edit_content(
        &self,
        document_id: i64,
        old_string: String,
        new_string: String,
        replace_all: bool,
    ) -> Result<Document, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.edit_document_content(document_id, &old_string, &new_string, replace_all)).await
    }

    pub async fn delete(&self, document_id: i64) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.delete_document(document_id)).await
    }
}

#[derive(Clone)]
pub struct CommentsApp {
    service: Arc<WorkService>,
}

impl CommentsApp {
    pub async fn list_task(
        &self,
        task_id: i64,
        limit: i32,
        page: i32,
    ) -> Result<PaginatedResponse<Comment>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.list_task_comments(task_id, limit, page)).await
    }

    pub async fn list_document(
        &self,
        document_id: i64,
        limit: i32,
        page: i32,
    ) -> Result<PaginatedResponse<Comment>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.list_document_comments(document_id, limit, page)).await
    }

    pub async fn upsert(
        &self,
        comment_id: Option<i64>,
        task_id: Option<i64>,
        document_id: Option<i64>,
        content: String,
        parent_comment_id: Option<i64>,
    ) -> Result<Comment, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || {
            svc.upsert_comment(
                comment_id,
                task_id,
                document_id,
                &content,
                parent_comment_id,
            )
        })
        .await
    }

    pub async fn delete(&self, comment_id: i64) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.delete_comment(comment_id)).await
    }
}

#[derive(Clone)]
pub struct LiveApp {
    service: Arc<WorkService>,
}

impl LiveApp {
    pub async fn get_live_board(
        &self,
        backlog_limit: Option<i32>,
    ) -> Result<LiveBoard, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_live_board(backlog_limit)).await
    }

    pub async fn select_tasks(
        &self,
        task_ids: Vec<i64>,
    ) -> Result<Vec<LiveBoardSelection>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.select_tasks(&task_ids)).await
    }

    pub async fn deselect_task(&self, task_id: i64) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.deselect_task(task_id)).await
    }

    pub async fn move_selection(
        &self,
        task_id: i64,
        position: String,
    ) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.move_selection(task_id, &position)).await
    }

    pub async fn clear_completed_selections(&self) -> Result<i32, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.clear_completed_selections()).await
    }

    pub async fn select_next_todo_for_agent(
        &self,
    ) -> Result<Option<LiveBoardSelection>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.select_next_todo_for_agent()).await
    }

    pub async fn find_live_tagged_unselected_tasks(&self) -> Result<Vec<i64>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.find_live_tagged_unselected_tasks()).await
    }

    pub async fn requeue_for_sdlc_phase(
        &self,
        task_id: i64,
        phase_comment: String,
    ) -> Result<(), TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.requeue_for_sdlc_phase(task_id, &phase_comment)).await
    }
}

#[derive(Clone)]
pub struct ActivityApp {
    service: Arc<WorkService>,
}

impl ActivityApp {
    pub async fn recent(
        &self,
        limit: i32,
        project_id: Option<i64>,
    ) -> Result<Vec<ActivityLog>, TwolebotError> {
        let svc = self.service.clone();
        run_blocking(move || svc.get_recent_activity(limit, project_id)).await
    }
}

#[derive(Clone)]
pub struct SearchApp {
    pm_search: Arc<OnceCell<Arc<PmSearch>>>,
    db: WorkDb,
}

impl SearchApp {
    /// Initialize PM semantic search. Safe to call multiple times (only first wins).
    pub async fn init(&self, embedder: Arc<Embedder>) {
        let db = self.db.clone();
        self.pm_search
            .get_or_init(|| async { Arc::new(PmSearch::new(db, embedder)) })
            .await;
    }

    fn get_pm(&self) -> Result<Arc<PmSearch>, TwolebotError> {
        self.pm_search
            .get()
            .cloned()
            .ok_or_else(|| TwolebotError::work("PM semantic search not initialized".to_string()))
    }

    pub async fn semantic_search(
        &self,
        query: String,
        project_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<PmSearchResult>, TwolebotError> {
        let pm = self.get_pm()?;
        run_blocking(move || pm.search(&query, limit, project_id)).await
    }

    pub async fn reindex(&self) -> Result<PmIndexStats, TwolebotError> {
        let pm = self.get_pm()?;
        run_blocking(move || pm.index_all()).await
    }

    pub async fn index_entity(
        &self,
        entity_type: String,
        entity_id: i64,
    ) -> Result<(), TwolebotError> {
        let pm = match self.pm_search.get() {
            Some(pm) => pm.clone(),
            None => return Ok(()), // silently skip if not initialized
        };
        run_blocking(move || pm.index_entity(&entity_type, entity_id)).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateTaskInput {
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CreateDocumentInput {
    pub project_id: i64,
    pub document_type: DocumentType,
    pub title: String,
    pub content: Option<String>,
}

