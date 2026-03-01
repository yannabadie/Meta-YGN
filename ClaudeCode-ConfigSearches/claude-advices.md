# Configuration optimale de Claude Code 4.6 Opus — Guide exhaustif février 2026

Claude Code 4.6 Opus avec fenêtre de **1M tokens** (beta, pricing premium au-delà de 200K) représente l'outil de développement assisté par IA le plus puissant disponible en février 2026. Ce guide couvre l'ensemble de l'écosystème : configuration, plugins, skills, hooks, MCP, mémoire, CLI et bonnes pratiques, en distinguant systématiquement les sources **officielles Anthropic** des **contributions communautaires**.

---

## 1. CLAUDE.md — le fichier le plus important de votre projet

Le fichier CLAUDE.md est le point de configuration à plus fort effet de levier dans Claude Code. Il fonctionne comme un onboarding pour l'agent : il lui dit QUOI (stack technique, structure), POURQUOI (objectif du projet) et COMMENT (workflows, commandes). La recommandation officielle est de **garder le fichier sous 500 lignes**, et la communauté converge vers **50 à 150 lignes** dans le fichier racine avec des imports `@` pour le reste.

### Hiérarchie de chargement

Claude Code charge les fichiers CLAUDE.md selon un système à deux mécanismes. Au démarrage, il remonte depuis le répertoire courant jusqu'à la racine du filesystem et charge tous les CLAUDE.md trouvés. Ensuite, les fichiers CLAUDE.md dans les sous-répertoires sont chargés **à la demande** quand Claude accède à des fichiers dans ces répertoires — c'est le lazy loading natif.

| Emplacement | Usage | Partage |
|---|---|---|
| `~/.claude/CLAUDE.md` | Global utilisateur | Privé |
| `./CLAUDE.md` (racine projet) | Instructions projet | Git (équipe) |
| `./CLAUDE.local.md` | Overrides personnels | `.gitignore` |
| `.claude/CLAUDE.md` | Alternative projet | Git |
| `parent/CLAUDE.md` | Monorepo racine | Git |
| `child/CLAUDE.md` | Lazy-loaded | Git |

### Structure recommandée et imports

```markdown
# Project: MyApp
See @README.md for project overview
See @docs/git-instructions.md for Git workflow

## Code Style
- Use ES modules (import/export), not CommonJS (require)
- Destructure imports when possible

## Workflow
IMPORTANT: Always typecheck after code changes with `npm run typecheck`
YOU MUST run single tests, not the full suite, for performance

## Testing
- Test framework: Vitest
- Run: `npm run test -- path/to/test`
- E2E: `npm run test:e2e`

## Architecture
- /src/api — Express REST routes
- /src/services — Business logic
- /src/models — TypeORM entities
```

Les imports `@` permettent de référencer des fichiers externes sans les embarquer systématiquement en mémoire. L'approche recommandée : ne pas écrire `@docs/file.md` pour tout (cela s'intègre à chaque session), mais plutôt « Pour l'usage complexe, voir `path/to/docs.md` » — Claude ira lire le fichier uniquement quand nécessaire.

### Auto Memory — le compagnon de CLAUDE.md

Claude Code maintient automatiquement un système de mémoire dans `~/.claude/projects/<project>/memory/`. Le fichier `MEMORY.md` (index, 200 premières lignes chargées au démarrage) est accompagné de fichiers thématiques (`debugging.md`, `api-conventions.md`) chargés à la demande. Ce système se toggle avec `/memory` et se désactive via `CLAUDE_CODE_DISABLE_AUTO_MEMORY`.

---

## 2. Settings.json — configuration complète et hiérarchie

Les settings suivent une hiérarchie stricte de précédence, du plus prioritaire au moins prioritaire : **managed policies** (entreprise, déployées via MDM) → **arguments CLI** → **`.claude/settings.local.json`** (projet, personnel) → **`.claude/settings.json`** (projet, partagé en git) → **`~/.claude/settings.json`** (utilisateur global).

### Exemple complet de settings.json

