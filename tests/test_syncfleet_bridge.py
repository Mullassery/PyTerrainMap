"""Real cross-repo test: PyTerrainMap observations -> SyncFleet journal.

Runs the actual ``syncfleet`` binary (built from the sibling repo) against a
temporary node, records observations through the bridge, and verifies the
SQLite ``operation_journal`` rows. Skipped when the binary is unavailable.
"""

from __future__ import annotations

import json
import os
import shutil
import sqlite3
import subprocess
from pathlib import Path

import pytest

from pyterrain_map import Observation
from pyterrain_map.syncfleet import SyncFleetBridge, observation_payload


def _find_syncfleet() -> str | None:
    explicit = os.environ.get("SYNCFLEET_BIN")
    if explicit and Path(explicit).is_file():
        return explicit
    # Repo layout: <Projects>/PyTerrainMap/tests -> <Projects>/SyncFleet/target/release
    candidate = Path(__file__).resolve().parents[2] / "SyncFleet" / "target" / "release" / "syncfleet"
    if candidate.is_file():
        return str(candidate)
    return shutil.which("syncfleet")


SYNCFLEET = _find_syncfleet()

pytestmark = pytest.mark.skipif(
    SYNCFLEET is None,
    reason="syncfleet binary not found (set SYNCFLEET_BIN or build ../SyncFleet)",
)


def _node(tmp_path: Path) -> tuple[Path, Path]:
    data_dir = tmp_path / "data"
    data_dir.mkdir()
    config = tmp_path / "syncfleet.toml"
    config.write_text(
        "[node]\n"
        'node_name = "pyterrainmap-test"\n'
        f'data_dir = "{data_dir}"\n'
        "log_level = \"warn\"\n"
        "[storage]\n"
        "wal_mode = true\n",
        encoding="utf-8",
    )
    subprocess.run(
        [SYNCFLEET, "init", "--node", "pyterrainmap-test", "--config", str(config)],
        check=True,
        capture_output=True,
        text=True,
    )
    return config, data_dir


def test_observations_become_syncfleet_operations(tmp_path: Path) -> None:
    config, data_dir = _node(tmp_path)
    thermal = Observation(
        robot_id="robot-1",
        timestamp=1_700_000_000,
        lat=40.7128,
        lon=-74.0060,
        sensor_type="thermal",
        value_json='{"celsius": 22.5}',
        confidence=0.95,
    )
    lidar = {
        "robot_id": "robot-2",
        "timestamp": 1_700_000_001,
        "lat": 40.7130,
        "lon": -74.0058,
        "sensor_type": "lidar",
        "value_json": '{"range_m": 15.3}',
        "confidence": 0.8,
    }

    bridge = SyncFleetBridge(binary=SYNCFLEET, config=config)
    assert bridge.apply_observations([thermal, lidar]) == 2

    db = data_dir / "syncfleet.db"
    assert db.is_file(), "SyncFleet did not create its SQLite database"
    with sqlite3.connect(db) as con:
        rows = con.execute(
            "SELECT entity_id, payload FROM operation_journal "
            "WHERE dataset_id = ? ORDER BY origin_seq",
            ("terrain",),
        ).fetchall()

    assert len(rows) == 2
    payloads = {json.loads(payload)["robot_id"]: json.loads(payload) for _, payload in rows}
    assert set(payloads) == {"robot-1", "robot-2"}
    assert payloads["robot-1"]["sensor_value"] == {"celsius": 22.5}
    assert payloads["robot-2"]["sensor_value"] == {"range_m": 15.3}
    assert payloads["robot-1"]["latitude"] == pytest.approx(40.7128)
    assert payloads["robot-1"] == observation_payload(thermal)
    # Deterministic entity ids make retries idempotent at the entity level.
    assert len({entity for entity, _ in rows}) == 2
