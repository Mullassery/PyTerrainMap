"""Tests for `pytm query` and the local observation store it reads.

`pytm query` previously printed "Query command - Not yet implemented" and
returned exit code 1. It now performs a real query against the local NDJSON
observation store via `LocalStorageBackend`.

These tests also pin down a pre-existing bug that made the entire
`pyterrain_map.storage` package unimportable: `storage/base.py` registered
its backends at import time (`_register_backends()`), which produced a
circular import (`base -> adls -> base`) whenever `storage/__init__.py` was
imported. Registration was moved into `storage/__init__.py` after each
backend module is imported.

The CLI drives the query with `asyncio.run`, matching how it is actually
invoked (no running event loop), so these tests are synchronous and seed
the store via `asyncio.run` as well.
"""

import asyncio
import json
import time
from types import SimpleNamespace

import pytest
from pyterrain_map import cli
from pyterrain_map.storage import StorageFactory
from pyterrain_map.storage.base import StorageObservation
from pyterrain_map.storage.local import LocalStorageBackend

NOW = int(time.time() * 1_000_000)


def _obs(obs_id, robot_id="robot-1", ts=None, lat=40.1, lon=-74.5, sensor="thermal", conf=0.9):
    return StorageObservation(
        id=obs_id,
        robot_id=robot_id,
        timestamp=ts if ts is not None else NOW,
        location_lat=lat,
        location_lon=lon,
        sensor_type=sensor,
        value_json='{"temp_c": 22.5}',
        confidence=conf,
    )


async def _seed(data_dir, observations):
    backend = LocalStorageBackend({"base_path": str(data_dir)})
    await backend.connect()
    written = await backend.write_batch(observations)
    assert written == len(observations)


def _args(data_dir, **overrides):
    defaults = dict(
        config_dir=None,
        data_dir=str(data_dir),
        robot_id=None,
        sensor_type=None,
        start_time=None,
        end_time=None,
        lat=None,
        lon=None,
        radius_km=None,
        limit=100,
        json=False,
        verbose=False,
    )
    defaults.update(overrides)
    return SimpleNamespace(**defaults)


def test_storage_package_imports_and_registers_backends():
    assert StorageFactory.list_backends() == ["local", "s3", "gcs", "adls"]


def test_parse_timestamp_accepts_micros_iso_and_date():
    assert cli._parse_timestamp(None) is None
    assert cli._parse_timestamp("") is None
    assert cli._parse_timestamp("1700000000000000") == 1_700_000_000_000_000
    assert cli._parse_timestamp("2023-11-14T22:13:20Z") == 1_700_000_000_000_000
    assert cli._parse_timestamp("1970-01-01") == 0


def test_query_all_observations(tmp_path, capsys):
    asyncio.run(_seed(tmp_path, [_obs("a"), _obs("b", sensor="lidar", conf=0.7)]))
    rc = cli.query_command(_args(tmp_path))
    assert rc == 0
    out = capsys.readouterr().out
    assert "Found 2 observation(s)" in out
    assert "robot-1" in out


def test_query_json_output_has_count_and_confidence(tmp_path, capsys):
    asyncio.run(_seed(tmp_path, [_obs("a", conf=0.8), _obs("b", conf=1.0)]))
    rc = cli.query_command(_args(tmp_path, json=True))
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    assert payload["count"] == 2
    assert payload["avg_confidence"] == pytest.approx(0.9)
    assert len(payload["observations"]) == 2


def test_query_filters_sensor_and_robot(tmp_path, capsys):
    rows = [
        _obs("a", sensor="thermal"),
        _obs("b", sensor="lidar"),
        _obs("c", robot_id="robot-2", sensor="thermal"),
    ]
    asyncio.run(_seed(tmp_path, rows))
    rc = cli.query_command(_args(tmp_path, sensor_type="thermal", robot_id="robot-1"))
    assert rc == 0
    assert "Found 1 observation(s)" in capsys.readouterr().out


def test_query_radius_excludes_far_observations(tmp_path, capsys):
    rows = [
        _obs("near", lat=40.1005, lon=-74.5005),
        _obs("far", lat=41.0, lon=-75.0),
    ]
    asyncio.run(_seed(tmp_path, rows))
    rc = cli.query_command(_args(tmp_path, lat=40.1, lon=-74.5, radius_km=1.0))
    assert rc == 0
    assert "Found 1 observation(s)" in capsys.readouterr().out


def test_query_time_window(tmp_path, capsys):
    rows = [
        _obs("old", ts=NOW - 10 * 60 * 1_000_000),
        _obs("new", ts=NOW - 60 * 1_000_000),
    ]
    asyncio.run(_seed(tmp_path, rows))
    cutoff = int((time.time() - 5 * 60) * 1_000_000)
    rc = cli.query_command(_args(tmp_path, start_time=str(cutoff), json=True))
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    assert payload["count"] == 1


def test_query_missing_lon_is_error(tmp_path, capsys):
    rc = cli.query_command(_args(tmp_path, lat=40.1))
    assert rc == 2
    assert "--lat and --lon must be provided together" in capsys.readouterr().err


def test_query_empty_store_returns_zero(tmp_path, capsys):
    rc = cli.query_command(_args(tmp_path))
    assert rc == 0
    assert "No observations found" in capsys.readouterr().out


def test_query_invalid_time_is_error(tmp_path, capsys):
    rc = cli.query_command(_args(tmp_path, start_time="not-a-date", end_time="not-a-date"))
    assert rc == 2
    assert "Invalid time value" in capsys.readouterr().err
