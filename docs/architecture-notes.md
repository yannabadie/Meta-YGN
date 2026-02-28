# Architecture Notes

Status: `[confirmed]` -- validated through implementation in v0.3.0 "Adaptive Topology"

---

## Core Principles

### 1. Keep the plugin shell thin
The plugin provides hooks, skills, agents, and output styles. All durable state,
learned heuristics, graph memory, and advanced verification live in the Rust
daemon. The plugin is an adapter, not the brain.

### 2. Route durable state to the runtime
Hook scripts call `ALETHEIA_DAEMON_URL` when available. If the daemon is down,
they fall back to deterministic local heuristics. This ensures the plugin always
works, but gets smarter when the daemon is connected.

### 3. Treat MCP and LSP as optional edge adapters
MCP tools expand the trust boundary and consume context tokens for schemas.
Use them only when a capability truly requires an external process. Prefer
local CLI for the hot path.

### 4. Prefer verification and observability over verbose self-reflection
Structured proof packets (Goal, Changes, Evidence, Uncertainty, Next step) are
more useful than paragraphs of reasoning. External verification (tests, lints,
type checks) is stronger than self-assessment.

### 5. Let the system learn from itself (Layer 0)
The heuristic evolver statistically mutates risk weights and strategy preferences
based on real session outcomes. No LLM in the loop -- pure evolutionary search.

---

## System Architecture Diagram

```
                                   ┌──────────────────────────────────┐
                                   │       Claude Code Agent          │
                                   │  (user prompts + tool calls)     │
                                   └──────────┬───────────────────────┘
                                              │ hooks.json
                                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│                     Plugin Shell (Tier 2)                            │
│                                                                      │
│  hooks/           scripts/         agents/        skills/            │
│  hooks.json       hook-runner.sh   aletheia-main  metacog-preflight  │
│                   common.py        skeptic         metacog-proof      │
│  packages/        *.py hooks       verifier        metacog-challenge  │
│  hooks/ (TS)                       researcher      metacog-threat     │
│  shared/ (TS)                      repo-carto.     metacog-compact    │
│                                    cost-auditor    metacog-quality    │
│                                                    metacog-tool-audit │
│  output-styles/                                    metacog-escalate   │
│  aletheia-proof                                                      │
└─────────────────────────┬────────────────────────────────────────────┘
                          │ HTTP (127.0.0.1:{port})
                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                   Aletheia Daemon (Tier 1)                           │
│                   aletheiad / crates/daemon                          │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ Guard        │  │ Control Loop │  │ Topology     │               │
│  │ Pipeline     │  │ (12 stages)  │  │ Planner      │               │
│  │ (5 guards)   │  │              │  │ S/V/H        │               │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │
│         │                 │                 │                        │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼───────┐               │
│  │ MASC Monitor │  │ Fatigue      │  │ Heuristic    │               │
│  │ (anomaly     │  │ Profiler     │  │ Evolver      │               │
│  │  detector)   │  │ (human)      │  │ (Layer 0)    │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ Episodic     │  │ Graph        │  │ Tool Forge   │               │
│  │ Memory       │  │ Memory       │  │ (4 templates)│               │
│  │ (FTS5)       │  │ (FTS5+cos)   │  │              │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ Process      │  │ Evidence     │  │ Context      │               │
│  │ Sandbox      │  │ Packs        │  │ Pruner       │               │
│  │ (py/node/sh) │  │ (hash+merkle │  │              │               │
│  │              │  │  +ed25519)   │  │              │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│                                                                      │
│  ┌──────────────────────────────────────────────────┐               │
│  │ Glass-Box TUI  (aletheia top)                     │               │
│  │ Real-time cognitive telemetry dashboard            │               │
│  └──────────────────────────────────────────────────┘               │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                     Tier 3: Edge Adapters (optional)                  │
│  - MCP: only for capabilities needing external processes             │
│  - mcp-bridge crate: planned for future (v0.4.0+)                   │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                     Evaluation (eval/)                                │
│  MetaCog-Bench: 15 scenarios across 5 families (Python)              │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Crate / Package Map

### Rust Crates (7)

| Crate              | Path              | Purpose                                              |
|--------------------|-------------------|------------------------------------------------------|
| `metaygn-shared`   | `crates/shared`   | Protocol types, state enums, kernel, events           |
| `metaygn-core`     | `crates/core`     | 12-stage control loop, topology planner, MASC monitor, heuristic evolver |
| `metaygn-memory`   | `crates/memory`   | Episodic memory (FTS5), graph memory (nodes+edges+FTS5+cosine), tiered storage |
| `metaygn-verifiers`| `crates/verifiers` | Guard pipeline (5 guards), evidence packs (hash chain+Merkle+ed25519) |
| `metaygn-sandbox`  | `crates/sandbox`  | Process-based sandbox with timeout, hypothesis testing |
| `metaygn-daemon`   | `crates/daemon`   | Axum HTTP server, all API routes, forge engine, fatigue profiler, context pruner |
| `metaygn-cli`      | `crates/cli`      | CLI (`aletheia` command), Glass-Box TUI dashboard     |

Note: an `mcp-bridge` crate is planned for future work to provide native MCP
server integration.

### TypeScript Packages (2)

| Package         | Path              | Purpose                                        |
|-----------------|-------------------|-------------------------------------------------|
| `@metaygn/hooks`  | `packages/hooks`  | TS hook handlers with daemon client + fallback |
| `@metaygn/shared` | `packages/shared` | Shared TypeScript types                        |

### Python Evaluation (1)

| Package         | Path    | Purpose                                          |
|-----------------|---------|--------------------------------------------------|
| `metaygn-eval`  | `eval/` | MetaCog-Bench: 15 scenarios across 5 families    |

---

## 12-Stage Control Loop

The control loop is the core of the metacognitive pipeline. Each stage reads/writes
a shared `LoopContext` and can signal Continue, Skip, or Escalate.

```
 Stage  1: classify     — classify task type (Bugfix/Feature/Refactor/Architecture/Security/Research/Release)
 Stage  2: assess       — assess difficulty (0.0-1.0) + risk level (Low/Medium/High)
 Stage  3: competence   — self-assess competence for this task type
 Stage  4: tool_need    — determine if a tool invocation is necessary
 Stage  5: budget       — allocate token/cost/latency budget based on risk
 Stage  6: strategy     — select reasoning strategy (StepByStep/TreeExplore/VerifyFirst/DivideConquer/Analogical/Adversarial/Rapid/Iterative)
 Stage  7: act          — execute (no-op in the daemon; actual execution is in Claude Code)
 Stage  8: verify       — verify tool output against expectations
 Stage  9: calibrate    — calibrate 5-D metacognitive vector (confidence/coherence/grounding/complexity/progress)
 Stage 10: compact      — memory compaction signal (context pruning)
 Stage 11: decide       — make final decision (Continue/Revise/Abstain/Escalate/Stop)
 Stage 12: learn        — collect lessons from this iteration
