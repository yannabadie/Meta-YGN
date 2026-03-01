# v0.5.0 "Smart Recovery" — Design Document

**Date**: 2026-02-28
**Status**: Approved
**Approach**: Hook-based recovery + proxy infra prep + implicit feedback + UX polish

---

## 1. Context Recovery (Progressive Amplification + Proxy Infra)

### Current state
The ContextPruner detects 3+ consecutive errors and generates recovery prompts returned as `additionalContext` in hook responses.

### Improvement: Progressive amplification
Track recovery effectiveness. If a recovery prompt was injected but the same error pattern returns:
- **Level 1** (first detection): Standard recovery hint (current behavior)
- **Level 2** (same pattern after recovery): Reformulated critique with emphasis + concrete alternative strategy suggestion
- **Level 3** (third occurrence): Auto-trigger escalation context recommending `/metacog-escalate`

### Proxy infrastructure prep
Create `crates/daemon/src/proxy/server.rs` as a transparent HTTP reverse proxy listener. NOT activated by default. Only starts with `--proxy-mode` flag. Prepares for full MITM context pruning in v0.6.0.

## 2. Implicit Feedback (Plasticity Tracker)

Track whether recovery prompts actually work:
- After recovery injection, if next hook shows **same error** → `plasticity_failure` +1
- After recovery injection, if next hook shows **success** → `plasticity_success` +1
- `plasticity_score = successes / total` (0.0-1.0)
- Feed into `calibrate.rs` to lower confidence when LLM ignores critiques

Stored in daemon session state (AppState), not persisted.

## 3. UX Polish

- **Human-readable messages**: Replace terse internal format with clear sentences
- **Latency tracking**: Every hook response includes `[latency: Xms]`
- **`aletheia init`**: New CLI command that generates a starter `.claude/settings.json`
- **Risk classification fix**: Pre-tool-use currently shows `risk:high` for everything due to control loop running on the hook metadata, not the actual command

## 4. Bug Fixes

- Completion verifier regex (already fixed in previous commit)
- Pre-tool-use risk classification (control loop evaluates the command, not the hook event name)
