---
title: MCP Bridge — 5 Outils
type: architecture
tags:
  - architecture
  - mcp
updated: 2026-04-07
---

# MCP Bridge

**Fichier** : `crates/daemon/src/mcp.rs`
**Feature gate** : `#[cfg(feature = "mcp")]`
**Transport** : stdio (rmcp 0.17)
**Status** : `[confirmed]`

## 5 Outils MCP

| Outil | Params | Description |
|-------|--------|-------------|
| `metacog_classify` | ClassifyParams | Classifie risque + strategie |
| `metacog_verify` | VerifyParams | Guidance de verification |
| `metacog_recall` | RecallParams | Recherche memoire |
| `metacog_status` | — | Status daemon |
| `metacog_prune` | — | Integration ContextPruner |

## Acces

Les outils MCP accedent directement a `AppState` — pas de hop HTTP.
Mapping des champs MCP vers `HookInput` :
- `tool_name` → `tool`
- `tool_output` → `response`

## Limitations

> [!warning] Pas de tests d'integration MCP en CI
> Le feature gate `mcp` n'est pas teste en CI.
> Les outils sont testes unitairement mais pas en integration.
