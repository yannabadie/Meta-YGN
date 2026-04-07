#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NO_LAUNCH=0
USER_PROMPT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-launch)
      NO_LAUNCH=1
      shift
      ;;
    *)
      USER_PROMPT="$*"
      break
      ;;
  esac
done

if ! command -v codex >/dev/null 2>&1; then
  echo "codex CLI not found in PATH." >&2
  exit 1
fi

cd "$ROOT"

# If an HTTP daemon is running, stop it to avoid DB lock/contention with MCP mode.
LATEST_CLI=""
while IFS= read -r -d '' f; do
  if [[ -z "$LATEST_CLI" || "$f" -nt "$LATEST_CLI" ]]; then
    LATEST_CLI="$f"
  fi
done < <(find "$ROOT/target" -type f -name aletheia -print0 2>/dev/null || true)

if [[ -n "$LATEST_CLI" && -x "$LATEST_CLI" ]]; then
  "$LATEST_CLI" stop >/dev/null 2>&1 || true
fi

# Ensure aletheia MCP server exists for Codex.
if ! codex mcp get aletheia >/dev/null 2>&1; then
  echo "MCP server 'aletheia' missing. Installing..."
  bash "$ROOT/scripts/install-codex.sh"
fi

BOOTSTRAP_FILE="$ROOT/docs/codex-bootstrap-prompt.txt"
if [[ ! -f "$BOOTSTRAP_FILE" ]]; then
  echo "Missing bootstrap prompt: $BOOTSTRAP_FILE" >&2
  exit 1
fi

BOOTSTRAP="$(cat "$BOOTSTRAP_FILE")"
if [[ -n "$USER_PROMPT" ]]; then
  BOOTSTRAP="${BOOTSTRAP}\n\nAdditional focus from user: ${USER_PROMPT}"
fi

if [[ "$NO_LAUNCH" == "1" ]]; then
  echo "Codex bootstrap prompt prepared:"
  echo "-----"
  printf "%b\n" "$BOOTSTRAP"
  echo "-----"
  exit 0
fi

codex "$BOOTSTRAP"
