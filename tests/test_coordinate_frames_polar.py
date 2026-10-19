"""Boundary tests for ENU <-> WGS84 conversion at extreme latitudes.

The ROS bridge's ``CoordinateConverter`` already implements the transform;
these tests cover the polar/high-latitude cases that were previously untested.
"""

from __future__ import annotations

import pytest

from pyterrain_map.ros.transforms.coordinate_frames import (
    CoordinateConverter,
    ENUPoint,
    GeoPoint,
)


@pytest.mark.parametrize("lat", [0.0, 45.0, 89.9, -89.9, 89.999, -89.999])
def test_enu_geodetic_round_trip_at_extreme_latitudes(lat: float) -> None:
    converter = CoordinateConverter(origin_lat=lat, origin_lon=10.0, origin_alt=0.0)
    enu = ENUPoint(east=100.0, north=50.0, up=2.0)

    geo = converter.enu_to_geodetic(enu)
    back = converter.geodetic_to_enu(geo)

    assert back.east == pytest.approx(enu.east, abs=0.5)
    assert back.north == pytest.approx(enu.north, abs=0.5)
    assert back.up == pytest.approx(enu.up, abs=0.5)


@pytest.mark.parametrize("lat", [89.999, -89.999])
def test_origin_maps_to_zero_enu_at_poles(lat: float) -> None:
    converter = CoordinateConverter(origin_lat=lat, origin_lon=10.0)
    back = converter.geodetic_to_enu(GeoPoint(lat=lat, lon=10.0, alt=0.0))

    assert back.east == pytest.approx(0.0, abs=1e-6)
    assert back.north == pytest.approx(0.0, abs=1e-6)
    assert back.up == pytest.approx(0.0, abs=1e-6)
