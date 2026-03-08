use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::storage::{
    ActiveChatRegistry, MainTopicStore, PromptFeed, PromptItem, PromptSource, Protocol,
    ResponseFeed, SettingsStore,
};
use crate::work::models::*;
use crate::work::service::WorkService;

const MAX_CONSECUTIVE_FAILURES: u32 = 3;
const MAX_HARDEN_ITERATIONS: u32 = 5;
#[cfg(not(test))]
const COOLDOWN_BETWEEN_TASKS: Duration = Duration::from_secs(5);
#[cfg(test)]
const COOLDOWN_BETWEEN_TASKS: Duration = Duration::from_millis(50);
#[cfg(not(test))]
const RESPONSE_POLL_INTERVAL: Duration = Duration::from_millis(500);
#[cfg(test)]
const RESPONSE_POLL_INTERVAL: Duration = Duration::from_millis(50);
#[cfg(not(test))]
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
#[cfg(test)]
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// SDLC phase for a task, tracked in-memory by the agent loop.
/// The iteration counter is the source of truth — not derived from comments.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum SdlcPhase {
    Dev,
    Harden { iteration: u32 },
}

impl SdlcPhase {
    /// Start the harden phase at iteration 0.
    fn new_harden() -> Self {
        SdlcPhase::Harden { iteration: 0 }
    }

    /// Advance to the next harden iteration.
    fn next_iteration(&self) -> Self {
        match self {
            SdlcPhase::Harden { iteration } => SdlcPhase::Harden {
                iteration: iteration + 1,
            },
            other => other.clone(),
        }
    }

    /// True when the iteration counter has reached the maximum (no more passes allowed).
    fn is_exhausted(&self) -> bool {
        matches!(self, SdlcPhase::Harden { iteration } if *iteration >= MAX_HARDEN_ITERATIONS)
    }
}

/// Check if a task has the `live` tag (indicating SDLC auto-execution).
fn is_live_tagged(task: &TaskModel) -> bool {
    task.tags.iter().any(|t| t == "live")
}

/// Agent loop that works through the live-board selection queue.
///
/// Integrates with the existing ClaudeManager prompt queue infrastructure:
/// - Enqueues prompts via PromptFeed (same as Telegram messages)
/// - Polls for responses via ResponseFeed
/// - ClaudeManager picks up and processes prompts automatically
pub struct AgentLoop {
    service: Arc<WorkService>,
    prompt_feed: Arc<PromptFeed>,
    response_feed: Arc<ResponseFeed>,
    settings_store: Arc<SettingsStore>,
    main_topic_store: Option<Arc<MainTopicStore>>,
    active_chats: Option<Arc<ActiveChatRegistry>>,
    state: Arc<Mutex<AgentLoopState>>,
    cancel_token: Arc<Mutex<CancellationToken>>,
}

impl AgentLoop {
    pub fn new(
        service: Arc<WorkService>,
        prompt_feed: Arc<PromptFeed>,
        response_feed: Arc<ResponseFeed>,
        settings_store: Arc<SettingsStore>,
    ) -> Self {
        let state = service.agent_loop_state().clone();
        Self {
            service,
            prompt_feed,
            response_feed,
            settings_store,
            main_topic_store: None,
            active_chats: None,
            state,
            cancel_token: Arc::new(Mutex::new(CancellationToken::new())),
        }
    }

    /// Enable topic-aware routing for agent loop prompts
    pub fn with_topic_routing(
        mut self,
        main_topic_store: Arc<MainTopicStore>,
        active_chats: Arc<ActiveChatRegistry>,
    ) -> Self {
        self.main_topic_store = Some(main_topic_store);
        self.active_chats = Some(active_chats);
        self
    }

