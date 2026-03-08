use proptest::prelude::*;
use rusqlite::{params, Connection};
use tempfile::tempdir;
use twolebot::storage::{PromptFeed, PromptItem, PromptSource, ResponseFeed, ResponseItem, SecretsStore};
use twolebot::work::WorkDb;

fn count_rows(conn: &Connection, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get(0)).unwrap_or(0)
}

#[test]
fn e2e_unified_general_db_contains_work_and_runtime_tables() {
    let dir = tempdir().expect("tempdir");
    let data_dir = dir.path();
    let general_db = data_dir.join("runtime.sqlite3");

    let _work_db = WorkDb::open(data_dir).expect("open work db");
    let prompt_feed = PromptFeed::new(&general_db)
        .expect("open prompt feed");
    let response_feed = ResponseFeed::new(&general_db)
        .expect("open response feed");
    let secrets = SecretsStore::new(&general_db).expect("open secrets");

    let prompt = PromptItem::new(PromptSource::telegram(1, 2, 12345, None), 99, "hello from unified db");
    let prompt_id = prompt.id.clone();
    prompt_feed.enqueue(prompt).expect("enqueue prompt");

    response_feed
        .enqueue(ResponseItem::new(
            prompt_id,
            PromptSource::telegram(1, 2, 12345, None),
            99,
            "reply",
            true,
            1,
        ))
        .expect("enqueue response");

    secrets
        .set_telegram_token("telegram-token".to_string())
        .expect("set token");

    let conn = Connection::open(&general_db).expect("open general db directly");
    conn.execute("INSERT INTO projects (name) VALUES (?1)", ["unified project"])
        .expect("insert project");

    assert_eq!(count_rows(&conn, "projects"), 1);
    assert_eq!(count_rows(&conn, "prompts"), 1);
    assert_eq!(count_rows(&conn, "responses"), 1);
    assert_eq!(count_rows(&conn, "runtime_secrets"), 1);
}

fn arb_case() -> impl Strategy<Value = (u8, u8, u8)> {
    (1u8..15, 1u8..15, 1u8..15)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_shared_general_db_keeps_counts_consistent((project_count, prompt_count, response_count) in arb_case()) {
        let dir = tempdir().expect("tempdir");
        let data_dir = dir.path();
        let general_db = data_dir.join("runtime.sqlite3");

        let work_db = WorkDb::open(data_dir).expect("open work db");
        let prompt_feed = PromptFeed::new(&general_db)
            .expect("open prompt feed");
        let response_feed = ResponseFeed::new(&general_db)
            .expect("open response feed");

        let work_conn = work_db.conn().expect("work conn");
        for idx in 0..project_count {
            work_conn
                .execute(
                    "INSERT INTO projects (name, description) VALUES (?1, ?2)",
                    params![format!("project-{idx}"), format!("desc-{idx}")],
                )
                .expect("insert project");
        }

        for idx in 0..prompt_count {
            let item = PromptItem::new(
                PromptSource::telegram(1000 + idx as i64, 2000 + idx as i64, 12345, None),
                idx as i64,
                format!("prompt-{idx}"),
            );
            prompt_feed.enqueue(item).expect("enqueue prompt");
        }

        for idx in 0..response_count {
            let item = ResponseItem::new(
                format!("prompt-{idx}"),
                PromptSource::telegram(3000 + idx as i64, 4000 + idx as i64, 12345, None),
                idx as i64,
                format!("response-{idx}"),
                true,
                idx as u32,
            );
            response_feed.enqueue(item).expect("enqueue response");
        }

        let conn = Connection::open(&general_db).expect("open general db");
        prop_assert_eq!(count_rows(&conn, "projects"), project_count as i64);
        prop_assert_eq!(count_rows(&conn, "prompts"), prompt_count as i64);
        prop_assert_eq!(count_rows(&conn, "responses"), response_count as i64);
    }
}
