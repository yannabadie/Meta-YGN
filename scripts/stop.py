#!/usr/bin/env python3
"""Stop hook: nudge proof packet and log session summary."""
from __future__ import annotations

from common import daemon_call, emit_json, log_event, read_stdin_json

payload = read_stdin_json()
log_event("Stop", payload)

# Ask daemon for session summary if available
daemon = daemon_call("/hooks/stop", payload)
if daemon:
    emit_json(daemon)
    raise SystemExit(0)

last_message = payload.get("last_assistant_message", "")

# Check if the response already has proof structure
has_proof = any(
    marker in last_message
    for marker in ["## Goal", "## Changes", "## Evidence", "## Uncertainty", "## Next step"]
)

if not has_proof and len(last_message) > 300:
    emit_json(
        {
            "hookSpecificOutput": {
                "hookEventName": "Stop",
                "additionalContext": (
                    "Reminder: for non-trivial work, finish with a proof packet "
                    "(Goal, Changes, Evidence, Uncertainty, Next step) "
                    "rather than unstructured narration."
                ),
            }
        }
    )
