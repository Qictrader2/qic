use crate::cron::feed::CronFeed;
use crate::cron::types::{CronExecution, CronJob, CronSchedule};
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// Polls active cron jobs and creates executions when their schedule fires.
/// Executions go into the waiting queue to be picked up by the gatekeeper.
pub struct CronScheduler {
    cron_feed: Arc<CronFeed>,
}

impl CronScheduler {
    pub fn new(cron_feed: Arc<CronFeed>) -> Self {
        Self { cron_feed }
    }

    /// Start the scheduler loop
    pub async fn start(self: Arc<Self>, poll_interval_ms: u64) {
        let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_and_fire_jobs().await {
                tracing::error!("Error in cron scheduler: {}", e);
            }
        }
    }

    /// Check all active jobs and fire any that are due
    async fn check_and_fire_jobs(&self) -> crate::error::Result<()> {
        let now = Utc::now();
        let active_jobs = self.cron_feed.list_active_jobs()?;

        for job in active_jobs {
            if self.should_fire(&job, now)? {
                self.fire_job(&job, now)?;
            }
        }

        Ok(())
    }

    /// Determine if a job should fire based on its schedule
    fn should_fire(&self, job: &CronJob, now: DateTime<Utc>) -> crate::error::Result<bool> {
        match &job.schedule {
            CronSchedule::OneShot { run_at } => {
                // Fire if run_at has passed and job hasn't run yet
                Ok(now >= *run_at && job.last_run.is_none())
            }
            CronSchedule::Cron { .. } => {
                // Parse cron expression and check if we're past next_run
                let Some(next_run) = job.next_run else {
                    // No next_run set, calculate it
                    return Ok(true);
                };

                Ok(now >= next_run)
            }
        }
    }

    /// Fire a job: create an execution and update the job's timestamps
    fn fire_job(&self, job: &CronJob, now: DateTime<Utc>) -> crate::error::Result<()> {
        let job_name = job.display_name();
        tracing::info!("Firing cron job {} ({})", job.id, job_name);

        // Create execution instance
        let exec = CronExecution::from_job(job, now);
        self.cron_feed.enqueue_execution(exec)?;

        // Calculate next run time
        if job.schedule.is_one_shot() {
            self.cron_feed.complete_job(&job.id)?;
            return Ok(());
        }

        let next_run = self.calculate_earliest_next_run(&job.schedule.all_expressions(), now)?;

        // Update job timestamps
        self.cron_feed.record_job_run(&job.id, now, next_run)?;

        Ok(())
    }

    /// Calculate the earliest next run time across one or more cron expressions
    fn calculate_earliest_next_run(
        &self,
        expressions: &[&str],
        after: DateTime<Utc>,
    ) -> crate::error::Result<Option<DateTime<Utc>>> {
        let mut earliest: Option<DateTime<Utc>> = None;
        for expr in expressions {
            let schedule = Schedule::from_str(expr).map_err(|e| {
                crate::error::TwolebotError::cron(format!(
                    "Invalid cron expression '{}': {}",
                    expr, e
                ))
            })?;
            if let Some(next) = schedule.after(&after).next() {
                earliest = Some(match earliest {
                    Some(current) if next < current => next,
                    Some(current) => current,
                    None => next,
                });
            }
        }
        Ok(earliest)
    }

    /// Initialize next_run for jobs that don't have it set
    pub fn initialize_job_schedules(&self) -> crate::error::Result<usize> {
        let now = Utc::now();
        let active_jobs = self.cron_feed.list_active_jobs()?;
        let mut initialized = 0;

        for job in active_jobs {
            if job.next_run.is_none() {
                let next_run = match &job.schedule {
                    CronSchedule::OneShot { run_at } => Some(*run_at),
                    CronSchedule::Cron { .. } => {
                        self.calculate_earliest_next_run(&job.schedule.all_expressions(), now)?
                    }
                };

                if let Some(next) = next_run {
                    let mut updated_job = job.clone();
                    updated_job.next_run = Some(next);
                    self.cron_feed.update_job(&updated_job)?;
                    initialized += 1;
                }
            }
        }

        Ok(initialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::types::CronJobStatus;
    use tempfile::TempDir;

    fn create_test_feed() -> (Arc<CronFeed>, TempDir) {
        let dir = TempDir::new().unwrap();
        let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        (feed, dir)
    }

    #[test]
    fn test_should_fire_one_shot_due() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        let past = Utc::now() - chrono::Duration::minutes(5);
        let job = CronJob::new("Test", "Test", CronSchedule::one_shot(past));
        feed.create_job(job.clone()).unwrap();

        let should = scheduler.should_fire(&job, Utc::now()).unwrap();
        assert!(should);
    }

    #[test]
    fn test_should_fire_one_shot_not_due() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        let future = Utc::now() + chrono::Duration::minutes(5);
        let job = CronJob::new("Test", "Test", CronSchedule::one_shot(future));
        feed.create_job(job.clone()).unwrap();

        let should = scheduler.should_fire(&job, Utc::now()).unwrap();
        assert!(!should);
    }

    #[test]
    fn test_fire_one_shot_job() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        let past = Utc::now() - chrono::Duration::minutes(1);
        let job = CronJob::new("Test prompt", "Test prompt", CronSchedule::one_shot(past));
        let created = feed.create_job(job.clone()).unwrap();

        scheduler.fire_job(&created, Utc::now()).unwrap();

        // Job should be marked completed
        let updated = feed.get_job(&created.id).unwrap().unwrap();
        assert_eq!(updated.status, CronJobStatus::Completed);

        // Execution should be in waiting queue
        let waiting = feed.list_waiting().unwrap();
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0].job_id, created.id);
    }

    #[test]
    fn test_fire_recurring_job() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        // cron crate expects 6 fields: sec min hour day-of-month month day-of-week
        let job = CronJob::new("Test prompt", "Test prompt", CronSchedule::cron("0 * * * * *"));
        let created = feed.create_job(job).unwrap();

        scheduler.fire_job(&created, Utc::now()).unwrap();

        // Job should still be active
        let updated = feed.get_job(&created.id).unwrap().unwrap();
        assert_eq!(updated.status, CronJobStatus::Active);
        assert!(updated.last_run.is_some());
        assert!(updated.next_run.is_some());

        // Execution should be in waiting queue
        let waiting = feed.list_waiting().unwrap();
        assert_eq!(waiting.len(), 1);
    }

    #[test]
    fn test_calculate_next_run() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed);

        // Use standard 5-field cron with seconds prefix for cron crate
        // The cron crate expects "sec min hour day-of-month month day-of-week"
        let next = scheduler
            .calculate_earliest_next_run(&["0 * * * * *"], Utc::now())
            .unwrap();
        assert!(next.is_some());

        // Next run should be within the next minute
        let next_time = next.unwrap();
        let diff = next_time - Utc::now();
        assert!(diff < chrono::Duration::minutes(2));
    }

    #[test]
    fn test_invalid_cron_expression() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed);

        let result = scheduler.calculate_earliest_next_run(&["invalid"], Utc::now());
        assert!(result.is_err());
    }

    #[test]
    fn test_earliest_next_run_picks_soonest_across_expressions() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed);

        // "every minute at :00" vs "every minute at :30"
        // One of them should always be sooner than the other
        let exprs = &["0 0 * * * *", "0 30 * * * *"];
        let next = scheduler
            .calculate_earliest_next_run(exprs, Utc::now())
            .unwrap();
        assert!(next.is_some());

        // The earliest should be within 30 minutes (whichever is closer)
        let diff = next.unwrap() - Utc::now();
        assert!(
            diff < chrono::Duration::minutes(31),
            "Expected within 30 min, got {} seconds",
            diff.num_seconds()
        );
    }

    #[test]
    fn test_fire_multi_cron_job() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        let job = CronJob::new(
            "Multi-trigger",
            "Check in",
            CronSchedule::multi_cron(vec![
                "0 0 * * * *".to_string(),
                "0 30 * * * *".to_string(),
            ]),
        );
        let created = feed.create_job(job).unwrap();

        scheduler.fire_job(&created, Utc::now()).unwrap();

        // Job should still be active (not one-shot)
        let updated = feed.get_job(&created.id).unwrap().unwrap();
        assert_eq!(updated.status, CronJobStatus::Active);
        assert!(updated.last_run.is_some());
        assert!(updated.next_run.is_some());

        // Execution should be in waiting queue
        let waiting = feed.list_waiting().unwrap();
        assert_eq!(waiting.len(), 1);
    }

    #[test]
    fn test_initialize_multi_cron_job_schedules() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        let job = CronJob::new(
            "Multi-trigger",
            "Check in",
            CronSchedule::multi_cron(vec![
                "0 0 * * * *".to_string(),
                "0 30 * * * *".to_string(),
            ]),
        );
        feed.create_job(job).unwrap();

        let initialized = scheduler.initialize_job_schedules().unwrap();
        assert_eq!(initialized, 1);

        let jobs = feed.list_active_jobs().unwrap();
        assert!(jobs[0].next_run.is_some());
    }

    #[test]
    fn test_initialize_job_schedules() {
        let (feed, _tmpdir) = create_test_feed();
        let scheduler = CronScheduler::new(feed.clone());

        // Create jobs without next_run
        // cron crate expects 6 fields: sec min hour day-of-month month day-of-week
        let job1 = CronJob::new("Test 1", "Test 1", CronSchedule::cron("0 * * * * *"));
        let job2 = CronJob::new("Test 2", "Test 2", CronSchedule::from_minutes(10));
        feed.create_job(job1.clone()).unwrap();
        feed.create_job(job2.clone()).unwrap();

        let initialized = scheduler.initialize_job_schedules().unwrap();
        assert_eq!(initialized, 2);

        // Jobs should now have next_run
        let updated1 = feed.get_job(&job1.id).unwrap().unwrap();
        let updated2 = feed.get_job(&job2.id).unwrap().unwrap();
        assert!(updated1.next_run.is_some());
        assert!(updated2.next_run.is_some());
    }
}