```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "model": "claude-opus-4-6-20260115",
  "permissions": {
    "allow": [
      "Bash(npm run lint)",
      "Bash(npm run test:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)",
      "Read(~/.zshrc)"
    ],
    "ask": [
      "Bash(git push:*)",
      "Bash(docker:*)"
    ],
    "deny": [
      "Bash(curl:*)",
      "Bash(rm -rf *)",
      "Read(./.env)",
      "Read(./.env.*)",
      "Read(./secrets/**)",
      "WebFetch"
    ],
    "defaultMode": "acceptEdits",
    "additionalDirectories": ["../shared-lib/"]
  },
  "hooks": { },
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",
    "MAX_THINKING_TOKENS": "16000",
    "OTEL_METRICS_EXPORTER": "otlp"
  },
  "sandbox": {
    "enabled": true,
    "autoAllowBashIfSandboxed": true,
    "network": {
      "allowLocalBinding": true,
      "httpProxyPort": 8080
    }
  },
  "enabledPlugins": {
    "superpowers@superpowers-marketplace": true,
    "context7-plugin@context7-marketplace": true
  },
  "enableAllProjectMcpServers": true,
  "includeCoAuthoredBy": true,
  "cleanupPeriodDays": 30,
  "outputStyle": "Explanatory"
}
```

### Paramètres clés documentés

Les paramètres couvrent : `model` (override du modèle), `apiKeyHelper` (script d'auth custom), `permissions` (allow/ask/deny avec prefix matching pour Bash), `hooks`, `env` (variables d'environnement injectées), `sandbox` (isolation des commandes), `enabledPlugins`, `enableAllProjectMcpServers`/`enabledMcpjsonServers`/`disabledMcpjsonServers` (contrôle MCP), `statusLine` (barre de statut custom), `outputStyle`, `forceLoginMethod`, et `companyAnnouncements` (messages au démarrage pour les équipes).

---

## 3. Context window 1M tokens — stratégies de gestion

Le contexte 1M tokens est disponible **en beta** pour Opus 4.6 et Sonnet 4.6, avec un pricing premium au-delà de 200K (**$10/$37.50 par MTok** en input/output). L'auto-compaction se déclenche à **98%** de la fenêtre effective. La compaction préserve le transcript complet sur disque tout en utilisant un résumé compact pour le contexte actif.

### Stratégies officielles de gestion du contexte

La gestion du contexte constitue **la contrainte fondamentale** selon la documentation officielle. Les subagents sont l'outil le plus puissant car ils opèrent dans des fenêtres de contexte séparées et renvoient des résumés. Démarrer des sessions fraîches par tâche avec `/clear`, compacter proactivement à ~70% d'utilisation, déléguer la recherche aux subagents, garder CLAUDE.md sous ~500 lignes, préférer les outils CLI aux MCP servers pour l'efficacité du contexte, et désactiver les MCP servers inutilisés — voilà les recommandations clés.

### Extended thinking et budget tokens

L'**extended thinking est activé par défaut** avec un budget de 31 999 tokens. Pour Claude 4.6, le mode **adaptive thinking** (`thinking: {type: "adaptive"}`) est désormais recommandé, et `budget_tokens` est **déprécié** sur Opus 4.6 / Sonnet 4.6. Les niveaux d'effort sont `low`, `medium`, `high` (défaut), `max`. On configure via `MAX_THINKING_TOKENS` (variable d'environnement), `/config` (toggle on/off), ou `/model` (ajuster l'effort). La communauté utilise les mots-clés magiques dans les prompts : « think » < « think hard » < « think harder » < « ultrathink » — chacun augmente le budget de raisonnement.

### Hook PreCompact pour injecter du contexte

