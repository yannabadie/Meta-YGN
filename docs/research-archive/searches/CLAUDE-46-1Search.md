# Métacognition et IA : le levier stratégique qui transforme les projets AI-driven

**La métacognition — la capacité d'un système à « penser sur sa propre pensée » — est en train de devenir le différenciateur principal entre les systèmes d'IA fragiles et ceux qui sont véritablement fiables.** Les données de la recherche 2023-2026 montrent des gains mesurables spectaculaires : **+23% de F1 sur la vérification factuelle** (Chain-of-Verification), **43% de réduction des hallucinations** (ReMA), **91% pass@1 sur HumanEval** (Reflexion vs. 80% baseline), et **67% de réduction des coûts en tokens** (TALE) — tout cela grâce à des mécanismes qui permettent à l'IA de monitorer, évaluer et réguler ses propres processus cognitifs. Ce rapport synthétise l'état de l'art, les méthodes établies et émergentes, les territoires inexplorés, et se conclut par un prompt ultra-détaillé pour construire le MCP de métacognition le plus avancé possible.

---

## L'état académique en 2026 : un champ en explosion

La métacognition en IA a connu une accélération remarquable entre 2023 et 2026, passant d'un concept de niche à un axe de recherche central porté par les plus grands laboratoires mondiaux. Le papier fondateur **"Imagining and Building Wise Machines: The Centrality of AI Metacognition"** (Johnson, Karimi, Bengio, Chater, Gerstenberg, Larson, Levine, Mitchell, Rahwan, Schölkopf, Grossmann — arXiv:2411.02478, 2024) a posé les bases en argumentant que la métacognition est la clé manquante pour des systèmes IA véritablement « sages ». Ce papier, co-signé par des figures comme Yoshua Bengio (Mila), Bernhard Schölkopf (Max Planck) et Melanie Mitchell (Santa Fe Institute), propose le framework TRAP — **Transparency, Reasoning, Adaptation, Perception** — comme les quatre dimensions de la métacognition IA.

En parallèle, la recherche empirique a démontré que les LLMs possèdent des capacités métacognitives mesurables mais limitées. **Didolkar et al. (NeurIPS 2024)** ont prouvé que les LLMs peuvent nommer et catégoriser les compétences qu'ils appliquent en résolution mathématique, avec des gains de performance quand ces « méta-connaissances » sont réutilisées via prompting ciblé. À l'inverse, **Scholten, Rebholz et Hütter (2024)** ont identifié la « myopie métacognitive » des LLMs — cinq symptômes systématiques (intégration de tokens invalides, susceptibilité à la redondance, négligence des taux de base, etc.) causés par l'absence de monitoring métacognitif réel.

L'une des découvertes les plus significatives vient d'**Anthropic** : l'expérience de « neurofeedback » de Li Ji-An et al. (arXiv:2505.13763, 2025) démontre que les LLMs peuvent monitorer et contrôler leurs propres patterns d'activation, mais seulement dans un sous-espace « métacognitif » de dimensionnalité bien inférieure à leur espace neuronal total. Autrement dit, les LLMs ne « voient » qu'une fraction de leurs propres processus internes — une limitation fondamentale avec des implications majeures pour la sécurité IA.

La revue systématique de **Nolte et al. (arXiv:2503.13467, 2025)** a catalogué 35 architectures métacognitives computationnelles distinctes dans 101 publications, révélant un constat préoccupant : **seulement 17% ont été évaluées quantitativement**, et l'incohérence terminologique bloque la comparaison inter-architectures. Le champ manque cruellement de benchmarks standardisés.

Côté multi-agents, **ReMA (Reinforced Meta-thinking Agents)** de Wan et al. (NeurIPS 2025) a démontré que la séparation explicite entre un agent de « méta-pensée » (oversight stratégique) et un agent d'exécution (raisonnement détaillé), optimisés conjointement par MARL, réduit les hallucinations de **43%** et améliore la performance sur les problèmes mathématiques les plus difficiles. **MetaMind** (arXiv:2505.18943, 2025), un framework multi-agents inspiré de la métacognition pour le raisonnement social, atteint **73.9%** sur les tâches sociales en simulation sandbox vs. **39.4%** pour GPT-4 seul — un bond de **+34.5 points**.

---

## Les six méthodes établies : un arsenal métacognitif opérationnel

### Chain-of-Thought et ses variantes : la transparence du raisonnement

Le Chain-of-Thought (Wei et al., NeurIPS 2022) est la forme la plus fondamentale de métacognition IA : externaliser le raisonnement intermédiaire. Ses extensions ont démontré des gains spectaculaires. **Tree of Thoughts** (Yao et al., NeurIPS 2023) transforme le raisonnement linéaire en exploration arborescente avec backtracking, passant de **4% à 74%** de réussite sur le Game of 24. **Graph of Thoughts** (Besta et al., AAAI 2024) améliore encore le tri de **62% par rapport à ToT** tout en réduisant les coûts de **31%**. Le CoT est aujourd'hui universellement adopté — les modèles o1/o3 d'OpenAI, Claude 3.7+ d'Anthropic, et DeepSeek-R1 l'implémentent nativement via des « thinking tokens ».

### Self-reflection : l'apprentissage verbal par l'erreur

**Reflexion** (Shinn et al., NeurIPS 2023) est le framework de référence pour l'auto-réflexion. Les agents réfléchissent verbalement sur leurs échecs, stockent ces réflexions en mémoire épisodique, et améliorent leur comportement aux essais suivants — du « reinforcement learning verbal » sans mise à jour de poids. Résultat : **91% pass@1 sur HumanEval** (vs. 80% pour GPT-4 baseline) et **130/134 tâches résolues sur ALFWorld**. **Self-Refine** (Madaan et al., NeurIPS 2023) implémente un cycle itératif génération → auto-feedback → raffinement, avec **~20% d'amélioration absolue** sur 7 tâches diverses sans aucun entraînement. L'étude systématique de Renze et Guven (2024) a testé 8 types de self-reflection sur 9 LLMs, confirmant une amélioration statistiquement significative (p < 0.001).

