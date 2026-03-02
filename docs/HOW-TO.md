# How to Use MetaYGN (Aletheia-Nexus)

> A local-first metacognitive control plane for coding agents.
> MetaYGN watches your AI coding assistant, catches its mistakes, and makes it smarter.

## What It Does

When you use Claude Code (or any MCP-compatible AI coding agent), MetaYGN:

1. **Blocks dangerous commands** — `rm -rf /`, `git push --force`, `sudo`, credential access → denied in <5ms
2. **Detects when Claude is wrong** — overconfidence detection, test failure parsing, syntax checking
3. **Tracks fatigue** — if you or the AI are looping on errors, it escalates
4. **Remembers across sessions** — graph memory stores decisions, lessons, outcomes
5. **Evolves its heuristics** — learns which strategies work for which task types

## Quick Start (5 minutes)

### Prerequisites
- [Rust](https://rustup.rs/) 1.85+
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 9+ (`npm install -g pnpm`)
- [Claude Code](https://claude.ai/code) CLI

### Install
```bash
git clone https://github.com/yannabadie/Meta-YGN && cd Meta-YGN
cargo build --workspace          # builds daemon + CLI (2-3 min first time)
pnpm install                     # installs TypeScript hook dependencies
```

### Run
```bash
# Terminal 1: Start the daemon
./target/debug/aletheia start

# Terminal 2: Use Claude Code with MetaYGN
claude --plugin-dir .
```

That's it. MetaYGN is now watching every tool call Claude makes.

### Verify it works
```bash
./target/debug/aletheia status   # should say RUNNING
./target/debug/aletheia doctor   # checks plugin health
```

## What Happens When You Code

```
You: "Fix the authentication bug in login.rs"

Claude Code tries to use a tool:
  ┌─────────────────────────────────────────────┐
  │ PreToolUse hook fires (< 5ms)               │
  │                                             │
  │ 1. Guard Pipeline: is this dangerous?       │
  │    → rm -rf? DENY                           │
  │    → cargo test? ALLOW (risk: low)          │
  │                                             │
  │ 2. Risk Classification: how risky?          │
  │    → low / medium / high                    │
  │                                             │
  │ 3. Strategy Selection: how to approach?     │
  │    → step-by-step / verify-first / rapid    │
  │                                             │
  │ 4. Budget Check: tokens remaining?          │
  │    → 95,000 / 100,000 tokens (5% used)     │
  └─────────────────────────────────────────────┘

After the tool runs:
  ┌─────────────────────────────────────────────┐
  │ PostToolUse hook fires (async, non-blocking)│
  │                                             │
  │ 1. Syntax Check: did the file parse?        │
  │    → JSON/YAML/TOML: in-process serde       │
  │    → Python: forge sandbox (ast.parse)      │
  │    → Rust/JS/TS: tree-sitter (if enabled)   │
  │                                             │
  │ 2. Graph Memory: store evidence             │
  │    → Task → Evidence (Produces)             │
  │    → Evidence → Decision (Verifies)         │
  │                                             │
  │ 3. Entropy Tracking: is Claude overconfident?│
  │    → Sliding window of confidence vs outcome│
  └─────────────────────────────────────────────┘

When Claude says "Done!":
  ┌─────────────────────────────────────────────┐
  │ Stop hook fires                             │
  │                                             │
  │ 1. Completion Verification: did Claude lie? │
  │    → Checks mentioned files actually exist  │
  │                                             │
  │ 2. Heuristic Evolution: learn from session  │
  │    → Record outcome (success/failure)       │
  │    → Evolve strategy weights after 5 sessions│
  │                                             │
  │ 3. Topology: what ran?                      │
  │    → Trivial task: 4 stages (fast)          │
  │    → Normal task: 12 stages                 │
  │    → Security task: 14 stages (double verify)│
  └─────────────────────────────────────────────┘
```

## CLI Commands

```bash
aletheia start [--db-path PATH]   # Start daemon (background, auto port)
aletheia stop                     # Stop daemon gracefully
aletheia status                   # Show daemon health + version
aletheia recall --query "auth"    # Search memory for past events
aletheia replay                   # List recorded sessions
aletheia replay <session-id>      # View session hook timeline
aletheia export [--limit N]       # Export RL trajectories to JSONL
aletheia eval                     # Show calibration report
aletheia doctor                   # Check installation health
aletheia top                      # Real-time TUI dashboard
aletheia mcp                      # Launch MCP stdio server
aletheia init                     # Scaffold .claude/ config
```

## Optional Features (Cargo Feature Flags)

```bash
cargo build --features mcp        # MCP stdio server (aletheiad --mcp)
cargo build --features syntax     # tree-sitter verification (Rust/Python/JS/TS)
cargo build --features otel       # OpenTelemetry tracing spans
cargo build --features embeddings # Neural embeddings (fastembed, bge-small-en-v1.5)
```

## Monitoring

### Prometheus Metrics
```bash
curl http://127.0.0.1:$(cat ~/.claude/aletheia/daemon.port)/metrics
```

Returns:
```
metaygn_active_sessions 2
metaygn_events_total 42
metaygn_graph_nodes_total 15
metaygn_fatigue_score 0.15
metaygn_tokens_consumed_total 25000
```

### Session Replay
```bash
aletheia replay                   # list all sessions
aletheia replay <session-id>      # view hook timeline with latencies
```

## Architecture (for AI agents reading this)

```
Claude Code
  │
  ├── hooks/ (8 lifecycle events, TypeScript via npx tsx)
  │     ├── Calls daemon HTTP API (350ms timeout, local fallback)
  │     ├── 5 hooks call daemon (PreToolUse, PostToolUse, UserPromptSubmit, Stop, SessionEnd)
  │     └── 3 hooks run locally in TS for speed (SessionStart, PostToolUseFailure, PreCompact)
  │
  ├── aletheiad (Rust daemon, 127.0.0.1:dynamic-port)
  │     ├── 12-stage control loop (classify→learn)
  │     ├── 5-guard security pipeline
  │     ├── Graph memory (SQLite + FTS5 + cosine similarity)
  │     ├── Heuristic evolver (population-based, statistical mutations)
  │     ├── Session context (cross-hook state persistence)
  │     ├── Forge engine (verification script templates)
  │     ├── Prometheus /metrics endpoint
  │     └── MCP server (--mcp flag, 5 metacognitive tools)
  │
  ├── skills/ (8 metacognitive workflows, user-invocable)
  └── agents/ (6 specialized agents: main, skeptic, verifier, researcher, cartographer, cost-auditor)
```

## For AI Agents Reading This

MetaYGN communicates with you through hook responses. Here's how to interpret them:

### PreToolUse Response
- `permissionDecision: "allow"` (or absent) — proceed normally
- `permissionDecision: "deny"` — **STOP**, do not execute this tool
- `permissionDecision: "ask"` — ask the human for confirmation first
- `additionalContext` — contains risk level, strategy hint, budget status

### PostToolUse Response
- `additionalContext` contains verification results (syntax check, test output parsing)
- Treat this as evidence for your reasoning chain

### Stop Response
- Contains completion verification and metacognitive vector
- Fill in the proof packet: Goal, Changes, Evidence, Uncertainty, Next Step

### UserPromptSubmit Response
- `additionalContext` — risk classification, strategy recommendation, topology
- Use this to calibrate your approach to the task

### Key Metrics
- `[risk:low|medium|high]` — how risky this action is
- `[strategy:Rapid|StepByStep|TreeExplore|...]` — recommended reasoning strategy
- `[budget: Ntok/$N used of Ntok/$N]` — token/cost budget remaining
- `[latency: Nms]` — daemon processing time

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Hooks don't fire | Run `pnpm install` in repo root |
| Daemon won't start | Check `~/.claude/aletheia/daemon.port` for stale file, delete it |
| "npx tsx not found" | Install Node.js 22+ |
| Plugin not loading | Run `claude plugin validate .` to check structure |
| Slow first hook | `npx tsx` downloads tsx on first run (~2s), then cached |
