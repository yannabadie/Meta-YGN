---
title: Control Loop — 12 Stages
type: architecture
tags:
  - architecture
  - control-loop
updated: 2026-04-07
---

# Boucle de Controle — 12 Etapes

Sequentielle, peut etre elaguee par le [[Topology-Planner]].

## Les 12 etapes

| # | Stage | Role | Crate |
|---|-------|------|-------|
| 1 | **classify** | Type de tache (Build/Test/Deploy/Security/Maintenance/Research) | core |
| 2 | **assess** | Score de risque (Low/Medium/High) via patterns outil + taille | core |
| 3 | **competence** | Auto-evaluation capacite (0-1) depuis historique session | core |
| 4 | **tool_need** | L'outil est-il vraiment necessaire ? (gate binaire) | core |
| 5 | **budget** | Allocation tokens pour cet appel (warning a 80%) | core |
| 6 | **strategy** | Strategie de raisonnement (Methodical/Rapid/Explorative) | core |
| 7 | **act** | Execution — **intentionnellement vide** (delegue a Claude Code) | core |
| 8 | **verify** | Verification output (test failure, file mod, patterns erreur) | verifiers |
| 9 | **calibrate** | Detection surconfiance via entropie, ajustement vecteur confiance | core |
| 10 | **compact** | Compaction memoire (clustering semantique de lecons) | memory |
| 11 | **decide** | Decision finale : Allow / Revise / Escalate | core |
| 12 | **learn** | Enregistrement lecons, mise a jour outcome session | memory |

## Topologies d'execution

Le [[Topology-Planner]] selectionne dynamiquement :

| Topologie | Stages | Quand |
|-----------|--------|-------|
| **Single** | 4 (classify, assess, act, decide) | Low risk + difficulte < 0.2 |
| **Vertical** | 12 (toutes) | Default |
| **Horizontal** | 14 (double verify + calibrate) | Security tasks ou High risk |
| **Research** | 6 (classify, assess, competence, strategy, act, learn) | Taches de recherche |

## Flux HTTP

1. Hook Claude Code → POST `/hooks/{hook_name}` → `HookInput`
2. Daemon cree `LoopContext`
3. `ControlLoop.run_plan()` execute les stages selon `ExecutionPlan`
4. System 2 async : graph insertion, entropy tracking, heuristic evolution
5. Retour `HookOutput` avec decision + budget + guidance

## Stage 7 (act) — Pourquoi c'est vide

Le daemon ne controle pas l'execution — c'est Claude Code qui execute les outils.
Le daemon ne fait que **decider** si l'outil devrait etre execute, avec quelle strategie.
C'est une separation deliberee : thin plugin (execution) / smart daemon (cognition).
