# v0.9.0 "Calibrated Mind" — Design Document

**Date**: 2026-03-01
**Status**: Approved

## Goal
Make MetaYGN detectably smarter: entropy-based overconfidence detection, plasticity loss monitoring, exploration-scored memory retrieval, and structured trajectory export for external RL pipelines.

## Research Foundation
- **EGPO** (arXiv:2602.22751) — entropy proxy for metacognitive calibration
- **RL2F** (Klissarov et al., DeepMind 2026) — plasticity loss detection, trajectory capture
- **U-Mem** (arXiv:2602.22406) — UCB-scored active memory retrieval
- **RLVR** — verifiable reward trajectories for RL fine-tuning

## 4 Features

### 1. Entropy Calibration (EGPO)

New `EntropyTracker` in `crates/core/src/stages/calibrate.rs`:
- Sliding window of 20 `(confidence, was_correct)` pairs
- Shannon entropy over binned confidences
- `overconfidence_score`: fraction of high-conf (>0.7) decisions that were wrong
- When `overconfidence_score > 0.3`: penalize `MetacognitiveVector.confidence`, signal `decide` stage to prefer Revise/Escalate
- Expose in `FatigueReport` and `/profiler/fatigue` endpoint

Integration: calibrate.rs (stage 9), learn.rs (stage 12), HeuristicEvolver fitness penalty.

### 2. Plasticity Detection (RL2F)

Extend existing `PlasticityTracker` in `crates/daemon/src/profiler/plasticity.rs`:
- Three levels: Responsive → Degraded → Lost
- After recovery injection: track if next tool output shows improvement
- Responsive: error pattern changed (good)
- Degraded: same error class recurs once → amplify feedback (CAPS, restructure)
- Lost: same error recurs 2+ times → force Escalate via decide stage
- Feed plasticity level into `MetacognitiveVector.confidence` via calibrate stage
- Expose `plasticity_level` in health/fatigue endpoints

Integration: verify.rs (stage 8), calibrate.rs (stage 9), decide.rs (stage 11), proxy/pruner.rs.

### 3. UCB-Scored Memory Retrieval (U-Mem)

Add `hit_count` (u32) and `reward_sum` (f64) fields to `MemoryNode` in `graph.rs`:
- UCB1 scoring: `score = mean_reward + sqrt(2 * ln(total_queries) / hit_count)`
- New `adaptive_recall` method blending 70% cosine + 30% UCB exploration bonus
- After successful session: increment `reward_sum` on recalled nodes
- Track `total_recall_queries` counter in store
- Opt-in via `?strategy=ucb` query param on `/memory/recall`

Integration: graph.rs schema + method, daemon/api/memory.rs, learn.rs (stage 12).

### 4. RL Trajectory Export (RL2F/RLVR)

New `Rl2fTrajectory` struct in `crates/shared/src/state.rs`:
- Fields: session_id, task_type, risk_level, strategy_used, initial_attempt, verifiable_error, critique_injected, revised_attempt, outcome, calibration_snapshot, overconfidence_score, plasticity_level
- Captured in learn.rs (stage 12) from LoopContext
- Persisted to `rl2f_trajectories` SQLite table
- Signed with evidence pack hash chain for integrity
- `GET /trajectories/export` endpoint returns signed JSONL
- CLI `aletheia export` dumps to `~/.claude/aletheia/trajectories/`

Integration: shared/state.rs, memory/store.rs, core/stages/learn.rs, daemon/api/trajectories.rs, cli/main.rs.

## EGPO × RL2F Synergy

Together, these features distinguish:
- **Calibrated agent**: confident & right (trustworthy)
- **Lucky agent**: confident & wrong, but plastic (recovers with feedback)
- **Stubborn agent**: confident & wrong, ignores feedback (needs escalation)

The `overconfidence_score` (EGPO) + `plasticity_level` (RL2F) form a 2D classification that the decide stage uses to route decisions.

## Evidence Tags
- Entropy calibration: `[experimental]` — requires 20+ sessions to converge
- Plasticity detection: `[experimental]` — behavioral heuristic, not formal proof
- UCB retrieval: `[experimental]` — needs sufficient node population to explore
- Trajectory export: `[confirmed]` — deterministic capture + hash signing
