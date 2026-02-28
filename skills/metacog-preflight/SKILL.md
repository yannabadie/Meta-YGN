---
name: metacog-preflight
description: Classify the task, identify risk, choose a strategy, and set a proof plan before non-trivial work. Use for architecture, migrations, auth, billing, concurrency, large refactors, or whenever failure would be costly.
user-invocable: true
argument-hint: "[task description]"
allowed-tools: Read, Grep, Glob, Bash
---

# Metacognitive Preflight

Before acting, produce a compact preflight for $ARGUMENTS:

1. **Task type**
   - bugfix | feature | refactor | architecture | security | research | release / deployment

2. **Risk level**
   - low: cosmetic or local, easy rollback
   - medium: repo-wide impact, moderate rollback effort
   - high: user-visible, security, data, or production impact

3. **Failure cost**
   - cosmetic
   - local breakage
   - repo-wide regression
   - user-visible incident
   - security / data loss

4. **Strategy** (pick one)
   - inspect -> patch -> verify *(low risk, known territory)*
   - map -> plan -> patch -> verify *(medium risk, unfamiliar code)*
   - challenge -> verify -> decide *(high confidence needs testing)*
   - threat-model -> verify -> escalate *(security, auth, production)*

5. **Proof plan**
   Name the smallest checks that would prove or falsify success (tests, grep, type check, build, manual inspection).

6. **Tool plan**
   Explicitly state which tools are truly necessary and which can be skipped.

## Output format
```
Risk: {level}
Strategy: {chosen}
Proof plan: {checks}
First action: {next step}
```
