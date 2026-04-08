# Troubleshooting

## Daemon not starting

**Symptom:** `aletheia start` exits immediately or `aletheia status` shows "not running".

**Check 1: Port conflict**

The daemon binds to `127.0.0.1:3100`. If another process is using that port:

```bash
# Linux/macOS
lsof -i :3100

# Windows
netstat -ano | findstr :3100
```

Kill the conflicting process or configure a different port.

**Check 2: Database path**

If the default database path (`~/.claude/aletheia/metaygn.db`) is inaccessible:

```bash
aletheia start --db-path ./metaygn.db
```

**Check 3: Rust version**

Ensure you have Rust stable 2024 edition or later:

```bash
rustc --version   # should be 1.85+
rustup update
```

---

## Hooks not firing

**Symptom:** Claude Code works normally but no safety checks appear. `aletheia replay` shows no events.

**Check 1: Plugin loaded**

Make sure you started Claude Code with the plugin:

```bash
claude --plugin-dir .
```

**Check 2: hooks.json valid**

Run the doctor command:

```bash
aletheia doctor
```

If it reports "Hooks ... FAIL", check that `hooks.json` exists at the project root and is valid JSON. The hooks file should reference the daemon endpoints.

**Check 3: Daemon reachable**

```bash
curl http://127.0.0.1:3100/health
```

If this fails, the daemon is not running. Start it with `aletheia start`.

**Check 4: TypeScript dependencies**

Ensure TypeScript hook dependencies are installed:

```bash
pnpm install
```

---

## 401 Unauthorized

**Symptom:** API calls return 401 or hooks fail with "unauthorized".

**Cause:** Bearer-token authentication is enabled (v2.6.0+). The daemon generates a new token on each start.

**Fix 1: Read the current token**

```bash
cat ~/.claude/aletheia/daemon.token
```

Use this token in your API calls:

```bash
curl -H "Authorization: Bearer $(cat ~/.claude/aletheia/daemon.token)" http://127.0.0.1:3100/memory/stats
```

**Fix 2: Check strict mode**

If `METAYGN_STRICT_AUTH=1` is set, unauthenticated requests are rejected outright. Without this flag, they are allowed with a warning (v2.5 backward compatibility).

```bash
# Disable strict auth temporarily
unset METAYGN_STRICT_AUTH
```

**Fix 3: Token file missing**

If `daemon.token` does not exist, the daemon may not have started correctly. Restart it:

```bash
aletheia stop
aletheia start
cat ~/.claude/aletheia/daemon.token
```

---

## False positives (safe commands blocked)

**Symptom:** A command you know is safe gets denied or flagged for approval.

**Understanding the score:** The daemon assigns a risk score from 0 (most dangerous) to 100 (completely safe). Score 0 means immediate block. Scores 1-30 trigger "ask" (human approval). Scores above 30 are allowed.

**Check the verdict:**

Look at the hook response:

```json
{
  "permissionDecision": "deny",
  "reason": "destructive command: ...",
  "score": 0
}
```

**Report it:** Open a GitHub issue with the command, score, and reason. The adaptive guard system will eventually learn from false positives, but bug reports help us fix the root cause.

**Workaround:** When the verdict is "ask", you can approve the command in Claude Code's permission prompt. The daemon records your override as feedback for heuristic evolution.

---

## High latency on hooks

**Symptom:** Claude Code feels slow, tool calls take longer than expected.

**Check 1: Daemon performance**

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3100/profiler/fatigue
```

If `high_friction` is true, the session may be in a meltdown loop. The meltdown detector (Shannon entropy, theta=1.711) flags this automatically.

**Check 2: Database size**

Large databases can slow down FTS queries:

```bash
ls -la ~/.claude/aletheia/metaygn.db
```

If the database is very large, consider starting fresh:

```bash
aletheia start --db-path ./fresh.db
```

**Check 3: Feature gates**

The `semantic` feature (fastembed) adds embedding computation time. The `wasm` feature adds Wasmtime initialization. If you do not need these, build without them:

```bash
cargo build --workspace --no-default-features
```

---

## Build failures

**Symptom:** `cargo build --workspace` fails.

**Check 1: tree-sitter dependencies**

The AST guard requires tree-sitter and tree-sitter-bash. These are compiled from C source:

```bash
# Ensure a C compiler is available
cc --version    # or gcc --version, cl.exe on Windows
```

**Check 2: SQLite**

The memory crate uses `rusqlite` which bundles SQLite by default. If you see SQLite-related build errors, ensure the `bundled` feature is enabled (it is by default).

**Check 3: Clean build**

```bash
cargo clean
cargo build --workspace
```

---

## MCP server issues

**Symptom:** `aletheia mcp` starts but Claude Code does not see the tools.

**Check 1:** Ensure the MCP configuration in `.mcp.json` points to the correct binary path.

**Check 2:** The MCP server exposes 5 tools: `metacog_assess`, `metacog_decide`, `metacog_recall`, `metacog_prune`, `metacog_verify`. Verify with an MCP client that these are listed.

**Check 3:** MCP runs over stdio. Ensure no other process is consuming stdin/stdout.