```json
{
  "hooks": {
    "PreCompact": [
      {
        "matcher": "auto",
        "hooks": [
          {
            "type": "command",
            "command": "cat .claude/compaction-context.md",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

Le stdout du hook est injecté dans les instructions de compaction, permettant de préserver des informations critiques spécifiques pendant la compaction automatique. Le matcher accepte `"auto"` ou `"manual"`.

### Variables d'environnement de performance

| Variable | Usage |
|---|---|
| `MAX_THINKING_TOKENS` | Budget thinking (ex: `8000`, `16000`) |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | Tokens de sortie max |
| `BASH_DEFAULT_TIMEOUT_MS` | Timeout commandes bash |
| `BASH_MAX_OUTPUT_LENGTH` | Chars max avant troncature bash |
| `MAX_MCP_OUTPUT_TOKENS` | Tokens max pour réponses MCP (défaut: 25000) |

---

## 4. Plugins — un écosystème de 9 000+ extensions

L'écosystème de plugins Claude Code a explosé depuis le lancement du système en octobre 2025. Le marketplace officiel (**`anthropics/claude-plugins-official`**, 7 500 stars) est pré-configuré et accessible sans setup. L'installation se fait via `/plugin install {name}@{marketplace}` ou `claude plugin install`.

### Plugins officiels Anthropic notables

| Plugin | Installs | Description |
|---|---|---|
| **Frontend Design** | ~96 000 | Interfaces production-grade sans esthétique AI générique |
| **Code Review** | ~50 000 | Review PR multi-agents avec scoring par confiance |
| **Playwright** | ~28 000 | Automatisation browser et tests E2E (Microsoft) |
| **Security Guidance** | ~25 000 | Détection de 9 patterns de sécurité via hooks PreToolUse |
| **Ralph Loop** | ~57 000 | Sessions de coding autonomes avec pattern stop-hook |
| **Feature Dev** | — | Workflow guidé avec agents code-explorer, code-architect, code-reviewer |
| **Hookify** | — | Convertit des patterns de conversation en hooks automatiquement |
| **Plugin Dev Toolkit** | — | 7 skills experts et workflow en 8 phases pour créer des plugins |
| **TypeScript LSP** | — | Language server pour intelligence de code TS/JS |

### Superpowers (obra/Jesse Vincent) — le plugin communautaire dominant

Avec **40 900 stars** GitHub et **118 874 installs**, Superpowers est le plugin tiers le plus influent. C'est un framework de skills agentiques qui transforme Claude Code en partenaire de développement discipliné, imposant un workflow structuré : brainstorm → plan → implement (avec TDD, développement par subagents et code review).

**Installation :**
```bash
/plugin marketplace add obra/superpowers-marketplace
/plugin install superpowers@superpowers-marketplace
```

**Architecture du répertoire :**
```
superpowers/
├── .claude-plugin/
│   └── plugin.json          # Manifest (name, version, author)
├── agents/                   # Définitions de subagents
├── commands/                 # Slash commands (/brainstorm, /write-plan, /execute-plan)
├── skills/                   # Skills SKILL.md (7+ skills core)
│   ├── brainstorming/
│   ├── test-driven-development/
│   ├── systematic-debugging/
│   ├── subagent-driven-development/
│   ├── verification-before-completion/
│   ├── writing-skills/
│   └── getting-started/
├── hooks/                    # Session-start bootstrap hook
├── lib/                      # Code helper
└── scripts/
```

Le mécanisme est **extrêmement léger en tokens** : le bootstrap core consomme moins de 2K tokens, et les skills sont chargés à la demande via des scripts shell. Le hook `SessionStart` injecte un message qui force Claude à lire le skill `getting-started`, qui lui enseigne le framework. Les skills se déclenchent automatiquement selon le contexte de la tâche.

### Autres plugins populaires

**Context7** (~71 000 installs, par Upstash) livre de la documentation à jour et version-spécifique directement dans le contexte via MCP. **Task Master AI** (`eyaltoledano/claude-task-master`) offre un système de gestion de tâches piloté par IA avec 49 slash commands, 3 agents spécialisés, et détection de 13 IDEs. **Firecrawl** fournit du web scraping et crawling. **Trail of Bits Skills** (1 300 stars) apporte des workflows de recherche de vulnérabilités.

Les répertoires communautaires incluent **ComposioHQ/awesome-claude-plugins** (800+ plugins), **skills.sh**, **claude-plugins.dev**, et **buildwithclaude.com** (400+ extensions).

---

## 5. Architecture complète d'un plugin Claude Code

### Structure standard

```
mon-plugin/
├── .claude-plugin/
│   └── plugin.json          # Seul fichier ici (manifest)
├── commands/                 # Slash commands (.md avec frontmatter YAML)
├── agents/                   # Subagents (.md avec frontmatter YAML)
├── skills/                   # Skills (sous-répertoires avec SKILL.md)
│   └── mon-skill/
│       ├── SKILL.md
│       ├── scripts/
│       └── references/
├── hooks/
│   └── hooks.json            # Configuration des event handlers
├── .mcp.json                 # Serveurs MCP du plugin
├── .lsp.json                 # Serveurs LSP du plugin
├── scripts/                  # Utilitaires partagés
└── README.md
```

**Règle critique** : tous les répertoires de composants (`commands/`, `agents/`, `skills/`, `hooks/`) doivent être au **niveau racine du plugin**, PAS imbriqués dans `.claude-plugin/`. Seul `plugin.json` va dans `.claude-plugin/`.

### plugin.json — le manifest

```json
{
  "name": "mon-plugin",
  "version": "1.2.0",
  "description": "Description brève du plugin",
  "author": {
    "name": "Auteur",
    "email": "auteur@example.com"
  },
  "repository": "https://github.com/org/mon-plugin",
  "license": "MIT",
  "keywords": ["tdd", "security", "workflow"]
}
```

Seul `name` est requis. Le manifest est même optionnel : Claude Code auto-découvre les composants dans les emplacements par défaut. La variable `${CLAUDE_PLUGIN_ROOT}` est disponible dans les hooks et scripts pour les chemins relatifs au plugin.

---

## 6. Skills — le système de chargement progressif

### Le standard Agent Skills (agentskills.io)

**agentskills.io** est un **standard ouvert officiel** publié le 18 décembre 2025 par Anthropic, maintenu sous licence Apache 2.0 / CC-BY-4.0. Il est adopté par **26+ plateformes** : Claude Code, GitHub Copilot, Cursor, OpenAI Codex, Gemini CLI, VS Code, Amp, Goose, et d'autres.

### SKILL.md — format et structure

```markdown
---
name: api-conventions
description: >
  API design patterns and conventions for this codebase. 
  Use when creating new endpoints, modifying API routes, 
  or reviewing API-related code changes.
