---
title: "MCP Bridge"
type: feature
evidence_tier: confirmed
crate: metaygn-daemon
tags:
  - feature
  - confirmed
  - mcp
  - integration
created: 2026-04-07
---

# MCP Bridge

**Tier**: `[confirmed]`
**Crate**: `metaygn-daemon`
**Fichier principal**: `crates/daemon/src/mcp.rs`

## Description

5 outils MCP exposes via un serveur stdio fusionne dans le daemon. Acces direct a `AppState` sans round-trip HTTP. Gate par `#[cfg(feature = "mcp")]`.

## Implementation

- Utilise le framework `rmcp` (ServerHandler, ToolRouter, tool macro)
- **metacog_classify** : classifie un prompt utilisateur (risk, intent, tool-necessity) via les 6 premiers stages du control loop
- **metacog_verify** : verifie la sortie d'un outil (stages verify + calibrate, indices 7..10)
- **metacog_recall** : recherche semantique dans la memoire (events episodiques, heuristiques)
- **metacog_status** : retourne le statut metacognitif (fatigue, budget, heuristiques, graph/event counts)
- **metacog_prune** : analyse un tableau de messages pour detecter les boucles d'erreur et suggerer la compaction
- Le serveur s'identifie comme "aletheia-nexus" avec version `CARGO_PKG_VERSION`
- Lance via `aletheia mcp` qui delegue a `aletheiad --mcp`

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | -- | -- |
| Test e2e | stdio server avec Claude Code plugin | pass |
| Production | actif quand feature `mcp` est activee | valide |

## Limitations connues

- Feature-gated : necessite `--features mcp` a la compilation.
- Pas de transport HTTP/SSE pour les clients non-stdio.
- Les parametres `tool_input` sont optionnels pour les callers MCP (pas de structured tool input).
