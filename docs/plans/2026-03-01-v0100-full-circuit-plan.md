# v0.10.0 "Full Circuit" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect all dormant code into the live execution path. Adopt System 1 / System 2 model: hooks return fast, heavy processing runs async. Every component works end-to-end or is explicitly `[experimental]`.

**Architecture:** A new `SessionContext` in the daemon persists state across all hooks of a session. Each hook handler does its fast work (System 1), returns the HTTP response, then spawns a `tokio::spawn` background task for heavy work (System 2): graph population, sandbox checks, heuristic evolution. An integration test suite validates the full flow.

**Tech Stack:** Rust 2024, tokio (spawn for async), rusqlite, axum, clap. No new crate dependencies.

---

## Task 1: SessionContext Struct

**Files:**
- Create: `crates/daemon/src/session.rs`
- Modify: `crates/daemon/src/app_state.rs`
- Modify: `crates/daemon/src/lib.rs`
- Test: `crates/daemon/tests/session_test.rs`

**What to build:**

`SessionContext` struct holding accumulated state across hooks:
```rust
pub struct SessionContext {
    pub session_id: String,
    pub created_at: std::time::Instant,
    pub task_type: Option<TaskType>,
    pub risk: RiskLevel,
    pub strategy: Strategy,
    pub difficulty: f32,
    pub competence: f32,
    pub entropy_tracker: EntropyTracker,
    pub metacog_vector: MetacognitiveVector,
    pub verification_results: Vec<String>,
    pub lessons: Vec<String>,
    pub execution_plan: Option<ExecutionPlan>,
    pub tool_calls: u32,
    pub errors: u32,
    pub success_count: u32,
}
```

`SessionStore` wrapper: `HashMap<String, Arc<Mutex<SessionContext>>>` with `get_or_create(session_id)` and `remove(session_id)`.

Add `pub sessions: Arc<SessionStore>` to `AppState`.

Tests: create/retrieve/remove sessions, get_or_create is idempotent.

Commit: `feat(daemon): SessionContext struct for cross-hook state persistence`

---

## Task 2: Wire SessionContext into Hook Handlers

**Files:**
- Modify: `crates/daemon/src/api/hooks.rs`

**What to build:**

Replace `LoopContext::new(input)` in each handler with session-aware flow:

**user_prompt_submit:**
```
1. get_or_create SessionContext
2. Run stages 0-6 (classify→strategy) — same as today
3. Generate ExecutionPlan via TopologyPlanner::plan()
4. Persist task_type, risk, strategy, difficulty, execution_plan into SessionContext
5. Return response (fast)
```

**pre_tool_use:**
```
1. Get SessionContext (fallback to fresh if missing)
2. Read risk from SessionContext (don't re-classify)
3. Run guard pipeline (same as today)
4. Increment session.tool_calls
5. Return response (fast)
```

**post_tool_use:**
```
1. Get SessionContext
2. Track error/success in session (increment errors or success_count)
3. Return response (fast)
4. tokio::spawn: async post-processing (Task 3)
```

**stop:**
```
1. Get SessionContext
2. Use session.execution_plan with run_plan() instead of run_range(8,12)
3. Feed session.verification_results and session.lessons into LoopContext before running
4. Return response
5. tokio::spawn: async finalization (Task 4)
6. Remove session from store
```

Tests: verify SessionContext persists task_type from user_prompt_submit to pre_tool_use.

Commit: `feat(daemon): wire SessionContext into all hook handlers`

---

## Task 3: Async Post-Processing (System 2) — Graph + Entropy + Sandbox

**Files:**
- Modify: `crates/daemon/src/api/hooks.rs`
- Create: `crates/daemon/src/postprocess.rs`

**What to build:**

A `postprocess` module with async functions spawned after hooks return:

```rust
pub async fn after_post_tool_use(
    state: AppState,
    session: Arc<Mutex<SessionContext>>,
    tool_name: String,
    tool_response: String,
    was_error: bool,
) {
    // 1. Update entropy tracker
    let confidence = session.lock().unwrap().metacog_vector.confidence;
    session.lock().unwrap().entropy_tracker.record(confidence, !was_error);

    // 2. Insert Evidence node into graph memory
    let node = MemoryNode { /* ... */ };
    let _ = state.graph.insert_node(&node).await;

    // 3. Optional sandbox check (Write/Edit of .py files)
    if (tool_name == "Write" || tool_name == "Edit") && looks_like_python(&tool_response) {
        let result = state.sandbox.execute("python", &format!("import ast; ast.parse('''{}''')", escaped), 2000).await;
        if let Ok(r) = result {
            if !r.success {
                session.lock().unwrap().verification_results.push(format!("sandbox_syntax_error: {}", r.stderr));
            }
        }
    }
}
```

