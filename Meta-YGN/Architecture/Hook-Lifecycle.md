---
title: Hook Lifecycle — 8 Hooks
type: architecture
tags:
  - architecture
  - hooks
updated: 2026-04-07
---

# Hooks Lifecycle — 8 Events

**Config** : `hooks/hooks.json`
**Runtime** : `npx tsx ${CLAUDE_PLUGIN_ROOT}/packages/hooks/src/*.ts`
**Timeouts** : 10s (SessionStart), 5s (autres), async (SessionEnd)

## Hooks

| # | Hook | Quand | Stages executes | Timeout |
|---|------|-------|----------------|---------|
| 1 | **SessionStart** | Debut de session | Init, lire daemon port | 10s |
| 2 | **UserPromptSubmit** | Apres prompt user | classify, assess, competence, tool_need, budget, strategy | 5s |
| 3 | **PreToolUse** | Avant chaque outil | Guard pipeline + risk assessment | 5s |
| 4 | **PostToolUse** | Apres chaque outil | verify, fatigue update, graph population | 5s |
| 5 | **PostToolUseFailure** | Apres echec outil | Error recovery, plasticity tracking | 5s |
| 6 | **PreCompact** | Avant compaction contexte | Housekeeping pre-compaction | 5s |
| 7 | **Stop** | Fin de tache | calibrate, compact, decide, learn | 5s |
| 8 | **SessionEnd** | Fin de session | Persistence async, heuristic evolution trigger | async |

## Matchers

Les hooks PreToolUse/PostToolUse/PostToolUseFailure matchent sur le nom de l'outil :
- `Bash`
- `Write`
- `Edit`
- `MultiEdit`
- `NotebookEdit`
- `mcp__.*` (tous les outils MCP)

## Fallback sans daemon

`fallback.ts` (`evaluateFallback`) :
- Heuristiques locales basees regex
- Budget tracking local
- Degradation gracieuse — le plugin fonctionne sans daemon

## Client daemon

`daemon-client.ts` (`callDaemon`) :
- Lit le port depuis `~/.claude/aletheia/daemon.port`
- Timeout 350ms (AbortController)
- Echec silencieux (retourne null si daemon injoignable)
