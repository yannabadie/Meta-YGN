# Architecture

## 5-Layer Protection Cascade

Every tool call from Claude Code passes through five layers evaluated in sequence. The strictest verdict wins.

```
Layer 1: AST Guard         Parses commands into syntax trees (tree-sitter).
                           Understands what "find / -delete" DOES, not just
                           that it contains "rm". Classifies effects: delete,
                           overwrite, root-targeting, tainted execution.

Layer 2: Smart Routing     Contextual risk scoring via semantic router.
                           "rm target/*.o" scores 20. "rm -rf /" scores 0.
                           Different responses for each based on confidence.

Layer 3: Sequence Monitor  DTMC-inspired multi-action pattern detection.
                           Catches multi-step attack chains:
                           clone -> modify -> force push.

Layer 4: Haiku Judge       Claude prompt hook for ambiguous commands.
                           AI second opinion when AST analysis is inconclusive.
                           Runs as a native Claude Code "type": "prompt" hook.

Layer 5: Auto-Checkpoint   Git stash or file backup BEFORE any risky operation.
                           Recovery instructions included in every block response.
```

### Decision flow

```
Tool call arrives
    |
    v
[AST Guard] -- score 0 --> DENY (blocked, no further layers)
    |
    | score > 0
    v
[Smart Routing] -- high risk + high confidence --> DENY
    |
    | ambiguous
    v
[Sequence Monitor] -- pattern match --> DENY or ESCALATE
    |
    | no pattern
    v
[Haiku Judge] -- LLM says unsafe --> ASK (human approval required)
    |
    | safe
    v
[Auto-Checkpoint] -- create recovery point --> ALLOW
```

Possible verdicts: `allow`, `deny`, `ask` (require human confirmation), `escalate`.

## Crate Dependency Graph

```
metaygn-shared          (types, constants, no dependencies)
    |
    +----> metaygn-core         (12-stage control loop, AST guard, heuristics)
    |          |
    |          +----> metaygn-memory     (SQLite + FTS5 graph memory, embeddings)
    |          |
    |          +----> metaygn-verifiers  (completion checker, test integrity)
    |
    +----> metaygn-sandbox      (process sandbox, WASM sandbox)
    |
    +----> metaygn-daemon       (Axum HTTP server, hooks API, auth, metrics)
    |          |
    |          +----> metaygn-core
    |          +----> metaygn-memory
    |          +----> metaygn-sandbox
    |
    +----> metaygn-cli          (clap CLI, TUI dashboard)
               |
               +----> metaygn-daemon (start command embeds the daemon)
               +----> metaygn-core
               +----> metaygn-memory
```

## Hook Lifecycle

Hooks fire on Claude Code lifecycle events:

| Hook event | When | Daemon endpoint |
|-----------|------|-----------------|
| `SessionStart` | New Claude Code session begins | (handled internally) |
| `UserPromptSubmit` | User submits a prompt | `POST /hooks/user-prompt-submit` |
| `PreToolUse` | Before any tool executes | `POST /hooks/pre-tool-use` |
| `PostToolUse` | After tool execution completes | `POST /hooks/post-tool-use` |
| `PostToolUseFailure` | After tool execution fails | `POST /hooks/post-tool-use` |
| `Stop` | Agent produces final response | `POST /hooks/stop` |
| `PreCompact` | Before context compaction | (handled by plugin) |
| `SessionEnd` | Session terminates | `POST /hooks/session-end` |

### 12-Stage Control Loop

Each hook invocation runs the daemon's internal control loop:

1. **Classify** -- Determine task type from prompt
2. **Assess** -- Evaluate risk level and difficulty
3. **Route** -- Semantic routing with confidence scoring
4. **Strategize** -- Select execution strategy based on risk/difficulty
5. **Tool Need** -- Determine if tool use is actually necessary
6. **Budget** -- Check token/cost budget constraints
7. **Competence** -- Evaluate agent competence for this task type
8. **Calibrate** -- Update prediction calibration
9. **Verify** -- Run completion and integrity checks
10. **Decide** -- Final allow/deny/ask/escalate verdict
11. **Learn** -- Record outcome for heuristic evolution
12. **Postprocess** -- Format response, create checkpoints

## Fallback Architecture

```
Claude Code --> Hooks --> Aletheia Daemon --> Decision + Checkpoint
                 |          |-- AST Guard (tree-sitter)
                 |          |-- 12-stage control loop
                 |          |-- Graph memory (SQLite + FTS5)
                 |          +-- Heuristic evolution
                 |
                 +-- (if daemon offline) --> TypeScript fallback --> Regex guards
```

When the daemon is unreachable, TypeScript hooks provide regex-based command guards as a fallback. Coverage is narrower but still catches common destructive patterns.
