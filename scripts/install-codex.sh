#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-debug}"

if ! command -v codex >/dev/null 2>&1; then
  echo "codex CLI not found in PATH." >&2
  exit 1
fi

cd "$ROOT"

# Stop daemon if running to avoid binary lock errors during rebuild.
LATEST_CLI=""
while IFS= read -r -d '' f; do
  if [[ -z "$LATEST_CLI" || "$f" -nt "$LATEST_CLI" ]]; then
    LATEST_CLI="$f"
  fi
done < <(find "$ROOT/target" -type f -name aletheia -print0 2>/dev/null || true)

if [[ -n "$LATEST_CLI" && -x "$LATEST_CLI" ]]; then
  "$LATEST_CLI" stop >/dev/null 2>&1 || true
fi

if [[ "$MODE" == "release" ]]; then
  cargo build -p metaygn-daemon -p metaygn-cli --release --features mcp
  PROFILE="release"
else
  cargo build -p metaygn-daemon -p metaygn-cli --features mcp
  PROFILE="debug"
fi

BIN=""
while IFS= read -r -d '' f; do
  if [[ -z "$BIN" || "$f" -nt "$BIN" ]]; then
    BIN="$f"
  fi
done < <(find "$ROOT/target" -type f -name aletheia -path "*/$PROFILE/aletheia" -print0 2>/dev/null || true)

if [[ -z "$BIN" || ! -x "$BIN" ]]; then
  echo "aletheia binary not found under target/*/$PROFILE/aletheia" >&2
  exit 1
fi

if codex mcp get aletheia >/dev/null 2>&1; then
  codex mcp remove aletheia >/dev/null 2>&1 || true
fi

codex mcp add aletheia -- "$BIN" mcp

echo "MetaYGN registered as Codex MCP server 'aletheia'."
echo "Verify with: codex mcp list"
