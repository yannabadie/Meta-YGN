# v0.11.0 "Hardened Core" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix P0 foundations (token accounting, session-scoped profilers, verification→decide, CLI truth), integrate ForgeEngine verification in post_tool_use, fuse MCP bridge into daemon, upgrade verify stage, and prove everything with E2E tests.

**Architecture:** P0 fixes are structural changes to SessionContext and hooks. ForgeEngine uses tiered verification: Tier 1 in-process (serde for config files), Tier 2 async sandbox (forge templates for code). MCP fusion moves handler.rs into daemon behind `--mcp` flag. Verify stage reads intended_action + tool-specific parsing.

**Tech Stack:** Rust 2024, tokio, axum, rmcp 0.17 (optional feature), serde_yaml, toml crate.

---

## Task 1: Fix Token Accounting (P0)

**Files:**
- Modify: `crates/daemon/src/session.rs` — add `tokens_consumed: u64` field
- Modify: `crates/daemon/src/api/hooks.rs` — copy budget.consumed_tokens into session at each hook
- Modify: `crates/daemon/src/postprocess.rs` — read tokens from session for SessionOutcome

**What to build:** Add `tokens_consumed: u64` to SessionContext. In each hook handler, after budget operations, copy `state.budget.lock().consumed_tokens` into `session.tokens_consumed`. In `after_stop`, read `session.tokens_consumed` instead of hardcoded `0`.

**Test:** Existing `stop_records_heuristic_outcome` E2E test should verify `tokens_consumed > 0` (add assertion).

**Commit:** `fix(daemon): wire token accounting into SessionContext and SessionOutcome`

---

## Task 2: Session-Scoped Profilers (P0)

**Files:**
- Modify: `crates/daemon/src/session.rs` — add `fatigue: FatigueProfiler`, `plasticity: PlasticityTracker`, `budget: SessionBudget` fields
- Modify: `crates/daemon/src/api/hooks.rs` — use session.fatigue/plasticity/budget instead of global AppState ones
- Test: `crates/daemon/tests/session_test.rs` — add test for session isolation

**What to build:** Move the per-session tracking into SessionContext. The global AppState profilers remain for aggregate /health endpoint data. Each hook handler reads/writes the session's profiler, not the global one. This prevents cross-session contamination.

**Key change in hooks.rs:** Replace `state.fatigue.lock()` with `session.lock().unwrap().fatigue` calls. Same for plasticity and budget.

**Test:** New test `session_profilers_are_isolated` — two sessions with different IDs don't share fatigue/budget state.

**Commit:** `fix(daemon): session-scoped fatigue/plasticity/budget profilers`

---

## Task 3: Verification → Decide Pipeline (P0)

**Files:**
- Modify: `crates/core/src/stages/calibrate.rs` — read verification_results, adjust confidence
- Modify: `crates/core/src/stages/decide.rs` — check for verification errors
- Test: `crates/core/tests/decide_test.rs` — add test for verification-driven revise

**What to build:** In `CalibrateStage::run()`, count entries in `ctx.verification_results` that indicate errors. Apply additional confidence penalty: `penalty = error_count * 0.1`. In `DecideStage::run()`, add a check: if `verification_results` contains any `tool_error` or `response_contains` patterns, force `Decision::Revise`.

Note: The stop handler already copies `session.verification_results` into `ctx.verification_results` before running stages 8-12. This task makes the stages actually READ those results.

**Commit:** `fix(core): verification results feed into calibrate and decide stages`

---

## Task 4: CLI Truth (P0)

**Files:**
- Modify: `crates/cli/src/main.rs` — remove `--host` and `--port` from Start command

