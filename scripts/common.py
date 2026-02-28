#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import pathlib
import re
import sys
import time
import urllib.error
import urllib.request
from typing import Any


# ---------------------------------------------------------------------------
# I/O helpers
# ---------------------------------------------------------------------------

def read_stdin_json() -> dict[str, Any]:
    raw = sys.stdin.read().strip()
    if not raw:
        return {}
    try:
        data = json.loads(raw)
        if isinstance(data, dict):
            return data
        return {"_value": data}
    except Exception:
        return {"_raw": raw}


def emit_json(payload: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(payload))


def emit_text(text: str) -> None:
    sys.stdout.write(text.strip() + "\n")


# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

def plugin_root() -> pathlib.Path:
    root = os.environ.get("CLAUDE_PLUGIN_ROOT")
    if root:
        return pathlib.Path(root)
    return pathlib.Path(__file__).resolve().parents[1]


def log_dir() -> pathlib.Path:
    path = pathlib.Path.home() / ".claude" / "aletheia"
    path.mkdir(parents=True, exist_ok=True)
    return path


# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

def log_event(kind: str, payload: dict[str, Any]) -> None:
    path = log_dir() / "events.jsonl"
    record = {
        "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "kind": kind,
        "payload": payload,
    }
    try:
        with path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record, ensure_ascii=False) + "\n")
    except OSError:
        pass  # non-critical: never break hook execution over logging


# ---------------------------------------------------------------------------
# Daemon integration
# ---------------------------------------------------------------------------

def daemon_call(route: str, payload: dict[str, Any], timeout: float = 0.35) -> dict[str, Any] | None:
    base = os.environ.get("ALETHEIA_DAEMON_URL", "").strip()
    if not base:
        return None
    url = base.rstrip("/") + route
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            body = resp.read().decode("utf-8")
        if not body.strip():
            return None
        data = json.loads(body)
        return data if isinstance(data, dict) else None
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, ValueError, OSError):
        return None


# ---------------------------------------------------------------------------
# Stack detection
# ---------------------------------------------------------------------------

def detect_stack(cwd: str | None) -> list[str]:
    if not cwd:
        return []
    path = pathlib.Path(cwd)
    hits: list[str] = []
    markers = {
        "Cargo.toml": "rust",
        "go.mod": "go",
        "pom.xml": "java-maven",
        "build.gradle": "java-gradle",
        "build.gradle.kts": "kotlin-gradle",
        "Package.swift": "swift",
        "Gemfile": "ruby",
        "composer.json": "php",
        "mix.exs": "elixir",
        "deno.json": "deno",
        "bun.lockb": "bun",
        "pnpm-lock.yaml": "pnpm",
        "yarn.lock": "yarn",
        "package-lock.json": "npm",
        "package.json": "node",
        "pyproject.toml": "python",
        "requirements.txt": "python",
        "Pipfile": "python",
        "CMakeLists.txt": "cpp-cmake",
        "Makefile": "make",
        "Dockerfile": "docker",
        "docker-compose.yml": "docker-compose",
        "docker-compose.yaml": "docker-compose",
        "terraform.tf": "terraform",
        ".terraform": "terraform",
    }
    for filename, stack in markers.items():
        if (path / filename).exists() and stack not in hits:
            hits.append(stack)
    # Claude plugin detection
    if (path / ".claude-plugin" / "plugin.json").exists():
        hits.append("claude-plugin")
    # .NET detection (multiple possible project files)
    for ext in ("*.csproj", "*.fsproj", "*.sln"):
        if list(path.glob(ext)):
            if "dotnet" not in hits:
                hits.append("dotnet")
            break
    return hits


# ---------------------------------------------------------------------------
# Payload extraction helpers
# ---------------------------------------------------------------------------

def prompt_text(payload: dict[str, Any]) -> str:
    for key in ("prompt", "user_prompt", "input", "text"):
        value = payload.get(key)
        if isinstance(value, str):
            return value
    return ""


def tool_name(payload: dict[str, Any]) -> str:
    for key in ("tool_name", "tool", "name"):
        value = payload.get(key)
        if isinstance(value, str):
            return value
    return ""


def tool_input(payload: dict[str, Any]) -> dict[str, Any]:
    value = payload.get("tool_input")
    return value if isinstance(value, dict) else {}


def changed_path(data: dict[str, Any]) -> str:
    for key in ("file_path", "path", "notebook_path"):
        value = data.get(key)
        if isinstance(value, str):
            return value
    return ""


