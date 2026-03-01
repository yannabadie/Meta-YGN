use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn save_and_export_trajectory() {
    let store = MemoryStore::open_in_memory().await.expect("open in-memory store");

    let trajectory_json = r#"{"session_id":"sess-t1","risk_level":"medium","strategy_used":"verify-first","success":true,"confidence":0.85}"#;
    let sig_hash = "abc123hash";

    store
        .save_trajectory("sess-t1", trajectory_json, Some(sig_hash))
        .await
        .expect("save trajectory");

    let rows = store
        .export_trajectories(10)
        .await
        .expect("export trajectories");

    assert_eq!(rows.len(), 1);

    let (id, session_id, json, signature_hash, timestamp) = &rows[0];
    assert!(*id > 0);
    assert_eq!(session_id, "sess-t1");
    assert_eq!(json, trajectory_json);
    assert_eq!(signature_hash.as_deref(), Some(sig_hash));
    assert!(!timestamp.is_empty());
}

#[tokio::test]
async fn export_empty_returns_empty_vec() {
    let store = MemoryStore::open_in_memory().await.expect("open in-memory store");

    let rows = store
        .export_trajectories(10)
        .await
        .expect("export trajectories from empty store");

    assert!(rows.is_empty());
}