### Self-consistency : le cross-checking interne

La self-consistency (Wang et al., ICLR 2023) échantillonne multiples chemins de raisonnement et sélectionne la réponse majoritaire — une forme de monitoring métacognitif par validation croisée interne. Les gains sont substantiels et consistants : **+17.9% sur GSM8K, +11.0% sur SVAMP, +12.2% sur AQuA**. La variante **CISC (Confidence-Informed Self-Consistency)** réduit les chemins nécessaires de **40%** en intégrant une auto-évaluation de confiance. Le paradigme **LLM-as-Judge** (Zheng et al., NeurIPS 2023) étend cette logique à l'évaluation, GPT-4 atteignant **>80% d'accord** avec les préférences humaines.

### Meta-learning, calibration et Constitutional AI

Le **meta-learning** (MAML, Finn et al., ICML 2017) implémente la métacognition au niveau de l'apprentissage même : apprendre à apprendre. Bien que largement supplanté par l'in-context learning pour le NLP, ses principes persistent dans le meta-RL et l'adaptation rapide. **Open-MAML (2026)** étend MAML aux tâches ouvertes avec **1-7% de gains** sous changements dimensionnels.

La **calibration de confiance** reste un défi ouvert. Kadavath et al. (Anthropic, 2022) ont montré que les LLMs « savent (principalement) ce qu'ils savent », mais Xiong et al. (ICLR 2024) confirment une **surestimation systématique** de la confiance verbalisée. Les LLMs en débat démarrent à **72.9% de confiance moyenne** quand 50% serait rationnel.

La **Constitutional AI** (Bai et al., Anthropic, 2022) implémente une métacognition normative : le modèle évalue ses propres outputs contre des principes éthiques, puis s'auto-corrige. En 2026, Anthropic a étendu sa constitution de ~2,700 à **23,000 mots** (84 pages), passant du respect mécanique de règles au raisonnement éthique principiel.

---

## Les frontières émergentes : 2024-2026, l'ère de la métacognition agentique

### Les « thinking tokens » comme paradigme métacognitif

Tous les grands fournisseurs implémentent désormais des tokens de raisonnement : OpenAI (o-series, GPT-5), Anthropic (Claude 3.7+ extended thinking), DeepSeek-R1, Alibaba (QwQ), Google (Gemini thinking). Ce paradigme matérialise la distinction System 1/System 2 de Kahneman dans l'IA. **DeepSeek-R1** (arXiv:2501.12948, janvier 2025) a démontré l'émergence de comportements métacognitifs sans programmation explicite : par pur RL, le modèle a développé spontanément la vérification de ses étapes, la correction d'erreurs, l'exploration d'alternatives, et des « moments eurêka » de réévaluation. Le **Think tool d'Anthropic** (2025), distinct de l'extended thinking, offre des « pauses réflexives » pendant la génération, avec un **gain relatif de 54%** sur Tau-Bench pour les environnements complexes multi-outils.

### Chain-of-Verification et inner monologue : la vérification active

**Chain-of-Verification (CoVe)** (Dhuliawala et al., Meta AI, ACL 2024 Findings) implémente un protocole en 4 étapes — brouillon → questions de vérification → réponses indépendantes → synthèse vérifiée. Il **double la précision** sur les tâches de listes (0.17→0.36) et réduit les entités hallucinées de **2.95 à 0.68**. L'insight clé : les questions courtes de vérification sont répondues plus factuellement que les requêtes complexes originales. **Quiet-STaR** (Zelikman et al., Stanford/NotBad AI, 2024) entraîne les LLMs à générer des « pensées internes » avant chaque token, améliorant le raisonnement de Mistral 7B de **36.3% à 47.2%** et les maths de **5.9% à 10.9%**. **IM-RAG** (Yang et al., 2024) intègre un monologue intérieur modulaire avec Reasoner, Retriever, Refiner et Progress Tracker, surpassant le RAG baseline de **>40 points F1** sur HotPotQA (82.5 vs 41.2).

### L'optimisation du budget cognitif : savoir quand arrêter de penser

La métacognition temporelle — la conscience du coût cognitif — connaît une progression rapide. **TALE (Token-Budget-Aware LLM Reasoning)** (Han et al., ACL 2025 Findings) estime la difficulté du problème et alloue un budget de tokens en conséquence, réduisant les coûts de **67%** tout en maintenant 80.22% de précision. **SAGE** (arXiv:2602.08354) démontre que les modèles de raisonnement savent implicitement quand arrêter de penser, mais cette capacité est obscurcie par les paradigmes d'échantillonnage actuels. **REFRAIN** réduit les tokens de **20-55%** via un contrôleur multi-armed bandit. Commercialement, le paramètre `reasoning_effort` d'OpenAI et `budget_tokens` d'Anthropic incarnent cette métacognition temporelle côté utilisateur.

### Self-debugging et auto-correction du code

**Self-Debug** (Xinyun Chen et al., ICLR 2024) permet aux LLMs de debugger leur propre code via l'exécution + l'explication en langage naturel (« rubber duck debugging »), avec **+12% sur TransCoder/MBPP** et **+9% sur les problèmes les plus difficiles de Spider**. L'insight métacognitif : l'explication verbale du code par le modèle détecte des erreurs que l'exécution seule manque. Sur SWE-bench, les agents de pointe atteignent ~65% de résolution, et **Live-SWE-agent** (Xia et al., novembre 2025) booste les taux de **22.6 points** en créant des outils à la volée — une capacité métacognitive de reconnaître ce qui manque et de le créer.

