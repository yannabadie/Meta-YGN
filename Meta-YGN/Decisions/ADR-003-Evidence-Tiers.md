---
title: "ADR-003: Evidence Tiers"
type: adr
status: accepte
date: 2026-03-01
tags:
  - adr
  - methodology
---

# ADR-003: 3 Tiers d'Evidence

## Contexte

Comment eviter de presenter des hypotheses comme des faits ?
Le projet a des features a differents stades de maturite.

## Decision

3 tiers appliques a toute claim, note d'architecture, et feature :

| Tier | Signification | Requis pour promotion |
|------|--------------|----------------------|
| `[confirmed]` | Verifie par test, type check, build, docs autoritatives | Test + production |
| `[experimental]` | Hypothese de travail, peut echouer | Tests unitaires |
| `[original-proposal]` | Idee nouvelle, pas encore validee | Rien |

## Consequences

- Positives : honnetete intellectuelle, pas de feature washing
- Negatives : friction supplementaire pour chaque claim
- Applique : dans le README, dans les docs, dans ce vault Obsidian

## Application dans le vault

Chaque note de feature dans [[00-Features-MOC]] porte son tier.
Le template [[Templates/Feature]] inclut un champ `evidence_tier`.
