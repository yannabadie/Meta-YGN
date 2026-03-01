# v0.12.0 "Observable Runtime" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove dead code, fix Expert 1 P0 issues (budget display, MCP contract), add OpenTelemetry + tree-sitter + adaptive competence + calibration harness + marketplace packaging.

**Architecture:** Cleanup first (delete mcp-bridge, scripts/, fix budget), then feature-gated additions (otel, syntax). Competence stage queries SQLite for historical success rates. Eval harness is a new CLI command reading existing tables. All heavy deps are optional features.

**Tech Stack:** Rust 2024, tree-sitter (feature `syntax`), opentelemetry (feature `otel`), serde_yaml, toml.

---

## Task 1: Delete Dead Code (mcp-bridge + scripts/)

**Files:**
- Delete: `crates/mcp-bridge/` (entire directory)
- Delete: `scripts/*.py`, `scripts/common.py`, `scripts/__pycache__/` (Python hooks — replaced by TS in v0.6.0)
- Keep: `scripts/install.sh`, `scripts/validate-plugin.sh`, `scripts/hook-runner.sh` (still useful)
- Modify: `Cargo.toml` (root) — remove `"crates/mcp-bridge"` from workspace members

**Verify:** `cargo build --workspace` — must compile (8→7 crates)

**Commit:** `chore: remove dead code — mcp-bridge crate + legacy Python hooks`

---

## Task 2: Fix Budget Display (Expert 1 P0)

**Files:**
- Modify: `crates/daemon/src/api/hooks.rs` — change `append_budget_to_output` to read session budget

**What to build:** The `append_budget_to_output` function currently reads `state.budget` (global). Change it to accept a session reference and read session-local budget instead:

```rust
fn append_budget_to_output(output: &mut HookOutput, session: &Arc<Mutex<SessionContext>>) {
    let sess = session.lock().expect("session mutex poisoned");
    let summary = sess.budget.summary();
    // ... rest unchanged
}
```

Update all call sites (7 occurrences) to pass `&session_ctx` instead of `&state`.

For hooks that run before a session exists (early returns in pre_tool_use for guard deny), fall back to global budget.

**Verify:** `cargo test -p metaygn-daemon`

**Commit:** `fix(daemon): budget display reads session budget, not global (Expert 1 P0)`

---

## Task 3: Fix MCP Tool Contract (Expert 1 P0)

**Files:**
- Modify: `crates/daemon/src/mcp.rs` — fix HookInput construction in metacog_classify and metacog_verify

**What to fix:**
- `metacog_classify` constructs HookInput without all required context — add `cwd`, ensure all fields present
- `metacog_verify` uses param name `tool_output` but maps to `tool_response` — this is correct mapping but field naming in schema is confusing. Add doc comment clarifying the mapping.
- Ensure all 5 MCP tools construct valid HookInput payloads that would pass the same validation as HTTP hooks

**Verify:** `cargo build -p metaygn-daemon --features mcp`

**Commit:** `fix(daemon): MCP tool HookInput contract alignment (Expert 1 P0)`

---

## Task 4: Align README Claims with Tests

**Files:**
- Modify: `README.md` — update "What Works Today" with exact test references

**What to build:** Each feature in "What Works Today" must cite the specific E2E test that proves it:
- Guard Pipeline → `guard_blocks_destructive_command`
- Session Replay → `full_session_lifecycle` (verifies replay/sessions endpoint)
- etc.

Features without E2E tests move to "Experimental" or get tests added.

**Commit:** `docs: align README claims with actual test coverage (Expert 1 P0)`

---

## Task 5: Tree-sitter Multi-Language Verification (feature: `syntax`)

**Files:**
- Modify: `crates/verifiers/Cargo.toml` — add tree-sitter deps behind `syntax` feature
- Create: `crates/verifiers/src/syntax.rs` — multi-language syntax checker
- Modify: `crates/verifiers/src/lib.rs` — conditional export
- Modify: `crates/daemon/Cargo.toml` — forward `syntax` feature to verifiers
- Modify: `crates/daemon/src/api/hooks.rs` — call syntax check in post_tool_use Tier 1.5
- Test: `crates/verifiers/tests/syntax_test.rs`

**What to build:**

```rust
// crates/verifiers/src/syntax.rs
#[cfg(feature = "syntax")]
pub fn check_syntax(content: &str, extension: &str) -> Vec<SyntaxError> {
    let language = match extension {
        "rs" => tree_sitter_rust::LANGUAGE,
        "py" => tree_sitter_python::LANGUAGE,
        "js" => tree_sitter_javascript::LANGUAGE,
        "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        _ => return vec![],
    };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language.into()).ok();
    let tree = parser.parse(content, None);
    // Walk tree, collect ERROR nodes with line/column
    extract_errors(tree)
}
```

Wire into `post_tool_use` Tier 1.5: after serde validation (Tier 1), before forge sandbox (Tier 2). Feature-gated behind `syntax`.

Tests: valid Rust, invalid Rust, valid Python, invalid Python, unknown extension → empty.

**Commit:** `feat(verifiers): tree-sitter multi-language syntax verification (feature: syntax)`

---

## Task 6: OpenTelemetry Observability (feature: `otel`)

**Files:**
- Modify: `crates/daemon/Cargo.toml` — add otel deps behind `otel` feature
- Create: `crates/daemon/src/metrics.rs` — Prometheus metrics + OTel span helpers
- Modify: `crates/daemon/src/api/mod.rs` — add `/metrics` route
- Modify: `crates/core/src/runner.rs` — add tracing spans per stage

**What to build:**