---

## Tableau comparatif des méthodes métacognitives

| Méthode | Type de métacognition | Catégorie | Entraînement requis | Gain mesuré | Maturité |
|---|---|---|---|---|---|
| **Chain-of-Thought / ToT / GoT** | Monitoring (transparence) | ✅ Établie | Non | 4%→74% (ToT, Game of 24) | Universelle |
| **Reflexion** | Régulation (correction) | ✅ Établie | Non | 80%→91% (HumanEval) | Production |
| **Self-Refine** | Régulation (itération) | ✅ Établie | Non | ~20% absolu moyen | Production |
| **Self-Consistency** | Monitoring (validation croisée) | ✅ Établie | Non | +6-18% absolu | Standard |
| **MAML / Meta-learning** | Connaissance (apprendre à apprendre) | ✅ Établie | Oui (lourd) | 1-7% few-shot | Spécialisée |
| **Constitutional AI** | Régulation normative | ✅ Établie | Oui (SL+RL) | Réduction significative toxicité | Industrie |
| **Calibration de confiance** | Monitoring (auto-évaluation) | ✅ Établie | Variable | -40% coûts (CISC) | Croissante |
| **Chain-of-Verification** | Monitoring (vérification active) | 🔄 En développement | Non | +23% F1, -77% hallucinations | Production-ready |
| **Thinking tokens (o1/Claude/R1)** | Monitoring + régulation | 🔄 En développement | Oui (RL) | -43% hallucinations (ReMA) | Production |
| **Think tool (Anthropic)** | Régulation (pause réflexive) | 🔄 En développement | Non | +54% relatif (Tau-Bench) | Production |
| **Quiet-STaR** | Monitoring (pensée interne) | 🔄 En développement | Oui (self-training) | 36.3%→47.2% raisonnement | Recherche |
| **TALE / budget tokens** | Temporelle (coût cognitif) | 🔄 En développement | Non/Léger | -67% coûts tokens | Early production |
| **SAGE / REFRAIN** | Temporelle (arrêt optimal) | 🔄 En développement | Oui (RL) | -20-55% tokens | Recherche |
| **MASC (multi-agent)** | Monitoring collectif | 🔄 En développement | Non-supervisé | +8.47% AUC-ROC | Recherche |
| **Self-Debug (code)** | Régulation (debugging) | 🔄 En développement | Non | +12% (TransCoder/MBPP) | Production |
| **Metacognitive State Vector** | Monitoring (5 dimensions) | 🔄 En développement | Non | Validation en cours | Early research |
| **SMART (outil)** | Connaissance (limites outils) | 🔄 En développement | Oui | -24% tool use, +37% perf. | Recherche avancée |
| **Métacognition de swarm** | Monitoring collectif émergent | 🔮 Inexploré | — | — | Conceptuel |
| **Planning métacognitif** | Planification pré-raisonnement | 🔮 Inexploré | — | SOFAI : preuve de concept | Recherche précoce |
| **Transfert métacognitif cross-domaine** | Généralisation meta | 🔮 Inexploré | — | — | Conceptuel |
| **Métacognition émotionnelle** | Régulation motivationnelle | 🔮 Inexploré | — | EG-MRSI : théorique | Théorique |
| **Dashboard qualité en temps réel** | Monitoring multi-dimensionnel | 🔮 Inexploré | — | CISC + MCP : partiellement | Assemblage possible |

---

## Sept territoires inexplorés : hypothèses créatives fondées sur les données

### 1. Métacognition de swarm : l'intelligence collective consciente d'elle-même

C'est le territoire le plus vierge et potentiellement le plus impactant. La biologie offre des fondations solides : les abeilles utilisent des « signaux d'arrêt » collectifs analogues à l'inhibition neuronale (Seeley, *Honeybee Democracy*), et les fourmis modulent l'intensité de leurs phéromones en fonction de leur incertitude (Czaczkes & Heinze, Regensburg). Aucune recherche n'a formellement transposé ces mécanismes aux systèmes multi-agents IA. **Hypothèse** : un vecteur d'état métacognitif partagé entre agents — agrégeant confiance, conflits et complexité — avec des « signaux d'arrêt » inspirés des abeilles permettrait de détecter quand le raisonnement collectif dérive. Cela pourrait prévenir les cascades d'erreurs qui coûtent jusqu'à **51.9 points de performance** selon les tests d'injection de fautes de MASC.

### 2. Planning métacognitif : penser comment penser avant de penser

L'architecture **SOFAI** (Bergamaschi Ganapini et al., *npj Artificial Intelligence*, octobre 2025) est la plus avancée, avec un agent métacognitif arbitrant entre System 1 (rapide) et System 2 (délibératif). Mais l'arbitrage reste binaire. **Hypothèse** : un agent disposant d'une « bibliothèque de stratégies cognitives » (déduction, analogie, élimination, divide-and-conquer, travail à rebours, pattern matching) qu'il sélectionne dynamiquement avant chaque tâche surpasserait les approches à stratégie fixe. **Meta-Reasoning Prompting (MRP)** de Gao et al. (2024) pointe dans cette direction en guidant les LLMs à sélectionner dynamiquement leur méthode de raisonnement.

### 3. Métacognition émotionnelle : les signaux affectifs comme régulateurs

