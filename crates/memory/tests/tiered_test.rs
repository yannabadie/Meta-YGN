use metaygn_memory::store::MemoryStore;
use metaygn_memory::tiered::{Tier, TieredMemory};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Hot-tier (sync) tests
// ---------------------------------------------------------------------------

#[test]
fn hot_tier_put_and_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = rt.block_on(MemoryStore::open_in_memory()).unwrap();
    let mut mem = TieredMemory::new(Arc::new(store), Duration::from_secs(300));

    mem.put("greeting", r#"{"msg":"hello"}"#, &["chat"]);

    let entry = mem.get("greeting").expect("should find hot entry");
    assert_eq!(entry.key, "greeting");
    assert_eq!(entry.value, r#"{"msg":"hello"}"#);
    assert_eq!(entry.tags, vec!["chat".to_string()]);
    assert!(matches!(entry.tier, Tier::Hot));
}

#[test]
fn hot_tier_expires() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = rt.block_on(MemoryStore::open_in_memory()).unwrap();
    // Use a very short TTL so the entry expires quickly.
    let mut mem = TieredMemory::new(Arc::new(store), Duration::from_millis(50));

    mem.put("temp", "data", &[]);

    // Still alive right away.
    assert!(mem.get("temp").is_some());

    // Wait for expiry.
    std::thread::sleep(Duration::from_millis(80));

    // After expiry the hot tier should no longer return it.
    assert!(
        mem.get("temp").is_none(),
        "entry should have expired from hot tier"
    );
}

#[test]
fn evict_removes_expired() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = rt.block_on(MemoryStore::open_in_memory()).unwrap();
    let mut mem = TieredMemory::new(Arc::new(store), Duration::from_millis(50));

    mem.put("a", "1", &[]);
    mem.put("b", "2", &[]);
    mem.put("c", "3", &[]);

    std::thread::sleep(Duration::from_millis(80));

    let evicted = mem.evict_expired();
    assert_eq!(evicted, 3, "all three entries should have been evicted");
    assert_eq!(mem.stats().hot_count, 0);
}

#[test]
fn stats_reflect_tier_counts() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = rt.block_on(MemoryStore::open_in_memory()).unwrap();
    let mut mem = TieredMemory::new(Arc::new(store), Duration::from_secs(300));

    assert_eq!(mem.stats().hot_count, 0);

    mem.put("x", "1", &["tag1"]);
    mem.put("y", "2", &["tag2"]);
    assert_eq!(mem.stats().hot_count, 2);

    mem.put("z", "3", &[]);
    assert_eq!(mem.stats().hot_count, 3);
}

// ---------------------------------------------------------------------------
// Warm-tier promotion test (async, needs tokio)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn frequent_access_promotes_to_warm() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    let store = Arc::new(store);
    let mut mem = TieredMemory::new(Arc::clone(&store), Duration::from_secs(300));

    mem.put("popular", r#"{"data":"important"}"#, &["ctx"]);

    // Access > 3 times to trigger promotion threshold.
    for _ in 0..4 {
        let _ = mem.get("popular");
    }

    // Now evict — the entry is not expired, but it has high access count.
    // Promote should persist it to warm via the store.
    mem.promote_hot_to_warm().await.unwrap();

    // The entry should now report as Warm tier.
    let entry = mem.get("popular").expect("should still be accessible");
    assert!(matches!(entry.tier, Tier::Warm));
}

// ---------------------------------------------------------------------------
// Cold-tier search test (async, needs tokio)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_finds_cold_entries() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    let store = Arc::new(store);

    // Insert directly into SQLite (cold tier).
    store
        .log_event("sess-1", "memory", r#"{"note":"rust ownership rules"}"#)
        .await
        .unwrap();
    store
        .log_event("sess-1", "memory", r#"{"note":"borrow checker basics"}"#)
        .await
        .unwrap();

    let mem = TieredMemory::new(Arc::clone(&store), Duration::from_secs(300));

    let results = mem.search("ownership", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].value.contains("ownership"));
    assert!(matches!(results[0].tier, Tier::Cold));
}
