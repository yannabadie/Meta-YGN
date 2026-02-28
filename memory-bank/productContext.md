# Product Context

## Overview

Aletheia-Nexus is a local-first metacognitive runtime for AI coding agents. It
adds verification, risk classification, safety gates, learned heuristics, graph
memory, anomaly detection, and context discipline to Claude Code through a thin
plugin shell backed by a Rust daemon.

The product wins by detecting when the AI is likely wrong, asking for evidence
before risky actions, avoiding unnecessary tool use, monitoring human fatigue,
and protecting humans from over-trusting AI output.

## Core Features

### Hooks (8 lifecycle events)
- Risk classification on every user prompt (high/medium/low) with topology recommendation
- Security gates on every tool call via 5-guard pipeline (deny/ask/allow)
- Verification signals after successful tool execution
- Error diagnosis after failed tool execution
- Proof packet enforcement at response completion with metacognitive calibration
- Structured context compaction guidance

### 12-Stage Control Loop
Classify, assess, competence, tool_need, budget, strategy, act, verify,
calibrate, compact, decide, learn. Dynamic topology (Single/Vertical/Horizontal)
skips unnecessary stages for trivial tasks and doubles verification for high-risk.

### Guard Pipeline (5 guards)
- Destructive guard (deny): rm -rf, mkfs, fork bombs
- High-risk guard (ask): git push, terraform, kubectl, sudo
- Secret path guard (ask): .env, .pem, credentials
- MCP guard (ask): any mcp__ tool call
- Default guard (allow): everything else

### Evidence Packs
Tamper-evident audit trails with SHA-256 hash chain, Merkle tree root, and
optional ed25519 signatures for non-repudiation.

### Graph Memory
Nodes (Task/Decision/Evidence/Tool/Agent/Code/Error/Lesson) connected by
edges (DependsOn/Produces/Verifies/Contradicts/Supersedes/RelatedTo) with
FTS5 search and cosine similarity over embeddings. Three scopes: Session,
Project, Global.

### Heuristic Evolver (Layer 0)
Statistical meta-metacognition. Population of heuristic versions with risk
weights and strategy preferences. Evolves through tournament selection and
random mutation based on multi-objective fitness (success rate 0.5, token
efficiency 0.3, latency 0.2).

### MASC Anomaly Detector
TF-IDF cosine similarity against a sliding window of reasoning steps. Detects
divergence (off-track) and stagnation (repetitive reasoning).

### Human Fatigue Profiler
Monitors developer behaviour: short prompts, error loops, late-night work,
rapid retries. High-Friction mode activates at score >= 0.7, restricting
destructive and large-scale changes.

### Tool Forge
Generates ephemeral verification scripts from 4 templates (grep-pattern-checker,
import-validator, json-validator, file-exists-checker). Scripts are content-hashed
for caching and executed in the process sandbox.

### Process Sandbox
Runs Python, Node, and Bash snippets with 5-second timeout and 64 KB output
limit. Supports hypothesis testing (expected_success comparison).

### Skills (8 metacognitive workflows)
Preflight risk assessment, proof packet building, assumption challenging,
threat modeling, tool necessity auditing, context compaction, quality
benchmarking, structured escalation.

### Agents (6 specialized roles)
Default executor (sonnet), skeptic (sonnet), verifier (sonnet), researcher
(haiku), repo-cartographer (haiku), cost-auditor (haiku).

### Output Style
Proof packet format: Goal, Changes, Evidence, Uncertainty, Next step. Evidence
tags: `[confirmed]`, `[experimental]`, `[unverified]`.

### Glass-Box TUI
`aletheia top` -- real-time cognitive telemetry dashboard showing daemon health,
memory stats, fatigue assessment, heuristic population, and graph state.

### CLI
`aletheia start|stop|status|recall|top` -- manage the daemon, query memory,
and launch the TUI dashboard.

## Technical Stack

- **Rust** (7 crates) for the daemon, core logic, memory, sandbox, verifiers, CLI
- **Python 3** (stdlib only) for hook handlers
- **TypeScript** (2 packages) for hook handlers and shared types
- **Markdown** for skills, agents, docs (native Claude Code format)
- **JSON** for plugin config, hooks, settings
- **SQLite** (WAL mode, FTS5) for episodic and graph memory
- **Axum** for async HTTP server
- **ed25519-dalek** for cryptographic signatures
- **sha2** for hash chain and Merkle tree
- **cargo-dist** for release pipeline
