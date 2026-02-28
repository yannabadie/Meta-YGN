---
name: metacog-compact
description: Compact the session into goal, verified facts, dead ends, open risks, and next actions. Use before long sessions, handoffs, context limits, or after repeated failed attempts.
user-invocable: true
disable-model-invocation: true
allowed-tools: Read, Grep, Glob
---

# Compact for handoff

Create a compact handoff with these sections:

## Current goal
One sentence describing what the session is trying to achieve.

## Verified facts
Only include facts backed by evidence (tests passed, files confirmed, patterns found). Tag each as `[confirmed]` or `[experimental]`.

## Failed approaches
Briefly list approaches that were tried and failed, with the reason. This prevents the next agent from repeating them.

## Open risks
List unresolved questions, unverified assumptions, and known fragile areas.

## Next best action
The single most valuable next step to make progress.

---

**Rules:**
- Keep the total under 50 lines.
- Drop: repetitive logs, dead-end narration, tool outputs already acted upon.
- Prioritize: anything another agent needs to avoid wasting turns.
- This compact should be self-contained: a new agent should understand the state without reading the full conversation.
