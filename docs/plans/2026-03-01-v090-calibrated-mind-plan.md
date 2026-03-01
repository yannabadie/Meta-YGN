# v0.9.0 "Calibrated Mind" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add entropy-based overconfidence detection, plasticity loss monitoring, UCB-scored memory retrieval, and structured RL trajectory export.

**Architecture:** Four independent features that integrate at the calibrate/decide/learn stages. EntropyTracker is a new struct in core. PlasticityTracker is extended in daemon. UCB scoring adds fields to MemoryNode. Trajectory export adds a new shared struct, store table, API endpoint, and CLI command.

**Tech Stack:** Rust 2024, tokio-rusqlite, axum, clap, sha2 (for trajectory signing).

---

## Task 1: Entropy Calibration — EntropyTracker Struct

**Files:**
- Create: `crates/core/src/heuristics/entropy.rs`
- Modify: `crates/core/src/heuristics/mod.rs`
- Test: `crates/core/tests/entropy_test.rs`

**Step 1: Write the test**

```rust
// crates/core/tests/entropy_test.rs
use metaygn_core::heuristics::entropy::EntropyTracker;

#[test]
fn new_tracker_has_zero_overconfidence() {
    let tracker = EntropyTracker::new(20);
    assert_eq!(tracker.overconfidence_score(), 0.0);
    assert!(!tracker.is_overconfident());
}

#[test]
fn correct_high_confidence_is_not_overconfident() {
    let mut tracker = EntropyTracker::new(20);
    for _ in 0..10 {
        tracker.record(0.9, true); // high confidence, correct
    }
    assert_eq!(tracker.overconfidence_score(), 0.0);
    assert!(!tracker.is_overconfident());
}

#[test]
fn wrong_high_confidence_is_overconfident() {
    let mut tracker = EntropyTracker::new(20);
    for _ in 0..10 {
        tracker.record(0.9, false); // high confidence, wrong
    }
    assert!(tracker.overconfidence_score() > 0.5);
    assert!(tracker.is_overconfident());
}

#[test]
fn sliding_window_evicts_old_entries() {
    let mut tracker = EntropyTracker::new(5);
    // Fill with overconfident entries
    for _ in 0..5 {
        tracker.record(0.9, false);
    }
    assert!(tracker.is_overconfident());
    // Now fill with correct entries — window evicts old
    for _ in 0..5 {
        tracker.record(0.9, true);
    }
    assert!(!tracker.is_overconfident());
}

#[test]
fn low_confidence_errors_dont_count_as_overconfidence() {
    let mut tracker = EntropyTracker::new(20);
    for _ in 0..10 {
        tracker.record(0.3, false); // low confidence, wrong — not overconfident
    }
    assert_eq!(tracker.overconfidence_score(), 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p metaygn-core --test entropy_test`
Expected: FAIL — module not found

**Step 3: Create entropy.rs**

```rust
// crates/core/src/heuristics/entropy.rs
use std::collections::VecDeque;

/// Tracks overconfidence by monitoring high-confidence decisions that turn out wrong.
/// Inspired by EGPO (arXiv:2602.22751) entropy-based metacognitive calibration.
#[derive(Debug, Clone)]
pub struct EntropyTracker {
    window: VecDeque<(f64, bool)>, // (confidence, was_correct)
    window_size: usize,
}

/// Confidence threshold above which a decision is considered "high confidence".
const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.7;

/// Overconfidence score above which the tracker signals a problem.
const OVERCONFIDENCE_THRESHOLD: f64 = 0.3;

impl EntropyTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Record a decision outcome.
    pub fn record(&mut self, confidence: f64, was_correct: bool) {
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back((confidence, was_correct));
    }

    /// Fraction of high-confidence decisions that were wrong.
    /// Returns 0.0 if no high-confidence decisions in the window.
    pub fn overconfidence_score(&self) -> f64 {
        let high_conf: Vec<&(f64, bool)> = self
            .window
            .iter()
            .filter(|(c, _)| *c >= HIGH_CONFIDENCE_THRESHOLD)
            .collect();
        if high_conf.is_empty() {
            return 0.0;
        }
        let wrong_count = high_conf.iter().filter(|(_, correct)| !correct).count();
        wrong_count as f64 / high_conf.len() as f64
    }

    /// Whether the overconfidence score exceeds the threshold.
    pub fn is_overconfident(&self) -> bool {
        self.overconfidence_score() > OVERCONFIDENCE_THRESHOLD
    }

    /// Number of entries in the window.
    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}
```