Prometheus endpoint at `GET /metrics`:
```
# HELP metaygn_hook_duration_seconds Hook processing time
# TYPE metaygn_hook_duration_seconds histogram
metaygn_hook_duration_seconds_bucket{hook="PreToolUse",le="0.01"} 42
...
metaygn_sessions_total 15
metaygn_escalations_total 2
metaygn_tokens_consumed_total 50000
```

Stage-level tracing spans (automatic via `tracing` crate — when OTel exporter is configured, they become OTel spans):
```rust
// In runner.rs run() method:
let _span = tracing::info_span!("metaygn_stage", name = stage.name()).entered();
```

Feature-gated: default build has zero overhead.

**Commit:** `feat(daemon): OpenTelemetry spans + Prometheus /metrics endpoint (feature: otel)`

---

## Task 7: Adaptive Competence Stage

**Files:**
- Modify: `crates/core/src/context.rs` — add `historical_success_rate: Option<f32>` field
- Modify: `crates/core/src/stages/competence.rs` — blend with historical rate
- Modify: `crates/daemon/src/api/hooks.rs` — query outcomes before running stages
- Test: `crates/core/tests/competence_test.rs`

**What to build:** Before running stages 0-6 in user_prompt_submit, query the last 20 SessionOutcome records for this task type:
```rust
let historical = state.memory.get_success_rate_for_task_type(&task_type_str, 20).await;
ctx.historical_success_rate = historical;
```

In CompetenceStage: `final = 0.5 * base + 0.5 * historical_rate` (fallback to base when historical is None or <5 data points).

New MemoryStore method: `get_success_rate_for_task_type(task_type, limit) -> Option<f32>`.

Tests: historical rate lowers competence for failing task types, raises for succeeding ones, fallback when no data.

**Commit:** `feat(core): adaptive competence from session history (SSR-inspired)`

---

## Task 8: Calibration Evaluation Harness

**Files:**
- Create: `crates/cli/src/eval.rs` — calibration report generator
- Modify: `crates/cli/src/main.rs` — add `Eval` command
- Modify: `crates/memory/src/store.rs` — add `load_calibration_data()` method
- Test: `crates/cli/tests/eval_test.rs` (or inline tests)

**What to build:** `aletheia eval` command:
1. Load all SessionOutcome + replay_events from SQLite
2. Compute Brier score: `mean((confidence - outcome)²)`
3. Compute escalation precision: `justified_escalations / total_escalations`
4. Output JSON + human-readable summary:
```
Calibration Report (42 sessions)
  Brier score: 0.18 (lower is better; <0.25 = well calibrated)
  Escalation precision: 0.85 (85% of escalations were justified)
  Avg hook latency: 23ms
  Token efficiency: 4,200 tokens/session avg
```

**Commit:** `feat(cli): 'aletheia eval' calibration harness with Brier score`

---

## Task 9: Marketplace Packaging

**Files:**
- Create: `crates/cli/src/doctor.rs` — health check command
- Modify: `crates/cli/src/main.rs` — add `Doctor` command
- Modify: `scripts/install.sh` — improve platform detection
- Modify: `.claude-plugin/plugin.json` — ensure marketplace compliance

**What to build:** `aletheia doctor`:
```
Aletheia Doctor
  Daemon:     RUNNING (port 52341)
  Plugin:     VALID (.claude-plugin/plugin.json)
  Hooks:      8/8 configured
  Skills:     8/8 present
  Agents:     6/6 present
  Version:    CLI=0.12.0 Daemon=0.12.0 Plugin=0.12.0
  DB:         ~/.claude/aletheia/metaygn.db (42 sessions, 156 events)
```

**Commit:** `feat(cli): 'aletheia doctor' health check + marketplace polish`

---

## Task 10: E2E Tests Reinforced

**Files:**
- Modify: `crates/daemon/tests/integration_e2e.rs`

Add 5+ new scenarios covering Expert 1 claims:
- `budget_display_is_session_scoped` — two sessions, verify budget doesn't bleed
- `forge_validates_invalid_json_write` — Write invalid JSON → response contains SYNTAX ERROR
- `completion_verifier_blocks_false_done` — stop with missing files → blocked
- `replay_records_all_hook_types` — full session → replay has entries for each hook
- `graph_populated_after_session` — full session → graph has Task + Evidence + Decision nodes

**Commit:** `test: reinforced E2E covering budget isolation, forge, completion, replay, graph`

---

## Task 11: Docs & Version Bump

**Files:** CHANGELOG.md, plugin.json, memory-bank/, daemon-contract.md, progress.md

**Commit:** `docs: v0.12.0 Observable Runtime — changelog, contract, memory-bank`

---

## Task 12: Full Verification

`cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Also: `cargo build --features "mcp,otel,syntax"` (all features)

---

## Dependency Graph

```
Task 1 (cleanup) ── independent, do first
Task 2 (budget fix) ── after Task 1
Task 3 (MCP contract) ── after Task 1
Task 4 (README) ── after Task 10
Task 5 (tree-sitter) ── independent
Task 6 (OTel) ── independent
Task 7 (competence) ── independent
Task 8 (eval harness) ── independent
Task 9 (marketplace) ── independent
Task 10 (E2E tests) ── after Tasks 1-3, 5, 7
Task 11 → Task 12
```

Parallel: Tasks 1+5+6+7+8+9 can all run in parallel. Tasks 2+3 after 1.

## Summary

| Task | What | Complexity |
|------|------|------------|
| 1 | Delete dead code | Low |
| 2 | Fix budget display (P0) | Low |
| 3 | Fix MCP contract (P0) | Low |
| 4 | Align README | Low |
| 5 | Tree-sitter | High |
| 6 | OpenTelemetry | High |
| 7 | Adaptive competence | Medium |
| 8 | Calibration harness | Medium |
| 9 | Marketplace packaging | Medium |
| 10 | E2E tests | High |
| 11 | Docs | Low |
| 12 | Verification | Low |
