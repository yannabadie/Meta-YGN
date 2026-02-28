#!/usr/bin/env python3
"""PostToolUseFailure hook: capture error signals to guide recovery."""
from __future__ import annotations

from common import emit_json, error_text, log_event, read_stdin_json, tool_input, tool_name

payload = read_stdin_json()
log_event("PostToolUseFailure", payload)

name = tool_name(payload)
data = tool_input(payload)
error = error_text(payload)
is_interrupt = payload.get("is_interrupt", False)

if is_interrupt:
    # User interrupted -- do not add noise
    raise SystemExit(0)

context_parts = []

if name == "Bash":
    command = str(data.get("command", ""))
    context_parts.append(
        f"Bash command failed: `{command[:120]}`. "
        "Diagnose before retrying. Check: wrong directory, missing dependency, "
        "syntax error, or permission issue."
    )
    if error:
        short_error = error[:200].replace("\n", " ")
        context_parts.append(f"Error hint: {short_error}")

elif name in {"Write", "Edit", "MultiEdit"}:
    path = str(data.get("file_path", "unknown"))
    context_parts.append(
        f"File operation failed on `{path}`. "
        "Check: file exists, old_string matches exactly, correct indentation."
    )

elif name.startswith("mcp__"):
    context_parts.append(
        f"MCP tool `{name}` failed. "
        "Consider: is the MCP server running? Is the input schema correct? "
        "Fall back to local CLI if possible."
    )

else:
    context_parts.append(
        f"Tool `{name}` failed. Review the error and adjust the approach "
        "rather than retrying the same call."
    )

if context_parts:
    emit_json(
        {
            "hookSpecificOutput": {
                "hookEventName": "PostToolUseFailure",
                "additionalContext": " ".join(context_parts),
            }
        }
    )
