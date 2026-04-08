---
title: Datalog Policy Engine
type: feature
evidence_tier: experimental
crate: metaygn-core
tags: [feature, experimental, policy, datalog]
created: 2026-04-08
---

# Datalog Policy Engine (crepe)

**Tier**: `[experimental]`
**Crate**: `metaygn-core`
**Fichier**: `crates/core/src/policy_engine.rs`

## Description

Moteur de politiques de securite declaratif utilisant Datalog via le crate `crepe`.
Les regles sont compilees en Rust natif (zero overhead runtime). Remplace les
regles imperatives du SequenceMonitor par des regles composables.

## Implementation

- **3 regles Datalog** : network_then_sensitive_write, delete_then_force_push, errors_then_test_modify
- **Compilation** : regles compilees via macro procedurale `crepe!` — micro-secondes d'evaluation
- **Input** : actions symboliques (step, action_type_id, target_type_id)
- **Output** : violations deduplicees par regle
- **Extensibilite** : ajouter une regle = ajouter une clause Datalog, pas du code Rust

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | policy_engine_test.rs (8 tests) | pass |
| Production | — | non valide |

## Limitation

Regles compile-time uniquement. Pas de chargement runtime (limitation du crate crepe).
Pour du hot-reload, migration vers `ascent` necessaire.

## Paper de reference

PCAS: Policy Compiler for Secure Agentic Systems (arxiv 2602.16708)
