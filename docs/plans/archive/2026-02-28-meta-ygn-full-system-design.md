# Meta-YGN Full System Design

**Date**: 2026-02-28
**Version**: v1.0
**Status**: Approved
**Author**: Yann Abadie + Claude Opus 4.6

---

## 1. Executive Thesis

Meta-YGN (Aletheia-Nexus) is a **local-first metacognitive runtime** for AI coding agents. It is not a linter, not a prompt wrapper, not a chatbot plugin. It is an **independent Prefrontal Cortex** that sits between the LLM and the developer, ensuring that every action is verified, every claim is evidence-backed, every risk is classified, and every human is protected from over-trusting AI output.

The system surpasses:
- **Simple prompting** — by maintaining persistent state, episodic memory, and learned heuristics across sessions
- **Simple self-reflection** — by using factored verification (independent checkers), not self-assessment
- **Simple MCP servers** — by keeping the hot path local and deterministic, with MCP only at the edge
- **Simple tool wrappers** — by implementing a 12-stage metacognitive control loop with budget awareness
- **Simple orchestration** — by adding stagnation prediction, inverse metacognition (human fatigue), and autopoietic tool creation

### Core differentiators
1. **Context Pruning (Time-Travel Cognition)** — reverse proxy that amputates failed reasoning from LLM context
2. **Shadow WASM Sandboxing** — speculative code execution in ephemeral Wasmtime containers before presenting results
3. **Inverse Metacognition** — profiling human fatigue and adapting AI behavior to protect the codebase
4. **Autopoietic Tool Forge** — the system creates its own WASM-compiled tools at runtime
5. **Evidence Packs** — hash-chained, ed25519-signed, Merkle-tree-rooted audit trails (adapted from Y-GN)
6. **MASC Anomaly Detection** — unsupervised step-level anomaly detection without external models (adapted from NEXUS)
7. **Skill Crystallizer** — auto-detects repeated tool patterns and compiles reusable skill templates (adapted from NEXUS)
8. **KERNEL Integrity** — immutable alignment rules verified by SHA-256 at boot (adapted from NEXUS)
9. **Glass-Box TUI** — real-time cognitive telemetry dashboard (`aletheia top`)
10. **Neuro-Symbolic Audit** — Z3 theorem prover for mathematical verification of state machines and security rules

---

## 2. Architecture Overview

### Three-Tier Symbiotic Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     TIER 1: RUST DAEMON                         │
│                   "aletheiad" — The Brain                        │
│                                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │   CORE   │ │  MEMORY  │ │ SANDBOX  │ │    VERIFIERS     │   │
│  │ 12-stage │ │ 3-tier   │ │ Wasmtime │ │ Code, Logic,     │   │
│  │ control  │ │ Hot/Warm │ │ + Forge  │ │ Factual, Policy  │   │
│  │ loop     │ │ /Cold    │ │ + Hypo-  │ │ + Guard Pipeline │   │
│  │          │ │ + FTS5   │ │   thesis │ │ + Evidence Pack  │   │
│  │ + KERNEL │ │ + sqlite │ │          │ │ + MASC Monitor   │   │
│  │ integrity│ │ -vec     │ │          │ │ + Z3 (optional)  │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────────┬─────────┘   │
│       │             │            │                 │             │
│  ┌────┴─────────────┴────────────┴─────────────────┴─────────┐  │
│  │              DAEMON (axum + tokio)                          │  │
│  │  HTTP API (:0 dynamic) + Context Pruning Proxy (:11434)    │  │
│  │  + Human Fatigue Profiler + Glass-Box Telemetry            │  │
│  └────────────────────────┬───────────────────────────────────┘  │
│                           │                                      │
│  ┌────────────────────────┴───────────────────────────────────┐  │
│  │              CLI (clap)                                     │  │
│  │  start · stop · status · recall · audit · sandbox · forge  │  │
│  │  amnesia · top (TUI ratatui)                               │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │ HTTP localhost / IPC
┌──────────────────────────────┴──────────────────────────────────┐
│                   TIER 2: TYPESCRIPT PLUGIN                      │
│               "Claude Code Plugin Shell"                         │
│                                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │  HOOKS   │ │  SKILLS  │ │  AGENTS  │ │  OUTPUT STYLES   │   │
│  │ 8 events │ │ 8+ meta- │ │ 6+ roles │ │ Proof packet     │   │
│  │ Bun 8ms  │ │ cognitive│ │ sonnet/  │ │ Evidence tags     │   │
│  │ startup  │ │ workflows│ │ haiku    │ │                   │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  MCP-BRIDGE (optional, thin, 5 tools max)                │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────┴──────────────────────────────────┐
│                   TIER 3: EDGE ADAPTERS                          │
│                                                                  │
│  MCP (interop) · A2A (multi-agent, future) · LSP (optional)    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Monorepo Structure

