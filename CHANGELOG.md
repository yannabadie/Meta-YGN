# Changelog

## 0.9.0 "Calibrated Mind"
### Added
- **Entropy calibration (EGPO)**: `EntropyTracker` detects overconfidence via sliding window of high-confidence errors
  - Wired into calibrate stage (stage 9): penalizes confidence when overconfidence exceeds 0.3 threshold
  - 20-entry sliding window, configurable
- **Plasticity detection (RL2F)**: 3-level `PlasticityLevel` enum (Responsive/Degraded/Lost)
  - Extends existing `PlasticityTracker` with `plasticity_level()` and `is_plasticity_lost()`
  - Detects when the LLM ignores recovery feedback (2+ consecutive failures)
- **UCB-scored memory retrieval (U-Mem)**: `adaptive_recall` method on `GraphMemory`
  - Blends 70% cosine similarity + 30% UCB exploration bonus
  - `record_recall_reward` for bandit-style feedback on recalled nodes
  - Nodes with `hit_count` and `reward_sum` fields for exploration-exploitation
- **RL trajectory export (RL2F/RLVR)**: structured trajectory capture for external RL pipelines
  - `Rl2fTrajectory` struct with full lifecycle fields (attempt, error, critique, revision, outcome, calibration)
  - `rl2f_trajectories` SQLite table with signature hash for integrity
  - `GET /trajectories/export` daemon endpoint returning signed JSONL
  - `aletheia export` CLI command writes to `~/.claude/aletheia/trajectories/`

## 0.8.0 "Neural Bridge"
### Added
- **MCP Bridge**: new `metaygn-mcp-bridge` crate with 5 metacognitive MCP tools via stdio transport (rmcp 0.17)
  - Tools: `metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`
  - CLI command `aletheia mcp` launches the MCP stdio server
- **Neural embeddings**: `FastEmbedProvider` implementing `EmbeddingProvider` trait (bge-small-en-v1.5, 384 dim)
  - Feature-gated behind `cargo build --features embeddings` — zero overhead when disabled
  - `GraphMemory.semantic_search()` for cosine-similarity vector search
  - `POST /memory/semantic` daemon endpoint for vector-based node retrieval
- **Session replay**: timeline recording of all hook calls for post-session debugging
  - `replay_events` SQLite table recording request/response/latency for every hook call
  - `GET /replay/sessions` and `GET /replay/{session_id}` daemon API endpoints
  - `aletheia replay` CLI command: list sessions or view hook timeline

### Changed
- All 4 hook handlers (pre-tool-use, post-tool-use, user-prompt-submit, stop) now record replay events automatically
- Workspace expanded to 8 crates (added `mcp-bridge`)

## 0.7.0 "Deep Foundation"
### Added
- **Typed event system**: 11 `MetaEvent` variants replacing ad-hoc string logging
- **Unified FTS search**: single query across events and graph nodes
- **Context pruning service**: `POST /proxy/anthropic` analyzes and prunes error loops from message payloads
- **Embedding provider trait**: pluggable `EmbeddingProvider` with hash-based and no-op implementations
- **Skill crystallizer**: auto-detects recurring tool patterns and generates SKILL.md templates
- **Cross-session learning**: daemon loads heuristic versions and outcomes from SQLite at startup

### Changed
- `act` stage now records intended actions for post-verification comparison
- `compact` stage generates real summaries and deduplicates lessons (was no-op)
- Heuristic mutations and outcomes are persisted to SQLite after every change
- All 4 stub modules (events, fts, act, compact) are now fully implemented

## 0.6.0 "Solid Ground"
### Added
- **Complete TypeScript hooks**: all 8 hooks (session-start, user-prompt-submit, pre-tool-use, post-tool-use, post-tool-use-failure, stop, pre-compact, session-end) implemented in TypeScript with Bun runtime
- **Daemon graceful lifecycle**: `POST /admin/shutdown` endpoint triggers clean shutdown with port file cleanup
- **`aletheia start`**: spawns daemon as detached process, polls for port file, health checks
- **`aletheia stop`**: sends shutdown request, waits for clean termination
- **Persistent heuristics**: HeuristicVersion and SessionOutcome stored in SQLite, survive daemon restart
- **CI integration tests**: GitHub Actions job that starts daemon, tests hooks, verifies budget, shuts down
- **Plugin validation script**: `scripts/validate-plugin.sh` checks all 26 plugin components
- Local fallback functions for user-prompt-submit, post-tool-use, and stop hooks

### Changed
- `hooks/hooks.json` now uses `bun run` for all hooks (was `bash hook-runner.sh`)
- Daemon supports dual shutdown triggers: Ctrl+C and `/admin/shutdown`
- CLI `start` finds `aletheiad` binary automatically next to the CLI executable

## 0.5.0 "Smart Recovery"
### Added
- **Plasticity Tracker**: implicit feedback loop measuring recovery prompt effectiveness (success/failure tracking)
- **Progressive amplification**: 3-level recovery (standard → emphatic → escalation via /metacog-escalate)
- **Latency tracking**: every hook response includes `[latency: Nms]`
- **`aletheia init`**: CLI command for project onboarding (generates .claude/settings.json)
- Human-readable hook messages (Risk: HIGH | Strategy: verify-first | Budget: N tokens)

