# Projet - forme exacte recommandée

## Décision architecturale

Le projet ne doit **pas** être conçu comme un simple serveur MCP ni comme un gros prompt métacognitif. La bonne forme est un **monorepo local-first** avec cinq couches :

1. **Un noyau métacognitif local et agnostique** (runtime + état + politiques + vérificateurs)
2. **Une interface CLI très légère** pour les appels fréquents et peu coûteux en contexte
3. **Un plugin Claude Code natif** (skills, agents, hooks, éventuellement MCP/LSP)
4. **Des adaptateurs standards** (MCP, A2A, Agent SDK, éventuellement WASM plus tard)
5. **Un harnais d'évaluation / observabilité / ROI**

En pratique :

- le **cerveau** = runtime local
- la **façade UX dans Claude Code** = plugin natif
- le **MCP** = couche de compatibilité et d'intégration externe, pas le coeur
- le **CLI** = voie rapide pour les micro-vérifications métacognitives
- l'**eval** = composant produit à part entière, pas une annexe

---

## Pourquoi cette forme est la bonne

### 1. Le véhicule réel est le plugin Claude Code
Le produit final que tu vises est distribué comme **plugin Claude Code**. Donc la forme canonique du projet doit épouser les primitives natives déjà supportées :

- `skills/` pour les workflows et protocoles de pensée à chargement paresseux
- `agents/` pour les sous-agents spécialisés
- `hooks/` pour les garde-fous déterministes
- `.mcp.json` pour les intégrations externes quand elles sont nécessaires
- `.lsp.json` pour la code intelligence
- `.claude-plugin/plugin.json` pour la distribution

### 2. Le noyau ne doit pas dépendre de Claude Code
La vraie valeur du projet est la **métacognition portable** :

- Claude Code aujourd'hui
- Cursor / VS Code / JetBrains demain
- Agent SDK en headless / CI
- A2A pour le multi-agent
- WASM / marketplace / déploiement plus tard

Donc le noyau ne doit pas connaître les détails d'un IDE ou d'une marketplace.

### 3. Le MCP est utile, mais au bord du système
Le MCP est parfait pour :

- brancher des services externes
- publier des capacités vers d'autres agents
- offrir une compatibilité standard

Mais le coeur métacognitif doit rester **local, compact, stateful et peu verbeux**. Les opérations de type :

- audit d'un plan
- détection de boucle
- calibration de confiance
- décision stop / continue / escalate
- compression d'état

ne doivent pas dépendre d'une exposition permanente de gros schémas d'outils.

### 4. Le produit gamechanger n'est pas "je réfléchis plus"
Le wedge n'est pas seulement le raisonnement. Le wedge est :

- **preuve** avant action
- **calibration** de confiance
- **friction utile** quand le risque monte
- **observabilité** en temps réel
- **mémoire métacognitive** entre sessions
- **sécurité / gouvernance** au niveau outil, repo, permissions, secrets
- **human-in-the-loop** intelligent

---

## La proposition produit

### Nom de travail
Garde pour l'instant un nom de travail interne. Ne fige pas encore le branding. Structure d'abord le système.

### Positionnement
Ce produit doit être présenté comme un :

**Metacognitive Runtime for Coding Agents**

et non comme :

- un assistant de prompting
- un simple plugin MCP
- un copilote de complétion
- un outil de benchmark pur

### Différenciation attendue
Le système doit battre une solution de type Poetiq non pas seulement par la qualité de raisonnement, mais par :

1. **Local-first**
2. **Compatibilité multi-surface**
3. **Preuve / vérification / calibration**
4. **Contrôle du coût contexte / tokens**
5. **Métacognition humain + IA**
6. **Sécurité et auditabilité**
7. **Instrumentation produit et ROI**

---

## Architecture cible

## Vue d'ensemble

