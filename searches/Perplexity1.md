# Aletheia-Nexus — Enrichissement du Cadrage Projet via Discover AI (@code4AI)

## Synthèse du Projet

Aletheia-Nexus est un **runtime métacognitif local-first** construit en Rust (daemon `aletheiad`) + plugin natif Claude Code (TypeScript) + thin-MCP, ciblant l'écosystème Claude Code 4.6 Opus. Le projet vise à surpasser Poetiq (ex-DeepMind, SOTA ARC-AGI) en gagnant sur 5 axes : zéro-token bloat, proof-carrying reasoning via shadow WASM sandboxing, métacognition inverse (calibration de la fatigue humaine), context pruning time-travel, et data-privacy absolue. L'architecture hybride repose sur 7 couches métacognitives (planification, monitoring, vérification, réflexion, calibration, outils, collectif) avec un vecteur métacognitif compact de ~30 tokens.[1][2][3]

Les documents joints couvrent les analyses croisées de ChatGPT 5.2, Claude 4.6, Gemini 3.1 Deep Think et Grok 4.20, convergeant vers un design CLI-first, daemon stateful SQLite, hooks déterministes natifs Claude Code et progressive disclosure.[4][5][2]

***

## Vidéos Discover AI les Plus Pertinentes pour le Projet

La chaîne **Discover AI** (@code4AI) — 85K abonnés, 1 200+ vidéos — est une source de veille de recherche IA appliquée couvrant les papiers les plus récents avec une profondeur technique rare. Voici les vidéos sélectionnées par pertinence directe avec l'architecture Aletheia-Nexus.[6][7]

### AgentOS : New SYSTEM Intelligence (fév. 2026)

Cette vidéo, la plus récente et la plus critique pour le projet, présente un **Agent Operating System** pour l'orchestration de systèmes multi-agents complexes. Le paper analysé introduit un « cognitive synchronization pulse » — un mécanisme qui suspend tous les threads d'agents, fusionne leurs états sémantiques latents et les redémarre depuis un état de vérité partagé lorsque la dérive sémantique dépasse un seuil mathématique.[8]

**Pertinence directe pour Aletheia-Nexus :**
- Le concept de **drift management** (dérive cognitive) résonne avec le module Context Pruning d'Aletheia — le daemon détecte les boucles logiques et ampute le contexte pollué[2]
- Le seuil de **cognitive collapse** à 60 agents interagissant simultanément montre la limite des ensembles multi-agents, validant l'approche d'Aletheia de spawner des sub-agents *jetables* plutôt que des essaims permanents[8]
- La transition évoquée de « prompt engineering → context engineering → context as addressable, sliceable memory » est exactement le paradigme du vecteur métacognitif compact d'Aletheia[8]

### Stanford : AI Agents DESTROY their Own Intelligence (fév. 2026)

Ce papier de Stanford (relayé par Discover AI) démontre que **l'alignement RLHF des modèles façonne — et dégrade — la cognition des agents en équipe**. Les agents alignés ne savent pas quand dominer et quand compromettre dans un contexte multi-agent, ce qui crée des échecs systémiques.[9]

**Pertinence directe :**
- Valide l'axe « métacognition inverse » d'Aletheia : si les agents sont incapables de juger leur propre fiabilité en contexte d'équipe, un daemon externe local est la bonne approche pour imposer des garde-fous déterministes[4]
- Pose la question : cette défaillance est-elle inhérente au post-training RLHF ? Si oui, Aletheia gagne un avantage structurel en externalisant la métacognition hors du LLM[9]

### Salesforce : AI Agents That Doubt Themselves — AUQ (janv. 2026)

Ce papier sur l'**Agentic Uncertainty Quantification** (AUQ) propose un framework dual-process inspiré de la cognition humaine : un System 1 (mémoire avec confiance verbalisée) et un System 2 (réflexion déclenchée uniquement quand la confiance chute).[10][11]

**Pertinence directe :**
- L'idée de **uncertainty-aware memory** (l'agent ne retient pas seulement ce qu'il a fait, mais *à quel point il était confiant*) est une brique manquante potentielle pour la table `metacognitive_state` SQLite d'Aletheia[10]
- Le concept de « spirale d'hallucination » — une erreur mineure en début de chaîne qui biaise tout le planning futur — valide directement le module `aletheia loop-breaker` et le context pruning[11]
- AUQ atteint le SOTA sur DeepResearch, ALFWorld et WebShop *sans entraînement*, ce qui renforce la thèse qu'un outil externe (plutôt qu'un fine-tuning) suffit pour la métacognition[11]

### SiriuS : Self-Improving Multi-Agent Systems (fév. 2025)

Vidéo Discover AI analysant le paper de Stanford sur **SiriuS** (Bootstrapped Reasoning pour le self-improvement multi-agent). Le système étend le framework STaR (Self-Taught Reasoner) des agents individuels aux systèmes multi-agents, permettant une amélioration continue du raisonnement via des trajectoires réussies.[12]

