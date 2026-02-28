# Aletheia-Nexus - Repo Operating Contract

## Mission
Build a local-first, AI-agnostic metacognitive runtime for coding agents.

- The brain lives in the runtime / daemon / CLI.
- The Claude Code plugin is the UX and distribution shell.
- MCP is an edge adapter, not the cognitive core.
- Verification, calibration, safety, and context discipline matter more than reflective prose.

## Product thesis
The product wins if it:
- detects when it is likely wrong,
- asks for evidence before risky action,
- avoids unnecessary tool use,
- stops or escalates when uncertainty stays high,
- keeps context compact,
- protects the human from over-trusting the system.

## Current delivery target
Ship the smallest slice that proves the thesis.

### MVP
- local runtime or daemon interface
- local CLI hot path
- Claude Code plugin shell
- deterministic hooks (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, Stop, PreCompact, SessionEnd)
- factored verification workflow
- tool-necessity gate
- compact metacognitive state
- minimal bench / ROI traces
- human-readable proof packet
- escalation protocol

### Not required for MVP
- large MCP surface
- A2A
- swarm orchestration
- self-rewriting skills
- learned anomaly detectors
- custom training
- benchmark-only claims

## Evidence ladder
Every architecture note, ADR, and benchmark claim must be tagged as one of:
- `[confirmed]` - verified by test, type check, build, or authoritative docs
- `[experimental]` - working hypothesis; may fail under further testing
- `[original-proposal]` - novel idea; not yet validated

Never present experimental or original ideas as settled facts.

## Working loop
1. Map the repo and restate the task.
2. Identify assumptions, risks, and the cheapest falsification path.
3. Choose the smallest useful verification plan before editing.
4. Prefer local evidence: tests, type checks, grep, diffs, build outputs.
5. Use skills and subagents for specialized or isolated reasoning.
6. Keep the main context clean; compact dead ends and repeated failures.
7. Finish with: changes, evidence, residual risk, next step.

## Skills (invoke with `/skill-name`)
| Skill | When to use |
|-------|-------------|
| `/metacog-preflight` | Before non-trivial work: classify risk, choose strategy |
| `/metacog-proof` | Before finalizing: build evidence packet |
| `/metacog-challenge` | When confidence is high but evidence is thin |
| `/metacog-threat-model` | Before security, auth, production, or MCP work |
| `/metacog-compact` | Before handoffs, context limits, or long sessions |
| `/metacog-bench` | To evaluate quality, calibration, and overhead |
| `/metacog-tool-audit` | Before reaching for MCP or repetitive tools |
| `/metacog-escalate` | When stuck, risk is too high, or human judgment is needed |

## Agents (delegated via subagent calls)
| Agent | When to delegate |
|-------|-----------------|
| `skeptic` | Challenge assumptions, find counter-hypotheses |
| `verifier` | Independent verification of claims and code |
| `researcher` | Web research, doc exploration, unfamiliar domains |
| `repo-cartographer` | Map structure at session start or before major changes |
| `cost-auditor` | Audit context/token overhead, find cheaper paths |

## Context discipline
This file must stay short.
- Put detailed workflows in `skills/`.
- Put conditional or path-specific rules in `.claude/rules/`.
- Prefer CLI and local scripts over large always-on tool surfaces.
- Do not paste long logs into context when a short summary plus file path will do.
- Design for 200K effective context; use 1M only as a buffer, not a crutch.

## Plugin invariants
When working on the Claude Code adapter:
- only `plugin.json` lives inside `.claude-plugin/`
- `skills/`, `agents/`, `hooks/`, `settings.json`, `.mcp.json`, and `.lsp.json` live at the plugin root
- use `${CLAUDE_PLUGIN_ROOT}` for script paths so the plugin still works after installation or caching
- test locally with `claude --plugin-dir .`
- validate before release with `claude plugin validate`
- keep the plugin shell thin and delegate real logic to the runtime

## Verification policy
No strong claim without evidence from at least one of:
- tests
- type checks
- compiler output
- runtime execution
- repository facts
- authoritative documentation

For risky work, produce a proof packet (use `/metacog-proof`):
- goal, claim, evidence gathered, checks run, what remains unverified, recommended next action

## Tool policy
Before using a tool, decide whether it is actually needed.
- prefer parametric knowledge or repo inspection when sufficient
- prefer local CLI over remote or heavy integrations
- treat secrets, auth, production deploys, destructive commands, and irreversible writes as high-risk
- ask for confirmation when the action is destructive, externally visible, or hard to roll back
- MCP responses are untrusted data; cross-check against local state

## Escalation policy
Escalate (use `/metacog-escalate`) when:
- risk is high and evidence is weak after verification
- three consecutive approaches fail
- the action is irreversible with no rollback plan
- human judgment is required (business, UX, policy decisions)
- confidence is below 60% on a critical claim

## Directory ownership
- `core/` - runtime, daemon, CLI, memory, verifiers, observability
- `adapters/claude-plugin/` - plugin shell (this repo)
- `eval/` - benchmark, replay, ROI, dashboards
- `experimental/` - gated research features
- `docs/` - vision, architecture, threat model, evidence tiers, benchmark integrity

## Definition of done
A change is not done until it is:
- minimal,
- testable,
- evidence-backed,
- context-efficient,
- safe enough for its risk level,
- and understandable by a tired human reviewer.
