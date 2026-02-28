# Active Context

## Current State

v0.6.0 "Solid Ground" is feature-complete. All eight phases (Foundation,
Plugin Shell, Rust Runtime, Advanced Cognitive, Distribution, Developer-First,
Smart Recovery, Solid Ground) are done. This release delivers complete TypeScript
hooks running via Bun, daemon graceful lifecycle (start/stop), persistent
heuristics in SQLite, CI integration tests, and plugin validation.

## v0.6.0 Completion Summary

### Solid Ground Features (new in v0.6.0)
- **Complete TypeScript hooks**: all 8 hooks (session-start, user-prompt-submit, pre-tool-use, post-tool-use, post-tool-use-failure, stop, pre-compact, session-end) implemented in TypeScript with Bun runtime
- **Daemon graceful lifecycle**: `POST /admin/shutdown` endpoint triggers clean shutdown with port file cleanup
- **`aletheia start`**: spawns daemon as detached process, polls for port file, health checks
- **`aletheia stop`**: sends shutdown request, waits for clean termination
- **Persistent heuristics**: HeuristicVersion and SessionOutcome stored in SQLite, survive daemon restart
- **CI integration tests**: GitHub Actions job that starts daemon, tests hooks, verifies budget, shuts down
- **Plugin validation script**: `scripts/validate-plugin.sh` checks all 26 plugin components
- Local fallback functions for user-prompt-submit, post-tool-use, and stop hooks

### Hook Pipeline Changes in v0.6.0
- `hooks/hooks.json` now uses `bun run` for all hooks (was `bash hook-runner.sh`)
- All hooks have 350ms timeout with local fallback
- Daemon supports dual shutdown triggers: Ctrl+C and `/admin/shutdown`
- CLI `start` finds `aletheiad` binary automatically next to the CLI executable

### Rust Runtime (7 crates, updated for v0.6.0)
- `metaygn-shared`: Protocol types, state enums, kernel, plasticity types
- `metaygn-core`: 12-stage control loop, topology planner, MASC monitor, heuristic evolver, plasticity tracker
- `metaygn-memory`: Episodic memory (FTS5), graph memory (nodes+edges+FTS5+cosine), persistent heuristic storage
- `metaygn-verifiers`: Guard pipeline (5 guards), evidence packs (hash+Merkle+ed25519)
- `metaygn-sandbox`: Process sandbox (Python/Node/Bash, 5s timeout)
- `metaygn-daemon`: Axum HTTP server, forge engine, fatigue profiler, context pruner, budget endpoints, latency tracking, graceful shutdown
- `metaygn-cli`: CLI (start/stop/status/recall/top/init), Glass-Box TUI, daemon spawning

### Plugin Shell
- 8 lifecycle hooks with daemon integration + local fallback (TypeScript via Bun)
- 8 metacognitive skills (user-invocable workflows)
- 6 specialized agents (orchestrator + 5 delegates)
- Proof packet output style with evidence tagging
- TypeScript hook packages (@metaygn/hooks, @metaygn/shared)

## Key Architecture Decisions in v0.6.0
- All hooks migrated from `bash hook-runner.sh` to `bun run` TypeScript execution
- Daemon shutdown via both SIGINT (Ctrl+C) and HTTP `/admin/shutdown` endpoint
- Port file written on startup, cleaned up on shutdown for daemon discovery
- CLI `start` spawns daemon as detached child, polls port file, then health-checks
- Heuristic versions and session outcomes persisted to SQLite for cross-restart learning
- CI pipeline starts daemon, runs hook round-trips, verifies budget, shuts down cleanly

## Resolved Limitations (fixed in v0.6.0)
- Daemon `start` command now spawns the daemon process (was: run `aletheiad` directly)
- Daemon `stop` command now sends shutdown signal (was: use Ctrl+C only)
- Heuristic evolver state now persisted in SQLite (was: in-memory only, lost on restart)
- CI integration tests now exist for daemon round-trips

## Remaining Limitations
- No WASM sandbox backend yet (process-based only)
- Graph memory cosine search requires externally-generated embeddings
- No MCP bridge crate yet

## What Comes Next (v0.7.0+)
- MCP bridge crate for native MCP server integration
- WASM sandbox backend (feature-gated)
- Multi-agent orchestration improvements
- Session replay and debugging tools
- Marketplace publication
