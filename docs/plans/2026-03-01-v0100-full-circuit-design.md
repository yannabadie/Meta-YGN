# v0.10.0 "Full Circuit" — Design Document

**Date**: 2026-03-01
**Status**: Approved

## Goal
Connect all dormant code into the live execution path. Every component built in v0.1.0–v0.9.0 must either work end-to-end or be explicitly marked `[experimental]`. Adopt the System 1 / System 2 model: hooks stay fast (System 1), heavy processing runs async (System 2).

## Guiding Principles (from expert review)
- **Hooks are fast telemetry + guards, not analysis engines.** Response in <50ms.
- **Heavy work is async post-processing.** Graph population, sandbox checks, heuristic evolution run in background `tokio::spawn` tasks after the hook returns.
- **Measure before claiming.** A 20-scenario eval harness validates every integration.
- **Honest labeling.** README reflects reality. Experimental features tagged `[experimental]`.

## Research Foundation
- System 1/System 2 (Kahneman) — fast intuition + slow deliberation
- DyTopo (arXiv:2602.06039) — dynamic topology for agent pipelines
- OpenSage (arXiv:2602.16891) — sandbox + tool synthesis + graph memory
- SideQuest (arXiv:2602.22603) — semantic compaction of stale context
- RL2F (Klissarov et al., 2026) — plasticity loss requires compaction to be effective

## 8 Chantiers

### 1. SessionContext Persistant
New `SessionContext` struct in `crates/daemon/src/session.rs`. Stored in `AppState` as `HashMap<String, Arc<Mutex<SessionContext>>>`. Carries accumulated task_type, risk, strategy, entropy_tracker, metacog_vector, verification_results, lessons, tool_calls count, execution_plan across all hooks of a session. Created at `user_prompt_submit`, updated by every subsequent hook, finalized at `stop`. TTL cleanup for abandoned sessions.

### 2. Graph Auto-Population (Async, System 2)
After each hook handler returns its response, a `tokio::spawn` background task inserts MemoryNodes into GraphMemory:
- `user_prompt_submit` → Task node (type, risk, strategy)
- `pre_tool_use` → Tool node (name, guard decision)
- `post_tool_use` → Evidence node (verification result, error/success)
- `stop` → Decision node + Lesson nodes + edges connecting them

Non-blocking: the HTTP response is already sent. Graph failures are logged but don't affect hook behavior.

### 3. Heuristic Wire-up + Topology run_plan()
**Read path**: When `user_prompt_submit` creates a SessionContext, it loads `evolver.best()` and injects `risk_weights` + `strategy_scores` as bias into the classify/strategy stages (via SessionContext fields).
**Write path**: At `stop`, construct a `SessionOutcome` from the SessionContext and call `evolver.record_outcome()` + `evolver.evaluate_all()`. Persist to SQLite.
**Topology**: `user_prompt_submit` generates `ExecutionPlan` via `TopologyPlanner::plan()` and stores it in `SessionContext.execution_plan`. The `stop` hook uses `run_plan()` instead of `run_range(8,12)`. Single topology tasks skip verify/calibrate. Horizontal tasks double-verify.

### 4. Entropy + Plasticity in Decide Stage
**Entropy**: `post_tool_use` async processing calls `session.entropy_tracker.record(confidence, was_correct)`. The `decide` stage (stage 11) checks `entropy_tracker.is_overconfident()` → force `Decision::Revise` with a warning message.
**Plasticity**: `decide` stage checks `plasticity_tracker.is_plasticity_lost()` → force `Decision::Escalate`. The plasticity_level is included in the stop hook output.

### 5. Sandbox Async in post_tool_use (Non-Blocking)
After `post_tool_use` returns, a `tokio::spawn` task runs sandbox validation:
- `Write`/`Edit` of `.py` files → `python -c "import ast; ast.parse(open('file').read())"`
- `Write`/`Edit` of `.js`/`.ts` files → `node --check file`
- `Bash` tool with test output → check exit code coherence
Timeout: 2s. Results written to `SessionContext.verification_results` for the next hook to read. ForgeEngine deferred to v0.11.0.

### 6. CompactStage Rewritten
Stage 10 becomes real semantic compaction:
- **Lesson clustering**: hash-based word overlap (>50% shared words → merge with count suffix). Cap at 10 unique clusters.
- **Verification aging**: each result gets an `age` counter incremented per loop iteration. Evict results aged >2.
- **Graph promotion**: verification results with no errors + confidence >0.7 → schedule a `MemoryNode` write (via channel/callback to daemon, since stages are sync).

### 7. Fix Contrat Rust/TS + README Honesty
- Add `source`, `reason`, `trigger` fields to TS `HookInputSchema` in `packages/shared/src/types.ts`
- Rewrite README.md to reflect what actually works E2E vs what's `[experimental]`
- Plugin description: "Local-first metacognitive control plane for coding agents" (not "runtime")
- Tag all OpenSage-derived features as `[experimental]` in docs

### 8. Harness d'Évaluation (20 Scénarios)
New `eval/integration/` directory with 20 scenarios across 4 families:
- **Safety** (5): destructive commands blocked, high-risk asked, secrets detected, MCP gated, safe commands allowed
- **Classification** (5): bugfix detected, security escalated, refactor low-risk, research identified, architecture high-difficulty
- **Memory** (5): graph populated after session, FTS recall works, semantic search finds relevant nodes, replay records all hooks, trajectory exported
- **Calibration** (5): entropy tracker detects overconfidence after N errors, plasticity lost triggers escalate, heuristic fitness improves over sessions, budget warning at 80%, session context persists across hooks

Each scenario: start daemon in-memory, POST hooks in sequence, assert state. Runnable with `cargo test --test integration`.

## Evidence Tags
- SessionContext: `[confirmed]` — deterministic, testable
- Graph auto-population: `[experimental]` — async, may miss writes under load
- Heuristic wire-up: `[experimental]` — needs 20+ sessions to converge
- Topology run_plan(): `[confirmed]` — already tested, just needs wiring
- Entropy/Plasticity in decide: `[experimental]` — thresholds need tuning
- Sandbox async: `[experimental]` — 2s timeout may miss slow validations
- CompactStage: `[experimental]` — clustering heuristic needs real-world validation
- Rust/TS contract fix: `[confirmed]` — type alignment, no ambiguity
- Eval harness: `[confirmed]` — deterministic test scenarios
