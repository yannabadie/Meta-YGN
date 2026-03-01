# Active Context

## Current State

v0.9.0 "Calibrated Mind" is feature-complete. All eleven phases are done. This
release adds entropy-based overconfidence detection, plasticity loss monitoring,
UCB-scored memory retrieval, and structured RL trajectory export.

## v0.9.0 Completion Summary

### Entropy Calibration (EGPO-inspired)
- `EntropyTracker` with sliding window of (confidence, was_correct) pairs
- Detects overconfidence: fraction of high-confidence (>0.7) decisions that were wrong
- Wired into calibrate stage (stage 9): applies confidence penalty when overconfidence > 0.3

### Plasticity Detection (RL2F-inspired)
- `PlasticityLevel` enum: Responsive (0 failures) → Degraded (1) → Lost (2+)
- Extends existing `PlasticityTracker` with `plasticity_level()` and `is_plasticity_lost()`

### UCB-Scored Memory (U-Mem-inspired)
- `adaptive_recall`: blends 70% cosine similarity + 30% UCB exploration bonus
- `record_recall_reward`: bandit-style feedback on recalled nodes

### RL Trajectory Export (RL2F/RLVR)
- `Rl2fTrajectory` struct + `rl2f_trajectories` SQLite table
- `GET /trajectories/export` daemon endpoint + `aletheia export` CLI command

## Remaining Limitations
- EntropyTracker needs 20+ sessions to converge [experimental]
- UCB scoring uses linear scan (no ANN index for scale)
- No WASM sandbox backend yet

## What Comes Next (v0.10.0+)
- Dialectic topology (RL2F teacher-student 3-call loop)
- LLM-driven heuristic mutation (AlphaEvolve-inspired)
- WASM sandbox backend (feature-gated)
- Marketplace publication