    /// Start the agent loop. Spawns a tokio task that processes the selection queue.
    /// Returns error if already Running or if selection queue is empty.
    pub async fn start(&self) -> Result<(), crate::TwolebotError> {
        let mut state = self.state.lock().await;

        if *state == AgentLoopState::Running {
            return Err(crate::TwolebotError::work(
                "agent loop is already running".to_string(),
            ));
        }

        // Check that there are queued or paused tasks
        let has_tasks = {
            let svc = self.service.clone();
            tokio::task::spawn_blocking(move || svc.get_next_selected_task())
                .await
                .map_err(|e| crate::TwolebotError::work(format!("spawn_blocking: {e}")))?
        }?;

        if has_tasks.is_none() {
            return Err(crate::TwolebotError::work(
                "no tasks in selection queue (queued or paused)".to_string(),
            ));
        }

        // Reset cancel token for new run
        // We need a new token each time since cancelled tokens stay cancelled
        let new_token = CancellationToken::new();
        *self.cancel_token.lock().await = new_token.clone();

        *state = AgentLoopState::Running;
        drop(state);

        // Broadcast state change
        self.service
            .event_tx()
            .send(WorkEvent::AgentProgress {
                task_id: 0,
                message: "Agent loop started".to_string(),
            })
            .ok();

        // Spawn the loop task
        let service = self.service.clone();
        let prompt_feed = self.prompt_feed.clone();
        let response_feed = self.response_feed.clone();
        let settings_store = self.settings_store.clone();
        let main_topic_store = self.main_topic_store.clone();
        let active_chats = self.active_chats.clone();
        let loop_state = self.state.clone();
        let token = new_token;

        tokio::spawn(async move {
            run_loop(
                service,
                prompt_feed,
                response_feed,
                settings_store,
                main_topic_store,
                active_chats,
                loop_state,
                token,
            )
            .await;
        });

        Ok(())
    }

    /// Stop the agent loop. The current task is allowed to finish.
    pub async fn stop(&self) -> Result<(), crate::TwolebotError> {
        let mut state = self.state.lock().await;

        if *state != AgentLoopState::Running {
            return Err(crate::TwolebotError::work(
                "agent loop is not running".to_string(),
            ));
        }

        *state = AgentLoopState::Paused;
        self.cancel_token.lock().await.cancel();

        self.service
            .event_tx()
            .send(WorkEvent::AgentProgress {
                task_id: 0,
                message: "Agent loop stopping (finishing current task)".to_string(),
            })
            .ok();

        Ok(())
    }

    /// Get current agent loop state.
    pub async fn state(&self) -> AgentLoopState {
        self.state.lock().await.clone()
    }
}

/// Resolve the Main topic routing info for agent loop prompts.
fn resolve_agent_routing(
    main_topic_store: &Option<Arc<MainTopicStore>>,
    active_chats: &Option<Arc<ActiveChatRegistry>>,
) -> (Option<i64>, Option<i64>) {
    let Some(ref store) = main_topic_store else {
        return (None, None);
    };
    let Some(ref chats) = active_chats else {
        return (None, None);
    };

    let targets = chats.get_broadcast_targets_all_users();
    let tg_chat_id = targets
        .iter()
        .find(|(_, proto, _)| *proto == Protocol::Telegram)
        .and_then(|(_, _, cid)| cid.parse::<i64>().ok());

    let Some(chat_id) = tg_chat_id else {
        return (None, None);
    };

    let thread_id = store.get(chat_id).ok().flatten();
    (Some(chat_id), thread_id)
}

