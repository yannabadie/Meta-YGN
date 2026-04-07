---
title: Sequence Monitor
tier: experimental
crate: metaygn-core
file: crates/core/src/sequence_monitor.rs
tags:
  - safety
  - sequence
  - dtmc
updated: 2026-04-07
---

# Sequence Monitor

DTMC-inspired multi-action pattern detector. Detects dangerous multi-action patterns that individual tool calls would not flag.

## Patterns detectes

| Rule | Sequence | Risque |
|------|----------|--------|
| `network_then_sensitive_write` | Network access followed by sensitive file write | Exfiltration / supply-chain injection |
| `delete_then_force_push` | File deletion followed by force push | Irreversible data loss |
| `errors_then_test_modify` | Repeated errors followed by test modification | Test weakening to hide failures |

## Parametres

- **Sliding window**: 20 actions
- **Alert policy**: each rule fires once per window (no spam)
- 3 safety rules, all evaluated on every new action

## Integration

- `SessionContext` includes a `SequenceMonitor` instance
- Hooks feed actions into the monitor and check for alerts
- Alerts are surfaced in hook responses

## References

- Pro2Guard (program-level provenance graphs for process-level intrusion detection)
- Spera 2026 (sequence-aware safety monitoring)
- SentinelAgent (multi-step attack detection in agent systems)
