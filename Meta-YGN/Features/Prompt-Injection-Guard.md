---
title: Prompt Injection Guard
type: feature
evidence_tier: confirmed
crate: metaygn-core
tags:
  - feature
  - confirmed
  - security
created: 2026-04-07
---

# Prompt Injection Guard

**Tier**: `[confirmed]`
**Crate**: `metaygn-core`
**Fichier principal**: `crates/core/src/stages/assess.rs`

## Description

Detecte les patterns d'injection de prompt courants et les classe comme HIGH risk.
Protege contre les tentatives de jailbreak via "ignore previous instructions",
"###(system_message)", et les attaques TODO-based.

## Implementation

- **Fonction**: `contains_prompt_injection_markers()`
- **Integration**: stage `assess` (stage 2), avant la classification par keywords
- **Patterns detectes**: 5 markers explicites + TODO-based avec termes risques
- **Feature gate**: non (toujours actif)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | assess.rs inline tests | pass |
| Test integration | assess_test.rs (92 tests) | pass |

## Pourquoi c'est important

Les agents de code sont exposes a du contenu non-trustee (fichiers, outputs MCP, messages).
La detection d'injection au niveau assess empeche l'agent d'executer des commandes
injectees avec un risque HIGH, declenchant les guards ou l'escalade.
