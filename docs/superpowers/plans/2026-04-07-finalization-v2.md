# Aletheia-Nexus Finalization Plan (v1.1 -> v2.0 Production)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring Aletheia-Nexus from working v1.1.0 to production-ready v2.0 by fixing blockers, hardening error handling, filling test gaps, resolving the codex branch, and wiring the remaining OTEL integration.

**Architecture:** The system is already functionally complete (7 Rust crates, 8 TS hooks, 12-stage control loop, graph memory, guards, sandbox, TUI). This plan focuses on production hardening: replacing panic-on-mutex patterns, adding tests for untested stages and API endpoints, completing OTEL export, and merging or closing the codex branch.

**Tech Stack:** Rust 2024 (axum, tokio, SQLite), TypeScript (hooks), tree-sitter, fastembed, rmcp, ratatui, ed25519-dalek

---

## Phase 1: Production Blockers (P0)

### Task 1: Replace mutex `.expect()` with graceful error handling in hooks

**Files:**
- Modify: `crates/daemon/src/api/hooks.rs:20,32,352,375,514,524`
- Test: `crates/daemon/tests/api_test.rs`

- [ ] **Step 1: Write a helper function for safe mutex locking**

```rust
// In crates/daemon/src/api/hooks.rs, add at top of helpers section:

use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Safely lock a mutex, returning a 500 error if poisoned.
fn lock_or_500<T>(
    mutex: &std::sync::Mutex<T>,
    name: &str,
) -> Result<std::sync::MutexGuard<'_, T>, (StatusCode, Json<HookOutput>)> {
    mutex.lock().map_err(|_| {
        tracing::error!("{name} mutex poisoned — returning 500");
        let output = HookOutput {
            decision: Some(PermissionDecision::Allow),
            reason: Some(format!("Internal error: {name} mutex poisoned")),
            hook_specific_output: None,
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(output))
    })
}
```

- [ ] **Step 2: Replace all 6 `.expect("...mutex poisoned")` calls**

Replace each occurrence:
```rust
// Line 20: state.budget.lock().expect("budget mutex poisoned")
let budget = lock_or_500(&state.budget, "budget")?;

// Line 32: session.lock().expect("session mutex poisoned")
let sess = lock_or_500(&session, "session")?;

// Line 352: state.fatigue.lock().expect("fatigue mutex poisoned")
let mut profiler = lock_or_500(&state.fatigue, "fatigue")?;

// Line 375: state.plasticity.lock().expect("plasticity mutex poisoned")
let mut tracker = lock_or_500(&state.plasticity, "plasticity")?;

// Line 514: state.fatigue.lock().expect("fatigue mutex poisoned")
let mut profiler = lock_or_500(&state.fatigue, "fatigue")?;

// Line 524: state.budget.lock().expect("budget mutex poisoned")
let mut budget = lock_or_500(&state.budget, "budget")?;
```

Note: Handler return types must be updated to `Result<Json<HookOutput>, (StatusCode, Json<HookOutput>)>` for handlers using these locks.

- [ ] **Step 3: Run tests**

