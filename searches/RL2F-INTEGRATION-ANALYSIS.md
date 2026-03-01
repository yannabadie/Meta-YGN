# RL2F × MetaYGN — Analyse d'Intégration

> **Paper** : *Improving Interactive In-Context Learning from Natural Language Feedback*
> (Klissarov et al., DeepMind, Février 2026)
>
> **Cible** : MetaYGN v0.3.0 "Adaptive Topology" — runtime métacognitif local-first, AI-agnostic
>
> **Statut** : `[original-proposal]` — non validé, requiert prototypage
>
> **Date** : 2026-02-28

---

## 1. Concepts Clés du Paper RL2F

### 1.1 Perte de Plasticité In-Context

**Problème identifié** : Lorsqu'un LLM reçoit une critique textuelle dans son contexte (KV cache), ses mécanismes d'attention (Fast Weights) n'accordent pas assez de poids aux tokens de critique face à son propre raisonnement initial. Le modèle "s'excuse" mais répète l'erreur au tour suivant.

**Mesure** : Le ratio attention(critique) / attention(raisonnement_initial) décroît avec la longueur du contexte. Plus le contexte s'allonge, plus la critique est noyée.

### 1.2 Asymétrie Teacher-Student

**Architecture** : Un modèle "Professeur" dispose d'un accès privilégié à la vérité terrain (erreurs compilateur, résultats de tests, logs d'exécution). Il formule des critiques en langage naturel — des indices stratégiques, pas la réponse directe — à destination d'un modèle "Élève".

**Contrainte clé** : L'environnement doit être à vérification automatique (code, maths, logique formelle). Les domaines subjectifs ne produisent pas de signal de récompense fiable.

### 1.3 RLVR — Reinforcement Learning with Verifiable Rewards

**Mécanisme** : Si l'Élève utilise la critique pour corriger sa réponse et que le vérificateur confirme le succès, la trajectoire complète (Erreur → Critique → Révision → Succès) génère une récompense positive. L'algorithme RL (PPO/DPO) utilise ces trajectoires pour mettre à jour les Slow Weights (paramètres internes du modèle).

**Objectif** : Modifier la distribution d'attention du modèle pour qu'il accorde un poids maximal aux tokens de critique textuelle.

### 1.4 Internalisation (Agent Autodidacte)

**Phase finale** : Le modèle est entraîné à prédire les critiques du Professeur avant même de les recevoir. Le signal externe (critique) devient une capacité interne (métacognition). Le modèle apprend à détecter ses propres erreurs et à s'auto-corriger avant de produire sa réponse finale.

---

## 2. Mapping sur l'Architecture MetaYGN Réelle

### 2.1 Ce que MetaYGN est (v0.3.0)

| Composant | Réalité | Fichier |
|-----------|---------|---------|
| Pipeline cognitif | 12 stages séquentiels dans un `ControlLoop` mono-agent | `crates/core/src/runner.rs` |
| Topologie dynamique | Skip-routing : Single(4) / Vertical(12) / Horizontal(14) | `crates/core/src/topology.rs` |
| Vérification | Pattern matching sur output ("error", "failed", "panic") | `crates/core/src/stages/verify.rs` |
| Calibration | Heuristiques fixes (+0.1/-0.15 sur vecteur métacog 5D) | `crates/core/src/stages/calibrate.rs` |
| Apprentissage | Logger de lessons (strings), pas de feedback loop | `crates/core/src/stages/learn.rs` |
| Mémoire graphe | SQLite + FTS5 + cosine natif + BFS, mais sans embeddings | `crates/memory/src/graph.rs` |
| Évolution heuristique | Population-based, mutations statistiques, fitness multi-objectif | `crates/core/src/heuristics/evolver.rs` |
| Fatigue humaine | 4 signaux comportementaux, High-Friction à score ≥ 0.7 | `crates/daemon/src/profiler/fatigue.rs` |
| Context pruning | Détection reasoning lock-in (3+ erreurs), injection recovery | `crates/daemon/src/proxy/pruner.rs` |
| Evidence packs | SHA-256 hash chain + Merkle tree + ed25519 | `crates/verifiers/src/evidence.rs` |
| Guard pipeline | 5 guards chaînés (destructive→high_risk→secret→mcp→default) | `crates/verifiers/src/guard_pipeline.rs` |
| Agents | Markdown promptés (personas), pas de processus séparés | `agents/*.md` |

