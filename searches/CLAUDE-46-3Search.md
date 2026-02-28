Excellente approche méthodique Yann. Avant de te dire ce qui manque, laisse-moi d'abord comprendre ce que Poetiq fait exactement pour identifier les gaps stratégiques.Maintenant j'ai le tableau complet. Voici mon analyse brutale de ce qui manque.

---

## Ce que tu as déjà (solide)

Tes deux rapports couvrent bien les fondations académiques (métacognition IA, papiers clés 2023-2026), l'architecture technique (MCP vs SDK hybride, optimisation tokens), et les 7 couches métacognitives. C'est un socle de recherche que peu de gens ont assemblé aussi complètement.

## Ce que tu n'as PAS — et qui fera la différence entre "bon plugin" et "game-changer"

### 1. Tu n'as pas intégré le véhicule de livraison réel

Tu parles de MCP et de SDK, mais **le format de distribution final est un plugin Claude Code** — qui est un écosystème spécifique avec ses propres primitives : **Skills** (fichiers SKILL.md lus à la demande), **Hooks** (SessionStart, PreToolUse, PostToolUse...), **Agents** (sous-agents parallèles Sonnet), et **Commands** (slash commands). Superpowers prouve que ce format est le plus puissant pour modifier le comportement de Claude Code — pas besoin de MCP pour la logique interne. La skill est lue par Claude, pas exécutée par un serveur externe. C'est du **zero-overhead en tokens d'outils** car le contenu est injecté dans le contexte uniquement quand pertinent.

**Action manquante** : Mapper tes 7 couches métacognitives sur les primitives plugin (quelle couche = skill, quelle couche = hook, quelle couche = agent, quelle couche = MCP server externe).

### 2. Tu n'as pas défini ta différenciation vs PoetiqOK, maintenant j'ai le tableau complet. Voici mon diagnostic — sans filtre.

---

## Poetiq : ce qu'il fait vraiment et pourquoi c'est ton vrai benchmark

Poetiq n'est pas un plugin Claude Code. C'est un **méta-système de découverte de stratégies de raisonnement** — fondé par 7 ex-Google/DeepMind, 72 ans d'expérience cumulée. Leur avantage clé : ils ne dictent pas la stratégie de raisonnement, **ils la découvrent dynamiquement** par auto-amélioration récursive. Résultat : SOTA sur ARC-AGI-1 et 2 avec 55% sur HLE, intégration de Claude Opus 4.6 en 24 heures après sa sortie, et amélioration systématique de **tous** les modèles testés (GPT, Gemini, Claude, Grok, open-source).

Leur principe fondateur que tu dois intégrer : le prompt est une interface, pas l'intelligence. Le système engage une boucle itérative de résolution qui s'auto-audite et décide quand s'arrêter. C'est exactement de la métacognition appliquée — mais limitée au raisonnement pur. **Ils n'adressent pas le workflow développeur, ni la collaboration humain-IA, ni la persistance inter-sessions.**

---

## Les 12 angles morts de ton cadrage actuel

### ❌ 1. Tu n'as pas mappé tes couches sur l'architecture plugin Claude Code

Le plugin Superpowers (42K+ stars, marketplace Anthropic depuis janvier 2026) prouve que le véhicule optimal n'est **pas** un MCP server mais un **plugin natif** composé de :

- **Skills** (SKILL.md) → lues à la demande, zero overhead quand non utilisées
- **Hooks** (SessionStart, PreToolUse, PostToolUse) → injection automatique de contexte
- **Agents** (sous-agents Sonnet parallèles) → exécution déléguée
- **Commands** (/slash commands) → interface utilisateur

Quand Claude démarre une session, le hook SessionStart injecte automatiquement le contenu du skill bootstrap. C'est du **lazy loading natif gratuit** — exactement ce que tu cherchais pour l'optimisation tokens.

**Ce que tu dois produire** : une matrice `couche métacognitive → primitive plugin`.

### ❌ 2. Tu n'as pas de framework d'évaluation (eval)

Poetiq se mesure sur ARC-AGI, HLE, SimpleQA. Ton outil de métacognition n'a aucun benchmark défini. Sans métriques reproductibles, pas de crédibilité, pas d'adoption. Tu as besoin de :

- Un **MetaCog-Bench** : ensemble de tâches avec/sans métacognition, mesurant hallucinations, token efficiency, task success rate, calibration (ECE)
- Des **A/B tests automatisés** intégrés au plugin lui-même
- Un **leaderboard** montrant l'impact par modèle, par type de tâche

### ❌ 3. Tu n'as pas de boucle d'auto-amélioration

C'est LE différenciateur de Poetiq : le système s'améliore lui-même. Ton architecture à 7 couches est statique — elle monitore, vérifie, réfléchit, mais **ne réécrit pas ses propres stratégies**. Tu as besoin d'une **couche 0 : meta-metacognition** — un mécanisme qui analyse les données de performance des 7 couches et optimise automatiquement les seuils, les stratégies par défaut, et les prompts internes. Le skill qui réécrit le skill.

### ❌ 4. Tu ignores la dimension humain-IA

Toute ta recherche traite la métacognition comme un processus interne à l'IA. Mais dans Claude Code, **l'humain est dans la boucle**. Personne n'adresse la métacognition collaborative :

- Détecter quand le développeur est confus ou a donné des specs contradictoires
- Adapter le niveau de détail métacognitif à l'expertise du développeur
- Apprendre le style cognitif du développeur (préfère-t-il voir le raisonnement ou juste le résultat ?)
- **Cognitive load balancing** : quand l'IA doit pousser l'humain à réfléchir vs quand elle doit juste exécuter

