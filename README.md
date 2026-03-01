# Aletheia-Nexus Claude Code Plugin

A local-first metacognitive plugin for Claude Code that adds verification, risk classification, safety gates, and context discipline to AI-assisted coding.

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

## Developer-First Features (v0.4.0)

These features solve the top 3 real-world Claude Code pain points:

### Completion Verifier
When Claude says "Done!", MetaYGN checks if the mentioned files actually exist. If they don't, it blocks the completion claim with a warning.

### Test Integrity Guard
When Claude tries to Edit a test file, MetaYGN analyzes the diff. If assertions are removed, test functions deleted, or expected values changed, it asks for confirmation with a detailed warning.

### Token Budget Dashboard
Every hook response includes a visible budget summary: `[budget: 3000tok/$0.03 used of 100000tok/$1.00 | 3%]`. Warns at 80% utilization.

## Smart Recovery (v0.5.0)

### Plasticity-Aware Recovery
When error recovery fails, MetaYGN escalates progressively:
- **Level 1**: Standard recovery hint
- **Level 2**: Emphatic critique with concrete alternative strategies
- **Level 3**: Auto-escalation recommending `/metacog-escalate`

### Implicit Feedback
MetaYGN tracks whether recovery prompts work without asking the developer. If the same error returns after recovery, plasticity score drops and amplification increases.

### Latency Transparency
Every hook response includes `[latency: Nms]` so developers know exactly how much overhead MetaYGN adds.

## Deep Foundation (v0.7.0)

### No More Stubs
Every declared module is now fully implemented. The `act` stage records intentions, the `compact` stage deduplicates and summarizes, the event system is typed, and FTS search spans both events and graph memory.

### Context Pruning Service
`POST /proxy/anthropic` accepts Anthropic message payloads, detects reasoning lock-in (3+ consecutive errors), and returns pruned messages with recovery prompts injected.

### Skill Crystallizer
MetaYGN observes your tool usage patterns. When a sequence repeats 3+ times, it generates a reusable SKILL.md template automatically.

## Solid Ground (v0.6.0)

### Full Plugin Pipeline
All 8 TypeScript hooks fire through the daemon with 350ms timeout and local fallback. Install and run:
```bash
aletheia start     # spawn daemon
claude --plugin-dir /path/to/MetaYGN
aletheia status    # verify
aletheia stop      # clean shutdown
```

### Persistent Learning
Heuristic versions and session outcomes survive daemon restarts via SQLite persistence.

## Components

### Skills (8 metacognitive workflows)
| Skill | Purpose |
|-------|---------|
| `metacog-preflight` | Classify risk, choose strategy before acting |
| `metacog-proof` | Build structured evidence packet |
| `metacog-challenge` | Pressure-test assumptions and plans |
| `metacog-threat-model` | Security and trust boundary review |
| `metacog-compact` | Compress session for handoff |
| `metacog-bench` | Evaluate quality and overhead |
| `metacog-tool-audit` | Assess tool necessity |
| `metacog-escalate` | Structured escalation when stuck or uncertain |

### Agents (6 specialized roles)
| Agent | Model | Purpose |
|-------|-------|---------|
| `aletheia-main` | sonnet | Default execution with verification discipline |
| `skeptic` | sonnet | Challenge assumptions, find counter-hypotheses |
| `verifier` | sonnet | Independent verification of claims |
| `researcher` | sonnet | Web research and documentation exploration |
| `repo-cartographer` | haiku | Map repository structure |
| `cost-auditor` | haiku | Audit token and context overhead |

### Hooks (8 lifecycle events)
- **SessionStart**: Detect tech stack, initialize profile
- **UserPromptSubmit**: Classify prompt risk (high/medium/low)
- **PreToolUse**: Security gates (deny destructive, ask risky, allow safe)
- **PostToolUse**: Verification signals and change tracking
- **PostToolUseFailure**: Error diagnosis guidance
- **Stop**: Proof packet enforcement
- **PreCompact**: Structured context compaction
- **SessionEnd**: Session finalization

### Security gates
- **Auto-deny**: `rm -rf /`, fork bombs, raw disk writes
- **Auto-ask**: `git push`, `terraform apply`, `kubectl delete`, `sudo`, `curl|bash`, secret files
- **MCP gate**: All external MCP calls require confirmation

## Installation

### Local development
```bash
claude --plugin-dir .
```

### Validation
```bash
claude plugin validate
claude --debug --plugin-dir .
```

### Wiring a local daemon
```bash
export ALETHEIA_DAEMON_URL=http://localhost:9000
```
Without it, hooks fall back to lightweight local heuristics.

## Design principles
1. **Verification over speculation**: Evidence before claims
2. **Context discipline**: Design for 200K, not 1M
3. **Safety by default**: Deny destructive, ask risky, allow safe
4. **Thin plugin shell**: Logic lives in runtime, not UI
5. **Graceful degradation**: Works without daemon, MCP, or advanced features
6. **Proof over prose**: Structured packets, not reflective narration

## License
MIT
