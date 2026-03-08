use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a cron job definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CronJobStatus {
    /// Job is actively scheduled
    Active,
    /// Job is paused (not scheduling new executions)
    Paused,
    /// One-shot job that has completed
    Completed,
    /// Job was cancelled
    Cancelled,
}

impl CronJobStatus {
    pub fn directory(&self) -> &'static str {
        match self {
            CronJobStatus::Active => "active",
            CronJobStatus::Paused => "paused",
            CronJobStatus::Completed => "completed",
            CronJobStatus::Cancelled => "cancelled",
        }
    }
}

/// Schedule type for a cron job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CronSchedule {
    /// One or more cron expressions (e.g., "0 12 * * *" for daily at noon).
    /// Uses `expressions` (Vec) for multi-trigger jobs; `expression` (String)
    /// is kept for backward compatibility with existing serialized data.
    #[serde(rename = "cron")]
    Cron {
        /// Legacy single expression — read by `all_expressions()` if `expressions` is empty.
        #[serde(default)]
        expression: String,
        /// Multiple cron expressions that share a single job/topic.
        #[serde(default)]
        expressions: Vec<String>,
    },
    /// One-time execution at a specific time
    #[serde(rename = "one_shot")]
    OneShot { run_at: DateTime<Utc> },
}

impl CronSchedule {
    /// Create a schedule with a single cron expression.
    pub fn cron(expression: impl Into<String>) -> Self {
        let expr = expression.into();
        CronSchedule::Cron {
            expression: String::new(),
            expressions: vec![expr],
        }
    }

    /// Create a schedule with multiple cron expressions (single job, single topic).
    pub fn multi_cron(expressions: Vec<String>) -> Self {
        CronSchedule::Cron {
            expression: String::new(),
            expressions,
        }
    }

    pub fn one_shot(run_at: DateTime<Utc>) -> Self {
        CronSchedule::OneShot { run_at }
    }

    pub fn from_minutes(minutes: i64) -> Self {
        let run_at = Utc::now() + chrono::Duration::minutes(minutes);
        CronSchedule::one_shot(run_at)
    }

    /// Check if this is a one-shot schedule
    pub fn is_one_shot(&self) -> bool {
        matches!(self, CronSchedule::OneShot { .. })
    }

    /// Get all cron expressions for this schedule.
    /// Handles backward compatibility: if `expressions` is empty, falls back to `expression`.
    pub fn all_expressions(&self) -> Vec<&str> {
        match self {
            CronSchedule::Cron {
                expression,
                expressions,
            } => {
                if !expressions.is_empty() {
                    expressions.iter().map(|s| s.as_str()).collect()
                } else if !expression.is_empty() {
                    vec![expression.as_str()]
                } else {
                    vec![]
                }
            }
            CronSchedule::OneShot { .. } => vec![],
        }
    }

    /// Get the first cron expression (backward compat convenience).
    pub fn cron_expression(&self) -> Option<&str> {
        self.all_expressions().into_iter().next()
    }

    /// Human-readable description of the schedule
    pub fn description(&self) -> String {
        match self {
            CronSchedule::Cron { .. } => {
                let exprs = self.all_expressions();
                if exprs.len() == 1 {
                    format!("cron: {}", exprs[0])
                } else {
                    format!("cron: [{}]", exprs.join(", "))
                }
            }
            CronSchedule::OneShot { run_at } => {
                format!("one-shot at {}", run_at.format("%Y-%m-%d %H:%M:%S UTC"))
            }
        }
    }
}

/// A cron job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub prompt: String,
    pub schedule: CronSchedule,
    /// Origin Telegram chat ID — when set, the gatekeeper routes
    /// executions to this chat instead of resolving from active chats.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_chat_id: Option<i64>,
    pub status: CronJobStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run: Option<DateTime<Utc>>,
}

impl CronJob {
    pub fn new(name: impl Into<String>, prompt: impl Into<String>, schedule: CronSchedule) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            prompt: prompt.into(),
            schedule,
            origin_chat_id: None,
            status: CronJobStatus::Active,
            created_at: Utc::now(),
            last_run: None,
            next_run: None,
        }
    }

    pub fn with_origin_chat_id(mut self, chat_id: i64) -> Self {
        self.origin_chat_id = Some(chat_id);
        self
    }

    pub fn with_next_run(mut self, next_run: DateTime<Utc>) -> Self {
        self.next_run = Some(next_run);
        self
    }

    pub fn filename(&self) -> String {
        format!("{}.json", self.id)
    }

    /// Get a display name for the job
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            // Legacy jobs without a name — use first 8 chars of ID
            &self.id[..self.id.len().min(8)]
        } else {
            &self.name
        }
    }
}

