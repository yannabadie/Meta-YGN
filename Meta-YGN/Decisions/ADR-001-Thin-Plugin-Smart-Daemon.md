---
title: "ADR-001: Thin Plugin, Smart Daemon"
type: adr
status: accepte
date: 2026-02-01
tags:
  - adr
  - architecture
---

# ADR-001: Plugin Fin, Daemon Intelligent

## Contexte

Comment repartir la logique entre le plugin Claude Code et le runtime ?
Le plugin a des contraintes (timeouts, pas de persistance, pas d'etat cross-hook).
Le daemon a la liberte (SQLite, etat, async, evolution).

## Decision

- Le **plugin** est un shell minimal : hooks TypeScript qui font un POST HTTP au daemon
- Le **daemon** (Rust) contient toute l'intelligence : control loop, guards, memory, evolution
- Si le daemon est down, les hooks utilisent des heuristiques locales (fallback)

## Alternatives considerees

| Option | Pour | Contre |
|--------|------|--------|
| **Thin plugin + smart daemon** | Separation propre, etat persistant, evolution | 2 processus, latence HTTP |
| Plugin lourd (tout en TS) | Un seul processus | Pas de persistance, pas d'evolution, timeouts |
| MCP-only | Standard, pas de daemon | Pas de hooks lifecycle, pas de gates |

## Consequences

- Positives : persistance cross-session, evolution heuristique, guard pipeline robuste
- Negatives : complexite de deployment (daemon + plugin), latence HTTP (attenuation: 350ms timeout)
- Le plugin fonctionne degradee sans daemon (fallback heuristiques locales)