```

### Dynamic Topology (Skip-Routing)

The `TopologyPlanner` selects which subset of stages to execute based on task
characteristics. This transforms the fixed 12-stage pipeline into a dynamic system.

| Topology     | Stages | When                                     |
|--------------|--------|------------------------------------------|
| **Single**   | 4      | Low risk + trivial difficulty (< 0.2): classify, assess, act, decide |
| **Vertical** | 12     | Default: all 12 stages sequentially      |
| **Horizontal** | 14   | High risk OR security tasks: all 12 + double verify+calibrate pass |

Research tasks use a slim 6-stage vertical pipeline (classify, assess, competence,
strategy, act, learn), skipping heavy verification overhead.

---

## Guard Pipeline (5 Guards)

Every `PreToolUse` hook call runs the full guard pipeline. Guards execute in order;
if any guard blocks, the pipeline returns a deny/ask decision. All guards run
regardless (for observability), and the aggregate score is the minimum.

| Guard            | Score on Block | Decision | Patterns                                    |
|------------------|----------------|----------|---------------------------------------------|
| `destructive`    | 0              | Deny     | `rm -rf /`, `mkfs`, `dd if=`, `shutdown`, `reboot`, fork bomb, `chmod 777 /` |
| `high_risk`      | 30             | Ask      | `git push`, `git reset --hard`, `terraform apply/destroy`, `kubectl apply/delete`, `curl|bash`, `sudo`, `docker push/prune` |
| `secret_path`    | 20             | Ask      | `.env`, `secrets/`, `.pem`, `.key`, `id_rsa`, `credentials.json`, `.npmrc`, `.pypirc`, `kubeconfig` |
| `mcp`            | 40             | Ask      | Any tool name starting with `mcp__`          |
| `default`        | 100            | Allow    | Everything else                              |

---

## Evidence Packs

Evidence packs provide tamper-evident, cryptographically-signed audit trails for
AI actions. Each pack maintains three integrity layers:

1. **Hash Chain**: Each entry stores the SHA-256 hash of the previous entry's
   serialized form. First entry has `prev_hash = [0; 32]`.
2. **Merkle Tree**: All entries are leaf-hashed and combined into a Merkle root
   for compact integrity verification.
3. **Ed25519 Signatures**: Optional per-session signing key signs the last entry.
   Public key is available for third-party verification.

---

## MASC Anomaly Detector

Metacognitive Anomaly via Similarity of Context (MASC) detects when the AI's
current reasoning step is anomalous compared to its history. Uses TF-IDF
cosine similarity against a sliding window of recent reasoning steps.

| Condition                     | Flag        | Meaning                     |
|-------------------------------|-------------|-----------------------------|
| similarity < 0.15 (default)   | Anomaly     | Reasoning diverged (off-track) |
| similarity > 0.95 (default)   | Stagnation  | Reasoning is repeating       |
| 0.15 <= similarity <= 0.95    | Normal      | Healthy variation            |

---

## Human Fatigue Profiler

"Inverse metacognition" -- the system monitors the human developer's behaviour
to protect the codebase when the human is exhausted.

| Signal          | Weight | Trigger                              |
|-----------------|--------|--------------------------------------|
| Short prompt    | 0.15   | Prompt < 20 characters               |
| Error loop      | 0.30   | 3+ consecutive errors                |
| Late night      | 0.20   | Prompts between 23:00-05:00          |
| Rapid retry     | 0.15   | < 5 seconds between prompts          |

When cumulative score >= 0.7, **High-Friction mode** activates: refuse major
refactors, require tests before destructive actions.

---

## Heuristic Evolver (Layer 0)

Statistical meta-metacognition. Maintains a population of `HeuristicVersion`s,
each containing risk-weight and strategy-preference parameters. Evolution cycle:

1. **Evaluate** all versions against recent `SessionOutcome`s
2. **Select** top performers (tournament selection, capped at max_population=20)
3. **Mutate** the best version (one random mutation per generation):
   - Adjust a random risk_weight by +/-10-20%
   - Adjust a random strategy_score by +/-10-20%
   - Add or remove a risk marker

Fitness is multi-objective (AlphaEvolve-inspired):
- Verification success rate (weight: 0.5)
- Token efficiency (weight: 0.3)
- Latency score (weight: 0.2)

---

## Tool Forge

Generates, caches, and executes ephemeral verification scripts from templates.
Scripts are content-hashed (SHA-256) for deduplication.

| Template              | Language | Purpose                      |
|-----------------------|----------|------------------------------|
| grep-pattern-checker  | Python   | Regex search over text       |
| import-validator      | Python   | Check Python imports         |
| json-validator        | Python   | Validate JSON structure      |
| file-exists-checker   | Bash     | Check if files exist         |

---

## Graph Memory

Nodes and edges stored in SQLite with FTS5 full-text search over labels and
content, plus cosine similarity over optional embedding vectors.

**Node types**: Task, Decision, Evidence, Tool, Agent, Code, Error, Lesson
**Edge types**: DependsOn, Produces, Verifies, Contradicts, Supersedes, RelatedTo
**Scopes**: Session, Project, Global

Features:
- BFS neighbor traversal with configurable depth
- FTS5-backed content search with triggers for auto-indexing
- Cosine similarity for embedding-based retrieval
- WAL mode with 5 s busy timeout for concurrent access

---

## Context Pruner

Analyses message history for reasoning lock-in (3+ consecutive errors) and
suggests injection of corrective prompts to break the agent out of loops.

---

## Process Sandbox

Runs code snippets (Python, Node, Bash) as sub-processes with:
- 5-second timeout (configurable)
- 64 KB output limit
- Graceful kill on timeout
- Windows-compatible (Git Bash detection, Python command resolution)

---

## Glass-Box TUI

`aletheia top` -- real-time cognitive telemetry dashboard. Polls the daemon's
health, memory, profiler, heuristics, and graph endpoints to display a live
overview of the metacognitive system state.

---

## Hook Execution Flow

```
SessionStart
  |