### Fixed
- Pre-tool-use risk classification now evaluates the actual command text, not hook metadata
- `ls -la` correctly classified as LOW risk (was incorrectly HIGH)
- Safe Bash commands (ls, cat, cargo test, git status, etc.) now have proper risk levels

### Changed
- Stop hook now uses plasticity-aware amplification for recovery prompts
- PostToolUse hook records implicit feedback (success/failure) for pending recoveries

## 0.4.0 "Developer-First"
### Added
- **Completion Verifier**: catches false "Done!" claims by checking that mentioned files actually exist on disk
- **Test Integrity Guard**: detects when Claude weakens test assertions instead of fixing implementation code (asks for confirmation)
- **Token Budget Tracker**: visible `[budget: Ntok/$X.XX used | Y%]` in every hook response
- `GET /budget` endpoint for current session budget status
- `POST /budget/consume` endpoint for recording token consumption
- Budget warning at 80% utilization, over-budget detection

### Changed
- Stop hook now runs completion verification before accepting "Done!" responses
- PreToolUse hook now checks test file integrity on Edit/MultiEdit operations
- All hook responses now include budget summary in additionalContext
- UserPromptSubmit hook estimates and tracks token consumption

## 0.3.0 "Adaptive Topology"
### Added
- Graph memory with nodes, edges, FTS5 search, cosine similarity (crates/memory/)
- Dynamic topology planner with skip-routing: Single/Vertical/Horizontal (crates/core/)
- Heuristic evolver — Layer 0 meta-metacognition with statistical fitness (crates/core/)
- Tool forge with 4 verification templates, sandbox execution, caching (crates/daemon/)
- MetaCog-Bench evaluation framework with 15 scenarios across 5 families (eval/)
- MASC anomaly detector with TF-IDF cosine similarity (crates/core/)
- Human fatigue profiler — inverse metacognition (crates/daemon/)
- Glass-Box TUI dashboard — aletheia top (crates/cli/)
- Process-based sandbox with timeout and hypothesis testing (crates/sandbox/)
- New daemon endpoints: /memory/graph/*, /heuristics/*, /forge/*
- Topology enum (Single/Vertical/Horizontal) in shared types

### Changed
- Control loop now supports run_plan() for executing stage subsets
- Daemon AppState includes graph memory, evolver, forge engine
- Hook responses include topology recommendation

## 0.2.0
### Added
- **PostToolUseFailure hook**: error diagnosis guidance for failed tool calls
- **Stop hook**: proof packet enforcement at end of responses
- **researcher agent**: web research and documentation exploration
- **metacog-escalate skill**: structured escalation protocol for high-risk or stuck situations
- MCP matcher on PostToolUse (treats external tool output as untrusted data)
- Timeout and statusMessage fields on all hook entries
- Timestamp (`ts`) field on all event log entries
- Expanded destructive patterns: fork bombs, chmod 777 /, raw disk writes
- Expanded high-risk patterns: curl|bash, docker push/prune, sudo, git rebase, az/gcloud
- Expanded secret path patterns: .key, id_ed25519, .npmrc, .pypirc, kubeconfig, service accounts
- Expanded stack detection: Go, Java, Kotlin, Swift, Ruby, PHP, Elixir, Deno, Bun, .NET, Docker, Terraform, CMake
- Evidence tagging in proof and compact skills: `[confirmed]`, `[experimental]`, `[unverified]`
- Keywords field in plugin.json for discovery
- Risk-differentiated hints in UserPromptSubmit (high/medium/low get different guidance)
- Daemon notification on SessionStart, PreCompact, and SessionEnd

### Changed
- **All agents**: added `description`, `skills`, `disallowedTools`, and `memory` frontmatter fields per Claude Code spec
- **All skills**: added `user-invocable`, `argument-hint`, improved descriptions, added output format templates
- **aletheia-main agent**: added delegation rules for when to invoke skeptic, verifier, researcher
- **common.py**: restructured with section headers, error-safe logging, expanded classification markers
- **CLAUDE.md**: added Skills and Agents reference tables, escalation policy, MCP trust boundary rules
- **README.md**: added architecture diagram, full component tables, security gates documentation
- **output-styles/aletheia-proof.md**: added evidence tagging and honest uncertainty guidance
- **pre_compact.py**: structured numbered list output instead of single-line text
- **session_start.py**: handles resume/clear/compact sources differently
- **user_prompt_submit.py**: risk-differentiated output (high suggests threat-model, low suggests lean workflow)
- **session_end.py**: notifies daemon for session finalization

### Fixed
- PostToolUse now matches MCP tools (was missing, inconsistent with PreToolUse)
- Secret path patterns now catch .key files, ed25519 keys, and cloud credential files
- Log events now include ISO 8601 timestamps for auditability
- Logging failures no longer crash hook execution (wrapped in try/except)

## 0.1.0
- Initial Aletheia-Nexus Claude Code plugin skeleton.
