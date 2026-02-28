# Configuration idéale de Claude Code 4.6 (1M) pour ce projet

## Objectif

Configurer Claude Code pour construire un **runtime métacognitif local-first**, distribué ensuite comme **plugin Claude Code** puis comme **marketplace plugin**, sans tomber dans :

- l'obésité de contexte
- la dépendance totale au MCP
- la confusion entre mémoire, skills, hooks et agents
- l'absence de bench / sécurité / ROI

Ce fichier décrit la configuration idéale de travail **maintenant**, pas la configuration la plus spectaculaire.

---

## 1. Politique générale

## Ce qu'il faut optimiser

1. **sobriété contexte / tokens**
2. **vitesse d'itération locale**
3. **sécurité**
4. **qualité de vérification**
5. **portabilité future**

## Ce qu'il ne faut pas optimiser d'abord

1. multi-agent exotique
2. gros MCP permanent
3. 1M de contexte utilisé comme solution architecturale
4. packaging marketplace avant bench + hooks + runtime

---

## 2. Surface de travail recommandée

## IDE principal recommandé

### Option par défaut
**VS Code + extension Claude Code**

C'est la recommandation la plus simple et la plus robuste pour ce projet parce que tu as :

- revue des plans
- diffs inline
- `@` mentions avec plages de lignes
- historique de conversations
- accès au CLI depuis le terminal intégré

### Option secondaire
**JetBrains / RustRover / IntelliJ + Claude Code**

Très pertinent si tu fais du Rust intensif pour le coeur runtime.

### Option complémentaire
**Chrome integration** uniquement si tu testes :

- documentation web
- interfaces marketplace
- parcours UI
- flows navigateur

### Recommandation pratique

- **VS Code** pour le workflow principal du plugin
- **JetBrains** si le noyau Rust devient le centre de gravité du repo
- **Chrome** seulement pour les cas E2E

---

## 3. Politique de modèles

## Réglage recommandé

### Modèle par défaut
`claude-sonnet-4-6`

Utilise Sonnet 4.6 pour :

- le développement quotidien
- l'itération sur le plugin
- la génération de hooks / skills
- le refactoring courant
- la création de tests

### Escalade vers Opus 4.6
Passe à `opus` uniquement pour :

- architecture système
- design du runtime métacognitif
- prompts maîtres
- protocoles d'évaluation
- sécurité / threat modeling
- synthèses longues à haute valeur

### Usage implicite de Haiku
Laisse Claude utiliser **Haiku** pour les sous-agents d'exploration rapides quand c'est possible.

### Règle simple

- **Sonnet = build**
- **Opus = think / design / decide**
- **Haiku = explore / classify / background**

---

## 4. Politique 1M contexte

## À faire

Utilise le 1M seulement pour :

- lire de gros corpus de notes / recherches
- analyser de longues traces d'évaluation
- revoir un grand repo ou plusieurs variantes
- synthétiser benchmark + docs + conversations

## À ne pas faire

Ne base pas le produit sur l'hypothèse :

> "on a 1M, donc on peut tout charger"

Le produit doit rester performant et intelligible même à 200K.

## Recommandation stratégique

- **sessions R&D / synthèse** : 1M si disponible
- **plugin dev quotidien** : pense comme si tu avais 200K
- **CI / replay / eval** : teste aussi un mode sobre pour détecter les dépendances cachées au contexte long

---

## 5. Extensions / intégrations les plus adaptées

## Extension IDE

### Priorité 1
- Claude Code extension pour **VS Code / Cursor**

### Priorité 2
- Claude Code integration pour **JetBrains** si Rust-first

### Priorité 3
- **Chrome integration** pour tests navigateur et flows docs

## Outils CLI locaux indispensables

Installe en local et laisse Claude les exploiter directement :

- `git`
- `gh`
- `jq`
- `rg`
- `fd`
- `just` ou `make`
- `cargo`
- `rustup`
- `wasmtime`
- `z3` (si tu veux un vérificateur symbolique)
- `pnpm` ou `npm`
- `uv` / `python`
- `sqlite3`

### Pourquoi
Pour ce projet, les outils CLI sont souvent plus efficaces qu'un gros bouquet de serveurs MCP toujours chargés.

---

## 6. Plugins à installer dans Claude Code

## Plugins officiels à installer dès le début