version: 1.0.0
context: fork
allowed-tools: Read Grep Glob
---

# API Conventions

## REST Endpoints
- Use plural nouns for resources: `/users`, `/orders`
- Nest sub-resources: `/users/{id}/orders`
- Version prefix: `/v1/`

## Error Handling
Always return structured error responses:
```json
{"error": {"code": "NOT_FOUND", "message": "User not found"}}
```

## References
For complete OpenAPI spec, see `docs/openapi.yaml`
```

### Champs frontmatter complets

| Champ | Requis | Description |
|---|---|---|
| `name` | Oui | Max 64 chars, lowercase/chiffres/tirets, doit correspondre au nom du répertoire parent |
| `description` | Oui | Max 1024 chars. Décrit QUOI et QUAND utiliser. Critique pour la découverte |
| `license` | Non | Nom de licence ou référence à fichier LICENSE |
| `compatibility` | Non | Prérequis environnement (ex: « Requires git, docker ») |
| `metadata` | Non | Mapping clé-valeur arbitraire |
| `allowed-tools` | Non | Liste d'outils pré-approuvés séparés par espaces |
| `context` | Non | `fork` pour exécution en subagent isolé |
| `disable-model-invocation` | Non | `true` empêche le déclenchement automatique |

### Le pattern de lazy loading en 3 niveaux

C'est le **principe architectural fondamental** du système de skills. Il fonctionne en trois tiers :

**Niveau 1 — Métadonnées** (~100 tokens/skill) : au démarrage, Claude Code scanne les répertoires de skills et parse uniquement le frontmatter YAML. Les noms et descriptions sont injectés dans la liste `<available_skills>` du tool Skill.

**Niveau 2 — Instructions** (<5 000 tokens) : quand Claude détermine qu'un skill est pertinent via son raisonnement (pur LLM, pas d'embeddings ni de classification algorithmique), il invoque le tool `Skill` qui lit le corps complet du SKILL.md et l'injecte dans la conversation.

**Niveau 3 — Ressources** (illimité) : les fichiers dans `scripts/`, `references/`, `assets/` ne sont lus qu'à l'exécution, à la demande.

Avec 20 skills de 2 000 tokens chacun, le chargement eager consommerait **40 000 tokens**. Le lazy loading n'utilise que ~2 000 tokens pour les métadonnées et charge uniquement les ~2 000 tokens du skill actif — soit une économie de **95%**.

### Skills natifs Claude Code vs plugins

| Source | Emplacement | Portée |
|---|---|---|
| Personal Skills | `~/.claude/skills/` | Tous les projets |
| Project Skills | `.claude/skills/` | Partagé via git |
| Plugin Skills | Bundlés avec les plugins installés | Quand le plugin est actif |

Les skills officiels Anthropic (`github.com/anthropics/skills`) incluent des skills pour documents (`docx`, `pdf`, `pptx`, `xlsx`), design créatif (`frontend-design`, `canvas-design`), développement (`mcp-builder`, `skill-creator`), et entreprise (`brand-guidelines`, `internal-comms`).

---

## 7. Hooks — le système déterministe complet

Les hooks sont le mécanisme **déterministe** de Claude Code : ils se déclenchent à chaque fois à l'événement exact, sans exception. Contrairement aux instructions CLAUDE.md qui sont **consultatives** et que Claude peut ignorer sous pression de contexte, les hooks garantissent l'exécution.

### Liste COMPLÈTE des événements hooks

| Événement | Déclenchement | Matchers | Usage principal |
|---|---|---|---|
| **PreToolUse** | Après création des paramètres, avant exécution | Oui (noms d'outils) | Approuver/refuser/modifier les appels |
| **PostToolUse** | Après exécution réussie de l'outil | Oui (noms d'outils) | Feedback, formatting, linting |
| **PostToolUseFailure** | Après échec de l'outil | Oui (noms d'outils) | Gestion d'erreurs |
| **PermissionRequest** | Quand le dialogue de permission s'affiche | Oui (noms d'outils) | Auto-allow/deny permissions |
| **UserPromptSubmit** | Quand l'utilisateur soumet un prompt | Non | Valider/bloquer prompts, injecter contexte |
| **Stop** | Quand l'agent principal finit de répondre | Non | Forcer continuation, vérifications qualité |
| **SubagentStop** | Quand un subagent (Task) finit | Non | Validation de complétion des subagents |
| **PreCompact** | Avant opération de compaction | Oui (`manual`, `auto`) | Injecter instructions de compaction |
| **SessionStart** | Au démarrage/reprise de session | Oui (`startup`, `resume`, `clear`, `compact`) | Charger contexte, configurer env |
| **SessionEnd** | À la fin de session | Non | Cleanup, logging |
| **Notification** | Quand Claude Code envoie une notification | Oui (type) | Alertes custom |
| **TaskCompleted** | Quand une tâche est complétée | Non | Workflow automation |

Les matchers pour les outils acceptent : `Bash`, `Edit`, `Write`, `Read`, `Glob`, `Grep`, `Task`, `WebFetch`, `WebSearch`, les outils MCP (`mcp__<server>__<tool>`), des regex (`Edit|Write`, `mcp__memory__.*`), et `*` pour tout matcher. Les matchers Notification acceptent : `permission_prompt`, `idle_prompt`, `auth_success`, `elicitation_dialog`.

### Format hooks.json avec exemples concrets

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/scripts/auto-format.sh",
            "timeout": 30
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo $CLAUDE_TOOL_INPUT | jq -r '.command' | grep -qE '(rm -rf|sudo|chmod 777)' && echo '{\"hookSpecificOutput\":{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"deny\",\"permissionDecisionReason\":\"Dangerous command blocked\"}}' && exit 0 || exit 0",
            "timeout": 5
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo '{\"decision\":\"block\",\"reason\":\"Please verify all tests pass before stopping\"}'"
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "cat .claude/session-context.md"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo \"Current branch: $(git branch --show-current), Modified files: $(git diff --name-only | head -5 | tr '\\n' ', ')\"",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

### Entrée/sortie des hooks

Chaque hook reçoit en **stdin** un JSON contenant `session_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`, `tool_name`, `tool_input`, et `tool_use_id`. Les codes de sortie déterminent le comportement : **exit 0** = succès (stdout montré en mode verbose), **exit 2** = erreur bloquante (stderr renvoyé à Claude), **autre** = erreur non-bloquante.

Pour PreToolUse, le hook peut retourner un JSON de décision :
```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow",
    "permissionDecisionReason": "Auto-approved: docs file only"
  }
}
```

Les hooks supportent aussi le type **`prompt`** qui utilise Haiku pour évaluer une condition via LLM, utile pour des vérifications sémantiques.

---

## 8. MCP Servers — configuration et écosystème

### Configuration .mcp.json

Le fichier `.mcp.json` à la racine du projet définit les serveurs MCP partagés avec l'équipe via git. Il supporte l'expansion de variables d'environnement avec `${VAR}` et `${VAR:-default}`.

```json
{
  "mcpServers": {
    "github": {
      "type": "http",
      "url": "https://api.githubcopilot.com/mcp/",
      "headers": {
        "Authorization": "Bearer ${GITHUB_PAT}"
      }
    },
    "context7": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp",
      "headers": {
        "CONTEXT7_API_KEY": "${CONTEXT7_API_KEY}"
      }
    },
    "sequential-thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    },
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"]
    },
    "sentry": {
      "type": "http",
      "url": "https://mcp.sentry.dev/mcp"
    }
  }
}
```

Les trois scopes de configuration sont : **local** (`~/.claude.json` sous le chemin projet), **project** (`.mcp.json` en racine, partagé via git), et **user** (`~/.claude.json` section globale). L'entreprise ajoute un scope **managed** (`managed-mcp.json` dans les répertoires système) qui ne peut pas être overridé.

### Serveurs MCP essentiels pour le développement

Les serveurs de **référence officielle** (maintenus par le steering group MCP) incluent : **Filesystem** (`@modelcontextprotocol/server-filesystem`) pour les opérations fichiers sécurisées, **Git** (`mcp-server-git` via uvx) pour la manipulation de repos, **Memory** (`@modelcontextprotocol/server-memory`) pour la persistance en knowledge graph, **Fetch** pour la récupération de contenu web, et **Sequential Thinking** pour le raisonnement structuré.

Le **GitHub MCP Server** est maintenant maintenu par GitHub directement (le package npm `@modelcontextprotocol/server-github` est déprécié depuis avril 2025). Le mode HTTP est recommandé pour Claude Code v2.1.1+.

### Serveurs de thinking et métacognition

**Sequential Thinking** (officiel) structure la résolution de problèmes en séquences de pensée révisables avec branchement. **Thinking Patterns** (`@emmahyde/thinking-patterns`, communautaire) offre **38+ opérations** de raisonnement structuré incluant `metacognitive_monitoring`, `visual_reasoning`, `temporal_thinking`, et `stochastic_algorithm`. **Vibe Check** (`@pv-bhat/vibe-check-mcp`, communautaire) agit comme un « rubber-duck debugger pour LLMs » avec trois outils : `vibe_check` (identifie les hypothèses incorrectes), `vibe_distill` (simplifie les plans complexes), et `vibe_learn` (log des erreurs pour auto-amélioration).

### MCP Tool Search — l'optimisation native révolutionnaire

Annoncé le **14 janvier 2026**, MCP Tool Search est la solution intégrée pour le problème de pollution du contexte par les définitions d'outils MCP. Avant cette fonctionnalité, 7 serveurs MCP consommaient ~67 000 tokens (33.7% de 200K) avant toute interaction utilisateur.

Le système s'active automatiquement quand les descriptions d'outils dépassent **10% de la fenêtre de contexte**. Il construit un index léger des noms et descriptions, puis charge les outils à la demande via recherche regex ou BM25. Résultat : **réduction de 85%** du contexte initial (de ~134K à ~5K tokens dans les tests Anthropic) et amélioration de la précision du modèle (Opus 4 passe de 49% à 74% sur les évaluations MCP).

Le registre officiel MCP est à **registry.modelcontextprotocol.io**, avec des sous-registres comme Smithery, PulseMCP, et mcp.so.

---

## 9. Memory et persistance inter-sessions

### Mécanismes natifs officiels

Claude Code dispose de **cinq systèmes de persistance** natifs. L'**Auto Memory** (`MEMORY.md` + fichiers thématiques dans `~/.claude/projects/<project>/memory/`) sauvegarde automatiquement le contexte utile. La **Session Memory** (résumés automatiques en arrière-plan, extraits après ~10 000 tokens de conversation) est injectée au démarrage suivant sous forme de « souvenirs passés ». Les **CLAUDE.md files** constituent la mémoire curatée par l'utilisateur. Les **Tasks** (v2.1.16+, janvier 2026) remplacent les anciens Todos avec persistance dans `~/.claude/tasks/`, tracking de dépendances, et coordination multi-sessions. Enfin, **`--continue`/`--resume`** permet de reprendre des sessions précédentes avec le transcript complet.

### Évolution TodoWrite → Tasks

Le système de Tasks, introduit le 22 janvier 2026, est une évolution majeure. Contrairement aux anciens Todos qui vivaient uniquement en mémoire et étaient perdus à la fermeture de session, les Tasks **persistent dans `~/.claude/tasks/`**, survivent aux fermetures de session, supportent le tracking de dépendances et les blockers, et permettent la **collaboration multi-sessions** : quand la Session A complète une tâche, la Session B voit la mise à jour immédiatement. La visibilité se toggle avec `Ctrl+T`.

### Solutions communautaires de mémoire

La communauté a développé plusieurs approches complémentaires. **claude-mem** (`thedotmack/claude-mem`) utilise **SQLite + FTS5 + ChromaDB** avec 5 hooks de lifecycle et un service worker avec web UI sur le port 37777. **memsearch** (par Zilliz/Milvus) crée des fichiers Markdown quotidiens avec un index vectoriel Milvus pour la recherche sémantique. **memory-mcp** utilise une architecture à deux niveaux : Tier 1 (CLAUDE.md auto-généré de ~150 lignes, compact) et Tier 2 (`.memory/state.json`, store complet searchable via MCP).

Le **Memory Bank pattern**, originaire de Cline, a été adapté pour Claude Code par plusieurs implémentations. Le concept central est une hiérarchie de documents Markdown : `projectbrief.md` → `productContext.md` → `systemPatterns.md` → `techContext.md` → `activeContext.md` → `progress.md`.

**Important à noter** : les patterns « vector index SQLite » et « résumés Haiku » sont des **patterns communautaires**, pas des fonctionnalités natives de Claude Code. Claude Code stocke nativement les sessions en SQLite avec transcripts JSONL mais n'utilise pas nativement l'indexation vectorielle ni la summarization Haiku pour la mémoire.

---

## 10. CLI, headless mode et Agent SDK

### Commandes CLI essentielles

```bash
# Sessions interactives
claude                              # Démarrer le REPL
claude "query"                      # Démarrer avec un prompt initial
claude -c                           # Continuer la dernière conversation
claude -r                           # Reprendre (choix interactif)

