# Active Context

## Current State

v0.3.0 "Adaptive Topology" is feature-complete. All five phases (Foundation,
Plugin Shell, Rust Runtime, Advanced Cognitive, Distribution) are done. The
project is ready for marketplace publication and real-world testing.

## v0.3.0 Completion Summary

### Rust Runtime (7 crates)
- `metaygn-shared`: Protocol types, state enums, kernel
- `metaygn-core`: 12-stage control loop, topology planner, MASC monitor, heuristic evolver
- `metaygn-memory`: Episodic memory (FTS5), graph memory (nodes+edges+FTS5+cosine)
- `metaygn-verifiers`: Guard pipeline (5 guards), evidence packs (hash+Merkle+ed25519)
- `metaygn-sandbox`: Process sandbox (Python/Node/Bash, 5s timeout)
- `metaygn-daemon`: Axum HTTP server, forge engine, fatigue profiler, context pruner
- `metaygn-cli`: CLI (start/stop/status/recall/top), Glass-Box TUI

### Plugin Shell
- 8 lifecycle hooks with daemon integration + local fallback
- 8 metacognitive skills (user-invocable workflows)
- 6 specialized agents (orchestrator + 5 delegates)
- Proof packet output style with evidence tagging
- TypeScript hook packages (@metaygn/hooks, @metaygn/shared)

### Evaluation
- MetaCog-Bench: 15 scenarios across 5 families (Python)

### Distribution
- cargo-dist release pipeline (GitHub Actions)
- Plugin marketplace packaging (.claude-plugin/plugin.json)
- hook-runner.sh for cross-platform hook execution
- install.sh for daemon binary installation

## Key Architecture Decisions in v0.3.0
- Dynamic topology: Single (4 stages) / Vertical (12) / Horizontal (14)
- Statistical heuristic evolution (no LLM in the loop)
- 5-guard pipeline: destructive (deny), high_risk/secret_path/mcp (ask), default (allow)
- Three-layer evidence integrity: SHA-256 hash chain + Merkle tree + ed25519
- Human fatigue profiler with High-Friction mode at score >= 0.7
- TF-IDF cosine anomaly detection (MASC)

## Known Limitations
- Daemon `start` command does not yet spawn the daemon process (run `aletheiad` directly)
- Daemon `stop` command does not yet send a shutdown signal (use Ctrl+C)
- No WASM sandbox backend yet (process-based only)
- Heuristic evolver state is in-memory only (lost on daemon restart)
- Graph memory cosine search requires externally-generated embeddings
- No CI integration tests for daemon round-trips yet

## What Comes Next (v0.4.0+)
- MCP bridge crate for native MCP server integration
- WASM sandbox backend (feature-gated)
- Persistent heuristic state in SQLite
- Multi-agent orchestration improvements
- Session replay and debugging tools
- Marketplace publication
