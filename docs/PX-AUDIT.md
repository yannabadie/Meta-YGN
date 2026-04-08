# MetaYGN PX Audit — Complete Issue List

All issues identified by expert reviews and internal audits, prioritized.
Status: ✅ FIXED | ⬜ OPEN | 🔄 PARTIAL | ⏭️ DEFERRED

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
| P0.11 | /budget endpoint exposes global only | ✅ FIXED v1.0.0 | GET /budget/{session_id} added |
| P0.12 | db_path CLI flag not consumed by daemon | ✅ FIXED v1.0.0 | Daemon reads METAYGN_DB_PATH env var |

## P1 — Blocks Quality Claims

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P1.1 | daemon-contract.md 90% outdated | ✅ FIXED v1.0.0 | Full rewrite with all endpoints, version from CARGO_PKG_VERSION |
| P1.2 | Tier 2 forge trigger fragile | ✅ FIXED post-merge | Now checks file_path extension |
| P1.3 | Competence stage fixed values | ✅ FIXED v0.12.0 | Adaptive blending with historical rate |
| P1.4 | Broader E2E coverage still incomplete | 🔄 PARTIAL | 10 daemon E2E scenarios automated; broader Claude Code / plugin flows still validated manually |
| P1.5 | Graph nodes have embedding: None | ✅ FIXED v1.0.0 | Auto-embed via EmbeddingProvider in postprocess |
| P1.6 | Benchmarks collect-only in CI | ⏭️ DEFERRED | No Python benchmarks in current stack |
| P1.7 | aletheia eval lacks real Brier score | ✅ FIXED v1.0.0 | /calibration endpoint + Brier formula + buckets |
| P1.8 | 8 hooks claimed but daemon routes only 4+1 | ✅ FIXED v1.0.0 | 5 daemon routes + 3 TS-only, documented in architecture-notes.md |
| P1.9 | No How-To guide for new users | ✅ FIXED v1.0.0 | docs/HOW-TO.md created |

## P2 — Improves Product Quality

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P2.1 | mcp-bridge crate still exists | ✅ FIXED v0.12.0 | Deleted |
| P2.2 | scripts/ has Python hooks | ✅ FIXED v0.12.0 | Python deleted, only .sh kept |
| P2.3 | Memory-bank progress.md outdated | ✅ FIXED post-merge | v0.11+v0.12 phases added |
| P2.4 | TS packaging not industrialized | ✅ FIXED v1.0.0 | pnpm build + typecheck pipeline added |
| P2.5 | tool_input TS type mismatch | ✅ FIXED v1.0.0 | Explicit typeof check in fallback.ts |
| P2.6 | Naming confusion (Meta-YGN vs Aletheia) | ⏭️ DEFERRED | Cosmetic, README leads with Aletheia-Nexus |
| P2.7 | pruner.prune() never called in main hooks | ✅ N/A | Pruner operates in proxy context, not hooks (by design) |
| P2.8 | 3 orphan fields in SessionContext | ✅ FIXED v1.0.0 | tool_calls wired to outcome logging |
| P2.9 | OpenSage label too strong | ✅ FIXED v0.10.0 | README says "OpenSage-inspired experimental" |

## P3 — Nice to Have / Future

| # | Issue | Status | Detail |
|---|-------|--------|--------|
| P3.1 | WASM sandbox backend | ⏭️ DEFERRED | Process sandbox sufficient for v1.0 |
| P3.2 | Dialectic topology (teacher-student) | ⏭️ DEFERRED | Requires 3-call orchestration |
| P3.3 | LLM-driven heuristic mutation | ⏭️ DEFERRED v1.1 | GPT-5.3-Codex xhigh identified |
| P3.4 | Cross-session learning in competence | 🔄 PARTIAL | Historical rate blending exists, domain-specific v1.1 |
| P3.5 | Real neural embeddings for graph nodes | ✅ FIXED v1.0.0 | EmbeddingProvider wired, fastembed behind feature flag |
| P3.6 | Marketplace publication | ⏭️ DEFERRED | Plugin validates, submission ready |

## Summary

| Priority | Total | Fixed | Remaining |
|----------|-------|-------|-----------|
| P0 | 12 | **12** | 0 |
| P1 | 9 | **8** | 1 partial (P1.4 broader E2E breadth) |
| P2 | 9 | **8** | 1 (P2.6 naming, cosmetic) |
| P3 | 6 | **1** | 5 (deferred to v1.1) |
| **Total** | **36** | **29** | **7** |

**v1.0.0 released. All P0 resolved. P1.4 is now partially automated with 10 daemon E2E scenarios, but broader Claude Code / plugin E2E remains. All P2 resolved except cosmetic naming. P3 deferred to v1.1.**

## Research: Key Resources for Future Work

### LLM for heuristic evolution (P3.3)
- **GPT-5.3-Codex** (xhigh reasoning): $1.75/M input, $14/M output, 400K context, API via Responses API
- Source: [OpenAI GPT-5.3-Codex](https://openai.com/index/introducing-gpt-5-3-codex/)

### Local multilingual embeddings (P3.5 — implemented)
- **BGE-Small-EN v1.5** (current default): 384d, English, fast, via fastembed v4
- **EmbeddingGemma-300M** (upgrade path): 768d, 100+ languages (FR+EN), <200MB, ONNX
- **Qwen3-Embedding-0.6B** (code-aware): 600M params, code + natural language
- Sources: [Google EmbeddingGemma](https://developers.googleblog.com/introducing-embeddinggemma/), [Best Embedding Models 2026](https://www.bentoml.com/blog/a-guide-to-open-source-embedding-models)

### Marketplace submission (P3.6)
- Plugin validates: `claude plugin validate .` passes
- Submission: [clau.de/plugin-directory-submission](https://clau.de/plugin-directory-submission)
- Alternative: Self-hosted marketplace via Git repo with `.claude-plugin/marketplace.json`
