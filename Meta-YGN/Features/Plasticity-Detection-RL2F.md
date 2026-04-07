---
title: "Plasticity Detection (RL2F)"
type: feature
evidence_tier: confirmed
crate: metaygn-daemon
tags:
  - feature
  - confirmed
  - plasticity
  - rl2f
  - profiler
created: 2026-04-07
---

# Plasticity Detection (RL2F)

**Tier**: `[confirmed]`
**Crate**: `metaygn-daemon`
**Fichier principal**: `crates/daemon/src/profiler/plasticity.rs`

## Description

Tracking implicite de l'efficacite des recovery prompts injectes par le context pruner. Classifie la plasticite en 3 niveaux (Responsive, Degraded, Lost) et amplifie les prompts de recovery quand la plasticite se degrade.

## Implementation

- `PlasticityTracker` : compteurs total_injections, successes, failures, consecutive_failures
- `RecoveryOutcome` : enum Success/Failure infere depuis les hook events suivants (pas de feedback explicite)
- `PlasticityLevel` : Responsive (0 echecs consecutifs), Degraded (1 echec), Lost (2+ echecs)
- `plasticity_score()` : fraction successes/total, defaut optimiste a 1.0 quand aucun recovery observe
- `is_low_plasticity()` : score < 0.3
- `amplification_level()` : 1 (standard), 2 (emphatic), 3 (escalated) selon consecutive_failures
- `has_pending_recovery()` : detection d'injection non encore resolue
- Un seul succes remet le niveau a Responsive (reset de consecutive_failures)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | plasticity.rs (5 tests inline) | pass |
| Test e2e | integration avec fatigue profiler | pass |
| Production | actif dans le daemon | valide |

## Limitations connues

- Le feedback est implicite (infere) : pas de confirmation explicite de l'utilisateur.
- Le seuil de low plasticity (0.3) et les niveaux d'amplification sont hard-codes.
- Pas de persistence entre sessions (le tracker est reinitialise a chaque session).
