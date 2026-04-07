---
title: UCB Memory Retrieval
type: feature
evidence_tier: confirmed
updated: 2026-04-07
crate: metaygn-memory
tags:
  - feature
  - confirmed
  - memory
created: 2026-04-07
---

# UCB Memory Retrieval (U-Mem)

**Tier** : `[confirmed]`
**Crate** : `metaygn-memory`
**Fichier** : `crates/memory/src/graph.rs` (`adaptive_recall`)

## Description

Scoring de retrieval memoire par Upper Confidence Bound.
Blend 70% cosine similarity + 30% UCB exploration bonus.
Feedback bandit via `record_recall_reward`.

## Implementation

- **adaptive_recall** : score = 0.7 * cosine + 0.3 * UCB
- **UCB** : sqrt(2 * ln(total_recalls) / hit_count_i) — favorise les noeuds peu consultes
- **Feedback** : `record_recall_reward(node_id, reward)` met a jour reward_sum
- **Tracking** : hit_count, reward_sum, access_count par noeud

## Pourquoi UCB

Cosine seul favorise toujours les memes noeuds "proches".
UCB ajoute un bonus d'exploration pour les noeuds rarement consultes
mais potentiellement utiles — exploration-exploitation classique.

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | ucb_test.rs | pass |
| Tests integration | adaptive_recall wired into UnifiedSearch, real BM25 scores | pass |
| Production | — | non valide |

## Transition experimental → confirmed

- [x] Valide au niveau code par tests exhaustifs (v2.0)
- [ ] Valide en production sur 50+ sessions

## Paper de reference

U-Mem — Uncertainty-Aware Memory (UCB-scored retrieval ranking)
