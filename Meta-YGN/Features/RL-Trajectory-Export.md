---
title: "RL Trajectory Export"
type: feature
evidence_tier: experimental
crate: metaygn-shared
tags:
  - feature
  - experimental
  - rl2f
  - trajectory
  - export
created: 2026-04-07
---

# RL Trajectory Export

**Tier**: `[experimental]`
**Crate**: `metaygn-shared` + `metaygn-memory` + `metaygn-cli`
**Fichier principal**: `crates/shared/src/trajectory.rs` + `crates/memory/src/store.rs`

## Description

Export de trajectoires signees en JSONL pour fine-tuning RL offline. Chaque trajectoire capture le cycle complet : tentative initiale, erreur, critique, revision, et resultat.

## Implementation

- **Rl2fTrajectory** (trajectory.rs) : struct avec session_id, task_type, risk_level, strategy_used, initial_attempt, verifiable_error, critique_injected, revised_attempt, success, overconfidence_score, plasticity_level, confidence, coherence, grounding, timestamp, signature_hash
- **Storage** (store.rs) : table `rl2f_trajectories` (session_id, trajectory_json, signature_hash, timestamp), index sur session_id
- `save_trajectory()` : insere une trajectoire serialisee en JSON avec signature optionnelle
- `export_trajectories(limit)` : retourne les trajectoires recentes (ordonnees par timestamp DESC)
- **CLI** (`aletheia export --limit N`) : interroge le daemon via `/trajectories/export`, ecrit un fichier JSONL horodate dans `~/.claude/aletheia/trajectories/export-YYYYMMDD-HHMMSS.jsonl`

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | -- | -- |
| Test e2e | `aletheia export` avec trajectoires enregistrees | pass |
| Production | -- | non valide a l'echelle |

## Limitations connues

- Le format JSONL n'est pas encore aligne sur un standard RL specifique (ex: D4RL, RLAIF).
- La signature_hash est optionnelle et son schema de verification n'est pas implemente.
- Pas de filtre par session, type de tache, ou plage de dates dans l'export.
- Experimental : la structure de la trajectoire peut evoluer.
