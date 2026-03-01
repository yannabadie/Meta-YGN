# MetaYGN v0.3.0 — Paper Triage & Integration Assessment

**Date**: 2026-02-28  
**Scope**: 22 papers from arXiv (Feb 2026 batch) evaluated against MetaYGN architecture and Adaptive Topology roadmap  
**Method**: Each paper scored on 3 axes → Relevance (R), Feasibility (F), Impact (I) — scale 1-5, then weighted composite  

---

## Executive Summary

Sur 22 papers analysés, **6 sont directement actionnables** pour MetaYGN v0.3.0, **4 fournissent des insights architecturaux** précieux sans nécessiter d'implémentation directe, et **12 sont hors-scope** ou trop éloignés de l'architecture runtime/plugin-shell.

### Tier S — Intégrer maintenant (impact direct sur la roadmap v0.3.0)

| # | Paper | Phase MetaYGN | Score |
|---|-------|---------------|-------|
| 1 | U-Mem (Autonomous Memory Agents) | Phase 1 — Graph Memory | ★★★★★ |
| 2 | EGPO (Metacognitive Entropy Calibration) | Phase 4 — Heuristic Evolver | ★★★★★ |
| 3 | Cognitive Models & AI Algorithms Templates | Architecture globale | ★★★★☆ |
| 4 | SideQuest (KV Cache Management) | Phase 3 — Context Firewall | ★★★★☆ |

### Tier A — Insights architecturaux (lire, adapter les concepts)

| # | Paper | Phase MetaYGN | Score |
|---|-------|---------------|-------|
| 5 | GiGPO (Hierarchical Credit Assignment) | Phase 4 — Heuristic Evolver | ★★★☆☆ |
| 6 | SSR (Strategy Executability) | Phase 2 — Dynamic Topology | ★★★☆☆ |
| 7 | Manifold of Failure (Attraction Basins) | Guards & KERNEL | ★★★☆☆ |
| 8 | I-MCTS (Introspective Monte Carlo) | Phase 4 — Heuristic Evolver | ★★★☆☆ |

### Tier B — Veille stratégique (pas d'action immédiate)

| # | Paper | Raison |
|---|-------|--------|
| 9 | Tribalism among AI Agents | Warning pattern pour multi-agent, pas applicable au runtime solo |
| 10 | Exgentic (Unified Protocol) | Benchmarking, pas d'impact sur l'architecture interne |
| 11 | MiroFlow | Framework concurrent, pas de technique extractible |
| 12 | FIRE (Financial Benchmark) | Domain-specific, hors scope |

### Tier C — Hors périmètre

| # | Paper | Raison |
|---|-------|--------|
| 13 | ContextRL | Entraînement MLLM, MetaYGN est AI-agnostic runtime |
| 14 | dLLM (Diffusion LM) | Architecture modèle, pas runtime |
| 15 | NoRA (Non-linear LoRA) | Fine-tuning, pas runtime |
| 16 | MoDora (Document Analysis) | Task-specific, pas metacognition |
| 17 | OmniGAIA | Benchmark multimodal |
| 18 | Medical Visual Adaptation | Domain-specific |
| 19 | Personalized LLM Agents Survey | Survey général, peu actionnable |
| 20 | Expert Investment Teams | Domain-specific multi-agent finance |
| 21 | AI Research Assistant | Case study, pas technique extractible |
| 22 | MiroFlow | Deep research framework, architecture différente |

---

## Analyses détaillées — Tier S

---

### 1. U-Mem: Towards Autonomous Memory Agents
**arXiv: 2602.22406** | UC Berkeley / DeepMind (Feb 2026)

**Concept clé**: Les memory agents actuels sont *passifs* — ils ne stockent que ce qui arrive dans la conversation. U-Mem rend la mémoire *active* via :
- (i) **Cascade d'acquisition cost-aware** : self-signal → teacher LLM → tool-verified research → expert feedback (escalade progressive)
- (ii) **Thompson Sampling sémantique** : balance exploration/exploitation sur les mémoires pour éviter le cold-start bias

**Résultats** : +14.6 points sur HotpotQA (Qwen2.5-7B), +7.33 sur AIME25 (Gemini-2.5-flash) vs baselines mémoire classiques.