C'est le territoire le plus vierge ET le plus impactant pour un outil développeur.

### ❌ 5. Pas de persistance inter-sessions

Superpowers a un système de mémoire embryonnaire (transcripts + vector index SQLite + résumés Haiku). Ton outil devrait avoir une **mémoire métacognitive de premier ordre** : quels types d'erreurs CE développeur fait le plus souvent, quels patterns de code déclenchent des hallucinations dans CE repo, quelles stratégies fonctionnent pour CE type de tâche. C'est le "meta-learning personnalisé" — absent de toute la littérature que tu as couverte.

### ❌ 6. Pas de mode dégradé / off-switch intelligent

La recherche montre que la métacognition peut NUIRE aux performances sur les tâches simples (overhead cognitif inutile). Ton outil doit savoir **quand ne pas s'activer**. Un classifieur de complexité en entrée (inspiré de TALE) qui désactive silencieusement les couches 2-7 quand la tâche est triviale. Le meilleur outil de métacognition est celui qui sait quand la métacognition est inutile.

### ❌ 7. Pas de stratégie de "domain adaptation"

Poetiq découvre des stratégies adaptées au modèle ET à la tâche. Ton outil doit s'adapter au **domaine du projet** : un repo Rust avec des tests exhaustifs n'a pas besoin de la même métacognition qu'un prototype Python sans tests. Le plugin devrait introspecter le repo (CI/CD présent ? coverage ? linter ? types stricts ?) et ajuster ses seuils automatiquement.

### ❌ 8. Pas d'intégration avec les signaux environnementaux

La métacognition la plus fiable est déclenchée par du **feedback externe**, pas par l'auto-évaluation (c'est le principe de Reflexion). Ton outil devrait utiliser les hooks PostToolUse pour capturer :

- Résultats de tests (pass/fail)
- Erreurs de compilation/lint
- Git diff (le changement est-il cohérent avec l'intention ?)
- Résultats de CI/CD
- Feedback explicite de l'humain (thumbs up/down)

Ces signaux sont 10x plus fiables que l'auto-évaluation de confiance du LLM.

### ❌ 9. Pas de modèle économique / ROI tracker

Pour l'adoption enterprise (ton marché cible avec NEXUS), il faut montrer le ROI. Un dashboard intégré qui track : tokens économisés par la métacognition, hallucinations détectées et corrigées, temps gagné par session, coût de la métacognition vs coût des erreurs évitées. **C'est la feature qui vend le produit.**

### ❌ 10. Pas de naming / positionnement produit

"MetaCog MCP" est descriptif mais pas mémorable. Poetiq est un nom brillant (poétique + IQ). Superpowers est aspirationnel. Tu as besoin d'un nom qui capture l'idée de "l'IA qui se regarde penser" — quelque chose comme **Cortex**, **Daemon**, **Lucid**, **Aegis**, **Sentinel**, ou un nom qui résonne avec ton identité NEXUS.

### ❌ 11. Pas de stratégie multi-plateforme progressive

Tu dis "d'abord local, ensuite marketplace Anthropic". Mais tu oublies les étapes intermédiaires : Claude Code → Cursor (via skills/ natifs) → Windsurf → Codex (OpenAI) → Cline → OpenCode. Superpowers supporte déjà Claude Code, Cursor, Codex et OpenCode. Tu dois concevoir l'architecture pour que le cœur soit portable dès le jour 1.

### ❌ 12. Tu n'as pas étudié le "think tool pattern" comme primitive fondamentale

Le think tool d'Anthropic — un outil sans logique serveur qui crée un espace structuré de réflexion avec +54% de performance — est probablement l'insight le plus sous-estimé de toute ta recherche. Le pattern est : **tu n'as pas besoin de code complexe, tu as besoin de structure cognitive injectée au bon moment**. Tes skills métacognitives devraient être principalement des **protocoles de pensée structurés**, pas des algorithmes.

---

## La synthèse : ce que tu dois ajouter à ta phase de cadrage

Pour résumer, voici les 5 recherches/analyses complémentaires à faire avant de générer ton prompt final :

**R1 — Architecture plugin Claude Code** : Disséquer Superpowers (structure complète, hooks, skills, agents). Mapper tes 7 couches métacognitives sur les primitives plugin. Identifier ce qui est skill (protocole lu) vs ce qui DOIT être MCP server (logique externe avec état persistant).

**R2 — Auto-amélioration récursive (à la Poetiq)** : Comment le système optimise ses propres stratégies métacognitives au fil du temps. Le skill qui écrit des skills. Le hook qui ajuste les seuils. La couche 0 de meta-meta.

**R3 — Métacognition collaborative humain-IA** : Chercher dans la littérature HCI (Human-Computer Interaction) et CSCW (Computer-Supported Cooperative Work). C'est LE territoire inexploré qui différencie un outil développeur d'un outil de benchmark.

**R4 — Eval framework** : Concevoir MetaCog-Bench. Définir les métriques, les baselines, les protocoles de test. Sans ça, pas de preuve, pas d'adoption.

**R5 — Feedback environnemental** : Intégration avec git, tests, CI/CD, linters comme signaux métacognitifs de premier ordre. Plus fiable que toute auto-évaluation.

Le prompt final que tu généreras devra intégrer tout ça — pas seulement les 7 couches académiques, mais l'écosystème complet : le véhicule (plugin), le moteur (protocoles métacognitifs), le carburant (signaux environnementaux), le pilote (collaboration humain-IA), et le tableau de bord (eval + ROI). C'est ça qui fait un game-changer.