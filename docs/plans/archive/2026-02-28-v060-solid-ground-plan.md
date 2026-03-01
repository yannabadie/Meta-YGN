# v0.6.0 "Solid Ground" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make MetaYGN a fully functional, installable plugin where `aletheia start` → `claude --plugin-dir .` works end-to-end with all 8 TS hooks firing through the daemon, persistent heuristics surviving restarts, and CI verifying the whole pipeline.

**Architecture:** Complete 7 missing TS hooks (same pattern as pre-tool-use.ts). Add `/admin/shutdown` endpoint. Implement daemon spawn/kill in CLI. Persist heuristic state in SQLite. Add CI integration job. Update hooks.json to use Bun directly.

**Tech Stack:** TypeScript (Bun), Rust (axum, clap, rusqlite), GitHub Actions

---

### Task 1: Complete 7 missing TypeScript hooks

**Files:**
- Create: `packages/hooks/src/session-start.ts`
- Create: `packages/hooks/src/user-prompt-submit.ts`
- Create: `packages/hooks/src/post-tool-use.ts`
- Create: `packages/hooks/src/post-tool-use-failure.ts`
- Create: `packages/hooks/src/stop.ts`
- Create: `packages/hooks/src/pre-compact.ts`
- Create: `packages/hooks/src/session-end.ts`

All hooks follow the EXACT same pattern as `packages/hooks/src/pre-tool-use.ts`:
1. `readStdin()` → parse JSON
2. Validate with `HookInputSchema.safeParse()`
3. `callDaemon("/hooks/{route}", input)` → if result, `respond(result)`
4. Local fallback (for hooks that need one) or exit silently
5. `.catch(() => process.exit(0))` — never block Claude Code

Each hook calls a different daemon route:
- `session-start.ts` → `GET /health` (just check daemon is alive, emit stack info on fallback)
- `user-prompt-submit.ts` → `POST /hooks/user-prompt-submit`
- `post-tool-use.ts` → `POST /hooks/post-tool-use`
- `post-tool-use-failure.ts` → `POST /hooks/post-tool-use` (same endpoint, different input)
- `stop.ts` → `POST /hooks/stop`
- `pre-compact.ts` → emit compaction guidance (no daemon endpoint needed, just emit text)
- `session-end.ts` → `POST /hooks/session-end` (fire-and-forget, no response needed)

For `session-end.ts`, the daemon may not respond (async hook). Just fire the request and exit 0.

**Verify:** `cd packages/hooks && npx tsc --noEmit` — no errors

**Commit:** `git commit -m "feat(hooks): complete all 8 TypeScript hooks for Claude Code integration"`

---

### Task 2: Update hooks.json to use Bun directly

**Files:**
- Modify: `hooks/hooks.json`

Replace ALL occurrences of:
```
"command": "bash \"${CLAUDE_PLUGIN_ROOT}/scripts/hook-runner.sh\" {hook-name}"
```

With:
```
"command": "bun run \"${CLAUDE_PLUGIN_ROOT}/packages/hooks/src/{hook-name}.ts\""
```

Mapping:
- `session-start` → `session-start.ts`
- `user-prompt-submit` → `user-prompt-submit.ts`
- `pre-tool-use` → `pre-tool-use.ts`
- `post-tool-use` → `post-tool-use.ts`
- `post-tool-use-failure` → `post-tool-use-failure.ts`
- `stop` → `stop.ts`
- `pre-compact` → `pre-compact.ts`
- `session-end` → `session-end.ts`

Keep the same timeouts, matchers, statusMessages, and async flags.

**Verify:** JSON is valid: `python -c "import json; json.load(open('hooks/hooks.json'))"`

**Commit:** `git commit -m "feat(hooks): update hooks.json to use Bun+TS hooks directly"`

---

### Task 3: Daemon shutdown endpoint + graceful lifecycle

**Files:**
- Create: `crates/daemon/src/api/admin.rs`
- Modify: `crates/daemon/src/api/mod.rs` — add admin module and routes
- Modify: `crates/daemon/src/main.rs` — wire shutdown signal
- Modify: `crates/daemon/src/lib.rs` — if needed
- Test: `crates/daemon/tests/api_test.rs` — add shutdown test

