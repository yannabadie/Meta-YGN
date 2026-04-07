# Aletheia-Nexus

> v2.0.0 "Production Hardened" — [Release Notes](https://github.com/yannabadie/Meta-YGN/releases/tag/v2.0.0)

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
- **Calibration Report** -- `aletheia eval` computes real Brier score with calibration buckets from session outcomes
- **Session Replay** -- `aletheia replay` shows the full hook timeline for any past session
- **MCP Bridge** -- 5 tools (`metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`) exposed as an MCP stdio server
- **Entropy Calibration (EGPO)** -- calibrates confidence using entropy-guided policy optimization (tested: `entropy_test.rs`)
- **Plasticity Detection (RL2F)** -- implicit feedback loop that adjusts recovery amplification based on whether errors recur (tested: `plasticity_test.rs`)
- **UCB Memory Retrieval** -- upper-confidence-bound scoring for graph memory recall ranking (tested: `ucb_test.rs`)
- **Heuristic Evolution** -- Layer-0 fitness-scored heuristic mutation and selection (tested: `heuristics_test.rs`)
- **OpenTelemetry** -- stage-level tracing spans wired to any OTLP collector via `--features otel`

## Experimental Features

These are implemented but not yet validated at scale. Treat claims about their effectiveness as hypotheses, not facts.

- `[experimental]` **Dynamic Topology** -- TopologyPlanner selects Single/Vertical/Horizontal execution topology per task
- `[experimental]` **Neural Embeddings** -- real embedding providers behind a feature gate (`fastembed`); hash-based fallback is the default
- `[experimental]` **RL Trajectory Export** -- `aletheia export` writes JSONL trajectories for offline RL training

## CLI Commands

```
aletheia start [--db-path PATH]                         Start the daemon
aletheia stop                                            Stop the daemon
aletheia status                                          Show daemon health
aletheia recall --query Q [--limit N]                    Search memory
aletheia eval                                            Show calibration report (Brier score)
aletheia doctor                                          Check installation health
aletheia top                                             Real-time TUI dashboard
aletheia init [--force]                                  Scaffold .claude/ config
aletheia mcp                                             Launch MCP stdio server
aletheia replay [SESSION_ID]                             Replay session hook timeline
aletheia export [--limit N]                              Export RL trajectories to JSONL
```

## Installation

### Prerequisites
- **Rust** 1.85+ (`rustup update`)
- **Node.js** 22+ with npm (`node --version`) — hooks run via `npx tsx`
- **pnpm** 9+ (`npm install -g pnpm`) — for TypeScript workspace dependencies

### Setup
```bash
git clone https://github.com/yannabadie/Meta-YGN && cd Meta-YGN
cargo build --workspace          # build daemon + CLI (7 crates)
pnpm install                     # install TypeScript hook dependencies
```

### Running with Claude Code
```bash
aletheia start                   # start the daemon (background, auto port)
claude --plugin-dir .            # run Claude Code with the plugin
```

If `aletheia` is not on your PATH yet, use:
`cargo run -p metaygn-cli -- start`

The daemon listens on a dynamic port (written to `~/.claude/aletheia/daemon.port`).
Without the daemon, hooks fall back to lightweight local heuristics.

### Running with Codex CLI (MCP)
```bash
# Windows PowerShell (one-shot installer)
powershell -ExecutionPolicy Bypass -File .\scripts\install-codex.ps1

# macOS/Linux (one-shot installer)
bash ./scripts/install-codex.sh

# Manual setup (if preferred):
# cargo build -p metaygn-daemon --features mcp
# codex mcp add aletheia -- "$PWD\\target\\debug\\aletheia.exe" mcp
```

For a guided Codex session with MetaYGN protocol preloaded:

```bash
# Windows
powershell -ExecutionPolicy Bypass -File .\scripts\start-codex-metaygn.ps1

# macOS/Linux
bash ./scripts/start-codex-metaygn.sh

# Print the bootstrap prompt without launching Codex
powershell -ExecutionPolicy Bypass -File .\scripts\start-codex-metaygn.ps1 -NoLaunch
```

This launcher enables a strict verification gate before final answers.

Then launch Codex in this repo normally (`codex`).
MetaYGN tools (`metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`) are available through MCP.
Note: Codex currently integrates through MCP tools (explicit calls), not Claude-style automatic lifecycle hooks.

### Validation
```bash
aletheia status                  # check daemon health
aletheia doctor                  # check plugin + hooks + agents + DB
claude plugin validate .         # validate plugin structure
```

### Optional features
```bash
cargo build --features mcp        # MCP stdio server (aletheiad --mcp)
cargo build --features syntax     # tree-sitter multi-language verification
cargo build --features otel       # OpenTelemetry tracing spans
cargo build --features embeddings # Neural embeddings (fastembed, bge-small-en-v1.5)
```

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
