---
title: "Guard Pipeline"
type: feature
evidence_tier: confirmed
crate: metaygn-verifiers
tags:
  - feature
  - confirmed
  - security
  - guards
created: 2026-04-07
---

# Guard Pipeline

**Tier**: `[confirmed]`
**Crate**: `metaygn-verifiers`
**Fichier principal**: `crates/verifiers/src/guard_pipeline.rs`

## Description

5 guards executent en parallele et evaluent chaque tool call via score agrege (MIN). 35+ regles regex couvrent les commandes destructives, les operations a haut risque, les chemins secrets, et les outils MCP non trustes.

## Implementation

- **DestructiveGuard** (score 0 = DENY) : 8 patterns (rm -rf /, sudo rm, mkfs, dd, shutdown, reboot, fork bomb, chmod 777 /)
- **HighRiskGuard** (score 30 = ASK) : 15 patterns (git push/reset, terraform, kubectl, curl|bash, sudo, docker, exfiltration, etc.)
- **SecretPathGuard** (score 20 = ASK) : 9 patterns (.env, secrets/, *.pem, *.key, id_rsa, credentials.json, .npmrc, .pypirc, kubeconfig)
- **McpGuard** (score 40 = ASK) : bloque tout outil `mcp__*` (data externe non trustee)
- **DefaultGuard** (score 100 = ALLOW) : baseline quand aucun guard ne matche
- Prompt injection detection dans `assess.rs` ajoute des patterns supplementaires (v2.0)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | guard_pipeline.rs (inline) | pass |
| Test e2e | hooks PreToolUse avec patterns dangereux | pass |
| Production | actif sur chaque PreToolUse | valide |

## Limitations connues

- Les patterns sont statiques (regex). Un obfuscation creative peut les contourner.
- McpGuard bloque tous les outils MCP sans distinction de risque individuel.
- Pas de liste blanche configurable par l'utilisateur (prevu post-MVP).
