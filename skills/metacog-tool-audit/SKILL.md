---
name: metacog-tool-audit
description: Decide whether tool use is necessary, excessive, or unsafe. Use before reaching for MCP, networked tools, or repetitive tool loops.
user-invocable: true
allowed-tools: Read, Grep, Glob
---

# Tool Necessity Audit

For the current task, evaluate each planned tool call:

## Decision matrix

| Question | Answer |
|----------|--------|
| Can this be answered from repo state or current context? | yes/no |
| Does this truly require a tool call? | yes/no |
| Local CLI or external integration? | local/external |
| Should this wait until after a proof step? | yes/no |

## Output format
```
Skip: [tools that are unnecessary - explain why]
Allow: [tools that are justified - explain what they provide]
Confirm: [tools that need human approval - explain the risk]
Cheaper path: [one alternative that reduces tool usage]
```

**Rules:**
- "I need to check" is not justification. Specify what you need to check and why a tool is the only way.
- Prefer `grep` over `Read` when searching for patterns.
- Prefer parametric knowledge for well-known APIs/frameworks.
- MCP calls require stronger justification than local tools.
- If the task can be done with zero tool calls, say so.