```
Meta-YGN/
├── Cargo.toml                        # Workspace: members = ["crates/*"]
├── pnpm-workspace.yaml               # packages: ["packages/*"]
├── turbo.json                        # TS build orchestration
├── justfile                          # Cross-language task runner
├── rust-toolchain.toml               # Edition 2024, stable channel
│
│
│   ╔══════════════════════════════════════════════════════════╗
│   ║                   RUST CRATES                            ║
│   ╚══════════════════════════════════════════════════════════╝
│
├── crates/
│   │
│   ├── shared/                        # SHARED TYPES & PROTOCOL
│   │   └── src/
│   │       ├── state.rs               # TaskSignature, GoalState, BeliefState,
│   │       │                          # CompetenceEstimate, UncertaintyBreakdown,
│   │       │                          # BudgetState, MetacognitiveStateVector (5D)
│   │       ├── events.rs              # Event types for logging and evidence
│   │       ├── protocol.rs            # Hook payloads, decisions, IPC messages
│   │       ├── posix.rs               # Exit codes: 0=valid, 1=flaw, 137=lock-in
│   │       └── kernel.rs              # KERNEL: immutable alignment rules
│   │                                  # SHA-256 hash verified at boot
│   │                                  # Adapted from NEXUS KERNEL.py
│   │
│   ├── core/                          # METACOGNITIVE CONTROL PLANE
│   │   └── src/
│   │       │
│   │       ├── state/                 # Full metacognitive state model
│   │       │   ├── task.rs            # TaskSignature (type, domain, complexity)
│   │       │   ├── goal.rs            # GoalState + BeliefState
│   │       │   ├── competence.rs      # CompetenceEstimate (per-domain)
│   │       │   ├── uncertainty.rs     # Epistemic vs aleatoric decomposition
│   │       │   ├── budget.rs          # Tokens, latency, cost, risk budget
│   │       │   └── vector.rs          # 5D compact vector (30 tokens)
│   │       │                          #   confidence, coherence, grounding,
│   │       │                          #   complexity, progress
│   │       │
│   │       ├── loop/                  # 12-STAGE RUNTIME CONTROL LOOP
│   │       │   ├── mod.rs             # Loop orchestrator
│   │       │   ├── classify.rs        # 1. Task type classification
│   │       │   ├── assess.rs          # 2. Difficulty estimation (entropy)
│   │       │   ├── competence.rs      # 3. Competence self-assessment (MUSE-inspired)
│   │       │   ├── tool_need.rs       # 4. Tool necessity estimation (SMART)
│   │       │   ├── budget.rs          # 5. Budget allocation (TALE-inspired)
│   │       │   ├── strategy.rs        # 6. Strategy selection (8 strategies)
│   │       │   ├── act.rs             # 7. Execute or delegate
│   │       │   ├── verify.rs          # 8. Factored verification routing
│   │       │   ├── calibrate.rs       # 9. Confidence calibration (3 methods)
│   │       │   ├── compact.rs         # 10. Memory compaction
│   │       │   ├── decide.rs          # 11. Stop/revise/abstain/escalate
│   │       │   │                      #     + Stagnation prediction
│   │       │   │                      #     (NEXUS seuils 0.15/0.25/0.40)
│   │       │   └── learn.rs           # 12. Learn from trace → episodic memory
│   │       │
│   │       ├── strategies/            # 8 COGNITIVE STRATEGIES
│   │       │   ├── step_by_step.rs    # CoT classique
│   │       │   ├── tree_explore.rs    # ToT exploration arborescente
│   │       │   ├── verify_first.rs    # CoVe verification d'abord
│   │       │   ├── divide_conquer.rs  # Decomposition hierarchique
│   │       │   ├── analogical.rs      # Raisonnement par analogie
│   │       │   ├── adversarial.rs     # Auto-debat dialectique
│   │       │   ├── rapid.rs           # System 1 rapide (taches triviales)
│   │       │   └── iterative.rs       # Self-Refine iteratif
│   │       │
│   │       ├── monitor.rs             # MASC ANOMALY DETECTION
│   │       │                          # TF-IDF cosine similarity, no external models
│   │       │                          # Adapted from NEXUS metacognitive_monitor.py
│   │       │
│   │       └── meta_meta.rs           # LAYER 0: META-METACOGNITION
│   │                                  # Optimizes own thresholds, strategies,
│   │                                  # and prompts based on performance data
│   │
│   ├── daemon/                        # ALETHEIAD — THE BRAIN
│   │   └── src/
│   │       ├── main.rs                # Entrypoint: daemon + proxy startup
│   │       │
│   │       ├── api/                   # HTTP API (axum, bind :0 dynamic)
│   │       │   ├── hooks.rs           # /hooks/{event} — respond to TS hooks
│   │       │   ├── memory.rs          # /memory/recall, /memory/stats
│   │       │   ├── health.rs          # /health, /status, /metrics
│   │       │   └── admin.rs           # /admin/config, /admin/kernel
│   │       │                          # Inspired by Y-GN gateway.rs
│   │       │
│   │       ├── proxy/                 # CONTEXT PRUNING REVERSE PROXY
│   │       │   ├── intercept.rs       # MITM transparent on localhost:11434
│   │       │   ├── prune.rs           # Detect 3+ consecutive errors →
│   │       │   │                      # amputate failed reasoning from payload
│   │       │   ├── inject.rs          # Inject compressed system prompt:
│   │       │   │                      # "[ALETHEIA: Context pruned. Start fresh.]"
│   │       │   └── compiler.rs        # Budget-aware context compilation
│   │       │                          # Session → EventLog → Processors →
│   │       │                          # WorkingContext
│   │       │                          # Adapted from Y-GN context_compiler/
│   │       │
│   │       ├── profiler/              # INVERSE METACOGNITION
│   │       │   ├── fatigue.rs         # Human fatigue detection
│   │       │   │                      # (keystroke freq, Ctrl+Z, git resets,
│   │       │   │                      #  time-of-day, prompt brevity/aggression)
│   │       │   ├── telemetry.rs       # OS-level behavioral telemetry
│   │       │   └── friction.rs        # High-Friction mode activation
│   │       │                          # (refuse major refactors, require tests)
│   │       │
│   │       └── config.rs              # Daemon configuration
│   │
│   ├── memory/                        # EPISODIC MEMORY ENGINE
│   │   └── src/
│   │       ├── store.rs               # SQLite WAL + rusqlite (bundled)
│   │       │                          # Inspired by Y-GN sqlite_memory.rs
│   │       ├── fts.rs                 # FTS5 + BM25 hybrid text search
│   │       │                          # Adapted from Y-GN sqlite_memory.rs
│   │       ├── embeddings.rs          # sqlite-vec vector similarity search
│   │       ├── tiered.rs             # 3-TIER MEMORY (Hot/Warm/Cold)
│   │       │                          # Hot: TTL-based recent interactions
│   │       │                          # Warm: temporal index + hierarchical tags
│   │       │                          # Cold: Knowledge Graph + embeddings
│   │       │                          # Adapted from Y-GN tiered_memory.py
│   │       ├── trauma.rs              # ERROR PATTERN INDEX
│   │       │                          # Abstract heuristics from failures
│   │       │                          # (1-line rules, not raw transcripts)
│   │       ├── crystallizer.rs        # SKILL CRYSTALLIZER
│   │       │                          # Auto-detect repeated tool patterns →
│   │       │                          # compile to parameterized skill templates
│   │       │                          # Adapted from NEXUS skill_crystallizer.py
│   │       ├── behavior.rs            # Behavior registry (metacognitive reuse)
│   │       ├── tool_reliability.rs    # Tool success/latency/quality history
│   │       └── calibration.rs         # Calibration history
│   │                                  # (ground truth vs predictions)
│   │
│   ├── sandbox/                       # WASM SHADOW SANDBOXING
│   │   └── src/
│   │       ├── runtime.rs             # Wasmtime + fuel metering + memory limits
│   │       │                          # Pre-compiled AOT for 2-5ms cold start
│   │       ├── hypothesis.rs          # Speculative execution
│   │       │                          # ("reve algorithmique" — test code before
│   │       │                          #  presenting to user)
│   │       ├── forge.rs               # AUTOPOIETIC TOOL CREATION
│   │       │                          # AI writes Rust/WASM extension →
│   │       │                          # daemon compiles → hot-reloads
│   │       │                          # Inspired by NEXUS evolution.py
│   │       │                          # and Y-GN self-healing scaffold
│   │       └── hot_reload.rs          # Dynamic extension loading
│   │
│   ├── verifiers/                     # VERIFICATION HIERARCHY
│   │   └── src/
│   │       ├── router.rs              # Dynamic routing to appropriate verifier
│   │       │                          # based on claim type + risk level
│   │       ├── evidence.rs            # EVIDENCE PACK
│   │       │                          # SHA-256 hash chain + ed25519 signing +
│   │       │                          # Merkle tree root
│   │       │                          # Adapted from Y-GN evidence.py
│   │       ├── guard_pipeline.rs      # COMPOSABLE GUARD PIPELINE
│   │       │                          # Scoring 0-100 + regex fast-path +
│   │       │                          # pattern classification
│   │       │                          # Adapted from Y-GN guard.py
│   │       ├── code.rs                # Compilation, lint, type check, test
│   │       ├── logic.rs               # Z3 NEURO-SYMBOLIC AUDIT (optional)
│   │       │                          # Convert deductions to first-order logic,
│   │       │                          # detect contradictions in 2ms
│   │       ├── factual.rs             # CoVe FACTORED VERIFICATION
│   │       │                          # Draft → verify questions →
│   │       │                          # independent answers → reconcile
│   │       ├── consistency.rs         # Cross-step coherence checking
│   │       ├── policy.rs              # Safety, scope, approval gates
│   │       │                          # Inspired by Y-GN policy.rs
│   │       └── confidence.rs          # Confidence auditor (3 methods:
│   │                                  # verbalized, consistency, multi-perspective)
│   │
│   ├── cli/                           # ALETHEIA-CLI
│   │   └── src/
│   │       ├── main.rs                # clap entry point
│   │       ├── commands/
│   │       │   ├── start.rs           # Start daemon (spawn + write port file)
│   │       │   ├── stop.rs            # Graceful shutdown (SIGTERM / TerminateProcess)
│   │       │   ├── status.rs          # Health + metrics + KERNEL integrity
│   │       │   ├── audit.rs           # Metacognitive audit of current session
│   │       │   ├── recall.rs          # Query episodic memory (text + vector)
│   │       │   ├── sandbox.rs         # `aletheia sandbox exec --code "..."`
│   │       │   ├── forge.rs           # `aletheia forge` (create WASM tools)
│   │       │   ├── amnesia.rs         # Manual context pruning trigger
│   │       │   └── top.rs             # TUI launch
│   │       │
│   │       └── tui/                   # GLASS-BOX TUI (ratatui)
│   │           ├── app.rs             # Main TUI app loop
│   │           ├── entropy_gauge.rs   # Real-time AI doubt gauge
│   │           ├── wasm_tree.rs       # WASM subconscious execution tree
│   │           ├── linter_events.rs   # Linter interrupt stream
│   │           ├── fatigue_meter.rs   # Human fatigue score
│   │           ├── memory_stats.rs    # Hot/Warm/Cold memory breakdown
│   │           └── budget_tracker.rs  # Token + cost burn rate
│   │
│   └── mcp-bridge/                    # THIN MCP FACADE (rmcp)
│       └── src/
│           └── main.rs                # 5 high-value tools maximum
│                                      # metacog_plan, metacog_verify,
│                                      # metacog_calibrate, metacog_reflect,
│                                      # metacog_tools
│
│
│   ╔══════════════════════════════════════════════════════════╗
│   ║                 TYPESCRIPT PACKAGES                      ║
│   ╚══════════════════════════════════════════════════════════╝
│
├── packages/
│   │
│   ├── hooks/                         # CLAUDE CODE HOOKS (Bun, 8ms startup)
│   │   └── src/
│   │       ├── session-start.ts       # Detect stack, init Aletheia profile
│   │       ├── user-prompt-submit.ts  # Risk classification → daemon
│   │       ├── pre-tool-use.ts        # Security gates → daemon (deny/ask/allow)
│   │       ├── post-tool-use.ts       # Verification signals → daemon
│   │       ├── post-tool-use-failure.ts # Error diagnosis → daemon
│   │       ├── stop.ts               # Proof packet enforcement → daemon
│   │       ├── pre-compact.ts         # Structured compaction → daemon
│   │       └── lib/
│   │           ├── daemon-client.ts   # HTTP client to aletheiad
│   │           ├── types.ts           # Typed hook payloads
│   │           │                      # (using @constellos/claude-code-kit)
│   │           └── fallback.ts        # Local heuristics if daemon is down
│   │                                  # (mirrors current Python logic)
│   │
│   ├── plugin/                        # CLAUDE CODE PLUGIN SHELL
│   │   ├── .claude-plugin/plugin.json # Plugin manifest
│   │   ├── hooks/hooks.json           # Points to bun run packages/hooks/...
│   │   ├── settings.json              # Default agent: aletheia-main
│   │   │
│   │   ├── skills/                    # 8+ METACOGNITIVE SKILLS
│   │   │   ├── metacog-preflight/     # Risk classification + strategy selection
│   │   │   ├── metacog-proof/         # Evidence packet building
│   │   │   ├── metacog-challenge/     # Assumption pressure-testing
│   │   │   ├── metacog-threat-model/  # Security + trust boundary review
│   │   │   ├── metacog-compact/       # Session compaction for handoff
│   │   │   ├── metacog-bench/         # Quality + calibration evaluation
│   │   │   ├── metacog-tool-audit/    # Tool necessity assessment
│   │   │   └── metacog-escalate/      # Structured escalation protocol
│   │   │
│   │   ├── agents/                    # 6+ SPECIALIZED AGENTS
│   │   │   ├── aletheia-main.md       # Default: sonnet, verification-first
│   │   │   ├── skeptic.md             # Challenge assumptions (plan-mode)
│   │   │   ├── verifier.md            # Independent verification
│   │   │   ├── researcher.md          # Web research (WebSearch/WebFetch)
│   │   │   ├── repo-cartographer.md   # Repo mapping (haiku)
│   │   │   └── cost-auditor.md        # Token/context audit (haiku)
│   │   │
│   │   └── output-styles/
│   │       └── aletheia-proof.md      # Proof packet: Goal, Changes, Evidence,
│   │                                  # Uncertainty, Next step
│   │
│   └── shared/                        # ZOD TYPES (mirrors Rust shared/)
│       └── src/
│           ├── hook-payloads.ts
│           ├── decisions.ts
│           └── state.ts
│
│
│   ╔══════════════════════════════════════════════════════════╗
│   ║                   PYTHON EVALUATION                      ║
│   ╚══════════════════════════════════════════════════════════╝
│
├── eval/
│   ├── pyproject.toml
│   ├── benchmarks/                    # MetaCog-Bench
│   │   ├── scenarios/                 # 5 families: reasoning, code, tool-use,
│   │   │                              # long-horizon, human-AI collaboration
│   │   ├── baselines/                 # With/without Aletheia comparisons
│   │   └── metrics/                   # ECE, hallucination rate, token efficiency,
│   │                                  # calibration accuracy, time-to-quality
│   ├── replay/                        # Trace replay from evidence packs
│   ├── roi/                           # ROI dashboard
│   │                                  # (tokens saved, errors prevented, time gained)
│   └── analysis/                      # Jupyter notebooks for analysis
│
│
│   ╔══════════════════════════════════════════════════════════╗
│   ║                EXPERIMENTAL (GATED)                      ║
│   ╚══════════════════════════════════════════════════════════╝
│
├── experimental/
│   ├── adversarial-local/             # Micro-SLM watchdog
│   │                                  # (candle-core or llama.cpp via Rust)
│   │                                  # Local paranoid model as "chien de garde"
│   ├── swarm-metacog/                 # Multi-agent collective stop signals
│   │                                  # Bio-inspired (honeybee democracy)
│   ├── domain-adaptation/             # Per-repo metacognition tuning
│   │                                  # (CI present? coverage? types? linter?)
│   ├── emotional-signals/             # Frustration/curiosity as strategy regulators
│   │                                  # (EG-MRSI framework)
│   ├── self-improving-skills/         # Skills that rewrite themselves (Layer 0)
│   └── metagraph-rag/                 # AST-based codebase intelligence
│                                      # (adapted from NEXUS MetagraphRAG)
│
│
│   ╔══════════════════════════════════════════════════════════╗
│   ║                  DOCUMENTATION                           ║
│   ╚══════════════════════════════════════════════════════════╝
│
├── docs/
│   ├── plans/                         # Design docs (this file)
│   ├── architecture.md                # Living architecture doc
│   ├── threat-model.md                # Security boundaries
│   ├── evidence-tiers.md              # [confirmed]/[experimental]/[unverified]
│   ├── context-economics.md           # Token budget strategy
│   ├── benchmark-integrity.md         # Anti-theater evaluation methodology
│   └── daemon-contract.md             # Daemon API request/response schema
│
├── memory-bank/                       # Persistent project context
├── CLAUDE.md                          # Repo operating contract
├── README.md
├── CHANGELOG.md
└── LICENSE                            # MIT
```