/// The main agent loop. Runs in a spawned task.
async fn run_loop(
    service: Arc<WorkService>,
    prompt_feed: Arc<PromptFeed>,
    response_feed: Arc<ResponseFeed>,
    settings_store: Arc<SettingsStore>,
    main_topic_store: Option<Arc<MainTopicStore>>,
    active_chats: Option<Arc<ActiveChatRegistry>>,
    state: Arc<Mutex<AgentLoopState>>,
    cancel_token: CancellationToken,
) {
    let mut consecutive_failures: u32 = 0;

    'outer: loop {
        // Check if we should stop
        if cancel_token.is_cancelled() || *state.lock().await != AgentLoopState::Running {
            break;
        }

        // Get next task
        let next_task = {
            let svc = service.clone();
            match tokio::task::spawn_blocking(move || svc.get_next_selected_task()).await {
                Ok(Ok(task)) => task,
                Ok(Err(e)) => {
                    tracing::error!("Agent loop: error getting next task: {e}");
                    break;
                }
                Err(e) => {
                    tracing::error!("Agent loop: spawn_blocking failed: {e}");
                    break;
                }
            }
        };

        let Some(selected_task) = next_task else {
            // No more tasks -- go to Idle
            tracing::info!("Agent loop: selection queue exhausted, going idle");
            *state.lock().await = AgentLoopState::Idle;

            service
                .event_tx()
                .send(WorkEvent::AgentProgress {
                    task_id: 0,
                    message: "All selected tasks completed".to_string(),
                })
                .ok();
            break;
        };

        let task_id = selected_task.task.id;
        let task_title = selected_task.task.title.clone();
        let task_is_live = is_live_tagged(&selected_task.task);

        tracing::info!(
            "Agent loop: starting task #{task_id}: {task_title} (live={task_is_live})"
        );

        // Start the task (mark as active)
        {
            let svc = service.clone();
            match tokio::task::spawn_blocking(move || svc.start_selected_task(task_id)).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    tracing::error!("Agent loop: failed to start task #{task_id}: {e}");
                    consecutive_failures += 1;
                    if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        tracing::warn!("Agent loop: max consecutive failures reached, pausing");
                        *state.lock().await = AgentLoopState::Paused;
                        break;
                    }
                    tokio::time::sleep(COOLDOWN_BETWEEN_TASKS).await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Agent loop: spawn_blocking failed: {e}");
                    break;
                }
            }
        }

        service
            .event_tx()
            .send(WorkEvent::AgentProgress {
                task_id,
                message: format!("Working on: {task_title}"),
            })
            .ok();

        // Resolve routing so agent-loop prompts go to the Main topic
        let (agent_chat_id, agent_thread_id) =
            resolve_agent_routing(&main_topic_store, &active_chats);

        // ── Dev phase ────────────────────────────────────────────────────
        let settings = settings_store.get();
        let dev_prompt = build_dev_prompt(&selected_task, &settings.dev_role_prompt);
        let dev_outcome = send_and_poll(
            &prompt_feed, &response_feed, &cancel_token, &service,
            task_id, &task_title, dev_prompt, agent_chat_id, agent_thread_id,
        ).await;

        let _dev_response = match dev_outcome {
            PhaseOutcome::Response(text) => {
                consecutive_failures = 0;
                add_summary_comment(&service, task_id, &text).await;
                text
            }
            PhaseOutcome::Cancelled => {
                let svc = service.clone();
                let _ = tokio::task::spawn_blocking(move || svc.pause_selected_task(task_id)).await;
                tracing::info!("Agent loop: paused task #{task_id} due to stop signal");
                break 'outer;
            }
            PhaseOutcome::TimedOut => {
                let svc = service.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    svc.fail_selected_task(task_id, "Dev phase timeout (10 minutes)")
                }).await;
                tracing::warn!("Agent loop: task #{task_id} dev phase timed out");
                consecutive_failures += 1;
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    tracing::warn!("Agent loop: max consecutive failures ({MAX_CONSECUTIVE_FAILURES}), pausing");
                    *state.lock().await = AgentLoopState::Paused;
                    break 'outer;
                }
                tokio::time::sleep(COOLDOWN_BETWEEN_TASKS).await;
                continue 'outer;
            }
            PhaseOutcome::Failed(reason) => {
                let svc = service.clone();
                let r = reason.clone();
                let _ = tokio::task::spawn_blocking(move || svc.fail_selected_task(task_id, &r)).await;
                tracing::error!("Agent loop: task #{task_id} dev phase failed: {reason}");
                consecutive_failures += 1;
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    tracing::warn!("Agent loop: max consecutive failures ({MAX_CONSECUTIVE_FAILURES}), pausing");
                    *state.lock().await = AgentLoopState::Paused;
                    break 'outer;
                }
                tokio::time::sleep(COOLDOWN_BETWEEN_TASKS).await;
                continue 'outer;
            }
        };

        service
            .event_tx()
            .send(WorkEvent::AgentProgress {
                task_id,
                message: format!("Dev complete: {task_title}"),
            })
            .ok();

        // ── Harden loop (live-tagged tasks only) ─────────────────────────
        let final_status = if task_is_live {
            let mut phase = SdlcPhase::new_harden();
            let mut result_status = TaskStatus::Done;

            loop {
                if phase.is_exhausted() {
                    tracing::warn!(
                        "Agent loop: task #{task_id} hit max harden iterations ({MAX_HARDEN_ITERATIONS}), completing"
                    );
                    break;
                }

                if cancel_token.is_cancelled() {
                    let svc = service.clone();
                    let _ = tokio::task::spawn_blocking(move || svc.pause_selected_task(task_id)).await;
                    tracing::info!("Agent loop: paused task #{task_id} (harden) due to stop signal");
                    break 'outer;
                }

                let iteration = match &phase {
                    SdlcPhase::Harden { iteration } => *iteration,
                    _ => unreachable!(),
                };

                service
                    .event_tx()
                    .send(WorkEvent::AgentProgress {
                        task_id,
                        message: format!("Harden {iteration}/{MAX_HARDEN_ITERATIONS}: {task_title}"),
                    })
                    .ok();

                let settings = settings_store.get();
                let harden_prompt = build_harden_prompt(&selected_task, iteration, &settings.harden_role_prompt);
                let harden_outcome = send_and_poll(
                    &prompt_feed, &response_feed, &cancel_token, &service,
                    task_id, &task_title, harden_prompt, agent_chat_id, agent_thread_id,
                ).await;

                match harden_outcome {
                    PhaseOutcome::Response(text) => {
                        add_summary_comment(&service, task_id, &text).await;

                        if text.contains("VERDICT: DONE") {
                            tracing::info!("Agent loop: task #{task_id} harden({iteration}) → DONE");
                            result_status = TaskStatus::Done;
                            break;
                        } else if text.contains("VERDICT: BLOCKED") {
                            tracing::info!("Agent loop: task #{task_id} harden({iteration}) → BLOCKED");
                            result_status = TaskStatus::ReadyForReview;
                            break;
                        } else {
                            tracing::info!(
                                "Agent loop: task #{task_id} harden({iteration}) → NEEDS_REVIEW, continuing"
                            );
                            phase = phase.next_iteration();
                        }
                    }
                    PhaseOutcome::Cancelled => {
                        let svc = service.clone();
                        let _ = tokio::task::spawn_blocking(move || svc.pause_selected_task(task_id)).await;
                        tracing::info!("Agent loop: paused task #{task_id} (harden) due to stop signal");
                        break 'outer;
                    }
                    PhaseOutcome::TimedOut => {
                        tracing::warn!("Agent loop: task #{task_id} harden({iteration}) timed out");
                        // Dev work is done; mark ready_for_review so human can pick up
                        result_status = TaskStatus::ReadyForReview;
                        consecutive_failures += 1;
                        break;
                    }
                    PhaseOutcome::Failed(reason) => {
                        tracing::error!("Agent loop: task #{task_id} harden({iteration}) failed: {reason}");
                        result_status = TaskStatus::ReadyForReview;
                        consecutive_failures += 1;
                        break;
                    }
                }
            }

            result_status
        } else {
            TaskStatus::ReadyForReview
        };

        // Complete the task with the determined status
        {
            let svc = service.clone();
            let status = final_status.clone();
            match tokio::task::spawn_blocking(move || svc.complete_selected_task(task_id, &status)).await {
                Ok(Ok(())) => {
                    tracing::info!("Agent loop: completed task #{task_id} as {final_status}");
                    service
                        .event_tx()
                        .send(WorkEvent::AgentProgress {
                            task_id,
                            message: format!("Completed ({final_status}): {task_title}"),
                        })
                        .ok();
                }
                Ok(Err(e)) => {
                    tracing::error!("Agent loop: error completing task #{task_id}: {e}");
                }
                Err(e) => {
                    tracing::error!("Agent loop: spawn_blocking failed completing task #{task_id}: {e}");
                }
            }
        }

        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            tracing::warn!(
                "Agent loop: max consecutive failures ({MAX_CONSECUTIVE_FAILURES}), pausing"
            );
            *state.lock().await = AgentLoopState::Paused;

            service
                .event_tx()
                .send(WorkEvent::AgentProgress {
                    task_id: 0,
                    message: format!("Stopped: {MAX_CONSECUTIVE_FAILURES} consecutive failures"),
                })
                .ok();
            break;
        }

        // Cooldown between tasks
        tokio::time::sleep(COOLDOWN_BETWEEN_TASKS).await;
    }
}