### Développement plugin / agent
- `plugin-dev@claude-plugins-official`
- `agent-sdk-dev@claude-plugins-official`
- `commit-commands@claude-plugins-official`

### Code intelligence - selon la stack
Comme le projet recommandé combine un coeur Rust, des wrappers TypeScript et potentiellement des outils Python, installe :

- `rust-analyzer-lsp@claude-plugins-official`
- `typescript-lsp@claude-plugins-official`
- `pyright-lsp@claude-plugins-official`

## Plugins optionnels

### À n'activer que si nécessaire
- `github@claude-plugins-official`
- `gitlab@claude-plugins-official`

### Règle importante
Pour les workflows courants repo / PR / commit, préfère souvent :

- `gh`
- `git`
- hooks
- skills

avant d'ajouter un MCP externe supplémentaire.

---

## 7. Configuration `~/.claude/settings.json` recommandée

```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "model": "claude-sonnet-4-6",
  "availableModels": ["sonnet", "opus", "haiku"],
  "outputStyle": "Default",
  "statusLine": {
    "type": "command",
    "command": "~/.claude/statusline.sh",
    "padding": 1
  },
  "permissions": {
    "allow": [
      "Bash(git status)",
      "Bash(git diff *)",
      "Bash(git add *)",
      "Bash(rg *)",
      "Bash(fd *)",
      "Bash(jq *)",
      "Bash(cargo fmt)",
      "Bash(cargo check)",
      "Bash(cargo test *)",
      "Bash(pnpm lint)",
      "Bash(pnpm test *)",
      "Bash(pyright *)"
    ],
    "deny": [
      "Read(./.env)",
      "Read(./.env.*)",
      "Read(./secrets/**)",
      "Read(./config/credentials.json)",
      "Edit(./.env)",
      "Edit(./.env.*)"
    ]
  },
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",
    "OTEL_METRICS_EXPORTER": "otlp",
    "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE": "70",
    "ENABLE_TOOL_SEARCH": "auto:5"
  },
  "enabledPlugins": {
    "plugin-dev@claude-plugins-official": true,
    "agent-sdk-dev@claude-plugins-official": true,
    "commit-commands@claude-plugins-official": true,
    "rust-analyzer-lsp@claude-plugins-official": true,
    "typescript-lsp@claude-plugins-official": true,
    "pyright-lsp@claude-plugins-official": true
  }
}
```

## Pourquoi cette base est bonne

- `model`: Sonnet par défaut pour le build quotidien
- `availableModels`: garde Opus disponible mais contrôlé
- `statusLine`: contexte visible en continu
- `permissions`: autorise les commandes sûres et bloque les zones sensibles
- `env`: active l'observabilité + compaction plus précoce + tool search agressif
- `enabledPlugins`: n'installe que le strict utile au projet

---

## 8. `statusline.sh` recommandée

Le but de la status line est de montrer à tout moment :

- modèle courant
- branche git
- pourcentage de contexte
- coût si disponible
- mode de permission

Exemple simple :

```bash
#!/usr/bin/env bash
jq -r '
  [
    "[" + (.model.display_name // "?") + "]",
    (.cwd | split("/") | last),
    (.permission_mode // "default"),
    ((.context_window.used_percentage // 0) | tostring) + "%ctx"
  ] | join(" ")
'
```

Rends-le exécutable :

```bash
chmod +x ~/.claude/statusline.sh
```

---

## 9. Configuration projet `.claude/settings.json`

Dans le repo du projet, versionne une configuration plus spécifique.

```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "enabledPlugins": {
    "your-metacog-plugin@your-org-marketplace": true
  },
  "extraKnownMarketplaces": {
    "your-org-marketplace": {
      "source": {
        "source": "github",
        "repo": "your-org/claude-metacog-marketplace"
      }
    }
  },
  "permissions": {
    "deny": [
      "Read(./prod-secrets/**)",
      "Edit(./infra/prod/**)",
      "Bash(kubectl apply *)",
      "Bash(terraform apply *)"
    ]
  }
}
```

## Intention

- partager le plugin avec l'équipe
- verrouiller certains chemins et actions sensibles
- préparer la transition vers marketplace privée puis publique

---

## 10. `settings.local.json`

À garder pour :

- préférences personnelles
- sandbox URLs
- expérimentations locales
- override de plugins
- configuration temporaire de debug

Exemple :

