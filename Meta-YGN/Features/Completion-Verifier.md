---
title: "Completion Verifier"
type: feature
evidence_tier: confirmed
crate: metaygn-verifiers
tags:
  - feature
  - confirmed
  - verification
  - safety
created: 2026-04-07
---

# Completion Verifier

**Tier**: `[confirmed]`
**Crate**: `metaygn-verifiers`
**Fichier principal**: `crates/verifiers/src/completion.rs`

## Description

Verifie que les fichiers cites dans une reponse "Done!" existent reellement sur le filesystem. Detecte les claims de completion, de tests passants, et de compilation reussie pour empecher les fausses declarations.

## Implementation

- `extract_claims(text)` : extrait les chemins de fichiers (regex), detecte les marqueurs de completion ("done", "finished", "implemented", etc.), de tests ("tests pass", "all tests green"), et de compilation ("compiles", "builds successfully")
- `verify_files_exist(claims, cwd)` : verifie l'existence de chaque fichier mentionne via `Path::exists()`
- `verify_completion(text, cwd)` : pipeline complet retournant `VerificationResult` avec `verified`, `checks`, `blocking_issues`, et `warnings`
- Les fichiers manquants sont des `blocking_issues` ; les claims de tests/compilation sont des `warnings` (verification manuelle recommandee)
- Detection de completion sans fichiers mentionnes = warning supplementaire

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | completion.rs (pas de tests inline actuellement) | -- |
| Test e2e | hook Stop avec last_assistant_message | pass |
| Production | actif dans le hook Stop | valide |

## Limitations connues

- Ne verifie que l'existence des fichiers, pas leur contenu ni leur validite syntaxique.
- Les claims de tests et compilation ne sont pas verifiees automatiquement (seulement signalees en warning).
- Les regex de detection de chemins peuvent produire des faux positifs (ex: URLs, e.g., i.e.).
