# Active Context

## Current State

v0.8.0 "Neural Bridge" is feature-complete. All ten phases (Foundation,
Plugin Shell, Rust Runtime, Advanced Cognitive, Distribution, Developer-First,
Smart Recovery, Solid Ground, Deep Foundation, Neural Bridge) are done. This
release delivers MCP bridge, neural embeddings, and session replay.

## v0.8.0 Completion Summary

### MCP Bridge (new in v0.8.0)
- New crate `metaygn-mcp-bridge` with 5 metacognitive MCP tools via rmcp 0.17 stdio transport
- Tools: `metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`
- `DaemonClient` HTTP bridge to existing daemon API (5s timeout)
- CLI command `aletheia mcp` launches the MCP stdio server

### Neural Embeddings (new in v0.8.0)
- `FastEmbedProvider` implementing `EmbeddingProvider` trait (bge-small-en-v1.5, 384 dim)
- Feature-gated behind `--features embeddings` — zero overhead when disabled
- `GraphMemory.semantic_search()` for cosine-similarity vector search over stored node embeddings
- `POST /memory/semantic` daemon endpoint for vector-based node retrieval
- `HashEmbedProvider` remains the default (no external dependency)

### Session Replay (new in v0.8.0)
- `replay_events` SQLite table recording all hook calls with request/response/latency
- `GET /replay/sessions` and `GET /replay/{session_id}` daemon API endpoints
- `aletheia replay` CLI command: list sessions or view hook timeline
- All 5 hook handlers record replay events automatically

### Rust Runtime (8 crates, updated for v0.8.0)
- `metaygn-shared`: Protocol types, state enums, kernel, typed events
- `metaygn-core`: 12-stage control loop, topology planner, MASC monitor, heuristic evolver
- `metaygn-memory`: Episodic + graph memory, FTS5, embedding providers (hash + fastembed), skill crystallizer, session replay
- `metaygn-verifiers`: Guard pipeline (5 guards), evidence packs
- `metaygn-sandbox`: Process sandbox (Python/Node/Bash)
- `metaygn-daemon`: Axum HTTP server, forge engine, fatigue profiler, context pruner, semantic search, replay API
- `metaygn-cli`: CLI (start/stop/status/recall/top/init/mcp/replay), Glass-Box TUI
- `metaygn-mcp-bridge`: MCP stdio server with 5 metacognitive tools (rmcp 0.17)

## Remaining Limitations
- No WASM sandbox backend yet (process-based only)
- Semantic search uses linear scan (no ANN index for large node counts)
- fastembed requires ~30MB model download on first use

## What Comes Next (v0.9.0+)
- WASM sandbox backend (feature-gated)
- Multi-agent orchestration improvements
- A2A protocol adapter
- Marketplace publication
