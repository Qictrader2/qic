//! End-to-end flow tests for twolebot
//!
//! These tests verify the full prompt lifecycle:
//! - Prompt enqueuing and state transitions
//! - Response creation and dispatch
//! - Cron job execution flow

use std::sync::Arc;
use tempfile::tempdir;

use twolebot::{
    cron::{CronFeed, CronJob, CronSchedule},
    storage::{PromptFeed, PromptItem, PromptSource, ResponseFeed, ResponseItem},
};

// ============ Prompt Lifecycle Tests ============

#[test]
fn test_prompt_lifecycle_pending_to_running_to_completed() {
    let dir = tempdir().unwrap();
    let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    // Create and enqueue a prompt
    let source = PromptSource::Telegram {
        update_id: 1,
        message_id: 100,
        chat_id: 12345,
        message_thread_id: None,
    };
    let prompt = PromptItem::new(source, 789, "Hello, Claude!");
    let prompt_id = prompt.id.clone();

    feed.enqueue(prompt).unwrap();

    // Should be in pending
    let pending = feed.next_pending().unwrap();
    assert!(pending.is_some());
    assert_eq!(pending.unwrap().id, prompt_id);

    // Mark as running
    let running = feed.mark_running(&prompt_id).unwrap();
    assert_eq!(running.id, prompt_id);

    // Should no longer be pending
    let pending_after = feed.next_pending().unwrap();
    assert!(pending_after.is_none());

    // Get running should return it
    let get_running = feed.get_running().unwrap();
    assert!(get_running.is_some());
    assert_eq!(get_running.unwrap().id, prompt_id);

    // Mark as completed
    feed.mark_completed(&prompt_id).unwrap();

    // Should no longer be running
    let running_after = feed.get_running().unwrap();
    assert!(running_after.is_none());
}

#[test]
fn test_prompt_lifecycle_with_failure() {
    let dir = tempdir().unwrap();
    let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    let source = PromptSource::Telegram {
        update_id: 2,
        message_id: 200,
        chat_id: 12345,
        message_thread_id: None,
    };
    let prompt = PromptItem::new(source, 789, "This will fail");
    let prompt_id = prompt.id.clone();

    feed.enqueue(prompt).unwrap();
    feed.mark_running(&prompt_id).unwrap();

    // Mark as failed
    feed.mark_failed(&prompt_id, "Test error").unwrap();

    // Should no longer be running
    let running = feed.get_running().unwrap();
    assert!(running.is_none());
}

#[test]
fn test_prompt_ordering_fifo() {
    let dir = tempdir().unwrap();
    let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    // Enqueue multiple prompts
    for i in 0..5 {
        let source = PromptSource::Telegram {
            update_id: i,
            message_id: i as i64 * 100,
            chat_id: 12345,
            message_thread_id: None,
        };
        let prompt = PromptItem::new(source, 789, format!("Prompt {}", i));
        feed.enqueue(prompt).unwrap();

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should retrieve in FIFO order
    for i in 0..5 {
        let pending = feed.next_pending().unwrap().unwrap();
        assert!(
            pending.prompt.contains(&format!("Prompt {}", i)),
            "Expected Prompt {}, got: {}",
            i,
            pending.prompt
        );
        feed.mark_running(&pending.id).unwrap();
        feed.mark_completed(&pending.id).unwrap();
    }
}

// ============ Response Lifecycle Tests ============

#[test]
fn test_response_lifecycle_pending_to_sent() {
    let dir = tempdir().unwrap();
    let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    let source = PromptSource::Telegram {
        update_id: 1,
        message_id: 100,
        chat_id: 12345,
        message_thread_id: None,
    };
    let response = ResponseItem::new("prompt-1", source, 789, "Hello!", true, 0);
    let response_id = response.id.clone();

    feed.enqueue(response).unwrap();

    // Should be in pending
    let pending = feed.all_pending().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, response_id);

    // Mark as sent
    feed.mark_sent(&response_id, "prompt-1", 0).unwrap();

    // Should no longer be pending
    let pending_after = feed.all_pending().unwrap();
    assert!(pending_after.is_empty());

    // Should be in sent
    let sent = feed.recent_sent(10).unwrap();
    assert_eq!(sent.len(), 1);
}

