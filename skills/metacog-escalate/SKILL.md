---
name: metacog-escalate
description: Escalate when uncertainty is too high, risk exceeds agent capability, or human judgment is required. Use when verification fails, threats are found, or the agent is stuck.
user-invocable: true
disable-model-invocation: false
allowed-tools: Read, Grep, Glob
---

# Escalation Protocol

When the agent cannot safely proceed, escalate with structured information.

## Triggers for escalation
- Risk is high and evidence is weak after verification attempts
- Threat model found unmitigated security risks
- Three consecutive failed approaches with no clear path forward
- The task requires human judgment (business decision, UX choice, policy)
- The action is irreversible and no rollback plan exists
- Confidence is below 60% on a critical claim

## Escalation format

```
## Escalation

**Reason**: [why the agent cannot safely proceed]
**Current state**: [what has been done so far]
**What was tried**: [approaches attempted and why they failed]
**Options**:
1. [Option A] - risk: X, effort: Y
2. [Option B] - risk: X, effort: Y
3. [Do nothing] - consequence: Z

**Recommendation**: [which option and why, or "needs human judgment"]
**Information needed**: [what the human could provide to unblock]
```

## Rules
- Never guess on high-risk decisions. Escalate.
- Present options, not just problems.
- Include "do nothing" as an explicit option with its consequences.
- If the escalation is about a technical blocker, include the exact error or observation.
- After escalating, stop and wait for human input. Do not continue on assumptions.
