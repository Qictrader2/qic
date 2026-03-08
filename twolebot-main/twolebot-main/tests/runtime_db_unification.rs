use proptest::prelude::*;
use rusqlite::{params, Connection};
use tempfile::tempdir;
use twolebot::storage::{
    PromptFeed, PromptItem, PromptSource, PromptStatus, ResponseFeed, ResponseItem, ResponseStatus,
};

fn count_rows(conn: &Connection, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get(0)).unwrap_or(0)
}

fn count_by_status(conn: &Connection, table: &str, status: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE status = ?1");
    conn.query_row(&sql, params![status], |row| row.get(0))
        .unwrap_or(0)
}

#[test]
fn e2e_prompt_and_response_feeds_share_same_general_db() {
    let dir = tempdir().expect("tempdir");
    let general_db = dir.path().join("runtime.sqlite3");

    let prompt_feed = PromptFeed::new(&general_db)
        .expect("open prompt feed with general db");
    let response_feed = ResponseFeed::new(&general_db)
        .expect("open response feed with general db");

    let prompt = PromptItem::new(PromptSource::telegram(10, 20, 12345, None), 42, "hello unified db");
    let prompt_id = prompt.id.clone();
    prompt_feed.enqueue(prompt).expect("enqueue prompt");

    let response = ResponseItem::new(
        prompt_id,
        PromptSource::cron("job", "exec", "job-name"),
        0,
        "response payload",
        true,
        1,
    );
    response_feed.enqueue(response).expect("enqueue response");

    assert_eq!(prompt_feed.pending_count(), 1);
    assert_eq!(response_feed.pending_count(), 1);

    let conn = Connection::open(&general_db).expect("open general db");
    assert_eq!(count_rows(&conn, "prompts"), 1);
    assert_eq!(count_rows(&conn, "responses"), 1);
    assert_eq!(count_by_status(&conn, "prompts", "pending"), 1);
    assert_eq!(count_by_status(&conn, "responses", "pending"), 1);
}

fn arb_prompt_statuses() -> impl Strategy<Value = Vec<PromptStatus>> {
    prop::collection::vec(
        prop_oneof![
            Just(PromptStatus::Pending),
            Just(PromptStatus::Running),
            Just(PromptStatus::Completed),
            Just(PromptStatus::Failed),
        ],
        1..40,
    )
}

fn arb_response_statuses() -> impl Strategy<Value = Vec<ResponseStatus>> {
    prop::collection::vec(
        prop_oneof![
            Just(ResponseStatus::Pending),
            Just(ResponseStatus::Sent),
            Just(ResponseStatus::Failed),
        ],
        1..40,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_shared_db_preserves_prompt_and_response_status_counts(
        prompt_statuses in arb_prompt_statuses(),
        response_statuses in arb_response_statuses(),
    ) {
        let dir = tempdir().expect("tempdir");
        let general_db = dir.path().join("runtime.sqlite3");

        let prompt_feed = PromptFeed::new(&general_db)
            .expect("open prompt feed");
        let response_feed = ResponseFeed::new(&general_db)
            .expect("open response feed");

        for (idx, status) in prompt_statuses.iter().enumerate() {
            let mut item = PromptItem::new(
                PromptSource::telegram(100 + idx as i64, 200 + idx as i64, 12345, None),
                idx as i64,
                format!("prompt-{idx}"),
            );
            item.status = *status;
            prompt_feed.enqueue(item).expect("enqueue prompt item");
        }

        for (idx, status) in response_statuses.iter().enumerate() {
            let mut item = ResponseItem::new(
                format!("prompt-{idx}"),
                PromptSource::cron("job", format!("exec-{idx}"), "job"),
                0,
                format!("resp-{idx}"),
                true,
                idx as u32,
            );
            item.status = *status;
            response_feed.enqueue(item).expect("enqueue response item");
        }

        let conn = Connection::open(&general_db).expect("open general db");

        let expected_prompt_pending = prompt_statuses.iter().filter(|s| **s == PromptStatus::Pending).count() as i64;
        let expected_prompt_running = prompt_statuses.iter().filter(|s| **s == PromptStatus::Running).count() as i64;
        let expected_prompt_completed = prompt_statuses.iter().filter(|s| **s == PromptStatus::Completed).count() as i64;
        let expected_prompt_failed = prompt_statuses.iter().filter(|s| **s == PromptStatus::Failed).count() as i64;

        prop_assert_eq!(count_by_status(&conn, "prompts", "pending"), expected_prompt_pending);
        prop_assert_eq!(count_by_status(&conn, "prompts", "running"), expected_prompt_running);
        prop_assert_eq!(count_by_status(&conn, "prompts", "completed"), expected_prompt_completed);
        prop_assert_eq!(count_by_status(&conn, "prompts", "failed"), expected_prompt_failed);

        let expected_response_pending = response_statuses.iter().filter(|s| **s == ResponseStatus::Pending).count() as i64;
        let expected_response_sent = response_statuses.iter().filter(|s| **s == ResponseStatus::Sent).count() as i64;
        let expected_response_failed = response_statuses.iter().filter(|s| **s == ResponseStatus::Failed).count() as i64;

        prop_assert_eq!(count_by_status(&conn, "responses", "pending"), expected_response_pending);
        prop_assert_eq!(count_by_status(&conn, "responses", "sent"), expected_response_sent);
        prop_assert_eq!(count_by_status(&conn, "responses", "failed"), expected_response_failed);
    }
}
