# Aletheia-Nexus: System Architecture

## Overview
Three-tier architecture separating stateful runtime from stateless plugin shell from optional edge adapters.

## Architectural Decisions

1. **Python for hooks, not TypeScript** `[confirmed]`
   Zero external dependencies. Python stdlib provides json, re, urllib, pathlib. Fast startup (<100ms). No npm install or build step required.

2. **Thin plugin shell, fat daemon** `[confirmed]`
   Plugin supplies hooks/skills/agents/output-styles only. All durable state, learned heuristics, and advanced verification routes to `ALETHEIA_DAEMON_URL`. Plugin always works standalone (graceful degradation).

3. **MCP as optional edge adapter, not core** `[confirmed]`
   MCP schemas consume 10K-15K tokens of context window. CLI-first approach reduces this 95%. MCP only for capabilities truly needing external processes.

4. **Proof packets over narrative prose** `[confirmed]`
   Structured output (Goal, Changes, Evidence, Uncertainty, Next step) with evidence tagging (`[confirmed]`, `[experimental]`, `[unverified]`). More useful for review than paragraphs of reasoning.

5. **Three-tier security gating** `[confirmed]`
   Destructive → deny, high-risk → ask, sensitive-path → ask, MCP → ask, default → allow. Conservative by design (false positives over false negatives).

6. **haiku for read-only agents, sonnet for reasoning** `[confirmed]`
   Cost optimization: repo-cartographer and cost-auditor use haiku (cheaper, read-only). Main agent, skeptic, verifier, researcher use sonnet (better reasoning).

7. **JSONL event logging** `[confirmed]`
   Append-only, crash-safe, ISO 8601 timestamps. Located at ~/.claude/aletheia/events.jsonl. Foundation for future daemon analytics and ROI tracking.

8. **Factored verification over self-correction** `[confirmed]`
   Verifier agent runs independently from main agent. Cross-checking is more reliable than self-assessment. Research shows self-verification has lower gain than independent verification.
