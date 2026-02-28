# Aletheia Daemon API Reference

**Version**: 0.3.0 "Adaptive Topology"
**Base URL**: `http://127.0.0.1:{port}` (port read from `~/.claude/aletheia/daemon.port`)
**Binary**: `aletheiad` (crate: `metaygn-daemon`)

The daemon binds to a dynamic localhost port on startup and writes the port number
to `~/.claude/aletheia/daemon.port`. Clients discover the port by reading this file.
On graceful shutdown (Ctrl+C / SIGTERM), the port file is removed automatically.

---

## Health

### `GET /health`

Returns daemon liveness, version, and kernel verification status.

**Request**: none

**Response**:

```json
{
  "status": "ok",
  "version": "0.1.0",
  "kernel": "verified"
}
```

**Example**:

```bash
curl http://127.0.0.1:$(cat ~/.claude/aletheia/daemon.port)/health
```

---

## Hooks

All hook endpoints accept a `HookInput` payload and return a `HookOutput`.

### `HookInput` schema

```json
{
  "hook_event_name": "PreToolUse",
  "session_id": "string | null",
  "cwd": "string | null",
  "tool_name": "string | null",
  "tool_input": "object | null",
  "tool_response": "string | null",
  "prompt": "string | null",
  "error": "string | null",
  "last_assistant_message": "string | null",
  "source": "string | null",
  "reason": "string | null",
  "trigger": "string | null"
}
```

### `HookOutput` schema

```json
{
  "hookSpecificOutput": {
    "hookEventName": "string | null",
    "permissionDecision": "allow | deny | ask | null",
    "permissionDecisionReason": "string | null",
    "additionalContext": "string | null"
  }
}
```

---

### `POST /hooks/pre-tool-use`

Permission decision for a tool call. Runs the 5-guard pipeline, then control loop
stages 1-6 (classify through strategy) for risk assessment.

**Request**: `HookInput` with `tool_name` and `tool_input` populated.

**Response**: `HookOutput` with either:
- `permissionDecision: "deny"` + reason (destructive pattern, score=0)
- `permissionDecision: "ask"` + reason (high-risk pattern, score > 0)
- `additionalContext` with risk/strategy/difficulty assessment (allowed)

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/hooks/pre-tool-use \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Bash",
    "tool_input": {"command": "git push origin main"}
  }'
```

---

### `POST /hooks/post-tool-use`

Records verification context after tool execution. Updates the fatigue profiler
with error or success signals based on tool output content.

**Request**: `HookInput` with `tool_name` and `tool_response` populated.

**Response**: `HookOutput` with `additionalContext` containing verification guidance:
- Test failure detected in Bash output
- File modification recorded
- MCP tool output recorded
- Generic tool output recorded

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/hooks/post-tool-use \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PostToolUse",
    "tool_name": "Bash",
    "tool_response": "All tests passed."
  }'
```

---

### `POST /hooks/user-prompt-submit`

Analyses a user prompt via the control loop (stages 1-6), records fatigue
signals, and runs the TopologyPlanner to recommend an execution topology.

**Request**: `HookInput` with `prompt` populated.

**Response**: `HookOutput` with `additionalContext` containing:
- `[risk:{low|medium|high}]`
- `[strategy:{StepByStep|TreeExplore|...}]`
- `[budget:{N}tok]`
- `[task:{Bugfix|Feature|...}]`
- `[topology={Single|Vertical|Horizontal}]`

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/hooks/user-prompt-submit \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "UserPromptSubmit",
    "prompt": "Refactor the authentication module to use JWT tokens"
  }'
```

---

### `POST /hooks/stop`

Runs control loop stages 9-12 (calibrate, compact, decide, learn) and returns
proof-packet enforcement hints with metacognitive vector and lessons.

**Request**: `HookInput` (typically with `last_assistant_message`).

**Response**: `HookOutput` with `additionalContext` containing:
- `[decision:{Continue|Revise|Abstain|Escalate|Stop}]`
- `[metacog:META:c{n}h{n}g{n}x{n}p{n}]`
- `[lessons:{...}]`

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/hooks/stop \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "Stop",
    "last_assistant_message": "I have completed the refactoring."
  }'
```

