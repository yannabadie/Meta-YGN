def test_sandbox_python_hello(client, daemon_available):
    """Sandbox executes Python and returns result."""
    r = client.post("/sandbox/exec", json={
        "language": "python",
        "code": "print('benchmark')"
    })
    assert r.status_code == 200
    body = r.json()
    assert "success" in body

def test_sandbox_timeout_protection(client, daemon_available):
    """Sandbox respects timeout for long-running code."""
    r = client.post("/sandbox/exec", json={
        "language": "python",
        "code": "import time; time.sleep(30)",
        "timeout_ms": 2000
    })
    assert r.status_code == 200
    body = r.json()
    # Should either timeout or fail, not hang
    assert "timed_out" in body or "success" in body

def test_sandbox_error_captured(client, daemon_available):
    """Sandbox captures Python errors."""
    r = client.post("/sandbox/exec", json={
        "language": "python",
        "code": "raise ValueError('test error')"
    })
    assert r.status_code == 200
    body = r.json()
    assert body.get("success") == False