# Mode headless / CI
claude -p "prompt" --output-format json
claude -p "Review auth.py" --allowedTools Read,Grep,Glob --max-turns 3
git diff main | claude -p "Review this diff" --output-format json

# Structured output
claude -p "Extract functions" --output-format json \
  --json-schema '{"type":"object","properties":{"functions":{"type":"array","items":{"type":"string"}}}}'

# Agents custom via CLI
claude --agents '{
  "code-reviewer": {
    "description": "Expert code reviewer",
    "prompt": "You are a senior code reviewer. Focus on quality and security.",
    "tools": ["Read", "Grep", "Glob", "Bash"],
    "model": "sonnet"
  }
}'

# MCP management
claude mcp add --transport http github https://api.githubcopilot.com/mcp/
claude mcp add --transport stdio db -- npx -y @bytebase/dbhub --dsn "postgresql://..."
claude mcp list
claude mcp serve                    # Utiliser Claude Code COMME serveur MCP
```

### Agent SDK Python et TypeScript

L'**Agent SDK** (anciennement Claude Code SDK) fournit les mêmes outils et boucle d'agent que Claude Code, programmable en Python et TypeScript. Le SDK TypeScript (`@anthropic-ai/claude-agent-sdk`) totalise **2.5M+ téléchargements hebdomadaires** sur npm. Le SDK Python (`claude-agent-sdk`) est à **5 063 stars** sur GitHub.

```python
import asyncio
from claude_agent_sdk import query, ClaudeAgentOptions