### 2.2 Ce que MetaYGN n'est PAS

- **MetaYGN ne modifie pas les poids du modèle.** C'est un runtime d'orchestration, pas un framework de training. Le principe fondateur est AI-agnostic : le même runtime doit fonctionner avec Claude, GPT, Gemini, ou un modèle local.
- **MetaYGN n'a pas de boucle Teacher-Student native.** Les agents `researcher`, `skeptic`, `verifier` sont des templates Markdown injectés dans le contexte d'un seul appel LLM, pas des processus parallèles avec des contextes asymétriques.
- **MetaYGN n'exporte pas de trajectoires d'entraînement.** Le stage `learn` est un logger, pas un pipeline de génération de données RLHF.

---

## 3. Analyse Concept par Concept

### 3.1 Perte de Plasticité In-Context

#### Applicabilité : ✅ HAUTE — MetaYGN est directement impacté

MetaYGN injecte des critiques dans le contexte du LLM via plusieurs mécanismes :
- `ContextPruner` injecte des recovery prompts après reasoning lock-in
- Les hooks `post_tool_use_failure.py` et `pre_tool_use.py` injectent des warnings
- Les skills `metacog-challenge` et `metacog-proof` forcent le LLM à se remettre en question

**Problème concret** : Si le LLM sous-jacent souffre de perte de plasticité, toutes ces injections sont ignorées. Le `FatigueProfiler` détecte les boucles d'erreur (symptôme), mais ne traite pas la cause (le LLM ignore la critique).

#### Actions possibles

| Action | Niveau | Faisabilité | Phase |
|--------|--------|-------------|-------|
| **A. Mesurer la plasticité** — Ajouter un compteur dans `calibrate.rs` : si la même erreur revient après une critique injectée, incrémenter un `plasticity_failure_count` | Runtime | ✅ Immédiat | v0.3.1 |
| **B. Amplifier le signal critique** — Quand `plasticity_failure_count > 2`, reformuler la critique avec emphase progressive (CAPS, répétition, restructuration) dans le `ContextPruner` | Runtime | ✅ Immédiat | v0.3.1 |
| **C. Exploiter le RL2F en aval** — Si le LLM sous-jacent est fine-tuné avec RL2F (par le fournisseur ou localement), la plasticité augmente et les modules A/B deviennent moins nécessaires | Training | ❌ Hors scope MetaYGN | Post v0.4.0 |

**Recommandation** : Implémenter A et B. Ce sont des compensations runtime pour un problème fondamental du modèle. Si le LLM s'améliore via RL2F, ces compensations s'activent moins (auto-régulation via les seuils).

---

### 3.2 Asymétrie Teacher-Student

#### Applicabilité : ⚠️ MOYENNE — Mapping séduisant mais architecturalement coûteux

L'analyse Gemini propose : `researcher` = Élève, `skeptic` + `verifier` = Professeur. C'est conceptuellement élégant mais **ne reflète pas l'architecture réelle**.

**État actuel** : Un seul appel LLM traverse les 12 stages. Le "skeptic" est un persona injecté dans le system prompt, pas un second modèle avec un contexte asymétrique.

**Ce qu'il faudrait pour une vraie boucle Teacher-Student** :

```
┌─────────────────────────────────────────────────────────┐
│ Appel 1 (Élève)                                         │
│ Context: prompt + code                                  │
│ Output: solution candidate                              │
├─────────────────────────────────────────────────────────┤
│ Sandbox: exécution → résultat/erreur                    │
├─────────────────────────────────────────────────────────┤
│ Appel 2 (Professeur)                                    │
│ Context: prompt + code + erreur brute (info privilégiée)│
│ Output: critique stratégique (pas la solution)          │
├─────────────────────────────────────────────────────────┤
│ Appel 3 (Élève)                                         │
│ Context: prompt + code + critique (sans l'erreur brute) │
│ Output: solution révisée                                │
└─────────────────────────────────────────────────────────┘
```

