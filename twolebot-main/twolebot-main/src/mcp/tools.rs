use crate::cron::{service as cron_service, CronFeed};
use crate::error::TwolebotError;
use crate::storage::CronTopicStore;
use crate::telegram::send::TelegramSender;
use crate::types::cron::{
    CancelJobRequest, CloseTopicRequest, JobSummary, ListJobsRequest, ScheduleJobRequest,
    ScheduleJobResponse, SnoozeJobRequest,
};
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};
use std::sync::Arc;

fn json_result(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// MCP tool handler for cron job management
#[derive(Clone)]
pub struct CronTools {
    cron_feed: Arc<CronFeed>,
    cron_topic_store: Option<Arc<CronTopicStore>>,
    telegram_sender: Option<Arc<TelegramSender>>,
    tool_router: ToolRouter<Self>,
}

impl CronTools {
    pub fn new(cron_feed: Arc<CronFeed>) -> Self {
        Self {
            cron_feed,
            cron_topic_store: None,
            telegram_sender: None,
            tool_router: Self::create_tool_router(),
        }
    }

    pub fn with_topic_management(
        mut self,
        cron_topic_store: Arc<CronTopicStore>,
        telegram_sender: Arc<TelegramSender>,
    ) -> Self {
        self.cron_topic_store = Some(cron_topic_store);
        self.telegram_sender = Some(telegram_sender);
        self
    }

    pub fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    fn create_tool_router() -> ToolRouter<Self> {
        Self::tool_router()
    }
}

#[tool_router]
impl CronTools {
    #[tool(
        name = "cron_schedule",
        description = "Schedule a new cron job. Provide either 'in_minutes' for a one-shot job, 'cron' for a recurring job with a cron expression, or 'crons' for a recurring job with multiple trigger times (shares a single topic)."
    )]
    async fn schedule_job(
        &self,
        request: Parameters<ScheduleJobRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;
        let job =
            cron_service::schedule_job(self.cron_feed.as_ref(), request).map_err(|e| match e {
                TwolebotError::Cron { message } => McpError::invalid_params(message, None),
                other => McpError::internal_error(format!("Failed to create job: {}", other), None),
            })?;

        let response = ScheduleJobResponse {
            job_id: job.id.clone(),
            next_run: job.next_run.map(|t| t.to_rfc3339()),
        };

        json_result(&response)
    }

    #[tool(
        name = "cron_list",
        description = "List scheduled cron jobs. Filter by status: 'active' (default), 'paused', or 'all'."
    )]
    async fn list_jobs(
        &self,
        request: Parameters<ListJobsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;
        let limit = request.limit.unwrap_or(50);
        let offset = request.offset.unwrap_or(0);
        let status = request.status.as_deref().unwrap_or("active");
        let jobs =
            cron_service::list_jobs(self.cron_feed.as_ref(), status).map_err(|e| match e {
                TwolebotError::Cron { message } => McpError::invalid_params(message, None),
                other => McpError::internal_error(format!("Failed to list jobs: {}", other), None),
            })?;

        let summaries: Vec<JobSummary> = jobs
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|j| JobSummary {
                job_id: j.id,
                name: j.name,
                schedule: j.schedule.description(),
                next_run: j.next_run.map(|t| t.to_rfc3339()),
                last_run: j.last_run.map(|t| t.to_rfc3339()),
                status: format!("{:?}", j.status).to_lowercase(),
            })
            .collect();

        json_result(&summaries)
    }

    #[tool(
        name = "cron_cancel",
        description = "Cancel a scheduled cron job. This also removes any pending executions for the job."
    )]
    async fn cancel_job(
        &self,
        request: Parameters<CancelJobRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;
        let (job, cancelled_execs) = cron_service::cancel_job(
            self.cron_feed.as_ref(),
            &request.job_id,
        )
        .map_err(|e| match e {
            TwolebotError::NotFound { .. } => {
                McpError::invalid_params(format!("Job '{}' not found", request.job_id), None)
            }
            other => McpError::internal_error(format!("Failed to cancel job: {}", other), None),
        })?;

        let message = format!(
            "Cancelled job '{}' ({}). {} pending executions removed.",
            job.display_name(),
            job.id,
            cancelled_execs
        );

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    #[tool(
        name = "cron_snooze",
        description = "Snooze a job by delaying its next execution by the specified number of minutes."
    )]
    async fn snooze_job(
        &self,
        request: Parameters<SnoozeJobRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;
        let job =
            cron_service::snooze_job(self.cron_feed.as_ref(), &request.job_id, request.minutes)
                .map_err(|e| match e {
                    TwolebotError::NotFound { .. } => McpError::invalid_params(
                        format!("Job '{}' not found", request.job_id),
                        None,
                    ),
                    TwolebotError::Cron { message } => McpError::invalid_params(message, None),
                    other => {
                        McpError::internal_error(format!("Failed to snooze job: {}", other), None)
                    }
                })?;

        let message = format!(
            "Snoozed job '{}'. Next run: {}",
            job.display_name(),
            job.next_run
                .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    #[tool(
        name = "cron_close_topic",
        description = "Close (delete) the Telegram forum topic associated with a cron job. Also cancels the job and removes the topic mapping. Use this to clean up cron topics that are no longer needed."
    )]
    async fn close_topic(
        &self,
        request: Parameters<CloseTopicRequest>,
    ) -> Result<CallToolResult, McpError> {
        let request = request.0;

        let store = self.cron_topic_store.as_ref().ok_or_else(|| {
            McpError::internal_error("Topic management not configured".to_string(), None)
        })?;
        let sender = self.telegram_sender.as_ref().ok_or_else(|| {
            McpError::internal_error("Telegram sender not configured".to_string(), None)
        })?;

        // Look up the job to get its name
        let job = self
            .cron_feed
            .get_job(&request.job_id)
            .map_err(|e| McpError::internal_error(format!("Failed to look up job: {}", e), None))?
            .ok_or_else(|| {
                McpError::invalid_params(format!("Job '{}' not found", request.job_id), None)
            })?;

        // Find the topic for this job
        let chat_id = request.chat_id;
        let thread_id = store
            .get(&request.job_id, chat_id)
            .map_err(|e| McpError::internal_error(format!("Failed to look up topic: {}", e), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!("No topic found for job '{}' in chat {}", request.job_id, chat_id),
                    None,
                )
            })?;

        // Delete the Telegram topic
        if let Err(e) = sender.delete_forum_topic(chat_id, thread_id).await {
            tracing::warn!(
                "Failed to delete forum topic {} for job '{}': {} (may already be deleted)",
                thread_id,
                job.display_name(),
                e
            );
        }

        // Remove the topic mapping
        if let Err(e) = store.remove(&request.job_id) {
            tracing::warn!("Failed to remove topic mapping: {}", e);
        }

        // Cancel the job too
        let cancelled_msg = match cron_service::cancel_job(self.cron_feed.as_ref(), &request.job_id)
        {
            Ok((_, n)) => format!(", cancelled job ({} pending executions removed)", n),
            Err(_) => String::new(),
        };

        let message = format!(
            "Closed topic for '{}' (thread {} in chat {}){}",
            job.display_name(),
            thread_id,
            chat_id,
            cancelled_msg,
        );

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::{CronJob, CronJobStatus, CronSchedule};
    use chrono::{Duration, Utc};
    use tempfile::TempDir;

    fn create_test_tools() -> (CronTools, TempDir) {
        let dir = TempDir::new().unwrap();
        let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        (CronTools::new(feed), dir)
    }

    #[tokio::test]
    async fn test_schedule_one_shot_job() {
        let (tools, _tmpdir) = create_test_tools();

        let request = Parameters(ScheduleJobRequest {
            prompt: "Test prompt".to_string(),
            name: "Test job".to_string(),
            in_minutes: Some(10),
            cron: None,
            crons: None,
            origin_chat_id: None,
        });

        let result = tools.schedule_job(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify job was created
        let jobs = tools.cron_feed.list_active_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Test job");
    }

    #[tokio::test]
    async fn test_schedule_cron_job() {
        let (tools, _tmpdir) = create_test_tools();

        let request = Parameters(ScheduleJobRequest {
            prompt: "Hourly check".to_string(),
            name: "Hourly check".to_string(),
            in_minutes: None,
            // cron crate expects 6 fields: sec min hour day-of-month month day-of-week
            cron: Some("0 0 * * * *".to_string()),
            crons: None,
            origin_chat_id: None,
        });

        let result = tools.schedule_job(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_schedule_multi_cron_job() {
        let (tools, _tmpdir) = create_test_tools();

        let request = Parameters(ScheduleJobRequest {
            prompt: "Study check-in".to_string(),
            name: "Study session".to_string(),
            in_minutes: None,
            cron: None,
            crons: Some(vec![
                "0 0 6 * * *".to_string(),
                "0 30 6 * * *".to_string(),
                "0 0 7 * * *".to_string(),
            ]),
            origin_chat_id: None,
        });

        let result = tools.schedule_job(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify single job was created with multi-cron schedule
        let jobs = tools.cron_feed.list_active_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Study session");
        assert_eq!(jobs[0].schedule.all_expressions().len(), 3);
        assert!(jobs[0].next_run.is_some());
    }

    #[tokio::test]
    async fn test_schedule_rejects_cron_and_crons_together() {
        let (tools, _tmpdir) = create_test_tools();

        let request = Parameters(ScheduleJobRequest {
            prompt: "Test".to_string(),
            name: "Test".to_string(),
            in_minutes: None,
            cron: Some("0 0 6 * * *".to_string()),
            crons: Some(vec!["0 30 6 * * *".to_string()]),
            origin_chat_id: None,
        });

        let result = tools.schedule_job(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schedule_rejects_invalid_expression_in_crons() {
        let (tools, _tmpdir) = create_test_tools();

        let request = Parameters(ScheduleJobRequest {
            prompt: "Test".to_string(),
            name: "Test".to_string(),
            in_minutes: None,
            cron: None,
            crons: Some(vec![
                "0 0 6 * * *".to_string(),
                "not valid".to_string(),
            ]),
            origin_chat_id: None,
        });

        let result = tools.schedule_job(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let (tools, _tmpdir) = create_test_tools();

        // Create some jobs
        tools
            .cron_feed
            .create_job(CronJob::new("Job 1", "Job 1", CronSchedule::from_minutes(10)))
            .unwrap();
        // cron crate expects 6 fields
        tools
            .cron_feed
            .create_job(CronJob::new("Job 2", "Job 2", CronSchedule::cron("0 * * * * *")))
            .unwrap();

        let request = Parameters(ListJobsRequest {
            status: None,
            limit: None,
            offset: None,
        });
        let result = tools.list_jobs(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_cancel_job() {
        let (tools, _tmpdir) = create_test_tools();

        let job = tools
            .cron_feed
            .create_job(CronJob::new("To cancel", "To cancel", CronSchedule::from_minutes(10)))
            .unwrap();

        let request = Parameters(CancelJobRequest {
            job_id: job.id.clone(),
        });
        let result = tools.cancel_job(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify cancelled
        let cancelled = tools.cron_feed.get_job(&job.id).unwrap().unwrap();
        assert_eq!(cancelled.status, CronJobStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_snooze_job() {
        let (tools, _tmpdir) = create_test_tools();

        // cron crate expects 6 fields
        let job = tools
            .cron_feed
            .create_job(
                CronJob::new("To snooze", "To snooze", CronSchedule::cron("0 * * * * *"))
                    .with_next_run(Utc::now()),
            )
            .unwrap();

        let original_next = job.next_run;

        let request = Parameters(SnoozeJobRequest {
            job_id: job.id.clone(),
            minutes: 30,
        });
        let result = tools.snooze_job(request).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));

        // Verify next_run was updated
        let snoozed = tools.cron_feed.get_job(&job.id).unwrap().unwrap();
        assert!(snoozed.next_run.is_some());
        if let Some(orig) = original_next {
            assert!(snoozed.next_run.unwrap() > orig);
        }
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_prompt() -> impl Strategy<Value = String> {
            // Non-empty strings with various characters including unicode
            prop::string::string_regex(".{1,500}").unwrap()
        }

        fn arb_job_name() -> impl Strategy<Value = String> {
            prop::string::string_regex("[a-zA-Z0-9 _-]{1,50}").unwrap()
        }

        fn arb_positive_minutes() -> impl Strategy<Value = i64> {
            1i64..1440 // 1 minute to 24 hours
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(20))]

            #[test]
            fn prop_schedule_roundtrip_preserves_prompt(prompt in arb_prompt(), name in arb_job_name()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let dir = TempDir::new().unwrap();
                    let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
                    let tools = CronTools::new(feed);

                    let request = Parameters(ScheduleJobRequest {
                        prompt: prompt.clone(),
                        name: name.to_string(),
                        in_minutes: Some(10),
                        cron: None,
                        crons: None,
                        origin_chat_id: None,
                    });

                    let result = tools.schedule_job(request).await.unwrap();
                    assert!(!result.is_error.unwrap_or(false));

                    // Verify prompt is preserved exactly
                    let jobs = tools.cron_feed.list_active_jobs().unwrap();
                    assert_eq!(jobs.len(), 1);
                    assert_eq!(jobs[0].prompt, prompt);
                    assert_eq!(jobs[0].name, name);
                });
            }

            #[test]
            fn prop_snooze_increases_next_run(minutes in arb_positive_minutes()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let dir = TempDir::new().unwrap();
                    let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
                    let tools = CronTools::new(feed);

                    // Create job with known next_run
                    let now = Utc::now();
                    let job = tools.cron_feed.create_job(
                        CronJob::new("Test", "Test", CronSchedule::cron("0 * * * * *"))
                            .with_next_run(now)
                    ).unwrap();

                    let request = Parameters(SnoozeJobRequest {
                        job_id: job.id.clone(),
                        minutes,
                    });

                    let result = tools.snooze_job(request).await.unwrap();
                    assert!(!result.is_error.unwrap_or(false));

                    let snoozed = tools.cron_feed.get_job(&job.id).unwrap().unwrap();
                    let new_next = snoozed.next_run.unwrap();

                    // New next_run should be at least `minutes` from now
                    // (with small tolerance for test execution time)
                    let expected_min = Utc::now() + Duration::minutes(minutes) - Duration::seconds(5);
                    assert!(new_next >= expected_min, "next_run {} should be >= {}", new_next, expected_min);
                });
            }

            #[test]
            fn prop_create_cancel_lifecycle(prompt in arb_prompt()) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let dir = TempDir::new().unwrap();
                    let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
                    let tools = CronTools::new(feed);

                    // Create
                    let create_req = Parameters(ScheduleJobRequest {
                        prompt,
                        name: "Test job".to_string(),
                        in_minutes: Some(10),
                        cron: None,
                        crons: None,
                        origin_chat_id: None,
                    });
                    tools.schedule_job(create_req).await.unwrap();

                    let jobs = tools.cron_feed.list_active_jobs().unwrap();
                    assert_eq!(jobs.len(), 1);
                    let job_id = jobs[0].id.clone();

                    // Cancel
                    let cancel_req = Parameters(CancelJobRequest { job_id: job_id.clone() });
                    tools.cancel_job(cancel_req).await.unwrap();

                    // Verify cancelled
                    let job = tools.cron_feed.get_job(&job_id).unwrap().unwrap();
                    assert_eq!(job.status, CronJobStatus::Cancelled);

                    // Active list should be empty
                    assert!(tools.cron_feed.list_active_jobs().unwrap().is_empty());
                });
            }

            #[test]
            fn prop_multiple_jobs_all_returned(count in 1usize..10) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let dir = TempDir::new().unwrap();
                    let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
                    let tools = CronTools::new(feed);

                    // Create N jobs
                    for i in 0..count {
                        let request = Parameters(ScheduleJobRequest {
                            prompt: format!("Job {}", i),
                            name: format!("Name {}", i),
                            in_minutes: Some(10 + i as i64),
                            cron: None,
                            crons: None,
                            origin_chat_id: None,
                        });
                        tools.schedule_job(request).await.unwrap();
                    }

                    // List all
                    let request = Parameters(ListJobsRequest { status: Some("all".to_string()), limit: None, offset: None });
                    let result = tools.list_jobs(request).await.unwrap();
                    assert!(!result.is_error.unwrap_or(false));

                    // Verify count
                    let jobs = tools.cron_feed.list_all_jobs().unwrap();
                    assert_eq!(jobs.len(), count);
                });
            }
        }

        #[tokio::test]
        async fn test_negative_minutes_accepted() {
            let dir = TempDir::new().unwrap();
            let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
            let tools = CronTools::new(feed);

            let request = Parameters(ScheduleJobRequest {
                prompt: "Test".to_string(),
                name: "Test".to_string(),
                in_minutes: Some(-5),
                cron: None,
                crons: None,
                origin_chat_id: None,
            });

            // This should create a job in the past, which is valid but will fire immediately
            // The cron system allows this for "run now" semantics
            let result = tools.schedule_job(request).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_invalid_cron_expression_rejected() {
            let dir = TempDir::new().unwrap();
            let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
            let tools = CronTools::new(feed);

            let request = Parameters(ScheduleJobRequest {
                prompt: "Test".to_string(),
                name: "Test".to_string(),
                in_minutes: None,
                cron: Some("not a valid cron".to_string()),
                crons: None,
                origin_chat_id: None,
            });

            let result = tools.schedule_job(request).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_cancel_nonexistent_job_fails() {
            let dir = TempDir::new().unwrap();
            let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
            let tools = CronTools::new(feed);

            let request = Parameters(CancelJobRequest {
                job_id: "nonexistent-job-id".to_string(),
            });

            let result = tools.cancel_job(request).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_snooze_nonexistent_job_fails() {
            let dir = TempDir::new().unwrap();
            let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
            let tools = CronTools::new(feed);

            let request = Parameters(SnoozeJobRequest {
                job_id: "nonexistent-job-id".to_string(),
                minutes: 30,
            });

            let result = tools.snooze_job(request).await;
            assert!(result.is_err());
        }
    }
}