Le framework **EG-MRSI** (arXiv:2505.07757, mai 2025) est la seule proposition formelle intégrant motivation intrinsèque et métacognition récursive, mais il reste purement théorique. **Hypothèse** : des variables « émotionnelles » légères (frustration = f(échecs répétés), curiosité = f(nouveauté), confiance = f(précision récente)) modulant la sélection de stratégie amélioreraient la persistance sur les tâches difficiles et l'exploration. Le framework « Synthetic Emotions » (arXiv:2505.01462) conceptualise l'émotion comme architecture de contrôle facilitant la sélection d'actions sous incertitude — exactement ce dont la métacognition a besoin.

### 4. Métacognition des outils : savoir ce qu'on ne sait pas faire

**SMART** (Qian et al., ACL 2025 Findings) a identifié que les LLMs utilisent des outils **>30% du temps inutilement**. En entraînant les agents à reconnaître les limites de leurs connaissances, SMART réduit l'utilisation d'outils de **24%** tout en améliorant la performance de **37%**. **Hypothèse** : un « modèle dynamique de fiabilité des outils » maintenant un historique de précision, latence et pertinence par outil permettrait des sélections adaptatives — contournant les outils dégradés, préférant les outils rapides pour les requêtes simples, et basculant vers les outils fiables pour les décisions critiques.

### 5. Transfert métacognitif cross-domaine

**Didolkar et al. (2024-2025)** ont montré que les LLMs peuvent extraire et réutiliser des « comportements » abstraits, réduisant les tokens de **46%**. Mais ce transfert reste intra-domaine. **Hypothèse** : les stratégies métacognitives (estimation d'incertitude, quand s'arrêter, sélection de stratégie) sont plus transférables entre domaines que les compétences spécifiques, car elles opèrent à un niveau d'abstraction partiellement indépendant du domaine. Un module métacognitif entraîné en mathématiques devrait transférer ses capacités de calibration au raisonnement juridique ou médical.

---

## Applications concrètes : l'impact quantifié sur les projets AI-driven

### Réduction des hallucinations : les preuves s'accumulent

Les techniques métacognitives offrent les gains les plus documentés sur la fiabilité. En production, les LLMs hallucinent dans **15-38%** des cas (TechRxiv 2025), atteignant **69%** en QA juridique (GPT-3.5) et **88%** (LLaMA-2). CoVe réduit les entités hallucinées de **2.95 à 0.68** par requête. Self-RAG surpasse ChatGPT et LLaMA-2 augmenté en QA ouvert et vérification factuelle. Le framework **DMC (Decoupling Metacognition from Cognition)** (AAAI 2025) confirme la corrélation : plus forte capacité métacognitive = meilleure performance globale. L'intégration synergique (retrieval hybride + vérification ensemble + seuil adaptatif) réduit l'abstention de **95%** (40%→2%) sans augmenter les hallucinations.

### Gestion de projet AI-driven : la métacognition comme filet de sécurité

Le papier **"Agentic Metacognition: Designing a 'Self-Aware' Low-Code Agent for Failure Prediction and Human Handoff"** (arXiv, septembre 2025) propose une architecture à deux couches — agent primaire (exécution) + agent métacognitif (monitoring) — qui booste le taux de réussite de **7-8%** en détectant les boucles de répétition, la latence excessive et la complexité hors-limites. Le transfert à l'humain est reframé comme une **fonctionnalité de design** plutôt qu'un aveu d'échec. Contexte critique : seuls **25%** des initiatives IA délivrent le ROI attendu (IBM CEO Study), et seules **16%** ont été mises à l'échelle. La métacognition adresse directement le gap de fiabilité qui bloque l'adoption entreprise.

### Multi-agent : MASC et l'orchestration auto-correctrice

**MASC (Metacognitive Self-Correction for Multi-Agent Systems)** (octobre 2025) est le framework de référence. Il détecte les erreurs au niveau des étapes avec **+8.47% d'AUC-ROC** sur toutes les baselines (y compris supervisées), de manière non-supervisée et agnostique à l'architecture. Sans protection métacognitive, les tests d'injection de fautes montrent des chutes de performance allant jusqu'à **51.9 points** — démontrant que la métacognition n'est pas un luxe mais une nécessité structurelle pour les systèmes multi-agents.

### L'écosystème MCP métacognitif : cinq serveurs à connaître

Le **Model Context Protocol** (Anthropic, novembre 2024, donné à la Linux Foundation en décembre 2025) est devenu le standard d'interopérabilité IA, adopté par OpenAI, Google, Microsoft, avec **200+ serveurs** communautaires. Cinq implémentations intègrent des capacités métacognitives :

- **Sequential Thinking MCP** (officiel Anthropic) : résolution structurée pas-à-pas avec révision et branchement
- **mirror-mcp** (GitHub: toby/mirror-mcp) : outil `reflect` pour auto-réflexion récursive via MCP sampling
- **Vibe Check MCP** : « Chain-Pattern Interrupts » recherche-backed, couche de signaux métacognitifs, dosage recommandé de **10-20%** des étapes d'agent
- **MCP Thinking Server** (Malakanov) : 4 modes (linéaire, arborescent, dialectique, créatif) avec type `ThoughtType.METACOGNITION` dédié et outil `metacognitive_reflection`
- **MAS Sequential Thinking MCP** : 6 agents pensants spécialisés avec analyse de complexité IA et support de révision

---

## L'architecture « full-stack metacognitive » : une vision unifiée

Les sept territoires de métacognition s'empilent en couches complémentaires, formant une architecture complète :

