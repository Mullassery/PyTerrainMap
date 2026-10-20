"""GPS adapter for PyTerrainMap ROS bridge."""

import json
import math
from typing import List, Optional, Tuple

from ..transforms.coordinate_frames import CoordinateConverter
from .base import SensorAdapter, StorageObservation


def _covariance(value) -> list:
    if value is None:
        return []
    try:
        return [float(item) for item in value]
    except (TypeError, ValueError):
        return []


class GPSAdapter(SensorAdapter):
    """Convert ``sensor_msgs/NavSatFix`` messages into geodetic observations."""

    @property
    def sensor_type(self) -> str:
        return "gps"

    def on_message(
        self,
        msg,
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        try:
            self.message_count += 1
            status = getattr(msg, "status", None)
            status_code = int(getattr(status, "status", 0))
            latitude = float(getattr(msg, "latitude"))
            longitude = float(getattr(msg, "longitude"))
            altitude = float(getattr(msg, "altitude", 0.0))

            if status_code < 0:
                raise ValueError(f"GPS has no fix (status={status_code})")
            if not math.isfinite(latitude) or not math.isfinite(longitude):
                raise ValueError("GPS latitude/longitude contains non-finite values")

            value = {
                "frame_id": self._header_frame(msg),
                "latitude": latitude,
                "longitude": longitude,
                "altitude": altitude if math.isfinite(altitude) else 0.0,
                "status": {
                    "status": status_code,
                    "service": int(getattr(status, "service", 0)),
                },
                "position_covariance": _covariance(
                    getattr(msg, "position_covariance", None)
                ),
                "position_covariance_type": int(
                    getattr(msg, "position_covariance_type", 0)
                ),
                "location_source": "navsatfix",
            }

            return [
                StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=self._ros_time_to_us(msg.header.stamp),
                    location_lat=latitude,
                    location_lon=longitude,
                    sensor_type=self.sensor_type,
                    value_json=json.dumps(value),
                    confidence=1.0,
                )
            ]
        except Exception as exc:
            print(f"GPS adapter error: {exc}")
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