#[test]
fn test_response_ordering_by_sequence() {
    let dir = tempdir().unwrap();
    let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    // Enqueue responses with different sequences (out of order)
    let source = PromptSource::Telegram {
        update_id: 1,
        message_id: 100,
        chat_id: 12345,
        message_thread_id: None,
    };

    for seq in [2, 0, 3, 1, 4] {
        let response = ResponseItem::new(
            "prompt-1",
            source.clone(),
            789,
            format!("Part {}", seq),
            seq == 4,
            seq,
        );
        feed.enqueue(response).unwrap();
    }

    // Should retrieve in sequence order
    let pending = feed.all_pending().unwrap();
    assert_eq!(pending.len(), 5);

    for (i, resp) in pending.iter().enumerate() {
        assert_eq!(
            resp.sequence, i as u32,
            "Expected sequence {}, got {}",
            i, resp.sequence
        );
    }
}

// ============ Cron Job Flow Tests ============

#[test]
fn test_cron_job_creation_and_execution_flow() {
    let dir = tempdir().unwrap();
    let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());

    // Create a one-shot job scheduled for the past (should fire immediately)
    let past = chrono::Utc::now() - chrono::Duration::minutes(1);
    let job = CronJob::new("Test Job", "Test cron prompt", CronSchedule::one_shot(past));

    let created = feed.create_job(job).unwrap();
    assert_eq!(created.name, "Test Job");

    // Manually create an execution (simulating what scheduler does)
    use twolebot::cron::CronExecution;
    let exec = CronExecution::from_job(&created, chrono::Utc::now());
    feed.enqueue_execution(exec.clone()).unwrap();

    // Should be in waiting queue
    let waiting = feed.list_waiting().unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].job_id, created.id);

    // Remove execution (simulating what gatekeeper does after promoting)
    feed.remove_execution(&waiting[0]).unwrap();

    // Should no longer be waiting
    let waiting_after = feed.list_waiting().unwrap();
    assert!(waiting_after.is_empty());
}

#[test]
fn test_cron_job_cancellation() {
    let dir = tempdir().unwrap();
    let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    let future = chrono::Utc::now() + chrono::Duration::hours(1);
    let job = CronJob::new("Cancel test", "To be cancelled", CronSchedule::one_shot(future));
    let created = feed.create_job(job).unwrap();

    // Create some executions
    use twolebot::cron::CronExecution;
    for _ in 0..3 {
        let exec = CronExecution::from_job(&created, chrono::Utc::now());
        feed.enqueue_execution(exec).unwrap();
    }

    // Cancel the job
    let cancelled_execs = feed.cancel_executions_for_job(&created.id).unwrap();
    assert_eq!(cancelled_execs, 3);

    feed.cancel_job(&created.id).unwrap();

    // Job should be cancelled
    let job_after = feed.get_job(&created.id).unwrap().unwrap();
    assert_eq!(job_after.status, twolebot::cron::CronJobStatus::Cancelled);

    // No waiting executions
    let waiting = feed.list_waiting().unwrap();
    assert!(waiting.is_empty());
}

// ============ Multi-Cron Tests ============

#[test]
fn test_multi_cron_job_creation_and_execution_flow() {
    let dir = tempdir().unwrap();
    let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());

    // Create a job with multiple trigger times (like Schaline study sessions)
    let job = CronJob::new(
        "Study sessions",
        "Check in on study progress",
        CronSchedule::multi_cron(vec![
            "0 0 6 * * *".to_string(),   // 06:00
            "0 30 6 * * *".to_string(),  // 06:30
            "0 0 7 * * *".to_string(),   // 07:00
        ]),
    );

    let created = feed.create_job(job).unwrap();
    assert_eq!(created.name, "Study sessions");
    assert_eq!(created.schedule.all_expressions().len(), 3);

    // Single job should be in the list
    let active = feed.list_active_jobs().unwrap();
    assert_eq!(active.len(), 1);

    // Simulate scheduler firing: creates one execution per trigger
    use twolebot::cron::CronExecution;
    let exec = CronExecution::from_job(&created, chrono::Utc::now());
    feed.enqueue_execution(exec.clone()).unwrap();

    // Only one execution (one job fires once, then recalculates next_run)
    let waiting = feed.list_waiting().unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].job_id, created.id);

    // Remove execution (simulating gatekeeper promotion)
    feed.remove_execution(&waiting[0]).unwrap();
    assert!(feed.list_waiting().unwrap().is_empty());

    // Job is still active for the next trigger time
    let job_after = feed.get_job(&created.id).unwrap().unwrap();
    assert_eq!(job_after.status, twolebot::cron::CronJobStatus::Active);
}