---

### `POST /hooks/analyze`

Debug endpoint. Runs the full 12-stage control loop and returns the complete
`LoopContext` as JSON for inspection.

**Request**: `HookInput` (any fields).

**Response**: Full `LoopContext` serialized as JSON:

```json
{
  "input": { ... },
  "task_type": "Feature",
  "risk": "Medium",
  "difficulty": 0.45,
  "competence": 0.7,
  "tool_necessary": true,
  "budget": {
    "max_tokens": 50000,
    "consumed_tokens": 0,
    "max_latency_ms": 30000,
    "max_cost_usd": 0.50,
    "risk_tolerance": "Medium"
  },
  "strategy": "StepByStep",
  "decision": "Continue",
  "metacog_vector": {
    "confidence": 0.7,
    "coherence": 0.8,
    "grounding": 0.6,
    "complexity": 0.45,
    "progress": 0.0
  },
  "verification_results": [],
  "lessons": []
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/hooks/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "UserPromptSubmit",
    "prompt": "Delete all user records from the database"
  }'
```

---

## Memory

### `GET /memory/stats`

Returns the total number of events in the episodic memory store.

**Request**: none

**Response**:

```json
{
  "event_count": 142
}
```

**Example**:

```bash
curl http://127.0.0.1:$PORT/memory/stats
```

---

### `POST /memory/recall`

Full-text search over stored events using SQLite FTS5.

**Request**:

```json
{
  "query": "search terms",
  "limit": 10
}
```

| Field   | Type   | Required | Default | Description                |
|---------|--------|----------|---------|----------------------------|
| `query` | string | yes      | --      | FTS5 search query          |
| `limit` | u32    | no       | 10      | Maximum results to return  |

**Response**:

```json
{
  "events": [
    {
      "id": 1,
      "session_id": "abc-123",
      "event_type": "pre_tool_use",
      "payload": "...",
      "timestamp": "2026-02-28T10:30:00Z"
    }
  ]
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/memory/recall \
  -H "Content-Type: application/json" \
  -d '{"query": "git push", "limit": 5}'
```

---

## Graph Memory

### `POST /memory/nodes`

Insert a `MemoryNode` into the graph. Replaces existing node if `id` matches.

**Request**:

```json
{
  "id": "node-uuid-or-label",
  "node_type": "Task",
  "scope": "Session",
  "label": "Fix auth bug",
  "content": "Investigated JWT token expiry issue...",
  "embedding": [0.1, 0.2, 0.3],
  "created_at": "2026-02-28T10:00:00Z",
  "access_count": 0
}
```

| Field          | Type           | Required | Description                                   |
|----------------|----------------|----------|-----------------------------------------------|
| `id`           | string         | yes      | Unique node identifier                        |
| `node_type`    | enum           | yes      | Task, Decision, Evidence, Tool, Agent, Code, Error, Lesson |
| `scope`        | enum           | yes      | Session, Project, Global                      |
| `label`        | string         | yes      | Short human-readable label                    |
| `content`      | string         | yes      | Full node content (indexed by FTS5)           |
| `embedding`    | float[] | null | no       | Optional embedding vector for cosine search   |
| `created_at`   | string         | yes      | ISO-8601 timestamp                            |
| `access_count` | u32            | yes      | Number of times this node has been accessed   |

**Response**:

```json
{ "ok": true, "id": "node-uuid-or-label" }
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/memory/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "task-001",
    "node_type": "Task",
    "scope": "Project",
    "label": "Fix auth bug",
    "content": "JWT token expiry causes 401 on refresh",
    "embedding": null,
    "created_at": "2026-02-28T10:00:00Z",
    "access_count": 0
  }'
```

---

### `POST /memory/edges`