1. **Pré-raisonnement** — Planning métacognitif : classifier le problème, sélectionner une stratégie cognitive, allouer un budget
2. **Pendant le raisonnement** — Monitoring en temps réel : confiance par étape, cohérence, ancrage factuel
3. **Temporel** — Budget cognitif adaptatif : savoir quand arrêter, quand approfondir
4. **Outils** — Sélection dynamique : évaluer fiabilité et utilité, éviter le tool overuse
5. **Émotionnel** — Signaux de régulation : frustration→changement de stratégie, curiosité→exploration
6. **Cross-domaine** — Transfert : réutiliser les stratégies métacognitives entre contextes
7. **Collectif** — Swarm metacognition : monitoring au niveau du groupe d'agents

---

## Le prompt de synthèse : construire le MCP de métacognition ultime

Ce qui suit est un prompt ultra-détaillé, intégrant toutes les découvertes de cette recherche, destiné à guider le développement du plugin/MCP de métacognition le plus complet possible.

---

```markdown
# PROMPT: Développement du MetaCog MCP Server — Le serveur MCP de métacognition le plus avancé

## CONTEXTE ET OBJECTIF

Tu es un architecte IA senior spécialisé dans les systèmes métacognitifs. Tu vas concevoir et
implémenter un serveur MCP (Model Context Protocol) appelé "MetaCog MCP" qui implémente une
architecture de métacognition à 7 couches pour les agents IA. Ce serveur doit être le système
de métacognition le plus complet jamais créé pour un agent IA.

Le MetaCog MCP doit transformer n'importe quel agent IA en un système capable de :
- Monitorer la qualité de son propre raisonnement en temps réel
- Planifier comment il va penser avant de penser
- Calibrer sa confiance et reconnaître ses limites
- Optimiser son budget cognitif (tokens, temps, coût)
- Évaluer dynamiquement ses outils et leur fiabilité
- Se corriger de manière itérative avec mémoire des erreurs passées
- Coordonner la métacognition collective dans les systèmes multi-agents

## ARCHITECTURE TECHNIQUE DU MCP SERVER

### Stack technologique
- Runtime : Node.js 20+ (TypeScript) OU Python 3.11+ (FastAPI + AsyncIO)
- Protocole : MCP SDK officiel (@modelcontextprotocol/sdk ou mcp Python SDK)
- Transport : stdio (local) + Streamable HTTP (remote)
- Stockage : SQLite pour mémoire épisodique + ChromaDB/Qdrant pour mémoire vectorielle
- Format : JSON-RPC 2.0 conforme au spec MCP

### Les 7 couches métacognitives (à implémenter comme outils MCP)

#### COUCHE 1 : metacog_plan — Planning métacognitif pré-raisonnement
Inspiré de : SOFAI (Bergamaschi Ganapini et al., 2025), Meta-Reasoning Prompting (Gao et al., 2024)

Outil MCP : `metacog_plan`
Paramètres d'entrée :
```json
{
  "task_description": "string — description de la tâche",
  "task_type_hint": "string? — optionnel : 'reasoning', 'creative', 'factual', 'code', 'decision'",
  "available_tools": "string[] — liste des outils MCP disponibles",
  "constraints": {
    "max_tokens": "number? — budget token maximum",
    "max_time_seconds": "number? — temps maximum",
    "max_cost_usd": "number? — coût maximum",
    "accuracy_priority": "number 0-1 — priorité précision vs vitesse"
  }
}
```
Logique interne :
1. Classifier le type de problème (utiliser un prompt léger de classification)
2. Estimer la difficulté (inspiré de TALE et DiffAdapt : classifieur de difficulté)
3. Sélectionner dans la bibliothèque de stratégies cognitives :
   - `step_by_step` : déduction séquentielle (CoT classique)
   - `tree_exploration` : exploration arborescente (ToT) — problèmes à multiples chemins
   - `verify_then_answer` : vérification d'abord (CoVe) — tâches factuelles critiques
   - `divide_and_conquer` : décomposition (tâches complexes multi-composants)
   - `analogical` : raisonnement par analogie (domaines nouveaux)
   - `adversarial` : auto-débat dialectique (décisions à enjeux élevés)
   - `rapid_retrieval` : System 1 rapide (tâches simples connues)
   - `iterative_refinement` : Self-Refine (tâches créatives, écriture, code)
4. Allouer le budget token basé sur la difficulté estimée et les contraintes
5. Configurer les seuils de monitoring (confiance minimum, max iterations)
Sortie :
```json
{
  "strategy": "string — stratégie sélectionnée",
  "estimated_difficulty": "number 0-1",
  "token_budget": "number",
  "monitoring_config": {
    "confidence_threshold": "number 0-1",
    "max_iterations": "number",
    "verification_required": "boolean",
    "tools_recommended": "string[]"
  },
  "rationale": "string — pourquoi cette stratégie"
}
```

#### COUCHE 2 : metacog_monitor — Monitoring en temps réel de la qualité du raisonnement
Inspiré de : Metacognitive State Vector (Sethi et al., 2025), CISC (ACL 2025), process reward models

Outil MCP : `metacog_monitor`
Paramètres d'entrée :
```json
{
  "reasoning_step": "string — l'étape de raisonnement courante",
  "step_number": "number",
  "previous_steps": "string[] — historique des étapes",
  "original_plan": "object — output de metacog_plan",
  "context": "string? — contexte additionnel"
}
```
Logique interne (5 dimensions du vecteur métacognitif) :
1. **Confiance** (0-1) : le modèle évalue sa certitude sur cette étape
   - Utiliser P(True) prompting + cohérence sémantique avec étapes précédentes
   - Détecter les marqueurs linguistiques d'incertitude ("perhaps", "might", "I think")
2. **Cohérence** (0-1) : consistance logique avec les étapes précédentes
   - Vérifier les contradictions avec le raisonnement antérieur
   - Score de similarité cosinus avec l'objectif initial
3. **Ancrage factuel** (0-1) : degré de fondement sur des faits vérifiables
   - Identifier les affirmations factuelles vs spéculatives
   - Flaguer les claims qui nécessiteraient une vérification externe
4. **Complexité** (0-1) : charge cognitive de cette étape
   - Longueur de l'étape relative au budget
   - Nombre de concepts nouveaux introduits
5. **Progression** (0-1) : avancement vers l'objectif
   - Distance sémantique entre l'état actuel et l'objectif
   - Détection de boucles (répétition de patterns similaires)
Sortie :
```json
{
  "metacognitive_state_vector": {
    "confidence": 0.82,
    "coherence": 0.91,
    "factual_grounding": 0.65,
    "complexity": 0.45,
    "progress": 0.60
  },
  "overall_quality_score": 0.77,
  "alerts": [
    {
      "type": "low_factual_grounding",
      "severity": "warning",
      "message": "L'étape 3 contient 2 claims non vérifiées",
      "recommendation": "Utiliser un outil de recherche pour vérifier"
    }
  ],
  "should_continue": true,
  "should_revise_current_step": false,
  "should_change_strategy": false,
  "tokens_consumed": 450,
  "tokens_remaining": 1550
}
```

#### COUCHE 3 : metacog_verify — Vérification active (Chain-of-Verification amélioré)
Inspiré de : CoVe (Dhuliawala et al., Meta AI, ACL 2024), Self-RAG, FaaF

Outil MCP : `metacog_verify`
Paramètres d'entrée :
```json
{
  "content_to_verify": "string — le contenu à vérifier",
  "verification_depth": "'quick' | 'standard' | 'thorough'",
  "domain": "string? — domaine de connaissance",
  "available_search_tools": "string[] — outils de recherche disponibles"
}
```
Logique interne (Factored CoVe amélioré) :
1. Extraire toutes les affirmations factuelles du contenu
2. Générer des questions de vérification indépendantes pour chaque affirmation
3. Répondre à chaque question INDÉPENDAMMENT (sans accès au contenu original —
   c'est le "factored" de CoVe qui empêche la copie d'hallucinations)
4. Si des outils de recherche sont disponibles, les utiliser pour les vérifications
5. Cross-référencer les réponses de vérification avec le contenu original
6. Produire un rapport de vérification avec score de confiance par affirmation
Sortie :
```json
{
  "verification_report": {
    "claims_found": 5,
    "claims_verified": 3,
    "claims_refuted": 1,
    "claims_uncertain": 1,
    "details": [
      {
        "claim": "string",
        "status": "verified | refuted | uncertain",
        "confidence": 0.95,
        "evidence": "string",
        "source": "string?"
      }
    ]
  },
  "corrected_content": "string — contenu révisé avec corrections",
  "overall_factuality_score": 0.72
}
```

#### COUCHE 4 : metacog_reflect — Auto-réflexion et apprentissage (Reflexion amélioré)
Inspiré de : Reflexion (Shinn et al., NeurIPS 2023), Self-Refine (Madaan et al., NeurIPS 2023), Gödel Agent

Outil MCP : `metacog_reflect`
Paramètres d'entrée :
```json
{
  "task": "string — la tâche originale",
  "output": "string — l'output produit",
  "outcome": "'success' | 'partial' | 'failure' | 'unknown'",
  "feedback": "string? — feedback externe optionnel",
  "error_details": "string? — détails de l'erreur si échec"
}
```
Logique interne :
1. Analyser l'écart entre l'intention et le résultat
2. Classifier le type d'erreur (factuelle, logique, stratégique, scope, complétude)
3. Générer une réflexion structurée en langage naturel :
   - Qu'est-ce qui a bien fonctionné ?
   - Qu'est-ce qui a échoué et pourquoi ?
   - Quelle stratégie alternative aurait été meilleure ?
   - Quel apprentissage en tirer pour le futur ?
4. Stocker la réflexion en mémoire épisodique (SQLite + embedding vectoriel)
5. Mettre à jour les préférences de stratégies (quel type de stratégie
   fonctionne mieux pour quel type de problème — apprentissage métacognitif)
6. Vérifier si des patterns d'erreurs récurrentes émergent
Sortie :
```json
{
  "reflection": {
    "success_factors": ["string"],
    "failure_analysis": "string",
    "error_type": "factual | logical | strategic | scope | completeness",
    "alternative_strategy": "string",
    "lesson_learned": "string",
    "recurring_pattern_detected": "boolean",
    "pattern_description": "string?"
  },
  "memory_entry_id": "string — ID de l'entrée en mémoire",
  "strategy_update": {
    "strategy": "string",
    "task_type": "string",
    "performance_delta": "number",
    "new_preference_score": "number 0-1"
  }
}
```

#### COUCHE 5 : metacog_calibrate — Calibration de confiance et estimation d'incertitude
Inspiré de : Xiong et al. (ICLR 2024), Kadavath et al. (Anthropic, 2022), PERAS framework

Outil MCP : `metacog_calibrate`
Paramètres d'entrée :
```json
{
  "question": "string — la question posée",
  "proposed_answer": "string — la réponse proposée",
  "reasoning_trace": "string? — le raisonnement qui a mené à cette réponse",
  "calibration_method": "'verbalized' | 'consistency' | 'multi_perspective' | 'all'"
}
```
Logique interne (3 méthodes de calibration combinées) :
1. **Confiance verbalisée** : demander explicitement au modèle d'estimer sa confiance
   avec correction du biais de surestimation (appliquer PERAS : prompt à basse
   conscienciosité pour contrebalancer la surestimation naturelle)
2. **Consistency check** : générer 3-5 réponses alternatives par sampling et
   mesurer la cohérence (self-consistency de Wang et al.)
3. **Multi-perspective** : reformuler la question de 2-3 manières différentes et
   vérifier si la réponse reste stable
4. Agréger les 3 signaux avec pondération apprise sur l'historique
5. Comparer avec l'historique de calibration pour corriger les biais systématiques
6. Si confiance < seuil → recommander abstention ou recherche supplémentaire
Sortie :
```json
{
  "calibrated_confidence": 0.73,
  "raw_confidence": 0.89,
  "calibration_adjustment": -0.16,
  "consistency_score": 0.80,
  "perspective_stability": 0.67,
  "recommendation": "proceed | verify | abstain | escalate_to_human",
  "uncertainty_decomposition": {
    "epistemic": 0.20,
    "aleatoric": 0.07
  },
  "known_unknowns": ["string — ce que le modèle sait qu'il ne sait pas"],
  "calibration_history_accuracy": 0.82
}
```

#### COUCHE 6 : metacog_tools — Métacognition sur les outils (Tool Metacognition)
Inspiré de : SMART (Qian et al., ACL 2025), AutoTool, TECTON (NAACL 2025)

Outil MCP : `metacog_tools`
Paramètres d'entrée :
```json
{
  "task": "string — la tâche en cours",
  "available_tools": [
    {
      "name": "string",
      "description": "string",
      "type": "string — catégorie de l'outil"
    }
  ],
  "tool_history": "object? — historique d'utilisation des outils (auto-rempli)"
}
```
Logique interne :
1. **Évaluation des limites de connaissance** (SMART) : le modèle peut-il répondre
   sans outil ? Si oui, NE PAS utiliser d'outil (réduction du tool overuse)
2. **Meta-raisonnement sur les outils** (TECTON) : raisonner sur la tâche →
   puis méta-raisonner sur ce raisonnement pour identifier les outils pertinents
3. **Modèle de fiabilité dynamique** : consulter l'historique pour chaque outil :
   - Taux de succès historique
   - Latence moyenne
   - Dernière utilisation et résultat
   - Fréquence d'utilisation (détecter la sur-utilisation)
4. **Composition intelligente** : si plusieurs outils nécessaires, planifier l'ordre
   optimal (parallélisation vs séquençage) basé sur les dépendances
5. **Post-utilisation** : après chaque utilisation d'outil, évaluer la qualité
   du résultat et mettre à jour le modèle de fiabilité
Sortie :
```json
{
  "tool_decision": "use_tool | no_tool_needed | escalate",
  "selected_tools": [
    {
      "name": "string",
      "reason": "string",
      "reliability_score": 0.92,
      "expected_utility": 0.85,
      "execution_order": 1
    }
  ],
  "tool_overuse_warning": false,
  "self_sufficient_confidence": 0.35,
  "tool_reliability_model": {
    "tool_name": {
      "historical_success_rate": 0.88,
      "avg_latency_ms": 450,
      "last_result_quality": 0.91
    }
  }
}
```

#### COUCHE 7 : metacog_collective — Métacognition collective pour multi-agents
Inspiré de : MASC (2025), Seeley's Honeybee Democracy, ReMA (NeurIPS 2025), MetaMind

Outil MCP : `metacog_collective`
Paramètres d'entrée :
```json
{
  "agent_id": "string — identifiant de l'agent courant",
  "agent_state": {
    "current_task": "string",
    "metacognitive_state_vector": "object — output de metacog_monitor",
    "confidence": "number",
    "progress": "number"
  },
  "swarm_state": "object? — état agrégé du swarm (auto-rempli)",
  "message_type": "'status_update' | 'stop_signal' | 'help_request' | 'conflict_report'"
}
```
Logique interne :
1. **Agrégation d'état** : combiner les vecteurs métacognitifs de tous les agents actifs
   en un vecteur de swarm (moyenne pondérée par progression)
2. **Détection de divergence** : si les agents convergent vers des conclusions
   contradictoires → déclencher un débat structuré
3. **Signaux d'arrêt collectifs** (inspiré des abeilles) : si >50% des agents
   signalent une confiance basse → arrêter et réévaluer la stratégie globale
4. **Allocation dynamique** : réassigner les agents peu productifs vers les
   sous-tâches où la confiance collective est la plus basse
5. **Détection de boucles collectives** : identifier quand le swarm tourne en rond
6. **Consensus métacognitif** : le swarm sait-il qu'il ne sait pas ?
Sortie :
```json
{
  "swarm_metacognitive_state": {
    "collective_confidence": 0.71,
    "coherence_across_agents": 0.85,
    "divergence_detected": false,
    "collective_progress": 0.55,
    "weakest_link": "agent_3 — low confidence on subtask B"
  },
  "collective_actions": [
    {
      "action": "reassign | debate | stop | continue | escalate",
      "target_agent": "string?",
      "reason": "string"
    }
  ],
  "stop_signal_active": false,
  "swarm_health_score": 0.78
}
```

### RESSOURCE MCP : metacog_memory — Mémoire métacognitive persistante

URI pattern : `metacog://memory/{category}/{id}`
Catégories :
- `reflections` — réflexions passées (output de metacog_reflect)
- `strategies` — préférences de stratégies apprises
- `tool_reliability` — modèle de fiabilité des outils
- `calibration_history` — historique de calibration
- `error_patterns` — patterns d'erreurs récurrents
- `swarm_logs` — logs d'interactions collectives

