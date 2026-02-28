# v0.6.0 "Solid Ground" — Design Document

**Date**: 2026-02-28
**Status**: Approved
**Approach**: Full Stack — complete TS hooks, daemon lifecycle, persistent heuristics, CI tests, plugin validation

---

## Goal

Make MetaYGN installable and functional without bricolage. A developer should be able to:
1. `aletheia init` → project configured
2. `aletheia start` → daemon running
3. `claude --plugin-dir /path/to/MetaYGN` → all hooks fire through daemon
4. `aletheia status` → everything green
5. `aletheia stop` → clean shutdown

## Section 1: Complete TypeScript hooks (7 missing)

Create all 7 remaining TS hooks in `packages/hooks/src/`:
- `session-start.ts` — calls daemon `/hooks/session-start`, emits stack detection on fallback
- `user-prompt-submit.ts` — calls daemon `/hooks/user-prompt-submit`, returns risk/strategy/budget
- `post-tool-use.ts` — calls daemon `/hooks/post-tool-use`, returns verification signals
- `post-tool-use-failure.ts` — calls daemon equivalent, returns error diagnosis
- `stop.ts` — calls daemon `/hooks/stop`, returns completion verification + proof packet
- `pre-compact.ts` — calls daemon equivalent, returns compaction guidance
- `session-end.ts` — calls daemon, logs session end (async, no blocking)

Each follows the same pattern as existing `pre-tool-use.ts`: stdin JSON → daemon call → fallback → stdout JSON.

Update `hooks/hooks.json` to point to `bun run packages/hooks/src/{hook}.ts` directly instead of `hook-runner.sh`.

## Section 2: Daemon lifecycle (start/stop)

### `POST /admin/shutdown` endpoint
New endpoint in daemon that triggers `tokio::signal` graceful shutdown, cleans up port file.

### `aletheia start`
1. Check if daemon already running (port file exists + health check)
2. Build path to `aletheiad` binary (same directory as `aletheia`, or `target/release/`)
3. Spawn as detached process (`std::process::Command` with `.spawn()`)
4. Poll for port file (max 10 seconds)
5. Health check the daemon
6. Print success message

### `aletheia stop`
1. Read port from port file
2. POST to `/admin/shutdown`
3. Poll until port file disappears (max 5 seconds)
4. Print success message
5. If timeout, print "Daemon did not stop cleanly"

## Section 3: Persistent heuristics

### Schema addition
Add to daemon's SQLite schema:
```sql
CREATE TABLE IF NOT EXISTS heuristic_versions (
    id TEXT PRIMARY KEY,
    generation INTEGER,
    parent_id TEXT,
    fitness_json TEXT,
    risk_weights_json TEXT,
    strategy_scores_json TEXT,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS session_outcomes (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    task_type TEXT,
    risk_level TEXT,
    strategy_used TEXT,
    success INTEGER,
    tokens_consumed INTEGER,
    duration_ms INTEGER,
    errors_encountered INTEGER,
    created_at TEXT
);
```

### Load on startup
When daemon starts, load all heuristic versions and recent outcomes from DB into HeuristicEvolver.

### Save on mutation
After `evolve_generation()`, persist the new version to DB.
After `record_outcome()`, persist the outcome to DB.

## Section 4: CI integration tests

New job in `.github/workflows/ci.yml`:
```yaml
integration-test:
  name: Integration Tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Build
      run: cargo build --workspace --release
    - name: Start daemon
      run: |
        target/release/aletheiad &
        sleep 3
        PORT=$(cat ~/.claude/aletheia/daemon.port)
        echo "DAEMON_PORT=$PORT" >> $GITHUB_ENV
    - name: Health check
      run: curl -sf http://127.0.0.1:$DAEMON_PORT/health
    - name: Test hooks
      run: |
        # Safe command → allow
        curl -sf -X POST http://127.0.0.1:$DAEMON_PORT/hooks/pre-tool-use \
          -H "Content-Type: application/json" \
          -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"}}'
        # Destructive → deny
        RESULT=$(curl -s -X POST http://127.0.0.1:$DAEMON_PORT/hooks/pre-tool-use \
          -H "Content-Type: application/json" \
          -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}')
        echo "$RESULT" | grep -q "deny"
    - name: Stop daemon
      run: kill %1 || true
```

## Section 5: Plugin validation structure

Ensure `hooks/hooks.json` commands work. Test with a dry-run script that verifies:
- All referenced scripts exist
- `bun` is available (or fallback to `python3`)
- `plugin.json` has required fields
- `settings.json` is valid JSON
- All skill SKILL.md files have valid frontmatter