---

## 4. Rust Daemon Design

### 4.1 Crate dependency graph

```
shared ─────────────────────────────────────┐
  │                                          │
  ├──→ core (depends on shared)              │
  │      │                                   │
  ├──→ memory (depends on shared)            │
  │      │                                   │
  ├──→ sandbox (depends on shared)           │
  │      │                                   │
  ├──→ verifiers (depends on shared, memory) │
  │      │                                   │
  ├──→ daemon (depends on ALL above)         │
  │                                          │
  ├──→ cli (depends on shared, daemon-client)│
  │                                          │
  └──→ mcp-bridge (depends on shared, core)  │
```

### 4.2 Key dependencies

```toml
# crates/daemon/Cargo.toml
[dependencies]
axum = { version = "0.8", features = ["json"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace"] }
hyper = { version = "1", features = ["http1", "client"] }   # for reverse proxy
rusqlite = { version = "0.38", features = ["bundled"] }
tokio-rusqlite = "0.6"
sqlite-vec = "0.1.7-alpha.2"
wasmtime = "41"
wasmtime-wasi = "41"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
ed25519-dalek = "2"                                          # evidence signing
sha2 = "0.10"                                                # hash chains
ratatui = "0.29"                                             # TUI
crossterm = "0.28"                                           # terminal backend
anyhow = "1"
thiserror = "2"
rmcp = "0.17"                                                # MCP bridge
interprocess = { version = "2.4", features = ["tokio"], optional = true }

# Optional
z3 = { version = "0.12", optional = true }                   # neuro-symbolic
```

