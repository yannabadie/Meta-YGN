#!/usr/bin/env python3
"""PreCompact hook: guide context compaction with structure."""
from __future__ import annotations

from common import daemon_call, emit_text, log_event, read_stdin_json

payload = read_stdin_json()
trigger = payload.get("trigger", "auto")  # manual or auto
log_event("PreCompact", {"trigger": trigger})

# Notify daemon if available
daemon_call("/hooks/pre-compact", {"trigger": trigger})

emit_text(
    "Compact into these sections:\n"
    "1. Current goal (one sentence)\n"
    "2. Verified facts (evidence-backed only)\n"
    "3. Failed approaches worth remembering (prevent loops)\n"
    "4. Open risks and unresolved questions\n"
    "5. Next best action\n"
    "\n"
    "Drop: repetitive logs, dead-end narration, tool outputs already acted upon, "
    "and any reasoning that led to a discarded approach."
)