/// Outcome of sending a prompt and waiting for the agent's response.
enum PhaseOutcome {
    /// Agent returned a final response.
    Response(String),
    /// Loop was cancelled (stop signal received).
    Cancelled,
    /// Response timed out.
    TimedOut,
    /// Prompt enqueue or response poll failed.
    Failed(String),
}

/// Enqueue a prompt via PromptFeed and poll ResponseFeed until a response arrives.
async fn send_and_poll(
    prompt_feed: &Arc<PromptFeed>,
    response_feed: &Arc<ResponseFeed>,
    cancel_token: &CancellationToken,
    service: &Arc<WorkService>,
    task_id: i64,
    task_title: &str,
    prompt_text: String,
    agent_chat_id: Option<i64>,
    agent_thread_id: Option<i64>,
) -> PhaseOutcome {
    let prompt_item = PromptItem::new_agent_routed(
        prompt_text, task_id, task_title, agent_chat_id, agent_thread_id,
    );
    let prompt_id = prompt_item.id.clone();

    match prompt_feed.enqueue(prompt_item) {
        Ok(_) => {
            tracing::info!("Agent loop: enqueued prompt {prompt_id} for task #{task_id}");
            service
                .event_tx()
                .send(WorkEvent::AgentProgress {
                    task_id,
                    message: format!("Queued prompt {prompt_id} for task #{task_id}"),
                })
                .ok();
        }
        Err(e) => {
            tracing::error!("Agent loop: failed to enqueue prompt: {e}");
            return PhaseOutcome::Failed(format!("Failed to enqueue prompt: {e}"));
        }
    }

    match poll_for_response(response_feed, &prompt_id, cancel_token).await {
        Ok(Some(text)) => PhaseOutcome::Response(text),
        Ok(None) if cancel_token.is_cancelled() => PhaseOutcome::Cancelled,
        Ok(None) => PhaseOutcome::TimedOut,
        Err(e) => PhaseOutcome::Failed(format!("{e}")),
    }
}