```json
{
  "outputStyle": "metacognitive-explanatory",
  "permissions": {
    "allow": [
      "Bash(cargo test metacog_*)"
    ]
  }
}
```

---

## 11. Politique de permissions et sandbox

## Mode recommandé par phase

### Exploration / cadrage
- **Plan mode**

### Implémentation quotidienne
- **Ask permissions** ou **Auto accept edits** selon le niveau de confiance

### Automatisation forte
- **sandbox** avant tout
- pas de `--dangerously-skip-permissions` hors environnement contenu

## Règle produit
Les opérations irréversibles doivent rester sous contrôle explicite :

- suppression massive
- déploiement
- rotation de secrets
- écriture infra critique
- modifications de configuration système

---

## 12. Hooks recommandés pour ce projet

## Principe

- ce qui est **déterministe et toujours nécessaire** = hook
- ce qui est **contextuel et réutilisable** = skill
- ce qui est **long / spécialisé / isolé** = agent
- ce qui est **externe** = MCP

## `hooks/hooks.json` recommandé

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "metacog session-start"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "agent",
            "agent": "prompt-classifier"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash|Write|Edit|WebFetch",
        "hooks": [
          {
            "type": "agent",
            "agent": "tool-governor"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash|Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "metacog observe --event post-tool"
          }
        ]
      }
    ],
    "PostToolUseFailure": [
      {
        "matcher": "Bash|Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "metacog observe --event post-tool-failure"
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "metacog compact"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "prompt",
            "prompt": "Check if the task has sufficient evidence, tests, and explicit uncertainties documented. Return {\"ok\":true} or {\"ok\":false,\"reason\":\"...\"}."
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "metacog session-end"
          }
        ]
      }
    ]
  }
}
```

## Intention des hooks

- `SessionStart`: contexte minimal et fingerprint repo
- `UserPromptSubmit`: typologie de tâche + risque
- `PreToolUse`: gouvernance + coût + sécurité
- `PostToolUse`: ingestion du feedback externe
- `PreCompact`: compaction intelligente
- `Stop`: interdiction de conclure trop tôt
- `SessionEnd`: mémoire et métriques

---

## 13. Skills plugin à créer

## Skills noyau

### 1. `metacog:plan`
- usage : classifier la tâche, choisir la stratégie, allouer un budget
- type : `context: fork`
- agent : `Plan`

### 2. `metacog:challenge`
- usage : produire la contre-hypothèse la plus destructrice
- type : `context: fork`
- agent : `Explore` ou agent custom `skeptic`

### 3. `metacog:proof`
- usage : vérifier qu'une proposition est soutenue par des tests / preuves / traces
- type : `disable-model-invocation: true`
- déclenchement manuel avant une action critique

### 4. `metacog:compact`
- usage : générer un résumé de compaction orienté preuves, risques, hypothèses ouvertes
- type : manuel ou auto via hook

### 5. `metacog:bench`
- usage : exécuter un scénario de bench et enregistrer les métriques
- type : manuel

### 6. `metacog:threat-model`
- usage : audit prompt injection / exfiltration / secrets / permissions
- type : manuel

### 7. `metacog:tool-audit`
- usage : décider quel outil / quel protocole utiliser et lesquels éviter
- type : manuel ou automatique

## Frontmatter recommandé

### Skill manuelle à effet de bord

```md
---
name: metacog-proof
description: Vérifie qu'une action ou une réponse est suffisamment prouvée avant validation
disable-model-invocation: true
allowed-tools: Read, Grep, Glob, Bash(just *), Bash(cargo test *), Bash(pnpm test *)
context: fork
agent: general-purpose
---
```

### Skill de connaissance invoquable par Claude

```md
---
name: metacog-patterns
description: Donne à Claude les conventions métacognitives du projet
user-invocable: false
---
```

---

## 14. Subagents recommandés

## Agents custom

### `skeptic.md`
- mission : chercher les hypothèses fragiles
- outils : lecture + recherche + bash limité
- modèle : `haiku` ou `sonnet` selon profondeur

### `verifier.md`
- mission : exécuter tests, checklists, preuves
- outils : lecture + bash + write très limité
- mode : `acceptEdits` seulement si contrôlé

### `repo-cartographer.md`
- mission : cartographier le repo, limites, conventions, surfaces à risque
- modèle : `haiku`
- outils : read-only

### `cost-auditor.md`
- mission : pointer le contexte inutile, skills mal placées, MCP superflus, sorties trop longues

### `human-calibrator.md`
- mission : reformuler pour l'humain, détecter ambiguïtés / surcharge / besoin de friction

---

## 15. Output style recommandé

## Par défaut
Garde **Default** pour les sessions de coding normales.

## Style custom utile
Crée un output style `metacognitive-explanatory.md` avec :

- `keep-coding-instructions: true`
- exigence d'expliciter : hypothèses, preuves, incertitudes, coût

Exemple :

```md
---
name: Metacognitive Explanatory
description: Met l'accent sur hypothèses, preuves, risques et incertitudes
keep-coding-instructions: true
---