Similar `after_user_prompt_submit` (insert Task node) and `after_stop` (insert Decision + Lesson nodes, heuristic evolution — Task 4).

Commit: `feat(daemon): async System 2 post-processing (graph, entropy, sandbox)`

---

## Task 4: Heuristic Wire-up + Topology run_plan()

**Files:**
- Modify: `crates/daemon/src/postprocess.rs` (after_stop function)
- Modify: `crates/daemon/src/api/hooks.rs` (stop handler)

**What to build:**

**In stop handler:** Replace `run_range(&mut ctx, 8, 12)` with:
```rust
let plan = session.lock().unwrap().execution_plan.clone()
    .unwrap_or_else(|| TopologyPlanner::full_pipeline());
let decision = state.control_loop.run_plan(&mut ctx, &plan);
```

**In after_stop async post-processing:**
```rust
// Build SessionOutcome from SessionContext
let outcome = SessionOutcome {
    id: Uuid::new_v4().to_string(),
    session_id: session.session_id.clone(),
    task_type: session.task_type.map(|t| format!("{:?}", t)),
    risk_level: format!("{:?}", session.risk),
    strategy_used: format!("{:?}", session.strategy),
    success: session.errors == 0,
    tokens_consumed: budget.consumed_tokens,
    duration_ms: session.created_at.elapsed().as_millis() as u64,
    errors_encountered: session.errors,
};

// Record and evolve
let mut evolver = state.evolver.lock().unwrap();
evolver.record_outcome(outcome.clone());
if evolver.outcomes_count() >= 5 {
    evolver.evaluate_all();
}

// Persist to SQLite
state.memory.save_outcome(...).await;
```

Commit: `feat(daemon): heuristic evolution at session end + topology run_plan()`

---

## Task 5: Entropy + Plasticity in Decide Stage

**Files:**
- Modify: `crates/core/src/stages/decide.rs`
- Modify: `crates/core/src/context.rs`

**What to build:**

Add `overconfidence_score: f64` and `plasticity_lost: bool` fields to `LoopContext` (set by hook handler before running stages 8-12, read from SessionContext).

In `DecideStage::run()`, add two new checks before the default `Continue`:
```rust
// Overconfidence detected → Revise
if ctx.overconfidence_score > 0.3 {
    ctx.decision = Decision::Revise;
    tracing::warn!(stage = self.name(), score = ctx.overconfidence_score, "overconfidence → revise");
    return StageResult::Continue;
}

// Plasticity lost → Escalate
if ctx.plasticity_lost {
    ctx.decision = Decision::Escalate;
    return StageResult::Escalate("plasticity lost: model ignoring recovery feedback".into());
}
```

Commit: `feat(core): entropy + plasticity checks in decide stage`

---

## Task 6: CompactStage Rewrite

**Files:**
- Modify: `crates/core/src/stages/compact.rs`
- Test: `crates/core/tests/compact_test.rs`

**What to build:**

Replace string dedup with semantic compaction:

```rust
fn run(&self, ctx: &mut LoopContext) -> StageResult {
    // 1. Cluster lessons by word overlap (>50% shared → merge)
    let clustered = cluster_lessons(&ctx.lessons, 10);
    ctx.lessons = clustered;

    // 2. Age verification results — evict stale (age > 2)
    ctx.verification_results.retain(|r| !r.starts_with("[aged:"));
    // Mark remaining for aging (hook handler increments age counter)

    // 3. Generate compact summary
    let summary = format!(
        "[compact] {} lessons, {} verifications, quality={:.2}",
        ctx.lessons.len(),
        ctx.verification_results.len(),
        ctx.metacog_vector.overall_quality()
    );
    ctx.lessons.push(summary);

    StageResult::Continue
}

fn cluster_lessons(lessons: &[String], max_clusters: usize) -> Vec<String> {
    // Hash each lesson's words into a set, merge lessons with >50% overlap
    // Return merged lessons with count suffix: "risk=High difficulty=0.80 (x3)"
}
```

Tests: clustering merges similar lessons, different lessons kept separate, cap respected.

Commit: `feat(core): semantic CompactStage with lesson clustering and verification aging`

---

## Task 7: Fix Contrat Rust/TS + README Honesty

**Files:**
- Modify: `packages/shared/src/types.ts`
- Modify: `README.md`

**What to build:**

**TS schema fix:** Add missing fields to `HookInputSchema`:
```typescript
export const HookInputSchema = z.object({
  hook_event_name: HookEventSchema,
  session_id: z.string().optional(),
  cwd: z.string().optional(),
  tool_name: z.string().optional(),
  tool_input: z.record(z.unknown()).optional(),
  tool_response: z.string().optional(),
  prompt: z.string().optional(),
  error: z.string().optional(),
  last_assistant_message: z.string().optional(),
  source: z.string().optional(),      // NEW
  reason: z.string().optional(),      // NEW
  trigger: z.string().optional(),     // NEW
});
```

