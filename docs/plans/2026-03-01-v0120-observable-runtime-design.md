# v0.12.0 "Observable Runtime" — Design Document

**Date**: 2026-03-01
**Status**: Approved

## Goal
Make MetaYGN provably valuable and frictionlessly installable. Remove dead code, add industry-standard observability (OpenTelemetry + Prometheus), multi-language syntax verification (tree-sitter), adaptive competence from session history, a calibration evaluation harness, and marketplace-ready packaging.

## Theme
"Observable Runtime" — the system can prove its value through metrics, reports, and standard tooling.

## 6 Chantiers

### 1. Cleanup (Dead Code + Debt)
- Delete `crates/mcp-bridge/` entirely (redundant since v0.11.0 MCP fusion)
- Remove from workspace `Cargo.toml` members (8→7 crates)
- Fix Tier 2 forge trigger: check `tool_input.file_path` extension instead of `tool_response.contains(".py")`
- Remove 3 orphan fields from SessionContext (`tool_calls`, `success_count`, `tokens_consumed` write-only)
- Update `docs/daemon-contract.md` to v0.12.0
- Update `memory-bank/progress.md` to reflect v0.11.0 completion

### 2. OpenTelemetry Observability (feature: `otel`)
- Add `tracing-opentelemetry`, `opentelemetry`, `opentelemetry-otlp` behind `otel` feature flag
- Instrument each control loop stage as an OTel span:
  - Attributes: `metaygn.stage.name`, `metaygn.stage.duration_ms`, `metaygn.risk_level`, `metaygn.confidence`
  - Follow OTel GenAI agent span conventions where applicable
- Add `GET /metrics` Prometheus endpoint:
  - `metaygn_hook_duration_seconds` (histogram by hook type)
  - `metaygn_sessions_total` (counter)
  - `metaygn_escalations_total` (counter)
  - `metaygn_tokens_consumed_total` (counter)
  - `metaygn_verification_errors_total` (counter by error type)
- Feature-gated: `cargo build --features otel`. Default build has zero overhead.

### 3. Tree-sitter Multi-Language Verification (feature: `syntax`)
- Add `tree-sitter` + language grammars (`tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-typescript`) behind `syntax` feature flag
- Create `crates/verifiers/src/syntax.rs`: `check_syntax(content, language) -> Vec<SyntaxError>`
- Wire as Tier 1.5 in `post_tool_use` handler (after serde Tier 1, before forge Tier 2):
  - Detect language from file extension
  - Parse with tree-sitter, extract ERROR nodes
  - Report in `verification_results` and hook output
- Target: <10ms per file. In-process, no subprocess.
- Explicitly deferred from v0.11.0 design doc.

### 4. Adaptive Competence Stage
- Extend `CompetenceStage` to query historical `SessionOutcome` records
- Load past outcomes for current `TaskType` from SQLite (via a callback or AppState reference)
- Compute empirical success rate: `successes / total` from last 20 sessions
- Blend with base competence: `final = 0.5 * base + 0.5 * empirical` (fallback to base when <5 data points)
- Apply SSR insight: if strategy chosen for this task type has historically low success, lower competence further
- This requires making CompetenceStage aware of historical data — either pass success rates via LoopContext, or add a `historical_competence: Option<f32>` field set by the hook handler before running stages

### 5. Calibration Evaluation Harness
- New CLI command: `aletheia eval`
- Reads `session_outcomes` + `replay_events` tables from SQLite
- Computes:
  - **Brier score**: mean((confidence - outcome)²) across sessions — measures calibration quality
  - **Escalation precision**: fraction of escalations that were justified (tool_error or test_failure within 3 subsequent hooks)
  - **Overhead ROI**: average hook latency vs. error prevention rate
  - **Per-task-type accuracy**: breakdown by TaskType
- Output: JSON report + human-readable summary
- Target: useful with as few as 10 sessions of data

### 6. Marketplace Packaging
- New CLI command: `aletheia doctor` — checks daemon health, plugin structure, binary alignment, version consistency
- Polish `install.sh`: detect platform, download binary from GitHub releases, verify checksum
- Validate plugin with `claude plugin validate`
- Write marketplace-ready description, feature list, screenshots guide
- Target: submission-ready to `platform.claude.com/plugins/submit`

## Evidence Tags
- Cleanup: `[confirmed]` — direct code inspection
- OpenTelemetry: `[experimental]` — OTel GenAI conventions still in development
- Tree-sitter: `[confirmed]` — stable crate, Aider proves the pattern
- Adaptive competence: `[experimental]` — blending formula needs tuning
- Calibration harness: `[original-proposal]` — novel metric, no prior art
- Marketplace: `[confirmed]` — format requirements known

## What NOT to do in v0.12.0
- NO dialectic topology (teacher-student loop) — defer to v0.13.0
- NO LLM-driven heuristic mutation — conflicts with local-first
- NO WASM sandbox — process sandbox sufficient
- NO dynamic forge template generation — high risk