Insert a `MemoryEdge` between two existing nodes. Replaces if the
`(source_id, target_id, edge_type)` triple already exists.

**Request**:

```json
{
  "source_id": "task-001",
  "target_id": "evidence-001",
  "edge_type": "Produces",
  "weight": 1.0,
  "metadata": "optional string"
}
```

| Field       | Type        | Required | Description                                        |
|-------------|-------------|----------|----------------------------------------------------|
| `source_id` | string      | yes      | Source node ID                                     |
| `target_id` | string      | yes      | Target node ID                                     |
| `edge_type` | enum        | yes      | DependsOn, Produces, Verifies, Contradicts, Supersedes, RelatedTo |
| `weight`    | f64         | no       | Edge weight (default 1.0)                          |
| `metadata`  | string/null | no       | Arbitrary metadata string                          |

**Response**:

```json
{ "ok": true }
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/memory/edges \
  -H "Content-Type: application/json" \
  -d '{
    "source_id": "task-001",
    "target_id": "evidence-001",
    "edge_type": "Produces",
    "weight": 1.0,
    "metadata": null
  }'
```

---

### `POST /memory/graph/search`

Full-text search over graph node labels and content using FTS5.

**Request**:

```json
{
  "query": "auth bug",
  "limit": 10
}
```

**Response**:

```json
{
  "nodes": [
    {
      "id": "task-001",
      "node_type": "Task",
      "scope": "Project",
      "label": "Fix auth bug",
      "content": "JWT token expiry causes 401 on refresh",
      "created_at": "2026-02-28T10:00:00Z",
      "access_count": 0
    }
  ]
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/memory/graph/search \
  -H "Content-Type: application/json" \
  -d '{"query": "auth", "limit": 5}'
```

---

### `GET /memory/graph/stats`

Returns node and edge counts in the graph.

**Request**: none

**Response**:

```json
{
  "node_count": 47,
  "edge_count": 83
}
```

**Example**:

```bash
curl http://127.0.0.1:$PORT/memory/graph/stats
```

---

## Sandbox

### `POST /sandbox/exec`

Execute a code snippet in a subprocess sandbox with timeout (5 s default)
and output limits (64 KB default).

**Request**:

```json
{
  "language": "python",
  "code": "print(2 + 2)",
  "timeout_ms": 5000
}
```

| Field        | Type   | Required | Default | Description                          |
|--------------|--------|----------|---------|--------------------------------------|
| `language`   | string | yes      | --      | `"python"`, `"node"`, or `"bash"`    |
| `code`       | string | yes      | --      | Code to execute                      |
| `timeout_ms` | u64    | no       | --      | Timeout override (reserved, not yet used) |

**Response** (`SandboxResult`):

```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "4\n",
  "stderr": "",
  "duration_ms": 42,
  "timed_out": false
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/sandbox/exec \
  -H "Content-Type: application/json" \
  -d '{"language": "python", "code": "print(2+2)"}'
```

---

### `POST /sandbox/hypothesis`

Test a hypothesis by executing code and comparing the result against
`expected_success`.

**Request**:

```json
{
  "description": "Division by zero should raise an error",
  "language": "python",
  "code": "print(1/0)",
  "expected_success": false
}
```

| Field              | Type   | Required | Default | Description                        |
|--------------------|--------|----------|---------|------------------------------------|
| `description`      | string | yes      | --      | Human-readable hypothesis          |
| `language`         | string | yes      | --      | `"python"`, `"node"`, or `"bash"` |
| `code`             | string | yes      | --      | Code to execute                    |
| `expected_success` | bool   | no       | true    | Whether execution should succeed   |

**Response**: `SandboxResult` (same schema as `/sandbox/exec`).

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/sandbox/hypothesis \
  -H "Content-Type: application/json" \
  -d '{
    "description": "JSON parsing should work",
    "language": "python",
    "code": "import json; json.loads(\"{\\\"a\\\": 1}\")",
    "expected_success": true
  }'
