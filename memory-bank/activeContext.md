# Active Context

## Current State

v0.4.0 "Developer-First" is feature-complete. All six phases (Foundation,
Plugin Shell, Rust Runtime, Advanced Cognitive, Distribution, Developer-First)
are done. This release addresses the top 3 real-world Claude Code pain points:
false completion claims, test manipulation, and invisible token costs.

## v0.4.0 Completion Summary

### Developer-First Features (new in v0.4.0)
- **Completion Verifier**: catches false "Done!" claims by checking mentioned files exist on disk
- **Test Integrity Guard**: detects when Claude weakens test assertions instead of fixing code (asks for confirmation)
- **Token Budget Tracker**: visible `[budget: Ntok/$X.XX used | Y%]` in every hook response
- `GET /budget` endpoint for current session budget status
- `POST /budget/consume` endpoint for recording token consumption
- Budget warning at 80% utilization, over-budget detection

### Hook Changes in v0.4.0
- Stop hook now runs completion verification before accepting "Done!" responses
- PreToolUse hook now checks test file integrity on Edit/MultiEdit operations
- All hook responses now include budget summary in additionalContext
- UserPromptSubmit hook estimates and tracks token consumption

### Rust Runtime (7 crates, unchanged from v0.3.0)
- `metaygn-shared`: Protocol types, state enums, kernel
- `metaygn-core`: 12-stage control loop, topology planner, MASC monitor, heuristic evolver
- `metaygn-memory`: Episodic memory (FTS5), graph memory (nodes+edges+FTS5+cosine)
- `metaygn-verifiers`: Guard pipeline (5 guards), evidence packs (hash+Merkle+ed25519)
- `metaygn-sandbox`: Process sandbox (Python/Node/Bash, 5s timeout)
- `metaygn-daemon`: Axum HTTP server, forge engine, fatigue profiler, context pruner, budget endpoints
- `metaygn-cli`: CLI (start/stop/status/recall/top), Glass-Box TUI

### Plugin Shell
- 8 lifecycle hooks with daemon integration + local fallback
- 8 metacognitive skills (user-invocable workflows)
- 6 specialized agents (orchestrator + 5 delegates)
- Proof packet output style with evidence tagging
- TypeScript hook packages (@metaygn/hooks, @metaygn/shared)

## Key Architecture Decisions in v0.4.0
- Completion verification runs file-existence checks before allowing "Done!" claims
- Test integrity guard uses diff analysis to detect assertion weakening patterns
- Token budget tracking is per-session with configurable limits and cost-per-token
- All three features are lightweight local checks (no daemon required)

## Known Limitations
- Daemon `start` command does not yet spawn the daemon process (run `aletheiad` directly)
- Daemon `stop` command does not yet send a shutdown signal (use Ctrl+C)
- No WASM sandbox backend yet (process-based only)
- Heuristic evolver state is in-memory only (lost on daemon restart)
- Graph memory cosine search requires externally-generated embeddings
- No CI integration tests for daemon round-trips yet

## What Comes Next (v0.5.0+)
- MCP bridge crate for native MCP server integration
- WASM sandbox backend (feature-gated)
- Persistent heuristic state in SQLite
- Multi-agent orchestration improvements
- Session replay and debugging tools
- Marketplace publication
