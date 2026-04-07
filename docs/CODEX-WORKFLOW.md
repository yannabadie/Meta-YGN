# MetaYGN Workflow For Codex

MetaYGN works in Codex through MCP tools (not automatic Claude hooks).
This workflow runs in strict mode by default.

## Quick Start

Windows:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-codex-metaygn.ps1
```

macOS/Linux:

```bash
bash ./scripts/start-codex-metaygn.sh

# Preview the bootstrap prompt only
bash ./scripts/start-codex-metaygn.sh --no-launch
```

This does three things:

1. Ensures `aletheia` is registered as a Codex MCP server.
2. Loads a bootstrap metacognitive protocol prompt.
3. Stops any running HTTP daemon to avoid DB contention with MCP mode.
4. Launches `codex` with that prompt.

## Runtime Protocol

- Start: call `metacog_classify`.
- Before risky changes: call `metacog_status`.
- After meaningful actions: call `metacog_verify`.
- If repeated failures: call `metacog_prune`.
- Before final response: call `metacog_verify` then `metacog_status`, then summarize risk/evidence/uncertainty.
- If the verification gate is not satisfied, the agent must emit the block message and run missing checks first.

## Notes

- This gives Codex a useful “self-check loop” close to Claude-hook behavior, but it remains explicit MCP calls.
- If `codex mcp list` does not show `aletheia`, run `scripts/install-codex.ps1` or `scripts/install-codex.sh`.
