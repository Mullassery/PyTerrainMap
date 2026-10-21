"""IMU adapter for PyTerrainMap ROS bridge."""

import json
import math
from typing import List, Optional, Tuple

from ..transforms.coordinate_frames import CoordinateConverter, ENUPoint
from .base import SensorAdapter, StorageObservation


def _vector3(value, *, default=(0.0, 0.0, 0.0)) -> dict:
    if value is None:
        x, y, z = default
    else:
        x = float(getattr(value, "x", default[0]))
        y = float(getattr(value, "y", default[1]))
        z = float(getattr(value, "z", default[2]))
    return {"x": x, "y": y, "z": z}


def _quaternion(value) -> dict:
    if value is None:
        return {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0}
    return {
        "x": float(getattr(value, "x", 0.0)),
        "y": float(getattr(value, "y", 0.0)),
        "z": float(getattr(value, "z", 0.0)),
        "w": float(getattr(value, "w", 1.0)),
    }


def _covariance(value) -> list:
    if value is None:
        return []
    try:
        return [float(item) for item in value]
    except (TypeError, ValueError):
        return []


def _location(
    robot_pose: Optional[Tuple[float, float, float, float, float, float, float]],
    converter: Optional[CoordinateConverter],
) -> Tuple[float, float, str]:
    if robot_pose is None:
        return 0.0, 0.0, "unavailable"
    x, y, z = float(robot_pose[0]), float(robot_pose[1]), float(robot_pose[2])
    if converter is not None:
        geo = converter.enu_to_geodetic(ENUPoint(east=x, north=y, up=z))
        return geo.lat, geo.lon, "geodetic"
    return x, y, "local"


class IMUAdapter(SensorAdapter):
    """Convert ``sensor_msgs/Imu`` messages into orientation/inertial observations."""

    @property
    def sensor_type(self) -> str:
        return "imu"

    def on_message(
        self,
        msg,
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        try:
            self.message_count += 1
            orientation = getattr(msg, "orientation", None)
            linear_acceleration = getattr(msg, "linear_acceleration", None)
            if orientation is None or linear_acceleration is None:
                raise ValueError("IMU message is missing orientation or linear_acceleration")

            quat = _quaternion(orientation)
            if not all(math.isfinite(quat[axis]) for axis in ("x", "y", "z", "w")):
                raise ValueError("IMU quaternion contains non-finite values")

            lat, lon, location_source = _location(robot_pose, converter)
            value = {
                "frame_id": self._header_frame(msg),
                "orientation": quat,
                "angular_velocity": _vector3(getattr(msg, "angular_velocity", None)),
                "linear_acceleration": _vector3(linear_acceleration),
                "orientation_covariance": _covariance(getattr(msg, "orientation_covariance", None)),
                "angular_velocity_covariance": _covariance(
                    getattr(msg, "angular_velocity_covariance", None)
                ),
                "linear_acceleration_covariance": _covariance(
                    getattr(msg, "linear_acceleration_covariance", None)
                ),
                "location_source": location_source,
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
            print(f"IMU adapter error: {exc}")
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
