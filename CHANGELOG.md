# Changelog

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
