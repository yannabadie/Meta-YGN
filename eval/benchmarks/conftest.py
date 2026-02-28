import pytest
import httpx
import os

DAEMON_PORT = os.environ.get("METAYGN_PORT", "9000")
DAEMON_URL = f"http://127.0.0.1:{DAEMON_PORT}"

@pytest.fixture
def client():
    """HTTP client for daemon API."""
    return httpx.Client(base_url=DAEMON_URL, timeout=5.0)

@pytest.fixture
def daemon_available(client):
    """Skip test if daemon is not running."""
    try:
        r = client.get("/health")
        if r.status_code != 200:
            pytest.skip("Daemon not running")
    except httpx.ConnectError:
        pytest.skip("Daemon not running")
