# OpenSage Integration Plan (Codex-52)

Status: [original-proposal]
Date: 2026-02-28

This document replaces the placeholder plan and provides a concrete integration path
to adapt OpenSage ideas for MetaYGN while keeping the plugin shell thin and local-first.

## Sources (local HTML)
- OpenSage: `docs/papers/html/arxiv-2602.16891.html`
- Discovering Multiagent Learning Algorithms: `docs/papers/html/arxiv-2602.16928.html`
- AlphaEvolve: `docs/papers/html/ar5iv-2506.13131.html`
- PSRO (Unified Game-Theoretic Approach): `docs/papers/html/ar5iv-1711.00832.html`

## Repo facts (current state)
- Plugin shell: hooks + skills + agents + output styles, deterministic Python hooks. [confirmed]
- Runtime/daemon: Rust control loop with 12 stages, stores events in SQLite via `MemoryStore`. [confirmed]
- Memory: event log + FTS; no graph memory. [confirmed]
- Hook flow: `UserPromptSubmit` classification via local heuristics and optional daemon override. [confirmed]

## Paper facts we will implement
- OpenSage: runtime sub-agent creation with metadata; vertical and horizontal topologies. [confirmed]
- OpenSage: short-term graph (AgentRun, Event, RawToolResponse) + long-term graph (Neo4j)
  with embedding search and pattern match. [confirmed]
- OpenSage: embeddings for labels used for retrieval (text-embedding-3-small). [confirmed]
- AlphaEvolve: LLM-driven evolutionary code mutation with evaluator feedback loops. [confirmed]
- Discovering MAL Algorithms: AlphaEvolve applied to CFR/PSRO variants; code as genome. [confirmed]
- PSRO: population of strategies with meta-strategy solver to reduce overfit. [confirmed]

## Integration goals
1) Add graph memory (short-term + long-term) without breaking existing event log.
2) Keep embeddings local-only (no remote embedding APIs).
3) Add topology selection to the control loop and expose to hooks.
4) Add PSRO-like multi-path mode for high-risk tasks (opt-in).
5) Document the plan and keep advanced pieces behind flags.

## Non-goals (for this phase)
- Do not implement Neo4j. Use SQLite-based graph tables. [original-proposal]
- Do not enable automatic tool synthesis by default. [original-proposal]
- Do not add heavy MCP surfaces or remote services. [confirmed]

## Architecture decisions
- Graph memory in SQLite, not Neo4j, for local-first and simple packaging. [original-proposal]
- Embeddings computed locally via Rust or local HTTP service. [original-proposal]
- Memory agent is a dedicated Claude agent with restricted tool access. [original-proposal]
- Topology selector is a deterministic stage in the control loop. [confirmed]
- PSRO mode is gated by env flag and only for high-risk tasks. [experimental]

## Data model (SQLite graph memory)
Add new tables alongside existing `events`:

```
memory_nodes(
  id TEXT PRIMARY KEY,
  node_type TEXT NOT NULL,   -- AgentRun | Event | RawToolResponse | Entity | Concept | ...
  label TEXT NOT NULL,
  content TEXT NOT NULL,
  embedding BLOB,            -- Vec<f32> serialized
  created_at TEXT NOT NULL
)

memory_edges(
  source_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  relation TEXT NOT NULL,    -- calls | summarizes | verified_by | derived_from | ...
  weight REAL DEFAULT 1.0,
  created_at TEXT NOT NULL,
  PRIMARY KEY (source_id, target_id, relation)
)
```

Indexes:
- `memory_nodes(node_type, label)`
- FTS5 for `label` and `content`

Mapping from existing events:
- Each event remains in `events`.
- Create a lightweight `Event` node per event with pointers to raw payload.
- Large tool outputs are stored as `RawToolResponse` nodes referenced by edges.

## Embeddings (local-only)
Provide a `EmbeddingProvider` trait and two implementations:

1) `fastembed` (Rust, local)
   - Default model: `bge-small-en-v1.5`
   - Good CPU performance and small memory footprint
2) Local HTTP service (optional)
   - Use `sentence-transformers` locally
   - Endpoint: `POST /embed` with `{"text": "..."}`

Configuration:
```
ALETHEIA_EMBED_PROVIDER=fastembed|local-http
ALETHEIA_EMBED_MODEL=bge-small-en-v1.5
ALETHEIA_EMBED_URL=http://127.0.0.1:8090/embed
```

## Memory agent
Add a new Claude agent file: `agents/memory-agent.md` with:
- Tools: only daemon endpoints for graph queries and updates
- Responsibilities:
  - Decide short-term vs long-term targets
  - Short-term: read-only queries
  - Long-term: add/update/delete nodes and edges

Daemon API additions (new routes):
- `POST /memory/graph/query` (label + type + topN -> nodes + one-hop subgraph)
- `POST /memory/graph/grep` (pattern match)
- `POST /memory/graph/add-node`
- `POST /memory/graph/add-edge`
- `POST /memory/graph/update-node`

## Topology selector
Add `Topology` to `metaygn_shared::state`:
```
Single | Vertical | Horizontal
```

Add a stage after `strategy` that selects topology based on:
- Risk (high -> horizontal or vertical)
- Difficulty (high -> vertical)
- Task type (security -> horizontal + verifier)

Expose in hook output:
- `UserPromptSubmit` should include topology hint.
- `PreToolUse` may include topology context for auditing.

## PSRO mode (experimental, opt-in)
Goal: reduce single-path bias for high-risk tasks.
Mechanism:
- If `ALETHEIA_PSRO=1` and risk=high, spawn 2-3 parallel sub-agents.
- Each sub-agent produces a plan + patch proposal.
- An ensemble agent (new role) merges or selects best.

Safety:
- Default off.
- Only activated for high-risk tasks.

## Dynamic tool synthesis (stub only)
Create a schema and validator for tool metadata, but keep disabled.
This is a placeholder for OpenSage-like dynamic tool creation.

## Implementation plan

Phase 0 - Doc + scaffolding (this change)
- Add this plan and link from placeholder.

Phase 1 - Graph memory foundation
- Add tables + migration in `metaygn-memory`.
- Add basic CRUD for nodes/edges.
- Add FTS label search.

Phase 2 - Embedding provider
- Add `EmbeddingProvider` and local `fastembed` impl.
- Integrate embeddings into node creation.

Phase 3 - Daemon API
- Add `/memory/graph/*` routes.
- Add tests for CRUD and search.

Phase 4 - Memory agent
- Add `agents/memory-agent.md`.
- Add usage guidelines in docs.

Phase 5 - Topology selection
- Add `Topology` enum + stage.
- Surface in `UserPromptSubmit` output.

Phase 6 - PSRO mode (optional)
- Add `ensemble` agent + gating logic.

## Testing plan
- Unit tests for node/edge creation and retrieval.
- Integration tests for daemon endpoints.
- Regression: event log remains unchanged and searchable.

## Risks and mitigations
- Risk: Embedding latency on CPU. Mitigation: small model + cache.
- Risk: Graph memory bloat. Mitigation: TTL and pruning policy.
- Risk: Hook timeouts. Mitigation: keep hooks deterministic and short.
- Risk: PSRO token cost. Mitigation: opt-in only.

## Open questions
- Preferred local embedding model (bge-small vs e5-small)?
- Any minimum hardware constraints we should target?
- Should memory agent run automatically or only on demand?

## Evidence notes
- All facts from papers come from local HTML sources listed above. [confirmed]
- Schema and integration steps are design proposals for MetaYGN. [original-proposal]