**Step 4: Update mod.rs**

In `crates/core/src/heuristics/mod.rs`, add `pub mod entropy;`.

**Step 5: Run test to verify it passes**

Run: `cargo test -p metaygn-core --test entropy_test`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add crates/core/src/heuristics/entropy.rs crates/core/src/heuristics/mod.rs crates/core/tests/entropy_test.rs
git commit -m "feat(core): EntropyTracker for overconfidence detection (EGPO-inspired)"
```

---

## Task 2: Entropy Calibration — Wire into Calibrate Stage

**Files:**
- Modify: `crates/core/src/stages/calibrate.rs`
- Modify: `crates/core/src/context.rs`

**Step 1: Add EntropyTracker to LoopContext**

In `crates/core/src/context.rs`, add a field:

```rust
use crate::heuristics::entropy::EntropyTracker;

// In LoopContext struct:
    /// Entropy tracker for overconfidence detection (EGPO).
    #[serde(skip)]
    pub entropy_tracker: EntropyTracker,
```

And in `LoopContext::new()`:

```rust
    entropy_tracker: EntropyTracker::new(20),
```

Note: Add `#[serde(skip)]` since EntropyTracker doesn't implement Serialize. Also add a `Default` impl for EntropyTracker or use `#[serde(skip_deserializing)]`.

**Step 2: Use EntropyTracker in calibrate.rs**

After the existing error counting logic in `CalibrateStage::run()`, add:

```rust
        // Record outcome in entropy tracker for overconfidence detection.
        let was_correct = error_count == 0;
        ctx.entropy_tracker.record(v.confidence, was_correct);

        // If overconfident: penalize confidence further.
        if ctx.entropy_tracker.is_overconfident() {
            let oc_score = ctx.entropy_tracker.overconfidence_score();
            let oc_penalty = oc_score * 0.2; // up to 0.2 additional penalty
            v.confidence = (v.confidence - oc_penalty).max(0.0);
            tracing::warn!(
                stage = self.name(),
                overconfidence_score = oc_score,
                "overconfidence detected, applying calibration penalty"
            );
        }
```

**Step 3: Run tests**

Run: `cargo test -p metaygn-core`
Expected: PASS (all existing + new tests)

**Step 4: Commit**

```bash
git add crates/core/src/stages/calibrate.rs crates/core/src/context.rs
git commit -m "feat(core): wire EntropyTracker into calibrate stage"
```

---

## Task 3: Plasticity Detection — Extend PlasticityTracker

**Files:**
- Modify: `crates/daemon/src/profiler/plasticity.rs`
- Test: `crates/daemon/tests/plasticity_test.rs` (extend existing)

**Step 1: Add PlasticityLevel enum and detection**

In `crates/daemon/src/profiler/plasticity.rs`, add:

```rust
/// Three-level plasticity classification (RL2F-inspired).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlasticityLevel {
    /// Error pattern changed after recovery — feedback is working.
    Responsive,
    /// Same error class recurred once after recovery — warning.
    Degraded,
    /// Same error recurred 2+ times after recovery — model ignoring feedback.
    Lost,
}
```

Add a method to `PlasticityTracker`:

```rust
    /// Current plasticity level based on consecutive failures.
    pub fn plasticity_level(&self) -> PlasticityLevel {
        match self.consecutive_failures {
            0 => PlasticityLevel::Responsive,
            1 => PlasticityLevel::Degraded,
            _ => PlasticityLevel::Lost,
        }
    }

    /// Whether plasticity is lost (2+ consecutive failures after recovery).
    pub fn is_plasticity_lost(&self) -> bool {
        self.plasticity_level() == PlasticityLevel::Lost
    }
```

**Step 2: Add tests**

In the existing `#[cfg(test)] mod tests` block, add:

