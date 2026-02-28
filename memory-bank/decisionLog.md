# Decision Log

## Phase 1-2 Decisions (v0.1.0 - v0.2.0)

| Date | Decision | Rationale | Status |
|------|----------|-----------|--------|
| 2026-02-28 | Use Python stdlib only for hooks | Zero dependencies, fast startup, no build step | `[confirmed]` |
| 2026-02-28 | 8 lifecycle hooks (added PostToolUseFailure, Stop) | PostToolUseFailure provides error recovery guidance; Stop enforces proof packet | `[confirmed]` |
| 2026-02-28 | 6 agents (added researcher) | Web research needs separate agent with WebSearch/WebFetch tools but no write access | `[confirmed]` |
| 2026-02-28 | 8 skills (added metacog-escalate) | Escalation protocol needed when risk exceeds agent capability or agent is stuck | `[confirmed]` |
| 2026-02-28 | Evidence tagging in proof packets | `[confirmed]`/`[experimental]`/`[unverified]` makes evidence strength explicit for reviewers | `[confirmed]` |
| 2026-02-28 | MCP responses treated as untrusted | PostToolUse emits trust boundary warning for MCP tool outputs. Cross-check against local state. | `[confirmed]` |
| 2026-02-28 | haiku for read-only agents | repo-cartographer and cost-auditor don't need sonnet reasoning; haiku is 10x cheaper | `[confirmed]` |
| 2026-02-28 | 350ms daemon timeout | Non-blocking; allows 2-3 round trips on localhost. Falls back silently. | `[experimental]` |
| 2026-02-28 | Timestamp all log events | ISO 8601 `ts` field enables time-based analysis and debugging | `[confirmed]` |
| 2026-02-28 | Expand security patterns (curl\|bash, sudo, fork bomb) | Original patterns missed common attack vectors | `[confirmed]` |

## Phase 3 Decisions (Rust Runtime)

| Date | Decision | Rationale | Status |
|------|----------|-----------|--------|
| 2026-02-28 | Rust for daemon implementation | Performance, memory safety, Axum async HTTP. No GC pauses on hot path. | `[confirmed]` |
| 2026-02-28 | 7-crate workspace (shared, core, memory, daemon, cli, verifiers, sandbox) | Clean separation of concerns; each crate independently testable | `[confirmed]` |
| 2026-02-28 | SQLite with WAL mode for all storage | Single-file database, zero setup, concurrent reads, FTS5 built-in | `[confirmed]` |
| 2026-02-28 | 12-stage control loop | Comprehensive coverage of the metacognitive cycle; each stage has a single responsibility | `[confirmed]` |
| 2026-02-28 | LoopContext as shared mutable state | Simpler than message-passing between stages; all stages are synchronous | `[confirmed]` |
| 2026-02-28 | 5-guard pipeline (destructive/high_risk/secret_path/mcp/default) | Composable; each guard independently testable; aggregate score is the minimum | `[confirmed]` |
| 2026-02-28 | SHA-256 hash chain + Merkle tree + ed25519 for evidence | Three-layer integrity: tamper detection, compact verification, non-repudiation | `[confirmed]` |
| 2026-02-28 | Dynamic port binding with port file discovery | Avoids port conflicts; clients read ~/.claude/aletheia/daemon.port | `[confirmed]` |
| 2026-02-28 | Axum for HTTP framework | Mature, async, tower-compatible, good ergonomics with extractors | `[confirmed]` |

## Phase 4 Decisions (Advanced Cognitive)

| Date | Decision | Rationale | Status |
|------|----------|-----------|--------|
| 2026-02-28 | Dynamic topology (Single/Vertical/Horizontal) | Trivial tasks skip 8 stages (Single=4); security tasks get double verification (Horizontal=14) | `[confirmed]` |
| 2026-02-28 | Statistical heuristic evolution (no LLM) | OPENSAGE-style mutation is cheaper, deterministic, and doesn't require API calls | `[experimental]` |
| 2026-02-28 | Multi-objective fitness (0.5 success + 0.3 tokens + 0.2 latency) | AlphaEvolve-inspired weights balance correctness against efficiency | `[experimental]` |
| 2026-02-28 | TF-IDF cosine for MASC anomaly detection | Lightweight, no model dependency, real-time computation, no external service needed | `[confirmed]` |
| 2026-02-28 | Anomaly threshold 0.15, stagnation threshold 0.95 | Empirically tuned; lower anomaly threshold catches subtle divergence | `[experimental]` |
| 2026-02-28 | Fatigue profiler with 4 signal types | Short prompts, error loops, late night, rapid retry cover the main behavioural indicators | `[experimental]` |
| 2026-02-28 | High-Friction mode at score >= 0.7 | Conservative threshold; should prevent most fatigue-driven mistakes | `[experimental]` |
| 2026-02-28 | Graph memory with 8 node types and 6 edge types | Sufficient granularity for project-level knowledge graphs without over-engineering | `[confirmed]` |
| 2026-02-28 | 4 forge templates (grep, import, json, file-exists) | Cover the most common verification needs; more can be added without code changes | `[confirmed]` |
| 2026-02-28 | Process-based sandbox (no WASM yet) | WASM adds complexity; process sandbox works cross-platform now, WASM can be feature-gated later | `[confirmed]` |
| 2026-02-28 | Content-hashed tool caching in forge | Avoids re-generating identical scripts; SHA-256 is fast and collision-resistant | `[confirmed]` |
| 2026-02-28 | Context pruner for reasoning lock-in | 3+ consecutive errors trigger corrective injection; prevents infinite error loops | `[confirmed]` |

## Phase 5 Decisions (Distribution)

| Date | Decision | Rationale | Status |
|------|----------|-----------|--------|
| 2026-02-28 | MetaCog-Bench with 15 scenarios across 5 families | Broad coverage: safety, memory, adaptation, verification, calibration | `[confirmed]` |
| 2026-02-28 | Glass-Box TUI for real-time telemetry | Developers need visibility into the cognitive system; terminal-native fits the workflow | `[confirmed]` |
| 2026-02-28 | cargo-dist for release pipeline | Standard Rust distribution tool; produces binaries for Linux/macOS/Windows | `[confirmed]` |
| 2026-02-28 | hook-runner.sh for cross-platform hooks | Single entry point handles Python/TS detection and daemon forwarding | `[confirmed]` |
| 2026-02-28 | install.sh for daemon binary | Downloads pre-built binary from GitHub releases; avoids requiring Rust toolchain | `[confirmed]` |
| 2026-02-28 | Plugin v0.3.0 marketplace packaging | .claude-plugin/plugin.json with postInstall script for daemon setup | `[confirmed]` |
