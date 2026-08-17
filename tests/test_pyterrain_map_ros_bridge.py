"""Tests for the PyTerrainMap ROS bridge and its sensor adapters.

Covers the fix for the previously-broken ROS bridge package: it wasn't
even present in any published wheel (maturin's mixed-layout packaging
never found the real `python/` source tree, so every release silently
shipped only the compiled extension), `bridge.py` didn't exist at all
(`import pyterrain_ros` — its pre-rename location — raised
`ModuleNotFoundError`), and even bypassing that, every LiDAR/thermal
message silently produced zero observations because `StorageObservation`
was missing the `id` field both adapters already constructed it with (a
swallowed `TypeError`). There was also no camera/RGB adapter at all, so
camera topics produced nothing with no error of any kind.
"""

import json
import logging
from types import SimpleNamespace

import numpy as np
import pytest

from pyterrain_map.ros import PyTerrainROSBridge
from pyterrain_map.ros.adapters import LiDARAdapter, ThermalAdapter, RGBAdapter
from pyterrain_map.ros.platforms import SPOT_CONFIG, GENERIC_CONFIG


def ros_image_msg(image: np.ndarray, encoding: str, sec: int = 1, nanosec: int = 0):
    return SimpleNamespace(
        header=SimpleNamespace(stamp=SimpleNamespace(sec=sec, nanosec=nanosec)),
        height=image.shape[0],
        width=image.shape[1],
        encoding=encoding,
        data=image.tobytes(),
    )


def ros_laser_scan_msg(ranges, angle_min=-1.0, angle_increment=0.1, sec=1, nanosec=0):
    return SimpleNamespace(
        header=SimpleNamespace(stamp=SimpleNamespace(sec=sec, nanosec=nanosec)),
        ranges=ranges,
        angle_min=angle_min,
        angle_increment=angle_increment,
    )


class TestStorageObservationIdRegression:
    """The root cause: adapters/base.py's StorageObservation was missing
    `id`, so every adapter that constructed one with `id=...` crashed and
    silently returned []. Guard against regressing that fix.
    """

    def test_lidar_adapter_produces_real_observations(self):
        adapter = LiDARAdapter(robot_id="r1", frame_id="lidar")
        msg = ros_laser_scan_msg(ranges=[1.0, 2.0, 3.0] * 10)
        observations = adapter.on_message(msg)

        assert adapter.error_count == 0
        assert len(observations) > 0
        assert all(hasattr(o, "id") and o.id for o in observations)

    def test_thermal_adapter_produces_real_observations(self):
        adapter = ThermalAdapter(robot_id="r1", frame_id="thermal", grid_size=4)
        image = np.full((32, 32), 30000, dtype=np.uint16)  # mono16
        msg = ros_image_msg(image, encoding="mono16")
        observations = adapter.on_message(msg)

        assert adapter.error_count == 0
        assert len(observations) > 0
        assert all(hasattr(o, "id") and o.id for o in observations)


class TestRGBAdapter:
    def _built_adapter(self, **kwargs):
        return RGBAdapter(robot_id="r1", frame_id="cam", **kwargs)

    def _settle_background(self, adapter, bg, frames=30):
        msg = ros_image_msg(bg, encoding="bgr8")
        for _ in range(frames):
            adapter.on_message(msg)

    def test_detects_real_foreground_object(self):
        adapter = self._built_adapter()
        bg = np.full((120, 160, 3), 50, dtype=np.uint8)
        self._settle_background(adapter, bg)

        frame = bg.copy()
        frame[40:80, 60:110] = 220  # 50x40 bright block at (60, 40)
        observations = adapter.on_message(ros_image_msg(frame, encoding="bgr8"))

        assert len(observations) == 1
        detections = json.loads(observations[0].value_json)["detections"]
        assert len(detections) == 1
        assert detections[0]["class_label"] == "object"
        x, y, w, h = detections[0]["bbox"]
        assert (x, y) == pytest.approx((60, 40), abs=5)
        assert (w, h) == pytest.approx((50, 40), abs=5)

    def test_static_scene_is_a_real_empty_result_not_a_dropped_message(self):
        """An observation IS still emitted with zero detections — that's
        genuinely different from no adapter existing at all (old behavior).
        """
        adapter = self._built_adapter()
        bg = np.full((120, 160, 3), 50, dtype=np.uint8)
        self._settle_background(adapter, bg)

        observations = adapter.on_message(ros_image_msg(bg, encoding="bgr8"))

        assert len(observations) == 1
        assert json.loads(observations[0].value_json)["detections"] == []
        assert adapter.error_count == 0

    def test_undecodable_frame_is_a_real_failure_distinct_from_empty(self):
        adapter = self._built_adapter()
        bad_msg = ros_image_msg(np.zeros((10, 10), dtype=np.uint8), encoding="bgr8")
        bad_msg.encoding = "totally_unsupported_encoding"

        observations = adapter.on_message(bad_msg)

        assert observations == []
        assert adapter.error_count == 1

    def test_sensor_type_matches_storage_convention(self):
        adapter = self._built_adapter()
        assert adapter.sensor_type == "rgb"


