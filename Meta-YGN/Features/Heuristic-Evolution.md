---
title: Heuristic Evolution
type: feature
evidence_tier: confirmed
updated: 2026-04-07
crate: metaygn-core
tags:
  - feature
  - confirmed
  - evolution
created: 2026-04-07
---

# Heuristic Evolution (Layer 0)

**Tier** : `[confirmed]`
**Crate** : `metaygn-core`
**Fichier** : `crates/core/src/heuristics/evolver.rs`

## Description

Population de `HeuristicVersion` avec evolution basee sur mutation et selection.
Fitness scoree sur : taux de succes, tokens consommes, latence.
Declenchee apres 5 outcomes accumules.

## Implementation

- **Population** : N versions d'heuristiques avec genotypes (risk_weights, strategy_scores)
- **Mutation** : perturbation gaussienne des poids
- **Selection** : fitness = f(success_rate, token_efficiency, latency)
- **Persistance** : table `heuristic_versions` dans SQLite
- **Trigger** : `SessionEnd` hook lance l'evolution si 5+ outcomes

## Limitations connues

> [!warning] Convergence lente
> 5 outcomes minimum avant trigger signifie que l'evolution est lente
> en usage normal (1-2 sessions/jour = 2-5 jours avant un cycle d'evolution).
> Sur des sessions courtes, le trigger peut ne jamais se declencher.

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | heuristics_test.rs | pass |
| Tests exhaustifs | population + mutation + selection tests (v2.0) | pass |
| Production | — | non valide a l'echelle |

## Transition experimental → confirmed

- [x] Valide au niveau code par tests exhaustifs (v2.0)
- [ ] Mesurer convergence sur 100+ sessions
- [ ] Comparer fitness generation N vs generation 0
- [ ] Verifier que les mutations n'introduisent pas de regressions