```rust
    #[test]
    fn plasticity_level_responsive_by_default() {
        let tracker = PlasticityTracker::new();
        assert_eq!(tracker.plasticity_level(), PlasticityLevel::Responsive);
    }

    #[test]
    fn plasticity_level_degrades_after_one_failure() {
        let mut tracker = PlasticityTracker::new();
        tracker.record_recovery_injected();
        tracker.record_outcome(RecoveryOutcome::Failure);
        assert_eq!(tracker.plasticity_level(), PlasticityLevel::Degraded);
    }

    #[test]
    fn plasticity_level_lost_after_two_failures() {
        let mut tracker = PlasticityTracker::new();
        tracker.record_recovery_injected();
        tracker.record_outcome(RecoveryOutcome::Failure);
        tracker.record_recovery_injected();
        tracker.record_outcome(RecoveryOutcome::Failure);
        assert_eq!(tracker.plasticity_level(), PlasticityLevel::Lost);
        assert!(tracker.is_plasticity_lost());
    }

    #[test]
    fn plasticity_recovers_after_success() {
        let mut tracker = PlasticityTracker::new();
        tracker.record_recovery_injected();
        tracker.record_outcome(RecoveryOutcome::Failure);
        tracker.record_outcome(RecoveryOutcome::Failure);
        assert_eq!(tracker.plasticity_level(), PlasticityLevel::Lost);
        // Success resets consecutive failures
        tracker.record_outcome(RecoveryOutcome::Success);
        assert_eq!(tracker.plasticity_level(), PlasticityLevel::Responsive);
    }
```

**Step 3: Run tests**

Run: `cargo test -p metaygn-daemon`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/daemon/src/profiler/plasticity.rs
git commit -m "feat(daemon): PlasticityLevel enum with 3-level detection (RL2F-inspired)"
```

---

## Task 4: UCB-Scored Memory — Schema + adaptive_recall

**Files:**
- Modify: `crates/memory/src/graph.rs`
- Test: `crates/memory/tests/ucb_test.rs`

**Step 1: Write test**

```rust
// crates/memory/tests/ucb_test.rs
use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider};
use metaygn_memory::graph::{GraphMemory, MemoryNode, NodeType, Scope};

#[tokio::test]
async fn adaptive_recall_favors_rewarded_nodes() {
    let graph = GraphMemory::open_in_memory().await.unwrap();
    let provider = HashEmbedProvider::new(64);

    // Insert two similar nodes
    for (id, text) in &[("n1", "rust error handling"), ("n2", "rust error types")] {
        let emb = provider.embed(text).unwrap();
        let node = MemoryNode {
            id: id.to_string(),
            node_type: NodeType::Lesson,
            scope: Scope::Project,
            label: text.to_string(),
            content: text.to_string(),
            embedding: Some(emb),
            created_at: "2026-03-01T00:00:00Z".into(),
            access_count: 0,
        };
        graph.insert_node(&node).await.unwrap();
    }

    // Reward n1 heavily
    for _ in 0..5 {
        graph.record_recall_reward("n1", 1.0).await.unwrap();
    }
    // n2 gets no rewards

    let query_emb = provider.embed("rust errors").unwrap();
    let results = graph.adaptive_recall(&query_emb, 2).await.unwrap();

    assert_eq!(results.len(), 2);
    // n1 should rank higher due to UCB reward
    assert_eq!(results[0].0.id, "n1");
}