Quand tu proposes une action importante :
1. résume l'objectif
2. liste les hypothèses non prouvées
3. indique quelle preuve existe déjà
4. indique quelle vérification manque
5. si le risque est élevé, recommande plan / proof / bench avant action
```

## Quand l'utiliser
- architecture
- sécurité
- benchmark
- revue d'options
- prompt design

Pas forcément pour un correctif trivial.

---

## 16. CLAUDE.md idéal pour ce projet

## Règle d'or
Le `CLAUDE.md` doit rester **court, transversal, stable**.

Ce qui est spécifique, long, occasionnel, ou procédural va dans :

- `.claude/rules/*.md`
- `skills/`
- `docs/`

## Structure recommandée

```md
# Project mission
Construire un runtime métacognitif local-first, AI-agnostic, avec wrapper Claude Code natif.

# Non-negotiable principles
- Le noyau métacognitif ne dépend pas de Claude Code.
- MCP est une façade d'intégration, pas le cerveau.
- Toute action importante doit être soutenue par une preuve, un test, ou un signal externe.
- La sobriété contexte/tokens est une contrainte produit.
- Les hooks portent les règles déterministes; les skills portent les protocoles réutilisables.
- Toute nouvelle capacité doit avoir une stratégie d'évaluation.

# Repo map
- core/: runtime, daemon, cli, verifiers, memory
- adapters/claude-plugin/: plugin Claude Code
- adapters/mcp-server/: façade MCP
- adapters/a2a-agent/: coordination multi-agent
- eval/: bench, replay, dashboards
- docs/: architecture, sécurité, roadmap

# Commands
- build rust: cargo build --workspace
- test rust: cargo test --workspace
- lint rust: cargo fmt --all && cargo clippy --workspace --all-targets
- ts checks: pnpm lint && pnpm test
- python checks: uv run pytest
- eval smoke: just eval-smoke

# Development protocol
1. Explore avant de modifier
2. Planifier avant refactor multi-fichiers
3. Vérifier avant de conclure
4. Bench avant de déclarer une amélioration
5. Mettre à jour docs + changelog

# Evidence protocol
Avant toute affirmation forte, chercher au moins un signal externe parmi:
- tests
- diagnostics LSP
- linter
- benchmark
- trace runtime
- doc officielle

# Security boundaries
- Ne jamais lire .env, secrets, credentials
- Demander confirmation avant action irréversible
- Préférer sandbox / plan mode / hooks à la confiance implicite

# Compact instructions
Quand tu compactes, garde surtout:
- décisions prises
- hypothèses ouvertes
- preuves disponibles
- échecs utiles
- coût / risque / prochaines étapes

# Imports
- @docs/architecture.md
- @docs/eval-framework.md
- @docs/threat-model.md
```

---

## 17. `.claude/rules/` recommandé

Au lieu d'un `CLAUDE.md` géant, crée :

```text
.claude/rules/
├─ 00-principles.md
├─ 10-runtime-core.md
├─ 20-claude-plugin.md
├─ 30-mcp-a2a.md
├─ 40-eval.md
├─ 50-security.md
└─ 60-release.md
```

## Rôle des rules

- `00-principles.md` : règles produit absolues
- `10-runtime-core.md` : architecture Rust / daemon / memory
- `20-claude-plugin.md` : skills, hooks, agents, packaging
- `30-mcp-a2a.md` : interfaces externes
- `40-eval.md` : bench, metrics, replay
- `50-security.md` : secrets, sandbox, permissions, threat model
- `60-release.md` : semver, changelog, marketplace

---

## 18. `CLAUDE.local.md`

À utiliser pour :

- préférences personnelles
- modèles préférés
- endpoints locaux de debug
- scripts temporaires
- règles de confort non partageables

Exemple :

```md
# Personal preferences
- Préférer Sonnet pour l'implémentation, Opus pour l'architecture.
- Me rappeler d'exécuter /metacog:proof avant de valider un design important.
- Mon endpoint OTEL local : http://localhost:4318
- Mon bac à sable local : http://localhost:3000
```

---

## 19. Workflow recommandé dans Claude Code

## Boucle standard

1. `/model sonnet`
2. **Plan mode** pour cadrer
3. `/metacog:plan <tâche>`
4. exploration via agents / LSP / CLI
5. implémentation
6. `PostToolUse` absorbe tests / diagnostics / erreurs
7. `/metacog:challenge <décision>` si architecture risquée
8. `/metacog:proof <résultat>` avant validation
9. `/compact` si la session s'étire
10. bench / replay / changelog

## Quand passer sur Opus

- design du noyau
- architecture plugin
- sécurité / threat model
- document de synthèse final

## Quand forcer la sobriété

- bugs simples
- refactors mécaniques
- tickets unitaires
- housekeeping

---

## 20. Politique MCP pour ce projet

## Règle
**MCP minimaliste.**

### MCP oui pour
- GitHub / GitLab / Jira / Notion / Slack si besoin réel
- connecteurs entreprise
- publication externe des capacités

### MCP non pour
- logique interne du runtime
- audit fréquent d'un raisonnement
- gouvernance haute fréquence
- verifications triviales déjà disponibles en CLI

## Pattern conseillé

- local CLI d'abord
- skill ensuite
- hook si déterministe
- MCP seulement si la capacité doit traverser les frontières de l'outil

---

## 21. Roadmap de configuration

## Semaine 1
- VS Code extension
- Sonnet par défaut
- LSP Rust / TS / Python
- `CLAUDE.md` minimal
- rules modulaires
- statusline

## Semaine 2
- hooks MVP
- plugin local avec `--plugin-dir`
- 3 à 4 skills
- 2 agents

## Semaine 3
- runtime local branché aux hooks
- bench smoke
- OTEL minimal

## Semaine 4
- private marketplace
- README de plugin
- versioning + changelog

---

## 22. Check-list finale de "bonne" config

### Oui
- Sonnet par défaut
- Opus ciblé
- Haiku pour exploration
- CLAUDE.md court
- rules modulaires
- skills ciblées
- hooks déterministes
- LSP installés
- CLI locaux privilégiés
- MCP limité et justifié
- status line + OTEL
- sandbox pour autonomie forte

### Non
- tout mettre dans CLAUDE.md
- installer trop de MCP externes
- utiliser 1M comme béquille d'architecture
- déployer avant benchmark
- faire confiance à l'auto-réflexion sans signal externe

---

## Sources consultées pour cette configuration

### Claude Code / Anthropic

- [Model config](https://code.claude.com/docs/en/model-config)
- [Context windows](https://platform.claude.com/docs/en/build-with-claude/context-windows)
- [VS Code integration](https://code.claude.com/docs/en/vs-code)
- [JetBrains integration](https://code.claude.com/docs/en/jetbrains)
- [CLI reference](https://code.claude.com/docs/en/cli-reference)
- [Settings](https://code.claude.com/docs/en/settings)
- [Memory](https://code.claude.com/docs/en/memory)
- [Skills](https://code.claude.com/docs/en/skills)
- [Subagents](https://code.claude.com/docs/en/sub-agents)
- [Hooks](https://code.claude.com/docs/en/hooks)
- [Plugins](https://code.claude.com/docs/en/plugins)
- [Plugins reference](https://code.claude.com/docs/en/plugins-reference)
- [Discover plugins](https://code.claude.com/docs/en/discover-plugins)
- [Plugin marketplaces](https://code.claude.com/docs/en/plugin-marketplaces)
- [Costs](https://code.claude.com/docs/en/costs)
- [Output styles](https://code.claude.com/docs/en/output-styles)
- [MCP in Claude Code](https://code.claude.com/docs/en/mcp)
- [Security](https://code.claude.com/docs/en/security)
- [Sandboxing](https://code.claude.com/docs/en/sandboxing)

### Standards / écosystème

- [MCP specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)
- [A2A protocol](https://a2a-protocol.org/latest/)
- [Agent Skills](https://agentskills.io/home)
- [mcporter](https://github.com/steipete/mcporter)
