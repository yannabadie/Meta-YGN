# Aletheia-Nexus Claude Code Plugin

A local-first metacognitive plugin for Claude Code that adds verification, risk classification, safety gates, and context discipline to AI-assisted coding.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Claude Code                        │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │  Skills   │  │  Agents  │  │  Output Styles   │  │
│  │ (8 metacog│  │ (6 roles)│  │ (proof packets)  │  │
│  │ workflows)│  │          │  │                  │  │
│  └─────┬─────┘  └────┬─────┘  └────────┬─────────┘  │
│        │              │                 │            │
│  ┌─────┴──────────────┴─────────────────┴─────────┐  │
│  │              Hooks (8 lifecycle events)          │  │
│  │  SessionStart → UserPromptSubmit → PreToolUse   │  │
│  │  → PostToolUse → PostToolUseFailure → Stop      │  │
│  │  → PreCompact → SessionEnd                      │  │
│  └─────────────────────┬───────────────────────────┘  │
│                        │                             │
│  ┌─────────────────────┴───────────────────────────┐  │
│  │         Python Hook Handlers (scripts/)          │  │
│  │  • Risk classification (prompt & tool level)     │  │
│  │  • Security gates (destructive/high-risk/secret) │  │
│  │  • Verification signals                          │  │
│  │  • Proof packet enforcement                      │  │
│  └─────────────────────┬───────────────────────────┘  │
│                        │                             │
│  ┌─────────────────────┴───────────────────────────┐  │
│  │    Optional: Aletheia Daemon (ALETHEIA_DAEMON_URL)│  │
│  │    • Durable state & learned heuristics          │  │
│  │    • Advanced verification & proof archival      │  │
│  │    • Graceful fallback to local heuristics       │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

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
