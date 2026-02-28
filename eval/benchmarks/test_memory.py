def test_memory_stats(client, daemon_available):
    """Memory stats endpoint returns event count."""
    r = client.get("/memory/stats")
    assert r.status_code == 200
    body = r.json()
    assert "event_count" in body

def test_memory_recall(client, daemon_available):
    """Memory recall with empty query returns results structure."""
    r = client.post("/memory/recall", json={"query": "test", "limit": 5})
    assert r.status_code == 200
    body = r.json()
    assert "results" in body or "error" in body

def test_health_endpoint(client, daemon_available):
    """Health check returns status ok."""
    r = client.get("/health")
    assert r.status_code == 200
    body = r.json()
    assert body["status"] == "ok"
    assert "version" in body
    assert "kernel" in body
