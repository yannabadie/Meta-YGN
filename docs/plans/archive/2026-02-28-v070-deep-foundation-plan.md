# v0.7.0 "Deep Foundation" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate every stub, implement the real context pruning proxy, add local embeddings (fastembed), skill crystallizer, and wire cross-session learning — making every declared module genuinely functional.

**Architecture:** Fill in 4 stub modules (fts, events, act, compact). Add hyper-based reverse proxy in daemon for context pruning. Integrate fastembed 5.x for local embeddings. Observe tool event patterns for skill crystallization. Load persisted heuristics at daemon boot.

**Tech Stack:** Rust (fastembed 5.x, hyper 1.x for proxy, rusqlite), existing crates

**IMPORTANT:** fastembed 5.x pulls in heavy ONNX dependencies (~100MB). Gate behind `feature = "embeddings"` so the default build stays lean.

---

### Task 1: Implement events.rs — typed event system

**Files:**
- Modify: `crates/shared/src/events.rs` (replace TODO stub)
- Test: `crates/shared/tests/events_test.rs`

Implement typed events that replace ad-hoc string logging:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaEvent {
    SessionStarted { stack: Vec<String>, source: String },
    PromptClassified { risk: String, strategy: String, topology: String },
    ToolGated { tool: String, decision: String, guard: String, score: u8 },
    ToolCompleted { tool: String, success: bool, duration_ms: u64 },
    ToolFailed { tool: String, error: String },
    RecoveryInjected { level: u8, reason: String },
    RecoveryOutcome { success: bool, plasticity_score: f64 },
    CompletionVerified { verified: bool, issues: Vec<String> },
    TestIntegrityWarning { file: String, issues: Vec<String> },
    BudgetConsumed { tokens: u64, cost_usd: f64, utilization: f64 },
    SessionEnded { reason: String },
}

impl MetaEvent {
    pub fn event_type(&self) -> &'static str { /* match on variant name */ }
    pub fn to_json(&self) -> String { serde_json::to_string(self).unwrap_or_default() }
}
```

Tests: `event_type_matches_variant`, `serialization_roundtrip`, `all_variants_serialize`.

**Commit:** `git commit -m "feat(shared): typed MetaEvent system replacing ad-hoc string logging"`

---

### Task 2: Implement fts.rs — unified search facade

**Files:**
- Modify: `crates/memory/src/fts.rs` (replace TODO stub)
- Test: `crates/memory/tests/fts_test.rs`

Unified search across events table AND graph nodes:

```rust
pub struct SearchResult {
    pub source: SearchSource,  // Event or GraphNode
    pub id: String,
    pub content: String,
    pub score: f64,            // relevance score (BM25 rank)
}

pub enum SearchSource { Event, GraphNode }

pub struct UnifiedSearch {
    store: Arc<MemoryStore>,
    graph: Arc<GraphMemory>,
}

impl UnifiedSearch {
    pub fn new(store: Arc<MemoryStore>, graph: Arc<GraphMemory>) -> Self
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>>
}
```

The `search` method calls both `store.search_events()` and `graph.search_content()`, merges results by score, deduplicates, and returns top `limit`.

Tests: `search_finds_events`, `search_finds_graph_nodes`, `results_merged_and_sorted`.

**Commit:** `git commit -m "feat(memory): unified FTS search facade across events and graph nodes"`

---

### Task 3: Implement act.rs — record intended action

**Files:**
- Modify: `crates/core/src/stages/act.rs` (replace no-op)
- Modify: `crates/core/src/context.rs` — add `intended_action` field to LoopContext
- Test: `crates/core/tests/loop_test.rs` — add test

```rust
// In context.rs, add:
pub intended_action: Option<IntendedAction>,

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntendedAction {
    pub tool: String,
    pub target: String,     // file path or command
    pub purpose: String,    // what this action aims to achieve
}