```

---

## Profiler

### `GET /profiler/fatigue`

Returns the current human fatigue assessment.

**Request**: none

**Response** (`FatigueReport`):

```json
{
  "score": 0.45,
  "high_friction": false,
  "signals": [
    "2 short prompt(s) detected",
    "1 rapid-retry signal(s)"
  ],
  "recommendation": "Moderate fatigue detected: prefer smaller, safer changes"
}
```

| Field            | Type     | Description                                   |
|------------------|----------|-----------------------------------------------|
| `score`          | f64      | 0.0 (fully alert) to 1.0 (exhausted)          |
| `high_friction`  | bool     | True when score >= 0.7                         |
| `signals`        | string[] | Human-readable signal descriptions             |
| `recommendation` | string   | Actionable guidance based on score             |

**Example**:

```bash
curl http://127.0.0.1:$PORT/profiler/fatigue
```

---

### `POST /profiler/signal`

Record a fatigue signal from a hook event. Returns the updated fatigue report.

**Request**:

```json
{
  "signal_type": "prompt",
  "prompt": "fix it",
  "timestamp": "2026-02-28T23:45:00Z"
}
```

| Field         | Type        | Required | Description                                   |
|---------------|-------------|----------|-----------------------------------------------|
| `signal_type` | string      | yes      | `"prompt"`, `"error"`, or `"success"`          |
| `prompt`      | string/null | no       | Required when signal_type is `"prompt"`        |
| `timestamp`   | string/null | no       | ISO-8601 timestamp (defaults to now)           |

**Response**: `FatigueReport` (same schema as `GET /profiler/fatigue`).

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/profiler/signal \
  -H "Content-Type: application/json" \
  -d '{"signal_type": "error"}'
```

---

## Heuristics

### `POST /heuristics/outcome`

Record a `SessionOutcome` for heuristic fitness evaluation.

**Request** (`SessionOutcome`):

```json
{
  "session_id": "sess-abc-123",
  "task_type": "bugfix",
  "risk_level": "medium",
  "strategy_used": "vertical",
  "success": true,
  "tokens_consumed": 15000,
  "duration_ms": 45000,
  "errors_encountered": 1
}
```

| Field               | Type   | Required | Description                     |
|---------------------|--------|----------|---------------------------------|
| `session_id`        | string | yes      | Unique session identifier       |
| `task_type`         | string | yes      | bugfix, feature, security, etc. |
| `risk_level`        | string | yes      | low, medium, high               |
| `strategy_used`     | string | yes      | single, vertical, horizontal    |
| `success`           | bool   | yes      | Whether the session succeeded   |
| `tokens_consumed`   | u64    | yes      | Total tokens used               |
| `duration_ms`       | u64    | yes      | Wall-clock duration in ms       |
| `errors_encountered`| u32    | yes      | Number of errors during session |

**Response**:

```json
{ "ok": true }
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/heuristics/outcome \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "sess-001",
    "task_type": "bugfix",
    "risk_level": "low",
    "strategy_used": "single",
    "success": true,
    "tokens_consumed": 5000,
    "duration_ms": 12000,
    "errors_encountered": 0
  }'
```

---

### `POST /heuristics/evolve`

Trigger one evolution generation: evaluate all versions, select top performers
(tournament selection), mutate the best, and return the new best heuristic.

**Request**: none (empty body or `{}`).

**Response**:

```json
{
  "ok": true,
  "best": {
    "id": "uuid",
    "generation": 3,
    "parent_id": "parent-uuid",
    "fitness": {
      "verification_success_rate": 0.85,
      "token_efficiency": 0.72,
      "latency_score": 0.68,
      "composite": 0.75
    },
    "risk_weights": { "fs_write": 0.55, "exec_command": 0.82, ... },
    "strategy_scores": { "(low,easy)": 0.18, "(high,hard)": 0.91, ... },
    "created_at": "2026-02-28T12:00:00Z"
  }
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/heuristics/evolve
```

---

### `GET /heuristics/best`

Return the current best heuristic version (highest composite fitness).

