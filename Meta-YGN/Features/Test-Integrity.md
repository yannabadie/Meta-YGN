---
title: Test Integrity Checker
type: feature
evidence_tier: confirmed
crate: metaygn-verifiers
tags:
  - feature
  - confirmed
  - verification
created: 2026-04-07
---

# Test Integrity Checker

**Tier** : `[confirmed]`
**Crate** : `metaygn-verifiers`
**Fichier** : `crates/verifiers/src/test_integrity.rs`

## Description

Detecte quand des tests sont affaiblis : assertions supprimees,
valeurs attendues modifiees, blocs de test commentes.
Demande confirmation avant de laisser passer.

## Implementation

- Analyse les diffs de fichiers de test
- Detecte : suppression d'`assert`, modification de valeurs expected, commentaire de blocs test
- Trigger : PostToolUse sur Write/Edit de fichiers `*test*`

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | test_integrity_test.rs | pass |
| Test e2e | integration_e2e.rs | pass |

## Pourquoi c'est important

Empeche le pattern ou un agent "fixe" un test en le rendant trivial
plutot qu'en corrigeant le code sous-jacent.
