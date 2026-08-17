"""Regression tests for pyterrain_map.TerrainMap core behavior.

Covers two real bugs found and fixed while integrating the server/query
functions layer:

1. TerrainMap.query() silently returned zero results for *any* query,
   because the longitude-delta calculation passed a latitude in degrees
   straight into `.cos()` (which expects radians), producing a negative
   `lon_delta` for most latitudes and making the `abs(diff) < lon_delta`
   filter false unconditionally.
2. TerrainMap previously had a `.clear()` method that let any caller wipe
   the "immutable, append-only" observation store -- removed as part of
   fixing the immutability contradiction (see docs/DATA_INTEGRITY.md).
"""

import json
import time

from pyterrain_map import GeoPoint, Observation, Region, TerrainMap


def _make_observation(robot_id: str, lat: float, lon: float, timestamp: int) -> Observation:
    return Observation(
        robot_id=robot_id,
        timestamp=timestamp,
        lat=lat,
        lon=lon,
        sensor_type="thermal",
        value_json=json.dumps({"celsius": 22.5}),
        confidence=0.9,
    )


class TestTerrainMapQuery:
    def test_query_finds_observation_at_exact_location(self):
        """Regression test: query() previously always returned 0 results."""
        tm = TerrainMap()
        ts = int(time.time())
        tm.push_observation(_make_observation("bot-1", 40.7128, -74.0060, ts))

        result = tm.query(
            GeoPoint(40.7128, -74.0060), region_radius_km=1.0, time_window_seconds=3600
        )
        assert result.count == 1
        assert len(result.observations) == 1
        assert result.observations[0].robot_id == "bot-1"

    def test_query_works_across_a_range_of_latitudes(self):
        """The old bug (cos() of degrees, not radians) failed differently
        depending on latitude -- check several, including near the equator
        and at high latitude, not just one lucky value."""
        for lat in (0.1, 15.0, 40.7128, 51.5, 89.0):
            tm = TerrainMap()
            ts = int(time.time())
            tm.push_observation(_make_observation("bot-1", lat, -74.0, ts))
            result = tm.query(GeoPoint(lat, -74.0), region_radius_km=5.0, time_window_seconds=3600)
            assert result.count == 1, f"query() found nothing at lat={lat}"

    def test_query_excludes_observations_outside_radius(self):
        tm = TerrainMap()
        ts = int(time.time())
        tm.push_observation(_make_observation("near", 40.7128, -74.0060, ts))
        tm.push_observation(_make_observation("far", 10.0, 10.0, ts))

        result = tm.query(
            GeoPoint(40.7128, -74.0060), region_radius_km=1.0, time_window_seconds=3600
        )
        assert result.count == 1
        assert result.observations[0].robot_id == "near"

    def test_region_stats_reflects_pushed_observations(self):
        tm = TerrainMap()
        ts = int(time.time())
        tm.push_observation(_make_observation("bot-1", 40.7128, -74.0060, ts))
        tm.push_observation(_make_observation("bot-2", 40.7130, -74.0062, ts))

        region = Region(north=40.72, south=40.70, east=-73.99, west=-74.02)
        stats = tm.region_stats(region)
        assert stats["total_observations"] == 2
        assert stats["unique_robots"] == 2


class TestTerrainMapImmutability:
    def test_no_clear_method_exists(self):
        """TerrainMap documents an append-only, immutable observation store
        (docs/DATA_INTEGRITY.md). A previous version exposed `.clear()`,
        directly contradicting that guarantee; it was removed rather than
        gated, since nothing in this codebase legitimately needed it."""
        tm = TerrainMap()
        assert not hasattr(tm, "clear")

    def test_observations_only_grow(self):
        tm = TerrainMap()
        ts = int(time.time())
        assert len(tm) == 0
        tm.push_observation(_make_observation("bot-1", 40.7128, -74.0060, ts))
        assert len(tm) == 1
        tm.push_observation(_make_observation("bot-2", 41.0, -75.0, ts))
        assert len(tm) == 2
        # No API exists to shrink the store back down.
        assert not any(name in dir(tm) for name in ("clear", "reset", "delete", "remove"))
