//! Cron "application/service" layer.
//!
//! This module centralizes cron business rules so REST handlers and MCP tools
//! cannot drift apart.

use crate::cron::{CronFeed, CronJob, CronJobStatus, CronSchedule};
use crate::error::{Result, TwolebotError};
use crate::types::cron::ScheduleJobRequest;
use chrono::{DateTime, Utc};
use std::str::FromStr;

/// Parse and validate a schedule request into a concrete schedule.
fn build_schedule(req: &ScheduleJobRequest) -> Result<CronSchedule> {
    let has_cron = req.cron.is_some();
    let has_crons = req.crons.as_ref().is_some_and(|v| !v.is_empty());
    let has_minutes = req.in_minutes.is_some();

    // Exactly one scheduling method must be provided
    let provided = [has_minutes, has_cron, has_crons]
        .iter()
        .filter(|&&b| b)
        .count();
    if provided == 0 {
        return Err(TwolebotError::cron(
            "Must provide one of 'in_minutes', 'cron', or 'crons'",
        ));
    }
    if provided > 1 {
        return Err(TwolebotError::cron(
            "Provide only one of 'in_minutes', 'cron', or 'crons'",
        ));
    }

    if let Some(minutes) = req.in_minutes {
        return Ok(CronSchedule::from_minutes(minutes));
    }

    if let Some(ref cron_expr) = req.cron {
        validate_cron_expression(cron_expr)?;
        return Ok(CronSchedule::cron(cron_expr));
    }

    if let Some(ref cron_exprs) = req.crons {
        for expr in cron_exprs {
            validate_cron_expression(expr)?;
        }
        return Ok(CronSchedule::multi_cron(cron_exprs.clone()));
    }

    unreachable!()
}

fn validate_cron_expression(expr: &str) -> Result<()> {
    if cron::Schedule::from_str(expr).is_err() {
        return Err(TwolebotError::cron(format!(
            "Invalid cron expression: {}",
            expr
        )));
    }
    Ok(())
}

fn calculate_next_run(
    schedule: &CronSchedule,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>> {
    match schedule {
        CronSchedule::OneShot { run_at } => Ok(Some(*run_at)),
        CronSchedule::Cron { .. } => earliest_next_run(&schedule.all_expressions(), now),
    }
}

/// Find the earliest next run across multiple cron expressions.
fn earliest_next_run(expressions: &[&str], after: DateTime<Utc>) -> Result<Option<DateTime<Utc>>> {
    let mut earliest: Option<DateTime<Utc>> = None;
    for expr in expressions {
        let parsed = cron::Schedule::from_str(expr).map_err(|e| {
            TwolebotError::cron(format!("Invalid cron expression '{}': {}", expr, e))
        })?;
        if let Some(next) = parsed.after(&after).next() {
            earliest = Some(match earliest {
                Some(current) if next < current => next,
                Some(current) => current,
                None => next,
            });
        }
    }
    Ok(earliest)
}

fn parse_list_filter(status: &str) -> Result<&str> {
    match status {
        "active" | "paused" | "all" => Ok(status),
        other => Err(TwolebotError::cron(format!(
            "Invalid status '{}', must be 'active', 'paused', or 'all'",
            other
        ))),
    }
}

/// Create and persist a new cron job.
pub fn schedule_job(feed: &CronFeed, req: ScheduleJobRequest) -> Result<CronJob> {
    let schedule = build_schedule(&req)?;
    let next_run = calculate_next_run(&schedule, Utc::now())?;

    let mut job = CronJob::new(req.name, req.prompt, schedule);
    if let Some(chat_id) = req.origin_chat_id {
        job = job.with_origin_chat_id(chat_id);
    }
    if let Some(next) = next_run {
        job = job.with_next_run(next);
    }

    feed.create_job(job)
}

/// List jobs by filter ("active" | "paused" | "all").
pub fn list_jobs(feed: &CronFeed, status: &str) -> Result<Vec<CronJob>> {
    let status = parse_list_filter(status)?;
    match status {
        "active" => feed.list_active_jobs(),
        "paused" => feed.list_paused_jobs(),
        "all" => feed.list_all_jobs(),
        _ => Err(TwolebotError::cron("Invalid status filter")),
    }
}

/// Cancel a job and remove any waiting executions for it.
pub fn cancel_job(feed: &CronFeed, job_id: &str) -> Result<(CronJob, usize)> {
    let cancelled_execs = feed.cancel_executions_for_job(job_id)?;
    let job = feed.cancel_job(job_id)?;
    Ok((job, cancelled_execs))
}

pub fn pause_job(feed: &CronFeed, job_id: &str) -> Result<CronJob> {
    feed.pause_job(job_id)
}

pub fn resume_job(feed: &CronFeed, job_id: &str) -> Result<CronJob> {
    feed.resume_job(job_id)
}

pub fn get_job(feed: &CronFeed, job_id: &str) -> Result<Option<CronJob>> {
    feed.get_job(job_id)
}

/// Snooze an active job by setting next_run = now + minutes, and cancelling any waiting executions.
pub fn snooze_job(feed: &CronFeed, job_id: &str, minutes: i64) -> Result<CronJob> {
    let Some(mut job) = feed.get_job(job_id)? else {
        return Err(TwolebotError::not_found("Job not found"));
    };

    if job.status != CronJobStatus::Active {
        return Err(TwolebotError::cron(format!(
            "Cannot snooze job with status '{:?}'",
            job.status
        )));
    }

    feed.cancel_executions_for_job(job_id)?;

    job.next_run = Some(Utc::now() + chrono::Duration::minutes(minutes));
    feed.update_job(&job)?;

    Ok(job)
}