/// Add a truncated agent response as a comment on a task.
async fn add_summary_comment(service: &Arc<WorkService>, task_id: i64, response: &str) {
    let truncated = if response.len() > 2000 {
        format!("{}...", &response[..2000])
    } else {
        response.to_string()
    };
    let svc = service.clone();
    let _ = tokio::task::spawn_blocking(move || {
        svc.upsert_comment(None, Some(task_id), None, &truncated, None)
    })
    .await;
}

/// Build a development prompt for the Claude agent.
/// Uses the stored role template, injecting task context and SDLC-specific arguments.
fn build_dev_prompt(selected: &SelectedTask, role_template: &str) -> String {
    let task = &selected.task;
    let comments = &selected.comments;

    let mut prompt = format!(
        "## Task #{} (project #{}): {}\n\
         Priority: {} | Status: in_progress\n\n\
         ## Description\n{}\n",
        task.id, task.project_id, task.title, task.priority, task.description,
    );

    append_comments(&mut prompt, comments);
    append_metadata(&mut prompt, task);

    let task_arguments = format!(
        "\n## SDLC Loop — Task Assignment\n\n\
         You are working on task #{task_id} (assigned automatically).\n\n\
         **Workflow rules:**\n\
         1. Comments listed above are authoritative context. Use them before taking actions.\n\
         2. When you move task status, you MUST add a comment stating your reason.\n\
         3. Do not move status without an explicit reason comment.\n\
         4. If blocked, include exactly what is blocking and what input is needed.\n\
         5. When done, mark the task as ready_for_review with a summary comment.\n\n\
         Use the available MCP tools to update task status and comments.\n",
        task_id = task.id,
    );

    // Apply the role template with $ARGUMENTS replaced
    let role_instructions = role_template.replace("$ARGUMENTS", &task_arguments);
    prompt.push_str("\n---\n\n");
    prompt.push_str(&role_instructions);

    prompt
}

