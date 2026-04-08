# CLI Reference

The CLI binary is named `aletheia`. Install it via `cargo install --path crates/cli` or run directly with `cargo run -p metaygn-cli --`.

## Commands

### `aletheia start`

Start the Aletheia daemon.

```
aletheia start [--db-path PATH]
```

| Flag | Description | Default |
|------|-------------|---------|
| `--db-path PATH` | Path to the SQLite database file | `~/.claude/aletheia/metaygn.db` |

Example:

```bash
$ aletheia start
Aletheia daemon started on 127.0.0.1:3100
Token written to ~/.claude/aletheia/daemon.token
```

---

### `aletheia stop`

Stop the running daemon via the `/admin/shutdown` endpoint.

```
aletheia stop
```

Example:

```bash
$ aletheia stop
Daemon stopped.
```

---

### `aletheia status`

Show daemon health by querying `/health`.

```
aletheia status
```

Example:

```bash
$ aletheia status
Daemon: running (v2.6.0)
Kernel: verified
```

---

### `aletheia doctor`

Check installation health. Validates daemon connectivity, plugin structure, hooks, skills, agents, and database.

```
aletheia doctor
```

Example:

```bash
$ aletheia doctor
MetaYGN Doctor
==============
Daemon .............. OK (http://127.0.0.1:3100)
Plugin .............. OK (.claude-plugin/plugin.json found)
Hooks ............... OK (hooks.json valid, 8 hooks registered)
Skills .............. OK (skills/ directory found)
Agents .............. OK (agents/ directory found)
Database ............ OK (SQLite accessible)
```

---

### `aletheia recall`

Search graph memory via FTS (full-text search).

```
aletheia recall --query QUERY [--limit N]
```

| Flag | Description | Default |
|------|-------------|---------|
| `--query QUERY` | Search query (required) | -- |
| `--limit N` | Maximum number of results | 10 |

Example:

```bash
$ aletheia recall --query "terraform destroy" --limit 5
[1] session=abc123 type=PreToolUse ts=2026-04-01T10:32:00Z
    payload: {"tool_name":"Bash","command":"terraform destroy","decision":"deny"}
```

---

### `aletheia eval`

Run calibration evaluation on session data. Displays Brier score and calibration buckets.

```
aletheia eval
```

Example:

```bash
$ aletheia eval
Calibration Report
==================
Brier score: 0.182 (lower is better)
Sample count: 47

Bucket       Count  Predicted  Actual
0-20%        3      0.10       0.00
20-40%       8      0.30       0.25
40-60%       12     0.50       0.58
60-80%       15     0.70       0.73
80-100%      9      0.90       0.89
```

---

### `aletheia top`

Launch real-time TUI (terminal UI) cognitive telemetry dashboard. Shows live session metrics, fatigue score, budget consumption, and guard activity.

```
aletheia top
```

Press `q` to quit.

---

### `aletheia replay`

Replay a past session's hook timeline.

```
aletheia replay [SESSION_ID]
```

When called without a session ID, lists available sessions. When called with a session ID, displays the full hook event timeline.

Example:

```bash
$ aletheia replay
Sessions:
  abc123  (42 events, 2026-04-01 10:30 - 11:15)
  def456  (18 events, 2026-04-01 14:00 - 14:22)

$ aletheia replay abc123
[10:30:01] UserPromptSubmit  -> classify=CodeGeneration, risk=Low
[10:30:05] PreToolUse        -> tool=Bash, command="cargo test", decision=allow
[10:30:12] PostToolUse       -> success=true, tokens=1200
...
```

---

### `aletheia init`

Initialize MetaYGN configuration in the current project. Scaffolds `.claude/` config files.

```
aletheia init [--force]
```

| Flag | Description |
|------|-------------|
| `--force` | Overwrite existing configuration |

Example:

```bash
$ aletheia init
Created .claude/settings.json
Created hooks.json
Initialized MetaYGN configuration.
```

---

### `aletheia mcp`

Launch the MCP (Model Context Protocol) stdio server. Used by Claude Code and other MCP clients for tool integration.

```
aletheia mcp
```

Exposes 5 MCP tools: `metacog_assess`, `metacog_decide`, `metacog_recall`, `metacog_prune`, `metacog_verify`.

---

### `aletheia export`

Export RL (reinforcement learning) trajectories to JSONL file for offline training.

```
aletheia export [--limit N]
```

| Flag | Description | Default |
|------|-------------|---------|
| `--limit N` | Maximum number of trajectories to export | 100 |

Example:

```bash
$ aletheia export --limit 50
Exported 50 trajectories to trajectories.jsonl
```