Run: `cargo test -p metaygn-daemon`
Expected: All existing tests pass (the lock_or_500 path doesn't change happy-path behavior)

- [ ] **Step 4: Commit**

```bash
git add crates/daemon/src/api/hooks.rs
git commit -m "fix(daemon): replace mutex .expect() with graceful 500 error — prevent daemon panics"
```

---

### Task 2: Grep and fix remaining `.unwrap()` in daemon hot paths

**Files:**
- Modify: `crates/daemon/src/api/*.rs` (scan all endpoint files)
- Modify: `crates/daemon/src/session.rs`
- Modify: `crates/daemon/src/postprocess.rs`

- [ ] **Step 1: Audit all unwrap/expect in non-test daemon code**

Run: `cargo clippy -p metaygn-daemon -- -W clippy::unwrap_used 2>&1 | head -80`

Categorize each hit as:
- **Hot path** (hook handlers, session management) -> must fix
- **Startup/init** (one-time setup) -> acceptable with `.expect("reason")`
- **Infallible** (e.g., regex compilation with known-good pattern) -> acceptable

- [ ] **Step 2: Fix hot-path unwraps with `?` or `.unwrap_or_default()`**

Only fix the ones identified in hot paths. Use `?` propagation where the function already returns `Result`, and `.unwrap_or_default()` for non-critical display values.

- [ ] **Step 3: Run tests and clippy**

Run: `cargo test -p metaygn-daemon && cargo clippy -p metaygn-daemon`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/daemon/src/
git commit -m "fix(daemon): replace hot-path unwrap() with error propagation"
```

---

## Phase 2: Test Coverage for Core Stages (P1)

### Task 3: Test classify stage

**Files:**
- Create: `crates/core/tests/classify_test.rs`
- Reference: `crates/core/src/stages/classify.rs`

- [ ] **Step 1: Read classify.rs to understand its logic**

- [ ] **Step 2: Write tests**

```rust
use metaygn_core::context::LoopContext;
use metaygn_core::stages::classify::ClassifyStage;
use metaygn_shared::state::TaskType;

#[test]
fn classifies_bugfix_from_error_keywords() {
    let mut ctx = LoopContext::new("test-session".into());
    ctx.raw_input = "fix the NullPointerException in auth".into();
    ClassifyStage.run(&mut ctx);
    assert_eq!(ctx.task_type, TaskType::Bugfix);
}

#[test]
fn classifies_feature_from_add_keywords() {
    let mut ctx = LoopContext::new("test-session".into());
    ctx.raw_input = "add a new login page with OAuth".into();
    ClassifyStage.run(&mut ctx);
    assert_eq!(ctx.task_type, TaskType::Feature);
}

#[test]
fn classifies_security_from_security_keywords() {
    let mut ctx = LoopContext::new("test-session".into());
    ctx.raw_input = "audit the authentication middleware for vulnerabilities".into();
    ClassifyStage.run(&mut ctx);
    assert_eq!(ctx.task_type, TaskType::Security);
}

#[test]
fn defaults_to_general_for_ambiguous_input() {
    let mut ctx = LoopContext::new("test-session".into());
    ctx.raw_input = "update the readme".into();
    ClassifyStage.run(&mut ctx);
    assert_eq!(ctx.task_type, TaskType::General);
}
```

- [ ] **Step 3: Run test to verify**

Run: `cargo test -p metaygn-core classify`
Expected: PASS (adjust assertions based on actual classify logic)

- [ ] **Step 4: Commit**

```bash
git add crates/core/tests/classify_test.rs
git commit -m "test(core): add classify stage tests — 4 scenarios"
```

---

### Task 4: Test assess stage

**Files:**
- Create: `crates/core/tests/assess_test.rs`
- Reference: `crates/core/src/stages/assess.rs`

- [ ] **Step 1: Read assess.rs to understand risk/difficulty logic**

- [ ] **Step 2: Write tests for risk assessment**

Write tests covering:
- Destructive commands (rm -rf, git push --force) -> HIGH risk
- Safe read commands (ls, cat, git status) -> LOW risk
- Production keywords (deploy, migrate) -> HIGH risk
- Standard dev commands (cargo test) -> LOW risk
- Entropy/difficulty calculation sanity checks

- [ ] **Step 3: Run and verify**

Run: `cargo test -p metaygn-core assess`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/core/tests/assess_test.rs
git commit -m "test(core): add assess stage tests — risk classification scenarios"
```

---

### Task 5: Test decide stage

**Files:**
- Create: `crates/core/tests/decide_test.rs`
- Reference: `crates/core/src/stages/decide.rs`

- [ ] **Step 1: Read decide.rs to understand decision logic**

- [ ] **Step 2: Write tests**

Test the decision matrix:
- LOW risk + high competence -> Allow
- HIGH risk + low competence -> Escalate
- Guard-blocked command -> Deny
- Overconfidence detected (entropy > 0.3) -> Revise
- Plasticity lost -> Escalate

- [ ] **Step 3: Run and verify**

Run: `cargo test -p metaygn-core decide`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/core/tests/decide_test.rs
git commit -m "test(core): add decide stage tests — permission decision matrix"
```

---

### Task 6: Test strategy, tool_need, and budget stages

**Files:**
- Create: `crates/core/tests/strategy_test.rs`
- Create: `crates/core/tests/tool_need_test.rs`
- Create: `crates/core/tests/budget_test.rs`
- Reference: `crates/core/src/stages/strategy.rs`, `tool_need.rs`, `budget.rs`

- [ ] **Step 1: Read each stage's source**

- [ ] **Step 2: Write strategy tests**

Test strategy selection:
- HIGH risk -> verify-first
- LOW risk + simple task -> incremental
- Security task -> verify-first regardless of risk

- [ ] **Step 3: Write tool_need tests**

Test tool necessity:
- Echo-only bash commands -> tool not needed
- File write operations -> tool needed
- Read-only commands -> tool needed (for evidence)

- [ ] **Step 4: Write budget tests**

Test budget allocation:
- Budget consumed tracking
- 80% utilization warning
- Over-budget detection

- [ ] **Step 5: Run all**

Run: `cargo test -p metaygn-core strategy tool_need budget`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/core/tests/strategy_test.rs crates/core/tests/tool_need_test.rs crates/core/tests/budget_test.rs
git commit -m "test(core): add strategy, tool_need, budget stage tests"
```

---

## Phase 3: Daemon API Test Coverage (P1)

### Task 7: Test key daemon API endpoints

**Files:**
- Create: `crates/daemon/tests/endpoint_coverage_test.rs`
- Reference: `crates/daemon/src/api/budget.rs`, `calibration.rs`, `graph.rs`, `memory.rs`, `session_state.rs`

- [ ] **Step 1: Write integration test using `build_app_with_state()`**

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use metaygn_daemon::build_app_with_state;

#[tokio::test]
async fn budget_endpoint_returns_200() {
    let app = build_app_with_state(/* test state */);
    let resp = app
        .oneshot(Request::get("/budget/test-session").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
```

Test endpoints: `/budget/{id}`, `/calibration`, `/memory/stats`, `/memory/graph/stats`, `/session/{id}/state`, `/profiler/fatigue`, `/heuristics/best`

- [ ] **Step 2: Run tests**

Run: `cargo test -p metaygn-daemon endpoint_coverage`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/daemon/tests/endpoint_coverage_test.rs
git commit -m "test(daemon): add API endpoint coverage tests — 7 endpoints"
```

---

## Phase 4: Codex Branch Resolution (P1)

### Task 8: Evaluate and cherry-pick from codex branch

**Files:**
- Review: `git diff master...origin/codex`
- Modify: various (depending on cherry-pick decisions)

- [ ] **Step 1: Review codex branch changes**

The codex branch adds:
- `AGENTS.md` - documentation for Codex workflow
- `docs/CODEX-WORKFLOW.md` - MCP-based workflow docs
- `scripts/install-codex.sh/.ps1` - installation scripts
- `scripts/start-codex-metaygn.sh/.ps1` - launch scripts
- Hardening in `assess.rs`, `hooks.rs`, `sandbox.rs`, `memory.rs`

Decision criteria:
- Keep: documentation, install scripts, hardening fixes
- Reject: anything that deletes policy traits or removes tests

- [ ] **Step 2: Cherry-pick useful commits**

```bash
# Review the single commit on codex
git log origin/codex --oneline --not master
# Cherry-pick or manually port the hardening changes
git cherry-pick --no-commit origin/codex
# Review staged changes, unstage anything that removes policy.rs or tests
git diff --cached --stat
```

- [ ] **Step 3: Resolve conflicts and test**

Run: `cargo test --workspace`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: cherry-pick codex hardening — install scripts, assess improvements, docs"
```

- [ ] **Step 5: Close the codex branch if fully integrated**

(Coordinate with team — this is a judgment call)

---

## Phase 5: OTEL Completion (P2)

### Task 9: Wire OpenTelemetry exporter

**Files:**
- Modify: `crates/daemon/src/main.rs`
- Create: `crates/daemon/src/telemetry.rs`
- Reference: `crates/daemon/Cargo.toml` (otel feature already declared)

- [ ] **Step 1: Read current tracing setup in main.rs**

- [ ] **Step 2: Create telemetry.rs module**

```rust
//! Optional OpenTelemetry initialization.
//! Active only when compiled with `--features otel`.

#[cfg(feature = "otel")]
pub fn init_otel_tracing() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace::TracerProvider;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .build()?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());

    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(provider.tracer("aletheiad"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}

#[cfg(not(feature = "otel"))]
pub fn init_otel_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // No-op when otel feature is disabled
    tracing_subscriber::fmt::init();
    Ok(())
}
```

- [ ] **Step 3: Wire into main.rs**

```rust
mod telemetry;

#[tokio::main]
async fn main() {
    telemetry::init_otel_tracing().expect("failed to initialize tracing");
    // ... rest of main
}
```

- [ ] **Step 4: Test with and without feature**

Run: `cargo build -p metaygn-daemon` (without otel - must compile)
Run: `cargo build -p metaygn-daemon --features otel` (with otel - must compile)
Expected: Both compile successfully

- [ ] **Step 5: Commit**

```bash
git add crates/daemon/src/telemetry.rs crates/daemon/src/main.rs
git commit -m "feat(daemon): wire OpenTelemetry OTLP exporter behind --features otel"
```

---

## Phase 6: Documentation & Release (P2)

### Task 10: Update documentation for v2.0

**Files:**
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `.claude-plugin/plugin.json`
- Modify: `docs/HOW-TO.md`

- [ ] **Step 1: Update CHANGELOG.md**

Add v2.0 section documenting:
- Mutex panic fix (P0)
- Test coverage expansion
- OTEL exporter wiring
- Codex branch resolution

- [ ] **Step 2: Bump version in plugin.json and Cargo.toml**

```bash
# Cargo.toml workspace version
sed -i 's/version = "1.0.0"/version = "2.0.0"/' Cargo.toml
# plugin.json
# Update version field
```

- [ ] **Step 3: Update README "What Works Today" section**

Move features from "Experimental" to "What Works Today" based on test evidence:
- EGPO entropy -> confirmed (tested)
- RL2F plasticity -> confirmed (tested)
- UCB recall -> confirmed (tested)
- Heuristic evolution -> confirmed (tested)
- OTEL -> confirmed (wired)

- [ ] **Step 4: Run full CI locally**

Run: `cargo test --workspace && cargo clippy --workspace`
Expected: PASS

- [ ] **Step 5: Tag release**

```bash
git tag -a v2.0.0 -m "v2.0.0: Production Hardened"
```

---

## Phase Summary

| Phase | Tasks | Priority | Effort |
|-------|-------|----------|--------|
| P0: Blockers | Task 1-2 | Critical | ~1h |
| P1: Core Tests | Task 3-6 | High | ~2h |
| P1: API Tests | Task 7 | High | ~1h |
| P1: Codex | Task 8 | High | ~30min |
| P2: OTEL | Task 9 | Medium | ~1h |
| P2: Docs | Task 10 | Medium | ~30min |

**Total estimated effort: ~6h**

---

## What's NOT in scope (and why)

| Item | Reason |
|------|--------|
| Self-rewriting skills | Not in MVP per CLAUDE.md |
| A2A / swarm orchestration | Not in MVP per CLAUDE.md |
| Custom training pipeline | Not in MVP per CLAUDE.md |
| Rate limiting on API | Daemon is localhost-only (127.0.0.1), no external exposure |
| CORS/CSRF protection | Same reason - localhost-only |
| Config file schema | Low impact - settings work via env vars + CLI flags |
| Systemd/launchd service | Nice-to-have, not production blocker for local-first tool |
