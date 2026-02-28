# Product Context

## Overview
Aletheia-Nexus is a Claude Code plugin that implements a metacognitive layer for AI coding agents. It intercepts the agent lifecycle (session start, prompt submission, tool use, response completion) to classify risk, gate dangerous operations, enforce verification, and maintain structured proof packets.

The plugin is intentionally thin: hooks, skills, agents, and output styles. The heavy cognitive logic is designed to live in an external daemon (`ALETHEIA_DAEMON_URL`) with graceful fallback to local heuristics.

## Core Features

### Hooks (8 lifecycle events)
- Risk classification on every user prompt (high/medium/low)
- Security gates on every tool call (deny/ask/allow)
- Verification signals after successful tool execution
- Error diagnosis after failed tool execution
- Proof packet enforcement at response completion
- Structured context compaction guidance

### Skills (8 metacognitive workflows)
- Preflight risk assessment, proof packet building, assumption challenging
- Threat modeling, tool necessity auditing, context compaction
- Quality benchmarking, structured escalation

### Agents (6 specialized roles)
- Default executor, skeptic, verifier, researcher, cartographer, cost auditor

### Output Style
- Proof packet format: Goal, Changes, Evidence, Uncertainty, Next step

## Technical Stack
- **Python 3** (stdlib only) for hook handlers
- **Markdown** for skills, agents, docs (native Claude Code format)
- **JSON** for plugin config, hooks, settings
- **JSONL** for event logging (~/.claude/aletheia/events.jsonl)
- **Optional**: Local daemon via HTTP (350ms timeout, graceful fallback)