/// An execution instance of a cron job waiting to be promoted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExecution {
    pub id: String,
    pub job_id: String,
    pub prompt: String,
    pub scheduled_for: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl CronExecution {
    pub fn from_job(job: &CronJob, scheduled_for: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            job_id: job.id.clone(),
            prompt: job.prompt.clone(),
            scheduled_for,
            created_at: Utc::now(),
        }
    }

    /// Filename for storage (sorted by scheduled time)
    pub fn filename(&self) -> String {
        format!(
            "{}_{}_{}.json",
            self.scheduled_for.format("%Y%m%d%H%M%S"),
            self.job_id,
            self.id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_job_creation() {
        let job = CronJob::new("Hourly status check", "Check server status", CronSchedule::cron("0 * * * *"));

        assert_eq!(job.name, "Hourly status check");
        assert_eq!(job.origin_chat_id, None);
        assert_eq!(job.status, CronJobStatus::Active);
    }

    #[test]
    fn test_cron_schedule_from_minutes() {
        let schedule = CronSchedule::from_minutes(30);
        assert!(schedule.is_one_shot());
        assert!(schedule.cron_expression().is_none());
    }

    #[test]
    fn test_execution_from_job() {
        let job = CronJob::new("Test", "Test prompt", CronSchedule::cron("* * * * *"));
        let scheduled_for = Utc::now();
        let exec = CronExecution::from_job(&job, scheduled_for);

        assert_eq!(exec.job_id, job.id);
        assert_eq!(exec.prompt, job.prompt);
    }

    #[test]
    fn test_json_roundtrip() {
        let job = CronJob::new("Test job", "Hello", CronSchedule::from_minutes(5));

        let json = serde_json::to_string_pretty(&job).unwrap();
        let parsed: CronJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.id, parsed.id);
        assert_eq!(job.name, parsed.name);
        assert_eq!(job.prompt, parsed.prompt);
    }

    #[test]
    fn test_multi_cron_all_expressions() {
        let schedule = CronSchedule::multi_cron(vec![
            "0 0 6 * * *".to_string(),
            "0 30 6 * * *".to_string(),
            "0 0 7 * * *".to_string(),
        ]);
        let exprs = schedule.all_expressions();
        assert_eq!(exprs.len(), 3);
        assert_eq!(exprs[0], "0 0 6 * * *");
        assert_eq!(exprs[1], "0 30 6 * * *");
        assert_eq!(exprs[2], "0 0 7 * * *");
        assert!(!schedule.is_one_shot());
    }

    #[test]
    fn test_multi_cron_description() {
        let single = CronSchedule::cron("0 0 6 * * *");
        assert_eq!(single.description(), "cron: 0 0 6 * * *");

        let multi = CronSchedule::multi_cron(vec![
            "0 0 6 * * *".to_string(),
            "0 30 6 * * *".to_string(),
        ]);
        assert_eq!(multi.description(), "cron: [0 0 6 * * *, 0 30 6 * * *]");
    }

    #[test]
    fn test_multi_cron_cron_expression_returns_first() {
        let schedule = CronSchedule::multi_cron(vec![
            "0 0 6 * * *".to_string(),
            "0 30 6 * * *".to_string(),
        ]);
        assert_eq!(schedule.cron_expression(), Some("0 0 6 * * *"));
    }

    #[test]
    fn test_multi_cron_json_roundtrip() {
        let job = CronJob::new(
            "Study session",
            "Check in",
            CronSchedule::multi_cron(vec![
                "0 0 6 * * *".to_string(),
                "0 30 6 * * *".to_string(),
                "0 0 7 * * *".to_string(),
            ]),
        );

        let json = serde_json::to_string_pretty(&job).unwrap();
        let parsed: CronJob = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.schedule.all_expressions().len(), 3);
        assert_eq!(parsed.schedule.all_expressions()[0], "0 0 6 * * *");
        assert_eq!(parsed.schedule.all_expressions()[2], "0 0 7 * * *");
    }

    #[test]
    fn test_legacy_expression_field_backward_compat() {
        // Simulate old serialized format with just "expression" field
        let json = r#"{
            "id": "test-id",
            "name": "Legacy job",
            "prompt": "Do something",
            "schedule": {
                "type": "cron",
                "expression": "0 0 6 * * *"
            },
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z"
        }"#;

        let job: CronJob = serde_json::from_str(json).unwrap();
        let exprs = job.schedule.all_expressions();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0], "0 0 6 * * *");
    }

    #[test]
    fn test_new_expressions_field_takes_precedence() {
        // If both expression and expressions are present, expressions wins
        let json = r#"{
            "id": "test-id",
            "name": "Test",
            "prompt": "Do something",
            "schedule": {
                "type": "cron",
                "expression": "old one",
                "expressions": ["0 0 6 * * *", "0 30 6 * * *"]
            },
            "status": "active",
            "created_at": "2026-01-01T00:00:00Z"
        }"#;

        let job: CronJob = serde_json::from_str(json).unwrap();
        let exprs = job.schedule.all_expressions();
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0], "0 0 6 * * *");
    }
}
