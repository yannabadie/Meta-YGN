# Architecture Notes

Status: `[confirmed]` - validated through implementation in v0.2.0

## Core principles

### 1. Keep the plugin shell thin
The plugin provides hooks, skills, agents, and output styles. All durable state, learned heuristics, and advanced verification live in the runtime/daemon. The plugin is an adapter, not the brain.

### 2. Route durable state to the runtime
Hook scripts call `ALETHEIA_DAEMON_URL` when available. If the daemon is down, they fall back to deterministic local heuristics. This ensures the plugin always works, but gets smarter when the daemon is connected.

### 3. Treat MCP and LSP as optional edge adapters
MCP tools expand the trust boundary and consume context tokens for schemas. Use them only when a capability truly requires an external process. Prefer local CLI for the hot path.

### 4. Prefer verification and observability over verbose self-reflection
Structured proof packets (Goal, Changes, Evidence, Uncertainty, Next step) are more useful than paragraphs of reasoning. External verification (tests, lints, type checks) is stronger than self-assessment.

## Three-tier architecture

```
Tier 1: Runtime/Daemon (stateful, persistent)
  - Durable state (SQLite, episodic memory)
  - Learned heuristics and policy
  - Proof archival and ROI metrics
  - Session summaries

Tier 2: Plugin Shell (stateless, deterministic)
  - 8 lifecycle hooks (Python scripts)
  - 8 metacognitive skills (Markdown workflows)
  - 6 specialized agents (Markdown definitions)
  - 1 output style (proof packet format)
  - Event logging (~/.claude/aletheia/events.jsonl)

Tier 3: Edge Adapters (optional)
  - MCP: only for capabilities needing external processes
  - LSP: only if packaged language servers add value
```

## Hook execution flow

```
SessionStart
  ↓
UserPromptSubmit (per user input)
  → classify risk → emit strategy hint
  ↓
PreToolUse (per tool call)
  → daemon_call → pattern check → Allow/Ask/Deny
  ↓
Tool Execution
  ↓
PostToolUse (on success)     PostToolUseFailure (on failure)
  → verification signals       → error diagnosis guidance
  ↓                            ↓
Stop (when Claude finishes)
  → proof packet enforcement
  ↓
PreCompact (on context limit or manual trigger)
  → structured compaction guidance
  ↓
SessionEnd (async)
  → event log + daemon notification
```

## Security model

Three-tier permission gating:

| Tier | Pattern | Decision | Example |
|------|---------|----------|---------|
| Destructive | `rm -rf /`, fork bomb, `mkfs` | **Deny** | Always blocked |
| High-risk | `git push`, `sudo`, `terraform apply`, `curl\|bash` | **Ask** | Requires confirmation |
| Sensitive path | `.env`, `.pem`, `credentials.json` | **Ask** | Requires confirmation |
| MCP call | `mcp__*` | **Ask** | Trust boundary crossing |
| Default | Everything else | **Allow** | Proceeds normally |

## Agent coordination

```
aletheia-main (orchestrator)
  ├── skeptic (challenge assumptions before irreversible decisions)
  ├── verifier (independent checks before finalizing risky work)
  ├── researcher (web research for unfamiliar domains)
  ├── repo-cartographer (structure mapping at session start)
  └── cost-auditor (overhead analysis when workflow feels bloated)
```

## Design decisions

| Decision | Rationale | Status |
|----------|-----------|--------|
| Python for hooks | Zero dependencies, stdlib only, fast startup | `[confirmed]` |
| Markdown for skills/agents | Native Claude Code format, lazy-loaded | `[confirmed]` |
| JSONL for event logging | Append-only, crash-safe, easy to parse | `[confirmed]` |
| 350ms daemon timeout | Non-blocking; falls back to local heuristics | `[experimental]` |
| haiku for cartographer/auditor | Cost-optimized for read-only analysis | `[confirmed]` |
| sonnet for main/skeptic/verifier | Balance of capability and cost | `[confirmed]` |
| Proof packet output style | Structured evidence > narrative prose | `[confirmed]` |
