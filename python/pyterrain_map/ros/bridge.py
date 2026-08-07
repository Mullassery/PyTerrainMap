"""
ROS/ROS2 bridge for PyTerrainMap.

Wires configured sensor topics to their adapters, maintains a TF cache, and
converts local robot poses to geodetic coordinates via `CoordinateConverter`.

This module deliberately never imports rclpy/rospy itself — ROS message
objects are accepted as opaque, duck-typed values (the same convention the
adapters already use), and actual topic subscription is handled by the
caller through `attach_to_node()`. That keeps `import pyterrain_map.ros` (and
this bridge's dispatch logic) fully usable and unit-testable without a ROS
installation, consistent with this project's policy of not hard-depending
on `rospy`/`rclpy` (see pyproject.toml comments on excluded dependencies).
"""

import logging
from typing import Any, Callable, Dict, List, Optional

from .adapters import ADAPTER_CLASSES
from .adapters.base import AdapterRegistry, StorageObservation
from .transforms.coordinate_frames import CoordinateConverter
from .transforms.tf_listener import TFListener

logger = logging.getLogger(__name__)


class PyTerrainROSBridge:
    """Coordinates sensor adapters, TF lookups, and coordinate conversion
    for a single robot, per a platform config (see `platforms/__init__.py`).
    """

    def __init__(self, platform_config: Dict[str, Any]):
        self.robot_id = platform_config["robot_id"]
        self.base_frame = platform_config.get("base_frame", "base_link")
        self.reference_frame = platform_config.get("reference_frame", "map")

        self.registry = AdapterRegistry()
        self.tf_listener = TFListener()
        self.converter: Optional[CoordinateConverter] = None

        self._topic_frame_ids: Dict[str, str] = {}
        self._build_adapters(platform_config.get("sensors", {}))

    def _build_adapters(self, sensors_config: Dict[str, Any]):
        for sensor_name, sensor_cfg in sensors_config.items():
            if not sensor_cfg.get("enabled", True):
                continue

            adapter_key = sensor_cfg.get("adapter")
            adapter_cls = ADAPTER_CLASSES.get(adapter_key)
            if adapter_cls is None:
                logger.warning(
                    "Sensor '%s' requests unknown adapter '%s' — skipping "
                    "(its topic(s) will produce no observations)",
                    sensor_name, adapter_key,
                )
                continue

            topics = sensor_cfg.get("topics") or [sensor_cfg.get("topic")]
            frame_ids = sensor_cfg.get("frame_ids") or [sensor_cfg.get("frame_id")]
            if len(frame_ids) < len(topics):
                frame_ids = list(frame_ids) + [frame_ids[-1]] * (len(topics) - len(frame_ids))

            params = sensor_cfg.get("params", {})
            for topic, frame_id in zip(topics, frame_ids):
                if not topic:
                    continue
                try:
                    adapter = adapter_cls(robot_id=self.robot_id, frame_id=frame_id, **params)
                except ImportError as e:
                    # e.g. RGBAdapter requested without opencv installed —
                    # skip only this topic, not the whole bridge.
                    logger.warning("Could not build adapter for topic '%s': %s", topic, e)
                    continue
                self.registry.register(topic, adapter)
                self._topic_frame_ids[topic] = frame_id

    def set_origin(self, lat: float, lon: float, alt: float = 0.0):
        """Set the geodetic origin used to convert local ENU poses to lat/lon."""
        self.converter = CoordinateConverter(lat, lon, alt)

    def on_tf_message(self, transforms_list):
        """Feed a /tf or /tf_static message's transforms into the TF cache."""
        self.tf_listener.on_tf_message(transforms_list)

    def on_message(
        self,
        topic: str,
        msg,
        timestamp_ns: Optional[int] = None,
    ) -> List[StorageObservation]:
        """
        Route a message on `topic` to its registered adapter.

        Returns the adapter's observations, or an empty list if no adapter
        is registered for this topic. Unlike the previous behavior (no
        registry at all — every message silently produced nothing), an
        unmapped topic now logs a warning instead of failing silently.
        """
        adapter = self.registry.get(topic)
        if adapter is None:
            logger.warning("No adapter registered for topic '%s' — message dropped", topic)
            return []

        robot_pose = None
        frame_id = self._topic_frame_ids.get(topic)
        if frame_id and timestamp_ns is not None:
            tf = self.tf_listener.lookup_transform(self.reference_frame, frame_id, timestamp_ns)
            if tf is not None:
                robot_pose = (tf.x, tf.y, tf.z, tf.qx, tf.qy, tf.qz, tf.qw)

        return adapter.on_message(msg, robot_pose=robot_pose, converter=self.converter)

    def get_stats(self) -> Dict[str, Dict[str, Any]]:
        """Per-topic adapter statistics (messages processed, errors)."""
        return {topic: adapter.get_stats() for topic, adapter in self.registry.get_all().items()}

    def attach_to_node(self, subscribe: Callable[[str, Callable[[Any], None]], None]):
        """
        Wire every configured topic (plus /tf and /tf_static) to this
        bridge's dispatch methods, using a caller-provided `subscribe`
        function.

        This bridge does not know or guess ROS message types (LaserScan vs
        PointCloud2 for LiDAR, sensor_msgs/Image for camera/thermal, etc.)
        — the caller's node already does, and is better positioned to set
        the right type and QoS per topic. Typical usage in a real rclpy
        node:

            MSG_TYPES = {"/scan": LaserScan, "/camera/rgb": Image, ...}
            bridge.attach_to_node(
                lambda topic, cb: node.create_subscription(MSG_TYPES[topic], topic, cb, 10)
            )

        Not covered by this package's unit tests (there is no live ROS
        runtime in this environment) — verify against a real rclpy/rospy
        node before relying on it in production.
        """
        subscribe("/tf", lambda msg: self.on_tf_message(self._extract_transforms(msg)))
        subscribe("/tf_static", lambda msg: self.on_tf_message(self._extract_transforms(msg)))
        for topic in self.registry.get_all():
            subscribe(topic, lambda msg, t=topic: self.on_message(t, msg))

    @staticmethod
    def _extract_transforms(tf_message):
        """Accept either a tf2_msgs/TFMessage-like object (`.transforms`) or
        a plain list of stamped transforms."""
        return getattr(tf_message, "transforms", tf_message)
