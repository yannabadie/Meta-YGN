---
title: Auto-Checkpoint
type: feature
evidence_tier: confirmed
updated: 2026-04-07
crate: metaygn-verifiers
tags:
  - feature
  - confirmed
  - safety
created: 2026-04-07
---

# Auto-Checkpoint

**Tier** : `[confirmed]`
**Crate** : `metaygn-verifiers`
**Fichier** : `crates/verifiers/src/checkpoint.rs`

## Description

Cree automatiquement un checkpoint avant les operations destructives :
- `git stash create` / `git rev-parse HEAD` avant les ops git destructives (reset, checkout --, clean)
- Copie des fichiers cibles avant `rm` / `unlink` / `find -delete`

Le message de recovery est inclus dans la reponse du hook :
`[checkpoint] To recover: git stash apply <sha>` ou `copy from <path>`.

## Implementation

- **git_checkpoint()** : `git stash create` (working-tree intacte), fallback `git rev-parse HEAD`
- **file_checkpoint()** : copie dans `<cwd>/.claude/aletheia/checkpoints/<timestamp>/`
- **extract_target_files()** : parse rm, unlink, find -delete pour identifier les cibles
- **Limites** : 100 fichiers max, 50 MB total
- **Logging** : chaque checkpoint est logged dans `checkpoints.log`

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | checkpoint_test.rs | pass |
| Tests in-module | shell_split, base_command | pass |
| Production | v2.5+ | actif |
