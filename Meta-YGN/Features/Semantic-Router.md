---
title: Semantic Router
tier: experimental
crate: metaygn-daemon
feature_gate: "--features semantic"
tags:
  - routing
  - knn
  - risk
updated: 2026-04-07
---

# Semantic Router

kNN-based risk classification for tool calls. Provides context-aware risk assessment as a lightweight alternative to full pipeline evaluation for well-known patterns.

## Design

- **37 labeled examples** covering common safe and dangerous tool patterns
- **kNN with k=5**: majority vote among 5 nearest neighbors
- **`classify_with_confidence`**: returns risk level + confidence score
- **Override policy**: when confidence >= 0.8, the semantic match overrides the `assess` stage risk classification

## API

- `routing_hint()` returns `SemanticMatch{confidence}` for safe matches
- `classify_with_confidence()` on `SemanticRouter` returns `(RiskLevel, f64)`

## Feature gate

Enabled with `--features semantic`. When disabled, the router is a no-op and the assess stage runs unconditionally.

## References

- RouteLLM (efficient router for LLM routing)
- Cascade Routing (adaptive model selection)
- BaRP (Bayesian Risk-aware Planning)