async def main():
    async for message in query(
        prompt="Find and fix the bug in auth.py",
        options=ClaudeAgentOptions(
            allowed_tools=["Read", "Edit", "Bash"],
            permission_mode="acceptEdits",
            system_prompt="You are a senior Python developer. Follow PEP 8.",
        ),
    ):
        print(message)

asyncio.run(main())
```

Le SDK V2 apporte des changements importants : **pas de chargement de fichiers settings par défaut** (comportement prédictible), contrôle explicite via `settingSources: ["user", "project", "local"]`, subagents programmatiques inline, fork de session, support du beta `context-1m-2025-08-07` pour le contexte 1M tokens, et callback `canUseTool` pour le contrôle programmatique des permissions.

### Monitoring OpenTelemetry

```bash
export CLAUDE_CODE_ENABLE_TELEMETRY=1
export OTEL_METRICS_EXPORTER=otlp
export OTEL_LOGS_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

**8 métriques** disponibles : `session`, `lines_of_code`, `pull_request`, `commit`, `cost`, `token` (input/output/cache read/cache write), `code_edit_tool_decision`, `active_time`. **5 types d'événements** : `user_prompt`, `tool_result`, `api_request`, `api_error`, `tool_decision`. Le support multi-équipes via `OTEL_RESOURCE_ATTRIBUTES="team.name=backend,department=engineering"` est documenté.