```text
repo/
├─ README.md
├─ LICENSE
├─ CHANGELOG.md
├─ docs/
│  ├─ vision.md
│  ├─ architecture.md
│  ├─ eval-framework.md
│  ├─ threat-model.md
│  └─ marketplace-plan.md
├─ core/
│  ├─ metacog-core/           # logique métier, état, politiques, scoring
│  ├─ metacog-daemon/         # service local stateful
│  ├─ metacog-cli/            # interface terminal faible coût
│  ├─ metacog-verifiers/      # tests, linters, fact-check, Z3, sandbox, etc.
│  ├─ metacog-memory/         # SQLite + compression + index
│  └─ metacog-observability/  # OTEL, métriques, traces, coût
├─ adapters/
│  ├─ claude-plugin/
│  │  ├─ .claude-plugin/
│  │  │  └─ plugin.json
│  │  ├─ skills/
│  │  ├─ agents/
│  │  ├─ hooks/
│  │  ├─ output-styles/
│  │  ├─ .mcp.json
│  │  ├─ .lsp.json
│  │  ├─ settings.json
│  │  └─ README.md
│  ├─ mcp-server/
│  ├─ a2a-agent/
│  ├─ agent-sdk/
│  └─ wasm/
├─ eval/
│  ├─ metacog-bench/
│  ├─ replay/
│  ├─ traces/
│  ├─ ab-tests/
│  └─ dashboards/
├─ examples/
│  ├─ rust-repo/
│  ├─ typescript-repo/
│  └─ python-repo/
├─ scripts/
│  ├─ dev/
│  ├─ packaging/
│  └─ release/
└─ marketplace/
   └─ .claude-plugin/
      └─ marketplace.json
```

---

## Ce qui vit dans le noyau

## 1. State model
Le noyau maintient un état structuré :

- `task_signature`
- `goal`
- `assumptions`
- `strategy`
- `confidence`
- `uncertainty`
- `evidence`
- `budget`
- `tool_risk`
- `human_risk`
- `stop_criteria`
- `escalation_reason`
- `memory_summary`

## 2. Boucle métacognitive
Le runtime doit implémenter une boucle explicite :

1. classify
2. assess difficulty
3. choose strategy
4. allocate budget
5. act / delegate
6. verify
7. calibrate
8. compact
9. decide stop / revise / escalate
10. learn

## 3. Vérificateurs
Le noyau doit router vers différents vérificateurs :

- compiler / test runner
- linter
- type checker
- factual verifier
- consistency checker
- dependency / security checker
- symbolic verifier (optionnel : Z3)
- sandbox runner (optionnel : Wasmtime)

## 4. Mémoire
Mémoire structurée et compacte, pas journal verbal infini :

- erreurs récurrentes
- patterns de succès
- règles locales du repo
- seuils de coût / confiance
- stratégie gagnante par type de tâche

## 5. Observabilité
Dès le MVP :

- tokens consommés / économisés
- latence par couche
- vérifications déclenchées
- faux positifs / faux négatifs
- taux de handoff humain
- coût vs valeur générée

---

## Ce qui vit dans le plugin Claude Code

## 1. Skills
Les skills portent les **protocoles de pensée réutilisables**. Ils doivent rester fins et ciblés.

Skills recommandées :

- `metacog:plan` - classification + stratégie + budget
- `metacog:challenge` - contre-hypothèse / devil's advocate
- `metacog:proof` - checklist de preuve avant action
- `metacog:compact` - compaction orientée apprentissage
- `metacog:bench` - exécution d'un protocole d'évaluation
- `metacog:threat-model` - revue sécurité / prompt injection / exfiltration
- `metacog:tool-audit` - analyse des outils à utiliser / éviter

## 2. Agents
Les agents portent les spécialisations longues :

- `skeptic` - cherche les hypothèses fragiles
- `verifier` - exécute et contrôle les preuves
- `repo-cartographer` - cartographie technique du repo
- `cost-auditor` - cherche les gaspillages de contexte / tokens
- `human-calibrator` - adapte le niveau d'explication et de friction

## 3. Hooks
Les hooks sont la couche déterministe.

Hooks recommandés :

- `SessionStart` - injecter le contexte minimal du projet
- `UserPromptSubmit` - classer la demande et le niveau de risque
- `PreToolUse` - décider allow / deny / ask / rewrite
- `PostToolUse` - absorber le résultat externe
- `PostToolUseFailure` - détecter les boucles d'échec
- `PreCompact` - résumer ce qu'il faut garder
- `Stop` - interdire l'arrêt si la preuve est insuffisante
- `SubagentStop` - contrôler la qualité des sous-agents
- `SessionEnd` - persister les leçons utiles

