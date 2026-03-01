# Active Context

## Current State

v0.7.0 "Deep Foundation" is feature-complete. All nine phases (Foundation,
Plugin Shell, Rust Runtime, Advanced Cognitive, Distribution, Developer-First,
Smart Recovery, Solid Ground, Deep Foundation) are done. This release delivers
typed events, unified FTS search, context pruning service, embedding provider
trait, skill crystallizer, and cross-session learning wire-up.

## v0.7.0 Completion Summary

### Deep Foundation Features (new in v0.7.0)
- **Typed event system**: 11 `MetaEvent` variants replacing ad-hoc string logging
- **Unified FTS search**: single query across events and graph nodes
- **Context pruning service**: `POST /proxy/anthropic` analyzes and prunes error loops from message payloads
- **Embedding provider trait**: pluggable `EmbeddingProvider` with hash-based and no-op implementations
- **Skill crystallizer**: auto-detects recurring tool patterns and generates SKILL.md templates
- **Cross-session learning**: daemon loads heuristic versions and outcomes from SQLite at startup

### Implementation Changes in v0.7.0
- `act` stage now records intended actions for post-verification comparison
- `compact` stage generates real summaries and deduplicates lessons (was no-op)
- Heuristic mutations and outcomes are persisted to SQLite after every change
- All 4 stub modules (events, fts, act, compact) are now fully implemented
- `HeuristicEvolver.restore_version()` loads persisted versions at startup
- API handlers persist outcomes and evolved versions to SQLite after every mutation

### Rust Runtime (7 crates, updated for v0.7.0)
- `metaygn-shared`: Protocol types, state enums, kernel, plasticity types, typed events
- `metaygn-core`: 12-stage control loop (all stages implemented), topology planner, MASC monitor, heuristic evolver
- `metaygn-memory`: Episodic memory (FTS5), graph memory (nodes+edges+FTS5+cosine), persistent heuristics, unified FTS, embedding providers, skill crystallizer
- `metaygn-verifiers`: Guard pipeline (5 guards), evidence packs (hash+Merkle+ed25519)
- `metaygn-sandbox`: Process sandbox (Python/Node/Bash, 5s timeout)
- `metaygn-daemon`: Axum HTTP server, forge engine, fatigue profiler, context pruner, budget endpoints, latency tracking, graceful shutdown, cross-session learning
- `metaygn-cli`: CLI (start/stop/status/recall/top/init), Glass-Box TUI, daemon spawning

### Plugin Shell
- 8 lifecycle hooks with daemon integration + local fallback (TypeScript via Bun)
- 8 metacognitive skills (user-invocable workflows)
- 6 specialized agents (orchestrator + 5 delegates)
- Proof packet output style with evidence tagging
- TypeScript hook packages (@metaygn/hooks, @metaygn/shared)

## Key Architecture Decisions in v0.7.0
- All 4 previously-stub modules (events, fts, act, compact) are fully implemented
- Typed event variants replace string-based logging for type safety and pattern matching
- Unified FTS merges event and graph node search into a single query interface
- Context pruning service detects 3+ consecutive error messages and prunes them with recovery injection
- Embedding provider trait allows swapping in real embedding models later (hash-based for now)
- Skill crystallizer observes tool sequences and auto-generates SKILL.md templates at 3+ repetitions
- Cross-session learning loads heuristic population and outcomes from SQLite at daemon startup
- API handlers persist mutations to SQLite after every evolve/record_outcome call

## Remaining Limitations
- No WASM sandbox backend yet (process-based only)
- Graph memory cosine search uses hash-based embeddings (no neural model yet)
- No MCP bridge crate yet

## What Comes Next (v0.8.0+)
- MCP bridge crate for native MCP server integration
- WASM sandbox backend (feature-gated)
- Neural embedding provider (e.g., via ONNX Runtime)
- Multi-agent orchestration improvements
- Session replay and debugging tools
- Marketplace publication
