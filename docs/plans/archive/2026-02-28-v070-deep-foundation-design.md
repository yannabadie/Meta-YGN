# v0.7.0 "Deep Foundation" — Design Document

**Date**: 2026-02-28
**Status**: Approved

## Goal
Eliminate all stubs, implement the flagship context pruning proxy, add local embeddings, skill crystallizer, and wire cross-session learning. Every module that was declared must now be real.

## 5 Chantiers

### 1. Stub completion (fts.rs, events.rs, act.rs, compact.rs)
- `fts.rs`: unified search facade over events + graph nodes
- `events.rs`: typed event structs (SessionEvent, ToolEvent, VerificationEvent, RecoveryEvent)
- `act.rs`: record intended action in LoopContext for later verification
- `compact.rs`: real memory compaction (Hot→Warm promotion, Cold eviction, summary generation)

### 2. Context pruning reverse proxy
- HTTP transparent proxy on localhost:11434 (configurable)
- Intercepts Anthropic API payloads, detects 3+ consecutive errors
- Amputates failed reasoning, injects recovery prompt
- Activated with `aletheia start --proxy` or `ALETHEIA_PROXY=1`

### 3. Local embeddings (fastembed-rs)
- `fastembed-rs` with bge-small-en-v1.5 (33MB ONNX)
- EmbeddingProvider trait for swappable backends
- Generate embeddings at MemoryNode insertion
- Vector search via cosine on BLOB-stored embeddings
- Controlled by ALETHEIA_EMBED_PROVIDER=fastembed|none

### 4. Skill crystallizer
- Observe tool patterns from event log
- Detect 3+ repetitions of same tool sequence
- Generate parameterized SKILL.md template
- Save to ~/.claude/aletheia/crystallized-skills/

### 5. Cross-session learning wire-up
- Load best HeuristicVersion from SQLite at daemon boot
- Apply learned thresholds to classify/assess/strategy stages
- Log which heuristic version was active per session
