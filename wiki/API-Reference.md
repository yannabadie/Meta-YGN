# API Reference

The Aletheia daemon listens on `127.0.0.1:3100`. All endpoints except `/health` require a bearer token.

## Authentication

The daemon generates a UUID v4 token at startup and writes it to `~/.claude/aletheia/daemon.token`. Include it in every request:

```bash
TOKEN=$(cat ~/.claude/aletheia/daemon.token)
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/...
```

Set `METAYGN_STRICT_AUTH=1` to reject unauthenticated requests with 401. Without this flag, unauthenticated requests are allowed with a warning (v2.5 backward compatibility).

---

## Health

### `GET /health`

No authentication required. Returns daemon status.

```bash
curl http://127.0.0.1:3100/health
```

```json
{
  "status": "ok",
  "version": "2.6.0",
  "kernel": "verified"
}
```

---

## Hooks

These endpoints are called by Claude Code hooks on each lifecycle event.

### `POST /hooks/pre-tool-use`

Evaluate a tool call before execution. Returns allow/deny/ask verdict.

```bash
curl -X POST http://127.0.0.1:3100/hooks/pre-tool-use \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}'
```

### `POST /hooks/post-tool-use`

Record tool execution result. Updates session state and budget.

```bash
curl -X POST http://127.0.0.1:3100/hooks/post-tool-use \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc","tool_name":"Bash","tool_result":"success","tokens_used":150}'
```

### `POST /hooks/user-prompt-submit`

Classify a user prompt. Returns task type, risk level, and strategy.

```bash
curl -X POST http://127.0.0.1:3100/hooks/user-prompt-submit \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc","prompt":"Deploy to production"}'
```

### `POST /hooks/stop`

Handle agent stop event. Runs completion verification.

```bash
curl -X POST http://127.0.0.1:3100/hooks/stop \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc","response":"Done! Created the file at src/main.rs"}'
```

### `POST /hooks/session-end`

End a session. Persists session outcome, triggers heuristic learning.

```bash
curl -X POST http://127.0.0.1:3100/hooks/session-end \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc"}'
```

### `POST /hooks/analyze`

Post-session analysis. Returns session summary with metrics.

```bash
curl -X POST http://127.0.0.1:3100/hooks/analyze \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc"}'
```

---

## Memory

### `POST /memory/recall`

Full-text search over stored events.

```bash
curl -X POST http://127.0.0.1:3100/memory/recall \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query":"terraform destroy","limit":5}'
```

```json
{
  "events": [...],
  "results": [...]
}
```

### `GET /memory/stats`

Return total event count.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/memory/stats
```

```json
{ "event_count": 1247 }
```

---

## Graph Memory

### `POST /memory/nodes`

Insert a memory node.

```bash
curl -X POST http://127.0.0.1:3100/memory/nodes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"id":"n1","node_type":"concept","scope":"session","label":"safety","content":"AST guard blocked rm -rf"}'
```

### `POST /memory/edges`

Insert a memory edge between nodes.

```bash
curl -X POST http://127.0.0.1:3100/memory/edges \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"source":"n1","target":"n2","edge_type":"related_to","weight":1.0}'
```

### `POST /memory/graph/search`

Full-text search over graph nodes.

```bash
curl -X POST http://127.0.0.1:3100/memory/graph/search \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query":"safety","limit":10}'
```

### `GET /memory/graph/stats`

Return node and edge counts.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/memory/graph/stats
```

```json
{ "node_count": 342, "edge_count": 891 }
```

### `POST /memory/semantic`

Vector-similarity search over graph nodes using embeddings.

```bash
curl -X POST http://127.0.0.1:3100/memory/semantic \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query":"destructive command","limit":5}'
```

```json
{
  "results": [
    { "id": "n1", "label": "rm-rf-block", "content": "...", "score": 0.87 }
  ],
  "provider": "fastembed",
  "dimension_warning": false
}
```

---

## Session

### `GET /session/{session_id}/state`

Return the full structured session state.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/session/abc123/state
```

```json
{
  "session_id": "abc123",
  "task_type": "CodeGeneration",
  "risk": "Low",
  "strategy": "Standard",
  "difficulty": 0.3,
  "competence": 0.85,
  "tool_calls": 12,
  "errors": 0,
  "success_count": 12,
  "tokens_consumed": 4500,
  "fatigue_score": 0.15,
  "lessons": [],
  "verification_results": [],
  "has_execution_plan": false
}
```

---

## Budget

### `GET /budget`

Return the full global budget status.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/budget
```

### `GET /budget/{session_id}`

