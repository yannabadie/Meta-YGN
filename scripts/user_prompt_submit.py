#!/usr/bin/env python3
"""UserPromptSubmit hook: classify prompt risk and suggest preflight strategy."""
from __future__ import annotations

from common import classify_prompt, daemon_call, emit_text, log_event, prompt_text, read_stdin_json

payload = read_stdin_json()
prompt = prompt_text(payload)
classification = classify_prompt(prompt)

log_event("UserPromptSubmit", {"prompt_length": len(prompt), "classification": classification})

# Daemon may override classification with learned heuristics
daemon = daemon_call("/hooks/user-prompt-submit", {"prompt": prompt, "classification": classification})
if daemon and "classification" in daemon:
    classification = daemon["classification"]

risk = classification["risk"]
budget = classification["budget"]
mode = classification["mode"]

if risk == "high":
    emit_text(
        f"Preflight: risk={risk}, budget={budget}, mode={mode}. "
        "HIGH RISK detected. Run /metacog-preflight or /metacog-threat-model before acting. "
        "Name the proof plan and whether each tool call is necessary."
    )
elif risk == "medium":
    emit_text(
        f"Preflight: risk={risk}, budget={budget}, mode={mode}. "
        "Before acting, name the proof plan and whether any tool call is actually necessary."
    )
else:
    emit_text(
        f"Preflight: risk={risk}, budget={budget}, mode={mode}. "
        "Lean workflow. Verify after patching."
    )