### 4.3 Runtime control loop (12 stages)

```
     ┌──────────────────────────────────────────────────────┐
     │                    INCOMING HOOK                      │
     └──────────────────────┬───────────────────────────────┘
                            ▼
              ┌─────────────────────────┐
              │  1. CLASSIFY task type   │  bugfix/feature/refactor/
              │                         │  architecture/security/release
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  2. ASSESS difficulty    │  entropy estimation
              │                         │  (simple → rapid path)
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  3. COMPETENCE estimate  │  per-domain self-assessment
              │                         │  (MUSE-inspired)
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  4. TOOL NECESSITY      │  can this be answered without
              │                         │  tools? (SMART: -24% tool use)
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  5. BUDGET allocation    │  tokens, latency, cost, risk
              │                         │  (TALE: -67% token cost)
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  6. STRATEGY selection   │  pick from 8 strategies
              │                         │  based on task + competence
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  7. ACT or delegate     │  execute, or fork to agent
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  8. VERIFY (factored)   │  route to appropriate verifier
              │                         │  code/logic/factual/policy
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │  9. CALIBRATE confidence│  verbalized + consistency +
              │                         │  multi-perspective (3 methods)
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │ 10. COMPACT memory      │  evict dead ends, promote
              │                         │  verified facts
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │ 11. DECIDE next action  │  continue / revise / abstain /
              │     + stagnation check  │  escalate / stop
              │     (NEXUS: 0.15/0.25)  │  (calibrated thresholds)
              └────────────┬────────────┘
                           ▼
              ┌─────────────────────────┐
              │ 12. LEARN from trace    │  store in episodic memory,
              │                         │  update behavior registry,
              │                         │  crystallize skills
              └─────────────────────────┘
```

