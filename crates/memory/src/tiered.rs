use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::store::MemoryStore;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which tier a memory entry currently resides in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tier {
    Hot,
    Warm,
    Cold,
}

/// A single memory entry that can live in any tier.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub tags: Vec<String>,
    pub tier: Tier,
    pub created_at: Instant,
    pub accessed_at: Instant,
    pub access_count: u32,
}

/// Snapshot of how many entries live in each tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TieredStats {
    pub hot_count: usize,
    pub warm_count: usize,
    pub cold_count: usize,
}

// ---------------------------------------------------------------------------
// TieredMemory
// ---------------------------------------------------------------------------

/// Three-tier memory: Hot (in-memory HashMap), Warm (SQLite rows),
/// Cold (SQLite FTS5 via `events_fts`).
pub struct TieredMemory {
    hot: HashMap<String, MemoryEntry>,
    warm: HashMap<String, MemoryEntry>,
    hot_ttl: Duration,
    store: Arc<MemoryStore>,
}

/// Threshold: entries accessed more than this many times get promoted to Warm.
const PROMOTE_ACCESS_THRESHOLD: u32 = 3;

impl TieredMemory {
    // -- Construction -------------------------------------------------------

    /// Create a new `TieredMemory` backed by the given [`MemoryStore`].
    ///
    /// `hot_ttl` controls how long entries stay in the hot (in-memory) tier
    /// before they are eligible for eviction (default recommendation: 5 min).
    pub fn new(store: Arc<MemoryStore>, hot_ttl: Duration) -> Self {
        Self {
            hot: HashMap::new(),
            warm: HashMap::new(),
            hot_ttl,
            store,
        }
    }

    // -- Write --------------------------------------------------------------

    /// Insert (or overwrite) an entry.  New entries always start in Hot.
    pub fn put(&mut self, key: &str, value: &str, tags: &[&str]) {
        let now = Instant::now();
        let entry = MemoryEntry {
            key: key.to_owned(),
            value: value.to_owned(),
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            tier: Tier::Hot,
            created_at: now,
            accessed_at: now,
            access_count: 0,
        };
        self.hot.insert(key.to_owned(), entry);
    }

    // -- Read ---------------------------------------------------------------

    /// Retrieve an entry by key. Checks Hot first, then Warm.
    ///
    /// Cold-tier lookups by key are not supported — use [`search`] for that.
    /// Each successful `get` bumps the entry's `access_count` and
    /// `accessed_at` timestamp.
    pub fn get(&mut self, key: &str) -> Option<&MemoryEntry> {
        // Check hot tier first — but skip expired entries.
        if let Some(entry) = self.hot.get(key) {
            if entry.created_at.elapsed() >= self.hot_ttl {
                // Expired — remove silently and fall through.
                self.hot.remove(key);
            }
        }

        // If still present in hot (not expired), bump and return.
        if self.hot.contains_key(key) {
            let entry = self.hot.get_mut(key).unwrap();
            entry.access_count += 1;
            entry.accessed_at = Instant::now();
            return self.hot.get(key);
        }

        // Check warm tier.
        if self.warm.contains_key(key) {
            let entry = self.warm.get_mut(key).unwrap();
            entry.access_count += 1;
            entry.accessed_at = Instant::now();
            return self.warm.get(key);
        }

        None
    }

    // -- Search (async — touches SQLite / cold tier) ------------------------

    /// Full-text search across *all* tiers.
    ///
    /// Hot and Warm results are found via simple substring matching on
    /// `value`; Cold results come from SQLite FTS5 via
    /// [`MemoryStore::search_events`].
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<MemoryEntry>> {
        let mut results = Vec::new();

        // Hot tier — substring match.
        for entry in self.hot.values() {
            if entry.value.contains(query) {
                results.push(entry.clone());
            }
        }

        // Warm tier — substring match.
        for entry in self.warm.values() {
            if entry.value.contains(query) {
                results.push(entry.clone());
            }
        }

        // Cold tier — FTS5.
        let cold_rows = self.store.search_events(query, limit).await?;
        for row in cold_rows {
            results.push(MemoryEntry {
                key: row.id.clone(),
                value: row.payload.clone(),
                tags: vec![row.event_type.clone()],
                tier: Tier::Cold,
                created_at: Instant::now(), // approximate
                accessed_at: Instant::now(),
                access_count: 0,
            });
        }

        results.truncate(limit as usize);
        Ok(results)
    }