**Coût** : 3× les appels LLM par itération. Pour un runtime qui a un `BudgetState` avec `max_cost_usd: 0.10`, c'est un triplement du coût.

#### Actions possibles

| Action | Niveau | Faisabilité | Phase |
|--------|--------|-------------|-------|
| **A. Teacher-Student léger** — Après un échec dans `verify.rs`, au lieu de réinjecter l'erreur brute, la reformuler en critique stratégique via un template Rust (pas un 2e appel LLM) | Runtime | ✅ 1-2 jours | v0.3.1 |
| **B. Teacher-Student complet** — Implémenter le schéma 3-appels ci-dessus comme topologie `Dialectic` dans `topology.rs` | Runtime | ⚠️ 3-5 jours, refactoring majeur | v0.4.0 |
| **C. Teacher-Student comme collecteur de données** — Logger les paires (erreur_brute, critique_générée, résultat) pour constituer un dataset RL2F | Data pipeline | ⚠️ 2-3 jours | v0.4.0+ |

**Recommandation** : Implémenter A maintenant (pas d'appel LLM supplémentaire, juste du reformatage intelligent de l'erreur). Reporter B et C à v0.4.0 quand le budget et le multi-agent seront mûrs.

---

### 3.3 RLVR — Récompenses Vérifiables

#### Applicabilité : ✅ HAUTE — MetaYGN a déjà les briques

MetaYGN possède nativement les 3 composants d'un signal de récompense vérifiable :

1. **Vérificateur** : `guard_pipeline.rs` (sécurité) + `verify.rs` (pattern matching) + `evidence.rs` (hash chain)
2. **Signal binaire** : `SessionOutcome.success` dans `fitness.rs`
3. **Métriques quantitatives** : `tokens_consumed`, `duration_ms`, `errors_encountered`

Ce qui manque : la **capture structurée des trajectoires complètes** pour exploitation par un pipeline RL externe.

#### Format de trajectoire proposé

```rust
/// Trajectoire complète capturée par le stage learn pour export RL2F.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rl2fTrajectory {
    /// Identifiant unique de la session
    pub session_id: String,
    /// Classification de la tâche
    pub task_type: TaskType,
    /// Tentative initiale du LLM (résumé)
    pub initial_attempt: String,
    /// Erreur vérifiable (sortie sandbox, test failure, guard block)
    pub verifiable_error: Option<String>,
    /// Critique formulée (par template ou par LLM Teacher)
    pub critique: Option<String>,
    /// Tentative révisée du LLM (résumé)
    pub revised_attempt: Option<String>,
    /// Résultat final vérifié
    pub outcome: SessionOutcome,
    /// Score de calibration au moment de la tentative
    pub calibration_snapshot: MetacognitiveVector,
    /// Timestamp
    pub timestamp: String,
}
```

#### Actions possibles

| Action | Niveau | Faisabilité | Phase |
|--------|--------|-------------|-------|
| **A. Struct `Rl2fTrajectory`** — Définir dans `crates/shared/src/state.rs` | Types | ✅ 0.5 jour | v0.3.1 |
| **B. Capture dans `learn.rs`** — Construire la trajectoire à partir du `LoopContext` accumulé | Runtime | ✅ 1 jour | v0.3.1 |
| **C. Export JSONL** — Écrire les trajectoires dans `~/.claude/aletheia/trajectories/` | I/O | ✅ 0.5 jour | v0.3.1 |
| **D. Pipeline RL externe** — Consommer les JSONL pour fine-tuner via PPO/DPO | Training | ❌ Hors scope MetaYGN | Projet séparé |

**Recommandation** : Implémenter A+B+C. MetaYGN devient un **collecteur passif de données RL2F** sans modifier son périmètre runtime. Le fine-tuning reste un projet externe.

---

### 3.4 Internalisation (Métacognition Autonome)

#### Applicabilité : ❌ HORS SCOPE — Confusion de niveaux

