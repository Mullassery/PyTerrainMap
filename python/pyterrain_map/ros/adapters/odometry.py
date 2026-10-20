"""Odometry adapter for PyTerrainMap ROS bridge."""

import json
from typing import List, Optional, Tuple

from ..transforms.coordinate_frames import CoordinateConverter, ENUPoint
from .base import SensorAdapter, StorageObservation


class OdometryAdapter(SensorAdapter):
    """Convert ``nav_msgs/Odometry`` messages into pose observations."""

    @property
    def sensor_type(self) -> str:
        return "odometry"

    def on_message(
        self,
        msg,
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        try:
            self.message_count += 1
            pose = getattr(msg.pose, "pose", msg.pose)
            position = pose.position
            orientation = pose.orientation
            twist = getattr(msg.twist, "twist", getattr(msg, "twist", None))
            linear = getattr(twist, "linear", None)
            angular = getattr(twist, "angular", None)

            x = float(position.x)
            y = float(position.y)
            z = float(position.z)
            if converter:
                geo = converter.enu_to_geodetic(ENUPoint(east=x, north=y, up=z))
                lat, lon = geo.lat, geo.lon
            else:
                lat, lon = x, y

            value = {
                "frame_id": self._header_frame(msg),
                "child_frame_id": str(getattr(msg, "child_frame_id", self.frame_id)),
                "x": x,
                "y": y,
                "z": z,
                "qx": float(orientation.x),
                "qy": float(orientation.y),
                "qz": float(orientation.z),
                "qw": float(orientation.w),
            }
            if linear is not None:
                value["linear"] = {
                    "x": float(linear.x),
                    "y": float(linear.y),
                    "z": float(linear.z),
                }
            if angular is not None:
                value["angular"] = {
                    "x": float(angular.x),
                    "y": float(angular.y),
                    "z": float(angular.z),
                }

            return [
                StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=self._ros_time_to_us(msg.header.stamp),
                    location_lat=lat,
                    location_lon=lon,
                    sensor_type=self.sensor_type,
                    value_json=json.dumps(value),
                    confidence=1.0,
                )
            ]
        except Exception as exc:
            print(f"Odometry adapter error: {exc}")
            self.error_count += 1
            return []

    @staticmethod
    def _header_frame(msg) -> str:
        header = getattr(msg, "header", None)
        return str(getattr(header, "frame_id", ""))

    @staticmethod
    def _ros_time_to_us(stamp) -> int:
        secs = getattr(stamp, "secs", getattr(stamp, "sec", 0))
        nsecs = getattr(stamp, "nsecs", getattr(stamp, "nanosec", 0))
        return secs * 1_000_000 + nsecs // 1000

    @staticmethod
    def _generate_id() -> str:
        import uuid

        return str(uuid.uuid4())
