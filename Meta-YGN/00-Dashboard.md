---
title: Meta-YGN Dashboard
type: moc
updated: 2026-04-07
---

# Aletheia-Nexus — Runtime Metacognitif

Control plane local-first pour coding agents. Daemon Rust + plugin Claude Code.
v2.2.0 "Sequence Sentinel" — Avril 2026.

## These produit

Le produit gagne s'il :
- detecte quand il a probablement tort
- demande des preuves avant une action risquee
- evite les appels d'outils inutiles
- escalade quand l'incertitude reste haute
- garde le contexte compact
- protege l'humain contre la sur-confiance systeme

## Etat du projet (Avril 2026)

| Composant | Status | Evidence |
|-----------|--------|----------|
| Guard Pipeline | `[confirmed]` | 5 guards, 28 regles, tests exhaustifs |
| Control Loop | `[confirmed]` | 12 etapes, toutes implementees, tests e2e |
| Memory System | `[confirmed]` | SQLite, FTS5, graph, UCB ranking |
| Budget Tracker | `[confirmed]` | Per-session, 80% warning |
| Test Integrity | `[confirmed]` | Detection d'affaiblissement d'assertions |
| Completion Verifier | `[confirmed]` | Verification existence fichiers |
| Fatigue Profiler | `[confirmed]` | Hint → critique → auto-escalate |
| Session Replay | `[confirmed]` | Timeline complete avec latence |
| MCP Bridge | `[confirmed]` | 5 outils, stdio, feature-gated |
| Entropy Calibration | `[confirmed]` | Tests exhaustifs, promoted dans README v2.0 |
| Plasticity Detection | `[confirmed]` | Tests exhaustifs, promoted |
| UCB Memory | `[confirmed]` | Tests exhaustifs, promoted |
| Heuristic Evolution | `[confirmed]` | Tests exhaustifs, promoted |
| Prompt Injection Guard | `[confirmed]` | Detection patterns injection dans assess stage |
| OTEL Exporter | `[confirmed]` | OTLP exporter feature-gated, wired |
| Dynamic Topology | `[experimental]` | 4 topologies, non battle-teste |
| Neural Embeddings | `[experimental]` | fastembed gate, fallback hash |
| RL Trajectory Export | `[experimental]` | JSONL signe, pas utilise pour training reel |
| Sequence Monitor | `[experimental]` | 3 regles, fenetre glissante, Pro2Guard-inspired |
| Semantic Router | `[experimental]` | kNN risk classification, context-aware override |

## Navigation

### Architecture
- [[00-Architecture-MOC|Architecture]] — Daemon, plugin, control loop
- [[Control-Loop|Boucle de controle]] — 12 etapes
- [[Guard-Pipeline|Guard Pipeline]] — 5 guards, 28 regles
- [[Hook-Lifecycle|Hooks]] — 8 hooks lifecycle
- [[Memory-System|Memoire]] — Graph + Store + FTS5

### Features
- [[00-Features-MOC|Features]] — Confirmees vs experimentales

### Skills & Agents
- [[00-Skills-Agents-MOC|Skills & Agents]] — 8 skills, 6 agents

### Decisions
- [[00-Decisions-MOC|Decisions]] — Choix architecturaux

## Chiffres cles

| Metrique | Valeur |
|----------|--------|
| Crates Rust | 7 |
| Rust LOC (src) | ~6,900 |
| Rust LOC (tests) | ~9,500+ |
| Fichiers de tests | 55+ |
| Packages TS | 2 (hooks, shared) |
| Guards/Regles | 5 / 28 |
| Stages control loop | 12 |
| Hooks lifecycle | 8 |
| MCP tools | 5 |
| Skills metacog | 8 |
| Agents | 6 |
| TODOs dans le code | 0 |

## Honnetete

> [!warning] Limitations
> - **Detection par regex contournable** : ADR-004 propose une architecture 5 tiers pour resoudre ce probleme
> - **Pas d'ADRs formels** : decisions dans les commentaires de code et daemon-contract.md
> - **Features experimentales non validees a l'echelle** : toutes marquees honnetement
> - **Pas de tests d'integration MCP** : feature-gated, pas teste en CI
> - **1 seul output style** : proof packet uniquement
> - **Evolution heuristique lente** : 5 outcomes minimum avant trigger
> - **Pas d'historique Telegram/externe** : Bot API ne donne pas de search/history

> [!success] Points forts
> - Code remarquablement propre : 0 TODO, 0 stubs, 0 dead code
> - Toutes les features `[experimental]` ont du vrai code (pas des stubs)
> - Evidence tiers (`confirmed`/`experimental`/`original-proposal`) appliques partout
> - Separation nette thin-plugin / smart-daemon
> - v2.0 : 0 mutex panics possibles dans le daemon (graceful degradation)
