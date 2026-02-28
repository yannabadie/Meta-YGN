---
name: metacog-challenge
description: Pressure-test the current plan or conclusion. Use when reasoning may be too linear, confidence seems too high, or there are plausible counter-hypotheses.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools: Read, Grep, Glob, Bash
---

# Challenge the current reasoning

Treat the current plan as a draft that may be wrong.

Do not solve from scratch. Instead:

1. **Identify the strongest hidden assumption**
   What is the plan taking for granted that has not been verified?

2. **Identify the strongest competing explanation**
   What alternative hypothesis explains the same observations?

3. **Point out missing evidence**
   What evidence would change the decision if found?

4. **Name the cheapest falsification step**
   What is the single cheapest action (grep, test, read) that would confirm or refute the plan?

## Output format
```
Brittle assumption: ...
Rival hypothesis: ...
Missing evidence: ...
Falsification step: ...
```

**Rules:**
- Be specific, not vague. "The assumption that X works" is better than "there might be issues."
- If the plan is actually sound, say so and explain why the assumption is justified.
- Suggest Bash commands or grep patterns for the falsification step when possible.