**Pertinence directe :**
- L'approche bootstrapped reasoning peut alimenter le module de **mémoire épisodique** d'Aletheia : stocker les trajectoires de raisonnement réussies par fichier/repo dans SQLite, puis les injecter comme few-shot context quand un pattern similaire est détecté[12]
- La distinction explicite entre self-improvement d'un agent unique vs. multi-agent valide la couche 7 (métacognition collective) de l'architecture[12]

### The $10T AI Economy : New Smart Protocol Emerges (fév. 2026)

Discover AI analyse le paper de Google proposant un **marketplace décentralisé** pour agents IA avec enchères, contrats, réputation et facteurs de confiance financiers.[13]

**Pertinence directe :**
- Le concept de **trust as a financial parameter** (pas un concept éthique) offre un modèle pour la Phase 3 « World Domination » d'Aletheia : si le daemon expose un score de confiance certifiable, il devient un **avantage compétitif sur un marketplace d'agents**[13]
- La notion de « certifiable agentic capabilities » résonne avec le proof-carrying reasoning d'Aletheia — un agent avec des preuves compilées (WASM sandbox) est plus « trustworthy » qu'un agent qui s'auto-évalue verbalement[13]
- Le protocole A2A (agent-to-agent) de Google est le véhicule naturel pour la couche 7 collective[1][13]

### In a Network of AI Agents : Pure CHAOS (mars 2025)

Discover AI analyse le paper UC Berkeley « Why Do Multi-Agent LLM Systems Fail? » démontrant que les systèmes multi-agents souffrent de task derailment, information withholding, hallucination cascades et conflits sans résolution.[14]

**Pertinence directe :**
- Justifie le design d'Aletheia d'utiliser des **sub-agents à context fork** isolés (avocats du diable) plutôt que des swarms ouverts[3]
- Le problème de « reasoning lock-in » identifié par Berkeley correspond exactement au `Exit 137` d'`aletheia loop-breaker`[5]
- L'observation que « plus d'agents = plus d'instabilité » conforte l'approche resource-rational : ne spawner la métacognition lourde que si la tâche est classifiée comme complexe[14]

### New AI Framework: Post-Training — Unifying SFT and RL (juil. 2025)

Cette vidéo Discover AI explore l'unification du Supervised Fine-Tuning (SFT) et du Reinforcement Learning pour l'alignement, montrant comment les « implicit rewards » servent de pont entre les deux paradigmes.[15]

**Pertinence directe :**
- Éclaire la question de *pourquoi* les agents alignés par RLHF échouent en contexte multi-agent (cf. Stanford ci-dessus) : si l'alignement crée des récompenses implicites mal calibrées, le daemon Aletheia peut servir de **correcteur d'alignement contextuel** en temps réel[15]
- Pertinent pour comprendre les futures évolutions de Claude Code 4.6+ et anticiper les changements de comportement post-training[15]

***

## Vidéos Complémentaires Hors Discover AI

### AI Self-Evolving Agents : Agent0 (nov. 2025)

Agent0 est un framework d'agents auto-évolutifs **sans données humaines** utilisant deux agents co-évoluant : un Curriculum Agent qui génère des tâches de difficulté croissante, et un Executor Agent qui les résout via des outils externes. Résultats : +18% en raisonnement mathématique, +24% en raisonnement général sur Qwen 3-8B.[16]

**Pertinence :** Le mécanisme d'auto-évolution par outils externes (code interpreter) valide le design d'Aletheia — c'est l'exécution WASM sandbox qui force l'amélioration, pas l'introspection verbale.[16]

### Multi-Agent Evolve (MAE) : LLM Self-Improve through Co-evolution (oct. 2025)

Le framework MAE instancie Proposer, Solver et Judge depuis un seul LLM, avec une boucle de self-rewarding co-évolutive sans données humaines ni vérificateurs externes.[17]

**Pertinence :** L'architecture Proposer-Solver-Judge peut inspirer les sub-agents d'Aletheia : le « devil-advocate » correspond au Judge, le sub-agent shadow WASM au Solver, et le hook `UserPromptSubmit` au Proposer.[17]

### AI Code That Fixes Itself : MCP + Knowledge Graphs (juin 2025)

