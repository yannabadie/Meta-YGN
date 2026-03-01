use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn success_rate_returns_none_when_insufficient_data() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    let rate = store
        .success_rate_for_task_type("Bugfix", 20, 5)
        .await
        .unwrap();
    assert!(rate.is_none());
}

#[tokio::test]
async fn success_rate_returns_none_when_below_min_samples() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    // Insert 3 outcomes, but min_samples is 5
    for i in 0..3 {
        store
            .save_outcome(
                &format!("id-{i}"),
                &format!("sess-{i}"),
                "Bugfix",
                "Low",
                "StepByStep",
                true,
                1000,
                5000,
                0,
            )
            .await
            .unwrap();
    }
    let rate = store
        .success_rate_for_task_type("Bugfix", 20, 5)
        .await
        .unwrap();
    assert!(rate.is_none());
}

#[tokio::test]
async fn success_rate_computes_correctly() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    // Insert 10 outcomes: 7 success, 3 failure
    for i in 0..10 {
        store
            .save_outcome(
                &format!("id-{i}"),
                &format!("sess-{i}"),
                "Bugfix",
                "Low",
                "StepByStep",
                i < 7, // first 7 succeed
                1000,
                5000,
                if i >= 7 { 1 } else { 0 },
            )
            .await
            .unwrap();
    }
    let rate = store
        .success_rate_for_task_type("Bugfix", 20, 5)
        .await
        .unwrap();
    assert!(rate.is_some());
    let r = rate.unwrap();
    assert!((r - 0.7).abs() < 0.01, "expected ~0.7, got {r}");
}

#[tokio::test]
async fn success_rate_filters_by_task_type() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    // Insert 5 Bugfix (all succeed) and 5 Security (all fail)
    for i in 0..5 {
        store
            .save_outcome(
                &format!("bf-{i}"),
                &format!("sess-bf-{i}"),
                "Bugfix",
                "Low",
                "Rapid",
                true,
                500,
                2000,
                0,
            )
            .await
            .unwrap();
        store
            .save_outcome(
                &format!("sec-{i}"),
                &format!("sess-sec-{i}"),
                "Security",
                "High",
                "Cautious",
                false,
                1200,
                5000,
                2,
            )
            .await
            .unwrap();
    }

    let bugfix_rate = store
        .success_rate_for_task_type("Bugfix", 20, 5)
        .await
        .unwrap();
    assert!(bugfix_rate.is_some());
    assert!(
        (bugfix_rate.unwrap() - 1.0).abs() < 0.01,
        "expected 1.0 for Bugfix, got {}",
        bugfix_rate.unwrap()
    );

    let security_rate = store
        .success_rate_for_task_type("Security", 20, 5)
        .await
        .unwrap();
    assert!(security_rate.is_some());
    assert!(
        (security_rate.unwrap() - 0.0).abs() < 0.01,
        "expected 0.0 for Security, got {}",
        security_rate.unwrap()
    );
}
