---
title: Memory System
type: architecture
tags:
  - architecture
  - memory
updated: 2026-04-07
---

# Systeme Memoire

**Crate** : `metaygn-memory`
**Status** : `[confirmed]`
**LOC** : ~1,200+ (src)

## GraphMemory (`graph.rs`, 654 lignes)

### Types de noeuds
Task, Decision, Evidence, Tool, Agent, Code, Error, Lesson

### Types d'edges
DependsOn, Produces, Verifies, Contradicts, Supersedes, RelatedTo

### Scopes
Session, Project, Global

### Stockage
- SQLite : tables `graph_nodes`, `graph_edges`
- **FTS5** : full-text search sur le contenu des noeuds
- **Embeddings** : optionnel (fastembed bge-small-en-v1.5, 384-dim), fallback hash
- **Cosine similarity** : recherche semantique

### UCB Scoring `[experimental]`
- `hit_count` + `reward_sum` pour exploration-exploitation
- `adaptive_recall` : blend 70% cosine + 30% UCB exploration bonus
- `record_recall_reward` : feedback bandit apres utilisation

### Access tracking
- `access_count` par noeud, mis a jour a chaque lecture

## MemoryStore (`store.rs`, 580 lignes)

### Tables SQLite

| Table | Contenu |
|-------|---------|
| events | id, session_id, event_type, payload, timestamp (FTS5) |
| heuristic_versions | id, generation, parent_id, fitness, risk_weights, strategy_scores |
| session_outcomes | session_id, task_type, risk_level, strategy, success, tokens, duration, errors |
| replay_events | session_id, hook_event, request/response JSON, latency_ms |

### Pragmas
- WAL (write-ahead logging)
- PRAGMA synchronous=NORMAL
- PRAGMA cache_size=-64000 (64MB)
- Index sur session_id, event_type, replay events

### Acces async
`tokio-rusqlite` pour queries non-bloquantes.