### 4.4 Context pruning reverse proxy

The daemon runs a transparent HTTP reverse proxy on `localhost:11434`. When `BASE_URL` of Claude Code (or Cursor) is pointed at this port:

1. **Intercept** every outgoing payload to `api.anthropic.com`
2. **Detect** if the last 3+ assistant messages contain errors (pattern matching)
3. **Amputate** the failed reasoning tokens from the messages array
4. **Inject** a compressed system prompt: `[ALETHEIA: Context pruned due to reasoning lock-in. {n} failed attempts removed. Start fresh with new approach.]`
5. **Forward** the cleaned payload to the real API
6. **Result**: The LLM regains lucidity, API cost drops significantly

### 4.5 Evidence packs (adapted from Y-GN)

Every significant metacognitive decision produces an evidence entry:

```rust
struct EvidenceEntry {
    id: Uuid,
    timestamp: DateTime<Utc>,
    event_type: EvidenceType,    // decision, verification, escalation, etc.
    payload: serde_json::Value,  // the actual evidence
    prev_hash: [u8; 32],        // SHA-256 of previous entry (hash chain)
    signature: Option<[u8; 64]>, // ed25519 signature
}

struct EvidencePack {
    session_id: String,
    entries: Vec<EvidenceEntry>,
    merkle_root: [u8; 32],      // RFC 6962 Merkle tree root
    signer_public_key: [u8; 32],
}
```

### 4.6 KERNEL integrity (adapted from NEXUS)

```rust
/// Immutable alignment rules verified at boot.
/// If KERNEL hash doesn't match, daemon refuses to start.
pub struct Kernel {
    rules: Vec<AlignmentRule>,
    hash: [u8; 32],  // SHA-256 of serialized rules
}

pub enum AlignmentRule {
    /// Never execute destructive commands without human approval
    RequireApprovalForDestructive,
    /// Never expose secrets in responses or logs
    NeverExposeSecrets,
    /// Always produce evidence for strong claims
    EvidenceRequired,
    /// Escalate when confidence < threshold
    EscalateOnLowConfidence { threshold: f32 },
    /// Context pruning must preserve original user intent
    PreserveUserIntent,
}
```

### 4.7 Human fatigue profiler

The daemon monitors behavioral signals (opt-in, local-only):