**Mapping MetaYGN — Phase 1 (Graph Memory) :**

| Concept U-Mem | Transposition MetaYGN | Fichier cible |
|---------------|----------------------|---------------|
| Cascade cost-aware | La mémoire `graph.rs` priorise: (1) Hot cache local → (2) FTS5 warm search → (3) cold semantic via embeddings → (4) fallback: demande au LLM de résumer le contexte manquant | `memory/src/graph.rs` |
| Thompson Sampling | Remplacer le retrieval déterministe par un score UCB (Upper Confidence Bound) sur les nœuds mémoire. Chaque nœud accumule `hit_count` et `reward_sum`, le score de retrieval = `mean_reward + sqrt(2*ln(total)/hit_count)` | `memory/src/retrieval.rs` |
| Active acquisition | Après chaque `session_end`, le daemon analyse les gaps dans le graph mémoire (nœuds avec faible confiance ou edges manquants) et génère des questions de clarification pour la prochaine session | `daemon/src/routes/memory.rs` |
| Cold-start mitigation | Initialiser le graph mémoire avec les 8 skills Markdown existants comme nœuds de seed (type=`skill`, embedding pré-calculé), évitant le démarrage à froid | `plugins/hooks/session_start.py` |

**Action concrète** : Ajouter au schema SQLite de `graph.rs` les champs `hit_count INTEGER DEFAULT 0`, `reward_sum REAL DEFAULT 0.0`, `last_accessed TIMESTAMP`. Implémenter `semantic_search_ucb()` qui combine cosine similarity ET UCB score.

**Risque** : Le Thompson Sampling nécessite assez de données pour converger. MVP : 20+ sessions minimum. Avant cela, fallback sur cosine pur.

---

### 2. EGPO: Know What You Know — Metacognitive Entropy Calibration
**arXiv: 2602.22751** (Feb 2026)

**Concept clé** : EGPO identifie le *uncertainty-reward mismatch* — les pipelines RLVR traitent les solutions haute-confiance et basse-confiance de manière identique. EGPO corrige via :
- **Entropy proxy zéro-coût** : estimé directement des token-level likelihoods, pas besoin de rollouts supplémentaires
- **Calibration asymétrique** : préserve le raisonnement correct, ne régule que les échecs surconfiants
- **Récupération de signaux** : extrait des gradients informatifs même des rollouts dégénérés (tout bon ou tout mauvais)

**Mapping MetaYGN — Phase 4 (Heuristic Evolver) :**

C'est **la pièce manquante** du pipeline de calibration de MetaYGN. Actuellement, le stage `calibrate` (étape 9/12) a un `CalibrationRecord` avec `predicted_confidence` et `actual_outcome`, mais **aucun mécanisme d'entropie interne**.

| Concept EGPO | Transposition MetaYGN | Fichier cible |
|-------------|----------------------|---------------|
| Entropy proxy | Calculer l'entropie de la distribution de confiance sur les N dernières prédictions du stage `assess`. Si l'entropie est basse ET les outcomes variés → surconfiance détectée | `core/src/stages/calibrate.rs` |
| Asymmetric calibration | Quand le modèle a raison + haute confiance → ne pas toucher. Quand il a tort + haute confiance → pénaliser le score de la stratégie dans l'evolver avec un multiplicateur `overconfidence_penalty = 1 + (confidence - 0.5) * wrong_flag` | `core/src/stages/strategy.rs` |
| Signal recovery | Même quand toutes les sessions sont "succès" (dégénéré), extraire de l'information en comparant les distributions d'entropie entre stratégies différentes | `heuristics/evolver.rs` |

**Implémentation MVP** :
```rust
// Dans calibrate.rs
pub struct EntropyTracker {
    window: VecDeque<(f64, bool)>, // (confidence, was_correct)
    window_size: usize,            // default: 20
}

impl EntropyTracker {
    pub fn entropy(&self) -> f64 {
        // Shannon entropy sur la distribution binned des confidences
        let bins = self.bin_confidences(5); // 5 bins: [0-0.2, 0.2-0.4, ...]
        -bins.iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| p * p.ln())
            .sum::<f64>()
    }
    
    pub fn overconfidence_score(&self) -> f64 {
        // Fraction des cas haute-confiance (>0.7) qui étaient incorrects
        let high_conf: Vec<_> = self.window.iter()
            .filter(|(c, _)| *c > 0.7)
            .collect();
        if high_conf.is_empty() { return 0.0; }
        high_conf.iter().filter(|(_, correct)| !correct).count() as f64 
            / high_conf.len() as f64
    }
}
```

