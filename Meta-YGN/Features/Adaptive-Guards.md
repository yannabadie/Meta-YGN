---
title: Adaptive Guard Memory
type: feature
evidence_tier: experimental
crate: metaygn-verifiers
tags: [feature, experimental, adaptation, guards]
created: 2026-04-08
---

# Adaptive Guard Memory (AGrail-inspired)

**Tier**: `[experimental]`
**Crate**: `metaygn-verifiers`
**Fichier**: `crates/verifiers/src/adaptive.rs`

## Description

Track l'efficacite de chaque regle de guard (TP/FP) depuis les feedbacks
de session. Permet de desactiver automatiquement les regles trop agressives
(over-refusal) sans appel LLM. Inspire de AGrail (ACL 2025).

## Implementation

- **GuardEffectiveness** : hit_count, true_positive, false_positive par regle
- **effectiveness()** : TP / (TP + FP)
- **should_disable()** : effectiveness < seuil ET observations >= min
- **Regles critiques** (DestructiveGuard) ne sont JAMAIS desactivables

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | adaptive_test.rs (11 tests) | pass |
| Production | — | non valide |

## Paper de reference

AGrail: A Lifelong Agent Guardrail (ACL 2025, arxiv 2502.11448)
