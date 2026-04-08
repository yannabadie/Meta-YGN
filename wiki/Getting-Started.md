# Getting Started

## Prerequisites

| Tool | Minimum version | Check |
|------|----------------|-------|
| Rust | 1.85+ (stable 2024 edition) | `rustc --version` |
| Node.js | 22+ | `node --version` |
| pnpm | 9+ | `pnpm --version` |

Install Rust via [rustup](https://rustup.rs/). Install pnpm via `npm install -g pnpm`.

## Clone and build

```bash
git clone https://github.com/yannabadie/Meta-YGN && cd Meta-YGN
cargo build --workspace          # builds 7 crates: shared, core, memory, daemon, cli, verifiers, sandbox
pnpm install                     # TypeScript hook dependencies
```

The workspace compiles these crates:

| Crate | Purpose |
|-------|---------|
| `metaygn-shared` | Shared types and constants |
| `metaygn-core` | Control loop stages, heuristics, AST guard |
| `metaygn-memory` | SQLite + FTS5 graph memory |
| `metaygn-daemon` | Axum HTTP daemon |
| `metaygn-cli` | CLI binary (`aletheia`) |
| `metaygn-verifiers` | Completion and test integrity checkers |
| `metaygn-sandbox` | Process and WASM sandboxing |

## Install on PATH

```bash
cargo install --path crates/cli
```

This installs the `aletheia` binary to `~/.cargo/bin/`.

## Start the daemon

```bash
cargo run -p metaygn-cli -- start
# or, if installed on PATH:
aletheia start
```

The daemon starts on `127.0.0.1:3100` by default. It generates a bearer token at `~/.claude/aletheia/daemon.token` for authenticated API access.

## Verify installation

```bash
aletheia doctor
```

`doctor` checks: daemon reachability, plugin structure, hooks.json validity, Codex assets, Codex MCP registration when the CLI is installed, skills directory, agents directory, and database connectivity.

Expected output:

```
MetaYGN Doctor
==============
Daemon .............. OK (http://127.0.0.1:3100)
Plugin .............. OK (.claude-plugin/plugin.json found)
Hooks ............... OK (hooks.json valid, 8 hooks registered)
Skills .............. OK (skills/ directory found)
Agents .............. OK (agents/ directory found)
Database ............ OK (SQLite accessible)
```

## Run with Claude Code

```bash
claude --plugin-dir .
```

This loads the Aletheia-Nexus plugin. Every tool call from Claude Code now flows through the 5-layer protection cascade.

## Run with Codex

Windows:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-codex-metaygn.ps1
```

macOS/Linux:

```bash
bash ./scripts/start-codex-metaygn.sh
```

This registers `aletheia` as a Codex MCP server if needed, loads the strict MetaYGN bootstrap protocol, and launches Codex against the same runtime.

## Initialize a new project

To scaffold Aletheia-Nexus config in a different project:

```bash
cd /path/to/project
aletheia init
```

Use `--force` to overwrite existing configuration.

## Next steps

- Read the [Architecture](Architecture) page to understand the 5-layer cascade
- See [CLI Reference](CLI-Reference) for all available commands
- Check [API Reference](API-Reference) for direct daemon interaction
