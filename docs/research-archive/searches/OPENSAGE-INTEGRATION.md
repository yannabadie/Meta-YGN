# Blueprint d'Intégration : MetaYGN x OpenSage (Architecture Locale)

Ce document détaille le plan directeur pour faire évoluer MetaYGN d'une topologie statique vers un système auto-généré (inspiré d'OpenSage, AlphaEvolve et PSRO), optimisé pour une exécution sur CPU (i7-1165G7, 32GB RAM).

## 1. Choix Technologiques & Contraintes Matérielles

Le matériel cible n'ayant pas de GPU dédié, nous adoptons une approche hybride :
- **LLM Reasoning (Topologie, Stratégie)** : APIs externes (Claude 3.7, DeepSeek-Coder, Codex, OpenAI) pour décharger le CPU.
- **Graph Memory & Embeddings** : 100% Local.
  - **Base de données Graph** : Extension de votre SQLite existant (`crates/memory/src/store.rs`) au lieu d'un Neo4j lourd.
  - **Moteur Vectoriel** : `sqlite-vec` (extension SQLite pour la recherche sémantique) chargé directement depuis Rust via `rusqlite`.
  - **Modèle d'Embeddings Local** : `nomic-embed-text-v1.5` (GGUF, ~250MB) ou `bge-small-en-v1.5`. Ils tournent parfaitement sur un i7 en utilisant `candle` (Rust) ou `llama.cpp` en backend, générant des vecteurs à la volée.

## 2. Le Plan d'Intégration (Les 4 Gaps)

### P0 (GAP 4) : Graph Memory & Embeddings Locaux (Fondation)
OpenSage nécessite une mémoire hiérarchique pour comprendre le contexte des agents créés.
*   **Action** : Modifier `crates/memory/src/store.rs` pour intégrer `sqlite-vec` et créer une structure de graphe.
*   **Schéma SQLite** :
    ```sql
    -- Nœuds du graphe
    CREATE TABLE memory_nodes (
        id TEXT PRIMARY KEY,
        node_type TEXT NOT NULL,  -- 'session', 'task', 'decision', 'verification', 'pattern'
        content TEXT NOT NULL,
        embedding FLOAT[768],     -- sqlite-vec syntax
        created_at TEXT DEFAULT (datetime('now'))
    );

    -- Arêtes du graphe
    CREATE TABLE memory_edges (
        source_id TEXT REFERENCES memory_nodes(id),
        target_id TEXT REFERENCES memory_nodes(id),
        relation TEXT NOT NULL,   -- 'caused', 'verified_by', 'contradicts', 'refined_to'
        weight REAL DEFAULT 1.0,
        PRIMARY KEY (source_id, target_id, relation)
    );
    ```
*   **Implémentation Rust** : Utiliser un modèle d'embedding très léger (ex: `bge-small`) appelé en local lors de l'insertion dans `memory_nodes`. `sqlite-vec` permettra de faire des requêtes `knn` (k-nearest neighbors) sur le CPU en millisecondes.

### P1 (GAP 1) : Sélecteur de Topologie Dynamique (Runtime Self-Assembly)
Au lieu de règles statiques dans `aletheia-main.md`, le système doit choisir la topologie des sous-agents.
*   **Action** : Mettre à jour le hook `pre_tool_use.py` ou `UserPromptSubmit`.
*   **Mécanisme** : 
    1. Le hook analyse la demande de l'utilisateur.
    2. Il interroge la Graph Memory SQLite (via le Daemon Rust) pour trouver des tâches similaires.
    3. Il injecte dynamiquement un `topology_plan` au LLM (ex: "Déploie un agent 'researcher' en parallèle d'un 'coder', puis un 'verifier'").

### P2 (GAP 2) : Évolution des Heuristiques par les Données (AlphaEvolve light)
MetaYGN utilise des mots-clés statiques (`HIGH_RISK_MARKERS`).
*   **Action** : Un thread asynchrone dans le `daemon` Rust qui s'exécute à la fin de chaque session (`SessionEnd`).
*   **Mécanisme** : Le daemon analyse les événements `Stop` avec leurs `outcomes` (succès/échec). S'il détecte qu'un marqueur classé "low_risk" provoque souvent des erreurs (escalation), il met à jour le score du marqueur dans une nouvelle table SQLite `heuristic_versions`. Pas besoin de LLM ici, des simples mathématiques statistiques suffisent pour ajuster la "pression évolutive" de vos filtres de risque.

### P3 (GAP 3) : Modèle PSRO (Multipath Reasoning)
Pour les modifications de code à haut risque, ne pas faire confiance à une seule itération LLM.
*   **Action** : Créer un nouveau skill/agent `ensemble_manager.md`.
*   **Mécanisme** : Lorsque le risque est évalué comme "CRITICAL", le Daemon orchestre l'appel de 2 prompts parallèles distincts (ex: un avec focus "Sécurité" et un avec focus "Performance"). L'agent "Ensemble" compare les deux résultats dans le scratchpad et fusionne la meilleure approche.

---

## 3. PROMPT ULTIME : Master Prompt de Développement

*À utiliser dans Claude Code ou votre LLM API préféré pour démarrer le refactoring.*

> "Tu es un Ingénieur Architecte Expert en systèmes Multi-Agents (Meta-Cognition et topologies dynamiques). Nous allons faire évoluer le projet MetaYGN (Rust + Python + Claude CLI) en intégrant les concepts des papiers OpenSage (2602.16891) et AlphaEvolve, adaptés pour une exécution CPU stricte.
>
> 1. Ton premier objectif est de remplacer l'historique plat des événements dans `crates/memory/src/store.rs` par une Graph Memory hiérarchique supportée par SQLite et `sqlite-vec`. Écris le code Rust complet pour ajouter les tables `memory_nodes` et `memory_edges`, et ajoute une méthode `search_similar_nodes(embedding: Vec<f32>)`.
> 2. Assure-toi que le daemon Rust soit prêt à exposer ces requêtes de graphes aux hooks Python.
> 3. Ne détruis pas la table `events` existante, fais cohabiter les deux systèmes de stockage pendant la transition. Fais un point d'arrêt et propose-moi les modifications de code pour validation."