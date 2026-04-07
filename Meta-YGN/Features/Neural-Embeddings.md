---
title: "Neural Embeddings"
type: feature
evidence_tier: experimental
crate: metaygn-memory
tags:
  - feature
  - experimental
  - embeddings
  - fastembed
created: 2026-04-07
---

# Neural Embeddings

**Tier**: `[experimental]`
**Crate**: `metaygn-memory`
**Fichier principal**: `crates/memory/src/fastembed_provider.rs` + `crates/memory/src/embeddings.rs`

## Description

Provider d'embeddings neuraux via fastembed (ONNX), modele BGE-Small-EN v1.5 (384 dimensions). Feature-gated avec fallback sur HashEmbedProvider ou NoOpProvider.

## Implementation

- **Trait `EmbeddingProvider`** (embeddings.rs) : interface commune `embed()`, `embed_batch()`, `dimension()`, `provider_name()` (Send + Sync)
- **FastEmbedProvider** (fastembed_provider.rs) : utilise `fastembed::TextEmbedding` avec `EmbeddingModel::BGESmallENV15`, download progress supprime. Supporte aussi `with_model()` pour modeles alternatifs.
- **HashEmbedProvider** (embeddings.rs) : fallback sans dependance externe, hashing de termes dans des buckets avec normalisation L2. Pas un vrai embedding neural mais fournit une similarite basique.
- **NoOpProvider** (embeddings.rs) : retourne des vecteurs vides, utilise quand les embeddings sont desactives.
- Feature gate : `#[cfg(feature = "embeddings")]` dans `crates/memory/src/lib.rs`

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Test unitaire | -- | -- |
| Test e2e | -- | non |
| Production | -- | non valide a l'echelle |

## Limitations connues

- Necessite le feature flag `embeddings` a la compilation (dependance ONNX lourde).
- Le modele BGE-Small-EN v1.5 est anglais uniquement.
- Le telechargerment du modele est automatique au premier usage (latence initiale).
- Le HashEmbedProvider est un fallback tres approximatif, pas un vrai remplacement.