// In act.rs:
impl Stage for ActStage {
    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // Extract intended action from tool_input
        if let Some(ref tool_input) = ctx.input.tool_input {
            let tool = ctx.input.tool_name.clone().unwrap_or_default();
            let target = tool_input.get("file_path")
                .or(tool_input.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ctx.intended_action = Some(IntendedAction {
                tool,
                target,
                purpose: format!("Execute {} stage action", ctx.strategy.map(|s| format!("{:?}", s)).unwrap_or_default()),
            });
        }
        StageResult::Continue
    }
}
```

Test: `act_stage_records_intended_action` — run with tool_input containing file_path, verify intended_action is Some.

**Commit:** `git commit -m "feat(core): act stage records intended action for post-verification comparison"`

---

### Task 4: Implement compact.rs — real memory compaction

**Files:**
- Modify: `crates/core/src/stages/compact.rs` (replace no-op)
- Test: `crates/core/tests/loop_test.rs` — add test

The compact stage should summarize the current session state:

```rust
impl Stage for CompactStage {
    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // Generate a compact summary from lessons and verification results
        let mut summary_parts = Vec::new();

        // Summarize task
        if let Some(ref tt) = ctx.task_type {
            summary_parts.push(format!("task={:?}", tt));
        }
        summary_parts.push(format!("risk={:?}", ctx.risk));
        summary_parts.push(format!("strategy={:?}", ctx.strategy));

        // Count verification results
        let errors = ctx.verification_results.iter()
            .filter(|r| r.contains("error") || r.contains("fail"))
            .count();
        let passes = ctx.verification_results.len() - errors;
        summary_parts.push(format!("verifications={}/{}", passes, ctx.verification_results.len()));

        // Compact lessons (keep only unique, max 5)
        let mut unique_lessons: Vec<String> = Vec::new();
        for lesson in &ctx.lessons {
            if !unique_lessons.iter().any(|l| l == lesson) && unique_lessons.len() < 5 {
                unique_lessons.push(lesson.clone());
            }
        }
        ctx.lessons = unique_lessons;

        // Store compact summary as a lesson
        ctx.lessons.push(format!("[compact] {}", summary_parts.join(", ")));

        StageResult::Continue
    }
}
```

Test: `compact_stage_summarizes_and_deduplicates` — add duplicate lessons, run compact, verify they're reduced.

**Commit:** `git commit -m "feat(core): compact stage generates real session summaries and deduplicates lessons"`

---

### Task 5: Context pruning reverse proxy

**Files:**
- Create: `crates/daemon/src/proxy/server.rs`
- Modify: `crates/daemon/src/proxy/mod.rs` — add `pub mod server;`
- Modify: `crates/daemon/Cargo.toml` — add `hyper = { version = "1", features = ["http1", "client", "server"] }` and `hyper-util = "0.1"`
- Modify: `crates/daemon/src/main.rs` — start proxy on --proxy flag
- Modify: `crates/cli/src/main.rs` — add --proxy flag to start command
- Test: `crates/daemon/tests/proxy_test.rs`

The proxy:
1. Listens on configurable port (default 11434)
2. Receives HTTP requests meant for api.anthropic.com
3. Deserializes the JSON body (Anthropic messages format)
4. Runs ContextPruner::analyze() on the messages array
5. If should_prune: calls ContextPruner::prune() to amputate errors and inject recovery
6. Forwards the (possibly modified) request to the real API
7. Returns the response to the client

```rust
pub async fn start_proxy(
    listen_addr: SocketAddr,
    target_base: String,   // "https://api.anthropic.com"
    pruner: Arc<ContextPruner>,
) -> Result<()>
```

For tests: mock the target server with another axum instance that echoes the request body. Verify the proxy forwards correctly and prunes when errors are detected.

Tests:
- `proxy_forwards_clean_request` — no errors, request passes through unchanged
- `proxy_prunes_error_loop` — 3+ errors in messages → pruned messages are shorter

**Commit:** `git commit -m "feat(daemon): context pruning reverse proxy on configurable port"`

---

### Task 6: Local embeddings with fastembed

**Files:**
- Modify: `crates/memory/Cargo.toml` — add `fastembed = { version = "5", optional = true }`
- Create: `crates/memory/src/embeddings.rs`
- Modify: `crates/memory/src/lib.rs` — add `pub mod embeddings;`
- Modify: `crates/memory/src/graph.rs` — generate embeddings on insert when provider available
- Test: `crates/memory/tests/embeddings_test.rs`

**IMPORTANT:** fastembed is heavy (~100MB ONNX runtime). Gate behind a Cargo feature:
```toml
[features]
default = []
embeddings = ["fastembed"]

