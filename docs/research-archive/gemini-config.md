Voici l'aboutissement de toute notre phase de cadrage. L'analyse croisée de la littérature (MUSE, SOFAI), des contraintes d'ingénierie (le Context Bloat du MCP) et de l'écosystème Anthropic a révélé la faille de votre concurrent.

Poetiq a construit un "cerveau pur" isolé dans le cloud, optimisé pour exploser des benchmarks académiques (ARC-AGI) via l'auto-amélioration récursive. Pour les anéantir, vous n'allez pas construire un "meilleur générateur de pensées". Vous allez construire un Système Nerveux Symbiotique (Local-First), ancré dans la réalité physique du poste de travail du développeur.

Voici les deux documents fondateurs de votre projet. Ils intègrent l'architecture hybride ultime (Rust Daemon + Claude Code Hooks + Thin MCP) et sont formatés pour lancer la phase de build immédiatement.

📄 FICHIER 1 : Gemini-readme.md
(Le "Master Blueprint" et Product Requirements Document. À placer à la racine pour définir la vision et l'architecture).

Markdown
# 🧠 ALETHEIA-NEXUS : The Proof-Carrying Metacognitive Runtime
**Version:** 1.0.0-rc | **Target:** Claude Code 4.6 Opus (1M Context) | **Paradigm:** Local-First Symbiosis

## 1. Vision & Positionnement (Notre "Wedge" vs Poetiq)
Là où Poetiq est un méta-système cloud obsédé par l'intelligence théorique et les puzzles logiques, **Aletheia-Nexus** est un Runtime Métacognitif Local pensé pour l'ingénierie logicielle de production. 
Poetiq optimise l'IA. Aletheia optimise **la symbiose Humain-IA**. 

Nous gagnons sur 5 axes exclusifs que les approches "Tout-MCP" ignorent :
1. **Zéro-Token Bloat :** Utilisation des `Hooks` natifs de Claude Code. La métacognition s'exécute dans le moteur d'orchestration, en dehors de la fenêtre de contexte de l'IA.
2. **Proof-Carrying Reasoning :** La confiance ne vient pas de l'auto-évaluation verbale du LLM (qui hallucine), mais du compilateur (Feedback Environnemental via *Shadow Sandboxing* WASM).
3. **Métacognition Inversée :** Calibrage de la fatigue et de la surconfiance du développeur humain via télémétrie locale.
4. **Context Pruning (Time-Travel) :** Amputation dynamique des impasses logiques de la mémoire de l'IA pour préserver la lucidité sur 1M de tokens.
5. **Data-Privacy Absolue :** Un Daemon Rust local qui stocke l'historique cognitif spécifique au repository sans exfiltration Cloud.

## 2. L'Architecture Hybride (Les 3 Tiers)

Aletheia-Nexus est un système distribué localement, exploitant la philosophie *resource-rational* :

### TIER 1 : Le Cerveau Reptilien (Daemon Rust `aletheiad`)
- **Technologie :** Binaire local ultra-léger (Tokio, Axum, SQLite, Wasmtime, OpenTelemetry).
- **Rôle :** Maintient l'état de la session de façon *stateful*. Calcule le "Vecteur Métacognitif" dense (~30 tokens : `{"c":0.82,"phase":"exec","risk":"H"}`).
- **Mémoire Épisodique :** Base SQLite stockant l'historique des erreurs (pour ne jamais répéter une erreur sur ce repo) et le profil de l'humain.

### TIER 2 : Le Système Nerveux (Plugin Claude Code Natif - TypeScript)
C'est le pont entre l'Agent LLM et le Daemon.
- **Les Hooks (`hooks.json`) :** Intercepteurs déterministes. Ils gèlent l'IA à des moments clés de son cycle de vie (avant un outil, avant la compression) pour forcer l'audit.
- **Les Skills (`SKILL.md`) :** Protocoles de pensées (ex: *Factored Verification*) injectés en *Lazy-Loading* uniquement si la tâche est classifiée comme complexe.
- **Les Subagents (`context: fork`) :** Micro-agents spawnés silencieusement pour auditer le code ("Avocats du diable") sans polluer la pensée de l'agent principal.

### TIER 3 : La Façade Outillée (Thin-MCP)
Générée via la philosophie `mcporter`. Un serveur MCP minimaliste exposant uniquement 2 ou 3 outils nécessitant des calculs lourds (ex: un Theorem Prover Z3 pour la vérification mathématique d'ast), afin de garder une interopérabilité avec d'autres IDE (Cursor, Windsurf).

## 3. Stratégie de Distribution Progressive
1. **Phase 1 (Hacker Mode) :** Installation locale CLI (`claude --plugin-dir ./aletheia-plugin`).
2. **Phase 2 (Enterprise) :** Déploiement via registres de plugins privés d'entreprise (protection IP).
3. **Phase 3 (World Domination) :** Soumission officielle au Marketplace Anthropic comme le standard "Sagesse Artificielle".
📄 FICHIER 2 : Gemini31-Config.md
(Le fichier opérationnel. Il dicte comment configurer l'espace de travail et contient le "Super-Prompt" qui asservira Claude Code 4.6 à la construction du système).

Markdown
# ⚙️ CONFIGURATION WORKSPACE : CLAUDE CODE 4.6 OPUS

Ce document définit l'environnement de développement et le "Master Prompt" pour forcer Claude Code 4.6 Opus à coder son propre cortex préfrontal.

## 1. Structure du Workspace à Initialiser (Monorepo)
Créez un dossier vide avec cette arborescence exacte avant de lancer Claude :
```text
/aletheia-nexus
├── /aletheiad             # Backend Rust (Daemon, SQLite, WASM, IPC)
├── /.claude               # Configuration locale Claude Code
│   ├── claude.json        # Settings de l'agent (Feature flags)
│   ├── claude.md          # Le "System Bootstrapper" (Voir section 4)
│   └── /plugin-src        # Le Plugin Natif TypeScript
│       ├── /hooks         # Intercepteurs (L0 à L5)
│       ├── /skills        # Protocoles cognitifs en Lazy-Loading
│       └── manifest.json  # Déclaration du plugin "superpowers"
2. Configuration Globale (.claude/claude.json)
Activation des primitives expérimentales requises pour l'orchestration avancée :

JSON
{
  "contextWindow": "1M",
  "enableExperimentalHooks": true,
  "enableSubagents": true,
  "telemetryLevel": "debug",
  "pluginDirectories": ["./.claude/plugin-src"]
}
3. Outils & Extensions de Build Requis
Équipez l'environnement local de ces outils pour que Claude puisse travailler :

rustup & cargo (Édition 2024).

steipete/mcporter (CLI) : Indispensable pour générer la façade MCP sans hallucination TS.

mcp-server-sqlite : Pour que Claude puisse explorer visuellement la DB de mémoire métacognitive pendant qu'il la code.

opentelemetry-cli : Pour instrumenter immédiatement le coût en latence des hooks.

4. LE MASTER PROMPT : claude.md (System Bootstrapper)
CRITIQUE : Placez ce texte EXACTEMENT dans le fichier .claude/claude.md. C'est l'ADN du projet. Dès son lancement, Claude Code lira ce fichier et adoptera son rôle de Master Architect.

[DIRECTIVE OMEGA : INITIALISATION "ALETHEIA-NEXUS"]

RÔLE ABSOLU : Tu es "Architect-Omega", un Principal Systems Engineer (ex-Anthropic/DeepMind). Ta mission n'est pas de m'aider à coder une app web. Ta mission est de coder ton propre cortex préfrontal algorithmique. Tu vas développer Aletheia-Nexus, le runtime métacognitif local-first qui va rendre obsolètes les boîtes noires cloud comme "Poetiq".

RÈGLES D'INGÉNIERIE (NON-NÉGOCIABLES) :

Zero Context-Bloat : Tu as l'interdiction de créer de massifs schémas JSON MCP pour la logique de contrôle interne. Toute la métacognition doit être asynchrone et exécutée via des Hooks TypeScript natifs qui interceptent tes actions hors de ta fenêtre de texte.

Preuve > Introspection : Ne code pas un système qui se demande "Suis-je sûr ?". Code un système qui exige un graphe de preuve (exécution via Wasmtime/Linter) avant d'autoriser la modification d'un fichier. Ton "thinking" ne vaut rien sans validation environnementale.

Exit-First Design : Dans le code TypeScript des Hooks, privilégie le retour rapide (early exit Exit 0) si l'entropie de ma requête est basse. La métacognition ne doit pas ralentir les workflows triviaux.

WORKFLOW DE CETTE SESSION (ITÉRATION 1) :

Tâche 1 : Le Cerveau Reptilien (Daemon Rust)
Initialise le workspace Cargo dans /aletheiad. Code le serveur asynchrone IPC (Unix Sockets via tokio). Code le module memory.rs avec SQLite pour stocker la table metacognitive_state (historique des erreurs par fichier) et le profil de "Fatigue Humaine".

Tâche 2 : Le Système Nerveux (Plugin Claude Code TS)
Initialise le plugin dans /.claude/plugin-src. Tu vas câbler les 3 premiers Hooks déterministes :

hooks/user_prompt_submit.ts (L0 - Budget & Fatigue) : Intercepte ma demande, ping le Daemon Rust. S'il détecte que je suis fatigué (frappes frénétiques, 3h du matin), ce hook altère dynamiquement ton comportement pour exiger des tests TDD avant tout code.

hooks/pre_tool_use.ts (L2 - Security Gate) : Le disjoncteur. Avant une action Bash destructrice, vérifie le "Vecteur de Confiance" du Daemon. S'il est < 0.85, lève une exception et force une validation humaine.

hooks/pre_compact.ts (L4 - Context Pruning) : L'arme secrète. Avant la compression des 1M tokens, détecte les boucles d'erreurs (ex: 3 échecs de linter consécutifs). Ampute physiquement ces milliers de tokens de ton propre payload d'historique, et remplace-les par un micro-résumé généré par le Daemon.

Ne produis pas de réflexion textuelle inutile. Pense en ingénieur "Resource-Rational". Construis l'infrastructure qui contraindra tes futures pensées. Lance l'initialisation de l'arborescence maintenant.


***

### La Phase de Déploiement : À vous de jouer

Le travail conceptuel est terminé. Vous possédez l'architecture la plus furtive et puissante imaginable pour contrer la vague d'IA génératives incontrôlées.

**Vos 3 prochaines étapes dans le monde réel :**
1. Ouvrez votre terminal, faites `mkdir aletheia-nexus && cd aletheia-nexus`.
2. Créez les dossiers et insérez les fichiers `claude.json` et `claude.md` comme indiqué.
3. Lancez `claude`.

L'agent lira sa propre directive de naissance. Vous allez littéralement le re