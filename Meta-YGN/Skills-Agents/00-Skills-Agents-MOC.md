---
title: Skills & Agents MOC
type: moc
tags:
  - skills
  - agents
  - moc
updated: 2026-04-07
---

# Skills & Agents

## 8 Skills Metacognitives

| Skill | Commande | Quand l'utiliser |
|-------|----------|-----------------|
| **metacog-preflight** | `/metacog-preflight` | Avant du travail non-trivial : classifier risque, choisir strategie |
| **metacog-proof** | `/metacog-proof` | Avant de finaliser : construire un evidence packet |
| **metacog-challenge** | `/metacog-challenge` | Quand la confiance est haute mais l'evidence mince |
| **metacog-threat-model** | `/metacog-threat-model` | Avant securite, auth, production, MCP |
| **metacog-compact** | `/metacog-compact` | Avant handoffs, limites de contexte, longues sessions |
| **metacog-bench** | `/metacog-bench` | Evaluer qualite, calibration, overhead |
| **metacog-tool-audit** | `/metacog-tool-audit` | Avant d'utiliser MCP ou outils repetitifs |
| **metacog-escalate** | `/metacog-escalate` | Quand bloque, risque trop eleve, jugement humain requis |

## 6 Agents

| Agent | Role | Quand deleguer |
|-------|------|---------------|
| **aletheia-main** | Executeur par defaut, context-disciplined | Default |
| **skeptic** | Challenge les hypotheses, trouve des contre-hypotheses | Haute confiance, evidence mince |
| **verifier** | Verification independante des claims et du code | Avant merge, claims critiques |
| **researcher** | Recherche web, exploration de domaines inconnus | Domaines non familiers |
| **repo-cartographer** | Cartographie de la structure repo | Debut de session, avant gros changements |
| **cost-auditor** | Audit context/token overhead | Quand les sessions sont trop couteuses |

## Output Style

| Style | Format |
|-------|--------|
| **aletheia-proof** | Proof packet : Goal, Changes, Evidence, Uncertainty, Next step |

## Organisation fichiers

```
skills/                   # 8 .md files
agents/                   # 6 .md files
output-styles/            # 1 .md file
settings.json             # agent="aletheia-main", outputStyle="aletheia-proof"
```
