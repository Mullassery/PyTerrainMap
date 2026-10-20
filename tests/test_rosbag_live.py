"""Opt-in replay of a real ROS 2 bag through the bridge."""

from __future__ import annotations

import os
from collections import Counter
from pathlib import Path

import pytest
from pyterrain_map.ros import PyTerrainROSBridge

ROS_BAG = os.environ.get("ROS_BAG")
ROS_BAG_ORIGIN = os.environ.get("ROS_BAG_ORIGIN")

ADAPTER_BY_TYPE = {
    "sensor_msgs/msg/PointCloud2": "lidar",
    "sensor_msgs/msg/LaserScan": "lidar",
    "nav_msgs/msg/Odometry": "odometry",
    "sensor_msgs/msg/Imu": "imu",
    "sensor_msgs/msg/NavSatFix": "gps",
}


def _discover_sensors(reader):
    sensors = {}
    for connection in reader.connections:
        adapter = ADAPTER_BY_TYPE.get(connection.msgtype)
        if adapter is None:
            continue
        key = connection.topic.strip("/").replace("/", "_") or "sensor"
        while key in sensors:
            key = f"{key}_"
        sensors[key] = {
            "adapter": adapter,
            "topic": connection.topic,
            "frame_id": "",
        }
    return sensors


@pytest.mark.skipif(not ROS_BAG, reason="set ROS_BAG to a recorded ROS 2 bag")
def test_live_rosbag_replay():
    AnyReader = pytest.importorskip("rosbags.highlevel").AnyReader
    bag_path = Path(ROS_BAG)
    assert bag_path.exists()

    with AnyReader([bag_path]) as reader:
        sensors = _discover_sensors(reader)
        assert sensors, "bag contains no supported PointCloud2/LaserScan/Odometry/Imu/NavSatFix topics"

        bridge = PyTerrainROSBridge(
            {
                "robot_id": os.environ.get("ROS_BAG_ROBOT_ID", "rosbag_live"),
                "base_frame": "base_link",
                "reference_frame": "map",
                "sensors": sensors,
            }
        )
        if ROS_BAG_ORIGIN:
            values = [float(value) for value in ROS_BAG_ORIGIN.split(",")]
            bridge.set_origin(values[0], values[1], values[2] if len(values) > 2 else 0.0)

        counts = Counter()
        for connection, timestamp, rawdata in reader.messages():
            if connection.topic in ("/tf", "/tf_static"):
                message = reader.deserialize(rawdata, connection.msgtype)
                bridge.on_tf_message(message.transforms)
                continue
            if connection.topic not in bridge.registry.get_all():
                continue
            message = reader.deserialize(rawdata, connection.msgtype)
            observations = bridge.on_message(
                connection.topic, message, timestamp_ns=timestamp
            )
            counts[connection.topic] += len(observations)

    stats = bridge.get_stats()
    errors = sum(item["errors"] for item in stats.values())

    assert sum(counts.values()) > 0
    assert errors == 0, stats
    assert any(count > 0 for count in counts.values())