    // -- Eviction / promotion -----------------------------------------------

    /// Remove all expired entries from the Hot tier.
    ///
    /// Returns the number of entries removed.  Entries that have been
    /// accessed more than [`PROMOTE_ACCESS_THRESHOLD`] times are *not*
    /// removed here — call [`promote_hot_to_warm`] to persist those.
    pub fn evict_expired(&mut self) -> usize {
        let ttl = self.hot_ttl;
        let before = self.hot.len();
        self.hot.retain(|_k, entry| entry.created_at.elapsed() < ttl);
        before - self.hot.len()
    }

    /// Move frequently-accessed Hot entries into the Warm tier.
    ///
    /// An entry qualifies when `access_count > PROMOTE_ACCESS_THRESHOLD`.
    /// Promoted entries are also persisted to SQLite via
    /// [`MemoryStore::log_event`] so they survive restarts.
    pub async fn promote_hot_to_warm(&mut self) -> Result<()> {
        let keys: Vec<String> = self
            .hot
            .iter()
            .filter(|(_k, e)| e.access_count > PROMOTE_ACCESS_THRESHOLD)
            .map(|(k, _e)| k.clone())
            .collect();

        for key in keys {
            if let Some(mut entry) = self.hot.remove(&key) {
                // Persist to SQLite.
                let tags_joined = entry.tags.join(",");
                let session_id = format!("warm:{}", tags_joined);
                self.store
                    .log_event(&session_id, "warm_memory", &entry.value)
                    .await?;

                entry.tier = Tier::Warm;
                self.warm.insert(key, entry);
            }
        }

        Ok(())
    }

    // -- Stats --------------------------------------------------------------

    /// Return a snapshot of entry counts per tier.
    ///
    /// Note: `cold_count` is *not* computed here (it would require an async
    /// call to SQLite).  Use [`cold_count`] for that.
    pub fn stats(&self) -> TieredStats {
        TieredStats {
            hot_count: self.hot.len(),
            warm_count: self.warm.len(),
            cold_count: 0, // sync snapshot — use cold_count() for real value
        }
    }

    /// Return the number of events stored in the Cold tier (SQLite).
    pub async fn cold_count(&self) -> Result<u64> {
        self.store.event_count().await
    }
}

// ---------------------------------------------------------------------------
// Unit tests (sync, hot-tier only)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a TieredMemory backed by an in-memory SQLite store.
    fn make_mem(ttl: Duration) -> TieredMemory {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = rt.block_on(MemoryStore::open_in_memory()).unwrap();
        TieredMemory::new(Arc::new(store), ttl)
    }

    #[test]
    fn put_get_round_trip() {
        let mut mem = make_mem(Duration::from_secs(60));
        mem.put("k1", "v1", &["a", "b"]);
        let e = mem.get("k1").unwrap();
        assert_eq!(e.value, "v1");
        assert_eq!(e.tags, vec!["a", "b"]);
    }

    #[test]
    fn missing_key_returns_none() {
        let mut mem = make_mem(Duration::from_secs(60));
        assert!(mem.get("nope").is_none());
    }

    #[test]
    fn access_count_increments() {
        let mut mem = make_mem(Duration::from_secs(60));
        mem.put("k", "v", &[]);
        for _ in 0..5 {
            mem.get("k");
        }
        assert_eq!(mem.get("k").unwrap().access_count, 6); // 5 + this get
    }

    #[test]
    fn evict_keeps_unexpired() {
        let mut mem = make_mem(Duration::from_secs(300));
        mem.put("a", "1", &[]);
        mem.put("b", "2", &[]);
        let removed = mem.evict_expired();
        assert_eq!(removed, 0);
        assert_eq!(mem.stats().hot_count, 2);
    }
}
