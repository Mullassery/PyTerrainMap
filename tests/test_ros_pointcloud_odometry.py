"""Tests for PointCloud2 decoding and odometry adaptation."""

import json
from types import SimpleNamespace

import numpy as np
import pytest

from pyterrain_map.ros import PyTerrainROSBridge
from pyterrain_map.ros.adapters import LiDARAdapter, OdometryAdapter


def point_field(name, offset, datatype=7, count=1):
    return SimpleNamespace(name=name, offset=offset, datatype=datatype, count=count)


def point_cloud_msg(points, *, intensity=None, row_step=None, width=None, height=1):
    points = np.asarray(points, dtype="<f4")
    if width is None:
        width = len(points)
    fields = [
        point_field("x", 0),
        point_field("y", 4),
        point_field("z", 8),
    ]
    columns = [points[:, 0], points[:, 1], points[:, 2]]
    if intensity is not None:
        fields.append(point_field("intensity", 12))
        columns.append(np.asarray(intensity, dtype="<f4"))
    point_step = 4 * len(columns)
    data = np.column_stack(columns).astype("<f4").tobytes()
    if row_step is None:
        row_step = point_step * width
    return SimpleNamespace(
        header=SimpleNamespace(stamp=SimpleNamespace(sec=1, nanosec=0)),
        fields=fields,
        width=width,
        height=height,
        point_step=point_step,
        row_step=row_step,
        is_bigendian=False,
        data=data,
    )


def odometry_msg(x=1.0, y=2.0, z=0.0, sec=1, nanosec=0):
    return SimpleNamespace(
        header=SimpleNamespace(stamp=SimpleNamespace(sec=sec, nanosec=nanosec), frame_id="odom"),
        child_frame_id="base_link",
        pose=SimpleNamespace(
            pose=SimpleNamespace(
                position=SimpleNamespace(x=x, y=y, z=z),
                orientation=SimpleNamespace(x=0.0, y=0.0, z=0.0, w=1.0),
            )
        ),
        twist=SimpleNamespace(
            twist=SimpleNamespace(
                linear=SimpleNamespace(x=0.5, y=0.0, z=0.0),
                angular=SimpleNamespace(x=0.0, y=0.0, z=0.1),
            )
        ),
    )


class TestPointCloud2:
    def test_voxelizes_points_and_intensity(self):
        adapter = LiDARAdapter(robot_id="r1", frame_id="lidar", voxel_size_m=1.0)
        msg = point_cloud_msg(
            [[0.2, 0.2, 0.1], [0.3, 0.3, 0.2], [2.2, 0.2, 0.1]],
            intensity=[10.0, 20.0, 30.0],
        )

        observations = adapter.on_message(msg)

        assert adapter.error_count == 0
        assert len(observations) == 2
        nearest = min(observations, key=lambda o: o.location_lat)
        value = json.loads(nearest.value_json)
        assert value["points"] == 2
        assert value["intensity"] == pytest.approx(15.0)
        assert value["z_mean"] == pytest.approx(0.15)

    def test_applies_pose_rotation_and_translation(self):
        adapter = LiDARAdapter(robot_id="r1", frame_id="lidar", voxel_size_m=1.0)
        msg = point_cloud_msg([[1.0, 0.0, 0.0]])
        qz = np.sqrt(0.5)
        pose = (10.0, 20.0, 0.0, 0.0, 0.0, qz, qz)

        observations = adapter.on_message(msg, robot_pose=pose)

        assert len(observations) == 1
        assert observations[0].location_lat == pytest.approx(10.0)
        assert observations[0].location_lon == pytest.approx(21.0)

    def test_handles_row_padding(self):
        adapter = LiDARAdapter(robot_id="r1", frame_id="lidar", voxel_size_m=1.0)
        row0 = np.array([[0.1, 0.1, 0.0], [1.1, 0.1, 0.0]], dtype="<f4").tobytes()
        row1 = np.array([[0.1, 2.1, 0.0], [1.1, 2.1, 0.0]], dtype="<f4").tobytes()
        msg = point_cloud_msg(
            [[0.1, 0.1, 0.0], [1.1, 0.1, 0.0], [0.1, 2.1, 0.0], [1.1, 2.1, 0.0]],
            width=2,
            height=2,
            row_step=32,
        )
        msg.data = row0 + b"\x00" * 8 + row1 + b"\x00" * 8

        observations = adapter.on_message(msg)

        assert adapter.error_count == 0
        assert len(observations) == 4

    def test_missing_xyz_field_is_reported_as_error(self):
        adapter = LiDARAdapter(robot_id="r1", frame_id="lidar")
        msg = point_cloud_msg([[1.0, 0.0, 0.0]])
        msg.fields = [point_field("x", 0), point_field("y", 4)]

        observations = adapter.on_message(msg)

        assert observations == []
        assert adapter.error_count == 1


class TestOdometryAdapter:
    def test_emits_pose_and_twist(self):
        adapter = OdometryAdapter(robot_id="r1", frame_id="base_link")
        observations = adapter.on_message(odometry_msg())

        assert adapter.error_count == 0
        assert len(observations) == 1
        obs = observations[0]
        assert obs.sensor_type == "odometry"
        assert obs.confidence == 1.0
        value = json.loads(obs.value_json)
        assert value["frame_id"] == "odom"
        assert value["x"] == 1.0
        assert value["linear"]["x"] == 0.5
        assert value["angular"]["z"] == 0.1

    def test_uses_converter_for_geodetic_location(self):
        bridge = PyTerrainROSBridge(
            {
                "robot_id": "r1",
                "sensors": {
                    "odom": {
                        "adapter": "odom",
                        "topic": "/odom",
                        "frame_id": "base_link",
                    }
                },
            }
        )
        bridge.set_origin(lat=37.7749, lon=-122.4194)

        observations = bridge.on_message("/odom", odometry_msg(x=1.0, y=2.0))

        assert len(observations) == 1
        assert observations[0].location_lat == pytest.approx(37.7749, abs=0.001)
        assert observations[0].location_lon == pytest.approx(-122.4194, abs=0.001)

    def test_bridge_registers_odom_alias(self):
        bridge = PyTerrainROSBridge(
            {
                "robot_id": "r1",
                "sensors": {
                    "odom": {
                        "adapter": "odom",
                        "topic": "/odom",
                        "frame_id": "base_link",
                    }
                },
            }
        )

        assert isinstance(bridge.registry.get("/odom"), OdometryAdapter)
