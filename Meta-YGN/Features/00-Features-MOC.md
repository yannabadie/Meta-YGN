---
title: Features MOC
type: moc
tags:
  - features
  - moc
updated: 2026-04-07
---

# Features

## Confirmees (`[confirmed]`)

Testees end-to-end, shipped avec le daemon.

| Feature | Description | Detail |
|---------|-------------|--------|
| [[Guard-Pipeline-Feature\|Guard Pipeline]] | 5 guards, 28 regles, gate chaque tool call | [[Guard-Pipeline]] |
| [[Test-Integrity]] | Detecte affaiblissement d'assertions dans les tests | verifiers |
| [[Completion-Verifier]] | Verifie que les fichiers cites dans "Done!" existent | verifiers |
| [[Fatigue-Profiler]] | Track plasticite, escalade progressivement | daemon/profiler |
| [[Budget-Tracker]] | Tokens + cout par session, warning a 80% | shared |
| [[Calibration-Report]] | Brier score reel avec buckets de calibration | cli |
| [[Session-Replay]] | Timeline complete des hooks avec latence | cli + store |
| [[MCP-Bridge-Feature\|MCP Bridge]] | 5 outils via stdio server | daemon/mcp |
| [[Entropy-Calibration-EGPO]] | Calibration confiance via entropie | core/heuristics/entropy.rs |
| [[Plasticity-Detection-RL2F]] | Feedback implicite sur recovery | daemon/profiler/ |
| [[UCB-Memory-Retrieval]] | UCB exploration/exploitation sur recall | memory/graph.rs |
| [[Heuristic-Evolution]] | Population d'heuristiques avec mutation | core/heuristics/evolver.rs |
| [[Prompt-Injection-Guard]] | Detection patterns injection dans assess | core/stages/assess.rs |
| [[OTEL-Exporter]] | OTLP exporter feature-gated | daemon/telemetry.rs |
| [[Auto-Checkpoint]] | git stash/commit ref before destructive ops, file copies before rm | verifiers/checkpoint.rs |
| [[Bearer-Auth]] | UUID bearer token on all endpoints except /health | daemon/auth.rs |

## Experimentales (`[experimental]`)

Code reel, non validees a l'echelle.

| Feature | Description | Code | Paper |
|---------|-------------|------|-------|
| [[Dynamic-Topology]] | Selection Single/Vertical/Horizontal/Research | core/topology.rs | DyTopo |
| [[Neural-Embeddings]] | fastembed bge-small-en-v1.5, fallback hash | memory/embeddings.rs | — |
| [[RL-Trajectory-Export]] | Trajectoires signees JSONL pour offline RL | memory/trajectory.rs | RL2F |
| [[Sequence-Monitor]] | DTMC multi-action pattern detector, 3 rules, sliding window | core/sequence_monitor.rs | Pro2Guard |
| [[Semantic-Router]] | kNN risk classification, context-aware override | daemon/semantic_router.rs | RouteLLM |
| [[Haiku-Judge]] | Tier 3 LLM judge via Anthropic API, LRU cache, 20-call budget | daemon/judge.rs | LLM-as-judge |
| [[WASM-Sandbox]] | Tier 4 Wasmtime capability-based isolation, fuel-limited | sandbox/wasm.rs | — |
