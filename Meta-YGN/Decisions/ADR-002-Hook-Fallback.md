---
title: "ADR-002: Hook Fallback"
type: adr
status: accepte
date: 2026-02-15
tags:
  - adr
  - reliability
---

# ADR-002: Fallback Local si Daemon Down

## Contexte

Le daemon peut ne pas etre lance, ou crasher en cours de session.
Le plugin doit continuer a fonctionner.

## Decision

`fallback.ts` implémente des heuristiques locales basees regex :
- Risk patterns pour les commandes dangereuses
- Budget tracking local
- Pas de memory, pas d'evolution, pas de calibration

## Consequences

- Positives : le plugin fonctionne toujours, degradation gracieuse
- Negatives : sans daemon, pas de guard pipeline complete, pas de learning
- Le client daemon a un timeout de 350ms — un echec est quasi-invisible pour l'utilisateur
