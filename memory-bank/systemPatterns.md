# System Patterns

## Architectural Patterns

- **Daemon-first with local fallback**: Every hook script tries `daemon_call()` first; if timeout (350ms) or unavailable, falls back to deterministic local heuristics. Never blocks on daemon failure.

- **Three-tier permission gating**: DESTRUCTIVE_PATTERNS → deny, HIGH_RISK_BASH → ask, SECRET_PATH_PATTERNS → ask, mcp__ → ask, default → allow. Patterns are regex-based in `common.py`.

- **Progressive disclosure**: Skill descriptions loaded into context at startup (lightweight). Full skill content only loaded when invoked. Keeps idle context cost near zero.

- **Agent delegation**: aletheia-main orchestrates by delegating to specialized agents (skeptic, verifier, researcher, cartographer, auditor) based on task characteristics. Each agent has restricted tools and focused instructions.

## Design Patterns

- **Evidence tagging**: All claims tagged `[confirmed]`, `[experimental]`, or `[unverified]`. Applied in proof packets, architecture notes, and decision logs.

- **Risk classification pipeline**: UserPromptSubmit → classify_prompt() → emit risk/budget/mode hint → PreToolUse → pattern matching → deny/ask/allow.

- **Proof packet structure**: Goal → Changes → Evidence → Uncertainty → Next step. Enforced by output style and Stop hook.

- **Structured compaction**: PreCompact emits numbered sections (goal, verified facts, failed approaches, open risks, next action). Prevents context rot in long sessions.

## Common Idioms

- **Hook I/O**: Read JSON from stdin, emit JSON to stdout. Exit 0 = success, exit 2 = blocking error.
- **`${CLAUDE_PLUGIN_ROOT}`**: Always use for script paths in hooks.json for portability after plugin installation.
- **`matches_any(text, patterns)`**: Standard pattern-matching gate in all security checks.
- **Daemon communication**: POST to `ALETHEIA_DAEMON_URL/{route}` with JSON payload, 350ms timeout, silent failure.