L'internalisation RL2F modifie les **poids du modèle** pour que la métacognition devienne intrinsèque au réseau de neurones. C'est du training, pas du runtime.

Les `skills/metacog-*` de MetaYGN sont du **prompting structuré** : ils forcent le LLM à suivre un template de réflexion dans le contexte. C'est efficace mais fragile — si le contexte est long ou si le modèle a une faible plasticité, les templates sont ignorés.

**Distinction fondamentale** :

| | Prompting (MetaYGN) | Internalisation (RL2F) |
|---|---|---|
| Mécanisme | Injection de texte dans le contexte | Modification des paramètres du modèle |
| Persistance | Par session (perdu au refresh) | Permanent (ancré dans les poids) |
| Coût runtime | Tokens supplémentaires à chaque appel | Zéro coût runtime additionnel |
| Dépendance modèle | Fonctionne avec tout LLM | Requiert accès aux poids |
| Robustesse | Dégradé en long contexte | Résistant à la longueur de contexte |

#### Action unique

| Action | Niveau | Faisabilité | Phase |
|--------|--------|-------------|-------|
| **A. Mesurer l'efficacité des skills metacog** — Tracker si l'invocation d'un skill metacog corrèle avec une amélioration du `metacog_vector` dans la même session | Runtime | ✅ 1 jour | v0.3.1 |

**Recommandation** : Mesurer, pas implémenter. Si les données montrent que les skills metacog sont ignorés dans les longs contextes, c'est un signal pour investir dans le Teacher-Student (§3.2.B) plutôt que dans plus de prompts.

---

## 4. Concepts RL2F Transversaux

### 4.1 Fatigue Profiler comme Signal Indirect de Plasticité

Le `FatigueProfiler` détecte quand le **humain** boucle. Mais il peut aussi servir de proxy pour la plasticité du **LLM** :

- Si `consecutive_errors ≥ 3` (ErrorLoop) ET que le `ContextPruner` a déjà injecté un recovery prompt → le LLM a ignoré la critique → **signal de perte de plasticité**

**Action** : Ajouter un champ `plasticity_warnings: u32` dans `FatigueReport`. Incrémenter quand ErrorLoop survient APRÈS une injection pruner. Ce signal alimente `calibrate.rs` pour baisser la confiance.

### 4.2 Connexion EGPO × RL2F

Le paper EGPO (entropy calibration, déjà en Tier S du Paper Triage) et RL2F sont **complémentaires** :

- **EGPO** détecte la surconfiance (le LLM est sûr de lui mais a tort)
- **RL2F** détecte la perte de plasticité (le LLM ignore les corrections)

Ces deux pathologies sont distinctes mais se renforcent mutuellement :
- Un LLM surconfiant ET non-plastique = le pire cas possible : il fait des erreurs, les ignore, et reste confiant

**Action** : L'`EntropyTracker` proposé dans le Paper Triage devrait inclure un flag `plasticity_responsive: bool` basé sur l'historique des corrections réussies après critique.

### 4.3 Evidence Pack comme Vérificateur RL2F

L'`EvidencePack` (hash chain + Merkle + ed25519) fournit un signal d'intégrité cryptographique. Dans un pipeline RL2F, les trajectoires doivent être **non-falsifiées** pour éviter le reward hacking.

**Action** : Signer les `Rl2fTrajectory` exportées avec l'evidence pack. Chaque trajectoire JSONL inclut un hash de preuve. Un pipeline RL externe peut vérifier l'intégrité avant d'entraîner.

---

## 5. Synthèse — Plan d'Action

### Intégrer maintenant (v0.3.1)