def tool_response(payload: dict[str, Any]) -> str:
    for key in ("tool_response", "response", "output"):
        value = payload.get(key)
        if isinstance(value, str):
            return value
    return ""


def error_text(payload: dict[str, Any]) -> str:
    for key in ("error", "error_message", "stderr"):
        value = payload.get(key)
        if isinstance(value, str):
            return value
    return ""


# ---------------------------------------------------------------------------
# Security patterns
# ---------------------------------------------------------------------------

DESTRUCTIVE_PATTERNS = [
    r"\brm\s+-rf\s+/(\s|$)",
    r"\bsudo\s+rm\s+-rf\b",
    r"\bmkfs\b",
    r"\bdd\s+if=",
    r"\bshutdown\b",
    r"\breboot\b",
    r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;",        # fork bomb
    r"\bchmod\s+(-R\s+)?777\s+/(\s|$)",               # chmod 777 /
    r"\b>\s*/dev/sda\b",                                # write to raw disk
]

HIGH_RISK_BASH = [
    r"\bgit\s+push\b",
    r"\bgit\s+push\s+--force\b",
    r"\bgit\s+reset\s+--hard\b",
    r"\bgit\s+clean\s+-fd",
    r"\bgit\s+rebase\b",
    r"\bterraform\s+(apply|destroy)\b",
    r"\bkubectl\s+(apply|delete|scale)\b",
    r"\bhelm\s+(upgrade|install|uninstall)\b",
    r"\b(vercel|flyctl|fly|npm|pnpm|cargo)\s+(publish|deploy)\b",
    r"\baws\b.*\b(delete|terminate|update|put)\b",
    r"\bgcloud\b.*\b(delete|deploy)\b",
    r"\baz\b.*\b(delete|create|update)\b",
    r"\bpsql\b.*\b(drop|truncate|alter)\b",
    r"\bmysql\b.*\b(drop|truncate|alter)\b",
    r"\bcurl\b.*\|\s*(ba)?sh\b",                       # curl pipe to shell
    r"\bwget\b.*\|\s*(ba)?sh\b",                       # wget pipe to shell
    r"\bdocker\s+(rm|rmi|system\s+prune|push)\b",
    r"\bnpm\s+install\s+-g\b",
    r"\bpip\s+install\s+--force\b",
    r"\bsudo\s+",
]

SECRET_PATH_PATTERNS = [
    r"(^|/)\.env(\.|$)",
    r"(^|/)\.env\.local$",
    r"(^|/)secrets?(/|$)",
    r"(^|/).*\.pem$",
    r"(^|/).*\.key$",
    r"(^|/).*id_rsa",
    r"(^|/).*id_ed25519",
    r"(^|/)credentials\.json$",
    r"(^|/)service[_-]?account.*\.json$",
    r"(^|/)\.npmrc$",
    r"(^|/)\.pypirc$",
    r"(^|/)\.docker/config\.json$",
    r"(^|/)kubeconfig",
    r"(^|/)\.aws/credentials$",
]


def matches_any(text: str, patterns: list[str]) -> bool:
    return any(re.search(pattern, text) for pattern in patterns)


# ---------------------------------------------------------------------------
# Risk classification
# ---------------------------------------------------------------------------

HIGH_RISK_MARKERS = [
    "auth", "oauth", "token", "secret", "deploy", "payment", "billing",
    "migration", "database", "prod", "production", "security", "permission",
    "mcp", "marketplace", "plugin", "publish", "delete", "encrypt", "decrypt",
    "certificate", "ssl", "tls", "firewall", "infra", "terraform", "k8s",
    "kubernetes", "docker", "ci/cd", "pipeline", "release", "rollback",
]

LOW_RISK_MARKERS = [
    "typo", "rename", "comment", "docs", "readme", "format", "lint",
    "small fix", "whitespace", "style", "log", "todo", "cleanup",
]


def classify_prompt(prompt: str) -> dict[str, str]:
    text = prompt.lower()
    if any(word in text for word in HIGH_RISK_MARKERS):
        return {"risk": "high", "budget": "deliberate", "mode": "verify-first"}
    if any(word in text for word in LOW_RISK_MARKERS):
        return {"risk": "low", "budget": "lean", "mode": "inspect-patch-verify"}
    return {"risk": "medium", "budget": "standard", "mode": "map-plan-patch-verify"}