| Signal | Detection method | Weight |
|--------|-----------------|--------|
| Ctrl+Z frequency | OS-level key event monitoring | 0.25 |
| Git resets/reverts | Watch `.git/` for reset operations | 0.20 |
| Prompt brevity | Short, aggressive prompts ("just fix it") | 0.20 |
| Time of day | Late night (23:00-05:00) | 0.15 |
| Error loop | Same error repeated 3+ times | 0.20 |

When fatigue score > 0.7: inject `High-Friction` mode that refuses major refactors and requires unit tests before any destructive action.

---

## 5. TypeScript Plugin Layer Design

### 5.1 Hook execution flow

```
Claude Code Event
       │
       ▼
  hooks.json (bun run packages/hooks/src/{event}.ts)
       │
       ▼
  TypeScript Hook
       │
       ├── Try: HTTP POST to daemon (http://127.0.0.1:{port}/hooks/{event})
       │         timeout: 350ms
       │
       ├── If daemon responds: return daemon's decision
       │
       └── If daemon down: execute local fallback heuristics
                           (mirrors current Python patterns)
```

### 5.2 Typed hook payloads

Using `@constellos/claude-code-kit` for type-safe hook payloads:

```typescript
import type { PreToolUseHookInput } from '@constellos/claude-code-kit';

export async function handlePreToolUse(input: PreToolUseHookInput) {
  const decision = await daemonClient.post('/hooks/pre-tool-use', input);
  if (decision) return decision;
  return fallback.preToolUse(input);  // local patterns if daemon is down
}
```

### 5.3 Why Bun, not Python

| Criterion | Python (current) | Bun (proposed) |
|-----------|-----------------|----------------|
| Cold start | 30-50ms | 8-15ms |
| Type safety | None | Full TypeScript |
| Claude Code integration | Custom JSON parsing | `@constellos/claude-code-kit` typed payloads |
| Ecosystem | stdlib only | npm ecosystem (Zod, etc.) |
| Hook payload types | Manual extraction | Discriminated unions by tool_name |

---

## 6. Tech Stack Summary

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| **Daemon core** | Rust (Edition 2024) | stable 1.85+ | Memory safety, WASM compilation, <100ms responses |
| **Async runtime** | tokio | 1.x | Industry standard, work-stealing scheduler |
| **HTTP framework** | axum | 0.8 | Lightest weight, built by tokio team |
| **Reverse proxy** | hyper | 1.x | Low-level HTTP for MITM proxy |
| **Database** | SQLite (rusqlite bundled) | 0.38 | Zero system deps, WAL mode, FTS5 |
| **Vector search** | sqlite-vec | 0.1.7 | Embeddings inside SQLite |
| **WASM sandbox** | wasmtime | 41 | Fuel metering, capability-based security |
| **CLI framework** | clap | 4.5 | Derive macros, shell completions |
| **TUI** | ratatui + crossterm | 0.29 | Terminal dashboard |
| **Signing** | ed25519-dalek | 2.x | Evidence pack signatures |
| **Hashing** | sha2 | 0.10 | Hash chains, KERNEL integrity |
| **MCP** | rmcp | 0.17 | Official Rust MCP SDK |
| **Theorem prover** | z3 (optional) | 0.12 | Neuro-symbolic verification |
| **Hook runtime** | Bun | 1.x | 8ms startup, native TS |
| **Hook types** | @constellos/claude-code-kit | latest | Typed hook payloads |
| **MCP SDK (TS)** | @modelcontextprotocol/sdk | 1.27+ | MCP server if needed |
| **Schema validation** | Zod v4 | 4.x | Required by MCP SDK |
| **Monorepo (TS)** | pnpm + Turborepo | latest | Fast, disk-efficient |
| **Task runner** | just | latest | Cross-language, simple |
| **Eval** | Python 3.11+ (pytest) | latest | Benchmarks, analysis |
| **CI** | GitHub Actions | N/A | Rust + TS + Python |
| **Distribution** | cargo-dist | latest | Cross-platform releases |

---

## 7. Security Model

### Multi-wall defense (inspired by Y-GN)

```
Layer 1: KERNEL Integrity
  │  SHA-256 verified immutable rules at boot
  │
Layer 2: WASM Sandbox
  │  Fuel-metered, capability-based, no host access by default
  │
Layer 3: Guard Pipeline
  │  Composable guards with 0-100 scoring
  │  Destructive → DENY, High-risk → ASK, Default → ALLOW
  │
Layer 4: Evidence Trail
  │  Hash-chained, ed25519-signed, Merkle-tree audit log
  │
Layer 5: Human Fatigue Gate
  │  High-friction mode when fatigue detected
  │
Layer 6: Context Pruning Safety
     Never prune user messages, only failed AI reasoning
```

### Trust boundaries

| Boundary | Protection |
|----------|-----------|
| Daemon ↔ LLM API | Context pruning proxy, payload inspection |
| Daemon ↔ Hooks | HTTP localhost only, no auth (same machine) |
| Daemon ↔ MCP | Trust boundary warning, output treated as untrusted |
| Daemon ↔ WASM | Sandboxed, fuel-limited, no network/filesystem by default |
| Daemon ↔ Human | Fatigue detection (opt-in), approval gates |
| Daemon ↔ Secrets | Never in logs, never in LLM context, redacted by guard pipeline |

---

## 8. Data Flow: Hook Lifecycle

