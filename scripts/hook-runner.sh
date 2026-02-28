#!/bin/bash
# Universal hook runner: tries Bun (TS) first, falls back to Python
HOOK_NAME="$1"
PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-$(dirname "$(dirname "$0")")}"

# Convert kebab-case to snake_case for Python
PY_HOOK_NAME=$(echo "$HOOK_NAME" | tr '-' '_')

# Try TypeScript via Bun
if command -v bun &>/dev/null && [ -f "$PLUGIN_ROOT/packages/hooks/src/$HOOK_NAME.ts" ]; then
    exec bun run "$PLUGIN_ROOT/packages/hooks/src/$HOOK_NAME.ts"
fi

# Fall back to Python
if command -v python3 &>/dev/null && [ -f "$PLUGIN_ROOT/scripts/$PY_HOOK_NAME.py" ]; then
    exec python3 "$PLUGIN_ROOT/scripts/$PY_HOOK_NAME.py"
fi

# Last resort: Python without version suffix
if command -v python &>/dev/null && [ -f "$PLUGIN_ROOT/scripts/$PY_HOOK_NAME.py" ]; then
    exec python "$PLUGIN_ROOT/scripts/$PY_HOOK_NAME.py"
fi

# If nothing works, exit 0 (allow) — don't block Claude Code
exit 0
