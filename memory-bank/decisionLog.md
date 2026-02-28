# Decision Log

| Date | Decision | Rationale | Status |
|------|----------|-----------|--------|
| 2026-02-28 | Use Python stdlib only for hooks | Zero dependencies, fast startup, no build step | `[confirmed]` |
| 2026-02-28 | 8 lifecycle hooks (added PostToolUseFailure, Stop) | PostToolUseFailure provides error recovery guidance; Stop enforces proof packet | `[confirmed]` |
| 2026-02-28 | 6 agents (added researcher) | Web research needs separate agent with WebSearch/WebFetch tools but no write access | `[confirmed]` |
| 2026-02-28 | 8 skills (added metacog-escalate) | Escalation protocol needed when risk exceeds agent capability or agent is stuck | `[confirmed]` |
| 2026-02-28 | Evidence tagging in proof packets | `[confirmed]`/`[experimental]`/`[unverified]` makes evidence strength explicit for reviewers | `[confirmed]` |
| 2026-02-28 | MCP responses treated as untrusted | PostToolUse emits trust boundary warning for MCP tool outputs. Cross-check against local state. | `[confirmed]` |
| 2026-02-28 | haiku for read-only agents | repo-cartographer and cost-auditor don't need sonnet reasoning; haiku is 10x cheaper | `[confirmed]` |
| 2026-02-28 | 350ms daemon timeout | Non-blocking; allows 2-3 round trips on localhost. Falls back silently. | `[experimental]` |
| 2026-02-28 | Timestamp all log events | ISO 8601 `ts` field enables time-based analysis and debugging | `[confirmed]` |
| 2026-02-28 | Expand security patterns (curl\|bash, sudo, fork bomb) | Original patterns missed common attack vectors | `[confirmed]` |
