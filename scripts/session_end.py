#!/usr/bin/env python3
"""SessionEnd hook: log session close and notify daemon."""
from __future__ import annotations

from common import daemon_call, log_event, read_stdin_json

payload = read_stdin_json()
reason = payload.get("reason", "other")
log_event("SessionEnd", {"reason": reason})

# Notify daemon for session finalization (proof archival, metrics flush)
daemon_call("/hooks/session-end", {"reason": reason}, timeout=1.0)
