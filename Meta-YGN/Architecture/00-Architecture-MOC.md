---
title: Architecture MOC
type: moc
tags:
  - architecture
  - moc
updated: 2026-04-07
---

# Architecture — Aletheia-Nexus

## Principe

> Plugin Claude Code fin → Daemon Rust intelligent.
> Le cerveau vit dans le runtime, pas dans le plugin.

## Composants

```
Claude Code
├── Skills (8 metacog workflows)
├── Agents (6 roles)
├── Output Styles (proof packets)
└── Hooks (8 lifecycle) ──HTTP──→ Aletheia Daemon (aletheiad)
                                    ├── 12-Stage Control Loop
                                    ├── Guard Pipeline (28 rules)
                                    ├── Graph Memory (FTS5, cosine)
                                    ├── Heuristic Evolver
                                    ├── Fatigue Profiler
                                    ├── MASC Anomaly Detector
                                    ├── Sequence Monitor
                                    ├── Tool Forge
                                    ├── Haiku Judge (Tier 3)
                                    ├── WASM Sandbox (Tier 4)
                                    └── SQLite (MemoryStore + GraphMemory)
```

## Pages

- [[Control-Loop|Boucle de controle 12 etapes]]
- [[Guard-Pipeline|Guard Pipeline — 5 guards, 28 regles]]
- [[Hook-Lifecycle|Hooks — 8 lifecycle events]]
- [[Daemon|Daemon aletheiad — HTTP API]]
- [[Memory-System|Systeme memoire — Graph + Store]]
- [[Plugin-Structure|Structure plugin Claude Code]]
- [[MCP-Bridge|Bridge MCP — 5 outils]]
- [[Topology-Planner|Topology Planner — Single/Vertical/Horizontal/Research]]

## 7 Crates Rust

| Crate | Responsabilite | LOC src |
|-------|---------------|---------|
| metaygn-shared | Types partages (HookInput/Output, Decision, Risk) | ~115 |
| metaygn-core | Control loop 12 stages, topology, heuristiques | ~800+ |
| metaygn-memory | Graph memory, SQLite store, FTS5, embeddings | ~1,200+ |
| metaygn-sandbox | Process sandbox, timeout, capture stdout/stderr | ~420 |
| metaygn-verifiers | Guard pipeline, test integrity, completion, evidence | ~800+ |
| metaygn-daemon | Axum HTTP API, 16+ endpoints, session management | ~1,800+ |
| metaygn-cli | Binary `aletheia`, TUI dashboard | ~892 |

## 2 Packages TypeScript

| Package | Contenu |
|---------|---------|
| packages/hooks | 8 hooks lifecycle, daemon-client, fallback heuristiques |
| packages/shared | types.ts (schemas Zod pour HookInput/HookOutput) |