---

## 11. Best practices 2026 — synthèse officielle et communautaire

### Cinq règles d'or (documentation officielle)

La documentation officielle converge vers cinq principes fondamentaux. Premièrement, **le contexte est la contrainte fondamentale** — chaque décision doit optimiser l'utilisation du contexte. Deuxièmement, **planifier avant d'implémenter** — utiliser Plan Mode (`Shift+Tab` × 2) pour les changements complexes et écrire les plans dans des fichiers externes (`plan.md`) qui survivent aux sessions. Troisièmement, **corriger tôt et souvent** — `Esc` pour stopper, `Esc+Esc` ou `/rewind` pour restaurer l'état précédent. Quatrièmement, **déléguer la recherche aux subagents** pour garder la conversation principale propre. Cinquièmement, **utiliser les hooks pour les actions déterministes** qui doivent se produire à chaque fois.

### Configuration monorepo recommandée

```
/monorepo/
├── CLAUDE.md                    # 50-150 lignes max, conventions partagées
├── .claude/
│   ├── settings.json            # Permissions, hooks (partagé via git)
│   ├── settings.local.json      # Overrides personnels (.gitignore)
│   ├── agents/
│   │   ├── code-reviewer.md     # Review agent (model: sonnet)
│   │   └── debugger.md          # Debug agent
│   ├── skills/
│   │   └── deploy/SKILL.md      # Skill de déploiement
│   └── rules/
│       ├── security.md          # Règles de sécurité
│       └── api-design.md        # Conventions API
├── .mcp.json                    # Serveurs MCP du projet
├── frontend/
│   └── CLAUDE.md                # Frontend-specific (lazy-loaded)
├── backend/
│   └── CLAUDE.md                # Backend-specific (lazy-loaded)
└── .github/
    └── workflows/
        └── claude.yml           # CI/CD avec Claude Code Action
```