```
User submits prompt
  │
  ▼
SessionStart hook → daemon: detect stack, load KERNEL
  │
  ▼
UserPromptSubmit hook → daemon: classify risk (12-stage loop stages 1-6)
  │                              → return: strategy, budget, proof plan
  ▼
Claude reasons and calls tools
  │
  ▼
PreToolUse hook → daemon: guard pipeline (deny/ask/allow)
  │                        → KERNEL integrity check
  │                        → human fatigue check
  ▼
Tool executes
  │
  ├── Success: PostToolUse hook → daemon: verification signals,
  │                                        evidence pack entry
  │
  └── Failure: PostToolUseFailure hook → daemon: error diagnosis,
                                                   stagnation detection,
                                                   trauma index update
  │
  ▼
Claude finishes response
  │
  ▼
Stop hook → daemon: proof packet enforcement,
                     evidence pack finalization,
                     learn from trace (stage 12)
  │
  ▼
PreCompact (if context limit) → daemon: structured compaction,
                                          3-tier memory eviction
  │
  ▼
SessionEnd hook → daemon: session summary,
                           evidence pack archive,
                           calibration history update
```

---

## 9. Performance Budget

| Operation | Target | Approach |
|-----------|--------|----------|
| Hook response (daemon up) | <50ms | axum + SQLite WAL + in-memory cache |
| Hook response (daemon down) | <15ms | Bun fallback heuristics |
| Context pruning | <100ms | Payload inspection + regex matching |
| WASM speculation | <50ms | Pre-compiled AOT, fuel-limited |
| Evidence pack entry | <5ms | Append-only SQLite + async signing |
| TUI refresh | 60fps | ratatui + crossterm |
| Daemon startup | <500ms | Static binary, SQLite pragmas |
| Memory recall (text) | <10ms | FTS5 + BM25 |
| Memory recall (vector) | <20ms | sqlite-vec KNN |

---

## 10. Evaluation Framework (MetaCog-Bench)

### 5 scenario families

| Family | What it measures | Baseline |
|--------|-----------------|----------|
| **Reasoning** | Hallucination rate, logical coherence | CoVe: -23% F1 |
| **Code** | Test pass rate, bug introduction rate | Reflexion: 91% pass@1 |
| **Tool-use** | Unnecessary tool calls, tool selection quality | SMART: -24% tool use |
| **Long-horizon** | Stagnation rate, context efficiency | TALE: -67% tokens |
| **Human-AI** | Over-reliance reduction, cognitive load | Literature: +7-8% success |

### 4 evaluation axes

1. **Quality**: accuracy, correctness, completeness
2. **Calibration**: ECE < 0.10, abstention appropriateness
3. **Efficiency**: tokens consumed, latency, cost
4. **Safety**: secrets protected, destructive actions gated, human fatigue detected

---

## 11. Phasing

### Phase 1: Foundation (weeks 1-3)
- `crates/shared/` — state types, protocol, KERNEL
- `crates/memory/` — SQLite store, FTS5, embeddings (adapted from Y-GN)
- `crates/daemon/` — axum API for hooks, health, memory
- `crates/cli/` — start, stop, status, recall
- `packages/hooks/` — migrate Python hooks to Bun/TypeScript
- `packages/plugin/` — restructured plugin shell
- **Milestone**: daemon responds to hooks, stores events, recalls memories

### Phase 2: Intelligence (weeks 4-6)
- `crates/core/` — 12-stage control loop, 8 strategies
- `crates/verifiers/` — guard pipeline, evidence packs, factored verification
- `crates/daemon/proxy/` — context pruning reverse proxy
- `crates/memory/` — 3-tier memory, trauma index, skill crystallizer
- **Milestone**: daemon actively classifies risk, routes verification, prunes context

### Phase 3: Sandbox + TUI (weeks 7-8)
- `crates/sandbox/` — Wasmtime runtime, hypothesis testing
- `crates/cli/tui/` — Glass-box cognitive dashboard
- `crates/daemon/profiler/` — human fatigue detection
- **Milestone**: speculative execution works, TUI shows real-time telemetry

### Phase 4: Evaluation + Hardening (weeks 9-10)
- `eval/` — MetaCog-Bench scenarios, replay infrastructure
- `crates/sandbox/forge.rs` — autopoietic tool creation
- `crates/core/meta_meta.rs` — Layer 0 self-optimization
- Security hardening, cross-platform testing
- **Milestone**: benchmarks prove measurable impact

### Phase 5: Distribution (weeks 11-12)
- `cargo-dist` release pipeline
- Claude Code marketplace packaging
- Documentation finalization
- **Milestone**: installable via marketplace or binary download

---

## 12. Heritage Matrix

### From Y-GN (Yggdrasil-Grid Nexus)

| Component | Y-GN Source | Meta-YGN Target | Adaptation |
|-----------|-------------|-----------------|------------|
| SQLite memory + FTS5 + BM25 | `ygn-core/sqlite_memory.rs` | `crates/memory/store.rs` + `fts.rs` | Direct port, add tiered architecture |
| Evidence Pack | `ygn-brain/evidence.py` | `crates/verifiers/evidence.rs` | Rewrite in Rust, same hash chain + Merkle + ed25519 |
| 3-Tier Memory | `ygn-brain/tiered_memory.py` | `crates/memory/tiered.rs` | Rust reimplementation with SQLite backend |
| Guard Pipeline | `ygn-brain/guard.py` | `crates/verifiers/guard_pipeline.rs` | Composable guards with scoring, pattern-first |
| Context Compiler | `ygn-brain/context_compiler/` | `crates/daemon/proxy/compiler.rs` | Budget-aware pipeline with artifact externalization |
| Tool Interrupt Handler | `ygn-brain/tool_interrupt/` | `packages/hooks/src/post-tool-use.ts` | Typed events, normalization, redaction |
| Gateway routes | `ygn-core/gateway.rs` | `crates/daemon/api/` | Same axum pattern: /health, /hooks, /memory |
| Skills topological sort | `ygn-core/skills.rs` | `crates/core/strategies/` | Dependency-ordered strategy execution |
| Refinement Harness | `ygn-brain/harness/` | `crates/verifiers/factual.rs` | Generate-verify-refine → CoVe pattern |

