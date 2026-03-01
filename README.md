# Aletheia-Nexus

A local-first metacognitive control plane for coding agents.

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         Claude Code                               │
│                                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐               │
│  │  Skills   │  │  Agents  │  │  Output Styles   │               │
│  │ (8 metacog│  │ (6 roles)│  │ (proof packets)  │               │
│  │ workflows)│  │          │  │                  │               │
│  └─────┬─────┘  └────┬─────┘  └────────┬─────────┘               │
│        │              │                 │                         │
│  ┌─────┴──────────────┴─────────────────┴─────────┐               │
│  │              Hooks (8 lifecycle events)          │               │
│  └─────────────────────┬───────────────────────────┘               │
│                        │ HTTP                                     │
│  ┌─────────────────────┴───────────────────────────────────────┐  │
│  │              Aletheia Daemon (aletheiad)                      │  │
│  │                                                              │  │
│  │  ┌──────────────────────────────────────────────────────┐    │  │
│  │  │  12-Stage Control Loop (classify..learn)             │    │  │
│  │  │  + TopologyPlanner (Single/Vertical/Horizontal)      │    │  │
│  │  └──────────────────────────────────────────────────────┘    │  │
│  │                                                              │  │
│  │  ┌───────────────┐ ┌─────────────┐ ┌────────────────────┐   │  │
│  │  │ Guard Pipeline │ │ MASC Anomaly│ │ Fatigue Profiler   │   │  │
│  │  │ (security gate)│ │ Detector    │ │ (inverse metacog)  │   │  │
│  │  └───────────────┘ └─────────────┘ └────────────────────┘   │  │
│  │                                                              │  │
│  │  ┌───────────────┐ ┌─────────────┐ ┌────────────────────┐   │  │
│  │  │ Graph Memory   │ │ Heuristic   │ │ Tool Forge         │   │  │
│  │  │ (nodes, edges, │ │ Evolver     │ │ (templates, cache, │   │  │
│  │  │  FTS5, cosine) │ │ (Layer 0)   │ │  sandbox execute)  │   │  │
│  │  └───────┬────────┘ └──────┬──────┘ └─────────┬──────────┘   │  │
│  │          │                 │                   │              │  │
│  │  ┌───────┴─────────────────┴───────────────────┴──────────┐  │  │
│  │  │  SQLite (MemoryStore + GraphMemory) │ ProcessSandbox   │  │  │
│  │  └────────────────────────────────────┴───────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  eval/ MetaCog-Bench (15 scenarios, 5 families)              │  │
│  └──────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## What Works Today

These features are tested end-to-end and ship with the daemon:

- **Guard Pipeline** -- 5 guards with 28 pattern rules (destructive, high-risk, secret-path, MCP, default) that gate every tool call
- **Test Integrity Checker** -- detects when tests are weakened (assertions removed, expected values changed) and asks for confirmation
- **Completion Verifier** -- validates that files mentioned in "Done!" claims actually exist before allowing the claim through
- **Fatigue Profiler** -- tracks error recovery plasticity and escalates progressively (hint, critique, auto-escalate)
- **Budget Tracker** -- per-session token and cost tracking with 80% utilization warnings, shown on every hook response
- **Session Replay** -- `aletheia replay` shows the full hook timeline for any past session
- **MCP Bridge** -- 5 tools (`metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`) exposed as an MCP stdio server

## Experimental Features

These are implemented but not yet validated at scale. Treat claims about their effectiveness as hypotheses, not facts.

- `[experimental]` **Entropy Calibration (EGPO)** -- calibrates confidence using entropy-guided policy optimization
- `[experimental]` **Plasticity Detection (RL2F)** -- implicit feedback loop that adjusts recovery amplification based on whether errors recur
- `[experimental]` **UCB Memory Retrieval** -- upper-confidence-bound scoring for graph memory recall ranking
- `[experimental]` **Heuristic Evolution** -- Layer-0 fitness-scored heuristic mutation and selection
- `[experimental]` **Dynamic Topology** -- TopologyPlanner selects Single/Vertical/Horizontal execution topology per task
- `[experimental]` **Neural Embeddings** -- real embedding providers behind a feature gate (`fastembed`); hash-based fallback is the default
- `[experimental]` **RL Trajectory Export** -- `aletheia export` writes JSONL trajectories for offline RL training

## CLI Commands

```
aletheia start [--host H] [--port P] [--db-path PATH]   Start the daemon
aletheia stop                                            Stop the daemon
aletheia status                                          Show daemon health
aletheia recall --query Q [--limit N]                    Search memory
aletheia top                                             Real-time TUI dashboard
aletheia init [--force]                                  Scaffold .claude/ config
aletheia mcp                                             Launch MCP stdio server
aletheia replay [SESSION_ID]                             Replay session hook timeline
aletheia export [--limit N]                              Export RL trajectories to JSONL
```

## Installation

### Local development
```bash
cargo build --workspace          # build daemon + CLI + MCP bridge
claude --plugin-dir .            # run Claude Code with the plugin
```

### Validation
```bash
claude plugin validate
claude --debug --plugin-dir .
```

### Wiring the daemon
```bash
export ALETHEIA_DAEMON_URL=http://localhost:9000
aletheia start
aletheia status
```
Without the daemon, hooks fall back to lightweight local heuristics.

## Research Foundation

The experimental features draw on ideas from these papers (none are fully replicated; see `[experimental]` tags above):

| Abbreviation | Paper | Relevance |
|---|---|---|
| EGPO | Entropy-Guided Policy Optimization | Confidence calibration via entropy |
| RL2F | Reinforcement Learning from LLM Feedback | Implicit feedback for recovery |
| U-Mem | Uncertainty-Aware Memory | UCB-scored retrieval ranking |
| OpenSage | Open-Source Sage Agent | Multi-agent topology patterns |
| DyTopo | Dynamic Topology Planning | Adaptive execution topology |
| SideQuest | SideQuest Exploration | Heuristic evolution fitness |
| AlphaEvolve | AlphaEvolve | Evolutionary program search |

## License

MIT