Return session-local budget.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/budget/abc123
```

### `POST /budget/consume`

Consume tokens and cost, return updated summary.

```bash
curl -X POST http://127.0.0.1:3100/budget/consume \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"tokens":500,"cost_usd":0.015}'
```

---

## Profiler

### `GET /profiler/fatigue`

Return current fatigue assessment.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/profiler/fatigue
```

```json
{
  "score": 0.35,
  "high_friction": false,
  "signals": ["3 errors in last 10 actions"],
  "recommendation": "Monitor error rate"
}
```

### `POST /profiler/signal`

Record a fatigue signal. Signal types: `prompt`, `error`, `success`.

```bash
curl -X POST http://127.0.0.1:3100/profiler/signal \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"signal_type":"error"}'
```

---

## Heuristics

### `POST /heuristics/outcome`

Record a session outcome for fitness evaluation.

```bash
curl -X POST http://127.0.0.1:3100/heuristics/outcome \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc","task_type":"CodeGeneration","risk_level":"Low","strategy_used":"Standard","success":true,"tokens_consumed":4500,"duration_ms":120000,"errors_encountered":0}'
```

### `POST /heuristics/evolve`

Trigger one evolution generation. Returns the best heuristic version.

```bash
curl -X POST http://127.0.0.1:3100/heuristics/evolve \
  -H "Authorization: Bearer $TOKEN"
```

### `GET /heuristics/best`

Return the current best heuristic version.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/heuristics/best
```

### `GET /heuristics/population`

Return population statistics (size, generation, best fitness).

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/heuristics/population
```

---

## Forge (Tool Synthesis)

### `POST /forge/generate`

Generate a tool from a named template.

```bash
curl -X POST http://127.0.0.1:3100/forge/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"template":"validator","params":{"name":"json-lint"}}'
```

### `POST /forge/execute`

Execute a tool spec with given input.

```bash
curl -X POST http://127.0.0.1:3100/forge/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"spec":{"name":"test","language":"python","code":"print(1+1)"},"input":""}'
```

### `GET /forge/templates`

List available template names and descriptions.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/forge/templates
```

---

## Sandbox

### `POST /sandbox/exec`

Execute code in the process sandbox.

```bash
curl -X POST http://127.0.0.1:3100/sandbox/exec \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"language":"python","code":"print(2+2)","timeout_ms":5000}'
```

```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "4\n",
  "stderr": "",
  "duration_ms": 45,
  "timed_out": false
}
```

### `POST /sandbox/hypothesis`

Test a hypothesis in the sandbox.

```bash
curl -X POST http://127.0.0.1:3100/sandbox/hypothesis \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"description":"2+2 equals 4","language":"python","code":"assert 2+2 == 4","expected_success":true}'
```

### `POST /sandbox/wasm` (feature: `wasm`)

Execute a WAT module in the WASM sandbox. Only available when compiled with `--features wasm`.

```bash
curl -X POST http://127.0.0.1:3100/sandbox/wasm \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"wat":"(module (func (export \"main\") (result i32) i32.const 42))","timeout_ms":5000}'
```

---

## Replay

### `GET /replay/sessions`

List all recorded sessions with event counts and time range.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/replay/sessions
```

### `GET /replay/{session_id}`

Retrieve all replay events for a session.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/replay/abc123
```

---

## Trajectories

### `GET /trajectories/export?limit=N`

Export RL trajectories as JSON.

```bash
curl -H "Authorization: Bearer $TOKEN" "http://127.0.0.1:3100/trajectories/export?limit=50"
```

---

## Calibration

### `GET /calibration`

Return Brier score and calibration buckets.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/calibration
```

```json
{
  "brier_score": 0.182,
  "sample_count": 47,
  "buckets": [
    { "range": "0-20%", "count": 3, "avg_predicted": 0.1, "avg_actual": 0.0 },
    { "range": "80-100%", "count": 9, "avg_predicted": 0.9, "avg_actual": 0.89 }
  ]
}
```

---

## Metrics

### `GET /metrics`

Prometheus-compatible metrics endpoint. Returns plain text.

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/metrics
```

Metrics exposed:
- `metaygn_active_sessions` (gauge)
- `metaygn_events_total` (counter)
- `metaygn_graph_nodes_total` (counter)
- `metaygn_fatigue_score` (gauge)
- `metaygn_tokens_consumed_total` (counter)

---

## Admin

### `POST /admin/shutdown`

Initiate graceful daemon shutdown.

```bash
curl -X POST http://127.0.0.1:3100/admin/shutdown \
  -H "Authorization: Bearer $TOKEN"
```

---

## Proxy

### `POST /proxy/anthropic`

Context pruning proxy. Prunes messages before forwarding to Anthropic API.

```bash
curl -X POST http://127.0.0.1:3100/proxy/anthropic \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"messages":[...]}'
```