**Ce que ça change** : La calibration passe de passive (enregistrer les résultats) à active (détecter et pénaliser la surconfiance). L'evolver peut maintenant distinguer une stratégie "chanceuse" d'une stratégie "calibrée".

---

### 3. Cognitive Models & AI Algorithms Provide Templates for Language Agents
**arXiv: 2602.22523** | Princeton / Columbia (Feb 2026)

**Concept clé** : Position paper formalisant la notion d'*agent template* — un blueprint qui spécifie les rôles des LLMs individuels et comment leurs fonctionnalités doivent être composées. Les auteurs montrent que les agents existants réimplémentent inconsciemment des architectures cognitives classiques (Soar, ACT-R, BDI).

**Pourquoi c'est critique pour MetaYGN** : Le pipeline 12 stages de MetaYGN EST une architecture cognitive, mais elle n'est pas formalisée comme telle. Ce paper fournit le vocabulaire et le cadre théorique pour :

1. **Nommer ce que MetaYGN fait déjà** :
   - Stages 1-3 (classify/assess/competence) = **Perception + Working Memory** (Soar)
   - Stages 4-6 (tool_need/budget/strategy) = **Deliberation + Planning** (BDI)
   - Stage 7 (act) = **Motor execution** (ACT-R)
   - Stages 8-10 (verify/calibrate/compact) = **Metacognitive monitoring** (Nelson & Narens)
   - Stages 11-12 (decide/learn) = **Episodic learning** (Soar chunking)

2. **Justifier le Dynamic Topology (Phase 2)** : Le paper montre que les architectures cognitives efficaces ne sont PAS séquentielles — elles ont des boucles de reconsidération (Soar impasses), des raccourcis (ACT-R production compilation), et des chemins parallèles. Le DAG planner de MetaYGN est exactement ça.

3. **Fournir des templates pour le Heuristic Evolver (Phase 4)** : Le pattern *assert → commit → reconsider* de Soar est directement transposable en : *propose_strategy → select_strategy → evaluate_outcome → reconsider_strategy*.

**Action concrète** : Ajouter dans `docs/architecture/COGNITIVE_MAPPING.md` une table de correspondance MetaYGN ↔ Cognitive Architecture. Cela sert de fondation théorique pour le `CLAUDE.md` et les justifications d'architecture.

---

### 4. SideQuest: Model-Driven KV Cache Management for Long-Horizon Agentic Reasoning
**arXiv: 2602.22603** | Cornell (Feb 2026)

**Concept clé** : Les méthodes classiques d'éviction KV cache sont token-level (attention score). SideQuest opère au **niveau objet** — il identifie les blocs sémantiques (un outil appelé, un résultat retourné, un raisonnement intermédiaire) et évince les blocs entiers les moins utiles via un thread auxiliaire.

**Mapping MetaYGN — Phase 3 (Context Firewall) :**

MetaYGN a le même problème à l'échelle du `LoopContext` : le contexte grossit à chaque stage, et le stage `compact` (étape 10) ne fait actuellement qu'un compactage naïf. SideQuest fournit l'algo pour un compactage intelligent :

