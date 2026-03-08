use std::sync::Arc;

use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};

use crate::types::work as wt;
use crate::work::{
    app::{CreateDocumentInput, CreateTaskInput},
    models::{DocumentType, TaskPriority, TaskStatus, TaskUpdate},
    WorkApp,
};

#[derive(Clone)]
pub struct WorkTools {
    app: Arc<WorkApp>,
    tool_router: ToolRouter<Self>,
}

impl WorkTools {
    pub fn new(app: Arc<WorkApp>) -> Self {
        Self {
            app,
            tool_router: Self::create_tool_router(),
        }
    }

    pub fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    fn create_tool_router() -> ToolRouter<Self> {
        Self::tool_router()
    }

    fn to_mcp_err(e: impl std::fmt::Display) -> McpError {
        McpError::internal_error(e.to_string(), None)
    }

    /// Fire-and-forget re-index of a single entity after mutation.
    fn reindex_entity(&self, entity_type: &'static str, entity_id: i64) {
        let search = self.app.search.clone();
        tokio::spawn(async move {
            if let Err(e) = search.index_entity(entity_type.to_string(), entity_id).await {
                tracing::debug!("PM reindex {entity_type}#{entity_id} failed: {e}");
            }
        });
    }

    fn json_result(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
        let json = serde_json::to_string_pretty(value)
            .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_router]
impl WorkTools {
    #[tool(
        name = "project_list",
        description = "List projects with optional filters (active_only, limit, git_remote_url)"
    )]
    async fn list_projects(
        &self,
        req: Parameters<wt::ListProjectsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .projects
            .list(
                req.active_only.unwrap_or(true),
                req.limit.unwrap_or(20),
                req.git_remote_url,
            )
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(name = "project_get", description = "Get project by ID")]
    async fn get_project(
        &self,
        req: Parameters<wt::GetProjectRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .projects
            .get(req.0.project_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "project_get_by_git_remote",
        description = "Find project by git remote URL"
    )]
    async fn get_project_by_git_remote(
        &self,
        req: Parameters<wt::GetProjectByGitRemoteRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .projects
            .get_by_git_remote(req.0.git_remote_url)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(name = "project_create", description = "Create a new project")]
    async fn create_project(
        &self,
        req: Parameters<wt::CreateProjectRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .projects
            .create(
                req.name,
                req.description,
                req.tags,
                req.git_remote_url,
            )
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(name = "project_update", description = "Update project properties")]
    async fn update_project(
        &self,
        req: Parameters<wt::UpdateProjectRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
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
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "project_archive",
        description = "Archive (deactivate) a project"
    )]
    async fn archive_project(
        &self,
        req: Parameters<wt::ArchiveProjectRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .projects
            .archive(req.0.project_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Project archived",
        )]))
    }

    #[tool(
        name = "task_list",
        description = "List tasks with status/project filters"
    )]
    async fn list_tasks(
        &self,
        req: Parameters<wt::ListTasksRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let limit = req.limit.unwrap_or(50);
        if req.compact.unwrap_or(false) {
            let result = self
                .app
                .tasks
                .list_compact(req.project_id, req.status, limit)
                .await
                .map_err(Self::to_mcp_err)?;
            Self::json_result(&result)
        } else {
            let result = self
                .app
                .tasks
                .list(req.project_id, req.status, limit)
                .await
                .map_err(Self::to_mcp_err)?;
            Self::json_result(&result)
        }
    }

    #[tool(name = "task_get", description = "Get task by ID with dependencies")]
    async fn get_task(
        &self,
        req: Parameters<wt::GetTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .tasks
            .get(req.0.task_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(name = "task_create", description = "Create a new task")]
    async fn create_task(
        &self,
        req: Parameters<wt::CreateTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let status = req
            .status
            .as_ref()
            .map(|s| s.parse::<TaskStatus>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e, None))?;
        let priority = req
            .priority
            .as_ref()
            .map(|p| p.parse::<TaskPriority>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let input = CreateTaskInput {
            project_id: req.project_id,
            title: req.title,
            description: req.description,
            status,
            priority,
            tags: req.tags,
        };

        let result = self
            .app
            .tasks
            .create(input)
            .await
            .map_err(Self::to_mcp_err)?;
        self.reindex_entity("task", result.id);
        Self::json_result(&result)
    }

    #[tool(
        name = "task_update",
        description = "Update task properties (partial update: only provided fields change)"
    )]
    async fn update_task(
        &self,
        req: Parameters<wt::UpdateTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let status = req
            .status
            .as_ref()
            .map(|s| s.parse::<TaskStatus>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e, None))?;
        let priority = req
            .priority
            .as_ref()
            .map(|p| p.parse::<TaskPriority>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e, None))?;
        let update = TaskUpdate {
            title: req.title,
            description: req.description,
            status,
            priority,
            tags: req.tags,
            comment: req.comment,
            ..Default::default()
        };
        let result = self
            .app
            .tasks
            .update(req.task_id, update)
            .await
            .map_err(Self::to_mcp_err)?;
        self.reindex_entity("task", result.id);
        Self::json_result(&result)
    }

    #[tool(
        name = "task_take_next",
        description = "Start highest-priority todo task (workflow operation)"
    )]
    async fn take_next_task(
        &self,
        req: Parameters<wt::TakeNextRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .tasks
            .take_next(req.project_id, req.force.unwrap_or(false))
            .await
            .map_err(Self::to_mcp_err)?;
        match result {
            Some(take) => Self::json_result(&take),
            None => Ok(CallToolResult::success(vec![Content::text(
                "No todo tasks available in this project",
            )])),
        }
    }

    #[tool(
        name = "task_take_next_review",
        description = "Start highest-priority review task (workflow operation)"
    )]
    async fn take_next_review_task(
        &self,
        req: Parameters<wt::TakeNextRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .tasks
            .take_next_review(req.project_id, req.force.unwrap_or(false))
            .await
            .map_err(Self::to_mcp_err)?;
        match result {
            Some(take) => Self::json_result(&take),
            None => Ok(CallToolResult::success(vec![Content::text(
                "No tasks ready for review",
            )])),
        }
    }

    #[tool(
        name = "task_reject_review",
        description = "Reject a task under review (sends back to todo with comment)"
    )]
    async fn task_reject_review(
        &self,
        req: Parameters<wt::RejectReviewRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .tasks
            .reject_review(req.task_id, req.reviewer_comment)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "task_move",
        description = "Move task to top or bottom priority position"
    )]
    async fn move_task_to_top_or_bottom(
        &self,
        req: Parameters<wt::MoveTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .tasks
            .move_to_top_or_bottom(req.task_id, req.position)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "task_dependency_add",
        description = "Add a dependency between tasks"
    )]
    async fn add_task_dependency(
        &self,
        req: Parameters<wt::DependencyRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .tasks
            .add_dependency(req.0.task_id, req.0.depends_on_task_id)
            .await
            .map_err(Self::to_mcp_err)?;
        let task = self
            .app
            .tasks
            .get(req.0.task_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&task)
    }

    #[tool(
        name = "task_dependency_remove",
        description = "Remove a dependency between tasks"
    )]
    async fn remove_task_dependency(
        &self,
        req: Parameters<wt::DependencyRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .tasks
            .remove_dependency(req.0.task_id, req.0.depends_on_task_id)
            .await
            .map_err(Self::to_mcp_err)?;
        let task = self
            .app
            .tasks
            .get(req.0.task_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&task)
    }

    #[tool(name = "doc_search", description = "Full-text search documents")]
    async fn search_documents(
        &self,
        req: Parameters<wt::SearchDocumentsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .documents
            .search(req.query, req.project_id, req.limit.unwrap_or(20))
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(name = "doc_get", description = "Get document by ID")]
    async fn get_document(
        &self,
        req: Parameters<wt::GetDocumentRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .documents
            .get(req.0.document_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "doc_read",
        description = "Read document content by ID. Returns line-numbered content (like the Read file tool). By default reads the entire document. Use offset and limit for large documents to read specific line ranges."
    )]
    async fn read_document_content(
        &self,
        req: Parameters<wt::DocReadRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .documents
            .read_content(req.document_id, req.offset, req.limit)
            .await
            .map_err(Self::to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "doc_edit",
        description = "Exact string replacement in a document (like the Edit file tool). The old_string must be unique in the document — provide more surrounding context if it isn't. Use replace_all for rename-style operations across the document. You should doc_read the document first."
    )]
    async fn edit_document_content(
        &self,
        req: Parameters<wt::DocEditRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .documents
            .edit_content(
                req.document_id,
                req.old_string,
                req.new_string,
                req.replace_all.unwrap_or(false),
            )
            .await
            .map_err(Self::to_mcp_err)?;
        self.reindex_entity("document", result.id);
        Self::json_result(&result)
    }

    #[tool(name = "doc_create", description = "Create a new document")]
    async fn create_document(
        &self,
        req: Parameters<wt::CreateDocumentRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let document_type: DocumentType = req
            .document_type
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;
        let input = CreateDocumentInput {
            project_id: req.project_id,
            document_type,
            title: req.title,
            content: req.content,
        };
        let result = self
            .app
            .documents
            .create(input)
            .await
            .map_err(Self::to_mcp_err)?;
        self.reindex_entity("document", result.id);
        Self::json_result(&result)
    }

    #[tool(
        name = "doc_update",
        description = "Update document content (increments version)"
    )]
    async fn update_document(
        &self,
        req: Parameters<wt::UpdateDocumentRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let document_type: DocumentType = req
            .document_type
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;
        let result = self
            .app
            .documents
            .update(
                req.document_id,
                req.title,
                req.content,
                document_type,
            )
            .await
            .map_err(Self::to_mcp_err)?;
        self.reindex_entity("document", result.id);
        Self::json_result(&result)
    }

    #[tool(name = "doc_delete", description = "Soft-delete a document")]
    async fn delete_document(
        &self,
        req: Parameters<wt::DeleteDocumentRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .documents
            .delete(req.0.document_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Document deleted",
        )]))
    }

    #[tool(name = "comment_list_task", description = "List comments on a task")]
    async fn list_task_comments(
        &self,
        req: Parameters<wt::ListTaskCommentsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .comments
            .list_task(req.task_id, req.limit.unwrap_or(20), req.page.unwrap_or(1))
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "comment_list_document",
        description = "List comments on a document"
    )]
    async fn list_document_comments(
        &self,
        req: Parameters<wt::ListDocumentCommentsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .comments
            .list_document(
                req.document_id,
                req.limit.unwrap_or(20),
                req.page.unwrap_or(1),
            )
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(name = "comment_upsert", description = "Create or update a comment")]
    async fn upsert_comment(
        &self,
        req: Parameters<wt::UpsertCommentRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
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
            .map_err(Self::to_mcp_err)?;
        self.reindex_entity("comment", result.id);
        Self::json_result(&result)
    }

    #[tool(name = "comment_delete", description = "Delete a comment")]
    async fn delete_comment(
        &self,
        req: Parameters<wt::DeleteCommentRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .comments
            .delete(req.0.comment_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Comment deleted",
        )]))
    }

    #[tool(
        name = "activity_recent",
        description = "View recent activity across projects"
    )]
    async fn get_recent_activity(
        &self,
        req: Parameters<wt::RecentActivityRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let result = self
            .app
            .activity
            .recent(req.limit.unwrap_or(50), req.project_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "task_analytics",
        description = "Task status breakdown with averages"
    )]
    async fn get_task_status_analytics(
        &self,
        req: Parameters<wt::AnalyticsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .tasks
            .analytics(req.0.project_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "selection_move",
        description = "Move a live board selection to top or bottom of the queue"
    )]
    async fn move_selection(
        &self,
        req: Parameters<wt::MoveSelectionRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        self.app
            .live
            .move_selection(req.task_id, req.position)
            .await
            .map_err(Self::to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Selection moved",
        )]))
    }

    #[tool(
        name = "live_board_get",
        description = "Get board state (backlog + selected + stats)"
    )]
    async fn get_live_board(
        &self,
        req: Parameters<wt::GetLiveBoardRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .live
            .get_live_board(req.0.backlog_limit)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "live_board_select",
        description = "Select todo tasks onto the queue"
    )]
    async fn select_tasks(
        &self,
        req: Parameters<wt::SelectTasksRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .live
            .select_tasks(req.0.task_ids)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "live_board_deselect",
        description = "Remove a task from the queue"
    )]
    async fn deselect_task(
        &self,
        req: Parameters<wt::DeselectTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .live
            .deselect_task(req.0.task_id)
            .await
            .map_err(Self::to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Task deselected from queue",
        )]))
    }

    #[tool(
        name = "live_board_clear_completed",
        description = "Clear done/failed selections"
    )]
    async fn clear_completed_selections(
        &self,
        #[allow(unused_variables)] req: Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        let cleared = self
            .app
            .live
            .clear_completed_selections()
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&serde_json::json!({ "cleared": cleared }))
    }

    #[tool(
        name = "pm_semantic_search",
        description = "Semantic (vector similarity) search across PM content: tasks, documents, and comments. Use for natural language queries like 'voice recording feature' or 'authentication bug'. Complements doc_search (FTS5 keyword search)."
    )]
    async fn pm_semantic_search(
        &self,
        req: Parameters<wt::PmSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let req = req.0;
        let limit = req.limit.unwrap_or(10) as usize;
        let result = self
            .app
            .search
            .semantic_search(req.query, req.project_id, limit)
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&result)
    }

    #[tool(
        name = "pm_reindex",
        description = "Re-index all PM content (tasks, documents, comments) for semantic search. Only needed after bulk changes or initial setup."
    )]
    async fn pm_reindex(
        &self,
        #[allow(unused_variables)] req: Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .app
            .search
            .reindex()
            .await
            .map_err(Self::to_mcp_err)?;
        Self::json_result(&serde_json::json!({
            "tasks_indexed": result.tasks_indexed,
            "documents_indexed": result.documents_indexed,
            "comments_indexed": result.comments_indexed,
            "chunks_created": result.chunks_created,
            "skipped_unchanged": result.skipped_unchanged,
        }))
    }
}