[dependencies]
fastembed = { version = "5", optional = true }
```

```rust
// embeddings.rs
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn dimension(&self) -> usize;
}

#[cfg(feature = "embeddings")]
pub struct FastEmbedProvider { model: fastembed::TextEmbedding }

#[cfg(feature = "embeddings")]
impl FastEmbedProvider {
    pub fn new() -> Result<Self> {
        let model = fastembed::TextEmbedding::try_new(
            fastembed::InitOptions::new(fastembed::EmbeddingModel::BGESmallENV15)
        )?;
        Ok(Self { model })
    }
}

#[cfg(feature = "embeddings")]
impl EmbeddingProvider for FastEmbedProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }
    fn dimension(&self) -> usize { 384 } // bge-small-en-v1.5
}

// No-op provider when feature is disabled
pub struct NoOpProvider;
impl EmbeddingProvider for NoOpProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> { Ok(vec![]) }
    fn dimension(&self) -> usize { 0 }
}
```

Tests (without embeddings feature — just test the trait and NoOp):
- `noop_provider_returns_empty`
- `trait_is_object_safe` (can be used as `Box<dyn EmbeddingProvider>`)

**Commit:** `git commit -m "feat(memory): embedding provider trait + fastembed backend (feature-gated)"`

---

### Task 7: Skill crystallizer

**Files:**
- Create: `crates/memory/src/crystallizer.rs`
- Modify: `crates/memory/src/lib.rs` — add `pub mod crystallizer;`
- Test: `crates/memory/tests/crystallizer_test.rs`

```rust
pub struct ToolPattern {
    pub tools: Vec<String>,        // ordered sequence: ["Grep", "Read", "Edit"]
    pub count: u32,                // how many times observed
    pub last_seen: String,
}

pub struct SkillCrystallizer {
    patterns: HashMap<String, ToolPattern>,  // hash of tool sequence → pattern
    threshold: u32,  // default 3
}

impl SkillCrystallizer {
    pub fn new(threshold: u32) -> Self
    pub fn observe(&mut self, tools_used: &[String])  // record a tool sequence
    pub fn crystallized(&self) -> Vec<&ToolPattern>    // patterns above threshold
    pub fn generate_skill_md(&self, pattern: &ToolPattern) -> String  // generate SKILL.md content
}
```

`generate_skill_md` produces:
```markdown
---
name: crystallized-{hash}
description: Auto-detected pattern: {tools joined}
user-invocable: true
---
# Crystallized Pattern
This pattern was detected {count} times.
Tools: {tools list}
```

Tests:
- `observe_counts_patterns`
- `threshold_filters_infrequent`
- `generate_produces_valid_markdown`

**Commit:** `git commit -m "feat(memory): skill crystallizer — auto-detect and template recurring tool patterns"`

---

### Task 8: Cross-session learning wire-up

**Files:**
- Modify: `crates/daemon/src/app_state.rs` — load heuristics from DB at startup
- Modify: `crates/daemon/src/api/heuristics.rs` — use loaded state
- Test: `crates/daemon/tests/api_test.rs` — verify loaded heuristics affect behavior

At daemon startup:
```rust
// In AppState::new():
let versions = store.load_heuristics().await?;
let outcomes = store.load_recent_outcomes(50).await?;
let mut evolver = HeuristicEvolver::new(20);
// Restore population from DB
for (id, gen, parent, fitness_json, rw_json, ss_json, created) in &versions {
    // Reconstruct HeuristicVersion from stored JSON
    // Add to evolver population
}
// Restore outcomes
for outcome_json in &outcomes {
    // Reconstruct SessionOutcome, record in evolver
}
```

Test: `heuristics_survive_restart` — save a version via API, recreate AppState (simulating restart), verify version is loaded.

**Commit:** `git commit -m "feat(daemon): cross-session learning — load heuristics from SQLite at daemon boot"`

---

### Task 9: Docs + version bump + merge

**Files:**
- `.claude-plugin/plugin.json` → v0.7.0
- `CHANGELOG.md` → v0.7.0 section
- `README.md` → update
- `memory-bank/progress.md` → Phase 9
- `memory-bank/activeContext.md` → v0.7.0 state

**Commit:** `git commit -m "docs: v0.7.0 Deep Foundation — changelog, readme, plugin version, memory-bank"`

Then push, PR, merge.