UserPromptSubmit (per user input)
  -> classify risk -> topology plan -> emit strategy hint
  |
PreToolUse (per tool call)
  -> guard_pipeline -> control_loop stages 1-6 -> Allow/Ask/Deny
  |
Tool Execution
  |
PostToolUse (on success)     PostToolUseFailure (on failure)
  -> verification signals      -> error diagnosis guidance
  -> fatigue signal (success)  -> fatigue signal (error)
  |                            |
Stop (when Claude finishes)
  -> control_loop stages 9-12
  -> proof packet enforcement
  -> metacognitive calibration
  |
PreCompact (on context limit or manual trigger)
  -> structured compaction guidance
  |
SessionEnd (async)
  -> event log + daemon notification
```

---

## Agent Coordination

```
aletheia-main (orchestrator, sonnet)
  |-- skeptic (challenge assumptions, sonnet)
  |-- verifier (independent checks, sonnet)
  |-- researcher (web research, haiku)
  |-- repo-cartographer (structure mapping, haiku)
  +-- cost-auditor (overhead analysis, haiku)
```

---

## Security Model

Three-tier permission gating:

| Tier            | Pattern                                  | Decision  | Example              |
|-----------------|------------------------------------------|-----------|----------------------|
| Destructive     | `rm -rf /`, fork bomb, `mkfs`            | **Deny**  | Always blocked       |
| High-risk       | `git push`, `sudo`, `terraform apply`    | **Ask**   | Requires confirmation|
| Sensitive path  | `.env`, `.pem`, `credentials.json`       | **Ask**   | Requires confirmation|
| MCP call        | `mcp__*`                                 | **Ask**   | Trust boundary       |
| Default         | Everything else                          | **Allow** | Proceeds normally    |

---

## Design Decisions

| Decision | Rationale | Evidence | Status |
|----------|-----------|----------|--------|
| Python for hooks | Zero dependencies, stdlib only, fast startup | v0.1.0-v0.2.0 | `[confirmed]` |
| Rust for daemon | Performance, safety, Axum async HTTP | v0.3.0 | `[confirmed]` |
| Markdown for skills/agents | Native Claude Code format, lazy-loaded | v0.1.0 | `[confirmed]` |
| JSONL for event logging | Append-only, crash-safe, easy to parse | v0.1.0 | `[confirmed]` |
| SQLite for episodic + graph memory | Single-file, WAL mode, FTS5, zero setup | v0.3.0 | `[confirmed]` |
| 350ms daemon timeout | Non-blocking; falls back to local heuristics | v0.2.0 | `[experimental]` |
| haiku for read-only agents | Cost-optimized for cartographer/auditor | v0.2.0 | `[confirmed]` |
| sonnet for reasoning agents | Balance of capability and cost | v0.2.0 | `[confirmed]` |
| Proof packet output style | Structured evidence > narrative prose | v0.1.0 | `[confirmed]` |
| 12-stage control loop | Comprehensive coverage; skip-routing avoids overhead | v0.3.0 | `[confirmed]` |
| Dynamic topology (S/V/H) | Trivial tasks skip 8 stages; security gets double verification | v0.3.0 | `[confirmed]` |
| Statistical heuristic evolution | No LLM in the loop; OPENSAGE-style mutation | v0.3.0 | `[experimental]` |
| 5-guard pipeline | Composable; each guard independently testable | v0.3.0 | `[confirmed]` |
| Hash chain + Merkle + ed25519 | Three-layer integrity: tamper detection + compact verification + non-repudiation | v0.3.0 | `[confirmed]` |
| TF-IDF cosine for MASC | Lightweight, no model dependency, real-time | v0.3.0 | `[confirmed]` |
| Fatigue profiler signals | Behavioural indicators correlated with poor decisions | v0.3.0 | `[experimental]` |
| Process-based sandbox | Simple, cross-platform, no WASM dependency | v0.3.0 | `[confirmed]` |
| Content-hashed tool cache | Avoids re-generating identical forge scripts | v0.3.0 | `[confirmed]` |