**Implementation:**

`admin.rs`:
```rust
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use tokio::sync::watch;

// The shutdown endpoint needs a way to signal the server to stop.
// Use a tokio::sync::watch channel stored in AppState.

pub async fn shutdown(State(tx): State<watch::Sender<bool>>) -> Json<Value> {
    let _ = tx.send(true);
    Json(json!({"ok": true, "message": "Shutdown initiated"}))
}
```

Add `pub shutdown_tx: watch::Sender<bool>` to AppState (or pass separately).

In `main.rs`, use `axum::serve(...).with_graceful_shutdown(async { rx.changed().await.ok(); })`.

After shutdown, delete the port file.

**Test:**
```rust
#[tokio::test]
async fn admin_shutdown_responds() {
    // POST /admin/shutdown → 200 with ok:true
}
```

**Commit:** `git commit -m "feat(daemon): /admin/shutdown endpoint with graceful lifecycle"`

---

### Task 4: CLI `aletheia start` and `aletheia stop`

**Files:**
- Modify: `crates/cli/src/main.rs` — rewrite Start and Stop handlers

**`aletheia start`:**
1. Check if daemon already running: read port file, health check
2. Find `aletheiad` binary: same directory as `aletheia`, or `../target/release/aletheiad`
3. Spawn detached: `std::process::Command::new(path).spawn()`
4. Poll for port file (every 500ms, max 10 seconds)
5. Health check the new daemon
6. Print "Daemon started on port {port}"

**`aletheia stop`:**
1. Read port from port file
2. POST to `http://127.0.0.1:{port}/admin/shutdown`
3. Poll until port file disappears (every 500ms, max 5 seconds)
4. Print "Daemon stopped" or "Daemon did not stop cleanly"

**Verify:** `cargo run -p metaygn-cli -- start` then `cargo run -p metaygn-cli -- status` then `cargo run -p metaygn-cli -- stop`

**Commit:** `git commit -m "feat(cli): working aletheia start/stop — spawn and gracefully shutdown daemon"`

---

### Task 5: Persistent heuristic state in SQLite

**Files:**
- Modify: `crates/memory/src/store.rs` — add heuristic persistence methods
- Modify: `crates/core/src/heuristics/evolver.rs` — add load/save trait
- Modify: `crates/daemon/src/app_state.rs` — load on startup
- Modify: `crates/daemon/src/api/heuristics.rs` — persist after mutations
- Test: `crates/memory/tests/store_test.rs` — add persistence tests

**Schema (add to MemoryStore::init_schema):**
```sql
CREATE TABLE IF NOT EXISTS heuristic_versions (
    id TEXT PRIMARY KEY,
    generation INTEGER NOT NULL,
    parent_id TEXT,
    fitness_json TEXT NOT NULL,
    risk_weights_json TEXT NOT NULL,
    strategy_scores_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS session_outcomes (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    task_type TEXT,
    risk_level TEXT,
    strategy_used TEXT,
    success INTEGER NOT NULL,
    tokens_consumed INTEGER,
    duration_ms INTEGER,
    errors_encountered INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Methods on MemoryStore:**
```rust
pub async fn save_heuristic_version(&self, version: &HeuristicVersion) -> Result<()>
pub async fn load_heuristic_versions(&self) -> Result<Vec<HeuristicVersion>>
pub async fn save_session_outcome(&self, outcome: &SessionOutcome) -> Result<()>
pub async fn load_recent_outcomes(&self, limit: u32) -> Result<Vec<SessionOutcome>>
```

**Tests:**
```rust
#[tokio::test] fn save_and_load_heuristic_version()
#[tokio::test] fn save_and_load_session_outcomes()
```

**Commit:** `git commit -m "feat(memory): persistent heuristic state — versions and outcomes survive daemon restart"`

---

### Task 6: CI integration tests

**Files:**
- Modify: `.github/workflows/ci.yml` — add integration-test job

**New job:**
```yaml
integration-test:
  name: Integration Test
  runs-on: ubuntu-latest
  needs: rust-check
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Build release
      run: cargo build --workspace --release
    - name: Start daemon
      run: |
        target/release/aletheiad &
        for i in $(seq 1 20); do
          [ -f ~/.claude/aletheia/daemon.port ] && break
          sleep 0.5
        done
        PORT=$(cat ~/.claude/aletheia/daemon.port)
        echo "DAEMON_PORT=$PORT" >> $GITHUB_ENV
    - name: Health check
      run: |
        curl -sf http://127.0.0.1:$DAEMON_PORT/health | jq .
    - name: Test pre-tool-use deny
      run: |
        RESULT=$(curl -s -X POST http://127.0.0.1:$DAEMON_PORT/hooks/pre-tool-use \
          -H 'Content-Type: application/json' \
          -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}')
        echo "$RESULT" | jq .
        echo "$RESULT" | grep -q '"deny"'
    - name: Test user-prompt-submit
      run: |
        curl -sf -X POST http://127.0.0.1:$DAEMON_PORT/hooks/user-prompt-submit \
          -H 'Content-Type: application/json' \
          -d '{"hook_event_name":"UserPromptSubmit","prompt":"fix typo"}' | jq .
    - name: Test budget
      run: |
        curl -sf http://127.0.0.1:$DAEMON_PORT/budget | jq .
    - name: Cleanup
      if: always()
      run: |
        curl -s -X POST http://127.0.0.1:$DAEMON_PORT/admin/shutdown || true
        sleep 1