#[test]
fn test_multi_cron_backward_compat_with_legacy_single_expression() {
    let dir = tempdir().unwrap();
    let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

    // Old-style single expression job
    let job = CronJob::new("Old job", "Legacy prompt", CronSchedule::cron("0 0 6 * * *"));
    let created = feed.create_job(job).unwrap();

    let retrieved = feed.get_job(&created.id).unwrap().unwrap();
    assert_eq!(retrieved.schedule.all_expressions().len(), 1);
    assert_eq!(retrieved.schedule.all_expressions()[0], "0 0 6 * * *");
    assert_eq!(retrieved.schedule.description(), "cron: 0 0 6 * * *");
}

// ============ Property Tests ============

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn prop_prompt_enqueue_preserves_content(
            user_id in 1i64..1000000,
            text in "[a-zA-Z0-9 ,.!?]{1,500}"
        ) {
            let dir = tempdir().unwrap();
            let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

            let source = PromptSource::Telegram {
                update_id: 1,
                message_id: 100,
                chat_id: 12345,
                message_thread_id: None,
            };
            let prompt = PromptItem::new(source, user_id, &text);

            feed.enqueue(prompt).unwrap();

            let retrieved = feed.next_pending().unwrap().unwrap();
            prop_assert_eq!(retrieved.user_id, user_id);
            prop_assert_eq!(retrieved.prompt, text);
        }

        #[test]
        fn prop_response_sequence_ordering_preserved(
            num_parts in 2usize..10
        ) {
            let dir = tempdir().unwrap();
            let feed = ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

            let source = PromptSource::Telegram {
                update_id: 1,
                message_id: 100,
                chat_id: 12345,
                message_thread_id: None,
            };

            // Enqueue in reverse order
            for seq in (0..num_parts).rev() {
                let response = ResponseItem::new(
                    "prompt-1",
                    source.clone(),
                    789,
                    format!("Part {}", seq),
                    seq == num_parts - 1,
                    seq as u32,
                );
                feed.enqueue(response).unwrap();
            }

            // Should retrieve in ascending sequence order
            let pending = feed.all_pending().unwrap();
            prop_assert_eq!(pending.len(), num_parts);

            for (i, resp) in pending.iter().enumerate() {
                prop_assert_eq!(
                    resp.sequence, i as u32,
                    "Sequence ordering violated"
                );
            }
        }

        #[test]
        fn prop_cron_job_roundtrip(
            prompt in "[a-zA-Z0-9 ]{1,200}",
            name in "[a-zA-Z0-9 ]{1,50}"
        ) {
            let dir = tempdir().unwrap();
            let feed = CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

            let job = CronJob::new(&name, &prompt, CronSchedule::from_minutes(10));

            let created = feed.create_job(job).unwrap();

            let retrieved = feed.get_job(&created.id).unwrap().unwrap();
            prop_assert_eq!(retrieved.prompt, prompt);
            prop_assert_eq!(retrieved.name, name);
        }

        #[test]
        fn prop_multiple_prompts_all_processed(
            count in 1usize..10
        ) {
            let dir = tempdir().unwrap();
            let feed = PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap();

            // Enqueue multiple prompts
            let mut ids = Vec::new();
            for i in 0..count {
                let source = PromptSource::Telegram {
                    update_id: i as i64,
                    message_id: i as i64 * 100,
                    chat_id: 12345,
                    message_thread_id: None,
                };
                let prompt = PromptItem::new(source, 789, format!("Prompt {}", i));
                ids.push(prompt.id.clone());
                feed.enqueue(prompt).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(5));
            }

            // Process all of them
            let mut processed = 0;
            while let Some(pending) = feed.next_pending().unwrap() {
                feed.mark_running(&pending.id).unwrap();
                feed.mark_completed(&pending.id).unwrap();
                processed += 1;
            }

            prop_assert_eq!(processed, count, "All prompts should be processed");
        }
    }
}
