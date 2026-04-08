---
title: MOP Detector (Meltdown Onset Point)
type: feature
evidence_tier: experimental
crate: metaygn-core
tags: [feature, experimental, reliability, meltdown]
created: 2026-04-08
---

# MOP Detector

**Tier**: `[experimental]`
**Crate**: `metaygn-core`
**Fichier**: `crates/core/src/heuristics/mop.rs`

## Description

Detecte le point de collapse comportemental d'un agent via l'entropie Shannon
de la distribution des tool calls sur une fenetre glissante. Base sur
"Beyond pass@1" (arxiv 2603.29231).

## Implementation

- **Fenetre glissante** de 5 tool calls
- **Seuil entropie** theta_H = 1.711 bits (74% du max pour 5 outils distincts)
- **Spike condition** : H(t) - H(t-w) > 0 (toute augmentation)
- **Repetition ratio** complementaire pour detecter les boucles monotones
- **Detection latching** : une fois detecte, reste actif pour la session

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | mop_test.rs (12 tests) | pass |
| Production | — | non valide |

## Paper de reference

Beyond pass@1: A Reliability Science Framework (arxiv 2603.29231)
