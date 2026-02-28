#!/usr/bin/env python3
"""SessionStart hook: detect tech stack, initialize Aletheia profile."""
from __future__ import annotations

import os

from common import daemon_call, detect_stack, emit_text, log_event, read_stdin_json

payload = read_stdin_json()
cwd = payload.get("cwd")
source = payload.get("source", "startup")  # startup, resume, clear, compact
stack = detect_stack(cwd if isinstance(cwd, str) else None)
stack_text = ", ".join(stack) if stack else "unknown stack"

# Set ALETHEIA_ACTIVE in env file if available
env_file = os.environ.get("CLAUDE_ENV_FILE")
if env_file:
    try:
        with open(env_file, "a", encoding="utf-8") as fh:
            fh.write("export ALETHEIA_ACTIVE=1\n")
    except OSError:
        pass

log_event("SessionStart", {"source": source, "stack": stack})

# Notify daemon if available
daemon_call("/hooks/session-start", {"source": source, "stack": stack, "cwd": cwd})

if source == "resume":
    emit_text(
        "Aletheia profile resumed. "
        f"Repo signals: {stack_text}. "
        "Review the compact summary above before continuing."
    )
else:
    emit_text(
        "Aletheia profile active. "
        f"Repo signals: {stack_text}. "
        "Prefer preflight, proof, minimal necessary tools, and concise evidence-backed updates."
    )
