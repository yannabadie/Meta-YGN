---
title: Bearer Auth
type: feature
evidence_tier: confirmed
updated: 2026-04-07
crate: metaygn-daemon
tags:
  - feature
  - confirmed
  - security
created: 2026-04-07
---

# Bearer Auth

**Tier** : `[confirmed]`
**Crate** : `metaygn-daemon`
**Fichier** : `crates/daemon/src/auth.rs`

## Description

Authentification bearer-token sur tous les endpoints du daemon HTTP.

Au demarrage, le daemon genere un UUID v4 aleatoire, l'ecrit dans
`~/.claude/aletheia/daemon.token`, et installe un middleware Axum qui
verifie `Authorization: Bearer <token>` sur toutes les routes sauf `/health`.

## Implementation

- **AuthToken** : newtype `AuthToken(String)` passee comme state Axum
- **auth_middleware()** : middleware Axum via `from_fn_with_state`
- **Bypass** : `/health` toujours public (monitoring sans token)
- **Mode strict** : `METAYGN_STRICT_AUTH=1` rejette avec 401 (sinon warn-only pour compatibilite v2.5)
- **Cleanup** : `daemon.token` supprime au shutdown (Ctrl+C ou `/admin/shutdown`)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | auth_token_is_cloneable | pass |
| Integration | E2E daemon test (health sans auth, hooks avec auth) | pass |
| Code review | middleware wire dans lib.rs build_app_with_state | verifie |
