use rusqlite::Connection;
use tempfile::tempdir;
use twolebot::storage::{MessageStore, StoredMessage};

#[test]
fn e2e_messages_persist_in_general_runtime_db() {
    let dir = tempdir().expect("tempdir");
    let runtime_db = dir.path().join("runtime.sqlite3");

    {
        let store = MessageStore::new(&runtime_db).expect("create message store");
        store
            .store(StoredMessage::inbound("m1", "chat-a", 10, "hello"))
            .expect("store m1");
        store
            .store(StoredMessage::outbound_with_user("m2", "chat-a", 10, "world"))
            .expect("store m2");
        store
            .store(StoredMessage::inbound("m3", "chat-b", 11, "other"))
            .expect("store m3");
    }

    // Re-open to verify persistence through SQLite, not process memory.
    let reopened = MessageStore::new(&runtime_db).expect("reopen message store");

    let chat_a = reopened.list("chat-a", 10).expect("list chat-a");
    assert_eq!(chat_a.len(), 2);

    let chats = reopened.list_chats().expect("list chats");
    assert_eq!(chats.len(), 2);

    let conn = Connection::open(&runtime_db).expect("open sqlite directly");
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .expect("count messages");
    assert_eq!(total, 3);
}