Cette vidéo montre comment des **knowledge graphs** permettent aux agents de coder de vérifier leur code contre des APIs réelles (pas d'hallucination syntaxique) via un MCP server de RAG crawlé, permettant l'auto-correction en temps réel.[18]

**Pertinence :** Le pattern « crawl → knowledge graph → hallucination detection → self-correction » est exactement le shadow sandboxing WASM d'Aletheia, mais appliqué aux dépendances externes plutôt qu'à la compilation seule. C'est un enrichissement possible de la Phase 2.[18]

### Wassette : Microsoft WASM + MCP Bridge (août 2025)

Microsoft a publié **Wassette**, un runtime Rust qui transforme automatiquement tout composant WebAssembly en outil MCP, permettant aux agents de **télécharger, inspecter et exécuter des outils de façon autonome** depuis des registres OCI.[19]

**Pertinence critique :** Wassette est exactement le pont technique entre le daemon Rust/Wasmtime d'Aletheia et l'écosystème MCP. Il utilise Wasmtime (déjà dans la stack Aletheia), est écrit en Rust, et fournit une isolation sécuritaire « browser-level » avec un modèle de capabilities par défaut deny-all. Ce composant pourrait accélérer massivement la Phase 2 (Marketplace Anthropic) en permettant au noyau Rust compilé en WASM d'être automatiquement exposé comme outil MCP.[19]

***

## Matrice de Pertinence

| Vidéo / Source | Couche Aletheia impactée | Impact |
|---|---|---|
| AgentOS (fév. 2026) [8] | Context Pruning (L4), Sub-agents (L5), Collectif (L7) | ★★★★★ |
| AUQ Salesforce (janv. 2026) [10] | Calibration (L3), Mémoire épisodique, Loop-breaker | ★★★★★ |
| Stanford Agents DESTROY (fév. 2026) [9] | Métacognition Inverse (L3), Hooks déterministes | ★★★★☆ |
| SiriuS Stanford (fév. 2025) [12] | Mémoire épisodique, Self-improvement, Collectif (L7) | ★★★★☆ |
| $10T Economy Protocol (fév. 2026) [13] | Distribution (Phase 3), Trust certifiable, A2A | ★★★★☆ |
| Agents Pure CHAOS (mars 2025) [14] | Sub-agents isolés, Loop-breaker, Resource-rational | ★★★★☆ |
| Wassette WASM+MCP (août 2025) [19] | Tier 3 Thin-MCP, Phase 2 Marketplace, Shadow WASM | ★★★★★ |
| Agent0 Self-Evolving (nov. 2025) [16] | Shadow WASM, Proof-Carrying, Tool-integrated reasoning | ★★★☆☆ |
| MAE Co-evolution (oct. 2025) [17] | Sub-agents architecture (devil-advocate pattern) | ★★★☆☆ |
| MCP + Knowledge Graphs (juin 2025) [18] | Shadow sandboxing enrichi, Hallucination detection | ★★★☆☆ |
| Post-Training SFT+RL (juil. 2025) [15] | Compréhension du pipeline d'alignement Claude | ★★☆☆☆ |

***

## Recommandations d'Intégration Prioritaire

### Enrichissements à intégrer immédiatement

1. **Uncertainty-aware memory (AUQ)** — Ajouter un champ `confidence_score` à chaque entrée de la table `metacognitive_state` SQLite. L'agent ne stocke pas seulement l'erreur, mais *son niveau de certitude au moment de l'erreur*. Cela permet au daemon de détecter les patterns de sur-confiance avant échec, pas seulement les échecs eux-mêmes[10][11]

2. **Cognitive synchronization pulse (AgentOS)** — Quand le daemon détecte une divergence sémantique entre l'agent principal et ses sub-agents shadow WASM, il doit pouvoir forcer une resynchronisation : suspendre, fusionner les états, et redémarrer depuis un état de vérité partagé. Cela évite le « cognitive collapse » observé à grande échelle[8]

3. **Wassette comme accélérateur de Phase 2** — Évaluer l'intégration de Wassette (MIT, Rust, Wasmtime) pour le bridge automatique WASM→MCP. Le noyau Rust d'Aletheia compilé en WASM pourrait être auto-exposé comme outil MCP compatible Claude, Cursor, Gemini CLI sans code d'intégration supplémentaire[19]

### Enrichissements pour la feuille de route

4. **Trust certifiable (Google $10T paper)** — En Phase 3 (Marketplace), le proof-carrying reasoning d'Aletheia peut être formaté comme un « Agent Card » avec des capacités certifiées et un score de confiance vérifiable, créant un avantage compétitif sur tout marketplace d'agents[13]

5. **Pattern Proposer-Solver-Judge (MAE)** — Restructurer les sub-agents shadow en trois rôles explicites : le hook `PreToolUse` comme Proposer (évalue le risque), le WASM sandbox comme Solver (teste l'hypothèse), et le daemon comme Judge (verdict confiance/rejet)[17]

6. **Hallucination cascade detection** — Le concept de « spirale d'hallucination » d'AUQ  et les « hallucination cascades » de Berkeley  suggèrent d'enrichir `aletheia loop-breaker` avec un détecteur de propagation : si l'erreur N+1 est *causalement liée* à l'erreur N (pas juste consécutive), escalader plus agressivement[14][11]