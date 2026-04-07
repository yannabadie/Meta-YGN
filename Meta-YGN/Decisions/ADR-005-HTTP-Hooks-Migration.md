---
title: "ADR-005: HTTP Hooks Migration"
type: adr
status: proposed
date: 2026-04-07
tags:
  - adr
  - performance
  - hooks
---

# ADR-005: Migration des Hooks `npx tsx` vers des Alternatives Plus Rapides

## Contexte

### Le probleme

Les hooks Claude Code actuels utilisent `type: command` avec `npx tsx` pour executer du TypeScript a chaque evenement hook (SessionStart, PreToolUse, PostToolUse, etc.).

Chaque invocation de hook entraine :
- **500-800ms de startup Node.js** : `npx tsx` doit demarrer le runtime Node.js, resoudre le module TSX, compiler le TypeScript, puis executer le hook
- Sur une session typique de 50 tool calls avec PreToolUse + PostToolUse : **50-80 secondes de pure overhead**
- Ce cout est incompressible avec l'architecture `type: command` actuelle

### Pourquoi c'est critique

Le daemon Aletheia repond en 2-5ms. Les hooks sont censes etre un transport leger vers le daemon. Au lieu de cela, le transport est 100-400x plus lent que le traitement.

```
Latence reelle par hook:
  npx tsx startup:    500-800ms  (95% du temps total)
  HTTP call daemon:     2-5ms    (le travail reel)
  Total:              502-805ms
  Overhead:           99%+
```

## Decision

**Statut : proposed** — en attente d'evaluation des alternatives.

Migrer les hooks `type: command` (`npx tsx`) vers une alternative a faible latence de demarrage. L'objectif est de reduire le temps de startup de hook de ~600ms a <10ms.

## Alternatives considerees

| Option | Startup | Pour | Contre |
|--------|---------|------|--------|
| **`type: http`** (Claude Code natif) | ~0ms (pas de process) | Zero overhead, le daemon ecoute deja en HTTP, supportu nativement par Claude Code | Le port est dynamique (assigne par l'OS), necessite un mecanisme de decouverte du port (fichier lock, env var). Pas clair si Claude Code supporte le port dynamique |
| **Sous-commande Rust compilee** (`aletheia hook <event>`) | ~1-2ms | Ultra rapide, deja dans le binary `aletheia`, pas de runtime externe | Necessite que le binary soit dans le PATH, installation plus complexe pour l'utilisateur |
| **JS pre-compile** (esbuild bundle) | ~50-100ms | Plus rapide que TSX, reste dans l'ecosysteme Node.js | Toujours Node.js startup (~50ms), necessite un build step supplementaire |
| **Shell script** (curl vers daemon) | ~5-10ms | Simple, universel, pas de runtime | Pas de logique de fallback, gestion d'erreurs limitee, pas cross-platform (bash vs PowerShell) |
| **Statu quo** (`npx tsx`) | ~600ms | Fonctionne, DX TypeScript familiere | 500-800ms par hook, inacceptable pour hot path |

## Analyse detaillee

### Option 1 : `type: http` (recommandee si faisable)

Claude Code supporte nativement `type: http` pour les hooks. Le daemon Aletheia ecoute deja en HTTP sur un port local. La question est : comment communiquer le port dynamique a la configuration du hook ?

Approches possibles :
1. **Port fixe configurable** : l'utilisateur configure un port dans `settings.json`, le daemon le respecte
2. **Fichier lock** : le daemon ecrit son port dans `~/.aletheia/daemon.lock`, le hook lit ce fichier
3. **Variable d'environnement** : le daemon exporte `ALETHEIA_PORT`, accessible dans la config du hook

### Option 2 : Sous-commande Rust compilee

Le binary `aletheia` existe deja (crate `metaygn-cli`). Ajouter une sous-commande `aletheia hook <event> --input <json>` qui :
1. Lit le port daemon depuis le fichier lock
2. Fait le HTTP call
3. Ecrit le resultat sur stdout
4. Fallback heuristique si daemon down

Startup ~1-2ms vs ~600ms pour `npx tsx`.

### Option 3 : JS pre-compile

Utiliser `esbuild` pour bundler le TypeScript en un seul fichier JS. Execution avec `node` au lieu de `npx tsx`.

```json
{
  "type": "command",
  "command": "node /path/to/compiled-hook.js"
}
```

Reduit le startup de ~600ms a ~50-100ms. Amelioration 6-12x mais toujours pas optimal.

### Option 4 : Shell script

```json
{
  "type": "command",
  "command": "curl -s -X POST http://localhost:$(cat ~/.aletheia/port)/hook/$HOOK_EVENT -d @-"
}
```

Simple mais fragile : pas de fallback, pas de gestion d'erreurs, pas cross-platform.

## Consequences

### Si `type: http` est viable
- **Positives** : latence de hook reduite a ~0ms (plus de process spawn), configuration simplifiee, suppression de la dependance Node.js pour le runtime
- **Negatives** : necessite un mecanisme de decouverte de port, le daemon doit etre demarre avant Claude Code

### Si sous-commande Rust choisie
- **Positives** : latence ~1-2ms, fallback heuristique integre, cross-platform
- **Negatives** : le binary `aletheia` doit etre installe et dans le PATH, complexite d'installation

### Communes
- Le code TypeScript de hooks (`packages/hooks/`) reste utilisable pour le fallback et les tests
- Les seuils heuristiques restent les memes, seul le transport change
- La logique de fallback (daemon down -> heuristiques locales) doit etre preservee

## Metriques de succes

| Metrique | Avant | Objectif |
|----------|-------|----------|
| Startup par hook | 500-800ms | <10ms |
| Overhead session (50 calls) | 50-80s | <1s |
| Ratio overhead/traitement | 99%+ | <50% |

## Evidence tag

`[confirmed]` — Le probleme de latence est mesure. Les alternatives sont basees sur de la documentation confirmee (Claude Code hooks spec, Rust startup benchmarks). Le choix final depend de la faisabilite de `type: http` dans Claude Code.