Implémentation : SQLite pour données structurées + ChromaDB pour recherche
sémantique dans les réflexions passées.

### PROMPT MCP : metacog_session — Template de session métacognitive

Le serveur expose un prompt MCP qui structure une session complète :

```
Session de raisonnement métacognitif — MetaCog Protocol v1.0

Phase 1 — PLAN (metacog_plan)
Avant toute action, analyse la tâche et planifie ta stratégie cognitive.
Estime la difficulté, sélectionne une stratégie, alloue un budget.

Phase 2 — EXECUTE + MONITOR (metacog_monitor)
Exécute ta stratégie. À chaque étape significative, appelle metacog_monitor
pour évaluer ton vecteur métacognitif. Si une alerte critique apparaît,
ARRÊTE et ajuste.

Phase 3 — VERIFY (metacog_verify)
Avant de finaliser, vérifie tes affirmations factuelles clés.
Utilise le mode "factored" : vérifie chaque claim indépendamment.

Phase 4 — CALIBRATE (metacog_calibrate)
Estime ta confiance calibrée finale. Si < 0.6, recommande une vérification
supplémentaire ou un transfert à un humain.

Phase 5 — REFLECT (metacog_reflect)
Après avoir reçu un feedback (ou auto-évalué le résultat),
réfléchis et stocke l'apprentissage en mémoire.

Phase 6 — TOOL AUDIT (metacog_tools) [si outils utilisés]
Évalue la pertinence et la qualité des outils utilisés.
Mets à jour le modèle de fiabilité.

Phase 7 — COLLECTIVE SYNC (metacog_collective) [si multi-agents]
Synchronise ton état métacognitif avec le swarm.
Vérifie l'alignement collectif.
```

