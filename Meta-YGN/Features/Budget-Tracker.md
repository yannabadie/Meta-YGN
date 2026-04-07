---
title: "Budget Tracker"
type: feature
evidence_tier: confirmed
crate: metaygn-shared
tags:
  - feature
  - confirmed
  - budget
  - observability
created: 2026-04-07
---

# Budget Tracker

**Tier**: `[confirmed]`
**Crate**: `metaygn-shared`
**Fichier principal**: `crates/shared/src/budget_tracker.rs`

## Description

Tracking per-session des tokens et couts USD. Emet un warning quand l'utilisation atteint 80% du budget configure. Le summary est inclus dans chaque reponse de hook.

## Implementation

- `SessionBudget::new(max_tokens, max_cost_usd)` : initialise avec seuil de warning a 80%
- `consume(tokens, cost_usd)` : enregistre la consommation (saturating_add pour les tokens)
- `utilization()` : fraction 0.0-1.0+ basee sur le MAX entre utilisation tokens et utilisation cout
- `should_warn()` : retourne `true` si utilization >= 0.80
- `is_over_budget()` : retourne `true` si tokens ou cout depasse le maximum
- `summary()` : string lisible `[budget: Xtok/$Y.YY used of Ztok/$W.WW | P%]`
- `remaining_tokens()` et `remaining_cost_usd()` : reste disponible avant cap

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | shared/tests/budget_tracker_test.rs | pass |
| Test e2e | integration avec daemon hooks | pass |
| Production | actif dans chaque session | valide |

## Limitations connues

- Le cout USD est estime (pas de feedback reel de l'API du provider LLM).
- Le warning threshold (80%) n'est pas configurable a runtime (hard-code).
- Pas de persistence du budget entre restarts du daemon (session-scoped).
