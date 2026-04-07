---
title: Daemon aletheiad
type: architecture
tags:
  - architecture
  - daemon
updated: 2026-04-07
---

# Daemon aletheiad

**Crate** : `metaygn-daemon`
**Binary** : `aletheiad`
**Framework** : Axum
**Port** : dynamique, ecrit dans `~/.claude/aletheia/daemon.port`

## 16+ Endpoints HTTP

| Method | Path | Description |
|--------|------|-------------|
| POST | `/hooks/session-start` | Init session |
| POST | `/hooks/user-prompt-submit` | Classify + assess + strategize |
| POST | `/hooks/pre-tool-use` | Guard pipeline + risk |
| POST | `/hooks/post-tool-use` | Verify + fatigue + graph |
| POST | `/hooks/post-tool-use-failure` | Error recovery + plasticity |
| POST | `/hooks/pre-compact` | Pre-compaction |
| POST | `/hooks/stop` | Finalization (calibrate, compact, decide, learn) |
| POST | `/hooks/session-end` | Async persistence + evo trigger |
| GET | `/health` | Health check |
| GET | `/status` | Daemon status detaille |
| GET | `/memory/recall` | Recherche memoire |
| POST | `/trajectories/export` | Export RL trajectories JSONL |
| GET | `/calibration` | Rapport calibration (Brier score) |
| GET | `/replay/{session_id}` | Replay timeline session |
| GET | `/session/{id}/state` | Session state query |
| GET | `/budget/{id}` | Session budget |
| GET | `/metrics` | Prometheus metrics |
| GET | `/heuristics/population` | Heuristic population |
| GET | `/heuristics/best` | Best heuristic version |

## Lifecycle

```bash
aletheia start [--db-path PATH]    # Demarre le daemon (background)
aletheia stop                       # Arrete le daemon
aletheia status                     # Health check
aletheia doctor                     # Verification installation
```

## Architecture interne

1. `AppState` : etat partage (MemoryStore, GraphMemory, ControlLoop, SessionContext)
2. Chaque hook POST → deserialization `HookInput` → `LoopContext` → `ControlLoop.run_plan()`
3. Post-processing System 2 (async) : graph insertion, entropy tracking, evolution
4. Retour `HookOutput` avec decision + budget + guidance

## MCP mode

```bash
aletheia mcp    # Lance aletheiad --mcp (stdio server, rmcp 0.17)
```

Feature gate : `--features mcp`. Acces direct `AppState`, pas de hop HTTP.

## Telemetry

Feature gate : `--features otel`. OTLP exporter via tonic/gRPC.
Reads `OTEL_EXPORTER_OTLP_ENDPOINT` (default: `http://localhost:4317`).
Stage-level tracing spans auto-exported when enabled.

## Robustesse (v2.0)

Tous les mutex locks utilisent `if let Ok(guard)` avec `tracing::warn!` en cas de poisoning.
Zero `.expect()` ou `.unwrap()` sur mutex dans les hot paths.
