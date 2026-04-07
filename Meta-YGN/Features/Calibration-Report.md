---
title: "Calibration Report"
type: feature
evidence_tier: confirmed
crate: metaygn-cli
tags:
  - feature
  - confirmed
  - calibration
  - evaluation
created: 2026-04-07
---

# Calibration Report

**Tier**: `[confirmed]`
**Crate**: `metaygn-cli`
**Fichier principal**: `crates/cli/src/main.rs` (fonction `cmd_eval`)

## Description

Rapport de calibration avec Brier score reel et buckets de calibration. Affiche les metriques accumulees : sessions, events, graph, heuristiques, fatigue, et calibration predicted vs actual par tranche de confiance.

## Implementation

- Commande `aletheia eval` : interroge le daemon via HTTP pour aggreer les donnees
- Collecte : session replay count, memory event count, graph nodes/edges
- Minimum 5 sessions requises pour afficher les metriques de calibration
- Brier score reel via endpoint `/calibration` du daemon (sample_count + score)
- Buckets de calibration : pour chaque tranche, affiche `avg_predicted`, `avg_actual`, et `count`
- Affiche aussi : best heuristic fitness, verification success rate, fatigue score
- Rendu en box-drawing Unicode dans le terminal

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | -- | -- |
| Test e2e | `aletheia eval` avec daemon actif | pass |
| Production | disponible via CLI | valide |

## Limitations connues

- Necessite au moins 5 sessions pour produire des metriques significatives.
- Le Brier score depend de la qualite des predictions de confiance enregistrees.
- Pas d'export du rapport en fichier (affichage terminal uniquement).
