# MetaYGN PX Audit — Complete Issue List

All issues identified by expert reviews and internal audits, prioritized.
Status: ✅ FIXED | ⬜ OPEN | 🔄 PARTIAL

## P0 — Blocks Basic Correctness

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P0.1 | Token accounting always 0 | ✅ FIXED v0.11.0 | SessionOutcome.tokens_consumed wired |
| P0.2 | Session-scoped profilers | ✅ FIXED v0.11.0 | fatigue/plasticity/budget per session |
| P0.3 | Verification → decide pipeline | ✅ FIXED v0.11.0 | 5 error patterns feed calibrate+decide |
| P0.4 | CLI --host/--port lies | ✅ FIXED v0.11.0 | Flags removed |
| P0.5 | log_event session_id = "daemon" | ✅ FIXED post-merge | Uses real session_id |
| P0.6 | README --host/--port | ✅ FIXED post-merge | Removed from docs |
| P0.7 | Graph edges not auto-inserted | ✅ FIXED post-merge | Task→Evidence→Decision edges |
| P0.8 | Hooks need Bun (not universal) | ✅ FIXED post-merge | Switched to npx tsx |
| P0.9 | MCP metacog_prune is stub | ✅ FIXED v0.12.0 | Wired to ContextPruner |
| P0.10 | Budget display reads global | ✅ FIXED v0.12.0 | Session-local budget in hook output |
| P0.11 | /budget endpoint exposes global only | ⬜ OPEN | Should also expose session budget |
| P0.12 | db_path CLI flag not consumed by daemon | ⬜ OPEN | METAYGN_DB_PATH env var passed but daemon ignores it |

## P1 — Blocks Quality Claims

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P1.1 | daemon-contract.md 90% outdated | ⬜ OPEN | Content is v0.3.0, version says v0.12.0 |
| P1.2 | Tier 2 forge trigger fragile | ✅ FIXED post-merge | Now checks file_path extension |
| P1.3 | Competence stage fixed values | ✅ FIXED v0.12.0 | Adaptive blending with historical rate |
| P1.4 | 10 E2E test scenarios missing | ⬜ OPEN | Only 7 E2E tests, need forge/plasticity/topology/budget scenarios |
| P1.5 | Graph nodes have embedding: None | ⬜ OPEN | No auto-embedding on insert |
| P1.6 | Benchmarks collect-only in CI | ⬜ OPEN | pytest benchmarks/ --collect-only, not executed |
| P1.7 | aletheia eval lacks real Brier score | ⬜ OPEN | Shows counts, not calibration math |
| P1.8 | 8 hooks claimed but daemon routes only 4+1 | ⬜ OPEN | Other 4 handled by TS shell (correct but undocumented) |
| P1.9 | No How-To guide for new users | ✅ FIXED now | docs/HOW-TO.md created |

## P2 — Improves Product Quality

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P2.1 | mcp-bridge crate still exists | ✅ FIXED v0.12.0 | Deleted |
| P2.2 | scripts/ has Python hooks | ✅ FIXED v0.12.0 | Python deleted, only .sh kept |
| P2.3 | Memory-bank progress.md outdated | ✅ FIXED post-merge | v0.11+v0.12 phases added |
| P2.4 | TS packaging not industrialized | ⬜ OPEN | No dist build, main points to .ts |
| P2.5 | tool_input TS type mismatch | ⬜ OPEN | z.record() vs serde_json::Value |
| P2.6 | Naming confusion (Meta-YGN vs Aletheia) | ⬜ OPEN | Cosmetic but confusing |
| P2.7 | pruner.prune() never called in main hooks | ⬜ OPEN | Only analyze() used |
| P2.8 | 3 orphan fields in SessionContext | ⬜ OPEN | tool_calls, success_count write-only |
| P2.9 | OpenSage label too strong | ✅ FIXED v0.10.0 | README says "OpenSage-inspired experimental" |

## P3 — Nice to Have / Future

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P3.1 | WASM sandbox backend | ⬜ DEFERRED | Process sandbox sufficient for now |
| P3.2 | Dialectic topology (teacher-student) | ⬜ DEFERRED | Requires 3-call orchestration |
| P3.3 | LLM-driven heuristic mutation | ⬜ DEFERRED | Requires external LLM (Codex 5.3?) |
| P3.4 | Cross-session learning in competence | 🔄 PARTIAL | Historical rate blending exists, but no domain-specific adaptation |
| P3.5 | Real neural embeddings for graph nodes | ⬜ DEFERRED | EmbeddingGemma-300M or nomic-embed identified |
| P3.6 | Marketplace publication | ⬜ DEFERRED | Plugin validates, needs submission |

## Summary

| Priority | Total | Fixed | Open |
|----------|-------|-------|------|
| P0 | 12 | 10 | 2 |
| P1 | 9 | 3 | 6 |
| P2 | 9 | 4 | 5 |
| P3 | 6 | 0 | 6 |
| **Total** | **36** | **17** | **19** |

## Research: Key Resources for Open Items

### LLM for heuristic evolution (P3.3)
- **GPT-5.3-Codex** (xhigh reasoning): $1.75/M input, $14/M output, 400K context, API available now
- Source: [OpenAI GPT-5.3-Codex](https://openai.com/index/introducing-gpt-5-3-codex/)

### Local multilingual embeddings (P3.5)
- **EmbeddingGemma-300M** (Google): 308M params, 768-dim, 100+ languages (FR+EN), <200MB RAM, ONNX available
- **Qwen3-Embedding-0.6B** (Alibaba): 600M params, 100+ languages, flexible dimensions
- **nomic-embed-text-v2**: open-source, MoE, efficient, multilingual
- Sources: [Google EmbeddingGemma](https://developers.googleblog.com/introducing-embeddinggemma/), [Best Embedding Models 2026](https://www.bentoml.com/blog/a-guide-to-open-source-embedding-models)
