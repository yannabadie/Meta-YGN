# Aletheia-Nexus

AI coding agents are confident even when wrong. They execute destructive commands without hesitation. They say "Done!" when files are missing.

Aletheia-Nexus is a local daemon that watches every agent action and intervenes -- a cognitive immune system for AI-assisted development.

![version](https://img.shields.io/badge/version-2.5.0-blue)

## What It Does

**Scenario 1: Dangerous command blocked**
```
You: "Clean up the project"
Claude tries: rm -rf /
  Aletheia: DENY -- "Destructive pattern detected: rm -rf /"
```

**Scenario 2: Risk classification**
```
You: "Fix the authentication bug"
  Aletheia: Risk: LOW | Strategy: adversarial | Task: security | Topology: horizontal
            Budget: 1000 tokens allocated
```

**Scenario 3: Completion verification**
```
Claude: "Done! I created auth.rs and tests.rs"
  Aletheia: COMPLETION CHECK FAILED -- auth.rs does not exist
            "Claude claimed completion but verification found issues"
```

## Quick Start

```bash
git clone https://github.com/yannabadie/Meta-YGN && cd Meta-YGN
cargo build --workspace && pnpm install
cargo run -p metaygn-cli -- start    # start the daemon
claude --plugin-dir .                 # Claude Code with Aletheia protection
```

To install `aletheia` on PATH: `cargo install --path crates/cli`

Then run `aletheia doctor` to verify everything works.

## How It Works

1. Claude Code hooks fire on every lifecycle event (tool use, completion, errors) and send HTTP requests to the Aletheia daemon.
2. The daemon runs a 12-stage control loop: classify risk, assess context, verify claims, and decide.
3. It returns a verdict -- allow / deny / ask / escalate -- plus a token budget and guidance.

```
Claude Code --hooks--> Aletheia Daemon --> Decision
                         |-- Guard Pipeline (35+ security rules)
                         |-- Control Loop (12 stages)
                         |-- Graph Memory (SQLite + FTS5)
                         +-- Heuristic Evolution (self-tuning)
```

## Features

### Confirmed (tested end-to-end)

| Feature | What it does |
|---|---|
| Guard Pipeline | 5 guards, 28 pattern rules gating every tool call |
| Test Integrity Checker | Detects weakened tests (removed assertions, changed expected values) |
| Completion Verifier | Validates files mentioned in "Done!" claims actually exist |
| Fatigue Profiler | Tracks error recovery plasticity, escalates progressively |
| Budget Tracker | Per-session token/cost tracking with 80% utilization warnings |
| Calibration Report | Real Brier score with calibration buckets from session outcomes |
| Session Replay | Full hook timeline for any past session |
| MCP Bridge | 5 tools exposed as an MCP stdio server |
| Entropy Calibration (EGPO) | Confidence calibration via entropy-guided policy optimization |
| Plasticity Detection (RL2F) | Implicit feedback loop adjusting recovery amplification |
| UCB Memory Retrieval | Upper-confidence-bound scoring for recall ranking |
| Heuristic Evolution | Layer-0 fitness-scored heuristic mutation and selection |
| OpenTelemetry | Stage-level tracing spans via `--features otel` |

### Experimental (implemented, not yet validated at scale)

| Feature | What it does |
|---|---|
| Dynamic Topology | Selects Single/Vertical/Horizontal execution topology per task |
| Neural Embeddings | Real embedding providers behind `fastembed` feature gate |
| RL Trajectory Export | JSONL trajectories for offline RL training |

## CLI Commands

```
aletheia start [--db-path PATH]       Start the daemon
aletheia stop                         Stop the daemon
aletheia status                       Show daemon health
aletheia recall --query Q [--limit N] Search memory
aletheia eval                         Show calibration report (Brier score)
aletheia doctor                       Check installation health
aletheia top                          Real-time TUI dashboard
aletheia init [--force]               Scaffold .claude/ config
aletheia mcp                          Launch MCP stdio server
aletheia replay [SESSION_ID]          Replay session hook timeline
aletheia export [--limit N]           Export RL trajectories to JSONL
```

## Installation

### Prerequisites

- **Rust** 1.85+ (`rustup update`)
- **Node.js** 22+ (`node --version`) — hooks are pre-compiled, no tsx needed
- **pnpm** 9+ (`npm install -g pnpm`) -- for TypeScript workspace dependencies

### Setup

```bash
git clone https://github.com/yannabadie/Meta-YGN && cd Meta-YGN
cargo build --workspace          # builds daemon + CLI (7 crates)
# Expected: Compiling metaygn-core, metaygn-daemon, metaygn-cli ...
pnpm install                     # install TypeScript hook dependencies
# Expected: Packages: +N, done in Xs
```

### Running with Claude Code

```bash
aletheia start                   # start the daemon (background, auto port)
# Expected: Daemon listening on http://127.0.0.1:<port>
claude --plugin-dir .            # run Claude Code with the plugin
```

If `aletheia` is not on your PATH yet, use:
`cargo run -p metaygn-cli -- start`

The daemon listens on a dynamic port (written to `~/.claude/aletheia/daemon.port`).
Without the daemon, hooks fall back to lightweight local heuristics.

### Advanced: MCP Integration (Codex CLI)

```bash
# One-shot installer (Windows / macOS / Linux)
powershell -ExecutionPolicy Bypass -File .\scripts\install-codex.ps1   # Windows
bash ./scripts/install-codex.sh                                        # macOS/Linux

# Guided session with MetaYGN protocol preloaded
powershell -ExecutionPolicy Bypass -File .\scripts\start-codex-metaygn.ps1  # Windows
bash ./scripts/start-codex-metaygn.sh                                       # macOS/Linux
```

MCP tools (`metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`) are available. Codex integrates through MCP tools (explicit calls), not automatic lifecycle hooks.

### Validation

```bash
aletheia status                  # check daemon health
# Expected: Daemon running, PID <N>, uptime <T>
aletheia doctor                  # check plugin + hooks + agents + DB
# Expected: All checks passed
claude plugin validate .         # validate plugin structure
# Expected: Plugin valid
```

### Optional features

```bash
cargo build --features mcp        # MCP stdio server (aletheiad --mcp)
cargo build --features syntax     # tree-sitter multi-language verification
cargo build --features otel       # OpenTelemetry tracing spans
cargo build --features embeddings # Neural embeddings (fastembed, bge-small-en-v1.5)
```

## FAQ

**Does it slow down Claude Code?**
Hook latency is ~30ms. The daemon runs locally, no network calls.

**What happens without the daemon?**
Hooks fall back to local regex heuristics. Still works, just less smart.

**Can I use it with other AI agents?**
The daemon exposes an HTTP API and MCP tools. Any agent that supports hooks or MCP can use it.

**What does it NOT do?**
It does not modify your code. It does not send data anywhere. It is a local watchdog, not a cloud service.

**Is it production-ready?**
v2.5 has 750+ tests and zero mutex panics. Experimental features are clearly tagged.

## Research Foundation

Experimental features draw on ideas from these papers (none fully replicated; see experimental tags):

| Abbreviation | Paper | Relevance |
|---|---|---|
| EGPO | Entropy-Guided Policy Optimization | Confidence calibration via entropy |
| RL2F | Reinforcement Learning from LLM Feedback | Implicit feedback for recovery |
| U-Mem | Uncertainty-Aware Memory | UCB-scored retrieval ranking |
| OpenSage | Open-Source Sage Agent | Multi-agent topology patterns |
| DyTopo | Dynamic Topology Planning | Adaptive execution topology |
| SideQuest | SideQuest Exploration | Heuristic evolution fitness |
| AlphaEvolve | AlphaEvolve | Evolutionary program search |

See `Meta-YGN/` for detailed architecture docs (Obsidian vault).

## License

MIT
