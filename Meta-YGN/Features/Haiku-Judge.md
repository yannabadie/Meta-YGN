---
title: Haiku LLM Judge
type: feature
evidence_tier: experimental
tags: [feature, experimental, security, judge]
created: 2026-04-07
updated: 2026-04-07
---

# Haiku LLM Judge

Tier 3 judge in the ADR-004 cascade verification pipeline. Handles ambiguous commands that deterministic guards cannot resolve with confidence.

## Implementation (v2.3.1+)

No Rust code. No API key. No dependency management. The judge is a native Claude Code `"type": "prompt"` hook — 5 lines of JSON in `hooks/hooks.json`.

```json
{
  "type": "prompt",
  "event": "PreToolUse",
  "model": "haiku",
  "prompt": "<few-shot safety classification prompt — 5 examples>"
}
```

Claude Code handles model auth and execution natively. The prompt hook runs in parallel with the daemon command hook; the strictest verdict between the two wins.

## How it works

- Activated on `PreToolUse` events where Tiers 1–2 return ambiguous results
- Few-shot prompt with 5 labelled examples guides the model toward consistent verdicts
- Runs in parallel with the existing daemon `"type": "command"` hook
- Strictest verdict (Dangerous > Risky > Safe > Abstain) is applied to the session
- No LRU cache, no budget counter, no timeout configuration — Claude Code manages lifecycle

## Verdicts

| Verdict | Meaning |
|---------|---------|
| Safe | Action is safe to proceed |
| Risky | Action has potential risk, warn user |
| Dangerous | Action is dangerous, block or escalate |
| Abstain | Judge could not decide |

## Configuration

Zero setup beyond hooks.json. No feature flag. No `ANTHROPIC_API_KEY` required — Claude Code handles auth.

Previously this was a Rust module behind `--features judge` requiring:
- `reqwest` optional dep
- `lru` crate
- `ANTHROPIC_API_KEY` env var
- `judge.rs` source (~150 LOC)

All of that is replaced by the prompt hook entry.

## References

- ADR-004 (cascade verification architecture)
- LLM-as-judge pattern (Zheng et al. 2023)
- Claude Code hooks documentation (`"type": "prompt"`)