### Configuration et paramètres globaux

```json
{
  "metacog_config": {
    "default_strategy": "step_by_step",
    "confidence_threshold_proceed": 0.6,
    "confidence_threshold_escalate": 0.3,
    "max_self_refine_iterations": 3,
    "max_verification_depth": "standard",
    "memory_retention_days": 30,
    "overconfidence_correction_factor": 0.85,
    "tool_overuse_threshold": 0.3,
    "swarm_stop_signal_threshold": 0.5,
    "token_budget_safety_margin": 0.1,
    "monitoring_frequency": "every_major_step",
    "verbose_logging": true
  }
}
```

## IMPLÉMENTATION : PRIORITÉS ET ORDRE

Phase 1 (MVP — 2 semaines) :
- metacog_plan (classification + sélection de stratégie)
- metacog_monitor (vecteur métacognitif 5 dimensions)
- metacog_reflect (réflexion basique + mémoire SQLite)

Phase 2 (Core — 4 semaines) :
- metacog_verify (CoVe factored)
- metacog_calibrate (3 méthodes de calibration)
- metacog_memory (ressource MCP avec ChromaDB)

Phase 3 (Advanced — 6 semaines) :
- metacog_tools (SMART + modèle de fiabilité dynamique)
- metacog_collective (protocole de swarm metacognition)
- metacog_session (prompt MCP orchestrateur)

