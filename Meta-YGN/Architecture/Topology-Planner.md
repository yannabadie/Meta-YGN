---
title: Topology Planner
type: architecture
tags:
  - architecture
  - topology
updated: 2026-04-07
---

# Topology Planner

**Fichier** : `crates/core/src/topology.rs`
**Status** : `[experimental]` — code reel, 4 topologies implementees, non battle-teste

## Topologies

| Topologie | Stages | Critere de selection |
|-----------|--------|---------------------|
| **Single** | classify, assess, act, decide (4) | Low risk ET difficulte < 0.2 |
| **Vertical** | Toutes les 12 stages | Default |
| **Horizontal** | 14 stages (double verify + calibrate) | Security tasks OU High risk |
| **Research** | classify, assess, competence, strategy, act, learn (6) | TaskType = Research |

## Logique de selection

```
if task_type == Security || risk == High:
    → Horizontal (14 stages, thorough)
elif task_type == Research:
    → Research (6 stages, focused)
elif risk == Low && difficulty < 0.2:
    → Single (4 stages, fast)
else:
    → Vertical (12 stages, default)
```

## Impact sur la latence

- **Single** : ~4 stages, le plus rapide (taches triviales)
- **Vertical** : ~12 stages, default
- **Horizontal** : ~14 stages, le plus lent mais le plus sur (security)
- **Research** : ~6 stages, skip verification (taches exploratoires)

## Questions ouvertes

- La selection est-elle correcte dans la pratique ?
- Les seuils (difficulty < 0.2) sont-ils calibres ?
- Faut-il ajouter d'autres topologies ?