```

**Commit:** `git commit -m "ci: add integration test job — daemon startup, hook verification, budget check"`

---

### Task 7: Plugin validation script

**Files:**
- Create: `scripts/validate-plugin.sh`

```bash
#!/bin/bash
set -e
echo "=== MetaYGN Plugin Validation ==="

ERRORS=0

# 1. Check plugin.json
if [ -f ".claude-plugin/plugin.json" ]; then
    python3 -c "import json; json.load(open('.claude-plugin/plugin.json'))" && echo "OK: plugin.json valid" || { echo "FAIL: plugin.json invalid"; ERRORS=$((ERRORS+1)); }
else
    echo "FAIL: .claude-plugin/plugin.json missing"; ERRORS=$((ERRORS+1))
fi

# 2. Check hooks.json
if [ -f "hooks/hooks.json" ]; then
    python3 -c "import json; json.load(open('hooks/hooks.json'))" && echo "OK: hooks.json valid" || { echo "FAIL: hooks.json invalid"; ERRORS=$((ERRORS+1)); }
else
    echo "FAIL: hooks/hooks.json missing"; ERRORS=$((ERRORS+1))
fi

# 3. Check settings.json
python3 -c "import json; json.load(open('settings.json'))" && echo "OK: settings.json valid" || { echo "FAIL: settings.json invalid"; ERRORS=$((ERRORS+1)); }

# 4. Check all skills have SKILL.md
for skill in skills/*/; do
    if [ -f "${skill}SKILL.md" ]; then
        echo "OK: ${skill}SKILL.md exists"
    else
        echo "FAIL: ${skill}SKILL.md missing"; ERRORS=$((ERRORS+1))
    fi
done

# 5. Check all agents have .md
for agent in agents/*.md; do
    echo "OK: $agent exists"
done

# 6. Check TS hooks exist
for hook in session-start user-prompt-submit pre-tool-use post-tool-use post-tool-use-failure stop pre-compact session-end; do
    if [ -f "packages/hooks/src/${hook}.ts" ]; then
        echo "OK: ${hook}.ts exists"
    else
        echo "FAIL: ${hook}.ts missing"; ERRORS=$((ERRORS+1))
    fi
done

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "=== ALL VALIDATIONS PASSED ==="
else
    echo "=== $ERRORS VALIDATION(S) FAILED ==="
    exit 1
fi
```

**Commit:** `git commit -m "build: add plugin validation script"`

---

### Task 8: Docs + version bump + merge

**Files:**
- Modify: `.claude-plugin/plugin.json` → v0.6.0
- Modify: `CHANGELOG.md` → v0.6.0 section
- Modify: `README.md` → update
- Modify: `memory-bank/progress.md`
- Modify: `memory-bank/activeContext.md`

**Commit:** `git commit -m "docs: v0.6.0 Solid Ground — changelog, readme, plugin version"`

Then push, PR, merge.
