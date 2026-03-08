use crate::cron::types::{CronExecution, CronJob, CronJobStatus};
use crate::error::{Result, TwolebotError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Manages cron job definitions and execution queue in SQLite.
pub struct CronFeed {
    conn: Mutex<Connection>,
}

/// Deserialize a JSON text column into T.
fn from_json_col<T: DeserializeOwned>(row: &rusqlite::Row<'_>, col: &str) -> rusqlite::Result<T> {
    let json: String = row.get(col)?;
    serde_json::from_str::<T>(&json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(json.len(), rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Deserialize a JSON text column by index into T.
fn from_json_idx<T: DeserializeOwned>(row: &rusqlite::Row<'_>, idx: usize) -> rusqlite::Result<T> {
    let json: String = row.get(idx)?;
    serde_json::from_str::<T>(&json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(json.len(), rusqlite::types::Type::Text, Box::new(e))
    })
}

impl CronFeed {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| TwolebotError::storage(format!("open cron db: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TwolebotError::storage(format!("set WAL mode: {e}")))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TwolebotError::storage(format!("set synchronous: {e}")))?;

        let feed = Self {
            conn: Mutex::new(conn),
        };
        feed.init_schema()?;
        Ok(feed)
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| TwolebotError::storage("cron db mutex poisoned"))
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.lock_conn()?;

        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS cron_jobs (
                id          TEXT PRIMARY KEY,
                status      TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                next_run    TEXT,
                job_json    TEXT NOT NULL,
                updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_cron_jobs_status_created ON cron_jobs(status, created_at);
            CREATE INDEX IF NOT EXISTS idx_cron_jobs_next_run ON cron_jobs(next_run);

            CREATE TABLE IF NOT EXISTS cron_waiting (
                id            TEXT PRIMARY KEY,
                job_id        TEXT NOT NULL,
                scheduled_for TEXT NOT NULL,
                created_at    TEXT NOT NULL,
                exec_json     TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_cron_waiting_schedule ON cron_waiting(scheduled_for, created_at);
            CREATE INDEX IF NOT EXISTS idx_cron_waiting_job ON cron_waiting(job_id);
            COMMIT;",
        )
        .map_err(|e| TwolebotError::storage(format!("init cron schema: {e}")))?;

        Ok(())
    }

    fn upsert_job(conn: &Connection, job: &CronJob) -> Result<()> {
        let json = serde_json::to_string(job)?;
        conn.execute(
            "INSERT INTO cron_jobs (id, status, created_at, next_run, job_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                created_at = excluded.created_at,
                next_run = excluded.next_run,
                job_json = excluded.job_json,
                updated_at = excluded.updated_at",
            params![
                job.id,
                job.status.directory(),
                job.created_at.to_rfc3339(),
                job.next_run.map(|d| d.to_rfc3339()),
                json,
            ],
        )
        .map_err(|e| TwolebotError::storage(format!("upsert cron job: {e}")))?;
        Ok(())
    }

    fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<CronJob> {
        from_json_col(row, "job_json")
    }

    fn row_to_exec(row: &rusqlite::Row<'_>) -> rusqlite::Result<CronExecution> {
        from_json_col(row, "exec_json")
    }

    // ==================== Job Management ====================

    pub fn create_job(&self, job: CronJob) -> Result<CronJob> {
        let conn = self.lock_conn()?;
        Self::upsert_job(&conn, &job)?;
        Ok(job)
    }

    pub fn get_job(&self, id: &str) -> Result<Option<CronJob>> {
        let conn = self.lock_conn()?;

        conn.query_row(
            "SELECT job_json FROM cron_jobs WHERE id = ?1",
            params![id],
            |row| from_json_idx(row, 0),
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("get cron job: {e}")))
    }

    pub fn update_job(&self, job: &CronJob) -> Result<()> {
        let conn = self.lock_conn()?;
        Self::upsert_job(&conn, job)
    }

    pub fn change_job_status(&self, id: &str, new_status: CronJobStatus) -> Result<CronJob> {
        let Some(mut job) = self.get_job(id)? else {
            return Err(TwolebotError::not_found(format!("Job {} not found", id)));
        };

        if job.status == new_status {
            return Ok(job);
        }

        job.status = new_status;
        self.update_job(&job)?;
        Ok(job)
    }

    pub fn list_active_jobs(&self) -> Result<Vec<CronJob>> {
        self.list_jobs_in_status("active")
    }

    pub fn list_paused_jobs(&self) -> Result<Vec<CronJob>> {
        self.list_jobs_in_status("paused")
    }

    pub fn list_all_jobs(&self) -> Result<Vec<CronJob>> {
        let conn = self.lock_conn()?;

        let mut stmt = conn
            .prepare("SELECT job_json FROM cron_jobs ORDER BY created_at ASC")
            .map_err(|e| TwolebotError::storage(format!("prepare list all jobs: {e}")))?;
        let rows = stmt
            .query_map([], Self::row_to_job)
            .map_err(|e| TwolebotError::storage(format!("query list all jobs: {e}")))?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row.map_err(|e| TwolebotError::storage(format!("row list all jobs: {e}")))?);
        }
        Ok(jobs)
    }

    fn list_jobs_in_status(&self, status: &str) -> Result<Vec<CronJob>> {
        let conn = self.lock_conn()?;

        let mut stmt = conn
            .prepare("SELECT job_json FROM cron_jobs WHERE status = ?1 ORDER BY created_at ASC")
            .map_err(|e| TwolebotError::storage(format!("prepare list jobs by status: {e}")))?;

        let rows = stmt
            .query_map(params![status], Self::row_to_job)
            .map_err(|e| TwolebotError::storage(format!("query list jobs by status: {e}")))?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(
                row.map_err(|e| TwolebotError::storage(format!("row list jobs by status: {e}")))?,
            );
        }
        Ok(jobs)
    }

    pub fn cancel_job(&self, id: &str) -> Result<CronJob> {
        self.change_job_status(id, CronJobStatus::Cancelled)
    }

    pub fn pause_job(&self, id: &str) -> Result<CronJob> {
        self.change_job_status(id, CronJobStatus::Paused)
    }

    pub fn resume_job(&self, id: &str) -> Result<CronJob> {
        self.change_job_status(id, CronJobStatus::Active)
    }

    pub fn complete_job(&self, id: &str) -> Result<CronJob> {
        self.change_job_status(id, CronJobStatus::Completed)
    }

    // ==================== Execution Queue Management ====================

    pub fn enqueue_execution(&self, exec: CronExecution) -> Result<CronExecution> {
        let conn = self.lock_conn()?;

        let json = serde_json::to_string(&exec)?;
        conn.execute(
            "INSERT OR REPLACE INTO cron_waiting
                (id, job_id, scheduled_for, created_at, exec_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                exec.id,
                exec.job_id,
                exec.scheduled_for.to_rfc3339(),
                exec.created_at.to_rfc3339(),
                json,
            ],
        )
        .map_err(|e| TwolebotError::storage(format!("enqueue execution: {e}")))?;

        Ok(exec)
    }

    pub fn next_waiting(&self) -> Result<Option<CronExecution>> {
        let conn = self.lock_conn()?;

        conn.query_row(
            "SELECT exec_json FROM cron_waiting ORDER BY scheduled_for ASC, created_at ASC LIMIT 1",
            [],
            |row| from_json_idx(row, 0),
        )
        .optional()
        .map_err(|e| TwolebotError::storage(format!("next waiting execution: {e}")))
    }

    pub fn list_waiting(&self) -> Result<Vec<CronExecution>> {
        let conn = self.lock_conn()?;

        let mut stmt = conn
            .prepare("SELECT exec_json FROM cron_waiting ORDER BY scheduled_for ASC, created_at ASC")
            .map_err(|e| TwolebotError::storage(format!("prepare list waiting: {e}")))?;
        let rows = stmt
            .query_map([], Self::row_to_exec)
            .map_err(|e| TwolebotError::storage(format!("query list waiting: {e}")))?;

        let mut execs = Vec::new();
        for row in rows {
            execs.push(row.map_err(|e| TwolebotError::storage(format!("row list waiting: {e}")))?);
        }

        Ok(execs)
    }

    pub fn remove_execution(&self, exec: &CronExecution) -> Result<()> {
        let conn = self.lock_conn()?;

        conn.execute("DELETE FROM cron_waiting WHERE id = ?1", params![exec.id])
            .map_err(|e| TwolebotError::storage(format!("remove waiting execution: {e}")))?;
        Ok(())
    }

    pub fn cancel_executions_for_job(&self, job_id: &str) -> Result<usize> {
        let conn = self.lock_conn()?;

        let count = conn
            .execute("DELETE FROM cron_waiting WHERE job_id = ?1", params![job_id])
            .map_err(|e| TwolebotError::storage(format!("cancel executions for job: {e}")))?;

        Ok(count)
    }

    pub fn record_job_run(
        &self,
        id: &str,
        run_time: DateTime<Utc>,
        next_run: Option<DateTime<Utc>>,
    ) -> Result<CronJob> {
        let Some(mut job) = self.get_job(id)? else {
            return Err(TwolebotError::not_found(format!("Job {} not found", id)));
        };

        job.last_run = Some(run_time);
        job.next_run = next_run;

        self.update_job(&job)?;
        Ok(job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::types::CronSchedule;
    use tempfile::tempdir;

    #[test]
    fn test_cron_feed_basic() {
        let dir = tempdir().unwrap();
        let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        let job =
            CronJob::new("Test job", "Test prompt", CronSchedule::cron("0 * * * * *"));
        let created = feed.create_job(job.clone()).unwrap();
        assert_eq!(created.status, CronJobStatus::Active);

        let retrieved = feed.get_job(&created.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test job");

        let active = feed.list_active_jobs().unwrap();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_job_status_transitions() {
        let dir = tempdir().unwrap();
        let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        let job = CronJob::new("Test", "Test", CronSchedule::cron("0 * * * * *"));
        let created = feed.create_job(job).unwrap();

        let paused = feed.pause_job(&created.id).unwrap();
        assert_eq!(paused.status, CronJobStatus::Paused);

        assert_eq!(feed.list_active_jobs().unwrap().len(), 0);
        assert_eq!(feed.list_paused_jobs().unwrap().len(), 1);

        let resumed = feed.resume_job(&created.id).unwrap();
        assert_eq!(resumed.status, CronJobStatus::Active);

        let cancelled = feed.cancel_job(&created.id).unwrap();
        assert_eq!(cancelled.status, CronJobStatus::Cancelled);
    }

    #[test]
    fn test_execution_queue() {
        let dir = tempdir().unwrap();
        let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        let job = CronJob::new("Test", "Test", CronSchedule::cron("0 * * * * *"));
        let job = feed.create_job(job).unwrap();

        let exec1 = CronExecution::from_job(&job, Utc::now() + chrono::Duration::minutes(1));
        let exec2 = CronExecution::from_job(&job, Utc::now() + chrono::Duration::minutes(2));

        feed.enqueue_execution(exec1.clone()).unwrap();
        feed.enqueue_execution(exec2.clone()).unwrap();

        let next = feed.next_waiting().unwrap().unwrap();
        assert_eq!(next.id, exec1.id);

        let waiting = feed.list_waiting().unwrap();
        assert_eq!(waiting.len(), 2);

        feed.remove_execution(&exec1).unwrap();
        let waiting = feed.list_waiting().unwrap();
        assert_eq!(waiting.len(), 1);
    }

    #[test]
    fn test_cancel_executions_for_job() {
        let dir = tempdir().unwrap();
        let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

        let job1 = feed
            .create_job(CronJob::new("Job1", "Job1", CronSchedule::cron("0 * * * * *")))
            .unwrap();
        let job2 = feed
            .create_job(CronJob::new("Job2", "Job2", CronSchedule::cron("0 * * * * *")))
            .unwrap();

        feed.enqueue_execution(CronExecution::from_job(&job1, Utc::now()))
            .unwrap();
        feed.enqueue_execution(CronExecution::from_job(&job1, Utc::now()))
            .unwrap();
        feed.enqueue_execution(CronExecution::from_job(&job2, Utc::now()))
            .unwrap();

        let cancelled = feed.cancel_executions_for_job(&job1.id).unwrap();
        assert_eq!(cancelled, 2);

        let waiting = feed.list_waiting().unwrap();
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0].job_id, job2.id);
    }

    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_job() -> impl Strategy<Value = CronJob> {
            (
                ".{1,80}",
                "[A-Za-z0-9 _-]{1,32}",
            )
                .prop_map(|(prompt, name)| {
                    CronJob::new(name, prompt, CronSchedule::cron("0 * * * * *"))
                })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(40))]

            #[test]
            fn prop_job_roundtrip(job in arb_job()) {
                let dir = tempfile::tempdir().unwrap();
                let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                let created = feed.create_job(job.clone()).unwrap();
                let loaded = feed.get_job(&created.id).unwrap().unwrap();

                prop_assert_eq!(loaded.id, created.id);
                prop_assert_eq!(loaded.prompt, created.prompt);
                prop_assert_eq!(loaded.name, created.name);
                prop_assert_eq!(loaded.origin_chat_id, created.origin_chat_id);
            }

            #[test]
            fn prop_execution_roundtrip(offset_secs in 0i64..3600i64) {
                let dir = tempfile::tempdir().unwrap();
                let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

                let job = feed
                    .create_job(CronJob::new("test", "test", CronSchedule::cron("0 * * * * *")))
                    .unwrap();
                let exec = CronExecution::from_job(&job, Utc::now() + chrono::Duration::seconds(offset_secs));

                feed.enqueue_execution(exec.clone()).unwrap();
                let next = feed.next_waiting().unwrap().unwrap();

                prop_assert_eq!(next.id, exec.id);
                prop_assert_eq!(next.job_id, exec.job_id);
                prop_assert_eq!(next.prompt, exec.prompt);
            }
        }
    }
}