### From NEXUS NX-CG

| Component | NEXUS Source | Meta-YGN Target | Adaptation |
|-----------|-------------|-----------------|------------|
| KERNEL integrity | `KERNEL.py` | `crates/shared/kernel.rs` | Rust with SHA-256 boot verification |
| MASC Monitor | `metacognitive_monitor.py` | `crates/core/monitor.rs` | TF-IDF cosine, no external models |
| Skill Crystallizer | `skill_crystallizer.py` | `crates/memory/crystallizer.rs` | Auto-detect patterns → compile skills |
| Stagnation Predictor | `stagnation.py` | `crates/core/loop/decide.rs` | Calibrated thresholds 0.15/0.25/0.40 |
| Rust/Python Bridge | `rust/nexus_core/` + `native/` | Reference for PyO3 pattern | Transparent fallback pattern |
| Budget Tracker | `budget_tracker.py` | `crates/core/loop/budget.rs` | Token + cost with daily limits |
| `.claude/` structure | `.claude/{agents,commands,skills,hooks}/` | `packages/plugin/` | Mature Claude Code config |
| Agent Write+Verify | `.claude/agents/*.md` | `packages/plugin/agents/*.md` | Mandatory Edit → Read verification |
| MetagraphRAG | `core/metagraph/` | `experimental/metagraph-rag/` | AST codebase intelligence |
| Collaboration modes | `core/intelligence/swarm.py` | `crates/core/strategies/` | Task-based mode selection |

---

## 13. Key Design Decisions

| # | Decision | Rationale | Evidence Tier |
|---|----------|-----------|--------------|
| 1 | Rust Edition 2024 for daemon | Latest stable edition (Feb 2025). Memory safety, WASM compilation, no GC. Pattern proven by Cursor, Zed, Temporal. | `[confirmed]` |
| 2 | axum 0.8 + tokio for HTTP | Lightest footprint, built by tokio team, graceful shutdown built-in | `[confirmed]` |
| 3 | rusqlite bundled + WAL mode | Zero system deps, readers never block writers, append-only I/O | `[confirmed]` |
| 4 | Bun for TypeScript hooks | 8-15ms cold start vs 30-50ms Python. Native TS. Typed payloads. | `[confirmed]` |
| 5 | wasmtime 41 for WASM sandbox | Fuel metering, epoch interruption, component model, Bytecode Alliance | `[confirmed]` |
| 6 | Context pruning proxy on :11434 | Eliminates reasoning lock-in, reduces API cost 10x on loops | `[experimental]` |
| 7 | Human fatigue profiler | Novel inverse metacognition. Opt-in, local-only. Research-backed. | `[experimental]` |
| 8 | Evidence packs with hash chain + Merkle | Auditable, tamper-evident. Adapted from Y-GN's EU AI Act compliance. | `[confirmed]` |
| 9 | KERNEL immutability at boot | Prevents AI from modifying its own safety rules. Adapted from NEXUS. | `[confirmed]` |
| 10 | 12-stage control loop | Covers full metacognitive cycle. Based on CLAUDE-46-1 research + ChatGPT52Pro-1 System 3 architecture. | `[experimental]` |
| 11 | Skill crystallizer | Auto-learning from tool patterns. Novel for plugin context. Adapted from NEXUS. | `[experimental]` |
| 12 | Z3 optional | Neuro-symbolic audit for critical code paths. Too heavy for every call. | `[original-proposal]` |
| 13 | Autopoietic forge | System writes its own WASM tools. Highest risk, highest reward. | `[original-proposal]` |
| 14 | Adversarial local SLM | Local paranoid watchdog model. Experimental, requires GPU. | `[original-proposal]` |

---

## 14. Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Daemon adds latency to every tool call | High | 350ms timeout, local fallback in hooks, async where possible |
| Context pruning removes useful context | High | Never prune user messages, only AI reasoning. Preserve summaries. |
| WASM sandbox too slow for hot path | Medium | Pre-compiled AOT, fuel limits, only for speculative execution |
| Human fatigue profiler privacy concerns | Medium | Opt-in only, local-only, no data leaves machine |
| Cross-platform complexity (Win/Mac/Linux) | Medium | `interprocess` crate for IPC, `cargo-dist` for builds, CI matrix |
| Monorepo build complexity (Rust+TS+Python) | Medium | `just` as unified task runner, clear Cargo/pnpm workspace separation |
| Autopoietic forge creates dangerous tools | High | KERNEL integrity check, WASM sandbox, human approval gate |
| SQLite contention under load | Low | WAL mode, `tokio_rusqlite` background thread, connection pooling |

---

## 15. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Hallucination reduction | -40% | MetaCog-Bench reasoning scenarios |
| Token efficiency | -50% | Before/after comparison on same tasks |
| Tool overuse reduction | -24% | SMART-inspired necessity gating |
| Calibration (ECE) | < 0.10 | Confidence vs actual accuracy plots |
| Stagnation detection | > 90% recall | Injected fault tests |
| Hook response time | < 50ms (p99) | Latency monitoring |
| Context pruning savings | > 5x on loops | API cost tracking |
| Human fatigue detection | > 80% precision | User study (opt-in) |
