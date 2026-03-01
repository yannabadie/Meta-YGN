use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn record_and_retrieve_replay_events() {
    let store = MemoryStore::open_in_memory()
        .await
        .expect("open in-memory store");

    store
        .record_replay_event(
            "sess-replay-1",
            "PreToolUse",
            r#"{"tool":"read_file","path":"src/main.rs"}"#,
            r#"{"decision":"allow"}"#,
            12,
        )
        .await
        .expect("record event 1");

    store
        .record_replay_event(
            "sess-replay-1",
            "PostToolUse",
            r#"{"tool":"read_file","result":"ok"}"#,
            r#"{"logged":true}"#,
            5,
        )
        .await
        .expect("record event 2");

    let events = store
        .replay_events("sess-replay-1")
        .await
        .expect("replay_events");
    assert_eq!(events.len(), 2);

    // Events should be ordered by id ascending
    assert!(events[0].0 < events[1].0);

    // Verify first event fields
    assert_eq!(events[0].1, "PreToolUse"); // hook_event
    assert_eq!(events[0].2, r#"{"tool":"read_file","path":"src/main.rs"}"#); // request_json
    assert_eq!(events[0].3, r#"{"decision":"allow"}"#); // response_json
    assert_eq!(events[0].4, 12); // latency_ms

    // Verify second event fields
    assert_eq!(events[1].1, "PostToolUse");
    assert_eq!(events[1].4, 5);
}

#[tokio::test]
async fn replay_sessions_lists_with_counts() {
    let store = MemoryStore::open_in_memory()
        .await
        .expect("open in-memory store");

    // Record events across two sessions
    store
        .record_replay_event("sess-a", "PreToolUse", "{}", "{}", 10)
        .await
        .expect("record a-1");
    store
        .record_replay_event("sess-a", "PostToolUse", "{}", "{}", 8)
        .await
        .expect("record a-2");
    store
        .record_replay_event("sess-a", "Stop", "{}", "{}", 2)
        .await
        .expect("record a-3");

    store
        .record_replay_event("sess-b", "PreToolUse", "{}", "{}", 15)
        .await
        .expect("record b-1");

    let sessions = store.replay_sessions().await.expect("replay_sessions");
    assert_eq!(sessions.len(), 2);

    // Find each session by id (ordering may vary when timestamps are identical)
    let sess_a = sessions
        .iter()
        .find(|s| s.0 == "sess-a")
        .expect("sess-a present");
    let sess_b = sessions
        .iter()
        .find(|s| s.0 == "sess-b")
        .expect("sess-b present");

    assert_eq!(sess_a.1, 3); // event_count for sess-a
    assert_eq!(sess_b.1, 1); // event_count for sess-b

    // first_event and last_event timestamps should be non-empty
    assert!(!sess_a.2.is_empty());
    assert!(!sess_a.3.is_empty());
    assert!(!sess_b.2.is_empty());
    assert!(!sess_b.3.is_empty());
}

#[tokio::test]
async fn empty_session_returns_empty_vec() {
    let store = MemoryStore::open_in_memory()
        .await
        .expect("open in-memory store");

    let events = store
        .replay_events("nonexistent-session")
        .await
        .expect("replay_events for nonexistent session");
    assert!(events.is_empty());

    let sessions = store
        .replay_sessions()
        .await
        .expect("replay_sessions empty");
    assert!(sessions.is_empty());
}
