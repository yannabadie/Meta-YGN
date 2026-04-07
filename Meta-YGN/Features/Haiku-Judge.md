---
title: Haiku LLM Judge
type: feature
evidence_tier: experimental
crate: metaygn-daemon
tags: [feature, experimental, security, judge]
created: 2026-04-07
---

# Haiku LLM Judge

Tier 3 judge in the ADR-004 cascade verification pipeline. Handles ambiguous commands that deterministic guards cannot resolve with confidence.

## How it works

- Calls Claude Haiku via Anthropic API with a structured safety prompt
- LRU cache (100 entries) avoids redundant API calls for identical inputs
- 20-call budget per session prevents runaway costs
- 500ms timeout per call; abstains on timeout or error

## Verdicts

| Verdict | Meaning |
|---------|---------|
| Safe | Action is safe to proceed |
| Risky | Action has potential risk, warn user |
| Dangerous | Action is dangerous, block or escalate |
| Abstain | Judge could not decide (timeout, budget, error) |

## Feature gate

- `--features judge`
- Requires `ANTHROPIC_API_KEY` environment variable
- When disabled, Tier 3 is skipped and the cascade falls through to Tier 4

## Integration

- Wired into `SessionContext` and `AppState`
- Called from `pre_tool_use` hook when Tiers 1-2 return ambiguous results
- Results cached by command hash for the session lifetime

## References

- ADR-004 (cascade verification architecture)
- LLM-as-judge pattern (Zheng et al. 2023)
