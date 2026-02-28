# Active Context

## Current State

v0.5.0 "Smart Recovery" is feature-complete. All seven phases (Foundation,
Plugin Shell, Rust Runtime, Advanced Cognitive, Distribution, Developer-First,
Smart Recovery) are done. This release adds plasticity-aware progressive
recovery, implicit feedback tracking, latency transparency, and fixes risk
classification for safe Bash commands.

## v0.5.0 Completion Summary

### Smart Recovery Features (new in v0.5.0)
- **Plasticity Tracker**: implicit feedback loop measuring recovery prompt effectiveness (success/failure tracking)
- **Progressive amplification**: 3-level recovery (standard → emphatic → escalation via /metacog-escalate)
- **Latency tracking**: every hook response includes `[latency: Nms]`
- **`aletheia init`**: CLI command for project onboarding (generates .claude/settings.json)
- Human-readable hook messages (Risk: HIGH | Strategy: verify-first | Budget: N tokens)

### Risk Classification Fixes in v0.5.0
- Pre-tool-use risk classification now evaluates the actual command text, not hook metadata
- `ls -la` correctly classified as LOW risk (was incorrectly HIGH)
- Safe Bash commands (ls, cat, cargo test, git status, etc.) now have proper risk levels

### Hook Changes in v0.5.0
- Stop hook now uses plasticity-aware amplification for recovery prompts
- PostToolUse hook records implicit feedback (success/failure) for pending recoveries

### Rust Runtime (7 crates, updated for v0.5.0)
- `metaygn-shared`: Protocol types, state enums, kernel, plasticity types
- `metaygn-core`: 12-stage control loop, topology planner, MASC monitor, heuristic evolver, plasticity tracker
- `metaygn-memory`: Episodic memory (FTS5), graph memory (nodes+edges+FTS5+cosine)
- `metaygn-verifiers`: Guard pipeline (5 guards), evidence packs (hash+Merkle+ed25519)
- `metaygn-sandbox`: Process sandbox (Python/Node/Bash, 5s timeout)
- `metaygn-daemon`: Axum HTTP server, forge engine, fatigue profiler, context pruner, budget endpoints, latency tracking
- `metaygn-cli`: CLI (start/stop/status/recall/top/init), Glass-Box TUI

### Plugin Shell
- 8 lifecycle hooks with daemon integration + local fallback
- 8 metacognitive skills (user-invocable workflows)
- 6 specialized agents (orchestrator + 5 delegates)
- Proof packet output style with evidence tagging
- TypeScript hook packages (@metaygn/hooks, @metaygn/shared)

## Key Architecture Decisions in v0.5.0
- Plasticity tracker uses exponential moving average (EMA) to score recovery effectiveness
- Progressive amplification escalates through 3 levels based on plasticity score thresholds
- Latency is measured per-hook and included in every response for transparency
- Risk classification evaluates the actual Bash command string, not hook metadata fields
- `aletheia init` generates a complete .claude/settings.json with sensible defaults

## Known Limitations
- Daemon `start` command does not yet spawn the daemon process (run `aletheiad` directly)
- Daemon `stop` command does not yet send a shutdown signal (use Ctrl+C)
- No WASM sandbox backend yet (process-based only)
- Heuristic evolver state is in-memory only (lost on daemon restart)
- Graph memory cosine search requires externally-generated embeddings
- No CI integration tests for daemon round-trips yet

## What Comes Next (v0.6.0+)
- MCP bridge crate for native MCP server integration
- WASM sandbox backend (feature-gated)
- Persistent heuristic state in SQLite
- Multi-agent orchestration improvements
- Session replay and debugging tools
- Marketplace publication
