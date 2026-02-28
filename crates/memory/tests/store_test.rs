use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn store_opens_and_creates_tables() {
    let store = MemoryStore::open_in_memory().await.expect("open in-memory store");
    let count = store.event_count().await.expect("event_count");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn store_logs_and_retrieves_events() {
    let store = MemoryStore::open_in_memory().await.expect("open in-memory store");

    let id1 = store
        .log_event("sess-1", "test_event", r#"{"key":"value1"}"#)
        .await
        .expect("log event 1");
    let id2 = store
        .log_event("sess-1", "test_event", r#"{"key":"value2"}"#)
        .await
        .expect("log event 2");

    // IDs should be valid UUIDs and distinct
    assert_ne!(id1, id2);
    assert_eq!(id1.len(), 36); // UUID v4 string length with hyphens

    // Count should be 2
    let count = store.event_count().await.expect("event_count");
    assert_eq!(count, 2);

    // recent_events should return them in timestamp ascending order
    let events = store
        .recent_events("sess-1", 10)
        .await
        .expect("recent_events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].id, id1);
    assert_eq!(events[1].id, id2);
    assert_eq!(events[0].session_id, "sess-1");
    assert_eq!(events[0].event_type, "test_event");
    assert_eq!(events[0].payload, r#"{"key":"value1"}"#);
    assert_eq!(events[1].payload, r#"{"key":"value2"}"#);
}

#[tokio::test]
async fn store_fts_search() {
    let store = MemoryStore::open_in_memory().await.expect("open in-memory store");

    store
        .log_event(
            "sess-1",
            "note",
            r#"{"text":"The quick brown fox jumps over the lazy dog"}"#,
        )
        .await
        .expect("log event 1");

    store
        .log_event(
            "sess-1",
            "note",
            r#"{"text":"A fast red car drives on the highway"}"#,
        )
        .await
        .expect("log event 2");

    // Search for "fox" should find only the first event
    let results = store.search_events("fox", 10).await.expect("search fox");
    assert_eq!(results.len(), 1);
    assert!(results[0].payload.contains("fox"));

    // Search for "highway" should find only the second event
    let results = store
        .search_events("highway", 10)
        .await
        .expect("search highway");
    assert_eq!(results.len(), 1);
    assert!(results[0].payload.contains("highway"));

    // Search for "the" should find both events
    let results = store.search_events("the", 10).await.expect("search the");
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn save_and_load_heuristic() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    store
        .save_heuristic("h1", 1, None, "{}", "{}", "{}", "2026-02-28T00:00:00Z")
        .await
        .unwrap();
    store
        .save_heuristic(
            "h2",
            2,
            Some("h1"),
            r#"{"score":0.8}"#,
            r#"{"destructive":0.9}"#,
            r#"{"rapid":0.7}"#,
            "2026-02-28T01:00:00Z",
        )
        .await
        .unwrap();
    let loaded = store.load_heuristics().await.unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].0, "h1"); // id
    assert_eq!(loaded[0].1, 1); // generation
    assert!(loaded[0].2.is_none()); // parent_id
    assert_eq!(loaded[1].0, "h2");
    assert_eq!(loaded[1].1, 2);
    assert_eq!(loaded[1].2.as_deref(), Some("h1"));
}

#[tokio::test]
async fn save_and_load_outcomes() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    store
        .save_outcome("o1", "s1", "Bugfix", "Low", "Rapid", true, 500, 2000, 0)
        .await
        .unwrap();
    store
        .save_outcome("o2", "s2", "Feature", "High", "Cautious", false, 1200, 5000, 3)
        .await
        .unwrap();
    let loaded = store.load_recent_outcomes(10).await.unwrap();
    assert_eq!(loaded.len(), 2);
    // Verify structure of returned JSON values
    assert_eq!(loaded[0]["id"].as_str().unwrap().len(), 2); // "o1" or "o2"
    // Check that boolean success field is properly round-tripped
    let success_values: Vec<bool> = loaded
        .iter()
        .map(|v| v["success"].as_bool().unwrap())
        .collect();
    assert!(success_values.contains(&true));
    assert!(success_values.contains(&false));
}