### CI/CD avec GitHub Actions

```yaml
name: Claude Code Review
on:
  issue_comment:
    types: [created]
  pull_request:
    types: [opened, synchronize]

jobs:
  claude:
    if: contains(github.event.comment.body, '@claude')
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      issues: write
    steps:
      - uses: actions/checkout@v4
      - uses: anthropic/claude-code-action@v1
        with:
          anthropic-api-key: ${{ secrets.ANTHROPIC_API_KEY }}
```

Le **claude-code-action** (5 909 stars) est l'intégration officielle GitHub Actions, construit sur l'Agent SDK. L'installation se fait via `/install-github-app` dans le terminal Claude Code.

### Patterns de subagents — fork vs summary

Les subagents se configurent en markdown avec frontmatter YAML dans `.claude/agents/`. Le flag **`context: fork`** dans les skills exécute le skill dans un contexte isolé — les résultats reviennent dans la conversation principale mais les messages intermédiaires restent isolés. Le flag **`isolation: worktree`** va plus loin en isolant le subagent dans un git worktree dédié. Le flag **`background: true`** permet l'exécution en arrière-plan (`Ctrl+B` pour backgrounder, `/tasks` pour monitorer).

La philosophie communautaire se divise en deux camps : le camp « agents spécialisés » (code-reviewer, build-error-resolver, etc.) et le camp « Master-Clone » qui met tout le contexte dans CLAUDE.md et laisse l'agent principal spawner des clones de lui-même via `Task(...)`.

### Anti-patterns à éviter

Les erreurs les plus coûteuses selon la communauté et la documentation officielle sont : dépasser 20K tokens d'outils MCP dans le contexte (le MCP Tool Search résout cela), bloquer les writes au lieu des commits (désoriente l'agent en plein plan), auto-générer CLAUDE.md (perd le levier du curation humain), et utiliser `/compact` au lieu de `/clear` + relecture ciblée des fichiers modifiés. La recherche d'Arize sur SWE-Bench Lite montre qu'un CLAUDE.md optimisé peut améliorer la précision de génération de code de **+5.19%** (cross-repo) à **+10.87%** (within-repo) sans aucun changement de modèle.

---

## Conclusion

L'écosystème Claude Code en février 2026 a atteint une maturité remarquable avec trois innovations structurantes. Le **système de plugins** (9 000+ extensions, marketplace officiel pré-configuré) transforme un outil monolithique en plateforme extensible. Le **standard Agent Skills** (agentskills.io, adopté par 26+ plateformes) établit la portabilité cross-outils avec un mécanisme de lazy loading qui réduit la consommation de tokens de 95%. Et le **MCP Tool Search** (janvier 2026) résout enfin le problème de pollution du contexte par les outils MCP avec une réduction de 85% du contexte initial.

La configuration optimale pour un projet enterprise combine un CLAUDE.md concis (150 lignes max avec imports `@`), des hooks déterministes pour les règles inviolables (sécurité, formatting, validation pré-commit), des skills pour le knowledge domain-spécifique chargé à la demande, et 3-5 serveurs MCP ciblés (GitHub, Context7, Sequential Thinking) avec Tool Search activé par défaut. Le **Superpowers plugin** de Jesse Vincent reste le framework communautaire le plus influent pour structurer les workflows de développement, tandis que l'**Agent SDK** ouvre la voie à l'automatisation programmatique complète. L'ensemble constitue un système où la gestion du contexte n'est plus un obstacle mais un avantage architectural, à condition de respecter le principe de progressive disclosure à chaque niveau de la stack.