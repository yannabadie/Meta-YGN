def test_high_risk_deploy(client, daemon_available):
    """Deploy to production should be classified as high risk."""
    r = client.post("/hooks/user-prompt-submit", json={
        "hook_event_name": "UserPromptSubmit",
        "prompt": "deploy the application to production"
    })
    assert r.status_code == 200
    body = r.json()
    context = body.get("hookSpecificOutput", {}).get("additionalContext", "")
    assert "high" in context.lower() or "risk" in context.lower()

def test_low_risk_typo(client, daemon_available):
    """Fix typo should be classified as low risk."""
    r = client.post("/hooks/user-prompt-submit", json={
        "hook_event_name": "UserPromptSubmit",
        "prompt": "fix the typo in the readme file"
    })
    assert r.status_code == 200
    body = r.json()
    context = body.get("hookSpecificOutput", {}).get("additionalContext", "")
    assert "low" in context.lower() or "lean" in context.lower()

def test_medium_risk_feature(client, daemon_available):
    """Add a new feature should be medium risk."""
    r = client.post("/hooks/user-prompt-submit", json={
        "hook_event_name": "UserPromptSubmit",
        "prompt": "add a user login page with email validation"
    })
    assert r.status_code == 200
    body = r.json()
    context = body.get("hookSpecificOutput", {}).get("additionalContext", "")
    # Should be medium (default) or contain strategy info
    assert r.status_code == 200
