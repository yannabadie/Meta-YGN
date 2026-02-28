#!/bin/bash
set -e

# ============================================================================
# Meta-YGN End-to-End Integration Test
#
# Tests the full daemon + CLI integration:
#   1. Build the workspace
#   2. Start daemon in background
#   3. Verify port file creation
#   4. Health check
#   5. Pre-tool-use: safe command (should allow)
#   6. Pre-tool-use: destructive command (should deny)
#   7. User prompt submit: high risk detection
#   8. Memory stats
#   9. CLI status
#
# Requirements: bash, curl, cargo (Rust toolchain)
# Platform: Linux, macOS, Windows (Git Bash / MSYS2)
# ============================================================================

DAEMON_PID=""
PORT_FILE="$HOME/.claude/aletheia/daemon.port"

# Resolve paths to binaries (set after build)
DAEMON_BIN=""
CLI_BIN=""

# ---------------------------------------------------------------------------
# Cleanup handler: always kill daemon and remove port file on exit
# ---------------------------------------------------------------------------
cleanup() {
    echo ""
    echo "Cleaning up..."
    if [ -n "$DAEMON_PID" ]; then
        kill "$DAEMON_PID" 2>/dev/null || true
        wait "$DAEMON_PID" 2>/dev/null || true
    fi
    rm -f "$PORT_FILE"
    echo "Done."
}
trap cleanup EXIT

fail() {
    echo "FAIL: $1"
    exit 1
}

# ---------------------------------------------------------------------------
# Determine the project root (script lives in tests/)
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Meta-YGN E2E Test ==="
echo "   Project root: $PROJECT_ROOT"
echo ""

# ---------- Step 1: Build workspace ----------
echo "[1/8] Building workspace..."
cargo build --workspace --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>&1 | tail -1

# Locate built binaries
if [ -f "$PROJECT_ROOT/target/debug/aletheiad.exe" ]; then
    DAEMON_BIN="$PROJECT_ROOT/target/debug/aletheiad.exe"
    CLI_BIN="$PROJECT_ROOT/target/debug/aletheia.exe"
elif [ -f "$PROJECT_ROOT/target/debug/aletheiad" ]; then
    DAEMON_BIN="$PROJECT_ROOT/target/debug/aletheiad"
    CLI_BIN="$PROJECT_ROOT/target/debug/aletheia"
else
    fail "Could not find aletheiad binary in target/debug/"
fi
echo "   Daemon binary: $DAEMON_BIN"
echo "   CLI binary:    $CLI_BIN"

# ---------- Step 2: Start daemon in background ----------
echo "[2/8] Starting daemon..."
# Remove stale port file if present
rm -f "$PORT_FILE"
"$DAEMON_BIN" &
DAEMON_PID=$!

# Wait for daemon to start and write port file (up to 10 seconds)
echo -n "   Waiting for port file"
for i in $(seq 1 20); do
    if [ -f "$PORT_FILE" ]; then
        echo " OK"
        break
    fi
    echo -n "."
    sleep 0.5
done

# ---------- Step 3: Read port file ----------
if [ ! -f "$PORT_FILE" ]; then
    echo ""
    fail "No port file at $PORT_FILE after 10s"
fi
PORT=$(cat "$PORT_FILE")
echo "   Daemon running on port $PORT"

# ---------- Step 4: Health check ----------
echo "[3/8] Health check..."
HEALTH=$(curl -sf "http://127.0.0.1:$PORT/health")
echo "   $HEALTH"
echo "$HEALTH" | grep -q '"ok"' || fail "health check did not return ok"

# ---------- Step 5: Pre-tool-use safe command (should allow) ----------
echo "[4/8] Pre-tool-use: safe command..."
SAFE=$(curl -sf -X POST "http://127.0.0.1:$PORT/hooks/pre-tool-use" \
    -H "Content-Type: application/json" \
    -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"}}')
echo "   $SAFE"
# HookOutput::allow() returns {} -- must NOT contain "deny"
if echo "$SAFE" | grep -qi '"deny"'; then
    fail "safe command was denied"
fi
echo "   OK: safe command was allowed"

# ---------- Step 6: Pre-tool-use destructive command (should deny) ----------
echo "[5/8] Pre-tool-use: destructive command..."
DESTRUCTIVE=$(curl -sf -X POST "http://127.0.0.1:$PORT/hooks/pre-tool-use" \
    -H "Content-Type: application/json" \
    -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}')
echo "   $DESTRUCTIVE"
echo "$DESTRUCTIVE" | grep -qi '"deny"' || fail "destructive command was not denied"
echo "   OK: destructive command was denied"

# ---------- Step 7: User prompt submit: high risk ----------
echo "[6/8] User prompt submit: high risk..."
PROMPT=$(curl -sf -X POST "http://127.0.0.1:$PORT/hooks/user-prompt-submit" \
    -H "Content-Type: application/json" \
    -d '{"hook_event_name":"UserPromptSubmit","prompt":"deploy to production"}')
echo "   $PROMPT"
echo "$PROMPT" | grep -qi "high" || fail "high risk not detected for deploy prompt"
echo "   OK: high risk detected"

# ---------- Step 8: Memory stats ----------
echo "[7/8] Memory stats..."
STATS=$(curl -sf "http://127.0.0.1:$PORT/memory/stats")
echo "   $STATS"
echo "$STATS" | grep -q "event_count" || fail "no event_count in memory stats"
echo "   OK: memory stats returned"

# ---------- Step 9: CLI status ----------
echo "[8/8] CLI status..."
CLI_OUTPUT=$("$CLI_BIN" status 2>&1)
echo "   $CLI_OUTPUT"
echo "$CLI_OUTPUT" | grep -qi "RUNNING" || fail "CLI status did not report RUNNING"
echo "   OK: CLI reports daemon running"

echo ""
echo "=== ALL E2E TESTS PASSED ==="