class TestPyTerrainROSBridge:
    def test_registers_adapters_from_platform_config(self):
        bridge = PyTerrainROSBridge(SPOT_CONFIG)
        topics = bridge.registry.get_all()

        assert "/scan" in topics
        assert isinstance(topics["/scan"], LiDARAdapter)
        assert "/camera/frontleft/image_raw" in topics
        assert isinstance(topics["/camera/frontleft/image_raw"], RGBAdapter)

    def test_unknown_adapter_key_is_logged_not_silently_dropped(self, caplog):
        with caplog.at_level(logging.WARNING):
            bridge = PyTerrainROSBridge(SPOT_CONFIG)  # SPOT_CONFIG's imu sensor has no adapter impl

        assert "imu" in caplog.text
        assert "/imu/data" not in bridge.registry.get_all()

    def test_on_message_dispatches_to_correct_adapter(self):
        bridge = PyTerrainROSBridge(SPOT_CONFIG)
        msg = ros_laser_scan_msg(ranges=[1.0, 2.0] * 20)

        observations = bridge.on_message("/scan", msg)

        assert len(observations) > 0
        assert bridge.get_stats()["/scan"]["messages_processed"] == 1

    def test_on_message_for_unregistered_topic_logs_and_returns_empty(self, caplog):
        bridge = PyTerrainROSBridge(SPOT_CONFIG)
        with caplog.at_level(logging.WARNING):
            observations = bridge.on_message("/no/such/topic", object())

        assert observations == []
        assert "/no/such/topic" in caplog.text

    def test_attach_to_node_wires_every_topic_plus_tf(self):
        bridge = PyTerrainROSBridge(GENERIC_CONFIG)
        # GENERIC_CONFIG has no sensors; use a config with one topic to keep this concrete.
        bridge = PyTerrainROSBridge({
            "robot_id": "r1",
            "sensors": {"lidar": {"adapter": "lidar", "topic": "/scan", "frame_id": "lidar"}},
        })

        subscribed = []
        bridge.attach_to_node(lambda topic, cb: subscribed.append(topic))

        assert set(subscribed) == {"/tf", "/tf_static", "/scan"}

    def test_set_origin_enables_geodetic_conversion_for_rgb(self):
        bridge = PyTerrainROSBridge({
            "robot_id": "r1",
            "sensors": {"cam": {"adapter": "rgb", "topic": "/cam", "frame_id": "cam"}},
        })
        bridge.set_origin(lat=37.7749, lon=-122.4194)
        bridge.tf_listener.cache.add_transform(SimpleNamespace(
            parent_frame="map", child_frame="cam", timestamp=1_000_000_000,
            x=5.0, y=5.0, z=0.0, qx=0.0, qy=0.0, qz=0.0, qw=1.0,
        ))

        bg = np.full((60, 80, 3), 50, dtype=np.uint8)
        observations = bridge.on_message("/cam", ros_image_msg(bg, encoding="bgr8"), timestamp_ns=1_000_000_000)

        assert len(observations) == 1
        # Near the origin, not left at the (0.0, 0.0) fallback.
        assert observations[0].location_lat == pytest.approx(37.7749, abs=0.01)
        assert observations[0].location_lon == pytest.approx(-122.4194, abs=0.01)
