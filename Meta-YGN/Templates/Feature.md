---
title: "{{nom_feature}}"
type: feature
evidence_tier: confirmed | experimental | original-proposal
crate: "{{crate}}"
tags:
  - feature
  - "{{tier}}"
created: {{date}}
---

# {{nom_feature}}

**Tier**: `[{{evidence_tier}}]`
**Crate**: `{{crate}}`
**Fichier principal**: `crates/{{crate}}/src/{{fichier}}`

## Description (3 lignes max)



## Implementation

- **LOC**: 
- **Tests**: 
- **Feature gate**: oui (`--features {{flag}}`) / non

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | | pass/fail |
| Test e2e | | pass/fail |
| Production | | valide/non valide |

## Limitations connues


## Paper de reference

[[]] — 

## Transition experimental → confirmed

- [ ] Valide en production sur N sessions
- [ ] Metriques collectees
- [ ] Pas de regression detectee