| # | Action | Fichiers impactés | Effort |
|---|--------|-------------------|--------|
| 1 | `plasticity_failure_count` dans `calibrate.rs` | `core/src/stages/calibrate.rs` | 0.5j |
| 2 | Amplification progressive des critiques dans `pruner.rs` | `daemon/src/proxy/pruner.rs` | 0.5j |
| 3 | Teacher-Student léger : reformatage d'erreur → critique | `core/src/stages/verify.rs` | 1j |
| 4 | Struct `Rl2fTrajectory` + capture dans `learn.rs` | `shared/src/state.rs`, `core/src/stages/learn.rs` | 1j |
| 5 | Export JSONL des trajectoires | `daemon/src/api/hooks.rs` | 0.5j |
| 6 | `plasticity_warnings` dans `FatigueReport` | `daemon/src/profiler/fatigue.rs` | 0.5j |
| 7 | Flag `plasticity_responsive` dans `EntropyTracker` (EGPO) | `core/src/stages/calibrate.rs` | 0.5j |
| | **Total** | | **4.5j** |

### Reporter (v0.4.0+)

| # | Concept | Raison du report |
|---|---------|------------------|
| 8 | Topologie `Dialectic` (Teacher-Student complet, 3 appels LLM) | Requiert refactoring du `ControlLoop` pour multi-appel + gestion budget 3× |
| 9 | Pipeline RL externe (PPO/DPO sur trajectoires exportées) | Hors scope MetaYGN — projet séparé nécessitant accès aux poids |
| 10 | Internalisation métacognitive | Fondamentalement du training, incompatible avec le principe AI-agnostic |

### Ne pas intégrer (rejeté)

| # | Concept | Raison du rejet |
|---|---------|-----------------|
| 11 | "Usine à données synthétiques" (claim Gemini) | Surestimation — MetaYGN n'est pas conçu pour générer des millions de trajectoires, mais pour en capturer quelques centaines par session réelle |
| 12 | Mapping natif Teacher-Student sur agents Markdown | Les agents sont des personas promptées, pas des processus. Le mapping est intellectuellement satisfaisant mais techniquement faux |
| 13 | Remplacement des skills metacog par l'internalisation | L'un n'exclut pas l'autre. Les skills restent utiles tant que MetaYGN est AI-agnostic |

---

## 6. Diagramme d'Impact

```
                    RL2F Paper
                        │
          ┌─────────────┼─────────────────┐
          │             │                 │
   Plasticité    Teacher-Student      RLVR
   In-Context         │              Récompenses
          │             │             Vérifiables
          ▼             ▼                 │
   ┌─────────────┐  ┌──────────┐         │
   │ calibrate.rs│  │ verify.rs│         ▼
   │ + plasticity│  │ + critique│  ┌───────────┐
   │   counter   │  │ formatter│  │ learn.rs   │
   └──────┬──────┘  └─────┬────┘  │ + Rl2f     │
          │               │       │ Trajectory  │
          │               │       └──────┬──────┘
          ▼               ▼              ▼
   ┌─────────────┐  ┌──────────┐  ┌───────────┐
   │ pruner.rs   │  │ fatigue  │  │ Export     │
   │ + amplif.   │  │ .rs +    │  │ JSONL     │
   │ progressive │  │ plasticity│  │ signé     │
   └─────────────┘  │ warnings │  └───────────┘
                    └──────────┘        │
                                        ▼
                                  ┌───────────┐
                                  │ Pipeline   │
                                  │ RL externe │
                                  │ (hors      │
                                  │  scope)    │
                                  └───────────┘
```

---

## 7. Relation avec le Paper Triage (Février 2026)

| Paper Triage (Tier S) | RL2F | Synergie |
|------------------------|------|----------|
| **EGPO** (entropy calibration) | Plasticité in-context | EGPO détecte surconfiance, RL2F détecte non-plasticité → `EntropyTracker` + `plasticity_responsive` |
| **U-Mem** (mémoire active UCB) | RLVR | UCB scoring sur les nœuds mémoire = reward signal exploitable par RL2F trajectories |
| **SideQuest** (compaction objet) | Plasticité in-context | Compaction intelligente réduit la longueur de contexte → atténue la perte de plasticité |
| **Cognitive Templates** | Internalisation | Les templates formalisent ce que RL2F tente d'internaliser — l'un documente, l'autre exécute |

---

*Document généré pour `C:\Projects\MetaYGN\docs\plans\RL2F-INTEGRATION-ANALYSIS.md`*
*Classification : `[original-proposal]` — requiert validation par prototypage*