## 4. MCP
Utilisation recommandée : **fine et sélective**.

À garder pour :

- GitHub / GitLab / issue trackers
- observabilité externe (Sentry, etc.)
- bases de connaissances d'entreprise
- connecteurs distants

À éviter pour :

- toute micro-opération métacognitive répétée à chaque tour
- les tâches simples que le shell ou un script local fait mieux

## 5. LSP
Pour ce projet, les plugins LSP sont très utiles car ils donnent à Claude une navigation symbolique et des diagnostics automatiques après édition.

---

## Mapping fonctionnel - 7 couches vers primitives Claude Code

| Couche | Fonction | Primitive principale | Primitive secondaire |
|---|---|---|---|
| 0 | Meta-meta / auto-amélioration | runtime + eval | SessionEnd + bench |
| 1 | Planification pré-raisonnement | skill `metacog:plan` | agent `Plan` |
| 2 | Monitoring continu | hooks `PreToolUse/PostToolUse` | status line |
| 3 | Vérification active | runtime verifier + skill `proof` | Stop hook |
| 4 | Réflexion / mémoire | `PreCompact` + `SessionEnd` | auto memory |
| 5 | Calibration confiance / incertitude | runtime + Stop hook | challenge skill |
| 6 | Métacognition outil / risque | PreToolUse | MCP adapter |
| 7 | Collective / swarm | A2A adapter | custom agents |

---

## Ce qu'il ne faut pas faire

## 1. Ne pas commencer par un MITM sur l'API Anthropic
C'est une idée intéressante pour la recherche, mais trop fragile comme fondation produit. Commence avec les primitives officielles : plugin, hooks, skills, compaction, mémoire, settings.

## 2. Ne pas faire du MCP le cerveau
Le MCP est une interface. Ce n'est pas ton système nerveux.

## 3. Ne pas charger 40 skills et 20 serveurs MCP au démarrage
Même avec le Tool Search, tu veux une architecture sobre.

## 4. Ne pas mettre toute la logique dans `CLAUDE.md`
`CLAUDE.md` doit rester court, stable, transverse. Les protocoles spécialisés vont dans les skills et les règles modulaires.

## 5. Ne pas lancer trop tôt un multi-agent complexe
Démarre mono-agent + hooks + quelques skills + runtime local. Le multi-agent vient après instrumentation.

---

## Forme de repo recommandée - phase 1

## MVP 30 jours

### Objet produit
Un prototype utilisable localement qui fait déjà quatre choses très bien :

1. classer la tâche
2. détecter les actions risquées
3. vérifier les sorties par tests / linters / règles
4. mémoriser ce qui marche et ce qui casse

### Livrables MVP

- runtime local
- CLI
- plugin Claude Code minimal
- 4 skills
- 3 agents
- 5 hooks
- bench minimal
- dashboard local simple

### Skills MVP

- `metacog:plan`
- `metacog:challenge`
- `metacog:proof`
- `metacog:compact`

### Agents MVP

- `skeptic`
- `verifier`
- `repo-cartographer`

### Hooks MVP

- `UserPromptSubmit`
- `PreToolUse`
- `PostToolUse`
- `PreCompact`
- `Stop`

---

## Distribution recommandée

## Phase A - Standalone local

- configuration directe dans `.claude/`
- runtime lancé en local
- tests rapides
- skills et hooks affûtés

## Phase B - Plugin local

- plugin testé via `claude --plugin-dir ./adapters/claude-plugin`
- itération rapide
- aucune dépendance marketplace

## Phase C - Marketplace privée

- `marketplace.json` dans un repo privé
- déploiement équipe / organisation
- contrôle de version des plugins

## Phase D - Marketplace officiel Anthropic

- packaging propre
- README complet
- versioning strict
- dépendances vendorisées ou contenues dans le plugin

## Phase E - Portabilité AI-agnostic

- export des skills sous format Agent Skills
- adaptateur MCP public
- adaptateur A2A pour coordination multi-agent
- wrappers Agent SDK / CLI pour CI