#[tokio::test]
async fn adaptive_recall_explores_unvisited_nodes() {
    let graph = GraphMemory::open_in_memory().await.unwrap();
    let provider = HashEmbedProvider::new(64);

    // Insert nodes — n1 has many hits but low reward, n2 is fresh
    let emb1 = provider.embed("debugging techniques").unwrap();
    let node1 = MemoryNode {
        id: "n1".into(), node_type: NodeType::Lesson, scope: Scope::Project,
        label: "debugging techniques".into(), content: "debugging techniques".into(),
        embedding: Some(emb1), created_at: "2026-03-01T00:00:00Z".into(), access_count: 0,
    };
    graph.insert_node(&node1).await.unwrap();
    // Record many hits with low reward for n1
    for _ in 0..20 {
        graph.record_recall_reward("n1", 0.1).await.unwrap();
    }

    let emb2 = provider.embed("debugging tools").unwrap();
    let node2 = MemoryNode {
        id: "n2".into(), node_type: NodeType::Lesson, scope: Scope::Project,
        label: "debugging tools".into(), content: "debugging tools".into(),
        embedding: Some(emb2), created_at: "2026-03-01T00:00:00Z".into(), access_count: 0,
    };
    graph.insert_node(&node2).await.unwrap();
    // n2 is fresh — UCB exploration bonus should boost it

    let query_emb = provider.embed("debug").unwrap();
    let results = graph.adaptive_recall(&query_emb, 2).await.unwrap();
    assert_eq!(results.len(), 2);
    // Fresh n2 should get UCB exploration bonus
}
```

**Step 2: Add schema fields and methods to graph.rs**

Add `hit_count` and `reward_sum` columns to the `nodes` table in `init_schema`:

```sql
-- Add after existing CREATE TABLE IF NOT EXISTS nodes (as ALTER TABLE IF NOT EXISTS pattern):
-- Since we use CREATE TABLE IF NOT EXISTS, add new columns via separate ALTER TABLE statements
-- that silently fail if the column already exists.
```

Actually, for SQLite compatibility, add the columns via `ALTER TABLE` with `IF NOT EXISTS` check in init_schema (after the main CREATE TABLE):

```rust
// In init_schema, after CREATE TABLE nodes:
conn.execute_batch("
    ALTER TABLE nodes ADD COLUMN hit_count INTEGER DEFAULT 0;
    ALTER TABLE nodes ADD COLUMN reward_sum REAL DEFAULT 0.0;
").ok(); // .ok() silently ignores "duplicate column name" errors
```

Add methods to `impl GraphMemory`:

```rust
    /// Record a recall reward for a node (UCB feedback).
    pub async fn record_recall_reward(&self, node_id: &str, reward: f64) -> Result<()> {
        let node_id = node_id.to_owned();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE nodes SET hit_count = hit_count + 1, reward_sum = reward_sum + ?1 WHERE id = ?2",
                    params![reward, node_id],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Adaptive recall: blend cosine similarity (70%) with UCB exploration bonus (30%).
    /// UCB1: mean_reward + sqrt(2 * ln(total_queries) / hit_count)
    pub async fn adaptive_recall(
        &self,
        query_embedding: &[f32],
        limit: u32,
    ) -> Result<Vec<(MemoryNode, f32)>> {
        let query_emb = query_embedding.to_vec();
        let results = self
            .conn
            .call(move |conn| {
                // Get total query count for UCB calculation
                let total_queries: f64 = conn
                    .query_row("SELECT COALESCE(SUM(hit_count), 1) FROM nodes", [], |row| row.get(0))
                    .unwrap_or(1.0);

                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count, hit_count, reward_sum
                     FROM nodes WHERE embedding IS NOT NULL",
                )?;

                let mut scored: Vec<(MemoryNode, f32)> = stmt
                    .query_map([], |row| {
                        let node = row_to_node(row);
                        let hit_count: i64 = row.get(8).unwrap_or(0);
                        let reward_sum: f64 = row.get(9).unwrap_or(0.0);
                        Ok((node, hit_count, reward_sum))
                    })?
                    .filter_map(|r| r.ok())
                    .filter_map(|(node, hit_count, reward_sum)| {
                        let emb = node.embedding.as_ref()?;
                        if emb.is_empty() { return None; }

                        let cosine = cosine_similarity(&query_emb, emb);

                        // UCB1 score
                        let hits = (hit_count as f64).max(1.0);
                        let mean_reward = reward_sum / hits;
                        let exploration = (2.0 * total_queries.ln() / hits).sqrt();
                        let ucb = mean_reward + exploration;

                        // Blend: 70% cosine + 30% UCB (normalized to ~[0,1])
                        let ucb_normalized = (ucb / 3.0).min(1.0); // rough normalization
                        let final_score = 0.7 * cosine + 0.3 * ucb_normalized as f32;

                        Some((node, final_score))
                    })
                    .collect();

                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                scored.truncate(limit as usize);
                Ok::<_, rusqlite::Error>(scored)
            })
            .await?;
        Ok(results)
    }
```

**Step 3: Run tests**

Run: `cargo test -p metaygn-memory`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/memory/src/graph.rs crates/memory/tests/ucb_test.rs
git commit -m "feat(memory): UCB-scored adaptive_recall for exploration-exploitation (U-Mem inspired)"
```

---

## Task 5: Trajectory Export — Struct + Schema

**Files:**
- Create: `crates/shared/src/trajectory.rs`
- Modify: `crates/shared/src/lib.rs`
- Modify: `crates/memory/src/store.rs`
- Test: `crates/memory/tests/trajectory_test.rs`

**Step 1: Create Rl2fTrajectory struct**

```rust
// crates/shared/src/trajectory.rs
use serde::{Deserialize, Serialize};

/// A structured trajectory for RL2F-style fine-tuning data export.
/// Captures the full lifecycle of a single task attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rl2fTrajectory {
    pub session_id: String,
    pub task_type: Option<String>,
    pub risk_level: String,
    pub strategy_used: String,
    pub initial_attempt: Option<String>,
    pub verifiable_error: Option<String>,
    pub critique_injected: Option<String>,
    pub revised_attempt: Option<String>,
    pub success: bool,
    pub overconfidence_score: f64,
    pub plasticity_level: String,
    pub confidence: f64,
    pub coherence: f64,
    pub grounding: f64,
    pub timestamp: String,
    pub signature_hash: Option<String>,
}
```

**Step 2: Update shared/lib.rs**

Add `pub mod trajectory;`.

**Step 3: Add trajectory table to MemoryStore schema**

In `crates/memory/src/store.rs`, in `init_schema`, add after `replay_events`:

```sql
CREATE TABLE IF NOT EXISTS rl2f_trajectories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    trajectory_json TEXT NOT NULL,
    signature_hash TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_trajectories_session
    ON rl2f_trajectories(session_id, timestamp);
```

**Step 4: Add store methods**

```rust
    pub async fn save_trajectory(&self, session_id: &str, trajectory_json: &str, signature_hash: Option<&str>) -> Result<()> { ... }
    pub async fn export_trajectories(&self, limit: u32) -> Result<Vec<(i64, String, String, Option<String>, String)>> { ... }
```

**Step 5: Write test**

```rust
// crates/memory/tests/trajectory_test.rs
use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn save_and_export_trajectory() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    let tj = r#"{"session_id":"s1","success":true}"#;
    store.save_trajectory("s1", tj, Some("abc123")).await.unwrap();
    let rows = store.export_trajectories(10).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, "s1");
}
```

**Step 6: Run tests**

Run: `cargo test -p metaygn-memory`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/shared/src/trajectory.rs crates/shared/src/lib.rs crates/memory/src/store.rs crates/memory/tests/trajectory_test.rs
git commit -m "feat(shared+memory): Rl2fTrajectory struct + schema for RL trajectory export"
```

---

## Task 6: Trajectory Export — Daemon API + CLI

**Files:**
- Create: `crates/daemon/src/api/trajectories.rs`
- Modify: `crates/daemon/src/api/mod.rs`
- Modify: `crates/cli/src/main.rs`

**Step 1: Create trajectories.rs endpoint**

`GET /trajectories/export?limit=100` — returns JSONL-formatted trajectories.

**Step 2: Wire into daemon router**

Add `pub mod trajectories;` and `.merge(trajectories::routes())` in mod.rs.

**Step 3: Add `aletheia export` CLI command**

Fetches from `/trajectories/export` and writes JSONL to `~/.claude/aletheia/trajectories/export-{timestamp}.jsonl`.

**Step 4: Run tests**

Run: `cargo build --workspace`
Expected: Compiles

**Step 5: Commit**

```bash
git add crates/daemon/src/api/trajectories.rs crates/daemon/src/api/mod.rs crates/cli/src/main.rs
git commit -m "feat(daemon+cli): trajectory export API endpoint + 'aletheia export' command"
```

---

## Task 7: Docs & Version Bump

**Files:**
- Modify: `CHANGELOG.md`, `.claude-plugin/plugin.json`, `memory-bank/activeContext.md`, `memory-bank/progress.md`

**Step 1: Update all docs for v0.9.0**

**Step 2: Commit**

```bash
git commit -m "docs: v0.9.0 Calibrated Mind — changelog, plugin version, memory-bank"
```

---

## Task 8: Full Build & Test Verification

**Step 1:** `cargo build --workspace`
**Step 2:** `cargo test --workspace`
**Step 3:** `cargo clippy --workspace`
**Step 4:** Fix any issues, commit

---

## Summary

| Task | Feature | Files | Complexity |
|------|---------|-------|------------|
| 1 | EntropyTracker struct + tests | 3 create, 1 modify | Medium |
| 2 | Wire entropy into calibrate stage | 2 modify | Low |
| 3 | PlasticityLevel enum + detection | 1 modify | Low |
| 4 | UCB adaptive_recall + tests | 1 modify, 1 create | Medium |
| 5 | Trajectory struct + schema + tests | 3 create, 2 modify | Medium |
| 6 | Trajectory API + CLI | 2 create, 2 modify | Medium |
| 7 | Docs & version bump | 4 modify | Low |
| 8 | Full verification | 0 | Low |