**What to build:** Remove the `host: String` and `port: u16` fields from the `Start` command variant. The daemon always binds to `127.0.0.1:0`. The `--db-path` flag stays (it's actually useful). Update the `cmd_start` function signature accordingly.

**Commit:** `fix(cli): remove misleading --host/--port flags from 'aletheia start'`

---

## Task 5: ForgeEngine Tier 1 — In-Process Config Validation

**Files:**
- Modify: `crates/daemon/Cargo.toml` — add `serde_yaml` and `toml` deps
- Create: `crates/daemon/src/verification.rs` — in-process file validation module
- Modify: `crates/daemon/src/api/hooks.rs` — call verification in post_tool_use
- Modify: `crates/daemon/src/lib.rs` — add `pub mod verification;`
- Test: `crates/daemon/tests/verification_test.rs`

**What to build:** A `verification` module with a `validate_file_content(file_path: &str, content: &str) -> Option<String>` function:
- `.json` → `serde_json::from_str::<serde_json::Value>(content)` — return error message on parse failure
- `.yaml`/`.yml` → `serde_yaml::from_str::<serde_yaml::Value>(content)` — return error on parse failure
- `.toml` → `toml::from_str::<toml::Value>(content)` — return error on parse failure
- Other extensions → `None` (no validation)

In `post_tool_use` handler, after extracting tool_name and response, if tool is Write or Edit:
1. Extract `file_path` from `tool_input`
2. Extract file content from `tool_response` (or `tool_input.content` for Write)
3. Call `validate_file_content`
4. If error, append to `context` string: `"SYNTAX ERROR in {file}: {error}"`

Tests: validate valid JSON, invalid JSON, valid YAML, invalid YAML, valid TOML, invalid TOML, unknown extension returns None.

**Commit:** `feat(daemon): Tier 1 in-process config file validation (JSON/YAML/TOML)`

---

## Task 6: ForgeEngine Tier 2 — Async Sandbox Verification

**Files:**
- Modify: `crates/daemon/src/forge/templates.rs` — add `syntax-checker` template
- Modify: `crates/daemon/src/postprocess.rs` — call forge in after_post_tool_use for .py files
- Test: `crates/daemon/tests/forge_test.rs` — add test for syntax-checker template

**What to build:** New template `syntax-checker`:
```python
import ast, sys, json
code = sys.stdin.read()
try:
    ast.parse(code)
    print(json.dumps({"valid": True}))
except SyntaxError as e:
    print(json.dumps({"valid": False, "error": str(e), "line": e.lineno}))
```

In `after_post_tool_use`, if `tool_name == "Write" || tool_name == "Edit"` and the file extension is `.py`:
1. Extract file content
2. Create a temporary ForgeEngine (to avoid mutex contention)
3. Call `forge.forge_and_run("syntax-checker", &HashMap::new(), &content)`
4. If result shows `valid: false`, append error to `session.verification_results`

Use 2-second timeout for forge execution.

**Commit:** `feat(daemon): Tier 2 async sandbox verification for Python files`

---

## Task 7: MCP Fusion — Move Handler into Daemon

**Files:**
- Modify: `crates/daemon/Cargo.toml` — add `rmcp` as optional dependency behind `mcp` feature
- Create: `crates/daemon/src/mcp.rs` — MCP handler using AppState directly
- Modify: `crates/daemon/src/main.rs` — add `--mcp` flag to switch modes
- Modify: `crates/daemon/src/lib.rs` — add `#[cfg(feature = "mcp")] pub mod mcp;`

**What to build:** Move the 5 tool definitions from `crates/mcp-bridge/src/handler.rs` into `crates/daemon/src/mcp.rs`. Key difference: instead of `DaemonClient` making HTTP calls, the tools directly access `AppState` methods.

The daemon's `main.rs` gets a `--mcp` CLI flag:
```rust
#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    mcp: bool,
}
```

If `--mcp`: start rmcp stdio server with the handler. Else: start axum HTTP server (default).

Feature-gated behind `mcp` feature in Cargo.toml so default builds don't pull rmcp.

**Commit:** `feat(daemon): fuse MCP server into daemon behind --mcp flag`

---

## Task 8: Update CLI + MCP Bridge Crate

**Files:**
- Modify: `crates/cli/src/main.rs` — update `cmd_mcp` to launch `aletheiad --mcp`
- Modify: `crates/mcp-bridge/Cargo.toml` — mark as deprecated or thin redirect
- Modify: `Cargo.toml` — optionally remove mcp-bridge from workspace members

**What to build:** Update `cmd_mcp()` in CLI to find and launch `aletheiad --mcp` instead of `aletheia-mcp`. The mcp-bridge crate can be kept as empty redirect or removed. If removed, update workspace Cargo.toml.

**Commit:** `refactor(cli): 'aletheia mcp' now launches 'aletheiad --mcp'`

---

## Task 9: Verify Stage Upgrade

**Files:**
- Modify: `crates/core/src/stages/verify.rs` — read intended_action, parse tool responses
- Test: `crates/core/tests/verify_test.rs`

**What to build:** Enhance `VerifyStage::run()`:

1. **Read intended_action** (if set by act stage):
   ```rust
   if let Some(ref action) = ctx.intended_action {
       // If tool_name is present, check it matches
       if let Some(ref tool_name) = ctx.input.tool_name {
           if *tool_name != action.tool {
               ctx.verification_results.push(format!(
                   "tool_mismatch: intended {} but got {}", action.tool, tool_name
               ));
           }
       }
   }
   ```

2. **Parse test results from Bash output:**
   ```rust
   if tool_name == "Bash" {
       // Look for "N passed; M failed" pattern
       if let Some(caps) = regex::Regex::new(r"(\d+) passed.*?(\d+) failed").ok()
           .and_then(|re| re.captures(&response))
       {
           let failed: u32 = caps[2].parse().unwrap_or(0);
           if failed > 0 {
               ctx.verification_results.push(format!("test_failures: {failed} tests failed"));
           }
       }
   }
   ```

Keep the existing keyword scan as a fallback for tools that don't have specific parsing.

Tests: verify intended_action mismatch detected, test result parsing extracts failure count, keyword scan still works as fallback.

**Commit:** `feat(core): verify stage reads intended_action + parses test results`

---

## Task 10: E2E Tests Reinforced

**Files:**
- Modify: `crates/daemon/tests/integration_e2e.rs` — add 4+ new scenarios

**What to build:** New E2E tests:
- `forge_validates_invalid_json` — POST post_tool_use with Write tool that wrote invalid JSON → verify response contains "SYNTAX ERROR"
- `token_tracking_nonzero` — full session → GET /heuristics/population → verify outcome has tokens > 0
- `session_budget_isolation` — two sessions in parallel → verify one session's budget doesn't affect the other
- `verify_stage_detects_test_failures` — POST stop with tool_response containing "3 passed; 2 failed" → verify response mentions test failures

**Commit:** `test: reinforced E2E harness — forge validation, token tracking, session isolation`

---

## Task 11: Docs & Version Bump

**Files:** CHANGELOG.md, .claude-plugin/plugin.json, memory-bank/

**Commit:** `docs: v0.11.0 Hardened Core — changelog, plugin, memory-bank`

---

## Task 12: Full Build & Test Verification

Run: `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Also: `cargo build --workspace --features mcp` to verify MCP feature compiles.

---

## Dependency Graph

```
Task 1 (tokens) ─────────────────────────┐
Task 2 (session profilers) ──────────────├──→ Task 10 (E2E tests)
Task 3 (verification→decide) ────────────┤
Task 4 (CLI truth) ── independent        │
Task 5 (Tier 1 forge) ──────────────────┤
Task 6 (Tier 2 forge) ── after Task 5   │
Task 7 (MCP fusion) ── independent       │
Task 8 (CLI update) ── after Task 7      │
Task 9 (verify upgrade) ── independent   ┘
Task 10 → Task 11 → Task 12
```

Parallelizable: Tasks 1+4+5+7+9 can run in parallel (different files). Tasks 2+3 after 1. Tasks 6+8 after 5+7.

## Summary

| Task | Chantier | Complexity |
|------|----------|------------|
| 1 | Token accounting P0 | Low |
| 2 | Session-scoped profilers P0 | High |
| 3 | Verification → decide P0 | Medium |
| 4 | CLI truth P0 | Low |
| 5 | Forge Tier 1 (serde) | Medium |
| 6 | Forge Tier 2 (sandbox) | Medium |
| 7 | MCP fusion into daemon | High |
| 8 | CLI + mcp-bridge update | Low |
| 9 | Verify stage upgrade | Medium |
| 10 | E2E tests | High |
| 11 | Docs | Low |
| 12 | Verification | Low |