**Request**: none

**Response**:

```json
{
  "best": {
    "id": "uuid",
    "generation": 2,
    "parent_id": "seed-uuid",
    "fitness": { ... },
    "risk_weights": { ... },
    "strategy_scores": { ... },
    "created_at": "2026-02-28T11:00:00Z"
  }
}
```

**Example**:

```bash
curl http://127.0.0.1:$PORT/heuristics/best
```

---

### `GET /heuristics/population`

Return population statistics for the heuristic evolver.

**Request**: none

**Response**:

```json
{
  "size": 12,
  "generation": 5,
  "best_fitness": 0.78
}
```

**Example**:

```bash
curl http://127.0.0.1:$PORT/heuristics/population
```

---

## Forge

### `POST /forge/generate`

Generate a verification tool from a named template with optional parameter
substitution. The resulting `ToolSpec` is cached by content hash (SHA-256).

**Request**:

```json
{
  "template": "grep-pattern-checker",
  "params": {}
}
```

| Field      | Type              | Required | Description                         |
|------------|-------------------|----------|-------------------------------------|
| `template` | string            | yes      | Template name (see `GET /forge/templates`) |
| `params`   | map<string,string>| no       | Parameter substitutions for `{{key}}` placeholders |

**Response**:

```json
{
  "ok": true,
  "spec": {
    "name": "grep-pattern-checker",
    "language": "Python",
    "source_code": "import re, sys, json...",
    "description": "Search for a regex pattern in given text",
    "content_hash": "a1b2c3d4..."
  }
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/forge/generate \
  -H "Content-Type: application/json" \
  -d '{"template": "json-validator", "params": {}}'
```

---

### `POST /forge/execute`

Execute a `ToolSpec` in the process sandbox. Input is piped to the script
via emulated stdin.

**Request**:

```json
{
  "spec": {
    "name": "json-validator",
    "language": "Python",
    "source_code": "import sys, json...",
    "description": "Validate JSON structure",
    "content_hash": "a1b2c3d4..."
  },
  "input": "{\"key\": \"value\"}"
}
```

| Field   | Type     | Required | Description                     |
|---------|----------|----------|---------------------------------|
| `spec`  | ToolSpec | yes      | The tool specification to run   |
| `input` | string   | no       | Input data piped to stdin       |

**Response**:

```json
{
  "ok": true,
  "result": {
    "tool_name": "json-validator",
    "success": true,
    "stdout": "{\"valid\": true, \"type\": \"dict\", \"keys\": [\"key\"]}",
    "stderr": "",
    "duration_ms": 85
  }
}
```

**Example**:

```bash
curl -X POST http://127.0.0.1:$PORT/forge/execute \
  -H "Content-Type: application/json" \
  -d '{
    "spec": {
      "name": "json-validator",
      "language": "Python",
      "source_code": "import sys, json\ninput_data = sys.stdin.read()\ntry:\n  parsed = json.loads(input_data)\n  print(json.dumps({\"valid\": True}))\nexcept json.JSONDecodeError as e:\n  print(json.dumps({\"valid\": False, \"error\": str(e)}))",
      "description": "Validate JSON",
      "content_hash": "abc123"
    },
    "input": "{\"test\": 1}"
  }'
```

---

### `GET /forge/templates`

List all available tool templates with their metadata.

**Request**: none

**Response**:

```json
{
  "templates": [
    {
      "name": "grep-pattern-checker",
      "description": "Search for a regex pattern in given text",
      "language": "Python",
      "params": []
    },
    {
      "name": "import-validator",
      "description": "Check if Python imports are valid",
      "language": "Python",
      "params": []
    },
    {
      "name": "json-validator",
      "description": "Validate JSON structure",
      "language": "Python",
      "params": []
    },
    {
      "name": "file-exists-checker",
      "description": "Check if files exist",
      "language": "Bash",
      "params": []
    }
  ]
}
```

**Example**:

```bash
curl http://127.0.0.1:$PORT/forge/templates
```
