"""End-to-end tests for the real HTTP/HTTPS API server (pyterrain_map.start_server).

Prior to this, `src/api/mod.rs` and `src/api_tls/mod.rs` defined a fully-typed
REST API that was never bound to an actual running server. These tests start
a real server on a real (OS-assigned) TCP port and make real HTTP/HTTPS
requests against it over real sockets -- this is what proves the API is
actually reachable, not just type-correct.
"""

import json
import socket
import ssl
import urllib.error
import urllib.request

import pytest

import pyterrain_map as ptm


def _free_port() -> int:
    """Ask the OS for a free ephemeral port, then release it immediately.

    There's a small window where another process could grab the same port
    before start_server() binds it, but that's true of any "find a free
    port" strategy without a shared broker; acceptable for tests.
    """
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


@pytest.fixture
def http_server():
    port = _free_port()
    handle = ptm.start_server(host="127.0.0.1", port=port, tls=False)
    try:
        yield handle, port
    finally:
        handle.stop()


@pytest.fixture
def https_server():
    port = _free_port()
    handle = ptm.start_server(host="127.0.0.1", port=port, tls=True)
    try:
        yield handle, port
    finally:
        handle.stop()


class TestHttpServer:
    def test_handle_reports_running_immediately(self, http_server):
        handle, port = http_server
        assert handle.is_running()
        assert handle.port == port
        assert handle.tls is False

    def test_health_endpoint_returns_real_response(self, http_server):
        _, port = http_server
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=5) as resp:
            assert resp.status == 200
            body = json.loads(resp.read())
        assert body["status"] == "ok"
        assert body["observations_stored"] == 0
        assert "version" in body

    def test_submit_then_query_round_trip(self, http_server):
        _, port = http_server
        payload = json.dumps(
            {
                "robot_id": "bot-1",
                "timestamp": 1_700_000_000_000_000,
                "latitude": 40.7128,
                "longitude": -74.0060,
                "elevation": 10.0,
                "sensor_type": "thermal",
                "sensor_value": {"celsius": 22.5},
                "confidence": 0.9,
                "metadata": {},
            }
        ).encode()
        req = urllib.request.Request(
            f"http://127.0.0.1:{port}/observations",
            data=payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=5) as resp:
            assert resp.status == 201
            submitted = json.loads(resp.read())
        assert submitted["status"] == "stored"

        query_payload = json.dumps(
            {
                "latitude": 40.7128,
                "longitude": -74.0060,
                "radius_m": 1000.0,
                "elevation_min": None,
                "elevation_max": None,
            }
        ).encode()
        req = urllib.request.Request(
            f"http://127.0.0.1:{port}/query/spatial",
            data=query_payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=5) as resp:
            assert resp.status == 200
            results = json.loads(resp.read())
        assert len(results) == 1
        assert results[0]["robot_id"] == "bot-1"
        assert results[0]["id"] == submitted["id"]

        with urllib.request.urlopen(f"http://127.0.0.1:{port}/stats", timeout=5) as resp:
            stats = json.loads(resp.read())
        assert stats["total_observations"] == 1
        assert stats["by_sensor_type"]["thermal"] == 1

    def test_invalid_observation_is_rejected_with_400(self, http_server):
        _, port = http_server
        payload = json.dumps(
            {
                "robot_id": "bot-1",
                "timestamp": 1000,
                "latitude": 999.0,  # invalid
                "longitude": -74.0060,
                "elevation": None,
                "sensor_type": "thermal",
                "sensor_value": {"celsius": 22.5},
                "confidence": 0.9,
                "metadata": {},
            }
        ).encode()
        req = urllib.request.Request(
            f"http://127.0.0.1:{port}/observations",
            data=payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with pytest.raises(urllib.error.HTTPError) as exc_info:
            urllib.request.urlopen(req, timeout=5)
        assert exc_info.value.code == 400

    def test_unknown_route_returns_404(self, http_server):
        _, port = http_server
        with pytest.raises(urllib.error.HTTPError) as exc_info:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/nope", timeout=5)
        assert exc_info.value.code == 404

    def test_stop_actually_stops_the_server(self, http_server):
        handle, port = http_server
        handle.stop()
        assert not handle.is_running()
        with pytest.raises(OSError):
            urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=2)


class TestHttpsServer:
    def test_real_tls_handshake_and_request(self, https_server):
        """Connects with a real TLS client and makes a real HTTPS request.

        Uses an unverified context because the server uses a freshly
        generated self-signed dev certificate (see src/server.rs
        generate_dev_certificate docs) -- that's expected for local
        dev/test, not a production trust model. tests/server_integration.rs
        covers the stronger case of a client that explicitly trusts the
        exact generated certificate (real certificate validation, not a
        bypass).
        """
        handle, port = https_server
        assert handle.tls is True

        ctx = ssl._create_unverified_context()
        with urllib.request.urlopen(
            f"https://127.0.0.1:{port}/health", timeout=5, context=ctx
        ) as resp:
            assert resp.status == 200
            body = json.loads(resp.read())
        assert body["status"] == "ok"
