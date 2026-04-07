---
title: "Dynamic Topology"
type: feature
evidence_tier: experimental
crate: metaygn-core
tags:
  - feature
  - experimental
  - topology
  - control-loop
created: 2026-04-07
---

# Dynamic Topology

**Tier**: `[experimental]`
**Crate**: `metaygn-core`
**Fichier principal**: `crates/core/src/topology.rs`

## Description

Selection dynamique du sous-ensemble des 12 stages du control loop a executer, en fonction du risque, de la difficulte, et du type de tache. Le principe est que le skip des stages inutiles est ce qui compte.

## Implementation

- `ALL_STAGES` : 12 stages ordonnes (classify, assess, competence, tool_need, budget, strategy, act, verify, calibrate, compact, decide, learn)
- `TopologyPlanner::plan(risk, difficulty, task_type)` retourne un `ExecutionPlan` (topology, stages, rationale)
- **Single** (4 stages) : risk=Low + difficulty < 0.2 -> classify, assess, act, decide
- **Vertical** (12 stages) : pipeline sequentiel complet par defaut
- **Vertical/Research** (6 stages) : task_type=Research -> classify, assess, competence, strategy, act, learn
- **Horizontal** (14 stages) : risk=High ou task_type=Security -> 12 stages + double verify + calibrate
- Implemente le trait `TopologyPolicy` (object-safe, `Box<dyn TopologyPolicy>` supporte)
- Feature gate : aucun (toujours compile)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | topology.rs (4 tests inline) | pass |
| Test e2e | -- | non |
| Production | -- | non valide a l'echelle |

## Limitations connues

- La selection de topologie est deterministe (pas de feedback loop pour ajuster dynamiquement).
- Le seuil de difficulte pour Single (0.2) est arbitraire et non calibre.
- Pas de topologie custom configurable par l'utilisateur.
- Experimental : pas encore valide en conditions reelles a l'echelle.
