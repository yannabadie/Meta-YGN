---
title: Competitive Landscape
type: architecture
tags:
  - architecture
  - competitive-analysis
  - strategy
updated: 2026-04-07
---

# Competitive Landscape — Agent Safety & Metacognition

## Positionnement

Aletheia-Nexus est le seul **runtime metacognitif** : il ne se contente pas de filtrer des actions, il detecte quand l'agent a probablement tort, demande des preuves, et escalade quand l'incertitude reste haute. Aucun concurrent ne combine verification en cascade, calibration d'incertitude, memoire adaptative, et auto-monitoring fatigue.

## Matrice comparative

| Produit | Type | Forces | Faiblesses vs nous |
|---------|------|--------|-------------------|
| **Guardrails AI** | Framework open source | Validators composables, Hub marketplace, adoption large | Pas de sequence detection, pas de contexte de tache, pas de metacognition |
| **NeMo Guardrails** | NVIDIA, Colang v2 | DSL event-driven, 5 types de rails, enterprise backing | Lie a NVIDIA stack, pas de verification formelle, pas d'apprentissage en ligne |
| **LlamaFirewall** | Meta, open source | Prompt injection + jailbreak + alignment, CodeShield | Code-centric, pas de sequence monitoring, pas de calibration d'incertitude |
| **Galileo Agent Control** | SaaS commercial | Observabilite + hallucination detection | Cloud-only, pas de runtime local, pas de cascade verification |
| **Snyk/Invariant** | Security scanner | MCP security, supply-chain analysis | Static analysis only, pas de runtime monitoring |

## Concurrents directs par categorie

### Frameworks guardrails open source

**Guardrails AI** (guardrailsai.com)
- Validators composables via Guardrails Hub
- Structured output validation (JSON, types)
- Integrations LangChain, LlamaIndex
- Pas de sequence detection ni contexte de tache
- Pas d'apprentissage en ligne des seuils

**NeMo Guardrails + Colang v2** (NVIDIA)
- DSL declaratif pour definir des rails (input, output, dialog, topic, retrieval)
- Event-driven architecture
- Enterprise backing et integration NVIDIA NIM
- Lie au stack NVIDIA, DSL proprietaire
- Pas de verification DTMC/PCTL

**LlamaFirewall** (Meta)
- Prompt injection guard + jailbreak detection + alignment checking
- CodeShield pour analyse de code genere
- PromptGuard fine-tuned model
- Focus code generation safety, pas agent runtime complet
- Pas de fatigue profiling ni calibration d'incertitude

### Claude Code plugin ecosystem

**Rulebricks** (plugin Claude Code)
- Rules engine for decision logic
- No-code interface for business rules
- Pas de safety focus, pas de verification formelle

**Sentinel AI** (plugin Claude Code)
- Token monitoring, cost tracking
- Basic guardrails
- Pas de cascade verification, pas de metacognition

**Trail of Bits** (outils securite)
- Security-focused tooling (Semgrep, etc.)
- Static analysis, vulnerability detection
- Pas de runtime agent monitoring

### MCP Security

**Snyk MCP Security** (Invariant Labs)
- Static analysis des MCP server configurations
- Supply-chain attack detection
- Pas de runtime monitoring

**MCP-Guard** (papier academique)
- Three-stage defense cascade pour MCP
- Input validation + tool authorization + output verification
- Academique, pas de produit deploye

**ShieldNet** (papier academique)
- Defense contre supply-chain attacks MCP
- Server authentication + capability attestation
- Academique, pas de produit deploye

### Observabilite agent

**Galileo Agent Control**
- Hallucination detection via Luna metric
- Agent tracing et debugging
- Cloud SaaS, pas de local-first
- Pas de verification en cascade

**LangSmith** (LangChain)
- Tracing, monitoring, evaluation
- Playground et dataset management
- Focuse developer experience, pas safety
- Pas de metacognition

## Notre differentiation

### Ce que nous sommes seuls a avoir

| Capacite | Aletheia-Nexus | Guardrails AI | NeMo | LlamaFirewall | Galileo |
|----------|---------------|---------------|------|---------------|---------|
| Cascade verification 5 tiers | Oui (ADR-004) | Non | Non | Non | Non |
| Sequence detection DTMC | Oui (Pro2Guard-inspired) | Non | Non | Non | Non |
| Calibration d'incertitude | Oui (entropy, EGPO) | Non | Non | Non | Partiel |
| Fatigue profiling | Oui (hint->critique->escalate) | Non | Non | Non | Non |
| Memoire adaptative UCB | Oui (UCB1 ranking) | Non | Non | Non | Non |
| Heuristic evolution | Oui (outcome-driven) | Non | Non | Non | Non |
| Local-first runtime | Oui (daemon Rust) | Cloud/local | Cloud | Local | Cloud |
| Proof packets | Oui | Non | Non | Non | Non |
| Test integrity monitor | Oui (assertion weakening) | Non | Non | Non | Non |
| Plasticity detection | Oui (RL2F-inspired) | Non | Non | Non | Non |

### Position strategique

```
                    Metacognition
                        ^
                        |
          Aletheia-Nexus * (seul ici)
                        |
                        |
    Static ----+--------+--------+---- Runtime
    Analysis   |        |        |     Monitoring
               |        |        |
         Snyk  *  NeMo  *  Galileo *
               |        |        |
               |   Guardrails    |
               |     AI  *      |
               |        |        |
               |  LlamaFirewall  |
               |        *        |
                        |
                        v
                    Guardrails
```

## Risques concurrentiels

1. **NVIDIA/Meta pourraient ajouter de la metacognition** a NeMo/LlamaFirewall — mais leur architecture n'est pas concue pour
2. **Anthropic pourrait integrer certaines capacites dans Claude Code natif** — attenuation : nous sommes AI-agnostic, pas un plugin Claude-only
3. **Guardrails AI a plus de communaute** — attenuation : notre depth est incomparable
4. **Galileo a du funding** — attenuation : local-first vs cloud-first, differents marches

## Evidence tag

`[experimental]` — Analyse basee sur documentation publique et papers. Les comparaisons specifiques peuvent etre incompletes. Actualise en avril 2026.
