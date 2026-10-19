"""Real cross-repo test: PyTerrainMap -> StatGuardian ``/validate``.

Starts an actual ``statguardian serve`` process, validates a real
``Observation``, and checks block/unreachable behavior. Skipped when the
StatGuardian CLI is unavailable — it never substitutes a fake for the service.
"""

from __future__ import annotations

import os
import shutil
import socket
import subprocess
import time
from pathlib import Path

import pytest

from pyterrain_map import Observation
from pyterrain_map.statguardian_client import (
    StatGuardianQualityClient,
    StatGuardianUnavailable,
)


def _find_statguardian() -> str | None:
    explicit = os.environ.get("STATGUARDIAN_BIN")
    if explicit and Path(explicit).is_file():
        return explicit
    projects = Path(__file__).resolve().parents[2]
    for candidate in (
        projects / "statguardian" / ".venv" / "bin" / "statguardian",
        projects / "StatGuardian" / ".venv" / "bin" / "statguardian",
    ):
        if candidate.is_file():
            return str(candidate)
    return shutil.which("statguardian")


STATGUARDIAN = _find_statguardian()

pytestmark = pytest.mark.skipif(
    STATGUARDIAN is None,
    reason="statguardian CLI not found (set STATGUARDIAN_BIN or install statguardian[api])",
)

CONTRACT = """dataset observation {
    schema {
        robot_id:    string, not_null
        timestamp:   int,    not_null
        latitude:    float
        longitude:   float
        sensor_type: string, not_null
        confidence:  float
    }
}
"""


def _free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _wait(port: int, timeout: float = 15.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                return
        except OSError:
            time.sleep(0.1)
    raise AssertionError(f"StatGuardian did not listen on {port}")


def test_validate_observation_against_live_service(tmp_path: Path) -> None:
    contract = tmp_path / "observation.sg"
    contract.write_text(CONTRACT, encoding="utf-8")
    port = _free_port()
    process = subprocess.Popen(
        [
            STATGUARDIAN,
            "serve",
            "--contract",
            str(contract),
            "--host",
            "127.0.0.1",
            "--port",
            str(port),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        _wait(port)
        client = StatGuardianQualityClient(f"http://127.0.0.1:{port}", timeout=10)
        assert client.is_available() is True

        good = Observation(
            robot_id="robot-1",
            timestamp=1_700_000_000,
            lat=40.7128,
            lon=-74.0060,
            sensor_type="thermal",
            value_json='{"celsius": 22.5}',
            confidence=0.95,
        )
        result = client.validate_observation(good)
        assert result["passed"] is True, result

        bad = {
            "robot_id": None,  # violates not_null
            "timestamp": 1_700_000_001,
            "latitude": 40.0,
            "longitude": -74.0,
            "sensor_type": "thermal",
            "confidence": 0.5,
        }
        result = client.validate_observation(bad)
        assert result["passed"] is False, result

        with pytest.raises(StatGuardianUnavailable):
            StatGuardianQualityClient("http://127.0.0.1:9", timeout=1).validate_observation(good)
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
