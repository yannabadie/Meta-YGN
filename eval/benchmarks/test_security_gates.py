def test_destructive_denied(client, daemon_available):
    """rm -rf / must be denied."""
    r = client.post("/hooks/pre-tool-use", json={
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "rm -rf /"}
    })
    body = r.json()
    decision = body.get("hookSpecificOutput", {}).get("permissionDecision")
    assert decision == "deny"

def test_high_risk_asks(client, daemon_available):
    """git push should require confirmation."""
    r = client.post("/hooks/pre-tool-use", json={
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "git push origin main"}
    })
    body = r.json()
    decision = body.get("hookSpecificOutput", {}).get("permissionDecision")
    assert decision == "ask"

def test_safe_allowed(client, daemon_available):
    """ls -la should be allowed."""
    r = client.post("/hooks/pre-tool-use", json={
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"}
    })
    body = r.json()
    # Allow = empty output or no permissionDecision
    decision = body.get("hookSpecificOutput", {}).get("permissionDecision")
    assert decision is None or decision == "allow"
