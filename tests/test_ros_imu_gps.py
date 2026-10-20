"""Tests for IMU and GPS adapters."""

import json
import math
from types import SimpleNamespace

import pytest
from pyterrain_map.ros import PyTerrainROSBridge
from pyterrain_map.ros.adapters import ADAPTER_CLASSES, GPSAdapter, IMUAdapter
from pyterrain_map.ros.platforms import SPOT_CONFIG, WARTHOG_CONFIG
from pyterrain_map.ros.transforms.coordinate_frames import CoordinateConverter


def vector3(x=0.0, y=0.0, z=0.0):
    return SimpleNamespace(x=x, y=y, z=z)


def quaternion(x=0.0, y=0.0, z=0.0, w=1.0):
    return SimpleNamespace(x=x, y=y, z=z, w=w)


def covariance(value=0.0):
    return [value] + [0.0] * 8


def imu_msg(*, sec=1, nanosec=500_000_000):
    return SimpleNamespace(
        header=SimpleNamespace(
            stamp=SimpleNamespace(sec=sec, nanosec=nanosec),
            frame_id="imu_link",
        ),
        orientation=quaternion(z=0.1, w=0.995),
        orientation_covariance=covariance(0.01),
        angular_velocity=vector3(0.1, 0.2, 0.3),
        angular_velocity_covariance=covariance(0.02),
        linear_acceleration=vector3(0.0, 0.0, 9.81),
        linear_acceleration_covariance=covariance(0.03),
    )


def gps_msg(*, latitude=37.7749, longitude=-122.4194, altitude=10.0, status=0):
    return SimpleNamespace(
        header=SimpleNamespace(
            stamp=SimpleNamespace(sec=2, nanosec=250_000_000),
            frame_id="gps_link",
        ),
        status=SimpleNamespace(status=status, service=1),
        latitude=latitude,
        longitude=longitude,
        altitude=altitude,
        position_covariance=covariance(1.5),
        position_covariance_type=2,
    )


class TestIMUAdapter:
    def test_emits_orientation_inertial_values_and_covariances(self):
        adapter = IMUAdapter(robot_id="r1", frame_id="imu_link")

        observations = adapter.on_message(
            imu_msg(), robot_pose=(10.0, 20.0, 0.0, 0.0, 0.0, 0.0, 1.0)
        )

        assert adapter.error_count == 0
        assert len(observations) == 1
        obs = observations[0]
        assert obs.sensor_type == "imu"
        assert obs.timestamp == 1_500_000
        assert obs.location_lat == 10.0
        assert obs.location_lon == 20.0
        value = json.loads(obs.value_json)
        assert value["frame_id"] == "imu_link"
        assert value["orientation"]["z"] == 0.1
        assert value["angular_velocity"]["z"] == 0.3
        assert value["linear_acceleration"]["z"] == 9.81
        assert value["orientation_covariance"][0] == 0.01
        assert value["location_source"] == "local"

    def test_converts_robot_pose_to_geodetic(self):
        adapter = IMUAdapter(robot_id="r1", frame_id="imu_link")
        converter = CoordinateConverter(37.7749, -122.4194)

        observations = adapter.on_message(
            imu_msg(),
            robot_pose=(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
            converter=converter,
        )

        assert len(observations) == 1
        assert observations[0].location_lat == pytest.approx(37.7749, abs=0.001)
        assert observations[0].location_lon == pytest.approx(-122.4194, abs=0.001)
        assert json.loads(observations[0].value_json)["location_source"] == "geodetic"

    def test_missing_orientation_is_an_error(self):
        adapter = IMUAdapter(robot_id="r1", frame_id="imu_link")
        msg = imu_msg()
        msg.orientation = None

        assert adapter.on_message(msg) == []
        assert adapter.error_count == 1

    def test_non_finite_quaternion_is_an_error(self):
        adapter = IMUAdapter(robot_id="r1", frame_id="imu_link")
        msg = imu_msg()
        msg.orientation.x = math.nan

        assert adapter.on_message(msg) == []
        assert adapter.error_count == 1


class TestGPSAdapter:
    def test_emits_navsatfix_fields(self):
        adapter = GPSAdapter(robot_id="r1", frame_id="gps_link")

        observations = adapter.on_message(gps_msg())

        assert adapter.error_count == 0
        assert len(observations) == 1
        obs = observations[0]
        assert obs.sensor_type == "gps"
        assert obs.timestamp == 2_250_000
        assert obs.location_lat == pytest.approx(37.7749)
        assert obs.location_lon == pytest.approx(-122.4194)
        assert obs.confidence == 1.0
        value = json.loads(obs.value_json)
        assert value["frame_id"] == "gps_link"
        assert value["altitude"] == 10.0
        assert value["status"] == {"status": 0, "service": 1}
        assert value["position_covariance"][0] == 1.5
        assert value["position_covariance_type"] == 2
        assert value["location_source"] == "navsatfix"

    def test_no_fix_is_dropped(self):
        adapter = GPSAdapter(robot_id="r1", frame_id="gps_link")

        assert adapter.on_message(gps_msg(status=-1)) == []
        assert adapter.error_count == 1

    def test_non_finite_coordinates_are_an_error(self):
        adapter = GPSAdapter(robot_id="r1", frame_id="gps_link")

        assert adapter.on_message(gps_msg(latitude=math.nan)) == []
        assert adapter.error_count == 1


@pytest.mark.parametrize("alias", ["gps", "navsat", "navsatfix"])
def test_gps_registry_aliases(alias):
    assert ADAPTER_CLASSES[alias] is GPSAdapter


def test_imu_registry_alias():
    assert ADAPTER_CLASSES["imu"] is IMUAdapter


def test_platform_configs_resolve_imu_and_gps():
    spot = PyTerrainROSBridge(SPOT_CONFIG)
    warthog = PyTerrainROSBridge(WARTHOG_CONFIG)

    assert isinstance(spot.registry.get("/imu/data"), IMUAdapter)
    assert isinstance(warthog.registry.get("/imu/data"), IMUAdapter)
    assert isinstance(warthog.registry.get("/gps/fix"), GPSAdapter)