/// Build a harden (review + fix) prompt for the Claude agent.
/// Uses the stored role template, injecting task context and SDLC-specific arguments.
fn build_harden_prompt(selected: &SelectedTask, iteration: u32, role_template: &str) -> String {
    let task = &selected.task;
    let comments = &selected.comments;

    let mut prompt = format!(
        "## Task #{} (project #{}): {}\n\
         Priority: {} | Status: under_review | Harden iteration {iteration}/{MAX_HARDEN_ITERATIONS}\n\n\
         ## Description\n{}\n",
        task.id, task.project_id, task.title, task.priority, task.description,
    );

    append_comments(&mut prompt, comments);
    append_metadata(&mut prompt, task);

    let task_arguments = format!(
        "\n## SDLC Loop — Harden Assignment (iteration {iteration}/{MAX_HARDEN_ITERATIONS})\n\n\
         You are reviewing task #{task_id} (assigned automatically).\n\n\
         **Steps:**\n\
         1. Read all comments to understand prior work and review history.\n\
         2. Check for pending commits: run `git diff` and `git diff --cached`.\n\
         3. Review BOTH the ticket requirements AND any pending/uncommitted changes.\n\
         4. Run the build/tests.\n\
         5. Follow the harden flow: post findings, fix issues, post fix summary.\n\n\
         **Workflow rules:**\n\
         - Comments listed above are authoritative context.\n\
         - When you move task status, you MUST add a comment with your reason.\n\
         - If blocked, include exactly what is blocking and what input is needed.\n\n\
         **REQUIRED OUTPUT** — end your response with exactly one of:\n\
         - `VERDICT: DONE` — zero issues found, task marked done\n\
         - `VERDICT: NEEDS_REVIEW` — fixes made, left in review for fresh eyes\n\
         - `VERDICT: BLOCKED` — unfixable problems\n\n\
         Use the available MCP tools to update task status and comments.\n",
        task_id = task.id,
    );

    // Apply the role template with placeholders replaced
    let role_instructions = role_template
        .replace("$ARGUMENTS", &task_arguments)
        .replace("{iteration}", &iteration.to_string())
        .replace("{max_iterations}", &MAX_HARDEN_ITERATIONS.to_string());
    prompt.push_str("\n---\n\n");
    prompt.push_str(&role_instructions);

    prompt
}

fn append_comments(prompt: &mut String, comments: &[Comment]) {
    if !comments.is_empty() {
        prompt.push_str(&format!("\n## Comments ({})\n", comments.len()));
        for comment in comments {
            prompt.push_str(&format!(
                "[{}] {}\n",
                comment.created_at, comment.content
            ));
        }
    }
}

fn append_metadata(prompt: &mut String, task: &TaskModel) {
    if !task.blocked_by.is_empty() {
        prompt.push_str(&format!(
            "\n## Dependencies\nBlocked by: {:?}\n",
            task.blocked_by
        ));
    }

    if !task.tags.is_empty() {
        prompt.push_str(&format!("\nTags: {}\n", task.tags.join(", ")));
    }
}

/// Poll the response feed for a completed response matching the given prompt ID.
/// Returns the response text, or None if cancelled/timed out.
async fn poll_for_response(
    response_feed: &Arc<ResponseFeed>,
    prompt_id: &str,
    cancel_token: &CancellationToken,
) -> Result<Option<String>, crate::TwolebotError> {
    let deadline = tokio::time::Instant::now() + RESPONSE_TIMEOUT;

    loop {
        if cancel_token.is_cancelled() {
            return Ok(None);
        }

        if tokio::time::Instant::now() > deadline {
            return Ok(None);
        }

        // Check if response is available
        let prompt_id_owned = prompt_id.to_string();
        let rf = response_feed.clone();
        let found = tokio::task::spawn_blocking(move || rf.find_final_for_prompt(&prompt_id_owned))
            .await
            .map_err(|e| crate::TwolebotError::work(format!("spawn_blocking: {e}")))?;

        if let Ok(Some(response)) = found {
            return Ok(Some(response.content));
        }

        tokio::select! {
            _ = tokio::time::sleep(RESPONSE_POLL_INTERVAL) => {}
            _ = cancel_token.cancelled() => return Ok(None),
        }
    }
}