**README rewrite:** Replace the current feature-heavy README intro with honest sections:
- "What works today" (guards, verification, budget, fatigue, replay, MCP bridge)
- "Experimental features" (entropy, plasticity, UCB memory, heuristic evolution, topology)
- "Roadmap" (forge integration, WASM sandbox, marketplace)

Commit: `fix: align Rust/TS HookInput contract + honest README`

---

## Task 8: Integration Test Harness (20 Scenarios)

**Files:**
- Create: `crates/daemon/tests/integration_e2e.rs`

**What to build:**

A test that starts an in-memory daemon and POSTs hooks in sequence:

```rust
#[tokio::test]
async fn full_session_lifecycle() {
    let app = metaygn_daemon::build_app().await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, app));

    let client = reqwest::Client::new();
    let base = format!("http://{addr}");

    // 1. user_prompt_submit
    let resp = client.post(format!("{base}/hooks/user-prompt-submit"))
        .json(&json!({"hook_event_name": "UserPromptSubmit", "session_id": "test-1", "prompt": "fix the authentication bug"}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    assert!(body["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("risk:"));

    // 2. pre_tool_use (safe command)
    let resp = client.post(format!("{base}/hooks/pre-tool-use"))
        .json(&json!({"hook_event_name": "PreToolUse", "session_id": "test-1", "tool_name": "Bash", "tool_input": {"command": "cargo test"}}))
        .send().await.unwrap();
    // Should be allowed

    // 3. pre_tool_use (dangerous command)
    let resp = client.post(format!("{base}/hooks/pre-tool-use"))
        .json(&json!({"hook_event_name": "PreToolUse", "session_id": "test-1", "tool_name": "Bash", "tool_input": {"command": "rm -rf /"}}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["hookSpecificOutput"]["permissionDecision"], "deny");

    // 4. post_tool_use
    let resp = client.post(format!("{base}/hooks/post-tool-use"))
        .json(&json!({"hook_event_name": "PostToolUse", "session_id": "test-1", "tool_name": "Bash", "tool_response": "test result: ok. 42 passed"}))
        .send().await.unwrap();

    // 5. stop
    let resp = client.post(format!("{base}/hooks/stop"))
        .json(&json!({"hook_event_name": "Stop", "session_id": "test-1", "last_assistant_message": "Done! All tests pass."}))
        .send().await.unwrap();

    // Wait for async post-processing
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 6. Verify graph was populated
    let resp = client.get(format!("{base}/memory/graph/stats")).send().await.unwrap();
    let stats: Value = resp.json().await.unwrap();
    assert!(stats["node_count"].as_u64().unwrap() > 0, "graph should have nodes after session");

    // 7. Verify replay was recorded
    let resp = client.get(format!("{base}/replay/sessions")).send().await.unwrap();
    let sessions: Value = resp.json().await.unwrap();
    assert!(!sessions["sessions"].as_array().unwrap().is_empty());
}
```

Plus 19 additional focused scenarios (safety, classification, memory, calibration).

Commit: `test: integration E2E harness — full session lifecycle + 20 scenarios`

---

## Task 9: Docs & Version Bump

**Files:**
- Modify: `CHANGELOG.md`, `.claude-plugin/plugin.json`, `memory-bank/activeContext.md`, `memory-bank/progress.md`

Commit: `docs: v0.10.0 Full Circuit — changelog, plugin version, memory-bank`

---

## Task 10: Full Build & Test Verification

Run: `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace`

Fix any issues. Commit.

---

## Dependency Graph

```
Task 1 (SessionContext) ──→ Task 2 (Hook wiring) ──→ Task 3 (Async post-processing)
                                                  ──→ Task 4 (Heuristic + topology)
                                                  ──→ Task 5 (Decide stage)
Task 6 (CompactStage) — independent
Task 7 (TS contract + README) — independent
Task 8 (E2E tests) — depends on Tasks 1-5
Task 9 (Docs) — depends on all
Task 10 (Verification) — last
```

Parallelizable: Tasks 1+6+7 can run in parallel. Tasks 3+4+5 can run after Task 2. Task 8 after 1-5.

## Summary

| Task | Chantier | Files | Complexity |
|------|----------|-------|------------|
| 1 | SessionContext struct | 4 | Medium |
| 2 | Hook wiring | 1 (big) | High |
| 3 | Async post-processing | 2 | High |
| 4 | Heuristic + topology | 2 | Medium |
| 5 | Decide stage | 2 | Low |
| 6 | CompactStage rewrite | 2 | Medium |
| 7 | TS contract + README | 2 | Low |
| 8 | E2E tests | 1 (big) | High |
| 9 | Docs | 4 | Low |
| 10 | Verification | 0 | Low |
