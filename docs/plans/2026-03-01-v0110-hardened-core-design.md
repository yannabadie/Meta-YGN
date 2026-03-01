# v0.11.0 "Hardened Core" — Design Document

**Date**: 2026-03-01
**Status**: Approved

## Goal
Fix the P0 foundations identified by expert review, integrate the ForgeEngine into the verification pipeline, fuse the MCP bridge into the daemon, and upgrade the verify stage from keyword scanning to real verification. Every fix proven by E2E tests.

## Expert-Driven Priorities
Two independent expert reviews (March 2026) identified critical gaps:
- Token accounting broken (SessionOutcome.tokens_consumed always 0)
- Budget/Fatigue/Plasticity are global, not session-scoped (state contamination)
- Verification results don't feed into decision-making (logging only)
- CLI --host/--port declared but ignored (truth debt)
- MCP bridge as separate process creates IPC overhead (should be in daemon)

## Research Foundation
- Aider: tree-sitter AST linting + test loops (per-edit, synchronous)
- Codex: milestone-based plan/implement/validate/repair cycle
- OpenSage: dynamic tool synthesis with sandbox (2602.16891)
- SideQuest: semantic compaction for stale context (2602.22603)
- NVIDIA: sandbox security guidance (filesystem + network isolation)

## 5 Chantiers

### 1. P0 Fixes (Foundations)

**1a. Token accounting**: Wire `AppState.budget.consumed_tokens` into SessionContext at each hook. Pass real token count to SessionOutcome in after_stop(). Remove `tokens_consumed: 0 // TODO`.

**1b. Session-scoped profilers**: Move `SessionBudget`, `FatigueProfiler`, `PlasticityTracker` into `SessionContext`. Each session gets its own counters. AppState keeps global aggregates for /health endpoint.

**1c. Verification → Decide pipeline**: `calibrate` stage reads `ctx.verification_results` and lowers confidence proportionally to error count. `decide` stage already checks quality — now quality reflects real verification. Feed `verification_results` from SessionContext into LoopContext before running stages 8-12 in stop handler (partially done in v0.10.0, needs completing).

**1d. CLI --host/--port truth**: Remove `--host` and `--port` from the `Start` command (they are ignored). Document that the daemon always binds to `127.0.0.1:0` (dynamic port). If users need a fixed port, expose `METAYGN_PORT` env var. Less is more.

### 2. ForgeEngine Integration (Tiered Verification)

**Tier 1 — Sync, in-process, <100ms** (in post_tool_use handler):
- Extract `file_path` from `tool_input` on Write/Edit tools
- `.json` → `serde_json::from_str(&content)` — report parse errors
- `.yaml`/`.yml` → add `serde_yaml` dep, validate — report parse errors
- `.toml` → `toml::from_str(&content)` — report parse errors
- Return verification errors in HookOutput.additionalContext
- Requires: add `serde_yaml` and `toml` to daemon Cargo.toml

**Tier 2 — Async, forge sandbox, <2s** (in postprocess.rs after_post_tool_use):
- `.py` files → forge `syntax-checker` template (new: `ast.parse()`)
- Any code file → forge `file-exists-checker` template
- Results → `SessionContext.verification_results`
- Surfaced at next hook or in stop response

**New templates** (add to `crates/daemon/src/forge/templates.rs`):
- `syntax-checker` — Python ast.parse() on stdin content
- `yaml-validator` — Python yaml.safe_load() (backup for non-Rust path)
- `toml-validator` — Python tomllib.loads() (backup for non-Rust path)

**Security**: Reduce forge sandbox timeout to 2s for verification. Read file content in Rust, pipe via stdin — never pass file paths to sandbox scripts.

### 3. MCP Fusion into Daemon

Move the 5 MCP tools from `crates/mcp-bridge/src/handler.rs` into the daemon crate. The daemon gets a `--mcp` flag that switches from HTTP mode to MCP stdio mode.

**Architecture change:**
```
Before: aletheia-mcp (binary) --HTTP→ aletheiad (binary)
After:  aletheiad --mcp (single binary, stdio mode)
```

**Implementation:**
- Move `handler.rs` logic into `crates/daemon/src/mcp.rs`
- MCP tools call internal AppState methods directly (no HTTP round-trip)
- `aletheiad --mcp` starts rmcp stdio server instead of axum HTTP server
- `aletheiad` (no flag) starts HTTP server as before
- `aletheia mcp` CLI command launches `aletheiad --mcp`
- `crates/mcp-bridge/` crate becomes empty or is removed (redirect to daemon)

**Dependency change:** Add `rmcp = { version = "0.17", features = ["server", "transport-io"], optional = true }` to daemon Cargo.toml behind a `mcp` feature flag. Default build doesn't pull rmcp.

### 4. Verify Stage Upgrade

Replace keyword scanning in `crates/core/src/stages/verify.rs` with real verification:

**Read intended_action** (from act stage):
- If intent was "write file X" → check tool_response confirms file created
- If intent was "run command" → check exit code semantics

**Parse tool-specific responses**:
- Bash: look for test result summaries (`N passed; M failed`), extract counts
- Write/Edit: confirm file path in response
- MCP tools: check for error fields in JSON response

**Connect to calibrate**: verification error count already adjusts confidence (from 1c). This ensures real verification results affect decisions.

### 5. E2E Tests Reinforced

Add to existing 7 integration tests:
- `forge_validates_invalid_json` — Write a .json file with syntax error → verify hook returns error context
- `token_tracking_across_session` — Run full session → verify SessionOutcome.tokens_consumed > 0
- `cascading_errors_trigger_amplification` — 3 consecutive error post_tool_use → verify recovery amplification level increases
- `session_budget_isolation` — Two sessions simultaneously → verify budgets don't bleed
- `mcp_tools_work_directly` — Test MCP tools via internal call (if feasible in test harness)

## Evidence Tags
- Token accounting fix: `[confirmed]` — deterministic wiring
- Session-scoped profilers: `[confirmed]` — struct relocation
- Verification → decide: `[confirmed]` — pipeline connection
- Tier 1 forge (serde): `[confirmed]` — in-process, no external deps
- Tier 2 forge (sandbox): `[experimental]` — 2s timeout may be tight
- MCP fusion: `[confirmed]` — architectural simplification
- Verify stage upgrade: `[experimental]` — tool-specific parsing is heuristic
- New templates: `[confirmed]` — static, deterministic

## What NOT to do in v0.11.0
- NO tree-sitter dependency (defer to v0.12.0)
- NO dynamic template generation (LLM writing scripts)
- NO full cargo check / tsc in hot path
- NO WASM sandbox (process sandbox is sufficient for now)
- NO cross-session learning in competence stage (defer)