// Extension to PromptItem for agent-originated prompts
impl PromptItem {
    /// Create a new prompt item for agent loop execution with optional routing.
    /// Uses a stable job_id per task so each task gets its own topic.
    pub fn new_agent_routed(
        prompt: String,
        task_id: i64,
        task_title: &str,
        chat_id: Option<i64>,
        message_thread_id: Option<i64>,
    ) -> Self {
        use chrono::Utc;
        let job_id = format!("agent-task-{task_id}");
        let job_name = format!("\u{1F916} {task_title}");
        let source = match chat_id {
            Some(cid) => PromptSource::cron_routed(
                &job_id,
                uuid::Uuid::new_v4().to_string(),
                &job_name,
                cid,
                message_thread_id,
            ),
            None => PromptSource::cron(
                &job_id,
                uuid::Uuid::new_v4().to_string(),
                &job_name,
            ),
        };
        let topic_key = source.topic_key();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            user_id: 0, // System user
            prompt,
            media_path: None,
            topic_key,
            status: crate::storage::PromptStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::ResponseItem;
    use crate::work::db::WorkDb;
    use crate::work::models::{TaskPriority, TaskStatus};
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration, Instant};

    fn test_context() -> (
        Arc<WorkService>,
        Arc<PromptFeed>,
        Arc<ResponseFeed>,
        Arc<SettingsStore>,
        TempDir,
    ) {
        let dir = TempDir::new().unwrap();
        let db = WorkDb::open(dir.path()).unwrap();
        let service = Arc::new(WorkService::new(db));
        let prompt_feed = Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let response_feed = Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let settings_store = Arc::new(SettingsStore::new(dir.path().join("settings.sqlite3")).unwrap());
        (service, prompt_feed, response_feed, settings_store, dir)
    }

    async fn wait_until<F>(timeout: Duration, mut predicate: F) -> bool
    where
        F: FnMut() -> bool,
    {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if predicate() {
                return true;
            }
            sleep(Duration::from_millis(20)).await;
        }
        false
    }

    #[tokio::test]
    async fn agent_loop_enqueues_prompt_and_completes_with_response() {
        let (service, prompt_feed, response_feed, settings_store, _dir) = test_context();

        let project = service
            .create_project("Agent Test", "", &[], None)
            .unwrap();
        let task = service
            .create_task(
                project.id,
                "Wire autowork",
                "Implement pipeline",
                Some(&TaskStatus::Todo),
                Some(&TaskPriority::High),
                &[],
            )
            .unwrap();
        service
            .upsert_comment(
                None,
                Some(task.id),
                None,
                "Please include test evidence before moving to review.",
                None,
            )
            .unwrap();
        service.select_tasks(&[task.id]).unwrap();

        let agent_loop =
            AgentLoop::new(service.clone(), prompt_feed.clone(), response_feed.clone(), settings_store.clone());
        agent_loop.start().await.unwrap();

        let prompt_seen = wait_until(Duration::from_secs(2), || {
            prompt_feed
                .all_pending()
                .map(|items| !items.is_empty())
                .unwrap_or(false)
        })
        .await;
        assert!(prompt_seen, "expected agent loop to enqueue a prompt");

        let pending = prompt_feed.all_pending().unwrap();
        let prompt_id = pending.first().unwrap().id.clone();
        let prompt_text = pending
            .first()
            .map(|p| p.prompt.clone())
            .unwrap_or_default();
        assert!(
            prompt_text.contains("Comments listed above are authoritative context"),
            "expected prompt to require comment-aware workflow"
        );
        assert!(
            prompt_text.contains("MUST add a comment stating your reason"),
            "expected prompt to require explicit move rationale comments"
        );
        assert!(
            prompt_text.contains("Please include test evidence before moving to review."),
            "expected task comments to be embedded directly in agent prompt"
        );
        let response = ResponseItem::new(
            prompt_id,
            PromptSource::cron("agent-loop", "test-exec", "Agent Loop Task"),
            0,
            "Implemented and verified.",
            true,
            1,
        );
        response_feed.enqueue(response).unwrap();

        let completed = wait_until(Duration::from_secs(3), || {
            service
                .get_task(task.id)
                .map(|t| t.status == TaskStatus::ReadyForReview)
                .unwrap_or(false)
        })
        .await;
        assert!(completed, "expected task to move to ready_for_review");

        let summary_seen = wait_until(Duration::from_secs(2), || {
            service
                .list_task_comments(task.id, 50, 1)
                .map(|comments| {
                    comments
                        .items
                        .iter()
                        .any(|c| c.content.contains("Implemented and verified"))
                })
                .unwrap_or(false)
        })
        .await;
        assert!(summary_seen, "expected agent summary comment on task");
    }
}
