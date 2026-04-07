---
title: "Session Replay"
type: feature
evidence_tier: confirmed
crate: metaygn-cli
tags:
  - feature
  - confirmed
  - replay
  - observability
created: 2026-04-07
---

# Session Replay

**Tier**: `[confirmed]`
**Crate**: `metaygn-cli` + `metaygn-memory`
**Fichier principal**: `crates/cli/src/main.rs` (fonction `cmd_replay`) + `crates/memory/src/store.rs`

## Description

Timeline complete des hooks d'une session avec latence par event. Stocke chaque requete/reponse de hook dans SQLite et permet de rejouer la sequence via CLI.

## Implementation

- **Storage** (`store.rs`) : table `replay_events` avec colonnes session_id, hook_event, request_json, response_json, latency_ms, timestamp. Index sur (session_id, timestamp).
- `record_replay_event()` : insere un event de replay
- `replay_sessions()` : liste toutes les sessions avec event count, first/last timestamp
- `replay_events(session_id)` : retourne tous les events d'une session ordonnee par id
- **CLI** (`cmd_replay`) : sans argument, affiche la liste des sessions ; avec session_id, affiche la timeline detaillee avec `[N] HookEvent (latency_ms) @ timestamp` et les request/response tronques a 120 chars
- Accessible via `aletheia replay` ou `aletheia replay <session-id>`

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | store.rs (SQLite integration) | pass |
| Test e2e | `aletheia replay` avec sessions enregistrees | pass |
| Production | chaque hook est enregistre automatiquement | valide |

## Limitations connues

- Les request/response JSON sont tronques a 120 chars dans l'affichage CLI.
- Pas de filtre par hook_event ou plage de temps dans la commande replay.
- Les donnees de replay grandissent indefiniment (pas de rotation automatique).