| Concept SideQuest | Transposition MetaYGN | Fichier cible |
|-------------------|----------------------|---------------|
| Object-level granularity | Le `LoopContext` n'est PAS un flux de tokens — c'est une struct avec des champs nommés. Le compactage doit raisonner au niveau champ, pas token. | `core/src/stages/compact.rs` |
| Auxiliary thread | Le daemon peut exécuter un compactage asynchrone entre les stages, sans bloquer le pipeline principal | `daemon/src/background.rs` |
| Eviction scoring | Score = `recency × relevance × access_count`. Les champs du contexte rarement lus (d'après `reads()` du scoped context Phase 3) sont candidats à l'éviction | `core/src/context/compactor.rs` |
| Semantic blocks | Mapper les 12 stages aux "blocs" : le résultat de `assess` est un bloc, celui de `competence` un autre. Quand on compacte, on évince des blocs entiers, pas des fragments | `core/src/context/mod.rs` |

**Action concrète pour Phase 3** : Enrichir `ScopedView` avec un `access_counter: HashMap<ContextField, u32>`. À chaque `reads()`, incrémenter. Le compacteur utilise ces compteurs pour prioriser l'éviction. Seuil MVP : tout champ non accédé dans les 3 derniers stages est compactable.

---

## Analyses détaillées — Tier A

---

### 5. GiGPO: Group-in-Group Policy Optimization
**arXiv: 2505.10978** | NTU Singapore (Oct 2025, v3)

**Pertinence pour MetaYGN** : GiGPO résout le credit assignment multi-step — exactement le problème du Heuristic Evolver (Phase 4) quand il doit savoir QUEL stage a causé le succès/échec.

**Insight extractible** : Le mécanisme d'*anchor state* — identifier les états identiques rencontrés par différentes trajectoires et comparer les actions prises à ces points communs. Pour MetaYGN, l'analogie est : quand deux sessions différentes atteignent le même type de tâche (ex: "refactor Rust file"), comparer les stratégies choisies et leurs résultats.

**Transposition** : Dans `heuristics/evolver.rs`, grouper les sessions par `task_classification` (sortie du stage classify) ET par `context_fingerprint` (hash du type de fichier + taille + complexité). Les sessions avec le même fingerprint mais des stratégies différentes permettent un credit assignment local.

**Limite** : MetaYGN n'entraîne pas de modèle — c'est un runtime qui orchestre un LLM existant. Le credit assignment de GiGPO s'applique donc à l'évolution des heuristiques, pas au LLM lui-même.

---

### 6. Strategy Executability (SSR)
**arXiv: 2602.22583** (Feb 2026)

**Pertinence pour MetaYGN** : Découverte clé — une stratégie peut être "correcte" mais "non-exécutable" par un modèle donné. La distinction *usage vs executability* est directement pertinente pour le stage `strategy` de MetaYGN.

**Insight extractible** : Le stage `strategy.rs` sélectionne actuellement la stratégie par une matrice dureté-codée. Avec SSR, on ajoute un filtre d'exécutabilité : la stratégie doit non seulement être pertinente, mais aussi avoir un historique de succès *avec le LLM backend actuellement utilisé*. Si MetaYGN tourne avec Claude, les stratégies qui marchent avec GPT ne sont pas forcément exécutables.

**Transposition** : Ajouter un champ `model_affinity: HashMap<String, f64>` dans les `HeuristicVersion` de l'evolver. Lors de l'évaluation, le fitness score est pondéré par l'affinité avec le modèle actuel.

---

### 7. Manifold of Failure: Behavioral Attraction Basins
**arXiv: 2602.22291** (Feb 2026)

**Pertinence pour MetaYGN** : Ce paper ne concerne pas la safety au sens classique — il fournit un framework pour cartographier les *zones de défaillance structurées* des LLMs. Pour MetaYGN, cela signifie que le GuardPipeline (5 guards) peut être informé par une carte topologique des failure modes du LLM backend.

**Insight extractible** : L'*Alignment Deviation* metric pourrait être adaptée pour le stage `verify` — au lieu de vérifier binaire (correct/incorrect), mesurer la déviation par rapport au comportement attendu sur un spectre continu. Les guards deviennent alors des détecteurs de bassins d'attraction vers des failure modes connus.

**Limite** : Nécessite des données de failure spécifiques au modèle, ce qui contredit le principe AI-agnostic de MetaYGN. À implémenter comme `[experimental]` feature-flaggée.

---

### 8. I-MCTS: Introspective Monte Carlo Tree Search
**arXiv: 2502.14693** (Feb 2025)

**Pertinence pour MetaYGN** : I-MCTS combine MCTS avec introspection (analyser les résultats des nœuds parent/sibling avant d'expandre). Le parallèle avec le Heuristic Evolver est direct — au lieu de mutations aléatoires, utiliser un arbre de décision explorable.

**Insight extractible** : Le hybrid rewarding (LLM-estimated + actual performance) est exactement ce dont l'evolver a besoin pour le MVP. Phase 1 : random mutations (actuel plan). Phase 2 : remplacer par un I-MCTS simplifié où chaque nœud = une configuration d'heuristique, et l'expansion introspective analyse les siblings avant de muter.

**Limite** : I-MCTS utilise un LLM comme value model, ce qui est coûteux. Pour MetaYGN MVP, un simple decision tree score suffit.

---

## Synthèse des actions par Phase MetaYGN

### Phase 1 — Graph Memory (3-4 jours)
- **U-Mem** : Implémenter UCB scoring sur les nœuds mémoire (`hit_count`, `reward_sum`)
- **U-Mem** : Cascade de retrieval cost-aware (hot → warm → cold → LLM fallback)
- **U-Mem** : Seed le graph avec les 8 skills existants au `session_start`

### Phase 2 — Dynamic Topology (4-5 jours)
- **Cognitive Templates** : Formaliser le mapping stages ↔ cognitive functions
- **SSR** : Ajouter `model_affinity` au `TopologyPlanner` pour adapter le DAG au LLM backend

### Phase 3 — Context Firewall (2 jours)
- **SideQuest** : Object-level compaction basée sur les access counters de `ScopedView`
- **SideQuest** : Background compaction dans le daemon entre les stages

### Phase 4 — Heuristic Evolver (3-4 jours)
- **EGPO** : `EntropyTracker` dans calibrate.rs pour détecter la surconfiance
- **EGPO** : `overconfidence_penalty` dans le fitness score de l'evolver
- **GiGPO** : Grouper les sessions par `task_fingerprint` pour credit assignment local
- **I-MCTS** : Roadmap Phase 2b — remplacer random mutations par arbre introspectif

### Phase 5 — Tool Forge (3-4 jours)
- Aucun paper Tier S/A directement applicable. Les templates existants suffisent.

### Phase 6 — Plugin Integration (2-3 jours)
- **Cognitive Templates** : Documenter l'architecture cognitive dans `CLAUDE.md`

---

## Papers explicitement NON recommandés pour intégration

| Paper | Raison du rejet |
|-------|----------------|
| **ContextRL** | Entraînement de MLLM avec rewards augmentées. MetaYGN est un runtime, pas un trainer. |
| **dLLM** | Architecture de modèle diffusion. Orthogonal à MetaYGN. |
| **NoRA** | Technique de fine-tuning. MetaYGN n'entraîne pas de modèle. |
| **MoDora** | Analyse de documents semi-structurés. Task-specific, pas metacognition. |
| **OmniGAIA** | Benchmark multimodal. MetaYGN n'est pas un modèle à benchmarker. |
| **FIRE** | Benchmark finance. Domain-specific. |
| **Medical Visual** | Domain-specific adaptation. |
| **Personalized LLM Survey** | Trop broad, peu actionnable. |
| **Expert Investment Teams** | Multi-agent finance. Architecture très différente. |
| **AI Research Assistant** | Case study, pas framework. |

---

## Conclusion

Les 4 papers Tier S s'intègrent directement dans la roadmap v0.3.0 sans la modifier :

1. **U-Mem** enrichit la Phase 1 avec du retrieval actif et un scoring UCB
2. **EGPO** donne à la Phase 4 un mécanisme de détection de surconfiance (la "metacognition" du nom MetaYGN)
3. **Cognitive Templates** fournit la fondation théorique de l'architecture entière
4. **SideQuest** rend la Phase 3 opérationnelle avec du compactage object-level

L'effort additionnel estimé est de **+3-4 jours** répartis sur les 6 phases, principalement sur les UCB scores (Phase 1) et l'EntropyTracker (Phase 4). Le reste est de la documentation et du paramétrage.

**La vraie découverte** : EGPO est la pièce théorique qui justifie le nom "MetaYGN" — c'est littéralement de la calibration métacognitive appliquée au runtime. Si un seul paper devait être intégré, c'est celui-là.