## MÉTRIQUES DE SUCCÈS

1. Réduction des hallucinations : objectif -40% (baseline : CoVe = -23% F1)
2. Amélioration task success rate : objectif +10% (baseline : agentic metacog = +7-8%)
3. Réduction coûts tokens : objectif -50% (baseline : TALE = -67%)
4. Calibration : ECE < 0.10 (baseline : LLMs non-calibrés ~0.15-0.30)
5. Time-to-quality : réduction du nombre d'itérations nécessaires de -30%

## TESTS ET VALIDATION

- Benchmark sur GSM8K, MATH, HumanEval, HotPotQA avec/sans MetaCog MCP
- A/B testing sur des tâches complexes multi-étapes
- Test de calibration : accuracy vs. confidence plots
- Test de robustesse : injection de fautes et mesure de détection
- Test multi-agent : coordination de 3-5 agents avec et sans couche 7
- Test de mémoire : amélioration de performance au fil des sessions
```

---

## Conclusion : la métacognition comme avantage compétitif structurel

La métacognition en IA n'est plus un concept théorique. Les données convergent : les systèmes qui « pensent sur leur pensée » surpassent systématiquement ceux qui ne le font pas, avec des marges allant de **+7% en taux de réussite** à **+70 points de pourcentage sur des tâches complexes** (ToT). Trois insights émergent de cette recherche qui n'étaient pas évidents au départ.

Premièrement, **la métacognition est composable, pas monolithique**. CoT, Self-Consistency, Reflexion, CoVe et calibration occupent des niches métacognitives distinctes et se combinent multiplicativement — comme des modules d'un même système nerveux. Le gain n'est pas dans une seule technique mais dans leur orchestration intelligente, ce que le MetaCog MCP vise à systématiser.

Deuxièmement, **le problème principal n'est pas le manque de capacité métacognitive mais son manque de fiabilité**. Les LLMs *peuvent* monitorer leurs propres activations (Li Ji-An et al., Anthropic), mais seulement dans un sous-espace restreint. Ils *peuvent* calibrer leur confiance (Kadavath et al.), mais sont systématiquement surestimants. La CoT *peut* être inspectée pour la sécurité (papier multi-labs 2025), mais les modèles peuvent apprendre à la rendre trompeuse. L'enjeu des 2-3 prochaines années n'est pas d'ajouter de la métacognition, mais de la rendre **digne de confiance**.

Troisièmement, **la métacognition collective est le territoire le plus vierge et le plus prometteur**. Avec l'explosion des systèmes multi-agents en production (CrewAI : 60% du Fortune 500, LangGraph : 400+ entreprises), l'absence totale d'une couche de métacognition au niveau du swarm — alors que la biologie (abeilles, fourmis) en démontre l'efficacité depuis des millions d'années — représente une opportunité majeure. Le premier framework à implémenter des « signaux d'arrêt » bio-inspirés et un monitoring collectif de la qualité de raisonnement définira probablement le standard pour la prochaine génération de systèmes AI-driven.