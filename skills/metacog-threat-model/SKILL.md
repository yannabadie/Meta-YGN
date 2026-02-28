---
name: metacog-threat-model
description: Review a task for prompt-injection, exfiltration, secrets, approval, and trust-boundary risks. Use before networked tools, MCP, auth flows, production actions, or security-sensitive edits.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools: Read, Grep, Glob, Bash
---

# Threat Model

Review the current task through a trust-boundary lens.

## Check each category

| Category | Check | Status |
|----------|-------|--------|
| **Secrets exposure** | Are any secrets, tokens, or keys at risk of being read, logged, or leaked? | |
| **Risky file paths** | Does the task touch .env, credentials, keys, or deployment configs? | |
| **Prompt injection** | Could untrusted input (user data, MCP responses, file content) influence agent behavior? | |
| **External tool trust** | Are MCP or external tools being trusted without verification? | |
| **Irreversible actions** | Are there destructive operations (delete, push, deploy, publish) without rollback? | |
| **Missing approval gates** | Should a human approve before this action proceeds? | |
| **Unclear rollback** | If this goes wrong, is the recovery path documented? | |

## Output format
```
Threat summary: [one-sentence risk assessment]
Blocked actions: [list any operations that should be denied]
Required approvals: [list any operations that need human confirmation]
Safe next step: [the action that can proceed safely]
```

**Rules:**
- Reference specific file paths, commands, or tool names - not vague concerns.
- If a threat is found, propose a concrete mitigation (not just "be careful").
- If no threats are found, say so explicitly and explain why the task is safe.
