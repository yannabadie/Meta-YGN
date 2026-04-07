---
title: Fatigue Profiler
type: feature
evidence_tier: confirmed
crate: metaygn-daemon
tags:
  - feature
  - confirmed
  - safety
created: 2026-04-07
---

# Fatigue Profiler

**Tier** : `[confirmed]`
**Crate** : `metaygn-daemon`
**Fichier** : `crates/daemon/src/profiler/`

## Description

Track la plasticite de recovery des erreurs et escalade progressivement :
1. **Hint** : suggestion legere
2. **Critique** : feedback plus appuye
3. **Auto-escalate** : escalade automatique vers l'humain

## Implementation

- Mesure : ratio erreurs recurrentes / erreurs nouvelles
- Fenetre glissante sur N dernieres actions
- Escalade quand le ratio depasse un seuil
- Per-session (pas persistant cross-sessions)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | fatigue_test.rs | pass |
| Test e2e | integration_e2e.rs | pass |

## Lien avec Plasticity Detection

La [[Plasticity-Detection-RL2F]] (`[experimental]`) etend ce concept
avec un feedback implicite qui ajuste l'amplification de recovery
basee sur la recurrence des erreurs (RL2F).
