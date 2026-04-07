---
title: Entropy Calibration (EGPO)
type: feature
evidence_tier: confirmed
updated: 2026-04-07
crate: metaygn-core
tags:
  - feature
  - confirmed
  - calibration
created: 2026-04-07
---

# Entropy Calibration (EGPO)

**Tier** : `[confirmed]`
**Crate** : `metaygn-core`
**Fichier** : `crates/core/src/heuristics/entropy.rs`

## Description

Calibre la confiance en utilisant l'entropie des predictions.
EntropyTracker avec fenetre glissante de (confiance, succes).
Trigger penalite de surconfiance quand score > 0.3.

## Implementation

- **EntropyTracker** : fenetre glissante, calcul entropie Shannon
- **Integration** : stage `calibrate` (stage 9 du control loop)
- **Seuil** : score > 0.3 → penalite de surconfiance
- **Effet** : ajuste le vecteur de confiance a la baisse

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | entropy_test.rs | pass |
| Tests stage | 92 assess tests cover entropy integration | pass |
| Production | — | non valide |

## Transition experimental → confirmed

- [x] Valide au niveau code par tests exhaustifs (v2.0)
- [ ] Valide en production sur 50+ sessions
- [ ] Mesure Brier score avant/apres
- [ ] Pas de regression sur les taches low-risk (pas de faux positifs)

## Paper de reference

EGPO — Entropy-Guided Policy Optimization (calibration confiance via entropie)
