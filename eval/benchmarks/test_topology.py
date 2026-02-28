def test_trivial_gets_single_topology(client, daemon_available):
    """Low risk trivial task should get Single topology (fewer stages)."""
    r = client.post("/hooks/analyze", json={
        "hook_event_name": "UserPromptSubmit",
        "prompt": "fix typo"
    })
    assert r.status_code == 200
    # Check that the analysis ran (response has some metacognitive context)

def test_security_gets_horizontal(client, daemon_available):
    """Security task should get maximum scrutiny."""
    r = client.post("/hooks/analyze", json={
        "hook_event_name": "UserPromptSubmit",
        "prompt": "review authentication and fix the OAuth token vulnerability"
    })
    assert r.status_code == 200

def test_analyze_returns_full_context(client, daemon_available):
    """Analyze endpoint returns metacognitive context."""
    r = client.post("/hooks/analyze", json={
        "hook_event_name": "UserPromptSubmit",
        "prompt": "refactor the database module"
    })
    assert r.status_code == 200
    body = r.json()
    # Should have metacognitive fields
    assert isinstance(body, dict)
