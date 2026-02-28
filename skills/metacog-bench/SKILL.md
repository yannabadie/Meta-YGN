---
name: metacog-bench
description: Run a minimal metacognitive evaluation pass over the current task or repository change. Use to assess quality, calibration, and overhead.
user-invocable: true
disable-model-invocation: true
allowed-tools: Read, Grep, Glob, Bash
---

# Minimal MetaCog Bench

Evaluate the current work on six axes:

| Axis | Question | Score 1-5 |
|------|----------|-----------|
| **Correctness evidence** | Is the result backed by tests, checks, or verifiable facts? | |
| **Calibration quality** | Does the stated confidence match the actual evidence strength? | |
| **Tool efficiency** | Were tools used only when necessary? Any redundant calls? | |
| **Context overhead** | How much context was consumed vs. the value delivered? | |
| **Workflow friction** | Were there unnecessary round-trips, retries, or dead ends? | |
| **Residual risk** | What is the likelihood of an undetected issue? | |

## Return exactly

```
Scores: correctness=X, calibration=X, tools=X, context=X, friction=X, risk=X
Strongest gain: [what Aletheia workflows helped most]
Strongest cost: [what added overhead without proportional value]
Improvement: [one concrete instrumentation or workflow change]
```

**Rules:**
- Be honest. A 5/5 on correctness with no tests run is wrong.
- "Strongest cost" must be actionable (not "it took time").
- The improvement must be specific enough to implement in the next session.
