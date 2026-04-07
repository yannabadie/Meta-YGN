---
title: Guard Pipeline — 5 Guards, 28 Rules
type: architecture
tags:
  - architecture
  - security
  - guards
updated: 2026-04-07
---

# Guard Pipeline

**Fichier** : `crates/verifiers/src/guard_pipeline.rs`
**Status** : `[confirmed]`

## Logique

Tous les guards executent en parallele. Score agrege = MIN(tous les scores).
Si un guard bloque, `allowed=false`, `blocking_guard` = premier bloqueur.

## Guard 1 — DestructiveGuard (score: 0 = DENY)

Bloque inconditionnellement :

| Pattern | Commande |
|---------|----------|
| `rm -rf /` | Suppression racine |
| `sudo rm -rf` | Suppression root |
| `mkfs` | Formatage disque |
| `dd if=` | Ecriture directe disque |
| `shutdown` | Arret systeme |
| `reboot` | Redemarrage |
| `:(){ ... }` | Fork bomb |
| `chmod 777 /` | Permissions ouvertes racine |

**8 patterns regex**

## Guard 2 — HighRiskGuard (score: 30 = ASK)

Demande confirmation :

| Pattern | Risque |
|---------|--------|
| `git push` | Push vers remote |
| `git reset --hard` | Perte de travail |
| `terraform apply\|destroy` | Infrastructure |
| `kubectl apply\|delete` | Kubernetes |
| `curl \| bash` | Execution distante |
| `sudo` | Elevation privileges |
| `docker push\|prune` | Registry / cleanup |
| `security code` | Code de securite |
| `password reset` | Reset mot de passe |
| `exfiltrat` | Exfiltration donnees |
| `forward to` | Redirection donnees |
| `send the code` | Envoi de code sensible |

**15 patterns regex**

## Guard 3 — SecretPathGuard (score: 20 = ASK)

Demande confirmation si acces a :

| Pattern | Type |
|---------|------|
| `.env` | Variables d'environnement |
| `secrets/` | Dossier secrets |
| `*.pem` | Certificats |
| `*.key` | Cles privees |
| `id_rsa` | SSH |
| `credentials.json` | Credentials cloud |
| `.npmrc` | Token npm |
| `.pypirc` | Token PyPI |
| `kubeconfig` | Config Kubernetes |

**9 patterns regex**

## Guard 4 — McpGuard (score: 40 = ASK)

Bloque tout outil commencant par `mcp__`.
Les outils MCP sont de la data externe non trustee.

**1 pattern**

## Guard 5 — DefaultGuard (score: 100 = ALLOW)

Toujours allow. Baseline quand aucun guard ne matche.

## Prompt Injection Detection (v2.0)

**Fichier** : `crates/core/src/stages/assess.rs`
**Fonction** : `contains_prompt_injection_markers()`

Detecte les patterns d'injection de prompt courants au niveau du stage `assess` :

| Pattern | Type |
|---------|------|
| "ignore your previous instructions" | Jailbreak classique |
| "###(system_message)" | Injection system prompt |
| TODO-based avec termes risques | Attaque par insertion TODO |

Retourne `RiskLevel::High` si un pattern est detecte.
L'injection est evaluee avant la classification par keywords,
ce qui garantit que les commandes injectees sont traitees comme HIGH risk.

## Total : 35+ regles (8 + 15 + 9 + 1 + 0 + injection patterns)