---

## Cadre d'évaluation obligatoire

Sans bench, le produit restera une intuition brillante.

Le repo doit inclure dès le départ :

- **accuracy / task success**
- **hallucination detection rate**
- **calibration**
- **abstention intelligente**
- **token efficiency**
- **latence**
- **taux d'escalade humaine**
- **prévention d'incidents**
- **ROI**

### Bench interne minimale

Créer `MetaCog-Bench` avec 5 familles de scénarios :

1. recherche / synthèse longue
2. refactor multi-fichiers
3. incident / debug
4. sécurité / risque / secret
5. génération de plugin / packaging / release

---

## Décisions figées maintenant

1. **Monorepo**
2. **Noyau local-first**
3. **Plugin Claude Code natif**
4. **MCP fin, pas central**
5. **CLI pour les boucles fréquentes**
6. **LSP dès le début**
7. **Bench et observabilité dès le MVP**
8. **Roadmap marketplace native**
9. **Portabilité future par Agent Skills + A2A + MCP**
10. **`CLAUDE.md` court, skills ciblées, hooks déterministes**

---

## Fichiers clés à produire juste après

### Repo racine

- `README.md`
- `VISION.md`
- `ARCHITECTURE.md`
- `SECURITY.md`
- `EVAL.md`
- `ROADMAP.md`

### Claude Code

- `adapters/claude-plugin/.claude-plugin/plugin.json`
- `adapters/claude-plugin/hooks/hooks.json`
- `adapters/claude-plugin/skills/...`
- `adapters/claude-plugin/agents/...`
- `adapters/claude-plugin/README.md`

### Repo utilisateur

- `CLAUDE.md`
- `CLAUDE.local.md`
- `.claude/settings.json`
- `.claude/settings.local.json`
- `.claude/rules/*.md`

---

## Conclusion opérationnelle

La forme exacte du projet doit être :

> **un runtime métacognitif local, stateful et AI-agnostic, encapsulé dans un plugin Claude Code natif pour l'expérience utilisateur, et exposé ensuite par des adaptateurs MCP/A2A/Agent SDK pour la portabilité et la distribution.**

Autrement dit :

- **pas un simple MCP**
- **pas un simple plugin**
- **pas un simple prompt**
- mais un **système à double coeur** : runtime + wrapper Claude Code

C'est cette forme qui a le plus de chances de devenir à la fois :

- localement puissant
- économiquement sobre
- vendable en entreprise
- publiable sur marketplace
- portable hors Claude

---

## Sources consultées pour cette synthèse

### Documentation officielle

- [Claude Code - Create plugins](https://code.claude.com/docs/en/plugins)
- [Claude Code - Plugins reference](https://code.claude.com/docs/en/plugins-reference)
- [Claude Code - Skills](https://code.claude.com/docs/en/skills)
- [Claude Code - Hooks](https://code.claude.com/docs/en/hooks)
- [Claude Code - Settings](https://code.claude.com/docs/en/settings)
- [Claude Code - Memory](https://code.claude.com/docs/en/memory)
- [Claude Code - Best practices](https://code.claude.com/docs/en/best-practices)
- [Claude Code - Discover plugins](https://code.claude.com/docs/en/discover-plugins)
- [Claude Code - Plugin marketplaces](https://code.claude.com/docs/en/plugin-marketplaces)
- [Claude Code - Costs](https://code.claude.com/docs/en/costs)
- [Claude Code - Model config](https://code.claude.com/docs/en/model-config)
- [Claude Platform - Context windows](https://platform.claude.com/docs/en/build-with-claude/context-windows)
- [Claude Code - Subagents](https://code.claude.com/docs/en/sub-agents)
- [Claude Code - Output styles](https://code.claude.com/docs/en/output-styles)
- [Claude Code - MCP](https://code.claude.com/docs/en/mcp)

### Standards et écosystème

- [Model Context Protocol specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)
- [A2A protocol](https://a2a-protocol.org/latest/)
- [Agent Skills](https://agentskills.io/home)
- [mcporter](https://github.com/steipete/mcporter)

### Notes de préparation relues

- séries GEMINI / GROK / CLAUDE chargées dans cette conversation
